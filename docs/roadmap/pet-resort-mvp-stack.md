# Pet Resort MVP Technical Stack and Integration Architecture

Status: recommended MVP stack pending human approval.

This document defines the recommended technical stack for the pet resort MVP implementation board. It is an architecture and cutline artifact only; it does not implement code, approve production deployment, approve live customer messaging, approve provider/PMS writes, or approve autonomous medical, vaccine, booking, incident, payment, or eligibility decisions.

## 1. Recommendation summary

Recommended MVP stack:

| Layer | Recommendation | Why this is the MVP default |
| --- | --- | --- |
| Frontend | Next.js + TypeScript staff/admin web app, using a small internal component system and server/client routes as needed | Fastest path to a usable staff dashboard, forms, queues, review packets, and local smoke tests while preserving a strict Rust API/domain boundary. |
| Backend/API | Rust API service, preferably Axum, in this existing Rust workspace | Keeps source-of-truth workflow, policy, idempotency, audit, and side-effect gates in the Rust domain direction already established by the repo. |
| Domain | Existing `domain` crate remains canonical for business types; add application service crates around it instead of letting frontend/provider strings drive behavior | Preserves semantic IDs, enums, builders, typestate/newtypes where useful, and avoids stringly workflow decisions. |
| Database | PostgreSQL with SQLx migrations and compile/runtime query checks | Strong relational fit for customers, pets, reservations, tasks, review packets, workflow events, outbox, and audit; easy local/staging parity. |
| Queue/events | Postgres-backed durable workflow inbox/outbox and leased jobs table for MVP; later replace/augment with a broker only if throughput requires it | Matches the existing architecture: app-owned durable events before AI/runtime calls, idempotency, replay, dead-letter, and operator visibility without adding Redis/NATS/Kafka early. |
| File storage | S3-compatible object storage, MinIO locally/staging and S3/R2-compatible in production | Private immutable document/vaccine evidence objects, hashes, scan/OCR state, retention metadata, and presigned access through the app only. |
| Auth/session | First-party staff auth with secure HTTP-only cookie sessions, role/location scopes, and audit-linked actor refs; defer customer portal auth | Fits internal staff/manager MVP; avoids committing to customer-facing auth before live portal/message gates. OIDC/SSO can be added behind the same actor/session boundary. |
| Hosting/deployment | Docker Compose for local dev; Coolify/self-hosted staging for review; production deployment remains a human gate | Fast, inspectable path that matches existing operations habits without implying production launch. |
| Observability | Rust `tracing` JSON logs, request/job correlation IDs, audit-event table, dead-letter views, health/readiness endpoints, and optional OpenTelemetry/Sentry later | MVP must prove traceability and fail-closed behavior before broad distributed observability spend. |
| AI runtime | `AgentRuntime` adapter behind the Rust app; app builds typed `AgentPromptPacket<T>`, validates `WorkflowResult<T>`, persists result/review packet, and executes no side effects without policy + approval | Aligns with the AI runtime architecture: Hermes/LLM is a bounded assistant, not a system of record or direct side-effect executor. |

Short version: build a Rust-owned workflow platform with a TypeScript staff UI, PostgreSQL as the durable system of record and queue, S3-compatible private evidence storage, and AI as a validated adapter behind app-owned policy/audit gates.

## 2. Source inputs and assumptions

Primary inputs used:

- `README.md`: Rust-first direction; typed packets instead of free-form prompts; app owns policy, state, and writes.
- `docs/roadmap/pet-resort-mvp-implementation.md`: implementation graph, MVP cutline, and approval gates.
- `docs/workflows/staff-operations-parts/inputs.md`: product-map substitute while `docs/product/pet-resort-product-map.md` is absent.
- `docs/architecture/pet-resort-data-model.md`: canonical entities, lifecycles, review gates, audit obligations, and AI boundaries.
- `docs/architecture/pet-resort-workflow-events.md`: workflow event envelope, durable queue, idempotency, replay, and outbox model.
- `docs/architecture/pet-resort-ai-runtime.md`: app-owned prompt packet/result validation/runtime boundary.
- `docs/security/pet-resort-security-audit.md`: role matrix, sensitive-data handling, audit model, AI governance, and retention gates.
- Core workflow specs under `docs/workflows/`: inquiry intake, booking triage, staff operations, vaccine/document handling, daily care updates, incident escalation, customer messaging, and payments/pricing.

Important product-map assumption: the requested `docs/product/pet-resort-product-map.md` is not present in this repo. The current MVP cutline uses `docs/workflows/staff-operations-parts/inputs.md` and the roadmap handoff as the product-map substitute. If the product map is restored later, this stack should be rechecked against it before production build-out.

## 3. MVP product cutline

Build for an internal staff/manager workflow tool for one resort or a small resort group.

MVP must include enough platform foundation to support:

1. Staff dashboard shell and role/session guard.
2. Customer, pet, reservation, document, vaccine, task, note, incident, message draft, workflow event, workflow result, and audit records.
3. Durable workflow event ingestion and job processing.
4. Review-oriented staff queues for intake, booking triage, vaccine/document review, daily update drafts, and incident escalation.
5. Draft-only customer messaging surfaces.
6. Local fake/deterministic AI runtime and optional real runtime adapter for review-mode testing.
7. Local smoke tests demonstrating event -> job -> AI/deterministic result -> review packet/task/draft -> audit, with no live customer or provider effects.

Explicitly outside MVP unless separately approved:

- Production deployment and real customer data launch.
- Live customer-message sending.
- Live Gingr/provider writes or reservation mutations.
- Autonomous booking confirmation/rejection/waitlist/capacity exceptions.
- Autonomous vaccine/medical/behavior/group-play eligibility decisions.
- Payment provider selection, live checkout links, charges, refunds, waivers, discounts, or forfeitures.
- Incident closure, severity finalization, owner-facing incident sends, legal/liability claims, or eligibility-affecting behavior flags without manager approval.
- Customer portal replacement or public self-service app beyond a local/staff-entered inquiry intake path.

## 4. Architecture shape

Recommended runtime topology:

```text
Next.js staff UI
  -> Rust HTTP API (Axum)
      -> app services / policy services
          -> domain crate types and validators
          -> PostgreSQL repositories
          -> workflow inbox/jobs/outbox/audit tables
          -> object-storage evidence service
          -> AgentRuntime adapter
              -> fake deterministic runtime for tests/local
              -> Hermes/OpenAI-compatible runtime when explicitly configured
          -> typed side-effect adapters, disabled/stubbed by default in MVP
```

Workflow pattern:

```text
staff action / local form / fixture / verified provider event
  -> API authenticates actor and validates subject scope
  -> API writes normalized WorkflowEvent + idempotency key + audit event
  -> worker leases job from Postgres queue
  -> context builder loads approved snapshots and evidence refs
  -> app builds AgentPromptPacket<T>
  -> AgentRuntime returns untrusted raw output
  -> parser validates WorkflowResult<T> and workflow-specific schema
  -> app persists result, safe summary, review packet/tasks/drafts
  -> deterministic policy + human approval records decide any later side effect
```

The queue/outbox tables should be treated as product state, not infrastructure glue. They are part of the safety model because replay, dedupe, approval linkage, dead-letter review, and audit correlation depend on them.

## 5. Frontend stack

Recommendation: Next.js + TypeScript for the staff dashboard.

MVP frontend responsibilities:

- Staff login/session flow and role/location-scoped navigation.
- Today/operations dashboard.
- Customer/pet/reservation detail pages.
- Task/review queue.
- Inquiry intake form and triage result review.
- Booking triage packet view.
- Vaccine/document upload/review surfaces.
- Daily care note input and update-draft preview.
- Incident intake and manager review views.
- Message draft approval/suppression views, with send disabled/stubbed.
- Audit-visible action history for staff decisions.

Recommended UI conventions:

- Treat API schemas as the contract; do not reimplement business authority in React components.
- Use typed generated clients from OpenAPI or a checked TypeScript client package after API routes stabilize.
- Prefer simple server-rendered or data-loader pages for MVP queues; add client state only for forms, previews, and review interactions.
- Keep sensitive fields out of broad dashboards by default; show redacted summaries, flags, gates, and evidence refs.

Alternatives considered:

| Alternative | Pros | Cons | Decision |
| --- | --- | --- | --- |
| Rust full-stack UI such as Leptos | Single language, strong typing end to end | Slower staff-dashboard iteration, less commodity UI/forms ecosystem | Defer. Good future option if the team wants all-Rust UI. |
| React/Vite SPA | Simple frontend build | Requires separate routing/deployment choices and more client-side auth/data decisions | Acceptable, but Next.js gives better conventions for internal app pages. |
| Server-rendered Axum/Askama/HTMX | Small operational footprint | Harder for rich review queues, previews, and future admin UX | Acceptable for a narrower prototype, not preferred for this MVP. |

## 6. Backend/API stack

Recommendation: Rust API service in this workspace, using Axum or an equivalent Tower-based HTTP stack.

MVP backend responsibilities:

- Auth/session middleware and actor extraction.
- Role/location/subject permission checks.
- CRUD/query surfaces for core entities and review queues.
- Workflow event ingestion endpoints and staff-action endpoints.
- Deterministic policy evaluation for hard stops, missing information, review gates, and allowed actions.
- Job workers for workflow events, AI runtime calls, OCR/extraction stubs, dead-letter handling, and replay.
- Audit writing for every security-relevant event.
- Object-storage mediation for uploads/downloads; no direct raw object URLs in ordinary UI logs.
- Optional provider adapter read paths for Gingr/PMS data, with writes disabled unless separately approved.

Recommended crates to add during implementation, subject to code-card review:

- `axum`, `tower`, `tower-http` for HTTP.
- `tokio` for async runtime.
- `sqlx` with PostgreSQL support for migrations/repositories.
- `uuid`, `chrono`, `serde`, `thiserror`, `nutype`, `bon`, and `statum` consistent with current workspace conventions.
- `tracing`, `tracing-subscriber`, and optionally `opentelemetry` later.
- `argon2` or passkey/OIDC-specific crates depending on approved auth choice.
- S3 client crate such as `aws-sdk-s3` or an S3-compatible abstraction for object storage.

Backend boundary rule: the frontend may request actions, but only the Rust API/domain layer may decide whether an action is allowed, denied, blocked, suppressed, or routed to review.

## 7. Database and migration strategy

Recommendation: PostgreSQL with SQLx migrations.

Why PostgreSQL:

- Core data is relational and audit-heavy.
- JSONB can hold schema-versioned workflow output fragments without giving up relational indexes on subject, status, policy version, and review gates.
- Row-level constraints, unique keys, and transactions are valuable for idempotency, leases, outbox records, and audit linkage.
- Postgres is enough for MVP queue/replay/dead-letter needs.

MVP table families:

- Identity and scope: staff users, roles, sessions, locations.
- Core domain: customers, pets, reservations, services/catalog snapshots, documents, vaccine records, tasks, care notes, incidents, messages, payments/deposit projections.
- Workflow: workflow events, workflow jobs, workflow results, review packets, prompt manifests, policy snapshots, idempotency keys.
- Side-effect safety: outbox records, provider action attempts, message send attempts, delivery/reconciliation state, all disabled/stubbed for live effects in MVP.
- Audit: append-only audit events and redacted audit projections.
- Storage metadata: object refs, content hashes, scan/quarantine state, retention/legal-hold metadata.

Migration conventions:

- Migrations must encode enum/check constraints for high-risk lifecycle states where feasible.
- Use semantic IDs in application types even if persisted as UUID/text.
- Provider IDs stay in external refs/adapters and should not replace canonical IDs.
- Every mutable business table that participates in workflow should have audit correlation and updated-at/version fields suitable for replay and source freshness checks.

## 8. Queue, event runtime, and idempotency

Recommendation: Postgres-backed workflow inbox/jobs/outbox for MVP.

Required records:

- `workflow_events`: normalized semantic events with `event_type`, subject, related IDs, source refs, policy context, approval requirements, causation/correlation IDs, and `source_event_key`.
- `workflow_jobs`: durable job records with status, lease owner, lease expiry, attempt count, next run, last error, dead-letter reason, and safe summary.
- `workflow_results`: validated `WorkflowResult<T>` envelopes plus schema version, status, risk flags, review gates, and safe output refs.
- `outbox_records`: approved action candidates and execution attempts. In MVP these should exist as draft/stub records but not execute live customer/provider effects.
- `audit_events`: append-only trace for event accepted, job started/completed/failed, result validated/rejected, review packet created, approval requested/applied, and dead-letter/replay actions.

Why not Redis/NATS/Kafka for MVP:

- The bottleneck is correctness, reviewability, and audit, not throughput.
- Adding a broker creates another failure/replay surface before product gates are proven.
- Postgres transactions make event + job + audit atomic enough for MVP.

Upgrade path:

- Add a broker only after Postgres-backed queue metrics show contention or latency that matters.
- Keep Postgres as the source of truth even if a broker later accelerates delivery.

## 9. File storage and document handling

Recommendation: private S3-compatible object storage with app-mediated access.

MVP behavior:

- Local/staging: MinIO.
- Production candidate: S3, R2, or another S3-compatible private bucket, subject to deployment/security approval.
- Store raw uploaded vaccine/document/incident media as immutable evidence objects with content hash, size, MIME type, uploader/source, scan/quarantine status, subject refs, retention class, and legal-hold state.
- Ordinary database rows store object refs and safe metadata, not raw file bytes.
- Prompt packets should use evidence refs and minimal approved excerpts; do not pass raw documents broadly into AI context.
- Download/preview URLs must be short-lived, actor-scoped, and audited.

MVP cutline:

- Implement upload/storage metadata and review surface.
- OCR/extraction may be fake, fixture-backed, or runtime-adapter-backed locally.
- Malware scanning can be a stubbed state in local MVP, but the data model must include quarantine/scan status so production cannot skip the gate silently.

## 10. Auth, sessions, roles, and approvals

Recommendation: first-party staff auth for MVP, with role/location-scoped sessions and audit-linked actor refs.

MVP auth model:

- Staff users only; customer portal auth is deferred.
- Secure HTTP-only cookie sessions.
- Roles/scopes aligned to the security spec: staff, lead staff, manager/admin, system, AI workflow worker.
- Location scope on every workflow event and dashboard query.
- Authorization checks in backend services, not frontend route guards only.
- Approval records linked to actor, role, scope, policy version, subject, reason, and effect.

Deferred auth decisions:

- OIDC/SSO provider for production staff accounts.
- Customer portal identity and household scoping.
- Break-glass, dual-control, export, legal-hold, and privileged raw-evidence access controls beyond MVP staff review needs.

## 11. Hosting and deployment

Recommendation:

- Local development: Docker Compose for Postgres, MinIO, the Rust API/worker, and the Next.js frontend.
- CI: run Rust tests, migrations against a disposable Postgres, frontend typecheck/lint/tests, and smoke fixtures.
- Staging/review: Coolify or equivalent Docker-based deployment for internal review with non-production data.
- Production: blocked until human approval of production deployment, security posture, retention, live integration credentials, and operational runbooks.

Deployment constraints:

- Do not deploy with live customer data by default.
- Do not enable live message sends or provider writes from environment variables alone; those should require code-level policy gates plus approved configuration and human approval records.
- Keep fake/stub side-effect adapters as the default for local and CI.

## 12. Observability, audit, and operations

MVP observability must prove safety and traceability, not just uptime.

Required MVP instrumentation:

- Structured JSON logs with request ID, correlation ID, workflow event ID, job ID, actor kind/id, location ID, subject ref, and safe error class.
- Health/readiness endpoints for API, worker, database, and object storage dependency checks.
- Queue/dead-letter dashboard or admin view showing safe summaries, retry counts, and blocked reason.
- Audit-event append for every state transition, approval, denied action, failed validation, dead-letter, replay, prompt packet build, AI result validation, and side-effect candidate.
- Redaction rules for logs and UI: no raw documents, OCR, message bodies, incident narratives, raw provider JSON, payment payloads, secrets, or hidden prompts in ordinary logs.

Optional after MVP foundation:

- OpenTelemetry traces.
- Sentry or similar error aggregation.
- Metrics dashboards for job latency, dead-letter rate, validation failure rate, review queue age, and audit write failures.

## 13. AI runtime integration

Recommendation: `AgentRuntime` trait/adapter owned by the Rust app.

MVP runtime modes:

1. `FakeAgentRuntime`: deterministic fixture outputs for CI/local smoke tests.
2. `HermesAgentRuntime` or OpenAI-compatible adapter: optional review-mode integration for demos, never a direct side-effect actor.
3. `DisabledAgentRuntime`: returns a safe `FailedSafely`/review-needed result when runtime is not configured.

Required runtime contract:

- App constructs `AgentPromptPacket<T>` after authorization, policy selection, data minimization, evidence lookup, and idempotency calculation.
- Runtime receives only the task-specific packet and allowed context; it cannot fetch arbitrary records or expand its own permissions.
- Runtime output is untrusted until parsed and schema-validated into `WorkflowResult<T>`.
- The app persists validation outcome, safe summary, risk flags, review gates, prompt manifest/hash, model/provider/config ref without secrets, and audit linkage.
- Any suggested task, draft message, status recommendation, or side-effect candidate is reviewable evidence, not execution proof.

MVP workflows using AI boundary:

- Inquiry intake: extract lead/request facts, route missing info, draft follow-up, create staff tasks.
- Booking triage: summarize deterministic gates, draft explanation, recommend review tasks/status only.
- Vaccine document: extract candidate facts from evidence; never verify final compliance.
- Daily care update: summarize approved notes and draft owner update; no live send.
- Incident escalation: summarize chronology, risk, tasks, and owner-message draft; no final severity/closure/send.

## 14. Provider/PMS and payment integration posture

Gingr/PMS:

- Treat provider data as boundary input until verified, mapped, and normalized.
- Read/import paths may be modeled for MVP fixtures or staging demos.
- Live provider writes are out of MVP and require explicit approval, typed commands, idempotency keys, outbox records, reconciliation, and audit.
- Provider IDs must remain external refs; they must not become canonical domain IDs.

Payments:

- Model semantic payment/deposit projections, policy snapshots, review tasks, and audit.
- Do not select or configure a production payment provider in this card.
- Do not create live checkout links, charges, refunds, waivers, discounts, forfeitures, or payment-provider webhooks in MVP without separate approval.

## 15. Stack tradeoffs and rationale

### Why Rust backend plus TypeScript frontend

Pros:

- Preserves the existing Rust domain and semantic-code direction.
- Keeps policy, state transitions, idempotency, audit, and side-effect gates in one strict backend boundary.
- Lets frontend iterate quickly on internal staff workflows.
- Avoids making the browser the owner of workflow authority.

Cons:

- Two-language app surface.
- Requires API schema discipline so frontend/backend do not drift.
- Some duplicated validation may exist for form UX; backend remains authoritative.

Mitigation:

- Generate or share API types from backend schemas after the first API skeleton is stable.
- Keep form validation user-friendly but non-authoritative.
- Add smoke tests around API contracts and core workflow fixtures.

### Why Postgres-backed queue first

Pros:

- Simple deployment and local dev.
- Strong transaction semantics for event + job + audit.
- Easy operator visibility and replay/dead-letter queries.
- Enough for MVP throughput.

Cons:

- Not ideal for very high-volume job streaming.
- Leases/retries must be implemented carefully.

Mitigation:

- Keep queue library/table small, tested, and explicit.
- Add broker later only when measured bottlenecks justify it.

### Why first-party staff auth first

Pros:

- Supports internal MVP quickly.
- Keeps customer portal identity out of scope.
- Easy to audit staff approvals and actor scopes.

Cons:

- Production may prefer OIDC/SSO.
- Password/session security must still be implemented correctly.

Mitigation:

- Encapsulate actor/session extraction behind backend traits/services.
- Keep role/location scopes independent of auth provider choice.

## 16. Implementation handoff for downstream cards

`t_ffcc45ad` should create the project skeleton around this cutline:

- Add a Rust API crate and worker crate to the workspace.
- Add a Next.js app under an `apps/` path or equivalent agreed repo layout.
- Add Docker Compose for Postgres and MinIO.
- Add SQLx migration framework and first empty/schema baseline migration.
- Add local `.env.example` with no secrets.
- Add fake runtime and stub side-effect adapters as defaults.
- Add test commands for Rust unit/integration tests, migration apply/check, frontend typecheck/lint, and local smoke fixtures.
- Add CI that runs the same gates without production credentials.

`t_71c866a7` should then implement the core data model and migrations using the canonical entity/audit/workflow documents, not frontend-first tables.

Feature-slice cards should build vertical paths through the same event/result/review/audit pipeline rather than custom per-feature side-effect shortcuts.

## 17. Human approval gates still open

The following remain explicit review gates before downstream code cards should treat them as settled:

1. Stack choice: approve or change the recommended Rust API + Next.js UI + Postgres + Postgres queue + S3-compatible storage stack.
2. MVP cutline: approve or narrow the internal staff/manager tool scope and the five workflow slices.
3. Production deployment: approve hosting, environment, secrets, data classification, retention, backup, and incident response before live deployment.
4. Live customer messaging: approve exact categories/templates/channels/consent/quiet-hours/suppression/idempotency before any auto-send.
5. Provider/PMS writes: approve exact Gingr/provider commands, credentials, reconciliation, and rollback before live mutation.
6. Booking automation: approve confirmation/rejection/waitlist/capacity/special-care/behavior exception policy before executable status changes.
7. Vaccine/document automation: approve source trust, uncertainty thresholds, auto-accept policy if any, and reviewer authority.
8. Incident workflow: approve owner-facing incident messages, severity policy, closure authority, and eligibility-affecting flags.
9. Payment/pricing: approve provider, policy source of truth, payment links/webhooks, refunds/waivers/discounts/fees, and customer-facing payment copy.

## 18. Decision needed

Recommended decision: approve this stack for MVP implementation, with all live side-effect gates preserved as listed above.

If the stack is approved, the next code card can safely build the project skeleton and local dev/CI foundation without enabling production deployment, live customer sends, live provider writes, or payment actions.
