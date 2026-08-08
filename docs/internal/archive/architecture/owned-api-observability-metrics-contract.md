# Owned API observability, audit, metrics, and BI-readiness contract

Status: architecture contract for the owned Pet Resorts operations API. This document defines the minimum useful logging, audit, metrics, and BI-read-model contract for downstream route, schema, storage, and dashboard cards. It does not implement dashboards, claim Prometheus, OpenTelemetry, Sentry, production monitoring, live NVA/Gingr access, provider writes, customer sends, payment movement, schedule/capacity changes, medical/safety decisions, or production deployment.

Source context: [owned operations API contract families](owned-operations-api-contract.md), [DTO/API/DB/observability readiness gap map](../audits/dto-api-db-observability-readiness-gap-map.md), [Pet Resort MVP stack](../roadmap/pet-resort-mvp-stack.md), [foundation migration](../../migrations/0001_mvp_foundation.sql), [migration contract tests](../../storage/tests/mvp_migration_contract.rs), [API main](../../apps/api/src/main.rs), [API HTTP shell](../../apps/api/src/http.rs), and [worker runtime](../../apps/worker/src/runtime.rs).

## Contract thesis

"Actually useful with logging and metrics" means an operator, reviewer, engineer, and BI analyst can answer these questions from product-owned evidence without scraping raw Gingr/provider tables or reading broad payload logs:

1. Which request accepted or rejected work?
2. Which workflow event, review packet, approval decision, outcome, outbox candidate, and audit rows came from that request?
3. Which source refs and location/tenant scope were used?
4. Which actor or reviewer role made a decision?
5. Which live action was blocked, why, and under which review gate?
6. Which outcome disposition was recorded, and how many estimated/actual minutes were associated with reviewed work?
7. Which import, source-quality, queue, outbox, or sync gaps need operator attention?
8. Which BI snapshots are safe, versioned, source-linked, and caveated enough to reduce manual analyst cleanup?

Business metrics and infrastructure telemetry are separate contracts:

- Business metrics measure reviewed operations work: source-quality backlog, review decisions, outcome dispositions, labor minutes, avoided/reworked labor, outbox posture, queue aging, service demand, and audit lineage.
- Infrastructure telemetry measures service health: request counts/latency/error class, readiness, worker lease age, queue depth, dead letters, audit write failures, adapter readiness, and object-storage/DB posture.

The current repo has a useful local baseline, but not a production observability stack. Downstream code cards should extend the baseline without claiming unimplemented Prometheus, OpenTelemetry, Sentry, live adapters, dashboards, or production alerting.

## Current baseline and honest gaps

| Surface | Already present | Current gap / code-card addition |
| --- | --- | --- |
| JSON service logs | `apps/api/src/main.rs` and `apps/worker/src/main.rs` initialize JSON `tracing_subscriber` output. | Add consistent span fields across handlers and workers: `request_id`, `correlation_id`, `workflow_event_id`, `review_packet_id`, `approval_record_id`, `outbox_record_id`, `location_id`, `tenant_id`, `actor_kind`, `actor_role`, `subject_ref`, `safe_error_class`. |
| HTTP request evidence | `apps/api/src/http.rs` accepts/generates `x-request-id`, echoes it in responses, and adds `http.route`, `http.request_id`, and `payload_logging=disabled` to `api_request` spans. | Propagate a stable `correlation_id` from request -> workflow event -> audit/outcome/outbox rows; include route/result/error class and sanitized actor/location evidence on every route span. |
| Readiness probes | `/healthz` and `/readyz` exist and explicitly show disabled live side-effect posture and missing durable dependencies. | Make readiness dependency checks real when DB/object storage/worker leases exist; keep readiness read-only and never use it to enable live effects. |
| Local metrics summary | `/ops/metrics/summary` returns aggregate-only local counters and labor rollups for manager daily brief and data-quality hygiene. | Add durable read models for request/queue/outbox/dead-letter/audit/source-quality metrics; keep local summary labeled as a demo/readiness proof until storage is wired. |
| Workflow/audit schema | `migrations/0001_mvp_foundation.sql` has `workflow_events`, `workflow_results`, `review_packets`, `approval_records`, `outbox_records`, and append-only `audit_events`. | Add missing correlation/location/actor/source-quality/import/job fields where needed; add indexes/read views for BI and operational dashboards. |
| Outcome/labor evidence | Migration and storage/API code model `manager_daily_brief_outcomes` and `data_quality_hygiene_outcomes` with source refs, outcome disposition, actual minutes, estimated minutes saved, persona, location, day, and correlation id. | Generalize to an `outcome_recorded` taxonomy/read model across workflows; preserve workflow/action/source refs and distinguish reviewed outcomes from provider mutation proof. |
| Outbox safety | Migration requires approved approval records for outbox candidates and supports `pending`, `claimed`, `published`, `failed`, and `dead_letter` statuses. Worker runtime remains fake/stubbed. | Add durable worker leasing, retry/dead-letter evidence, replay/admin summaries, and safe failure classes before any live adapter is considered. |
| Source-quality proof | Data-Quality Hygiene API/storage slice records issue refs, source refs, resolution status, and reviewed labor outcomes. | Add import-run/source-snapshot/sync-gap storage and read models so BI can see freshness, drift, missing evidence, and source defects without raw provider mirrors. |

## Cross-cutting correlation contract

Every accepted command and every audit-worthy query should carry a single correlation spine. The names below are recommended DTO/storage/log fields for downstream implementation cards.

| Field | Required on | Purpose / notes |
| --- | --- | --- |
| `request_id` | API request span, response header, route result, audit metadata. | Safe per-request identifier. Accept `x-request-id` when syntactically safe; otherwise generate a UUID. Never put raw payload or secrets in it. |
| `correlation_id` | Workflow event, review packet, approval, outcome, outbox, audit, BI snapshot metadata. | Stable business/workflow correlation key across request, worker, review, outcome, and read-model rows. Generate if absent. |
| `workflow_event_id` | Workflow event, result, review packet, outcome, outbox/audit metadata, worker logs. | Durable event accepted before worker/review processing. |
| `workflow_event_kind` | Workflow event, logs, read models. | Semantic event category, not provider endpoint name. Examples: `inquiry_submitted`, `data_quality_hygiene_draft_submitted`, `manager_daily_brief_outcome_captured`. |
| `source_ref` / `source_refs` | Workflow event payload, review packet evidence, outcome, import/sync events, BI read models. | External/provider/import evidence reference only. It is not a canonical owned entity id. |
| `location_id` | All workflow/audit/outcome/read-model records when known. | Current location scope. If auth/location model is not wired yet, use nullable/demo placeholder and call that out. |
| `tenant_id` | Same as `location_id` where multi-tenant deployment is later added. | Placeholder for tenant isolation; do not fabricate tenant scope in current local shell. |
| `actor_kind` | Audit, approvals, outcomes, logs. | `staff`, `manager`, `system`, `agent`, `customer` where appropriate. |
| `actor_id` | Audit, approvals, outcomes. | Safe internal actor ref, not a password/email dump. Current shell has staff ids in payloads; future auth should supply it. |
| `actor_role` / `reviewer_role` | Approval, review packet, outcome, logs/read models. | Business role/persona such as `front_desk_lead`, `general_manager`, `regional_operator`, `operations_analyst`. |
| `subject_ref` | Logs/audit/read models. | Safe typed subject reference (`customer:{id}`, `pet:{id}`, `reservation:{id}`, `source_issue:{id}`); avoid broad PII. |
| `review_packet_id` | Review events, approvals, outcome/outbox linkage. | Shows which packet a human or policy reviewed. |
| `approval_record_id` | Approval, outcome, outbox. | Required before outbox candidate execution authority can be considered. |
| `blocked_action` | Review packet, outbox blocked event, audit, BI outbox posture. | Exact blocked live action family: customer send, provider write, payment move, schedule/capacity mutation, medical/safety decision, raw-payload export. |
| `blocked_reason` | Same as `blocked_action`. | Human-readable and safe. No raw secrets, payment payloads, document contents, or provider JSON. |
| `outcome_disposition` | Outcome records and BI labor read models. | Examples: `completed`, `deferred`, `suppressed_by_manager`, `source_fact_was_wrong`, `not_actionable`, workflow-specific reviewed dispositions. |
| `estimated_minutes` | Review packet/action recommendation/outcome/read models. | Expected minutes before/avoided/saved; define exact meaning per workflow. |
| `actual_minutes` | Outcome record/read models. | Reviewed staff time spent or saved after action. Required for labor claims. |
| `safe_error_class` | Logs, audit failure metadata, dead-letter records, metrics. | Stable low-cardinality class such as `validation_failed`, `review_gate_missing`, `dependency_unavailable`, `adapter_disabled`, `source_quality_issue`, `dead_lettered`. |
| `payload_logging` | Request/workflow spans. | Should remain `disabled` or `redacted_summary_only`; never raw customer/provider/payment/document payloads. |

## Event taxonomy

The API and worker should emit structured log/audit/read-model events using these event names. Log events are operational evidence; audit events are durable product/security evidence. Use both where a state transition matters.

| Event | Trigger | Mandatory fields | Useful measures/read models | Current status / add next |
| --- | --- | --- | --- | --- |
| `api_request` | Every HTTP request. | `request_id`, `http.method`, `http.route`, `status_code`, `safe_error_class`, optional `correlation_id`, `actor_role`, `location_id`, `tenant_id`. | Request count, error rate by safe class/route, latency by route, unsupported/blocked route count. | Partially present as `api_request` span with request id, route, disabled payload logging. Add status/error/correlation/actor/location. |
| `workflow_event` | Command accepts work or a durable worker job is created/claimed. | `request_id`, `correlation_id`, `workflow_event_id`, `workflow_event_kind`, `source_ref`, `location_id`, `tenant_id`, `actor_role`, `subject_ref`. | Workflow intake volume, event aging, job backlog, source family coverage. | DB table exists; current API often remains local/in-memory. Add durable writes and full correlation fields. |
| `review_packet` | Packet created/updated for human/policy review. | `request_id`, `correlation_id`, `workflow_event_id`, `review_packet_id`, `source_ref`, `location_id`, `actor_role`, `reviewer_role`, `blocked_action`, `review_gate`, `subject_ref`. | Review queue count/age by gate, blocked-action families, source issue categories. | Migration/API model review packets for vaccine and data-quality slices. Add durable BI queue views. |
| `approval_decision` | Reviewer approves/rejects/cancels/supersedes. | `correlation_id`, `workflow_event_id`, `review_packet_id`, `approval_record_id`, `actor_role`, `reviewer_role`, `location_id`, `decision`, `blocked_action`, `reason_ref`. | Approval rate, rejection reasons, gate SLA, reviewer workload. | Migration enforces decision integrity. Add route/storage correlation and read model. |
| `outcome_recorded` | Reviewed work outcome/labor captured. | `request_id`, `correlation_id`, `workflow_event_id`, `approval_record_id`, `source_ref`, `location_id`, `actor_role`, `outcome_disposition`, `estimated_minutes`, `actual_minutes`, `reporting_group`, `operating_day`. | Labor minutes, completion/defer/suppression/source-wrong rates, ROI caveated by workflow/source. | Present for Manager Daily Brief and Data-Quality Hygiene. Generalize and index/read-model it. |
| `outbox_candidate` | Approved/reviewed work creates a publishable or internal handoff candidate. | `correlation_id`, `workflow_event_id`, `approval_record_id`, `outbox_record_id`, `topic`, `blocked_action`, `review_gate`, `status`, `idempotency_key`, `location_id`. | Outbox backlog by topic/status, approved-but-stubbed count, retry posture. | Migration exists and Data-Quality Hygiene can produce local internal handoff evidence. Add durable worker metrics. |
| `outbox_blocked` | A requested side effect is refused or held behind review/stub mode. | `request_id`, `correlation_id`, `workflow_event_id`, `review_packet_id`, `blocked_action`, `blocked_reason`, `review_gate`, `actor_role`, `location_id`. | Blocked side-effect count by action/reason; unsafe automation prevented. | Current DTOs expose disabled live effects and blocked requested side effects in workflow responses. Add durable audit/read model. |
| `source_quality_issue` | Import/mapper/workflow identifies bad, stale, conflicting, sensitive, or missing source evidence. | `correlation_id`, `source_ref`, `issue_ref`, `issue_kind`, `severity`, `freshness`, `sensitivity`, `affected_entity_ref`, `location_id`, `workflow_event_id`. | Source-quality backlog, aging, severity mix, workflow-blocking defects, provider drift categories. | Data-Quality Hygiene models issue/source refs. Add source snapshot/import tables and read views. |
| `import_run` | Provider/file/fixture import starts/completes/fails. | `correlation_id`, `import_run_id`, `source_system`, `adapter_version`, `location_id`, `tenant_id`, `started_at`, `completed_at`, `safe_error_class`, record counts, redaction posture. | Freshness, failure rate, changed record counts, import coverage. | Deferred by current migration. Add before real source adapters or BI exports depend on freshness. |
| `sync_gap` | Expected source/adapter/read-model freshness or reconciliation invariant fails. | `correlation_id`, `source_system`, `source_ref`, `location_id`, `gap_kind`, `detected_at`, `age_seconds`, `safe_error_class`, `workflow_event_id` if tied to work. | Stale/missing source evidence, unclosed reservations, unresolved mapping drift, BI caveats. | Mostly gap today. Use Data-Quality Hygiene issue refs as first product-facing slice. |
| `BI_snapshot` | Versioned BI/read-model projection generated or queried. | `correlation_id`, `snapshot_id`, `projection_name`, `projection_version`, `generated_at`, `freshness`, `location_id`, `tenant_id`, `source_refs`, `workflow_event_id`/lineage refs, `caveats`. | BI export readiness, stale projections, lineage completeness, caveated rows by category. | Not implemented as durable snapshots. Add read-model tables/views before calling BI production-ready. |

## Minimum useful logs

Every structured log should be useful without exposing broad payloads. The first implementation target is JSON `tracing` fields, not a vendor-specific monitoring stack.

Required API log fields:

- `event="api_request"`
- `request_id`
- `correlation_id` when the route creates/loads workflow evidence
- `http.method`, `http.route`, `status_code`, `duration_ms`
- `actor_kind`, `actor_role`, `actor_id` when auth/session exists; until then, explicit `actor_source="payload_or_unset"`
- `location_id` and future `tenant_id` when known
- `workflow_event_id`, `review_packet_id`, `approval_record_id`, `outbox_record_id` when the route touches them
- `subject_ref` and `source_ref` only as safe references
- `safe_error_class`
- `payload_logging="disabled"` or `payload_logging="redacted_summary_only"`

Required worker log fields once durable leasing exists:

- `event="workflow_event"` or `event="outbox_candidate"` / `event="outbox_blocked"`
- `job_id` or durable lease id
- `workflow_event_id`, `correlation_id`, `review_packet_id`, `approval_record_id`, `outbox_record_id`
- `worker_id`, `attempt_count`, `lease_age_ms`, `runtime_mode`, `side_effect_mode`
- `review_gate`, `blocked_action`, `safe_error_class`
- `duration_ms`, `status`

Never log ordinary raw payloads containing customer contact details, document contents, OCR text, incident narratives, raw provider JSON, payment payloads, secrets, hidden prompts, signed URLs, or full message bodies. Prefer object refs, evidence refs, redacted summaries, hashes, and typed issue refs.

## Minimum useful durable audit events

The durable `audit_events` path should be append-only and product/security meaningful. It is not a substitute for high-volume request metrics.

Audit these transitions:

1. Workflow event accepted/rejected.
2. Review packet created/updated/cancelled.
3. Approval requested/approved/rejected/cancelled/superseded.
4. Outcome recorded or corrected by reviewed process.
5. Side-effect request blocked.
6. Outbox candidate created/claimed/published/failed/dead-lettered/replayed, once durable worker exists.
7. Source-quality issue created/acknowledged/repaired/suppressed/superseded.
8. Import run started/completed/failed and source snapshot mapped/rejected, once imports exist.
9. BI snapshot generated/exported/invalidated, once BI snapshots exist.
10. Privileged raw-evidence access or export, only through an approved future access-control path.

Minimum audit fields:

- `audit_event_id`
- `event_name` from the taxonomy above, or a workflow-specific sub-action under it
- `occurred_at`, `recorded_at`
- `request_id`, `correlation_id`, `workflow_event_id`
- `actor_kind`, `actor_id`, `actor_role` / `reviewer_role`
- `location_id`, future `tenant_id`
- `subject_ref`, `source_ref` / `source_refs`
- `review_packet_id`, `approval_record_id`, `outbox_record_id` when applicable
- `blocked_action`, `blocked_reason` when applicable
- `outcome_disposition`, `estimated_minutes`, `actual_minutes` when applicable
- `safe_error_class` for failures
- `metadata` limited to safe refs and redacted summaries

Current `audit_events` has append-only protection, actor/subject/action/workflow metadata, and should be extended by code cards rather than replaced.

## Business metrics and BI read-model contract

These are product metrics for operators and BI. They should be queryable as read models or BI snapshots with lineage and caveats; they are not Prometheus counters.

| Read model / metric | Key dimensions | Measures | Source lineage | Current state / add next |
| --- | --- | --- | --- | --- |
| `source_quality_backlog` | `location_id`, `tenant_id`, `source_system`, `issue_kind`, `severity`, `sensitivity`, `freshness`, `workflow_blocking`, `owner_role`. | Open/acknowledged/repaired/suppressed counts, age, affected entity count, blocked workflow count. | `source_ref`, `issue_ref`, `workflow_event_id`, import/sync refs when present. | Data-Quality Hygiene proves issue/source refs; add durable source-quality issue table/read view. |
| `review_queue_aging` | `location_id`, `review_gate`, `workflow_name`, `blocked_action`, `reviewer_role`, status. | Queue count, oldest age, SLA buckets, approval/rejection rates. | `review_packet_id`, `workflow_event_id`, `source_ref`, audit refs. | Review packets exist; add durable query and aging projection. |
| `labor_outcomes` | `location_id`, `operating_day`, `workflow_name`, `action_kind`, `reporting_group`, `actor_role`, `outcome_disposition`. | Outcome count, completed/deferred/suppressed/source-wrong/not-actionable count, estimated minutes, actual minutes, estimated minutes saved. | `workflow_event_id`, `approval_record_id`, `source_refs`, `action_id`, `correlation_id`. | Present for Manager Daily Brief and Data-Quality Hygiene; create shared projection/view. |
| `outbox_posture` | `location_id`, `topic`, `blocked_action`, `review_gate`, `status`, `safe_error_class`. | Pending/claimed/published/failed/dead-letter counts, retry count, oldest pending age, approved-but-stubbed count. | `outbox_record_id`, `approval_record_id`, `workflow_event_id`, audit refs. | Schema exists; worker leasing/retry/dead-letter read model is missing. |
| `audit_lineage` | `correlation_id`, `request_id`, `workflow_event_id`, `subject_ref`, `source_ref`. | Ordered lifecycle from request -> workflow -> review -> approval -> outcome/outbox -> audit. | Direct IDs from each durable row. | Audit table exists; add complete correlation fields and query route. |
| `occupancy_service_demand` | `location_id`, `operating_day`, `service_line`, reservation status, source caveat. | Demand counts, reservation/service counts, capacity caveat counts, source freshness caveats. | Reservation/source refs, import/sync refs, data-quality issue refs. | Core reservation schema exists; BI-ready projection and source caveats are missing. |
| `import_freshness` | `location_id`, `source_system`, `adapter_version`, import mode. | Last successful import time, failed import count, record counts, rejected records, drift flags. | `import_run_id`, source snapshot refs, sync gaps. | Deferred by current schema. Add before BI depends on source freshness. |
| `BI_snapshot_manifest` | `projection_name`, `projection_version`, `generated_at`, `location_id`, `tenant_id`, freshness/caveat class. | Row count, caveated row count, stale-source row count, lineage coverage percent. | Snapshot id plus source/workflow/audit refs. | Not implemented. Add if BI exports need durable handoff files/tables. |

BI export/read-model responses should include:

- `projection_name`
- `projection_version`
- `generated_at`
- `freshness` / `source_cutoff_at`
- `location_id` and future `tenant_id`
- row-level or aggregate `source_refs` / lineage refs
- `workflow_event_id`, `review_packet_id`, `approval_record_id`, `outbox_record_id`, `audit_event_id` where applicable
- `caveats` such as `source_stale`, `review_pending`, `mapping_uncertain`, `live_side_effects_disabled`, `raw_payload_redacted`

This is the handoff that reduces BI scraping burden: analysts should query product-owned `source_quality_backlog`, `labor_outcomes`, `review_queue_aging`, `outbox_posture`, `audit_lineage`, `occupancy_service_demand`, `import_freshness`, and `BI_snapshot_manifest` instead of reverse-engineering Gingr/provider rows.

## Infrastructure telemetry contract

Infrastructure telemetry should remain low-cardinality and safe. It should support engineering operations without becoming a business truth source.

Minimum counters/gauges/histograms once durable services exist:

- `api_request_count{route,status,safe_error_class}`
- `api_request_duration_ms{route,status}`
- `workflow_event_count{workflow_name,event_kind,status}`
- `workflow_job_queue_depth{workflow_name,status}`
- `workflow_job_lease_age_ms{workflow_name}`
- `workflow_job_attempt_count{workflow_name,status}`
- `dead_letter_count{workflow_name,safe_error_class}`
- `outbox_count{topic,status,review_gate}`
- `outbox_retry_count{topic,safe_error_class}`
- `audit_write_failure_count{safe_error_class}`
- `import_run_count{source_system,status,safe_error_class}`
- `sync_gap_count{source_system,gap_kind,severity}`
- `object_storage_quarantine_count{scan_status,redaction_status}`
- `agent_runtime_validation_failure_count{workflow_name,safe_error_class}`
- `review_needed_count{workflow_name,review_gate,blocked_action}`

Do not put raw ids, customer names, emails, message bodies, provider payload fragments, prompt text, signed URLs, or secrets in metric labels. High-cardinality ids belong in logs/audit/read-model queries, not metric labels.

## Route/schema/storage implementation handoff

Downstream route, schema, and storage cards should add these exact fields/events in small safe slices.

### API route cards

1. Add an extractor/middleware for `request_id` and `correlation_id`.
   - Accept safe `x-request-id`; echo it.
   - Accept safe `x-correlation-id` or route payload correlation id when present; otherwise generate one.
   - Store both in request extensions and include both in workflow DTO metadata.
2. Extend `api_request` spans with `status_code`, `duration_ms`, `safe_error_class`, `correlation_id`, `actor_role`, `location_id`, `tenant_id` placeholder, `workflow_event_id` when available, and `payload_logging="disabled"`.
3. Ensure every command response that creates work returns `request_id`, `correlation_id`, `workflow_event_id`, and any `review_packet_id`/`approval_record_id`/`outbox_record_id` generated.
4. Add typed error responses with `safe_error_class` and no raw sensitive details.
5. Keep `/ops/metrics/summary` aggregate-only; add new read-model endpoints rather than overloading it with raw rows.

### Storage/schema cards

Recommended additions or migration extensions:

- Add `request_id`, `correlation_id`, `location_id`, optional `tenant_id`, `actor_kind`, `actor_id`, `actor_role`, `source_refs`, and `safe_error_class` to workflow/audit/outcome tables where absent.
- Add `workflow_jobs` when durable leasing is implemented: `id`, `workflow_event_id`, `status`, `lease_owner`, `lease_expires_at`, `attempt_count`, `next_run_at`, `last_error_class`, `last_error_summary`, `dead_letter_reason`, `created_at`, `updated_at`.
- Add durable `source_quality_issues` or a source-quality read-model table: `issue_ref`, `source_ref`, `location_id`, `tenant_id`, `issue_kind`, `severity`, `freshness`, `sensitivity`, `affected_entity_ref`, `resolution_status`, `workflow_event_id`, `created_at`, `updated_at`.
- Add `import_runs`: `import_run_id`, `source_system`, `adapter_version`, `location_id`, `tenant_id`, `status`, `started_at`, `completed_at`, `record_count`, `rejected_count`, `safe_error_class`, `redaction_posture`.
- Add `sync_gaps`: `sync_gap_id`, `source_system`, `source_ref`, `location_id`, `tenant_id`, `gap_kind`, `severity`, `detected_at`, `age_seconds`, `status`, `workflow_event_id`, `safe_error_class`.
- Add read-model views/tables for `source_quality_backlog`, `review_queue_aging`, `labor_outcomes`, `outbox_posture`, `audit_lineage`, and `BI_snapshot_manifest`.

### Worker cards

1. Log every lease attempt with `workflow_event_id`, `job_id`, `correlation_id`, `attempt_count`, `lease_owner`, `runtime_mode`, `side_effect_mode`, and `safe_error_class`.
2. On validation failure, write an audit event and dead-letter or retry evidence with `safe_error_class` and redacted summary.
3. On outbox candidate processing, distinguish `outbox_candidate` from `outbox_blocked`; blocked/stubbed live effects are success for safety, not silent failure.
4. Keep fake/disabled runtime modes explicit in logs/readiness/metrics until live runtime/adapters are separately approved.

## Safety and non-overclaim rules

- Do not claim production monitoring, dashboards, Prometheus, OpenTelemetry, Sentry, alerting, or live adapter observability until the code actually wires them.
- Do not treat business metrics as raw provider truth. Reviewed outcomes and source-quality caveats must remain visible.
- Do not treat infrastructure telemetry as BI source of record. Metrics can tell operators something broke; read models tell BI what happened in the business workflow.
- Do not log or export raw documents, OCR text, message bodies, incident narratives, raw provider JSON, payment payloads, secrets, hidden prompts, signed URLs, or real customer/provider data.
- Environment variables alone must not enable live customer sends, provider writes, payment moves, schedule/capacity changes, or medical/safety decisions.
- Every BI export/read model must carry projection version, freshness, lineage, and caveats.

## Acceptance checklist for code cards

A code card that claims this contract is implemented for a route or workflow should show:

- `request_id` and `correlation_id` appear in response metadata, route logs, workflow/audit rows, and tests.
- `workflow_event_id`, `review_packet_id`, `approval_record_id`, `outbox_record_id`, and `audit_event_id` are linked when applicable.
- `source_ref`, `location_id`/tenant placeholder, `actor_role`/reviewer role, `blocked_action`, `outcome_disposition`, `estimated_minutes`, and `actual_minutes` are present for the workflow.
- Business read model is separate from infrastructure metrics.
- No broad payload logging or high-cardinality/sensitive metric labels were added.
- Tests cover successful path, blocked side effect, safe error class, and no production-observability overclaim.
