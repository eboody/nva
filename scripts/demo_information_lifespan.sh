#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

compose=(docker compose)
api_host="${PET_RESORT_API_HOST:-127.0.0.1}"
api_port="${PET_RESORT_API_HOST_PORT:-3001}"
staff_web_host="${PET_RESORT_STAFF_WEB_HOST:-127.0.0.1}"
staff_web_port="${PET_RESORT_STAFF_WEB_HOST_PORT:-3000}"
api_url="http://${api_host}:${api_port}"
staff_web_url="http://${staff_web_host}:${staff_web_port}"
processor_output_dir="${INFORMATION_LIFESPAN_OUTPUT_DIR:-$repo_root/.var/information-lifespan}"
artifact_dir="${INFORMATION_LIFESPAN_ARTIFACT_DIR:-$repo_root/.artifacts/information-lifespan}"
transcript_path="$artifact_dir/demo-information-lifespan-transcript.log"

# The processor output directory may have been created by Docker as root on an earlier run.
# It only needs to be readable by this script; script-owned API/transcript artifacts go elsewhere.
mkdir -p "$processor_output_dir" 2>/dev/null || true
mkdir -p "$artifact_dir"
exec > >(tee "$transcript_path") 2>&1

fail() {
  echo "[information-lifespan] failure: $*" >&2
  dump_diagnostics || true
  exit 1
}

need_command() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

dump_diagnostics() {
  echo "[information-lifespan] compose ps"
  "${compose[@]}" ps || true
  echo "[information-lifespan] recent core service logs"
  "${compose[@]}" logs --tail=120 postgres minio migrate-seed hermes-processor pet-resort-api pet-resort-worker staff-web || true
}

run() {
  echo "+ $*"
  "$@" || fail "command failed: $*"
}

wait_for_url() {
  local url="$1"
  local label="$2"
  local attempts="${3:-120}"
  local response_path="$artifact_dir/${label//[^A-Za-z0-9_.-]/_}.json"

  echo "[information-lifespan] waiting for ${label}: ${url}"
  for _ in $(seq 1 "$attempts"); do
    if curl -fsS "$url" >"$response_path" 2>"$artifact_dir/${label//[^A-Za-z0-9_.-]/_}.curl.err"; then
      echo "[information-lifespan] ${label}_reachable=true response=${response_path}"
      return 0
    fi
    sleep 1
  done
  fail "${label} did not become reachable at ${url}; curl stderr saved under ${artifact_dir}"
}

need_command docker
need_command curl
need_command python

if ! docker compose version >/dev/null 2>&1; then
  fail "docker compose plugin is required"
fi

cat <<'BANNER'
[information-lifespan] Safe local full-stack demo
  Source evidence: synthetic/mock Gingr fixture only
  Live side effects: disabled (customer sends, provider/PMS writes, payments/refunds/discounts, schedule changes, medical/safety decisions)
BANNER

echo "[information-lifespan] transcript=${transcript_path}"

run "${compose[@]}" build hermes-processor
run "${compose[@]}" run --rm hermes-processor

processor_output_path="$processor_output_dir/processor-output.json"
processor_log_path="$processor_output_dir/processor-log.jsonl"
processor_schema_path="$repo_root/schemas/information-lifespan-hermes-processor-output.schema.json"

echo "[information-lifespan] validating processor output contract"
python - "$processor_output_path" "$processor_log_path" "$processor_schema_path" <<'PY'
import json
import re
import sys
from pathlib import Path

output_path = Path(sys.argv[1])
log_path = Path(sys.argv[2])
schema_path = Path(sys.argv[3])

for path in (output_path, log_path, schema_path):
    if not path.exists():
        raise SystemExit(f"required artifact missing: {path}")

output = json.loads(output_path.read_text(encoding="utf-8"))
schema = json.loads(schema_path.read_text(encoding="utf-8"))
logs = log_path.read_text(encoding="utf-8")

assert output["contract_version"] == "information_lifespan_hermes_processor_output.v1"
assert output["correlation_id"] == "info-lifespan-demo-2026-06-29"
assert output["synthetic_data_only"] is True
assert output["provider_payloads_are_source_evidence_only"] is True
assert output["live_side_effects_allowed"] is False
assert output["processor"]["service"] == "hermes-processor"
assert output["processor"]["runtime_status"] in {
    "hermes_cli_available",
    "hermes_cli_unavailable_explicit_fallback",
}
assert output["calculations"]["reported_estimated_labor_minutes_difference"] == 42
assert output["final_report"]["artifact_ref"] == "artifact://manager-daily-report/synthetic-2026-06-29"
assert len(output["final_report"]["manager_actions"]) >= 2
assert all(gate["locked"] is True for gate in output["review_gates"])
assert schema["properties"]["calculations"]["properties"]["reported_estimated_labor_minutes_difference"]["const"] == 42
assert "manager_daily_report_enriched" in logs
assert "unsafe_side_effects_locked" in logs

for name, text in {
    "processor output": json.dumps(output, sort_keys=True),
    "processor logs": logs,
}.items():
    forbidden = [
        r"sk-[A-Za-z0-9_-]{12,}",
        r"(?i)(api[_-]?key|secret|token|password)\s*[:=]\s*[^\s,}]+",
        r"(?i)live_side_effects_allowed\s*[:=]\s*true",
    ]
    for pattern in forbidden:
        if re.search(pattern, text):
            raise AssertionError(f"forbidden secret/live claim in {name}: {pattern}")

print(
    "processor_contract_ok "
    f"correlation_id={output['correlation_id']} "
    f"runtime_status={output['processor']['runtime_status']} "
    f"reported_estimated_labor_minutes_difference={output['calculations']['reported_estimated_labor_minutes_difference']}"
)
PY

echo "[information-lifespan] starting DB/API/worker/staff-web stack"
run "${compose[@]}" up --build -d --wait pet-resort-api pet-resort-worker staff-web

wait_for_url "${api_url}/v0/healthz" "api-health"
wait_for_url "${api_url}/v0/readyz" "api-readiness"
wait_for_url "${staff_web_url}/" "staff-web"

echo "[information-lifespan] triggering API information-lifespan run"
run curl -fsS -X POST "${api_url}/v0/demo/information-lifespan/run" -o "$artifact_dir/api-information-lifespan-run.json"

correlation_id="$(python - "$artifact_dir/api-information-lifespan-run.json" <<'PY'
import json
import sys
from pathlib import Path
payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
assert payload["correlation_id"] == "info-lifespan-demo-2026-06-29"
assert payload["safety"]["synthetic_data_only"] is True
assert payload["safety"]["live_side_effects_allowed"] is False
processor = payload["processor_proof"]
assert processor["contract_version"] == "information_lifespan_hermes_processor_output.v1", processor
assert processor.get("simulated") is not True, processor
assert processor["processor"]["service"] == "hermes-processor"
assert payload["final_report"]["artifact_ref"] == "artifact://manager-daily-report/synthetic-2026-06-29"
assert len(payload["network_proof"]) >= 1
assert len(payload["log_proof"]) >= 1
print(payload["correlation_id"])
PY
)"
echo "[information-lifespan] api_run_ok correlation_id=${correlation_id} artifact=$artifact_dir/api-information-lifespan-run.json"

run curl -fsS "${api_url}/v0/demo/information-lifespan/${correlation_id}/report" -o "$artifact_dir/api-information-lifespan-report.json"
python - "$artifact_dir/api-information-lifespan-report.json" <<'PY'
import json
import sys
from pathlib import Path
payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
assert payload["api_contract"]["workflow"] == "information_lifespan_demo_report"
assert payload["correlation_id"] == "info-lifespan-demo-2026-06-29"
assert payload["processor_proof"]["contract_version"] == "information_lifespan_hermes_processor_output.v1"
assert payload["safety"]["live_side_effects_allowed"] is False
print("api_report_replay_ok final_report=" + payload["final_report"]["artifact_ref"])
PY

echo "[information-lifespan] running core compose smoke"
run ./scripts/smoke_local_demo.sh

cat <<SUMMARY
[information-lifespan] proof artifacts
  transcript=${transcript_path}
  processor_output=${processor_output_path}
  processor_logs=${processor_log_path}
  api_run=${artifact_dir}/api-information-lifespan-run.json
  api_report=${artifact_dir}/api-information-lifespan-report.json
  staff_web=${staff_web_url}/
information_lifespan_demo_ok local_fixture_only=true dockerized_hermes_processor=true live_side_effects_allowed=false
SUMMARY
