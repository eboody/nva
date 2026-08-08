# Owned API storage and BI read-model cutline

Status: architecture/storage plan for the owned Pet Resorts operations API. This page defines the operational write model, BI/read-model cutline, first Data-Quality Hygiene persistence slice, and exact migration/storage gaps for downstream implementation. It does not implement tables or repository code, replace the existing migration wholesale, adopt Toasty/ORM, claim live NVA/Gingr access, mutate provider/PMS records, send customers/member messages, move payments/refunds/discounts, change schedules/capacity, approve medical/safety decisions, or deploy anything.

Source context: [owned operations API contract families](owned-operations-api-contract.md), [owned API observability/audit/metrics contract](owned-api-observability-metrics-contract.md), [storage persistence crosswalk](../entity-atlas/contract-crosswalk/storage-persistence.md), [Data Quality Hygiene workflow](../workflows/operator/data-quality-hygiene.md), [foundation migration](../../migrations/0001_mvp_foundation.sql), [migration contract tests](../../storage/tests/mvp_migration_contract.rs), [storage README](../../storage/README.md), [storage operations records](../../storage/src/operations.rs), [API HTTP shell](../../apps/api/src/http.rs), and [DTO/API/DB/observability gap map](../audits/dto-api-db-observability-readiness-gap-map.md).

## Cutline thesis

The owned API should persist product-owned operational truth and publish BI-friendly read models. Gingr/provider records are source evidence during migration, not the database shape NVA should ask BI to reverse-engineer.

The cutline is:

```text
provider/source observations and staff inputs
  - adapter/import evidence, source refs, raw payload refs, staff-entered corrections
  - authority: what was observed, by which source/adapter, at which time
        |
        v
owned operational write model
  - normalized workflow events, review packets, approvals, outcomes, audit, internal outbox
  - authority: review-gated product state and append-only evidence
        |
        v
BI/read-model layer
  - source-quality backlog, reviewed labor outcomes, review queue aging, audit lineage, import freshness
  - authority: queryable projections with lineage, freshness, caveats, and no live mutation authority
```

A write table is where the API records product actions and review evidence. A read model is a query/product analytics projection over those write rows. A provider import table is evidence about source observations and gaps, not product authority by itself.

## Clear answer: what DB/API makes the BI workaround obsolete for one workflow?

For Data-Quality Hygiene, the current BI workaround becomes unnecessary when BI can stop scraping Gingr/source tables to infer dirty data and instead call/query these owned surfaces:

1. `GET /read-models/source-quality-backlog?location_id=&operating_day=&issue_kind=&severity=&workflow_blocking=`
   - backed by durable `source_quality_issues`, `source_import_runs`, `source_record_snapshots`, `sync_gaps`, `workflow_events`, review status, and issue/outcome linkage.
   - returns issue refs, affected entity refs, source refs, field paths, source freshness, severity, sensitivity/redaction posture, owner/reviewer role, resolution status, workflow blocking status, age, and caveats.
2. `GET /data-quality-hygiene/outcomes/summary?location_id=&operating_day=&correlation_id=` and `GET /read-models/labor-outcomes?workflow=data-quality-hygiene&...`
   - backed by `data_quality_hygiene_outcomes`, `workflow_events`, `review_packets`, `approval_records`, and `audit_events`.
   - returns reviewed dispositions, source-wrong/not-actionable/suppressed counts, actual minutes, estimated minutes saved, issue refs, source refs, reviewer/actor roles, and approval lineage.
3. `GET /read-models/audit-lineage?correlation_id=`
   - backed by request/workflow/review/outcome/outbox/audit IDs.
   - returns the ordered trace of which request created the workflow, which review packet and approval gate governed it, which outcome was recorded, and which internal outbox candidate was allowed or blocked.

That API/DB makes the first BI patch database obsolete for this workflow because the supported query is no longer: "pull raw Gingr records and guess which facts are missing/stale/conflicting." The supported query becomes: "read product-owned source-quality issues and reviewed cleanup outcomes with source lineage and caveats." BI can still warehouse those read models, but it is not inventing the product semantics downstream.

## Current storage baseline to preserve

The current foundation is useful and should be extended in small migrations, not replaced wholesale.

### Current SQL write-model spine

`migrations/0001_mvp_foundation.sql` already defines:

- Operational core: `locations`, `customers`, `pets`, `reservations`, `reservation_pets`, `documents`, `object_metadata`, `vaccine_records`, `vaccine_extractions`, `pet_eligibility_projections`, `operational_tasks`, `care_notes`, `incidents`, `messages`, `payment_deposit_projections`.
- Workflow/review/audit: `workflow_events`, `workflow_results`, `review_packets`, `approval_records`, `outbox_records`, `audit_events`.
- Owned labor/outcome projections: `manager_daily_brief_outcomes`, `data_quality_hygiene_outcomes`.
- Safety invariants: canonical `review_gate_is_valid()` values, approval decision integrity, outbox approval matching, open-outbox approval demotion guard, append-only audit triggers.

`storage/tests/mvp_migration_contract.rs` locks those table names and invariants. Downstream work should add a follow-on migration plus tests rather than editing away this baseline.

### Current Rust storage/API surface

`storage/src/operations.rs` already has useful storage records/codes/codecs:

- `StoredSourceRecordRef` for provider/source evidence pointers.
- `DataQualityHygieneOutcomeRecord`, `DataQualityHygieneOutcomeSummary`, `DataQualityHygieneReportingGroup`, `DataQualityHygieneOutcomeCode`, `DataQualityHygieneActionKindCode`, `DataQualityResolutionStatusCode`, `StoredDataQualityHygieneLaborMinutes`.
- `DataQualityHygieneLocalPersistenceRecords` with storage-shaped `WorkflowEventRecord`, `WorkflowResultRecord`, `ReviewPacketRecord`, `ApprovalRecordRow`, `DataQualityHygieneOutcomeRow`, `AuditEventRecord`, and optional `OutboxRecord` for a reviewed internal handoff slice.
- `ManagerDailyBriefOutcomeRecord` and related labor/reporting codes for the next workflow.

`apps/api/src/http.rs` already exposes Data-Quality Hygiene context/draft/outcome/summary routes and an in-memory `WorkflowRepository` seam. The seam is the correct replacement point for a future Postgres repository; the HTTP handlers and review gates should not be rewritten around raw provider shapes.

## Write model vs read model

### Operational write model

The operational write model should store facts only where NVA owns the workflow semantics:

| Family | Existing table/record | Authority | First gap to close |
| --- | --- | --- | --- |
| Workflow intake | `workflow_events`, `WorkflowEventRecord` | accepted owned workflow event with idempotency/source payload | add `request_id`, `correlation_id` consistency, `location_id`, actor/reviewer role fields, and indexes by workflow/location/day. |
| Worker/reviewable output | `workflow_results` | reviewable worker/app output, not live execution proof | add durable `workflow_jobs` later for leasing/retry/dead-letter; keep fake/disabled mode explicit. |
| Review queue | `review_packets` | human/policy review packet before risky action | add BI aging indexes/view by gate/status/reviewer role/location/workflow. |
| Approval ledger | `approval_records` | reviewed decision with actor and timestamp | add correlation/location/actor_role fields or put them in audit/read view until migration extension. |
| Outcome/labor | `data_quality_hygiene_outcomes`, `manager_daily_brief_outcomes` | reviewed labor/outcome evidence | add shared labor read model; preserve workflow_event_id, approval_record_id, action_id, source_refs, issue_refs, outcome disposition. |
| Outbox | `outbox_records`, `OutboxRecord` | approved internal or future live candidate, never unreviewed send/write authority | add blocked/stubbed posture and worker lease/dead-letter metrics before live adapters. |
| Audit | `audit_events`, `AuditEventRecord` | append-only evidence of transitions | add complete correlation/request/source/outcome metadata and query/read model. |

### Source import and provenance model

The API needs source import tables before BI can trust freshness or coverage:

| Table/record to add first | Purpose | Minimal fields |
| --- | --- | --- |
| `source_import_runs` | proves source/import freshness and failure posture | `id`, `source_system`, `adapter_version`, `location_id`, nullable `tenant_id`, `mode`, `status`, `started_at`, `completed_at`, `record_count`, `rejected_count`, `safe_error_class`, `redaction_posture`, `created_at`. |
| `source_record_snapshots` | stores safe evidence pointers/hashes for imported source records without making provider JSON the API contract | `id`, `import_run_id`, `source_system`, `record_type`, `source_record_id`, `observed_at`, `payload_ref`, `payload_sha256`, `schema_version`, `redaction_status`, `mapped_entity_kind`, `mapped_entity_id`, `mapping_status`, `created_at`; store raw payload elsewhere only under redaction/access rules. |
| `source_quality_issues` | durable Data-Quality Hygiene backlog row | `id`/`issue_ref`, `source_snapshot_id` or `source_ref`, `location_id`, nullable `tenant_id`, `affected_entity_kind`, `affected_entity_id`, `field_path`, `issue_kind`, `severity`, `freshness`, `sensitivity`, `workflow_blocking`, `owner_role`, `review_gate`, `resolution_status`, `workflow_event_id`, `created_at`, `updated_at`, `resolved_at`. |
| `sync_gaps` | tracks expected source/read-model coverage failures | `id`, `source_system`, `source_ref`, `location_id`, nullable `tenant_id`, `gap_kind`, `severity`, `detected_at`, `age_seconds`, `status`, `workflow_event_id`, `safe_error_class`, `created_at`, `updated_at`. |

These rows must retain `StoredSourceRecordRef`-equivalent fields: `system`, `record_type`, `record_id`, `observed_at`, and `adapter_version`. They should also add payload hashes/refs when source evidence is imported, but should not expose raw provider payloads through ordinary BI APIs.

### BI/read-model layer

BI read models should be read-only projections over write/import rows. They can be SQL views, materialized views, or explicit projection tables; the first slice can use ordinary SQL views if freshness and lineage fields are included.

| Read model | Backing rows | Purpose | Minimum dimensions/measures |
| --- | --- | --- | --- |
| `source_quality_backlog` | `source_quality_issues`, `source_import_runs`, `source_record_snapshots`, `sync_gaps`, `workflow_events`, outcomes | BI/operator view of open and reviewed source defects | `location_id`, `source_system`, `issue_kind`, `severity`, `sensitivity`, `freshness`, `workflow_blocking`, `owner_role`, `review_gate`, `resolution_status`, `issue_age`, `source_refs`, `workflow_event_id`, `latest_outcome_id`, caveats. |
| `data_quality_hygiene_review_queue` | `review_packets`, `workflow_events`, `source_quality_issues`, `approval_records` | staff/ops queue for hygiene review | packet id, gate, status, owner/reviewer role, oldest age, affected entity, issue count, blocked actions, source refs. |
| `data_quality_hygiene_labor_outcomes` | `data_quality_hygiene_outcomes`, `workflow_events`, `approval_records`, `audit_events` | labor and reviewed disposition reporting | outcome counts, completed/deferred/suppressed/source-wrong/not-actionable counts, before/actual/saved minutes, action kind, operating day, location, actor persona, issue refs, source refs. |
| `audit_lineage` | workflow/review/approval/outcome/outbox/audit rows | correlation/debug/BI trust chain | correlation id, request id when available, workflow event id, review packet id, approval record id, outcome id, outbox id, audit event ids, subject/source refs, ordered timestamps. |
| `import_freshness` | `source_import_runs`, `source_record_snapshots`, `sync_gaps` | lets BI caveat stale/missing imports | last successful import, failed count, rejected count, record counts, stale age, gap counts, adapter version, redaction posture. |

Do not overload `/ops/metrics/summary` with row-level BI data. Keep it aggregate/local-runtime only; add `/read-models/...` or SQL BI surfaces separately.

## Data-Quality Hygiene rows and projections to implement first

The first persistence card should target one reviewed Data-Quality Hygiene workflow path, because that is the cleanest proof that owned storage/read models can replace a BI patch database without provider writes.

### First write rows

1. Add durable `source_quality_issues`.
   - Primary record/codecs: `DataQualityIssueRecord` in `storage::operations`.
   - Stable code enums: `DataQualityIssueKindCode`, `DataQualitySeverityCode`, `DataQualityFreshnessCode`, `DataQualitySensitivityCode`, `DataQualityResolutionStatusCode` (the existing resolution code can be reused), `AffectedEntityKindCode`, `ReviewGateCode` (already exists for local persistence records).
   - Source fields: `StoredSourceRecordRef` list, plus optional `source_snapshot_id` when imports are wired.
   - Required fields: `issue_ref`, `location_id`, `affected_entity_ref`, `field_path`, `issue_kind`, `severity`, `freshness`, `sensitivity`, `workflow_blocking`, `owner_persona`/`owner_role`, `review_gate`, `resolution_status`, `created_at`, `updated_at`.
2. Add `source_import_runs` and `sync_gaps` as minimal provenance/freshness rows.
   - They can be populated by fixtures/import adapters later; the schema/read models should exist before BI claims production freshness.
3. Wire a Postgres repository behind the existing `WorkflowRepository` seam for Data-Quality Hygiene only.
   - Persist `workflow_events`, `workflow_results`, `review_packets`, `approval_records`, `data_quality_hygiene_outcomes`, append-only `audit_events`, and approved internal `outbox_records` in one transaction for a completed reviewed outcome.
   - Preserve the existing in-memory implementation for tests/demos until the durable adapter is enabled explicitly.
4. Add read model/view `source_quality_backlog`.
   - It should be directly queryable by BI or exposed through `GET /read-models/source-quality-backlog` later.
5. Add read model/view `data_quality_hygiene_labor_outcomes` or fold into a shared `labor_outcomes` view with `workflow_name='data-quality-hygiene'`.

### First projections/API reads

Implement these before broader migrations:

- `source_quality_backlog`: open/reviewed source defects with source refs, affected entity, field path, severity, freshness, sensitivity, workflow-blocking, owner/reviewer role, resolution state, issue age, and latest reviewed outcome.
- `data_quality_hygiene_labor_outcomes`: reviewed Data-Quality Hygiene outcomes grouped by location/day/action/owner persona, with completed/deferred/suppressed/source-wrong/not-actionable counts and minutes.
- `audit_lineage`: one correlation ID -> workflow event -> review packet -> approval -> outcome -> outbox -> audit event chain.
- `import_freshness`: source import recency/failed/rejected/gap posture for caveating backlog rows.

## Exact migration/storage gaps for the first code slice

The next persistence card should not touch unrelated service-line migrations. It should add a narrow follow-on migration such as `migrations/0002_data_quality_read_models.sql` and update storage/API tests.

### Migration gaps

Add:

1. `source_import_runs` table.
2. `source_record_snapshots` table or defer payload snapshots if the card explicitly chooses source refs only; if deferred, document why `source_import_runs` + `source_quality_issues.source_refs jsonb` is enough for the slice.
3. `source_quality_issues` table with indexes:
   - `(location_id, resolution_status, severity)`
   - `(workflow_event_id)`
   - `(issue_ref)` unique
   - optionally GIN on `source_refs`/`issue_refs` if stored as JSONB.
4. `sync_gaps` table with indexes by `(location_id, status, severity)` and `(source_system, detected_at)`.
5. Views/materialized views:
   - `source_quality_backlog`
   - `data_quality_hygiene_labor_outcomes`
   - `audit_lineage`
   - `import_freshness`
6. Optional columns on existing tables only if needed for exact lineage:
   - `workflow_events.correlation_id` (currently payload carries it in local projection, but a real DB read model should index it).
   - `workflow_events.request_id`.
   - `review_packets.workflow_event_id` already exists.
   - `data_quality_hygiene_outcomes.correlation_id`, `workflow_event_id`, `approval_record_id`, `source_refs`, and `issue_refs` already exist.
   - `audit_events.workflow_event_id` already exists; correlation/request/source refs can remain in `metadata` for the first slice if the read model extracts them, then become columns later.

Do not remove or rewrite existing `0001_mvp_foundation.sql`. Do not adopt Toasty/ORM as part of this storage/read-model cutline.

### Rust storage gaps

Add or extend in `storage/src/operations.rs`:

- `DataQualityIssueRecord` with `decode_json`/`encode_json` and source-ref preservation.
- `SourceImportRunRecord`, `SourceRecordSnapshotRecord` if snapshots are in scope, and `SyncGapRecord`.
- Code enums matching migration check constraints: `SourceImportStatusCode`, `SyncGapStatusCode`, `SyncGapKindCode`, and data-quality issue/freshness/severity/sensitivity/affected-entity codes.
- Read-model DTO records if the API/storage crate owns response codecs: `SourceQualityBacklogRow`, `DataQualityHygieneLaborOutcomeReadModelRow`, `AuditLineageRow`, `ImportFreshnessRow`.
- Repository-facing row structs should remain storage-shaped and not provider-shaped; they may include `StoredSourceRecordRef` but must not embed raw Gingr DTOs.

Update tests:

- `storage/tests/mvp_migration_contract.rs` or a new `storage/tests/data_quality_read_model_migration_contract.rs` should assert the new table/view names, key fields, indexes, and no-live-side-effect comments/invariants.
- `storage/tests/data_quality_hygiene_outcome_storage.rs` should stay focused on outcome codecs; add separate tests for issue/import/sync/read-model records.
- Add API/repository integration tests only after the Postgres adapter exists; until then, keep API route tests honest about in-memory state.

### API/repository gaps

Use the existing `WorkflowRepository` seam in `apps/api/src/http.rs` rather than rewriting handlers around DB code:

- Introduce a trait implementation backed by Postgres/SQLx later, e.g. `PostgresWorkflowRepository` or module-local adapter. If SQLx is not already adopted in the repo slice, a follow-up card should decide exact crate/config; this card does not require picking an ORM.
- Persist one Data-Quality Hygiene completed outcome transactionally:
  - `workflow_events`
  - `workflow_results`
  - `review_packets`
  - `approval_records`
  - `data_quality_hygiene_outcomes`
  - `audit_events`
  - optional internal `outbox_records`
- Preserve blocked live effects in payload/audit/outbox metadata: `provider_writes_allowed=false`, `customer_messages_allowed=false`, `live_delivery_allowed=false`.
- Add read methods for source backlog and labor outcomes only after backing rows/views exist.

## Preservation rules

- Preserve source/provenance: every source-derived issue, outcome, backlog row, and BI projection must include source refs or a lineage path to source refs/import run/snapshot rows.
- Preserve review gates: data-quality cleanup can draft/recommend/measure, but cannot mutate Gingr/provider data, merge profiles, approve vaccines, hide ambiguity, move money, or send messages.
- Preserve audit/outbox safety: outbox candidates require approved approvals; audit remains append-only; live adapter enablement is a separate owner-approved design.
- Preserve caveats in BI: read models must expose `freshness`, `projection_version` or view version, `generated_at`/source cutoff where applicable, and caveats such as `source_stale`, `review_pending`, `mapping_uncertain`, `live_side_effects_disabled`, or `raw_payload_redacted`.
- Preserve storage/domain separation: storage records/codecs are persisted shapes; domain/app own business meaning; provider DTOs remain adapter evidence.

## Handoff to persistence implementation card

Recommended first code slice:

1. Create follow-on migration for Data-Quality Hygiene source/import/read-model rows and views:
   - `source_import_runs`
   - `source_quality_issues`
   - `sync_gaps`
   - `source_quality_backlog`
   - `data_quality_hygiene_labor_outcomes`
   - `audit_lineage`
   - `import_freshness`
2. Add storage records/codecs:
   - `DataQualityIssueRecord`
   - `SourceImportRunRecord`
   - `SyncGapRecord`
   - optional `SourceRecordSnapshotRecord`
   - `SourceQualityBacklogRow`
   - `DataQualityHygieneLaborOutcomeReadModelRow`
   - `AuditLineageRow`
   - `ImportFreshnessRow`
3. Extend migration contract tests with exact table/view/field assertions and no-live-side-effect invariants.
4. Keep API route behavior unchanged until the repository adapter is ready; when wiring, implement the durable repository behind `WorkflowRepository` and prove one completed Data-Quality Hygiene outcome writes workflow/review/approval/outcome/audit/internal-outbox rows transactionally.
5. Do not claim production BI replacement until source imports, freshness, retention/redaction policy, and BI stakeholder query definitions are owner-reviewed. The first safe claim is narrower: the owned API has the write/read-model cutline that would replace the BI workaround for Data-Quality Hygiene once wired.

## Verification

For this planning artifact:

```sh
./scripts/check_docs.sh
python scripts/check_markdown_links.py --repo-root .
```

For downstream implementation:

```sh
cargo test -p storage --test mvp_migration_contract
cargo test -p storage data_quality
cargo test -p pet-resort-api data_quality_hygiene
```

Add or adjust exact test names when the new migration and repository adapter land.
