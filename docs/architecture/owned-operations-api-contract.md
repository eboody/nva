# Owned operations API contract families

Status: architecture/API contract plan for the owned Pet Resorts operations API. This page defines product-owned contract families and recommended v0 schema surfaces for downstream OpenAPI contract work; it does not implement routes, claim live NVA/Gingr access, claim production readiness, authorize provider/PMS writes, send customer/member messages, move payments/refunds/discounts, change schedules/capacity, approve medical/safety decisions, or deploy anything.

## Contract thesis

NVA should call an owned Pet Resorts operations API for source-backed work, review queues, outcome capture, audit, and BI-ready read models instead of pulling raw Gingr/provider data into a separate BI database and repairing meaning downstream.

Gingr and other provider systems remain source adapters during migration. They may provide source refs, observed facts, raw/source snapshots, and mapping-gap evidence. They should not define public API resources, lifecycle states, review gates, labor metrics, BI projections, or live side-effect authority.

The v0 contract is intentionally job-to-be-done oriented:

1. Make source evidence and provenance inspectable.
2. Promote only validated facts into owned customers, pets, reservations, service lines, care, vaccine, document, workflow, and outcome resources.
3. Keep every risky action behind explicit review gates and blocked live-action policy.
4. Capture labor/outcome evidence where staff actually reviewed work.
5. Give BI and operations stable read contracts with lineage and caveats instead of raw provider table mirrors.
6. Prove the shape first with Data-Quality Hygiene, then likely Manager Daily Brief.

Primary source context: [owned operations API replacement thesis](owned-operations-api-replacement.md), [runtime contract boundaries](runtime-contract-boundaries.md), [API shell README](../../apps/api/README.md), [domain README](../../domain/README.md), [app README](../../app/README.md), [workflow-to-entity navigation map](../design/workflow-to-entity-navigation-map.md), and [Data Quality Hygiene workflow](../workflows/operator/data-quality-hygiene.md).

## Contract layering

```text
source adapters and staff inputs
  - provider reads, fixture imports, staff forms, uploaded evidence metadata
  - authority: what was observed, from where, when, under which adapter/schema version
        |
        v
owned operational write contracts
  - commands that accept work, create workflow events, draft review packets, record decisions/outcomes
  - authority: review-scoped product state and append-only evidence
        |
        v
owned query/read-model contracts
  - staff queues, BI projections, audit lookup, source-quality/labor metrics, readiness probes
  - authority: supported read views with source refs, review status, caveats, and disabled side-effect posture
```

Design rule: if a field exists only because Gingr has it, keep it in source evidence or adapter-mapping metadata until an NVA workflow, review gate, read model, or metric gives it product meaning.

## v0 API family table

| API family | Product owner | Source authority | Review gate | Primary consumers | Current proof / gap |
| --- | --- | --- | --- | --- | --- |
| Sources, provenance, and data quality | Operations data-quality owner with analytics/regional ops input. | `domain::source`, `domain::data_quality`, adapter/import metadata, fixture or approved source observations; provider ids stay external refs. | Front-desk lead, manager, regional/ops analyst, or sensitive-data reviewer depending on issue category; no hidden source repair. | Staff cleanup queue, Data-Quality Hygiene workflow, BI source-quality reporting, migration/adapter reconciliation. | Strong first slice: Data-Quality Hygiene app/API/storage proof exposes source refs, issue refs, cleanup drafts, outcome summary, and disabled side effects. Gap: no durable source snapshot/import tables or production adapter feed yet. |
| Customers, pets, reservations, and service lines | Pet Resorts operations product owner; service-line owners for boarding/daycare/grooming/training/retail policy. | Owned domain aggregates and service-line contracts promoted from staff input, provider evidence, or migration imports; provider IDs are cross-references only. | Staff/manager/care/payment/policy review when facts are missing, conflicting, payment-sensitive, medical/behavioral, or capacity-affecting. | Staff UI, booking/checkout workflows, Manager Daily Brief, BI occupancy/service demand views, future integration adapters. | Domain and migration spine already name customers, pets, reservations, service offerings, payment projections, service-line contracts. Gap: no v0 CRUD/query API family published as OpenAPI and current API routes are workflow-specific. |
| Care, vaccine, document, and evidence handling | Care/medical-document operations owner with manager oversight. | Uploaded/staff/provider document metadata, object metadata, vaccine extraction suggestions, care notes, incident evidence, provenance/source refs. | Medical document review, behavior review, manager approval, sensitive-data/redaction gate; no autonomous vaccine/medical/safety acceptance. | Vaccine/document reviewers, care staff, front desk, booking/daycare eligibility workflows, audit and BI exception views. | API local vaccine-document upload/review routes and migration tables prove document, extraction, review packet, approval, eligibility projection, and audit shape. Gap: object storage adapter, OCR/runtime, role/location auth, and durable repository wiring are not complete. |
| Workflows and review queues | Workflow product owner for staff/manager labor reduction. | App workflow packets over domain/source evidence: booking triage, Data-Quality Hygiene, Manager Daily Brief, checkout, retention, inquiry, daily updates, vaccine/docs. | Named `domain::policy::ReviewGate` values and workflow-specific blocked actions; outbox candidates require approval before execution. | Staff dashboard, agent runtime, managers, workers, audit reviewers, downstream BI queue metrics. | Current API exposes inquiries, vaccine docs, Manager Daily Brief, Data-Quality Hygiene, metrics, readiness. Gap: most state remains in-memory and OpenAPI schemas are not exported. |
| Outcomes and labor metrics | Operations/labor-value owner with analytics owner for reporting definitions. | Reviewed staff dispositions, app outcome records, storage outcome projections, source refs, issue/action refs, reporting groups, estimated and actual labor minutes. | Manager/front-desk/regional review depending on workflow; outcome capture measures reviewed labor and does not prove provider mutation. | Managers, regional ops, finance/ROI stakeholders, BI read models, presentation/demo surfaces. | Manager Daily Brief and Data-Quality Hygiene outcome records exist in app/API/storage; `/ops/metrics/summary` exposes aggregate local counters. Gap: production KPI definitions and durable analytics/read-model materialization need owner validation. |
| Outbox and audit | Platform/safety owner with operations approval owners per action type. | Workflow events/results, review packets, approval records, outbox records, append-only audit rows, request/correlation ids. | Approval gate must match aggregate/action; live execution remains disabled unless later approved adapter path exists. | Workers, audit reviewers, compliance/security, support/debugging, BI audit lookup. | Migration encodes review gates, outbox approval coherence, and append-only audit; API Data-Quality Hygiene local projection creates review-gated outbox-candidate evidence. Gap: durable worker leasing, retries/dead-letter, replay/admin UI, and live adapters are shells or future work. |
| BI and read-model queries | Analytics/BI owner with operations data owner. | Product-owned read models over source-quality, normalized operations, review status, outcomes/labor, audit, and queue posture; every projection should carry lineage and caveats. | Read-only by default; any reclassification, policy interpretation, or destructive cleanup feeds review rather than silently rewriting source truth. | BI warehouse, operations dashboards, regional leaders, manager daily brief, data-quality hygiene analysts. | BI pain is established as strategy signal; Data-Quality Hygiene summary is the first concrete read-model proof. Gap: exact BI stakeholder query inventory and persistent read-model tables/views are not finalized. |
| Readiness and operational metrics | Platform/runtime owner. | Health/readiness probes, safe runtime counters, request/correlation fields, queue/outbox/dead-letter metrics, audit write health, adapter readiness. | No live side effects from readiness. Production readiness claims require dependency checks, monitoring, auth, deployment, and owner approval. | Engineers, operators, deployment reviewers, demo/presentation reviewers. | `/healthz`, `/readyz`, request-id middleware, JSON tracing startup, `/ops/metrics/summary` exist. Gap: durable traces, safe error classes, queue/dead-letter metrics, worker lease metrics, alerting, and dashboards are not complete. |

## Recommended v0 schema surfaces for OpenAPI contract work

Use these surfaces as the handoff table for the OpenAPI contract card. Names are recommended contract families, not frozen Rust type names.

| Family | Commands | Queries / read models | Core v0 DTO/resource fields | Explicitly not included in v0 |
| --- | --- | --- | --- | --- |
| Sources/provenance/data quality | `POST /operations/source-imports`; `POST /data-quality/issues/{issue_id}/actions/{action_id}/outcome`; `POST /agent/drafts/data-quality-hygiene` | `GET /agent/context/data-quality-hygiene`; `GET /data-quality/issues`; `GET /data-quality-hygiene/outcomes/summary`; `GET /read-models/source-quality-backlog` | `source_ref`, `source_system`, `external_record_ref`, `observed_at`, `adapter_version`, `field_path`, `issue_kind`, `severity`, `freshness`, `sensitivity`, `resolution_status`, `affected_entity_ref`, `review_gate`, `source_visibility`, `issue_refs`, `correlation_id` | Raw provider payload pass-through, provider repair, destructive merge/delete, sensitive payload release, or hiding ambiguity. |
| Customers/pets/reservations/service lines | `POST /customers`; `POST /pets`; `POST /reservations/intake`; `POST /reservations/{reservation_id}/staff-decisions` | `GET /customers/{customer_id}`; `GET /pets/{pet_id}`; `GET /reservations/{reservation_id}`; `GET /service-lines`; `GET /read-models/occupancy-service-demand` | Semantic IDs, external refs, location, contact summary/redaction status, pet profile/care refs, reservation lifecycle, service-line contract, source evidence refs, missing-info/hard-stop flags, review requirements | Gingr endpoint-shaped resources, direct provider booking writes, capacity/schedule mutation, payment/refund/discount changes. |
| Care/vaccine/docs | `POST /vaccine-documents/uploads`; `POST /vaccine-documents/review-packets/{review_packet_id}/approve`; `POST /vaccine-documents/review-packets/{review_packet_id}/reject`; `POST /care-notes` | `GET /pets/{pet_id}/care-profile`; `GET /pets/{pet_id}/vaccine-status`; `GET /documents/{document_id}/metadata`; `GET /review-queues/medical-documents` | Document id, subject ref, classification, object/storage metadata, hash, scan/redaction status, extraction suggestion, confidence/uncertainty policy, vaccine status, eligibility projection, approval record, audit refs | Autonomous medical/vaccine/behavior acceptance, raw object URLs in ordinary logs, raw OCR text in broad prompts, provider medical writes. |
| Workflows/review queues | `POST /workflow-events`; `POST /review-packets/{review_packet_id}/decision`; `POST /inquiries`; `POST /agent/drafts/manager-daily-brief`; `POST /manager-daily-brief/actions/{action_id}/outcome` | `GET /review-queues`; `GET /review-packets/{review_packet_id}`; `GET /staff/inquiries`; `GET /agent/context/manager-daily-brief`; `GET /workflow-events/{workflow_event_id}` | Workflow event id, subject ref, source refs, policy context, packet schema version, safe actions, blocked actions, required review gates, approval status, actor refs, idempotency key, request/correlation id | Direct AI side effects, unreviewed sends/writes, provider state changes, schedule/capacity/payment mutations. |
| Outcomes/labor metrics | `POST /workflows/{workflow_id}/outcomes`; workflow-specific outcome capture routes for Data-Quality Hygiene and Manager Daily Brief | `GET /read-models/labor-outcomes`; `GET /ops/metrics/summary`; `GET /manager-daily-brief/outcomes`; `GET /data-quality-hygiene/outcomes/summary` | Outcome status, actor/persona, action kind, reporting group, location, operating day, estimated minutes, actual minutes spent/saved, source-fact correctness, issue/action refs, review packet/approval refs, caveats | Production ROI claims without audited pilot data, labor metrics from unreviewed suggestions, provider activity logs as sufficient labor proof. |
| Outbox/audit | `POST /outbox-candidates` only from approved workflow/review path; `POST /audit-events` only from trusted runtime paths | `GET /audit-events`; `GET /outbox-records`; `GET /dead-letter-records`; `GET /correlations/{correlation_id}` | Outbox id, aggregate ref, topic, payload summary/ref, approval ref, idempotency key, status, retry/claim fields, audit event id, actor kind/id, safe error class, redaction policy | Live delivery adapters, unapproved publish, secrets/raw payloads in audit, approval demotion with open outbox. |
| BI/read-model queries | No direct mutation except reviewed owner-managed projection definitions later | `GET /read-models/source-quality-backlog`; `GET /read-models/review-queue-aging`; `GET /read-models/labor-outcomes`; `GET /read-models/outbox-posture`; `GET /read-models/audit-lineage`; `GET /read-models/occupancy-service-demand` | Projection version, generated_at, freshness, location/day/service dimensions, source refs, review status, issue/outcome counts, labor minutes, caveats, lineage to workflow/audit/outcome rows | Raw Gingr mirror tables as public contract, silently reclassified data, customer/provider PII exports without approved scope. |
| Readiness/metrics | None beyond safe admin configuration cards later | `GET /healthz`; `GET /readyz`; `GET /ops/metrics/summary`; future `GET /ops/queues/summary`; future `GET /ops/adapters/summary` | Service name, live side-effect mode, dependency readiness, adapter mode, fake/disabled runtime mode, aggregate counters, queue/outbox/dead-letter counts, request id support, production gaps | Production deployment claims, enabling live writes by environment variable alone, raw payload logging. |

## Command/query boundary

Commands accept intent or reviewed decisions. They may create workflow events, internal tasks, review packets, approvals, outcome records, audit rows, or disabled/stubbed outbox candidates. They must be idempotent where repeated submissions are plausible, record actor/source/correlation evidence, validate required review gates, and fail closed on unsupported side effects.

Queries expose current state, review queues, context packets, read models, readiness, and audit lookup. They must not mutate provider state, repair source data, send messages, move payments, change schedules, change capacity, approve medical/safety decisions, or silently reclassify ambiguous facts. Query responses should carry source refs, projection versions/freshness, review status, caveats, and redaction posture.

Command contract defaults:

- `POST` routes that create work should return `workflow_event_id`, `review_packet_id` when applicable, `audit_event_refs`, `request_id`, `correlation_id`, and `live_side_effects_allowed: false` unless a future approved adapter explicitly changes the route.
- Draft-submission commands validate against the context packet: known action id, required source refs, issue refs when applicable, required review gates, allowed safe actions, and blocked-action denial reasons.
- Outcome commands measure reviewed work only. They should distinguish completed, deferred, suppressed, source-wrong, not-actionable, and similar dispositions from actual source/provider repair.
- Approval/outbox commands require a coherent review gate, actor, subject aggregate, reason, approval timestamp, idempotency key, and audit linkage before any outbox candidate can exist.

Query contract defaults:

- Staff queue queries return review packets and task summaries, not raw provider payloads.
- BI/read-model queries return projection metadata and caveats, not source-of-record replacement claims.
- Audit/correlation queries return safe metadata and redacted summaries, not secrets, raw documents, payment payloads, raw provider JSON, or hidden prompts.
- Readiness/metrics queries are aggregate or operational metadata only; they do not enable live actions.

## Blocked live-action policy

The owned operations API is allowed to describe, draft, validate, queue for review, and measure work. It is not allowed to execute these live actions in v0:

- customer/member sends, notifications, or outreach;
- provider/PMS/POS/Gingr writes, repairs, merges, deletes, or status changes;
- schedule, capacity, booking confirmation/waitlist/denial/check-in/checkout mutation;
- payment, refund, deposit, waiver, discount, or forfeiture movement;
- vaccine, medical, behavior, safety, incident, eligibility, or legal/liability decisions;
- production deployment or production data handling;
- raw sensitive payload exposure to broad logs, prompts, BI exports, or public API DTOs.

A future live-action path must be designed as a separate approval-controlled adapter surface: source-backed workflow packet -> explicit review gate -> approval record with actor/scope/reason -> outbox record with idempotency/retry/audit -> adapter execution under approved credentials and monitoring -> outcome/audit/reconciliation. Environment variables alone must not turn a draft/review route into a live sender or provider writer.

## First implementation slice: Data-Quality Hygiene

Data-Quality Hygiene is the first v0 slice because it converts the current BI workaround into an owned product contract:

- Source facts and provider/import defects become `source_ref`, `data_quality_issue`, `issue_ref`, freshness, sensitivity, affected entity, and field-path evidence.
- The API context route exposes ranked cleanup candidates without repairing source records.
- Draft validation accepts internal cleanup recommendations only when source refs, issue refs, review gates, and blocked-action policy match the packet.
- Outcome capture records reviewed disposition, actual labor evidence, issue/source refs, review/outbox correlation, and summary rollups.
- BI can ask for source-quality backlog, reviewed cleanup outcomes, labor minutes, issue categories, freshness/sensitivity posture, and workflow-blocking defects without querying raw provider tables.

Recommended durable v0 vertical slice for this family:

```text
POST/GET Data-Quality Hygiene API
  -> workflow_events + review_packets + audit_events
  -> data_quality_hygiene_outcomes + source refs + issue refs
  -> disabled/stubbed internal outbox candidate when review is required
  -> read-model summary for BI/operator backlog and labor evidence
```

Do not broaden the first slice into live provider cleanup. The strongest proof is that the API can make source-data quality visible, reviewable, measurable, and BI-friendly while still refusing unsafe writes.

## Next likely slice: Manager Daily Brief

Manager Daily Brief is the next likely slice because it turns source-grounded operations context into prioritized manager actions with labor-value evidence. It should build on the same contract families:

- query context by location and operating day;
- assemble source-grounded demand, checkout, retention, and data-quality facts;
- validate manager-action drafts against source refs, action kinds, review gates, and blocked actions;
- capture manager feedback, actual minutes, reporting group, and outcome disposition;
- expose labor/read-model projections for managers and BI.

It should not become a staffing/schedule mutation API in v0. Staffing changes, provider writes, customer sends, payment moves, and policy exceptions remain review-only or blocked until explicit owner-approved action paths exist.

## What NVA would call instead of BI pulling raw Gingr tables

For the current BI workaround, the owned API target is a set of read contracts such as:

- `GET /read-models/source-quality-backlog?location_id=&operating_day=&severity=&workflow_blocking=` for source defects, freshness, sensitivity, affected entities, review status, and issue aging.
- `GET /read-models/labor-outcomes?location_id=&from=&to=&workflow=&reporting_group=` for reviewed outcome counts, completed/deferred/suppressed/source-wrong dispositions, estimated/actual minutes, and source refs.
- `GET /read-models/review-queue-aging?location_id=&gate=&workflow=&status=` for review-packet counts, gate ownership, SLA/age, and blocked-action families.
- `GET /read-models/occupancy-service-demand?location_id=&operating_day=&service_line=` for normalized operations demand with projection version, source refs, and data-quality caveats.
- `GET /read-models/outbox-posture?topic=&status=&location_id=` for approved/stubbed side-effect candidate visibility and retry/dead-letter posture.
- `GET /read-models/audit-lineage?correlation_id=` for request -> workflow -> review -> approval -> outcome/outbox -> audit evidence.

Those calls answer operational questions directly. They preserve lineage and review status, and they avoid forcing analysts to infer NVA business meaning from raw Gingr endpoint/table shapes.

## Honest gaps before implementation

- Current API DTOs are Rust/private/local-demo contracts, not exported OpenAPI/client schemas.
- Most current API state is in-memory; durable Postgres repository wiring is still downstream work.
- Auth/session/role/location authorization is deferred.
- Worker durable leasing, dead-letter/replay, and outbox execution are not production-ready.
- Object storage and document evidence handling are modeled but do not yet have a durable adapter connection.
- Detailed BI stakeholder query inventory, production KPI definitions, retention/redaction rules, and live-adapter owner decisions require human validation.
- No live NVA/Gingr credentials, production data, provider writes, member sends, payment/schedule/medical/safety actions, or production deployment are claimed by this contract.
