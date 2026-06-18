#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

LOCATION_ID="${LOCATION_ID:-00c0ffee-0000-0000-0000-000000000001}"
OPERATING_DAY="${OPERATING_DAY:-2026-06-17}"
PET_RESORT_API_HOST_PORT="${PET_RESORT_API_HOST_PORT:-3001}"
PET_RESORT_API_URL="${PET_RESORT_API_URL:-http://127.0.0.1:${PET_RESORT_API_HOST_PORT}}"
OPENVIKING_HEALTH_URL="${OPENVIKING_HEALTH_URL:-http://127.0.0.1:${PET_RESORT_OPENVIKING_HOST_PORT:-1933}/health}"
SMOKE_TMP_DIR="${SMOKE_TMP_DIR:-$(mktemp -d)}"
export PET_RESORT_API_URL

log() {
  printf '[manager-daily-brief-smoke] %s\n' "$*" >&2
}

require() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'missing required command: %s\n' "$1" >&2
    exit 127
  fi
}

wait_for_http() {
  local url="$1"
  local label="$2"
  local attempts="${3:-60}"
  local delay="${4:-2}"
  local i
  for ((i = 1; i <= attempts; i++)); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      log "$label is reachable at $url"
      return 0
    fi
    sleep "$delay"
  done
  printf '%s did not become reachable at %s after %s attempts\n' "$label" "$url" "$attempts" >&2
  docker compose --profile agent-infra ps >&2 || true
  docker compose --profile agent-infra logs --no-color --tail=80 pet-resort-api openviking >&2 || true
  exit 1
}

check_openviking_or_document_blocker() {
  local url="$1"
  if OPENVIKING_HEALTH_URL="$url" scripts/preflight_openviking_agent_infra.sh \
    --allow-uninitialized \
    --status-file "${SMOKE_TMP_DIR}/openviking-status.txt"; then
    if grep -q '^openviking_status=healthy$' "${SMOKE_TMP_DIR}/openviking-status.txt"; then
      log "openviking is reachable at $url"
    else
      log "OpenViking agent-infra preflight documented an initialization blocker; continuing app-owned smoke loop"
      docker compose --profile agent-infra logs --no-color --tail=40 openviking >"${SMOKE_TMP_DIR}/openviking-blocker.log" 2>&1 || true
    fi
    return 0
  fi

  log "OpenViking config exists but health failed; documenting logs and continuing app-owned smoke loop"
  docker compose --profile agent-infra logs --no-color --tail=80 openviking >"${SMOKE_TMP_DIR}/openviking-blocker.log" 2>&1 || true
  return 0
}

require docker
require curl
require python

log "starting local postgres, minio, app API/worker, and OpenViking via docker compose"
docker compose --profile agent-infra up --build -d postgres minio pet-resort-api pet-resort-worker openviking

wait_for_http "${PET_RESORT_API_URL}/healthz" "pet-resort-api"
check_openviking_or_document_blocker "${OPENVIKING_HEALTH_URL}"

log "reading app-owned Manager Daily Brief context through Hermes/tool bridge"
scripts/hermes-tools/get_manager_daily_brief_context \
  --location-id "$LOCATION_ID" \
  --operating-day "$OPERATING_DAY" \
  >"${SMOKE_TMP_DIR}/context.json"

python - "${SMOKE_TMP_DIR}/context.json" <<'PY'
import json, sys
payload = json.load(open(sys.argv[1], encoding="utf-8"))
assert payload["workflow"]["name"] == "manager_daily_brief"
assert payload["manager_brief_actions"], "expected source-grounded manager brief actions"
assert payload["source_refs"], "expected app-owned source refs"
assert "send_customer_message" in payload["blocked_actions"]
assert payload["labor_impact"]["minutes_saved"] > 0
print(
    "context_ok actions={actions} minutes_saved={minutes}".format(
        actions=len(payload["manager_brief_actions"]),
        minutes=payload["labor_impact"]["minutes_saved"],
    )
)
PY

log "building a Hermes-authored draft that cites app source refs and requests no side effects"
python - "${SMOKE_TMP_DIR}/context.json" "${SMOKE_TMP_DIR}/draft.json" <<'PY'
import json, sys
context_path, draft_path = sys.argv[1:]
context = json.load(open(context_path, encoding="utf-8"))
source_refs = context["manager_brief_actions"][0].get("source_refs") or context["source_refs"][:1]
draft = {
    "context_packet_id": context["audit"]["context_packet_id"],
    "correlation_id": context["audit"]["correlation_id"],
    "submitted_by": "hermes-agent-local-smoke",
    "actions": [
        {
            "id": "smoke-demand-staffing-1",
            "kind": "review_demand_against_staffing_plan",
            "recommendation": "Review boarding demand against the staffing plan before morning drop-off; keep this as an internal manager review action only.",
            "source_refs": source_refs,
            "review_gates": ["manager_approval"],
            "requested_side_effects": [],
        }
    ],
}
json.dump(draft, open(draft_path, "w", encoding="utf-8"), indent=2)
PY

scripts/hermes-tools/submit_manager_daily_brief_draft \
  --draft-file "${SMOKE_TMP_DIR}/draft.json" \
  >"${SMOKE_TMP_DIR}/accepted-draft-response.json"

python - "${SMOKE_TMP_DIR}/accepted-draft-response.json" <<'PY'
import json, sys
payload = json.load(open(sys.argv[1], encoding="utf-8"))
assert payload["validation"]["status"] == "accepted"
assert len(payload["accepted_actions"]) == 1
assert payload["live_side_effects_allowed"] is False
print("draft_validation_ok accepted_actions=1 live_side_effects_allowed=false")
PY

log "proving app validation rejects an unsafe agent side-effect request"
python - "${SMOKE_TMP_DIR}/draft.json" "${SMOKE_TMP_DIR}/blocked-draft.json" <<'PY'
import json, sys
src, dst = sys.argv[1:]
draft = json.load(open(src, encoding="utf-8"))
draft["actions"][0]["id"] = "smoke-blocked-side-effect-1"
draft["actions"][0]["requested_side_effects"] = ["change_staff_schedule"]
json.dump(draft, open(dst, "w", encoding="utf-8"), indent=2)
PY

set +e
scripts/hermes-tools/submit_manager_daily_brief_draft \
  --draft-file "${SMOKE_TMP_DIR}/blocked-draft.json" \
  >"${SMOKE_TMP_DIR}/blocked-draft-response.json" 2>"${SMOKE_TMP_DIR}/blocked-draft-stderr.txt"
blocked_status=$?
set -e
if [[ "$blocked_status" -eq 0 ]]; then
  printf 'expected blocked draft to be rejected, but bridge command succeeded\n' >&2
  exit 1
fi
python - "${SMOKE_TMP_DIR}/blocked-draft-stderr.txt" <<'PY'
import sys
text = open(sys.argv[1], encoding="utf-8").read()
assert "HTTP 422" in text, text
assert "http://" not in text and "Authorization" not in text, text
print("blocked_draft_validation_ok http_422_secret_safe_error=true")
PY

log "recording reviewed staff outcome and actual labor minutes through the app"
python - "${SMOKE_TMP_DIR}/context.json" "${SMOKE_TMP_DIR}/outcome.json" "${SMOKE_TMP_DIR}/action-id.txt" <<'PY'
import json, sys
context_path, outcome_path, action_id_path = sys.argv[1:]
context = json.load(open(context_path, encoding="utf-8"))
action = next(action for action in context["manager_brief_actions"] if action["kind"] == "resolve_checkout_exception")
raw_source_refs = action.get("source_refs") or context["source_refs"][:1]
source_refs = []
for ref in raw_source_refs:
    normalized = dict(ref)
    normalized.setdefault("record_type", "reservation")
    normalized.setdefault("observed_at", "2026-06-17T12:00:00Z")
    normalized.setdefault("adapter_version", "local-manager-daily-brief-smoke-v1")
    source_refs.append(normalized)
outcome = {
    "outcome": "completed",
    "actual_minutes": 12,
    "actor": {"id": "front-desk-lead-local-smoke", "persona": "front_desk_lead"},
    "feedback": "Fake reviewed outcome: checkout exception was resolved before the rush; no customer/PMS/provider/payment side effect was attempted.",
    "source_refs": source_refs,
    "timestamp": "2026-06-17T13:15:00Z",
    "audit": {"correlation_id": context["audit"]["correlation_id"]},
    "reporting": {"location_id": context["location_id"], "operating_day": context["operating_day"]},
    "requested_side_effects": [],
}
json.dump(outcome, open(outcome_path, "w", encoding="utf-8"), indent=2)
open(action_id_path, "w", encoding="utf-8").write(action["id"])
PY

ACTION_ID="$(cat "${SMOKE_TMP_DIR}/action-id.txt")"
scripts/hermes-tools/record_manager_daily_brief_outcome \
  --action-id "$ACTION_ID" \
  --outcome-file "${SMOKE_TMP_DIR}/outcome.json" \
  >"${SMOKE_TMP_DIR}/outcome-response.json"

python - "${SMOKE_TMP_DIR}/outcome-response.json" <<'PY'
import json, sys
payload = json.load(open(sys.argv[1], encoding="utf-8"))
assert payload["accepted"] is True
assert payload["outcome_persisted"] is True
assert payload["live_side_effects_allowed"] is False
assert payload["labor_savings_evidence"]["estimated_minutes_saved"] > 0
assert payload["labor_savings_evidence"]["actual_minutes_saved"] > 0
print(
    "outcome_ok estimated_minutes_saved={estimated} actual_minutes_saved={actual}".format(
        estimated=payload["labor_savings_evidence"]["estimated_minutes_saved"],
        actual=payload["labor_savings_evidence"]["actual_minutes_saved"],
    )
)
PY

log "smoke artifacts written under ${SMOKE_TMP_DIR}"
if [[ -f "${SMOKE_TMP_DIR}/openviking-status.txt" ]]; then
  log "$(tr '\n' ';' <"${SMOKE_TMP_DIR}/openviking-status.txt")"
fi
log "full local Manager Daily Brief loop passed without live customer/PMS/payment side effects"
