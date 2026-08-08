# DTO/API/DB/observability readiness gap map

Status: current repo audit for presentation readiness. This is a source-grounded inventory, not an implementation card and not a claim of live NVA, Gingr, customer, payment, or production access.

Audience thesis: the repo is strongest when it shows that we own the typed workflow/API/DB/observability layer around pet-resort labor reduction. Gingr and other provider systems are evidence inputs; they are not the product strategy, not the domain model, and not an authority to mutate live operations.

## Source trail read for this audit

- [README.md](../../README.md) names the repo as an entity-first operating model for labor-cost reduction and says provider evidence must flow into app/domain contracts, review gates, outcome/labor-minute evidence, and only then storage/runtime proof.
- [docs/public/index.html](../public/index.html) presents the same public front door: source-grounded workflow automation with app-owned packets and review gates, not a free-form chatbot or live side-effect executor.
- [docs/roadmap/pet-resort-mvp-stack.md](../roadmap/pet-resort-mvp-stack.md) recommends the target MVP shape: Rust API, PostgreSQL, Postgres-backed workflow inbox/jobs/outbox, S3-compatible evidence storage, JSON tracing, health/readiness, request/job correlation, and AI behind typed app packets.
- [migrations/0001_mvp_foundation.sql](../../migrations/0001_mvp_foundation.sql) is the current durable schema spine.
- [storage/tests/mvp_migration_contract.rs](../../storage/tests/mvp_migration_contract.rs) proves the migration contains core tables, review gates, workflow/outbox/audit paths, append-only audit triggers, and approval integrity checks.
- [apps/api/src/http.rs](../../apps/api/src/http.rs), [apps/api/src/main.rs](../../apps/api/src/main.rs), [apps/worker/src/main.rs](../../apps/worker/src/main.rs), and [apps/worker/src/runtime.rs](../../apps/worker/src/runtime.rs) are the current runtime/API shells.
- [integrations/gingr/src/dto/README.md](../../integrations/gingr/src/dto/README.md) documents provider DTO quarantine boundaries.
- [storage/src/operations.rs](../../storage/src/operations.rs) documents persisted business/labor projection records and source-reference boundaries.

## What is already owned and presentation-ready

### 1. Owned API DTOs currently present and workflow served

These DTOs live in [apps/api/src/http.rs](../../apps/api/src/http.rs). They are local/demo API contracts owned by this repo; they expose app/domain/storage-shaped workflow evidence and deliberately do not call live customer messaging, provider writes, payments, or production systems.

| Surface | DTOs / routes | Workflow served | Presentation value |
| --- | --- | --- | --- |
| Runtime probes | `HealthPayload` via `GET /healthz`; `ReadinessPayload` via `GET /readyz` | Shows local API liveness/readiness and explicit disabled live side-effect posture | Good demo anchor: the API advertises `live_side_effects`, `live_customer_messaging`, and `live_provider_writes` as disabled rather than implying production authority. |
| Inquiry intake | `InquirySubmissionRequest`, `InquiryCustomerRequest`, `InquiryPetRequest`, `InquiryDateWindowRequest`, `InquiryIntakeRecord`, `InquiryEvent`, `ParsedInquiryLead`, `InquiryDraftReply`, `InquiryTask`, `InquiryAuditEvent`, `InquiryStaffQueuePayload`; routes `POST /inquiries`, `GET /staff/inquiries` | Staff-entered/local inquiry intake: parse lead facts, draft a reply, create a review task, and record audit-shaped events | Shows the “app-owned intake packet + review draft” story without customer sends. Good next bridge to durable workflow events. |
| Vaccine/document review | `VaccineDocumentUploadRequest`, `VaccineReviewDecisionRequest`, `VaccineDocumentWorkflowPayload`, `DocumentRecord`, `VaccineExtractionRecord`, `VaccineRecord`, `ReviewPacket`, `ApprovalRecord`, `PetEligibility`, `AuditEvent`; routes under `/vaccine-documents/...` | Vaccine proof upload/extraction/review packet/approval/rejection/eligibility projection | Strong safety story: medical-document uncertainty remains review-gated and eligibility changes are represented as local projection evidence, not provider writes. |
| Manager daily brief | request/response DTOs around `/agent/context/manager-daily-brief`, `/agent/drafts/manager-daily-brief`, and `/manager-daily-brief/actions/{action_id}/outcome` | Builds manager action context, accepts an agent draft, captures reviewed action outcome/labor minutes | Strongest “labor-cost” presentation surface because it ties workflow actions to outcome capture and reporting dimensions. |
| Data-quality hygiene | request/response DTOs around `/agent/context/data-quality-hygiene`, `/agent/drafts/data-quality-hygiene`, `/data-quality-hygiene/actions/{action_id}/outcome`, and `/data-quality-hygiene/outcomes/summary` | Builds source-quality review context, accepts draft findings, captures reviewed resolution/suppression/wrong-source outcomes and summary | Strong proof that provider/source data is evidence to review, not blind business truth. |

Current caveat: these API DTOs are private Rust structs inside the API shell rather than a generated OpenAPI/client contract. That is fine for a local demo, but it is a presentation gap for “API ownership” if an interviewer expects a published schema.

### 2. Provider DTOs and why they remain quarantined evidence

Provider DTOs are documented in [integrations/gingr/src/dto/README.md](../../integrations/gingr/src/dto/README.md), and the current supported family is intentionally narrow:

- `gingr::dto::retail::ItemId` and `gingr::dto::retail::Item` preserve a Gingr-scoped retail catalog payload: provider item id, optional name, optional SKU, optional category/retail category, optional active flag, optional quantity on hand, and retained unknown fields.
- `gingr::dto::ProviderSurface` records known provider surfaces when no safe/documented DTO exists.
- `gingr::dto::grooming::provider_surface()` and `gingr::dto::training::provider_surface()` intentionally return `NoDocumentedServiceDto` for `get_services_by_type` rather than inventing grooming/training service DTOs.
- General raw provider records such as owner, animal, reservation, and reference records live outside `dto` in `gingr::response` and are mapped only when `gingr::mapping` can safely promote fields.

Why quarantine matters:

1. Provider ids are Gingr-scoped source identifiers, not canonical `domain::*` ids.
2. Optional and unknown provider fields stay evidence until mapping proves semantic meaning.
3. Unknown-field retention is drift evidence, not permission to infer business semantics from arbitrary JSON.
4. Unsupported grooming/training surfaces are explicit gaps, preventing false automation confidence.
5. DTOs that grow customer, payment, care, medical, staff, notes, or custom-field content must update sensitivity/redaction docs before being exposed in logs or review packets.

Presentation framing: “We can ingest or fixture provider evidence safely, but our product contracts live in domain/app/API/storage. Gingr DTOs are adapter evidence, not workflow authority.”

### 3. Current DB migration spine and what it proves

[migrations/0001_mvp_foundation.sql](../../migrations/0001_mvp_foundation.sql) creates the current Postgres MVP foundation. It already covers these table families:

- Location and core identity: `locations`, `customers`, `pets`, `reservations`, `reservation_pets`.
- Evidence and medical review: `documents`, `object_metadata`, `vaccine_records`, `vaccine_extractions`, `pet_eligibility_projections`.
- Operations and communications: `operational_tasks`, `care_notes`, `incidents`, `messages`, `payment_deposit_projections`.
- Workflow and review: `workflow_events`, `workflow_results`, `review_packets`, `approval_records`.
- Owned outcome/labor projections: `manager_daily_brief_outcomes` and `data_quality_hygiene_outcomes`, keyed back to workflow events and approval records.
- Side-effect and audit safety: `outbox_records`, `audit_events`.

Important constraints already encoded:

- Review gates are centralized by `review_gate_is_valid()` and `review_gates_are_valid()` with canonical gates for manager approval, medical document review, behavior review, customer message approval, and refund/deposit exceptions.
- High-risk enums are represented as check constraints for service, reservation status/source, document classification/source/scan/redaction/verification, vaccine status, incident category/severity/status, message direction/status/channel, workflow result status, review packet status, approval status, and outbox status.
- Approval decisions require deciding actor kind, actor id, and timestamp when approved/rejected, so a bare status flip cannot masquerade as review proof.
- Outbox rows require an idempotency key, coherent status/timestamp state, and a matching approved approval record for the same aggregate and review gate before any future worker can treat them as side-effect candidates; open pending/claimed outbox rows also block approval demotion or retargeting.
- Manager daily-brief and data-quality hygiene outcome tables keep labor-minute and source-reference evidence as product-owned projections rather than relying on provider mirrors or in-memory-only results.
- Audit is append-only through update/delete rejection triggers.
- Outbox, workflow event/result, approval, and audit tables are present, so the schema already tells the “events before side effects, review before action” story.

[storage/tests/mvp_migration_contract.rs](../../storage/tests/mvp_migration_contract.rs) currently proves the presence of core tables, canonical review gates, audit/outbox/workflow write paths, append-only audit triggers, approval integrity, outbox approval/idempotency constraints, owned outcome/labor projections, and explicit deferred DB surfaces. It is not a live database migration test yet; it is a source contract test over the SQL text.

Explicitly deferred DB surfaces: auth/session/role/location authorization, infra metrics snapshots/dashboard read models, durable job leasing/worker ownership, and real provider source snapshots. Naming them keeps the current schema honest: it is a product workflow/audit spine, not a fake provider mirror or a claim of production runtime completeness.

### 4. Current in-memory/demo runtime caveat

The API runtime currently uses `VaccineDocumentState`, an `Arc<Mutex<...>>` in-memory store in [apps/api/src/http.rs](../../apps/api/src/http.rs). That store holds documents, extractions, vaccine records, review packets, approvals, eligibility projections, manager daily-brief outcomes, data-quality hygiene outcomes, inquiry records, and audit events.

This is useful for deterministic demo routes and tests because it keeps all side effects local. It is not production persistence and does not exercise the Postgres migration spine. Data disappears on process restart, there is no durable transaction boundary, no leased job processing, and no real object storage dependency.

The worker is also intentionally a shell: [apps/worker/src/main.rs](../../apps/worker/src/main.rs) logs startup and says durable leasing is downstream work, while [apps/worker/src/runtime.rs](../../apps/worker/src/runtime.rs) only selects `FakeDeterministic` or `Disabled` agent runtime modes with `SideEffectMode::Stubbed`.

Presentation framing: “The demo proves contract shape and safety posture locally; the next slice is wiring these contracts to the Postgres schema and durable worker loop.”

### 5. Logging/tracing baseline and missing request/job correlation

Baseline present:

- [apps/api/src/main.rs](../../apps/api/src/main.rs) initializes `tracing_subscriber::fmt().json()` with an env filter defaulting to `pet_resort_api=info,tower_http=info` and logs the bound API address.
- [apps/api/src/http.rs](../../apps/api/src/http.rs) installs HTTP trace/request-id middleware, echoes a safe `x-request-id` response header, adds `http.route`, `http.request_id`, and disabled payload-logging posture to request spans, and includes request/correlation evidence in workflow DTO metadata where the local shell has a correlation id.
- [apps/worker/src/main.rs](../../apps/worker/src/main.rs) initializes JSON tracing with default `pet_resort_worker=info` and logs `agent_runtime_mode` and `side_effect_mode` at startup.
- Health/readiness endpoints exist and explicitly state disabled live side effects.

Gap against [docs/roadmap/pet-resort-mvp-stack.md](../roadmap/pet-resort-mvp-stack.md): the recommended MVP instrumentation calls for structured JSON logs with request ID, correlation ID, workflow event ID, job ID, actor kind/id, location ID, subject ref, and safe error class. The current shell has request-id tracing, but it still does not propagate one correlation model across all inquiry/draft/outcome handlers, attach workflow event/job ids to durable spans, carry actor/location/subject refs consistently, or create dead-letter/replay/admin metrics.

Nuance: some DTOs already carry correlation-like fields, especially manager daily-brief draft/outcome requests, and API route spans now have request-id evidence. That proves the vocabulary exists, but it is not yet a durable end-to-end observability spine.

### 6. Business/labor metrics modeled vs infra metrics missing

Business/labor metrics already modeled:

- [storage/src/operations.rs](../../storage/src/operations.rs) defines manager daily-brief outcome codes (`completed`, `deferred`, `suppressed_by_manager`, `source_fact_was_wrong`), personas, action kinds, reporting groups, and labor-minute fields.
- The same storage module defines data-quality hygiene outcome records and labor-minute fields, keeping source-fact correctness and avoided/reworked labor visible.
- API handlers keep manager daily-brief and data-quality hygiene outcomes in memory and expose a data-quality hygiene outcome summary route.
- The migration includes workflow events/results, review packets, approvals, messages, tasks, outbox, and audit surfaces that can later correlate outcomes to reviewed workflow actions.

Infra/operations metrics still missing:

- Request latency/error counters by route and safe error class.
- Queue/job metrics: lease age, attempt count distribution, retry/dead-letter rate, job duration, and stuck job count.
- AI/runtime metrics: validation failure rate, model/provider config reference, prompt manifest hash coverage, review-needed rate, disabled/fake/runtime mode counts.
- Persistence health: DB migration version, audit write failure rate, outbox backlog, object storage scan/quarantine backlog.
- Observability plumbing: OpenTelemetry traces or an equivalent span model, request/job correlation propagation, and dashboard/admin read models for dead-letter/replay.

## Ranked readiness gaps

| Rank | Gap | Why it matters for the “we own DTO/API/DB/observability” story | Suggested next implementation slice |
| --- | --- | --- | --- |
| P0 | Durable API persistence is not wired to the migration spine | The schema is strong, but current handlers persist in memory. An interviewer can still ask whether DB ownership is theoretical. | Add Postgres repository wiring for one vertical slice: inquiry intake or vaccine document review writes `workflow_events`, `review_packets`, `audit_events`, and local records transactionally. |
| P0 | Cross-cutting request/job correlation is incomplete | Observability is a named MVP requirement and the current JSON logs do not yet prove traceability from HTTP request to workflow event/job/audit row. | Add request-id/correlation middleware, span fields, DTO propagation, and tests/log assertions for one route. |
| P1 | API contracts are not exported as OpenAPI/client schema | Owned DTOs exist, but they are private Rust structs. Presentation is easier if staff UI/interviewer can inspect a stable contract. | Add an OpenAPI generation or checked schema artifact for current safe routes, explicitly marking live side effects disabled. |
| P1 | Worker durable leasing is a shell | The architecture depends on Postgres-backed jobs/outbox; current worker only logs config and awaits shutdown. | Implement minimal job lease loop over `workflow_events`/job table for one fake deterministic workflow, with dead-letter/audit records. |
| P1 | Runtime and storage are not connected to object storage | Document DTOs model bucket/key/hash/scan/redaction, but local upload uses in-memory content-derived metadata rather than MinIO/S3 object metadata. | Add local MinIO/object metadata adapter behind safe test config, keeping raw document content out of ordinary logs. |
| P2 | Infra metrics/dashboard surfaces are missing | Business labor metrics are modeled, but platform health metrics are not yet visible. | Add read-only admin endpoints for queue/dead-letter/outbox/audit summaries; then add counters/traces. |
| P2 | Provider DTO coverage remains intentionally sparse | Quarantine is good, but unsupported grooming/training DTOs limit demo breadth for service-line data. | Collect fixture-backed provider evidence and add DTO/mapping only where documentation supports stable semantics. |
| P2 | Auth/session/role scope is roadmap-level, not runtime | Staff actor ids appear in DTOs, but there is no session middleware or role/location authorization layer. | Add staff session/actor extraction and location scope checks before widening demo routes. |

## Highest-leverage next board slices

1. **Durable inquiry-intake vertical slice.** Take `POST /inquiries` from in-memory demo to Postgres-backed `workflow_events` + `operational_tasks`/message draft/review packet + `audit_events`, with all live sends disabled. This is the clearest product/API/DB ownership slice.
2. **Correlation middleware and audit linkage.** Add request id + correlation id extraction/generation, attach it to tracing spans, response headers, DTO audit fields, and `audit_events.workflow_event_id` where applicable.
3. **One worker lease loop.** Implement the smallest Postgres-backed worker that leases one workflow job, runs `FakeDeterministic`/`Disabled`, writes `workflow_results`, audit, and dead-letter state, and never performs live side effects.
4. **OpenAPI/schema artifact for safe routes.** Publish current owned API DTOs as a generated or checked schema so the staff UI and interview story can point to explicit app-owned contracts.
5. **Vaccine document storage adapter.** Connect document metadata to local object storage/MinIO with hash, scan/quarantine status, and redacted evidence refs, preserving the existing review gate.
6. **Read-only operations dashboard endpoints.** Add safe summaries for queue/dead-letter/outbox/audit/business-labor outcomes before adding richer infra dashboards.
7. **Fixture-backed Gingr DTO expansion only after evidence.** Keep retail as the example of safe provider DTO promotion; add grooming/training DTOs only with documented or fixture-backed payloads.

## Bottom line

The repo already has strong presentation evidence for the intended ownership boundary: app-owned API DTOs, quarantined provider DTOs, a real Postgres migration spine, append-only audit/review/outbox tables, JSON tracing startup, local safe runtime probes, and business/labor outcome records. The remaining readiness gap is not strategy; it is wiring. The next board should connect one safe workflow end-to-end through API -> Postgres workflow/audit/review records -> worker result -> correlated logs/metrics, while keeping Gingr/customer/payment/provider side effects stubbed or disabled.
