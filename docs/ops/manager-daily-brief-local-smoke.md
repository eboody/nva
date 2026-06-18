# Manager Daily Brief local smoke loop

Purpose: prove the first local agent/app loop for the NVA Pet Resorts labor-cost reduction platform without live customer, PMS/provider, schedule, payment, refund, or discount side effects.

The smoke keeps the deterministic app in charge of facts, source refs, validation, review gates, persistence, audit, and outcome reporting. Hermes/tool scripts only read typed context and submit draft/outcome packets through app-owned endpoints.

## Prerequisites

- Docker with Compose support.
- Rust/Cargo available if images need to build.
- Python 3 and curl.
- Ports available, or override host ports with environment variables.

Default host ports:

- pet-resort-api: `127.0.0.1:3001`
- postgres: `127.0.0.1:54329`
- minio: `127.0.0.1:9000` and console `127.0.0.1:9001`
- OpenViking: `127.0.0.1:1933`

If another local stack already owns a default port, override only the colliding port, for example:

```bash
PET_RESORT_POSTGRES_HOST_PORT=55432 ./scripts/smoke_manager_daily_brief_local_loop.sh
```

## One-command smoke

```bash
./scripts/smoke_manager_daily_brief_local_loop.sh
```

This command starts:

```bash
docker compose --profile agent-infra up --build -d \
  postgres minio pet-resort-api pet-resort-worker openviking
```

Then it executes the Manager Daily Brief loop:

1. Waits for the app API health endpoint.
2. Checks OpenViking health. If OpenViking is not initialized, the script records a precise blocker and continues because OpenViking is agent-side context infrastructure, not the app source of truth for this smoke.
3. Reads app-owned Manager Daily Brief context through `scripts/hermes-tools/get_manager_daily_brief_context`.
4. Builds a Hermes-authored draft packet from app source refs.
5. Submits the draft through `scripts/hermes-tools/submit_manager_daily_brief_draft` and verifies app validation accepts only source-grounded, review-gated, no-side-effect recommendations.
6. Submits an intentionally unsafe draft requesting `change_staff_schedule` and verifies the app returns HTTP 422 through the bridge instead of allowing the side effect.
7. Records a fake reviewed staff outcome through `scripts/hermes-tools/record_manager_daily_brief_outcome`.
8. Verifies the app reports estimated vs actual labor minutes saved.

Expected success lines look like:

```text
context_ok actions=3 minutes_saved=62
draft_validation_ok accepted_actions=1 live_side_effects_allowed=false
blocked_draft_validation_ok http_422_secret_safe_error=true
outcome_ok estimated_minutes_saved=12 actual_minutes_saved=8
full local Manager Daily Brief loop passed without live customer/PMS/payment side effects
```

The script writes JSON artifacts to a temp directory and prints the path.

## Manual command sequence

Use this when debugging individual contracts.

```bash
export PET_RESORT_API_URL=http://127.0.0.1:3001
docker compose --profile agent-infra up --build -d \
  postgres minio pet-resort-api pet-resort-worker openviking
curl -fsS "$PET_RESORT_API_URL/healthz"

scripts/hermes-tools/get_manager_daily_brief_context \
  --location-id 00c0ffee-0000-0000-0000-000000000001 \
  --operating-day 2026-06-17 \
  > /tmp/manager-daily-brief-context.json
```

Create a draft that cites source refs from the context packet and requests no side effects:

```bash
python - /tmp/manager-daily-brief-context.json /tmp/manager-daily-brief-draft.json <<'PY'
import json, sys
context = json.load(open(sys.argv[1], encoding="utf-8"))
source_refs = context["manager_brief_actions"][0].get("source_refs") or context["source_refs"][:1]
draft = {
    "context_packet_id": context["audit"]["context_packet_id"],
    "correlation_id": context["audit"]["correlation_id"],
    "submitted_by": "hermes-agent-local-smoke",
    "actions": [{
        "id": "smoke-demand-staffing-1",
        "kind": "review_demand_against_staffing_plan",
        "recommendation": "Review boarding demand against the staffing plan before morning drop-off.",
        "source_refs": source_refs,
        "review_gates": ["manager_approval"],
        "requested_side_effects": []
    }]
}
json.dump(draft, open(sys.argv[2], "w", encoding="utf-8"), indent=2)
PY

scripts/hermes-tools/submit_manager_daily_brief_draft \
  --draft-file /tmp/manager-daily-brief-draft.json
```

Record a fake reviewed outcome for the checkout exception action:

```bash
python - /tmp/manager-daily-brief-context.json /tmp/manager-daily-brief-outcome.json /tmp/manager-daily-brief-action-id.txt <<'PY'
import json, sys
context = json.load(open(sys.argv[1], encoding="utf-8"))
action = next(a for a in context["manager_brief_actions"] if a["kind"] == "resolve_checkout_exception")
source_refs = []
for ref in action.get("source_refs") or context["source_refs"][:1]:
    normalized = dict(ref)
    normalized.setdefault("record_type", "reservation")
    normalized.setdefault("observed_at", "2026-06-17T12:00:00Z")
    normalized.setdefault("adapter_version", "local-manager-daily-brief-smoke-v1")
    source_refs.append(normalized)
outcome = {
    "outcome": "completed",
    "actual_minutes": 12,
    "actor": {"id": "front-desk-lead-local-smoke", "persona": "front_desk_lead"},
    "feedback": "Fake reviewed outcome; no live external side effect attempted.",
    "source_refs": source_refs,
    "timestamp": "2026-06-17T13:15:00Z",
    "audit": {"correlation_id": context["audit"]["correlation_id"]},
    "reporting": {"location_id": context["location_id"], "operating_day": context["operating_day"]},
    "requested_side_effects": []
}
json.dump(outcome, open(sys.argv[2], "w", encoding="utf-8"), indent=2)
open(sys.argv[3], "w", encoding="utf-8").write(action["id"])
PY

scripts/hermes-tools/record_manager_daily_brief_outcome \
  --action-id "$(cat /tmp/manager-daily-brief-action-id.txt)" \
  --outcome-file /tmp/manager-daily-brief-outcome.json
```

The outcome response includes `labor_savings_evidence.estimated_minutes_saved` and `labor_savings_evidence.actual_minutes_saved`.

## OpenViking preflight

The smoke calls `scripts/preflight_openviking_agent_infra.sh --allow-uninitialized` before continuing through the app-owned Manager Daily Brief loop. In a fresh local Compose volume, the `openviking` container may start but return HTTP 503 because `/app/.openviking/ov.conf` is missing. The preflight writes `openviking-status.txt` under the smoke temp directory and prints the remediation:

```bash
docker compose --profile agent-infra up -d openviking
docker compose exec openviking openviking-server init
docker compose restart openviking
scripts/preflight_openviking_agent_infra.sh
```

For non-interactive setup, an operator may provide the full `ov.conf` JSON through a local secret channel and let Compose pass it to the upstream entrypoint:

```bash
export OPENVIKING_CONF_CONTENT='<full ov.conf JSON from .env, 1Password, or another local secret source>'
docker compose --profile agent-infra up -d openviking
scripts/preflight_openviking_agent_infra.sh
```

Do not commit real provider keys, `root_api_key` values, or generated `ov.conf` files. This remains a precise agent-infra blocker only: the Manager Daily Brief labor loop does not rely on OpenViking for app-owned facts, source refs, validation, review gates, persistence, or labor-savings reporting.

## Canonical repo gate

After changes, run:

```bash
./scripts/test.sh
```
