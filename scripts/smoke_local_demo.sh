#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

compose=(docker compose)
api_host="${PET_RESORT_API_HOST:-127.0.0.1}"
api_port="${PET_RESORT_API_HOST_PORT:-3001}"
staff_web_host="${PET_RESORT_STAFF_WEB_HOST:-127.0.0.1}"
staff_web_port="${PET_RESORT_STAFF_WEB_HOST_PORT:-3000}"
api_url="http://${api_host}:${api_port}"
staff_web_url="http://${staff_web_host}:${staff_web_port}"
require_optional_agent_infra="${REQUIRE_OPTIONAL_AGENT_INFRA:-0}"

fail() {
  echo "[smoke-local-demo] core demo failure: $*" >&2
  exit 1
}

optional_warn() {
  echo "[smoke-local-demo] optional agent-infra warning: $*" >&2
}

need_command() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

need_command docker
need_command curl
need_command python

if ! docker compose version >/dev/null 2>&1; then
  fail "docker compose plugin is required"
fi

wait_for_url() {
  local url="$1"
  local label="$2"
  local attempts="${3:-120}"
  for _ in $(seq 1 "$attempts"); do
    if curl -fsS "$url" >/tmp/pet-resort-local-demo-response.json 2>/dev/null; then
      return 0
    fi
    sleep 1
  done
  fail "${label} did not become reachable at ${url}"
}

query_db() {
  "${compose[@]}" exec -T postgres psql -U pet_resort -d pet_resort -Atc "$1"
}

assert_json_file() {
  local path="$1"
  local check_name="$2"
  python - "$path" "$check_name" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
check_name = sys.argv[2]
payload = json.loads(path.read_text())
checks = {
    "api_health_side_effect_posture": lambda data: data.get("status") == "ok"
    and data.get("live_side_effects") == "disabled",
    "api_readiness_safe_runtime": lambda data: data.get("agent_runtime") == "fake_deterministic"
    and data.get("live_customer_messaging") == "disabled"
    and data.get("live_provider_writes") == "disabled",
    "api_source_quality_backlog_read_model": lambda data: data.get("database", {}).get("status") == "connected"
    and data.get("records", []) != []
    and data.get("data_posture", {}).get("live_side_effects_allowed") is False
    and data.get("data_posture", {}).get("provider_writes_allowed") is False,
}
try:
    passed = checks[check_name](payload)
except KeyError as exc:
    raise SystemExit(f"unknown JSON smoke check: {check_name}") from exc
if not passed:
    raise SystemExit(f"{check_name} failed for payload: {json.dumps(payload, sort_keys=True)}")
print(f"[smoke-local-demo] {check_name}=ok")
PY
}

echo "[smoke-local-demo] checking core compose services"
"${compose[@]}" ps postgres minio migrate-seed pet-resort-api pet-resort-worker staff-web >/tmp/pet-resort-compose-core-ps.txt || fail "core compose services are not known to this project"

wait_for_url "${api_url}/v0/healthz" "pet-resort-api health"
curl -fsS "${api_url}/v0/healthz" >/tmp/pet-resort-api-health.json
assert_json_file /tmp/pet-resort-api-health.json "api_health_side_effect_posture"

curl -fsS "${api_url}/v0/readyz" >/tmp/pet-resort-api-ready.json
assert_json_file /tmp/pet-resort-api-ready.json "api_readiness_safe_runtime"

curl -fsS "${api_url}/v0/read-models/source-quality-backlog" >/tmp/pet-resort-source-quality-backlog.json
assert_json_file /tmp/pet-resort-source-quality-backlog.json "api_source_quality_backlog_read_model"

wait_for_url "${staff_web_url}/" "staff-web"
curl -fsS "${staff_web_url}/" >/tmp/pet-resort-staff-web.html
if ! grep -qi "MVP Staff Demo\|NVA\|staff" /tmp/pet-resort-staff-web.html; then
  fail "staff-web responded but did not look like the staff dashboard"
fi
echo "[smoke-local-demo] staff_web_reachable=ok"

echo "[smoke-local-demo] checking seeded Postgres read models"
core_counts="$(query_db "SELECT 'locations=' || COUNT(*) FROM locations UNION ALL SELECT 'source_quality_issues=' || COUNT(*) FROM source_quality_issues UNION ALL SELECT 'source_quality_backlog=' || COUNT(*) FROM source_quality_backlog UNION ALL SELECT 'data_quality_hygiene_labor_outcomes=' || COUNT(*) FROM data_quality_hygiene_labor_outcomes UNION ALL SELECT 'outbox_records=' || COUNT(*) FROM outbox_records ORDER BY 1;")"
printf '%s\n' "$core_counts"
python - "$core_counts" <<'PY'
import sys
pairs = {}
for line in sys.argv[1].splitlines():
    key, value = line.split('=', 1)
    pairs[key] = int(value)
required = [
    'locations',
    'source_quality_issues',
    'source_quality_backlog',
    'data_quality_hygiene_labor_outcomes',
    'outbox_records',
]
missing = [key for key in required if pairs.get(key, 0) < 1]
if missing:
    raise SystemExit(f"missing seeded/read-model rows: {missing}; counts={pairs}")
print("[smoke-local-demo] db_seeded_read_models=ok")
PY

side_effect_posture="$(query_db "SELECT COALESCE(bool_and(payload->>'live_delivery_allowed' = 'false'), false) FROM outbox_records WHERE topic = 'internal.data_quality_hygiene.reviewed_handoff';")"
if [[ "${side_effect_posture}" != "t" ]]; then
  fail "outbox_records do not preserve live_delivery_allowed=false"
fi
echo "[smoke-local-demo] db_side_effect_posture=ok"

if "${compose[@]}" ps --services --filter status=running | grep -qx openviking; then
  openviking_health="$("${compose[@]}" ps --format json openviking 2>/dev/null | python -c 'import json,sys; raw=sys.stdin.read().strip(); print((json.loads(raw).get("Health") if raw else "unknown") or "unknown")' || true)"
  case "${openviking_health}" in
    healthy|starting|unknown)
      echo "[smoke-local-demo] optional_agent_infra_openviking=${openviking_health}"
      ;;
    *)
      if [[ "${require_optional_agent_infra}" == "1" ]]; then
        fail "optional openviking profile is running but unhealthy (${openviking_health}) and REQUIRE_OPTIONAL_AGENT_INFRA=1"
      fi
      optional_warn "openviking is ${openviking_health}; core demo remains smoke-passed because agent-infra is optional/profiled"
      ;;
  esac
else
  echo "[smoke-local-demo] optional_agent_infra_openviking=not_running_profile_optional"
fi

echo "[smoke-local-demo] core_demo_ok local_fixture_only=true live_side_effects_allowed=false optional_agent_infra_required=${require_optional_agent_infra}"
