# Runtime contract boundaries: DTOs, domain packets, database projections, and review-safe side effects

Status: presentation-grade architecture explainer for the current repo. This page summarizes source-grounded boundaries; it does not claim live NVA/Gingr credentials, production data, customer-message authority, provider/PMS write permission, payment authority, or production deployment.

Audience thesis: this product does not mirror Gingr. Gingr/provider records are one source of evidence. The product owns the semantic domain model, app workflow packets, staff/review API contracts, durable database projections, approval/outbox gates, and traceability records that make access-constrained development safe.

Use this page when a technical evaluator asks, "If you did not have live provider access, what exactly did you build?" The answer is: a review-gated workflow platform boundary that can ingest evidence safely, preserve provenance, expose owned DTOs, persist audit/review/outcome records, and keep side effects disabled until an approved system-of-record path exists.

## Five-minute mental model

```text
provider / staff / fixture evidence
  -> provider DTO or raw response envelope
  -> provenance and source refs
  -> semantic domain types
  -> app workflow packet / draft / result
  -> API DTO for staff or reviewer
  -> human or system-of-record review gate
  -> DB storage projection + append-only audit record
  -> approved outbox handoff only after policy/review allows it
  -> outcome, labor proof, logs, metrics, and traceability
```

The direction matters. Data can be promoted from evidence into meaning only through explicit validation/mapping. Data can be demoted into storage/API shapes only when a boundary needs to persist or expose it. No boundary may silently turn raw provider JSON, AI output, or a local demo DTO into customer-facing action.

## Boundary ownership at a glance

| Boundary | Owns | Does not own | Current source evidence |
| --- | --- | --- | --- |
| Provider DTOs | Source payload shape, provider ids, unknown-field retention, explicit unsupported-surface markers | NVA business truth, canonical IDs, review decisions, durable workflow authority | [`integrations/gingr/src/dto/README.md`](../../integrations/gingr/src/dto/README.md), [`integrations/gingr/README.md`](../../integrations/gingr/README.md) |
| Domain types | Business meaning after validation: customers, pets, reservations, services, source refs, review gates, workflow decisions, audit ids | HTTP JSON shape, provider payload shape, table layout, live side effects | [`domain/README.md`](../../domain/README.md), [`domain/src/lib.rs`](../../domain/src/lib.rs) |
| App workflow packets | Review-gated work units, deterministic checks, prompt packets, draft artifacts, blocked/safe actions, tool-port contracts | Canonical domain truth, provider transport, persistence schema, customer sends | [`app/README.md`](../../app/README.md), [`app/src/agents.rs`](../../app/src/agents.rs), [`app/src/tools.rs`](../../app/src/tools.rs) |
| API DTOs | Staff/reviewer request and response contracts for safe demo routes and local workflow review | Direct domain authority, durable persistence guarantees, provider writes, customer sends | [`apps/api/src/http.rs`](../../apps/api/src/http.rs), [`apps/api/README.md`](../../apps/api/README.md) |
| DB tables / storage projections | Durable workflow, review, outcome, object-metadata, outbox, and append-only audit records | Live decision authority, raw-provider mirroring, policy invention | [`migrations/0001_mvp_foundation.sql`](../../migrations/0001_mvp_foundation.sql), [`storage/README.md`](../../storage/README.md) |
| Outbox | Approved side-effect candidates and execution handoff state | Permission to execute unapproved live customer/provider/payment actions | [`migrations/0001_mvp_foundation.sql`](../../migrations/0001_mvp_foundation.sql#L359), [`docs/roadmap/pet-resort-mvp-stack.md`](../roadmap/pet-resort-mvp-stack.md#8-queue-event-runtime-and-idempotency) |
| Logs, metrics, audit | Traceability, correlation, labor proof, safe operational debugging | Raw secrets, raw documents, provider JSON dumps, hidden prompts, unreviewed action authority | [`docs/roadmap/pet-resort-mvp-stack.md`](../roadmap/pet-resort-mvp-stack.md#12-observability-audit-and-operations), [`docs/audits/dto-api-db-observability-readiness-gap-map.md`](../audits/dto-api-db-observability-readiness-gap-map.md) |

## Provider DTOs = source evidence

Provider DTOs are intentionally narrow and quarantined. The Gingr adapter currently treats `gingr::dto::retail::Item` and `gingr::dto::retail::ItemId` as provider-shaped retail catalog evidence, retains unknown fields as drift evidence, and records unsupported grooming/training service DTOs as `ProviderSurface::NoDocumentedServiceDto` instead of inventing payloads.

That posture is a product feature, not a gap:

- Provider ids remain provider-scoped; they are not canonical `domain::*` ids.
- Optional provider fields stay optional until mapping proves a semantic value.
- Unknown fields show source drift; they do not authorize arbitrary business interpretation.
- Unsupported provider surfaces stay visible as gaps so automation cannot pretend to know what it does not know.
- Sensitive provider expansions would require redaction/sensitivity updates before logs, prompts, API DTOs, or review packets expose them.

This is how lack of live provider access is handled safely: fixture-backed or documented provider evidence can enter the adapter boundary, but unverified shapes stop there. The repo does not fake complete Gingr authority just to make a demo look broader.

## Domain types = business meaning

The `domain` crate is the semantic core. It names the pet-resort business concepts after validation: location/customer/pet/reservation/document/vaccine/message/incident aggregates, source provenance and record refs, workflow events/results, review gates, approval records, payment/deposit policy facts, care/vaccine/incident classifications, audit ids, and agent specifications.

Domain types are deliberately not the same thing as provider DTOs, API JSON, or database rows. For example:

- `domain::source::Provenance` and `domain::source::RecordRef` say where a fact came from and how it was observed.
- `domain::policy::ReviewGate` says which approval boundary must stop automation.
- `domain::workflow::Event` and `domain::workflow::Result<T>` say what work happened and how a typed workflow result should be interpreted.
- `domain::entities::approval::Record` and `domain::audit::EventId` keep review and audit concepts first-class.

Promotion into domain meaning should be explicit. A raw provider field, API string, storage code, or AI output becomes business meaning only after the mapping/validation boundary says so.

## App workflow packets = review-gated work units

The `app` crate turns domain meaning into reviewable work. It owns request packets, evaluation/result packets, draft artifacts, audit-event drafts, safe/blocked action enums, agent prompt packets, and tool-port contracts.

Representative packet families include:

- Booking triage: deterministic rule evaluations, `StaffEvaluationPacket`, safe draft actions, and blocked provider/PMS mutations.
- Checkout completion: source/status/handoff evaluation, `Packet`, and audit-event drafts without provider or payment changes.
- CRM retention: contact permission evidence, source-grounded opportunities, draft-only follow-up, and outcome records.
- Daily update/Pawgress: customer-safe draft text, review disposition, included/omitted facts, approval records, send stubs, and audit logs.
- Manager daily brief: source-grounded action packets, labor-minute estimates, feedback outcomes, and reporting groups.
- Shared AI boundary: `AgentPromptPacket<T>` and `WorkflowAgent<Input, Output>` use domain agent/workflow contracts before any model runtime sees a task.

The app layer is the reason an LLM/runtime is not a system of record. AI receives bounded packets and returns untrusted output; the app validates the result, records review needs, and leaves side effects behind policy/review gates.

## API DTOs = staff/review workflow contracts

The Rust API shell exposes owned request/response DTOs for safe staff workflows. These are not Gingr DTOs and not raw database rows; they are local API contracts for review work.

Current safe route families in `apps/api/src/http.rs` include:

- Health/readiness DTOs that explicitly report live side effects, customer messaging, and provider writes as disabled.
- Inquiry intake DTOs that parse lead facts, draft a follow-up, create a review task, and record audit-shaped events without sending to customers.
- Vaccine/document review DTOs that handle upload metadata, extraction records, review packets, approval/rejection decisions, eligibility projections, and audit events without autonomous medical acceptance.
- Manager daily brief DTOs that build context, accept drafts, and capture reviewed outcome/labor minutes.
- Data-quality hygiene DTOs that capture reviewed data-quality outcomes and summary counts rather than trusting provider/source data blindly.

Presentation caveat from the readiness audit: these DTOs are currently Rust structs inside the API shell, not yet an exported OpenAPI/client schema. That is a schema-publishing gap, not evidence that the product is provider-shaped.

### API repository cutline: in-memory today, Postgres adapter later

`apps/api/src/http.rs` now names a `WorkflowRepository` seam around the responsibilities that had been implicit in `VaccineDocumentState`: workflow/runtime counters, inquiry intake records, review queues, manager daily-brief outcomes, data-quality hygiene outcomes, and the review/audit/document projections used by the vaccine document workflow. The default adapter remains deterministic in-memory state so local demos and contract tests do not require pretending a production Postgres database is live.

The readiness DTO exposes the same posture under `workflow_repository`:

- `active_adapter = "in_memory"` means the API shell is still local/demo state; readiness does not promote Postgres to an active adapter from environment variables alone.
- `postgres_adapter = "planned_same_contract"` means no database URL is configured yet but the durable adapter should implement the same handler-facing contract, not force route rewrites.
- `postgres_adapter = "env_configured_not_verified"` means a database URL is present for read models, but readiness has not executed a connectivity/query probe; the read-model endpoint itself reports `database.status = "connected"` only after its query succeeds.
- `contract = ["workflow_events", "review_packets", "audit_events", "outcomes", "documents"]` names the durable surface area that maps to the migration spine.

A future SQLx/Postgres implementation should sit behind this seam and persist the same semantic records to `workflow_events`, `workflow_results`, `review_packets`, `approval_records`, `audit_events`, outcome tables, document/object metadata, and outbox candidates. It must keep the same safety posture: no live customer sends, provider/PMS writes, medical acceptance, payments, schedule changes, or production claims without explicit approval/outbox gates.

## DB tables / storage projections = durable workflow, audit, and outcome records

The Postgres migration and storage crate provide the durable record model. They do not mirror every provider payload. They persist normalized projections, workflow/review records, evidence metadata, outcome/labor facts, and audit trails.

The current migration spine includes:

- Identity and scope: `locations`, `customers`, `pets`, `reservations`, `reservation_pets`.
- Evidence and medical review: `documents`, `object_metadata`, `vaccine_records`, `vaccine_extractions`, `pet_eligibility_projections`.
- Operations and communication projections: `operational_tasks`, `care_notes`, `incidents`, `messages`, `payment_deposit_projections`.
- Workflow and review: `workflow_events`, `workflow_results`, `review_packets`, `approval_records`.
- Owned outcome/labor projections: `manager_daily_brief_outcomes` and `data_quality_hygiene_outcomes`, keyed back to workflow events and approval records so labor evidence remains review-scoped.
- Side-effect and audit safety: `outbox_records`, `audit_events`.

The schema already encodes high-risk states as constraints, centralizes review gates, requires decision actor/timestamp fields for approved/rejected approval records, requires outbox rows to reference a matching approved approval record for the same aggregate and review gate, guards open outbox rows from approval demotion, and makes `audit_events` append-only through mutation-rejecting triggers.

The migration intentionally does not claim the next DB surfaces: auth/session/role/location authorization, infra metrics snapshots/dashboard read models, durable job leasing/worker ownership, or real provider source snapshots. Those remain product surfaces to implement after one safe workflow is wired durably.

`storage` complements the migration by naming storage records, stable code values, codecs, and promotion/demotion paths. Storage projections can support reporting and review; they do not decide bookings, payments, provider writes, staffing, eligibility, or customer sends.

## Outbox = approved side-effect handoff

The outbox boundary is where future side effects become trackable work after approval. It is not a loophole around review.

A safe side-effect path is:

```text
workflow packet recommends or drafts an action
  -> policy marks the action blocked or review-required
  -> reviewer/system-of-record approval is recorded
  -> outbox record is created with aggregate, topic, payload, status, and retry fields
  -> worker or adapter claims/publishes only within approved configuration
  -> audit/outcome records preserve what happened
```

For the current access-constrained artifact, live customer sends, provider/PMS writes, payment movement, schedule changes, vaccine acceptance, incident closure, and production deployment remain disabled or out of scope. The outbox proves where an approved handoff would live; it does not prove that live execution is enabled.

## Logs, metrics, and audit = traceability and labor proof

Traceability has two layers:

1. Durable audit and outcome records: `audit_events`, `workflow_events`, `workflow_results`, `review_packets`, `approval_records`, outcome records, labor-minute fields, source refs, and reporting groups.
2. Operational observability: structured JSON logs, request/job correlation, health/readiness, dead-letter/replay/admin views, and later metrics/traces.

The API and worker currently initialize JSON tracing and expose safe runtime probes. The roadmap requires a fuller observability spine: request ID, correlation ID, workflow event ID, job ID, actor kind/id, location ID, subject ref, safe error class, queue/dead-letter visibility, and redaction rules that keep raw documents, raw provider JSON, payment payloads, secrets, and hidden prompts out of ordinary logs.

Labor proof belongs in reviewed outcome records, not in marketing claims. Manager daily brief and data-quality hygiene surfaces already model outcome status, source-fact correctness, reporting groups, and labor minutes. The next readiness slice is wiring those records through durable Postgres and correlated logs instead of keeping them only in an in-memory demo shell.

## End-to-end flow with safety stops

```text
1. Provider/staff/fixture input
   - Gingr DTO, general raw response record, staff form, local fixture, uploaded evidence metadata.
   - Boundary: raw evidence, not business truth.

2. DTO/raw response preservation
   - Provider ids, optional fields, unknown fields, and unsupported surfaces stay visible.
   - Boundary: no inferred semantics from undocumented fields.

3. Provenance and source refs
   - Source system, endpoint, record ref, observed timestamp, adapter/schema version, evidence ref, data-quality issue.
   - Boundary: every downstream claim should be explainable from source evidence.

4. Domain/app promotion
   - Validated values enter `domain::*`; app workflows assemble packets, drafts, review gates, safe actions, blocked actions, and agent prompt packets.
   - Boundary: AI/runtime output remains untrusted until parsed and validated.

5. API review contract
   - Staff/reviewer DTOs expose queues, packets, drafts, decisions, outcome capture, and disabled live side-effect posture.
   - Boundary: frontend/API callers request work; backend policy/app/domain decide whether it is allowed, blocked, or review-required.

6. Review gate
   - Human or approved system-of-record role decides manager approval, medical document review, behavior review, customer-message approval, refund/deposit exception, or other policy gates.
   - Boundary: no live customer/provider/payment/schedule action before the right approval record exists.

7. DB/audit/outbox/outcome
   - Workflow events/results, review packets, approval records, audit events, storage projections, object metadata, outcome/labor records, and approved outbox candidates are persisted.
   - Boundary: projections measure and replay; they do not invent authority.

8. Observability and labor proof
   - Correlated logs, audit rows, safe summaries, dead-letter/replay views, outcome statuses, and labor-minute records prove what happened and why.
   - Boundary: logs and metrics must redact sensitive raw content and show disabled/stubbed modes honestly.
```

## Why access constraints were handled safely

The repo is intentionally honest about what is present:

- It does not claim live NVA/Gingr credentials or production data.
- It does not invent undocumented provider DTOs to fill demos.
- It does not let provider ids replace domain ids.
- It does not persist raw provider payloads as if they were canonical business records.
- It does not enable live customer messaging, provider/PMS writes, payment actions, vaccine acceptance, incident closure, or production deployment.
- It does expose a typed product boundary: provider evidence -> provenance/source refs -> domain/app contracts -> API review DTOs -> DB audit/review/outcome/outbox records.

That is the architecture story: lack of access did not produce hand-waving. It produced explicit seams, proof paths, and disabled side-effect gates so the next implementation slice can wire one workflow durably without weakening review safety.

## Current proof and remaining readiness gaps

Strong current proof:

- Source-grounded boundary docs: [`README.md`](../../README.md), [`docs/audits/dto-api-db-observability-readiness-gap-map.md`](../audits/dto-api-db-observability-readiness-gap-map.md), and [`docs/roadmap/pet-resort-mvp-stack.md`](../roadmap/pet-resort-mvp-stack.md).
- Provider DTO quarantine and mapping discipline in `integrations/gingr`.
- Domain/app packet surfaces in `domain` and `app`.
- API DTO route families in `apps/api/src/http.rs`.
- Postgres migration spine with workflow/review/outbox/audit tables.
- Storage projections and source-reference boundaries in `storage`.
- JSON tracing startup and disabled live-side-effect health/readiness posture.

Remaining readiness gaps to call out honestly:

1. Durable API persistence has only a narrow read-model query path so far; most mutable API state intentionally remains in-memory for demo determinism until each workflow is promoted through reviewed Postgres repository ports.
2. API DTOs are not yet exported as OpenAPI/client schema.
3. Cross-cutting request/job correlation is not yet implemented end to end.
4. Worker durable leasing/outbox processing remains a shell.
5. Object storage is modeled but not yet wired as the durable evidence adapter.
6. Infra metrics/dead-letter/admin surfaces are not yet presentation-complete.

Recommended next implementation slice: choose one safe workflow, such as inquiry intake or vaccine/document review, and wire API -> Postgres workflow/review/audit rows -> fake/disabled worker result -> reviewed outcome/outbox candidate -> correlated logs, with all live customer/provider/payment side effects disabled.
