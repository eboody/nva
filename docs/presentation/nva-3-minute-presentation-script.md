# NVA 3-minute presentation script

Use this as a spoken script for a job, networking, hiring-manager, product, operations, or technical conversation. It is intentionally conversational. It does not claim live NVA/Gingr credentials, production data, production deployment, provider/PMS writes, customer/member sends, payment/refund/discount movement, schedule/capacity changes, medical/safety decisions, or a completed Gingr replacement.

## Recommended opening line

"I built a safe local proof of the operations layer I think NVA Pet Resorts would want above systems like Gingr — not a fake production integration, but the owned API, review, audit, and BI-read-model seam you would need before connecting anything live."

## Three-minute version

### 0:00-0:25 — Open with the honest thesis

"I built this under a real constraint: I did not have live NVA or Gingr access, so I did not pretend to integrate with production systems. Instead, I built the safer first step: a local proof of an NVA-owned operations API and read-model layer above provider systems like Gingr.

The point is not to clone Gingr. Gingr is source evidence. NVA still needs product-owned contracts for review queues, source-quality cleanup, labor outcomes, audit, and BI-friendly read models. This repo shows what that seam could look like while keeping live side effects disabled."

### 0:25-1:10 — Explain the visual

Open `docs/presentation/owned-operations-api-visual-guide.md` or `docs/presentation/assets/owned-operations-api-replacement.html`.

"This picture is the project in one frame. On the left is the current pattern: a provider system such as Gingr emits source data, BI extracts it, and operations still have to infer a lot of meaning downstream — which records are stale, what cleanup was reviewed, what labor was saved, and what is safe to act on.

On the right is the product direction I would build toward. Provider data stays visible as evidence, with provenance and caveats. NVA owns the operations API, workflow packets, review gates, audit/events/metrics, and BI read models. The safety boundary is explicit: no customer sends, no provider writes, no payment, schedule, capacity, or medical decisions, and no production claim in this local proof."

### 1:10-2:15 — Narrate the demo

Run or point to:

```sh
./scripts/demo_owned_operations_api.sh
```

"The runnable slice is Data-Quality Hygiene because it connects the business need to something concrete. The demo has three lanes.

First, the contract lane checks the OpenAPI boundary for the owned operations API and confirms live side effects are disabled.

Second, the workflow lane turns a source-quality issue into a reviewable cleanup packet. It validates a safe internal recommendation, rejects unsafe actions like customer messaging or source repair, and records the reviewed outcome with estimated and actual labor minutes.

Third, the operations lane proves the worker/outbox posture is disabled and local. Outbox-shaped work is treated as a future approved handoff, not as a live sender. The important closeout marker is `demo_owned_operations_api_ok local_fixture_only=true live_side_effects_allowed=false`."

### 2:15-2:45 — State caveats confidently

"What is real here is the architecture and local proof: typed contracts, safe demo routes, source refs, review gates, blocked actions, labor-outcome records, OpenAPI evidence, and tests.

What I am not claiming is equally important: no live NVA or Gingr credentials, no production data, no production deployment, no provider/PMS writes, no member sends, no money movement, no schedule changes, and no medical or safety decisions. The API is still local/in-memory for the demo; durable production wiring, auth/location scope, worker leasing, monitoring, and owner-approved live-action paths are future work."

### 2:45-3:00 — Ask for the next safe step and close

"The next useful step is not production credentials or write access. I would ask for narrow read-only validation: endpoint or report docs, redacted exports or sample source snapshots, provider ID/status/service-line mappings, BI query inventory, and approved KPI definitions. Then we can compare the owned read models against real source shape and scope one safe dual-run pilot while live writes and sends remain disabled.

The reason I think this is credible is that it says exactly where automation stops: local contract proof now, read-only validation next, and no live side effects until the approval path and system-of-record authority are real."

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

### Add before the close: what I would build next

"The next engineering slice I would choose is durable Data-Quality Hygiene: API request to Postgres workflow/review/audit/outcome rows, disabled or fake worker execution, reviewed internal outbox candidate, correlated logs, local metrics, and eventually queue/dead-letter views. I would still keep customer sends, provider writes, payments, schedule changes, and medical/safety actions disabled until explicit owner approval and system-of-record authority exist."

## If the local demo is slow or fails

Say this and move on; do not spend the conversation debugging or pretend stale output is fresh proof:

"Pre-flight was run before the conversation; if this shell is slow or unavailable, I won’t pretend stale output is fresh proof — I’ll show the static diagram, the checked OpenAPI artifact, and the expected local-demo anchors instead."

Then open `docs/presentation/nva-static-demo-fallback.md`, point to `docs/presentation/job-presentation-walkthrough.md#local-demo-slice-data-quality-hygiene-in-five-minutes`, and continue the story from the expected anchors.

## Short phrases to keep it human

Use these instead of spec-heavy language:

- "I did not fake a production integration."
- "Gingr is source evidence, not product authority."
- "The proof shows where automation stops."
- "BI should get cleaner upstream meaning, not just another raw export to repair."
- "The next ask is read-only shape, not write access."
- "This is presentation-ready architecture and local proof, not a production replacement claim."

## Claims to avoid while presenting

Do not say:

- "I integrated with live NVA/Gingr."
- "This replaces Gingr today."
- "The API is production-ready."
- "The database is live behind every route."
- "The worker sends messages or repairs provider data."
- "BI can turn off its current database now."

Say instead:

- "The provider boundary is modeled and fixture/local proof runs now."
- "This is an owned API replacement path, starting with a safe local slice."
- "Production readiness requires approved access, durable persistence, auth/location scope, monitoring, rollback, and explicit live-action gates."
- "The current worker/outbox path is disabled and review-gated."
- "BI could consume owned read models after durable wiring and read-only/dual-run validation."

## Recommended closing line

"This is credible because it says exactly where automation stops: local contract proof now, read-only validation next, and no live side effects until approval, system-of-record authority, and production controls are real."
