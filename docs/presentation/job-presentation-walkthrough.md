# Job presentation walkthrough: access-constrained Pet Resorts platform story

Status: five-minute presenter packet for a person introducing this work in a job/networking conversation. This page is intentionally honest: it does not claim live NVA/Gingr credentials, production data, permission to mutate provider systems, customer-message authority, payment authority, or production deployment.

Use this when the audience asks: "What did you actually build if you did not have live NVA access?" The answer is: a source-grounded, review-gated workflow platform boundary for pet-resort labor reduction, with typed DTO/API contracts, a durable DB/audit/outbox model, and explicit observability/readiness gaps.

## Local demo slice: Data-Quality Hygiene in five minutes

Use this slice when the audience wants something concrete to run before or during the conversation. It is fixture-only/local proof of the Data-Quality Hygiene loop: source-quality evidence enters an app-owned workflow packet, draft recommendations are validated, unsafe side effects are rejected, a reviewed outcome records labor evidence, and the worker/outbox posture remains disabled/fake rather than live.

From the repo root:

```sh
./scripts/smoke_data_quality_hygiene_local_loop.sh
./scripts/smoke_data_quality_hygiene_disabled_worker_outbox.sh
```

Expected output shape:

- The first smoke prints `[data-quality-hygiene-smoke]` progress lines and markers such as `context_ok`, `draft_validation_ok`, `blocked_draft_validation_ok`, `outcome_ok`, `live_side_effects_allowed=false`, and `smoke_assertions_ok estimated_minutes_saved=... actual_minutes_saved=...`.
- The second smoke runs the worker runtime contract test and ends with `[data-quality-hygiene-worker-outbox-smoke] disabled worker/outbox proof passed as local internal handoff only`.
- Both commands should complete without attempting customer sends, Gingr/PMS/provider writes, schedule changes, payment/refund/discount movement, or production deployment.

Talk track for the demo:

1. "This starts from a source-data hygiene problem: repeated staff time lost to stale, missing, duplicate, contradictory, or sensitive facts."
2. "The app packet ranks internal cleanup work with source refs, issue refs, review gates, blocked actions, and estimated labor impact."
3. "The draft-validation path accepts reviewable internal cleanup recommendations and rejects provider repair, customer messages, schedule/payment movement, ambiguity hiding, and missing evidence."
4. "Outcome capture records reviewed cleanup disposition, actual minutes spent/saved, request/workflow/review/outbox correlation, and a pending internal outbox candidate with live delivery disabled."
5. "The worker/outbox proof shows the runtime is fake deterministic or disabled and treats outbox as a future approved handoff, not a live sender."

What is real in this slice:

- App/domain contracts for data-quality candidates, actions, draft validation, blocked actions, outcomes, source refs, issue refs, and labor-minute evidence.
- API route contracts in `apps/api/src/http.rs` for context, draft submission, outcome capture, outcome summary, readiness, request-id propagation, and local metrics summary.
- Storage-shaped outcome records and migration/test proof for durable review/audit/outbox/outcome concepts.
- Local observability proof: request/workflow/review/outbox correlation fields, readiness language, and aggregate local counters for reviewed/pending Data-Quality Hygiene outbox candidates.

What remains access-gated or not production-ready:

- No live NVA/Gingr credentials, production data, provider/PMS writes, customer/member sends, payment/refund/discount actions, schedule changes, medical/safety decisions, or production deployment.
- API state is still local/in-memory for this demo; the Postgres migration/storage model is present, but a production Postgres repository adapter is future work.
- Worker leasing, retries/dead-letter handling, queue dashboards, durable traces, alerting, role/location authorization, OpenAPI export, object storage, and rollback/playbooks are still implementation gaps.

Toasty note: Toasty is only a future storage-adapter candidate to evaluate after this demo slice is stable and only if storage boilerplate becomes painful. It is not part of the current proof, and no Toasty types should enter the domain, app, or API contracts for this presentation.

## Five-minute walkthrough script

### 0:00-0:30 — Open with the product thesis

"This repo is not a chatbot demo and it is not a wrapper around Gingr. It is an entity-first operating model for Pet Resorts labor reduction. The product starts from recognizable resort work: booking triage, vaccine/document review, manager daily brief, data-quality cleanup, checkout exceptions, retention, and daily-update drafts. The design goal is to let automation summarize, rank, draft, validate, and route work while humans and approved systems of record keep live authority."

Point to:

- [README.md](../../README.md)
- [Public landing page source](../public/index.html)
- [Runtime contract boundaries](../architecture/runtime-contract-boundaries.md)

### 0:30-1:15 — Explain the source-to-action boundary

"The architecture is deliberately access-constrained. Provider data is evidence, not business truth. Provider DTOs and raw responses preserve what came from Gingr or fixtures, including unknown fields and unsupported surfaces. Only explicit mapping and validation promote that evidence into domain concepts like pet, reservation, vaccine proof, source ref, review gate, workflow event, or outcome."

Key sentence to memorize:

"Gingr DTOs are adapter evidence; the product-owned contracts live in domain/app/API/storage."

Point to:

- [Gingr provider DTO boundary](../../integrations/gingr/src/dto/README.md)
- [Gingr integration README](../../integrations/gingr/README.md)
- [Source/provider flows crosswalk](../entity-atlas/contract-crosswalk/source-provider-flows.md)

### 1:15-2:05 — Show what the platform owns

"The platform owns the semantic layer and the review workflow layer. Domain types name validated pet-resort meaning. App workflow packets package safe work for staff and agents. API DTOs expose local/demo staff-review contracts such as inquiry intake, vaccine document review, manager daily brief, and data-quality hygiene. The runtime advertises live side effects as disabled rather than pretending to be production."

What to emphasize:

- Provider DTOs are quarantined evidence.
- Domain/app packets are the product-owned workflow meaning.
- API DTOs are local staff/reviewer contracts, not raw provider JSON and not DB rows.
- AI/runtime output is untrusted until parsed, validated, and reviewed.

Point to:

- [domain README](../../domain/README.md)
- [app README](../../app/README.md)
- [API shell README](../../apps/api/README.md)
- [API handlers and DTOs](../../apps/api/src/http.rs)

### 2:05-2:55 — Explain DB, audit, and outbox safety

"The database model is where this stops being a loose demo. The migration names durable workflow events, review packets, approval records, audit events, outcome/labor records, document metadata, and outbox records. Outbox is not permission to send; it is the approved side-effect handoff after policy and review. Audit is append-only. Approval decisions require actor and timestamp fields. Open outbox rows are protected from approval demotion."

Key sentence to memorize:

"The schema tells the same story as the product: events before side effects, review before action, audit before claims."

Point to:

- [MVP migration](../../migrations/0001_mvp_foundation.sql)
- [Storage README](../../storage/README.md)
- [Storage operations source](../../storage/src/operations.rs)
- [Migration contract tests](../../storage/tests/mvp_migration_contract.rs)

### 2:55-3:40 — Explain observability without overclaiming

"The current runtime has JSON tracing startup, an `x-request-id` response/header correlation convention, safe readiness probes that describe the observability scope, and local aggregate metrics for labor outcomes plus Data-Quality Hygiene outbox posture. The Data-Quality Hygiene outcome response now carries correlation id, workflow event id, reviewed outbox candidate id, what happened, what stayed blocked, and the production next step. What is not done yet is durable trace storage, durable worker leasing, dead-letter/replay views, OpenAPI schema export, alerting, and infrastructure metrics. I would present that as local proof, not completed production observability."

Key sentence to memorize:

"Business/labor metrics and local correlation proof are modeled; production observability still needs durable traces, queue/dead-letter views, and alerting."

Point to:

- [Readiness gap map](../audits/dto-api-db-observability-readiness-gap-map.md)
- [MVP stack roadmap](../roadmap/pet-resort-mvp-stack.md)
- [API main tracing setup](../../apps/api/src/main.rs)
- [Worker runtime mode source](../../apps/worker/src/runtime.rs)

### 3:40-4:30 — Name the strongest demo story

"The strongest current story is not 'we operate live NVA systems.' It is 'we built the safe contract shape for a vertical slice.' The best five-minute demo is Data-Quality Hygiene: it carries source evidence, safe/blocked actions, agent draft context, review disposition, outcome status, local request/workflow/review/outbox correlation, and labor-minute proof while keeping live delivery disabled. That is exactly the shape a pilot would need before connecting real systems."

Suggested live explanation:

"Run `./scripts/smoke_data_quality_hygiene_local_loop.sh` and `./scripts/smoke_data_quality_hygiene_disabled_worker_outbox.sh`. The markers should show context, draft validation, blocked draft validation, outcome capture, positive minutes saved, and disabled live side effects. If I had the next implementation card after this demo, I would wire this same Data-Quality Hygiene contract from API to Postgres workflow/review/audit/outcome rows and durable queue metrics, with live customer/provider/payment side effects still disabled."

Point to:

- [Manager Daily Brief workflow](../workflows/operator/manager-daily-brief.md)
- [Data-quality hygiene workflow](../workflows/operator/data-quality-hygiene.md)
- [Manager Daily Brief app source](../../app/src/manager_daily_brief.rs)
- [Data-quality hygiene app source](../../app/src/data_quality_hygiene.rs)

### 4:30-5:00 — Close with honest readiness

"The impressive part is the discipline: instead of faking credentials or overclaiming production readiness, the repo models DTO/API/DB/observability seams that make live integration safer later. What is real now is the typed architecture, source trails, safe local/demo routes, migration spine, audit/review/outbox model, and tests. What requires access and implementation is production persistence wiring, identity/location authorization, real provider credentials, object storage, worker leasing, observability dashboards, and any live member-facing action."

Closing line:

"This is credible because it says exactly where automation stops."

## One-slide diagram: DTO/API/DB/observability boundaries

```text
Provider / staff / fixture evidence
  - Gingr DTOs, raw responses, staff forms, uploaded evidence metadata
  - Owns source shape; does not own business truth
        |
        v
Provenance + source refs
  - source system, record refs, observed timestamps, adapter/schema version
  - Every claim remains traceable to source evidence
        |
        v
Domain + app workflow packets
  - validated pet/reservation/care/vaccine/message/workflow meaning
  - review gates, blocked actions, draft packets, agent prompt packets
  - AI output is untrusted until parsed, validated, and reviewed
        |
        v
API DTOs for staff/review flows
  - health/readiness, inquiries, vaccine docs, manager brief, data-quality hygiene
  - local/demo contracts; live sends/provider writes/payments are disabled
        |
        v
Review gate
  - human or approved system-of-record approval for risky actions
  - no hidden source cleanup or medical/safety/payment/schedule decision by agent
        |
        v
DB projections + append-only audit + approved outbox handoff
  - workflow_events, review_packets, approval_records, audit_events, outcomes
  - outbox_records only after coherent approval; projections measure/replay
        |
        v
Logs, metrics, and proof
  - JSON tracing baseline, readiness posture, request/workflow correlation fields, labor/outcome records, aggregate local counters
  - production gap: durable traces, worker leasing, queue/outbox/dead-letter metrics, alerting
```

## What is real now vs what requires access or implementation

| Area | Real now in the repo | Requires access or implementation before claiming production readiness |
| --- | --- | --- |
| Product thesis | Entity-first labor-reduction model with source evidence, review gates, blocked actions, outcome/labor proof, and proof paths. | Pilot-specific ROI baselines, staff workflow adoption data, and operator-approved metric definitions. |
| Provider/Gingr boundary | Quarantined provider DTO posture; retail DTO example; unsupported grooming/training surfaces explicitly marked rather than invented. | Live NVA/Gingr credentials, broader fixture-backed DTO coverage, real provider source snapshots, redaction updates for sensitive expansions. |
| Domain/app contracts | Typed pet-resort meaning, review gates, workflow packets, agent prompt packets, safe/blocked action vocabulary. | More vertical slices connected end to end through durable storage and reviewer UI. |
| API DTOs | Safe local/demo routes for health/readiness, inquiries, vaccine documents, manager daily brief, and data-quality hygiene. | Published OpenAPI/client schema, auth/session/role/location authorization, durable repository adapter for one workflow. |
| DB/storage | Postgres migration spine for workflow events/results, review packets, approval records, outcomes, object metadata, outbox, append-only audit; storage records/codecs. | Running production DB, migration deployment process, SQLx/Postgres API wiring, object storage adapter, backup/retention policy. |
| Worker/outbox | Runtime modes are fake deterministic or disabled; side effects are stubbed; outbox table models approved handoff. | Durable job leasing, retries/dead-letter handling, approved adapter execution, operational dashboards. |
| Observability | JSON tracing startup; `x-request-id` response/header convention; health/readiness with observability scope; business/labor outcome fields, summary route, and local Data-Quality Hygiene outbox counters. | Durable trace store/export, safe error classes, queue/outbox/dead-letter dashboards, worker lease/retry metrics, alerting. |
| Live operations | Safety boundaries are explicit: no customer sends, provider/PMS writes, payments/refunds/discounts, schedule changes, medical/safety decisions, or production deployment. | Real credentials, legal/ops approval, review UI, audit retention, rollback/playbook, least-privilege scopes, and approved system-of-record paths. |

## Suggested answers to likely interview questions

### "What are your DTOs?"

"There are two kinds, and the difference is the point. Provider DTOs at the Gingr boundary preserve source evidence and unknown fields without becoming business truth. API DTOs in the Rust API are product-owned staff/reviewer contracts for safe local workflows: readiness, inquiry intake, vaccine document review, manager daily brief, and data-quality hygiene. Domain/app types sit between them so raw provider JSON does not leak directly into workflow authority."

Proof paths:

- [Provider DTO README](../../integrations/gingr/src/dto/README.md)
- [API HTTP source](../../apps/api/src/http.rs)
- [Runtime contract boundaries: API DTOs](../architecture/runtime-contract-boundaries.md)

### "What is the API story?"

"The API is a thin Rust runtime shell over app/domain workflow contracts. It exposes safe local/demo routes and readiness payloads that explicitly say live side effects, customer messaging, and provider writes are disabled. Current caveat: DTOs are private Rust structs and not yet exported as OpenAPI, so the next presentation slice should publish a checked schema."

Proof paths:

- [apps/api README](../../apps/api/README.md)
- [apps/api/src/http.rs](../../apps/api/src/http.rs)
- [API DTO contract test](../../apps/api/tests/api_dto_contracts.rs)

### "What is the database story?"

"The database is not a provider mirror. It is the durable workflow/audit/review/outcome model: workflow events/results, review packets, approval records, document/object metadata, outcome/labor tables, outbox records, and append-only audit events. Constraints enforce review and approval coherence. Current caveat: API handlers still use deterministic in-memory state; wiring one workflow to Postgres is the obvious next slice."

Proof paths:

- [MVP migration](../../migrations/0001_mvp_foundation.sql)
- [Migration contract tests](../../storage/tests/mvp_migration_contract.rs)
- [Storage operations source](../../storage/src/operations.rs)
- [Readiness gap map: DB migration spine](../audits/dto-api-db-observability-readiness-gap-map.md)

### "What is the logging and metrics story?"

"There is a local proof, not a finished platform. API and worker startup use JSON tracing; the API echoes/creates `x-request-id`; readiness states the local observability scope; Data-Quality Hygiene context/outcome responses carry request/correlation/workflow/outbox ids; and `/ops/metrics/summary` reports aggregate labor plus local outbox posture. Missing work is durable trace export/storage, safe error classes, queue/outbox/dead-letter dashboards, worker lease/retry metrics, and alerting. I would not claim production observability yet."

Proof paths:

- [apps/api/src/main.rs](../../apps/api/src/main.rs)
- [apps/worker/src/main.rs](../../apps/worker/src/main.rs)
- [Worker runtime source](../../apps/worker/src/runtime.rs)
- [Readiness gap map: observability](../audits/dto-api-db-observability-readiness-gap-map.md)

### "Is this production-ready?"

"No, and that is an important part of the story. It is CI/PR-ready or demo-ready as an architecture artifact: typed contracts, local safe routes, migration spine, audit/outbox constraints, documentation, and tests. It is not production-ready until real access, identity/location authorization, durable API persistence, worker leasing, object storage, monitoring, and explicit approval for live side effects exist."

### "What would you build next?"

"I would take the current Data-Quality Hygiene demo slice from local/in-memory proof to durable proof: API request -> Postgres workflow event/review/audit/outcome rows -> fake or disabled worker result -> reviewed internal outbox candidate -> correlated logs, local metrics, and eventually durable queue/dead-letter views. I would keep all customer sends, provider writes, schedule changes, payments, and medical decisions disabled until the approval path and system-of-record authority are real. If storage boilerplate becomes the bottleneck after this slice, Toasty is worth a storage-adapter spike only; it is not part of the current proof and should not reshape domain/app/API contracts."

## Source, Rustdoc, and test proof paths

Start with narrative/proof docs:

- [README canonical path](../../README.md)
- [Runtime contract boundaries](../architecture/runtime-contract-boundaries.md)
- [DTO/API/DB/observability readiness gap map](../audits/dto-api-db-observability-readiness-gap-map.md)
- [MVP stack roadmap](../roadmap/pet-resort-mvp-stack.md)
- [Public landing page source](../public/index.html)

Source paths to cite:

- [Provider DTOs](../../integrations/gingr/src/dto/README.md)
- [Domain crate README](../../domain/README.md)
- [App crate README](../../app/README.md)
- [API routes and DTOs](../../apps/api/src/http.rs)
- [API tracing startup](../../apps/api/src/main.rs)
- [Worker runtime modes](../../apps/worker/src/runtime.rs)
- [MVP migration](../../migrations/0001_mvp_foundation.sql)
- [Storage operations](../../storage/src/operations.rs)

Tests and verification proof:

- [API DTO contract test](../../apps/api/tests/api_dto_contracts.rs)
- [Worker runtime contract test](../../apps/worker/tests/runtime_mode_contract.rs)
- [Storage migration contract test](../../storage/tests/mvp_migration_contract.rs)
- [Staff dashboard smoke test](../../apps/staff-web/smoke/staff-dashboard-mvp.test.mjs)

Generated Rustdoc proof after `./scripts/check_docs.sh` or `cargo doc --workspace --no-deps`:

- `target/doc/domain/index.html`
- `target/doc/app/index.html`
- `target/doc/storage/index.html`
- `target/doc/gingr/index.html`
- `target/doc/pet_resort_api/index.html`
- `target/doc/pet_resort_worker/index.html`

## Presenter caveats to keep the story honest

- Do not say "integrated with NVA/Gingr production." Say "provider boundary is modeled; live credentials and production access are not present."
- Do not say "agents can contact customers." Say "agents can draft/recommend inside app-owned packets; sends remain review-gated and disabled locally."
- Do not say "the DB is live behind the API." Say "the migration and storage contracts are present; current API handlers are deterministic in-memory demo state pending a Postgres repository adapter."
- Do not say "observability is complete." Say "local JSON tracing, request/workflow correlation fields, readiness posture, outcome records, and aggregate counters are present; durable traces, queue/dead-letter views, and alerting are next."
- Do not say "production-ready." Say "presentation-ready architecture and local/demo contract proof; production requires access, approval, durable wiring, identity, monitoring, and rollback."
