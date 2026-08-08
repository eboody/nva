# NVA 3-minute presentation script

Use this as a spoken script for a job, networking, hiring-manager, product, operations, or technical conversation. It is intentionally conversational. It does not claim live NVA/Gingr credentials, production data, production deployment, provider/PMS writes, customer/member sends, payment/refund/discount movement, schedule/capacity changes, medical/safety decisions, or a completed Gingr replacement.

## Recommended opening line

"I built a safe local proof of the operations layer I think NVA Pet Resorts would want above systems like Gingr — not a fake production integration, but the owned API, realtime queue, review, audit, and BI-read-model seam you would need before connecting anything live."

## Three-minute version

### 0:00-0:25 — Open with the honest thesis

"I built this under a real constraint: I did not have live NVA or Gingr access, so I did not pretend to integrate with production systems. Instead, I built the safer first step: a local proof of an NVA-owned operations layer above provider systems like Gingr.

The point is not to clone Gingr. Gingr is source evidence. NVA still needs product-owned contracts for review queues, source-quality cleanup, labor outcomes, audit, BI-friendly read models, and now a realtime role/location-scoped operating loop. This repo shows that seam while keeping live side effects disabled."

### 0:25-1:10 — Explain the visual

Open `docs/presentation/owned-operations-api-visual-guide.md` or `docs/presentation/assets/owned-operations-api-replacement.html`.

"This picture is the project in one frame. Provider data stays visible as evidence, with provenance and caveats. NVA owns the operations API, workflow packets, review gates, audit/events/metrics, BI read models, and scoped queue views. SpacetimeDB fits on the runtime edge: reducers and subscriptions can publish staff and manager views, while domain and app code still own the business rules. The safety boundary is explicit: no customer sends, no provider writes, no payment, schedule, capacity, or medical decisions, and no production claim in this local proof."

### 1:10-2:15 — Narrate the demo

Run or point to:

```sh
./scripts/demo_owned_operations_api.sh
```

"The runnable slice is Data-Quality Hygiene. The contract lane checks the OpenAPI boundary and confirms live side effects are disabled. The workflow lane turns a source-quality issue into a reviewable cleanup packet, validates a safe internal recommendation, rejects unsafe actions like customer messaging or source repair, and records reviewed labor evidence. The operations lane proves the worker/outbox posture is disabled and local. The closeout marker is `demo_owned_operations_api_ok local_fixture_only=true live_side_effects_allowed=false`.

If there is time, the realtime queue script adds the SpacetimeDB story: Alice at Location 101 sees queue updates, Sam at Location 202 is hidden/blocked, Morgan sees manager-gated work, and unsafe customer/provider side effects stay blocked. On this workstation it may use the documented ABI fallback, so I would not call it a live published module."

### 2:15-2:45 — State caveats confidently

"What is real here is the architecture and local proof: typed contracts, safe demo routes, source refs, review gates, blocked actions, labor-outcome records, OpenAPI evidence, and tests.

What I am not claiming is equally important: no live NVA or Gingr credentials, no production data, no production deployment, no production SSO, no provider/PMS writes, no member sends, no money movement, no schedule changes, and no medical or safety decisions. Auth and role/location scope are fixtures; durable production wiring and approved live-action paths are future work."

### 2:45-3:00 — Ask for the next safe step and close

"The next useful step is not production credentials or write access. I would ask for narrow read-only validation: endpoint/report docs, redacted exports or snapshots, provider and role/location mappings, BI query inventory, and approved KPI definitions. Then we can compare the owned read models against real source shape and scope one safe dual-run pilot while live writes and sends remain disabled.

That is credible because it says where automation stops: local proof now, read-only validation next, and no live side effects until approval and system-of-record authority are real."

## Optional five-minute expansion

Use this when the conversation is going well or the audience wants more technical/product depth.

### Add after the opening: why BI is the product clue

"A useful clue is that BI often already has to pull provider data into a separate reporting path. That does not mean BI is wrong; it means raw provider shape is not enough for operations. If NVA wants consistent reporting and safer automation, the product should make source quality, review status, labor outcomes, audit lineage, and caveats first-class upstream concepts instead of forcing every analyst to rediscover them downstream."

### Add during the visual: provider evidence versus product authority

"I would describe the boundary this way: provider DTOs are adapter evidence; owned API DTOs are NVA operating contracts. A source can say what it emitted, but it should not automatically own NVA's review queue, workflow authority, labor metric, or BI projection. Promotion from source evidence into product meaning should be explicit, validated, and reviewable."

### Add during the demo: what each marker means in plain English

When the command prints anchors, narrate them lightly rather than reading every line:

- `openapi_title` and `openapi_paths` mean there is a checked contract artifact, not only prose.
- `context_ok` means the app can assemble a source-backed cleanup packet.
- `draft_validation_ok` means a safe internal recommendation can pass validation.
- `blocked_draft_validation_ok` means unsafe side effects are rejected.
- `outcome_ok` means reviewed labor evidence can be recorded.
- `live_side_effects_allowed=false` means the proof is intentionally not sending, writing, moving money, changing schedules, or making safety decisions.

Suggested narration:

"I would not over-rotate on the terminal output. The business meaning is: source evidence enters with caveats, reviewable work is created, unsafe actions stay blocked, staff outcome evidence is recorded, and BI gets cleaner concepts to consume later."

### Add before the close: why SpacetimeDB does not eliminate Postgres/S3

"SpacetimeDB is the realtime operations runtime: reducers, subscriptions, and scoped staff/manager queue views. I would still keep Postgres and S3-compatible storage in the enterprise design because they have different jobs: Postgres is the durable audit, history, SQL reporting, reconciliation, and export ledger; S3 or MinIO is for immutable source snapshots, documents, media, export bundles, hashes, and manifests. I would remove Postgres only after SpacetimeDB proves years-scale audit retention, point-in-time history, BI-safe exports, source reconciliation, object-evidence handling, backup/restore, and operational tooling."

### Add before the close: what I would build next

"The next engineering slice I would choose is durable realtime Data-Quality Hygiene: API request or reducer command to scoped SpacetimeDB queue/read-model rows, durable Postgres workflow/review/audit/outcome rows with the same correlation id, S3/MinIO evidence refs when raw source snapshots are needed, disabled or fake worker execution, reviewed internal outbox candidate, correlated logs, local metrics, and eventually queue/dead-letter views. I would still keep customer sends, provider writes, payments, schedule changes, and medical/safety actions disabled until explicit owner approval and system-of-record authority exist."

## If the local demo is slow or fails

Say this and move on; do not spend the conversation debugging or pretend stale output is fresh proof:

"Pre-flight was run before the conversation; if this shell is slow or unavailable, I won’t pretend stale output is fresh proof — I’ll show the static diagram, the checked OpenAPI artifact, and the expected local-demo anchors instead."

Then open `docs/presentation/nva-static-demo-fallback.md`, point to `docs/presentation/job-presentation-walkthrough.md#local-demo-slice-data-quality-hygiene-in-five-minutes`, and continue the story from the expected anchors.

## Short phrases to keep it human

Use these instead of spec-heavy language:

- "I did not fake a production integration."
- "Gingr is source evidence, not product authority."
- "The proof shows where automation stops."
- "SpacetimeDB is the realtime adapter, not a business-logic rewrite."
- "Role and location scope matters when one operating model has to work across roughly 170 locations."
- "BI should get cleaner upstream meaning, not just another raw export to repair."
- "The next ask is read-only shape, not write access."
- "This is presentation-ready architecture and local proof, not a production replacement claim."

## Claims to avoid while presenting

Do not say:

- "I integrated with live NVA/Gingr."
- "This replaces Gingr today."
- "The API is production-ready."
- "The database is live behind every route."
- "The SpacetimeDB module is published and production-ready."
- "SpacetimeDB means Postgres and object storage are unnecessary now."
- "The worker sends messages or repairs provider data."
- "BI can turn off its current database now."

Say instead:

- "The provider boundary is modeled and fixture/local proof runs now."
- "This is an owned API replacement path, starting with a safe local slice."
- "Production readiness requires approved access, durable persistence, auth/location scope, monitoring, rollback, and explicit live-action gates."
- "The current worker/outbox path is disabled and review-gated."
- "BI could consume owned read models after durable wiring and read-only/dual-run validation."
- "The current realtime demo uses fixture actors/local fallback where the SpacetimeDB host ABI is incompatible; it proves the boundary/story, not production SSO or live Gingr access."
- "Postgres/S3 remain the durable audit/reporting/evidence backbone unless SpacetimeDB-only proof gates are met."

## Recommended closing line

"This is credible because it says exactly where automation stops: local contract proof now, read-only validation next, and no live side effects until approval, system-of-record authority, and production controls are real."
