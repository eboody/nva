# Hermes Manager Daily Brief app bridge

This is the first minimal Hermes-to-app bridge for the Manager Daily Brief labor loop. It intentionally uses small runnable scripts rather than committing the repo to MCP packaging too early.

## Boundary

The deterministic pet-resort app remains the source of truth. These scripts only:

1. read an app-owned typed context packet;
2. submit a Hermes-authored draft packet to app-owned validation;
3. record reviewed staff/manager outcome feedback.

They do not read Postgres, object storage, Gingr, customer channels, schedules, payments, or provider/PMS systems directly. Live side effects remain app-owned and review-gated.

## Configuration

Set the app URL for the runtime that is calling the tools:

```bash
export PET_RESORT_API_URL=http://127.0.0.1:3001
```

Optional bearer auth is supported for future deployments:

```bash
export PET_RESORT_API_TOKEN=...
```

Do not commit model/provider secrets or API tokens. Error messages intentionally avoid printing request URLs, headers, tokens, or raw upstream response bodies.

## Tools

All scripts live under `scripts/hermes-tools/` and use only Python stdlib.

### get_manager_daily_brief_context

Calls the app's read-only context endpoint:

```bash
scripts/hermes-tools/get_manager_daily_brief_context \
  --location-id 00c0ffee-0000-0000-0000-000000000001 \
  --operating-day 2026-06-17
```

HTTP contract:

```text
GET /agent/context/manager-daily-brief?location_id=...&operating_day=...
```

Hermes should use the returned packet as typed context for ranking manager/front-desk actions, summarizing source evidence, and estimating labor minutes saved. The packet includes audit/correlation ids, allowed agent actions, blocked actions, source refs, and data-quality issues.

### submit_manager_daily_brief_draft

Posts a draft/recommendation packet back to the app validation endpoint. The JSON may be passed on stdin:

```bash
scripts/hermes-tools/submit_manager_daily_brief_draft < draft.json
```

or from a file:

```bash
scripts/hermes-tools/submit_manager_daily_brief_draft --draft-file draft.json
```

HTTP contract:

```text
POST /agent/drafts/manager-daily-brief
```

The app validates action kinds, source refs, review gates, and requested side effects. Hermes must treat rejected actions as app-owned policy feedback, not as something to bypass.

Minimal draft shape:

```json
{
  "context_packet_id": "manager-daily-brief-context:00c0ffee-0000-0000-0000-000000000001:2026-06-17",
  "correlation_id": "manager-daily-brief:00c0ffee-0000-0000-0000-000000000001:2026-06-17",
  "submitted_by": "hermes-agent",
  "actions": [
    {
      "id": "draft-demand-staffing-1",
      "kind": "review_demand_against_staffing_plan",
      "recommendation": "Review demand against the staffing plan before morning drop-off.",
      "source_refs": [
        {
          "system": "gingr",
          "record_type": "service_demand_forecast",
          "record_id": "demand:00c0ffee-0000-0000-0000-000000000001:2026-06-17",
          "observed_at": "2026-06-17T12:00:00Z",
          "adapter_version": "local-manager-daily-brief-fixture-v1"
        }
      ],
      "review_gates": ["manager_approval"],
      "requested_side_effects": []
    }
  ]
}
```

### record_manager_daily_brief_outcome

Posts reviewed outcome feedback for a specific action id:

```bash
scripts/hermes-tools/record_manager_daily_brief_outcome \
  --action-id checkout-exception-reservation-4242 \
  --outcome-file outcome.json
```

or:

```bash
scripts/hermes-tools/record_manager_daily_brief_outcome \
  --action-id checkout-exception-reservation-4242 < outcome.json
```

HTTP contract:

```text
POST /manager-daily-brief/actions/{action_id}/outcome
```

Minimal outcome shape:

```json
{
  "outcome": "completed",
  "actual_minutes": 12,
  "actor": {"id": "front-desk-lead-17", "persona": "front_desk_lead"},
  "feedback": "Resolved before checkout rush; brief saved a manual open-stay audit.",
  "source_refs": [],
  "timestamp": "2026-06-17T13:15:00Z",
  "audit": {
    "correlation_id": "manager-daily-brief:00c0ffee-0000-0000-0000-000000000001:2026-06-17"
  },
  "reporting": {
    "location_id": "00c0ffee-0000-0000-0000-000000000001",
    "operating_day": "2026-06-17"
  },
  "requested_side_effects": []
}
```

Outcome capture is evidence for labor savings. It does not authorize provider/PMS mutation, customer sends, schedule changes, payments/refunds/discounts, or hiding source data-quality issues.

## How Hermes or a worker profile should call the bridge

For a scripts-based worker profile, expose each script as a command/tool wrapper with:

```text
PET_RESORT_API_URL=http://pet-resort-api:3001
```

in Docker/Compose, or:

```text
PET_RESORT_API_URL=http://127.0.0.1:3001
```

for local development.

Suggested agent loop:

1. Call `get_manager_daily_brief_context` for a location/day.
2. Draft only actions whose `kind` appears in the app's allowed workflow contract and whose claims cite packet source refs.
3. Call `submit_manager_daily_brief_draft` and keep only app-accepted actions.
4. After a human/staff review decision exists, call `record_manager_daily_brief_outcome` with the reviewed outcome and actual minutes.
5. Never add live side effects to either draft or outcome packets.

## Local verification

Run the full local smoke loop:

```bash
./scripts/smoke_manager_daily_brief_local_loop.sh
```

If another local stack already uses the default Postgres port, override only that host port:

```bash
PET_RESORT_POSTGRES_HOST_PORT=55432 ./scripts/smoke_manager_daily_brief_local_loop.sh
```

The smoke starts Postgres, MinIO, app API/worker, and OpenViking via Compose; reads app-owned context through this bridge; submits a source-grounded draft; verifies app validation rejects an unsafe side-effect request; records a fake reviewed outcome; and checks estimated vs actual minutes saved. See `docs/ops/manager-daily-brief-local-smoke.md` for the full reproducible sequence and OpenViking initialization blocker.

The canonical gate runs the bridge unit tests:

```bash
./scripts/test.sh
```

The focused bridge test can be run directly:

```bash
python -m unittest scripts.tests.test_hermes_agent_bridge -v
```

Those tests use a fake local HTTP server to prove the scripts call the configured app URL, post the expected JSON, pass optional bearer auth, and keep error output secret-safe.

To call the real local demo API instead:

```bash
cargo run -p pet-resort-api
PET_RESORT_API_URL=http://127.0.0.1:3001 \
  scripts/hermes-tools/get_manager_daily_brief_context \
  --location-id 00c0ffee-0000-0000-0000-000000000001 \
  --operating-day 2026-06-17
```

## Migration note: MCP later

If these scripts remain useful after the first labor-loop pilot, wrap the same three app-owned contracts as MCP or Hermes native custom tools. Keep the tool boundary identical: MCP should be a packaging improvement, not a permission expansion. The deterministic app must still own facts, validation, review gates, persistence, audit, and side effects.
