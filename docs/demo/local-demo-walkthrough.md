# Local demo walkthrough: Data-Quality Hygiene owned-operations proof

Status: job-contact / recruiter / technical-interviewer walkthrough for the local demo. This is a safe local proof, not a production deployment and not a claim of live NVA, Gingr, PMS, customer, payment, schedule, or medical/safety access.

Use this when you need to explain the value before showing code, then give a technical interviewer exact proof paths for the schema, API, storage, and smoke commands.

## One-page recruiter / hiring-manager framing

### Problem

Pet-resort operators lose time reconciling messy source-system facts: stale vaccination evidence, duplicate or incomplete profiles, conflicting source fields, and BI cleanup that happens downstream instead of where the work is understood. The expensive part is not just a missing field; it is repeated manager/front-desk attention across many locations.

### What the demo shows

The local demo proves an NVA-owned operations API layer above provider evidence. Gingr or another PMS can supply source facts, but the owned layer decides how those facts become reviewable cleanup work, labor-outcome evidence, audit records, metrics, and BI-ready read models.

The selected slice is Data-Quality Hygiene because it is easy to explain:

1. source evidence says a fact may be stale, missing, duplicate, or conflicting;
2. the owned app builds a reviewable internal cleanup packet;
3. the agent/draft path is allowed to summarize and rank safe internal work only;
4. unsafe side effects are rejected;
5. reviewed outcomes record estimated and actual labor minutes saved;
6. metrics/read-models can then report operational progress without hiding the source caveat.

### Architecture in one sentence

Provider/source evidence -> product-owned app/domain workflow packet -> versioned `/v0` API DTOs -> review gate -> storage-shaped review/audit/outcome/outbox projections -> aggregate metrics/read-models, with live side effects disabled.

### Labor-cost loop

The loop is not “AI sends messages.” It is “AI reduces repetitive reconciliation by turning ambiguous source facts into reviewable internal work and measured outcomes.” The proof marker is the local smoke output: `estimated_minutes_saved=15` and `actual_minutes_saved=17` on fixture data, with `live_side_effects_allowed=false` preserved.

### Proof chain

- Business/problem framing: [README presentation path](../../README.md#presentation-path-safe-local-owned-api-proof).
- Workflow contract: [Data-Quality Hygiene local smoke](../ops/data-quality-hygiene-local-smoke.md).
- App/domain contract: [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs), [`domain/src/data_quality.rs`](../../domain/src/data_quality.rs), and [`domain/src/source.rs`](../../domain/src/source.rs).
- API contract: [`apps/api/README.md`](../../apps/api/README.md) and checked OpenAPI artifact [`apps/api/openapi/owned-operations-v0.openapi.json`](../../apps/api/openapi/owned-operations-v0.openapi.json).
- Storage/schema proof path: [`migrations/0001_mvp_foundation.sql`](../../migrations/0001_mvp_foundation.sql), [`migrations/0002_data_quality_read_models.sql`](../../migrations/0002_data_quality_read_models.sql), [`storage/src/operations.rs`](../../storage/src/operations.rs), and storage tests under [`storage/tests/`](../../storage/tests/).
- Presentation fallback: [NVA static demo fallback packet](../presentation/nva-static-demo-fallback.md).

### Safe boundaries to say out loud

This demo does not send customer/member messages, write to Gingr/PMS/provider systems, move money, change schedules/capacity, make medical/safety decisions, merge/delete profiles, or hide source ambiguity. Outbox/review records are handoff candidates only, not permission to send.

## Why no NVA/Gingr access is needed for this proof

The demo is intentionally access-constrained. It proves the owned contract shape before asking anyone for live access.

- Synthetic source evidence is enough for the first proof because the question is whether the owned app preserves provenance, ambiguity, review gates, blocked actions, and labor outcomes. Fixture evidence can prove those invariants without touching real customers.
- Review gates are the product boundary. The demo shows that source facts become reviewable internal work, not automatic provider repair or customer messaging.
- Disabled side effects are a feature, not a gap. `live_side_effects_allowed=false`, fake/deterministic worker behavior, and disabled outbox publishing demonstrate that the integration can be inspected safely before live credentials exist.
- The clear next step with access is narrow read-only validation: approved docs, exports, sample records, or source snapshots to compare actual Gingr/NVA field shape against the owned DTOs/read models. Live writes, sends, payment movement, schedule changes, and medical/safety decisions remain separately approved future work.

Suggested line:

> I did not have live NVA or Gingr access, so I did not fake a production integration. I built the safe seam first: source evidence in, reviewable workflow and labor proof out, with live side effects disabled. The next useful access request is read-only validation against real docs or sample exports.

## Setup commands

From a clean checkout of this branch/worktree:

```sh
git status --short
cargo --version
python --version
./scripts/check_docs.sh
```

Primary demo wrapper:

```sh
./scripts/demo_owned_operations_api.sh
```

Separate lanes if you want to show only one part:

```sh
./scripts/smoke_data_quality_hygiene_local_loop.sh
./scripts/smoke_data_quality_hygiene_disabled_worker_outbox.sh
```

Expected wrapper anchors:

```text
openapi_title=NVA Pet Resorts Owned Operations API
openapi_paths=8
contract_lane_ok live_side_effects_allowed=false
context_ok workflow=data-quality-hygiene actions=1 estimated_minutes_saved=15 live_side_effects_allowed=false
draft_validation_ok accepted_actions=1 requested_side_effects=0
blocked_draft_validation_ok blocked_side_effect=send_customer_message
outcome_ok estimated_minutes_saved=15 actual_minutes_saved=17 live_side_effects_allowed=false
smoke_assertions_ok estimated_minutes_saved=15 actual_minutes_saved=17
[data-quality-hygiene-worker-outbox-smoke] disabled worker/outbox proof passed as local internal handoff only
demo_owned_operations_api_ok local_fixture_only=true live_side_effects_allowed=false
```

If the shell is unavailable during a conversation, use the static packet rather than pretending stale output is fresh: [NVA static demo fallback packet](../presentation/nva-static-demo-fallback.md).

## Endpoint list for the technical proof path

The checked OpenAPI v0 surface currently lists these eight paths:

- `GET /v0/healthz`
- `GET /v0/readyz`
- `GET /v0/ops/metrics/summary`
- `GET /v0/agent/context/data-quality-hygiene`
- `POST /v0/agent/drafts/data-quality-hygiene`
- `POST /v0/data-quality-hygiene/actions/{action_id}/outcome`
- `GET /v0/data-quality-hygiene/outcomes/summary`
- `GET /v0/read-models/source-quality-backlog`

Final smoke / inspection commands:

```sh
./scripts/demo_owned_operations_api.sh
./scripts/smoke_data_quality_hygiene_local_loop.sh
./scripts/smoke_data_quality_hygiene_disabled_worker_outbox.sh
python - <<'PY'
import json
from pathlib import Path
p = Path('apps/api/openapi/owned-operations-v0.openapi.json')
data = json.loads(p.read_text())
print(data['info']['title'])
for path in sorted(data['paths']):
    print(path)
PY
```

Optional live-server curl walkthrough after starting the local Docker demo:

```sh
docker compose up --build -d --wait
./scripts/smoke_local_demo.sh
curl -sS http://127.0.0.1:3001/v0/readyz | python -m json.tool
curl -sS http://127.0.0.1:3001/v0/read-models/source-quality-backlog | python -m json.tool
curl -sS http://127.0.0.1:3000/ >/tmp/pet-resort-staff-web.html
```

Checked local Compose verification snapshots:

```json
{
  "readyz": {
    "service": "pet-resort-api",
    "database": "configured_not_verified",
    "object_storage": "env_configured_not_verified",
    "agent_runtime": "fake_deterministic",
    "live_customer_messaging": "disabled",
    "live_provider_writes": "disabled"
  },
  "source_quality_backlog": {
    "database": { "status": "connected", "adapter": "tokio_postgres", "error": null },
    "records_len": 3,
    "first_issue": "SQI-LOCAL-001",
    "data_posture": {
      "safe_synthetic_data": true,
      "live_side_effects_allowed": false,
      "provider_payload_passthrough": false,
      "provider_writes_allowed": false,
      "customer_messages_allowed": false
    }
  }
}
```

## Three-to-five-minute demo script

### 0:00-0:30 — frame the problem

“Pet resorts waste expensive manager/front-desk time reconciling source-system ambiguity: stale evidence, missing fields, duplicate profiles, and BI cleanup that happens too late. I built a local proof of the operations layer I would want above Gingr/provider evidence.”

Point at the one-frame visual if using slides: [owned operations visual guide](../presentation/owned-operations-api-visual-guide.md) or the standalone visual [`docs/presentation/assets/owned-operations-api-replacement.html`](../presentation/assets/owned-operations-api-replacement.html).

### 0:30-1:15 — show the safety boundary

“Provider facts are evidence, not product authority. The app keeps provenance and ambiguity visible, routes work through review gates, and blocks live side effects. This is why the demo can run without NVA/Gingr credentials.”

Run or show:

```sh
./scripts/demo_owned_operations_api.sh
```

Call out `contract_lane_ok live_side_effects_allowed=false`.

### 1:15-2:15 — narrate the workflow lane

Point to these markers:

- `context_ok`: the app built a source-grounded Data-Quality Hygiene packet.
- `draft_validation_ok`: a safe internal cleanup recommendation passed validation.
- `blocked_draft_validation_ok blocked_side_effect=send_customer_message`: an unsafe customer-send request was rejected.
- `outcome_ok`: reviewed outcome evidence recorded positive labor savings.

Suggested line:

“Notice the agent is not being trusted to act in the world. It can draft and rank internal work, but app policy rejects the side effect before any adapter exists.”

### 2:15-3:00 — show the contract/API proof

Open the checked OpenAPI artifact and route list:

```sh
python - <<'PY'
import json
from pathlib import Path
p = Path('apps/api/openapi/owned-operations-v0.openapi.json')
data = json.loads(p.read_text())
print(data['info']['title'], data['info']['version'])
print('\n'.join(sorted(data['paths'])))
PY
```

Explain that the public `/v0` DTOs are product-owned. Provider DTOs are not the API boundary; source refs and review gates are.

### 3:00-4:00 — show storage/schema path for a technical interviewer

Open these files if asked how the proof becomes durable:

- [`migrations/0001_mvp_foundation.sql`](../../migrations/0001_mvp_foundation.sql) for workflow/review/approval/outbox/audit foundation tables.
- [`migrations/0002_data_quality_read_models.sql`](../../migrations/0002_data_quality_read_models.sql) for source-quality read-model posture.
- [`storage/src/operations.rs`](../../storage/src/operations.rs) for outcome/source-ref/labor projection types.
- [`storage/tests/data_quality_hygiene_outcome_storage.rs`](../../storage/tests/data_quality_hygiene_outcome_storage.rs) and [`storage/tests/mvp_migration_contract.rs`](../../storage/tests/mvp_migration_contract.rs) for executable storage/schema expectations.

Suggested line:

“The point is not a slide-only architecture. The schema and storage tests name the review, approval, audit, outbox, source-ref, issue-ref, and labor-minute path a pilot would need.”

### 4:00-5:00 — close with the access ask

“The next step is not live writes. It is read-only validation: docs, exports, sample rows, and BI query inventory. That would let us compare the owned read models against actual source shape, identify gaps, and choose one pilot workflow while keeping customer sends, provider writes, money movement, schedule changes, and medical/safety decisions blocked.”

## Screenshot / visual fallback references

Use these fallback references if no fresh screenshots are produced:

1. Terminal output from `./scripts/demo_owned_operations_api.sh` highlighting `contract_lane_ok`, `context_ok`, `blocked_draft_validation_ok`, `outcome_ok`, and `demo_owned_operations_api_ok`.
2. OpenAPI path list from [`apps/api/openapi/owned-operations-v0.openapi.json`](../../apps/api/openapi/owned-operations-v0.openapi.json).
3. Static visual [`docs/presentation/assets/owned-operations-api-replacement.html`](../presentation/assets/owned-operations-api-replacement.html) showing provider evidence -> owned operations API -> review/metrics/read models.
4. [`docs/ops/data-quality-hygiene-local-smoke.md`](../ops/data-quality-hygiene-local-smoke.md) expected-output block with labor-minute markers.

## Q&A

### “Is this connected to real NVA or Gingr data?”

No. It is fixture-only/local proof. That is intentional: it proves the contract and safety posture before live access. The next request is read-only validation, not live write credentials.

### “Why build an owned API instead of using Gingr directly?”

Gingr can be source evidence. NVA still needs owned workflow authority: provenance, review gates, labor outcomes, audit lineage, BI-facing read models, and explicit blocked actions. Those are product/operations contracts, not provider DTOs.

### “What makes this more than a chatbot demo?”

The agent is constrained by app/domain contracts. It can summarize evidence and draft internal work, but validation rejects unsafe side effects. The smoke proves reviewable workflow packets, blocked action handling, labor outcome evidence, and disabled worker/outbox posture.

### “What does the labor metric mean?”

The fixture shows a source-quality cleanup loop with estimated manual reconciliation minutes and reviewed actual minutes. In a real pilot, the same shape would compare pre-agent reconciliation time against reviewed outcomes across locations and source issue categories.

### “What would production need next?”

Production would need approved read-only source validation, auth/authorization, retention/redaction policy, durable infrastructure, monitoring/rollback, live adapter review, security review, and explicit policy approval for any side effect. None of that is claimed by this demo.

### “Can the outbox send customer messages?”

No. In this demo, outbox/review rows are handoff candidates only. Customer sends, provider/PMS writes, payment/refund/discount changes, schedule/capacity changes, and medical/safety decisions remain blocked.

### “Where do I inspect the API/schema proof?”

Start with [`apps/api/README.md`](../../apps/api/README.md), the checked OpenAPI artifact [`apps/api/openapi/owned-operations-v0.openapi.json`](../../apps/api/openapi/owned-operations-v0.openapi.json), migrations [`0001`](../../migrations/0001_mvp_foundation.sql) and [`0002`](../../migrations/0002_data_quality_read_models.sql), and [`storage/tests/`](../../storage/tests/).

## Caveats that build confidence

- This is local/demo proof, not production deployment.
- It uses synthetic/fixture source evidence, not production customer/provider data.
- The agent runtime/outbox posture is fake, deterministic, or disabled; that is the expected safe mode for this stage.
- The current smoke proves app-owned workflow, checked contract anchors, API/readiness posture, DB-backed source-quality read models, and staff-web reachability from local Compose.
- Toasty/ORM adoption is out of the critical path. The proof path is plain migrations plus repository/storage ports.
- The demo’s credibility comes from what it refuses to do: no hidden ambiguity resolution, no live customer sends, no provider repairs, no money movement, no schedule mutation, and no medical/safety decisions.

## Pre-flight checklist

Before a conversation:

```sh
git status --short
./scripts/demo_owned_operations_api.sh
./scripts/check_docs.sh
python scripts/check_markdown_links.py --repo-root .
```

If docs link checking fails on pre-existing generated Rustdoc targets outside this file, say so exactly and verify that this walkthrough’s relative links are clean.
