# Gingr adapter to owned API migration map

Status: migration-planning artifact. This page documents how Gingr fits as a legacy/source adapter while NVA moves BI and operator workflows toward an owned Pet Resorts operations API. It does not implement routes, create database tables, use live NVA/Gingr credentials, claim production readiness, authorize provider/PMS writes, send customer/member messages, move payments/refunds/discounts, change schedules/capacity, approve medical/safety decisions, or deploy anything.

Source context: [Gingr provider boundary atlas](provider-boundary-atlas.md), [Gingr source inventory](source-inventory.md), [Gingr SDK read-only endpoint catalog](sdk-endpoint-catalog.md), [Gingr BI read-model contract](bi-read-model-contract.md), [adapter boundary and labor-source expansion points](adapter-boundary-and-labor-source-expansion.md), [Gingr SDK architecture](sdk-architecture.md), [Gingr SDK webhook contract catalog](sdk-webhooks.md), [`integrations/gingr` crate README](../../../integrations/gingr/README.md), [owned operations API replacement thesis](../../architecture/owned-operations-api-replacement.md), [owned operations API contract families](../../architecture/owned-operations-api-contract.md), and [owned API storage and BI read-model cutline](../../architecture/owned-api-storage-read-model-cutline.md).

## Migration thesis

NVA should not migrate from Gingr by cloning Gingr. Gingr remains useful during migration because it can provide observed source evidence: endpoint request shapes, provider IDs, raw response envelopes, verified webhook facts, and fixture-backed DTO fragments. The owned API should own the product contract: source refs, review packets, outcomes, audit, labor metrics, and BI/read-model projections that answer NVA operating questions.

The intended replacement path is:

```text
Gingr extract workaround
  -> Gingr read-only/source adapter
  -> source snapshots + provenance + data-quality issues
  -> owned API workflow/write model
  -> owned BI/read models and operator queues
  -> reduced Gingr dependency per workflow
```

The current BI workaround is evidence that teams already need normalized read models Gingr does not provide cleanly. The migration should move those read-model semantics upstream into the owned API rather than preserving raw provider tables as the public contract.

## Current source surfaces NVA can safely treat as adapter evidence

These surfaces are safe to name as source evidence because they are documented in repo docs/source or encoded by the `integrations/gingr` boundary. They are not automatically product schemas.

| Source surface | Evidence available today | Adapter use during migration | Owned API destination |
| --- | --- | --- | --- |
| Config and transport | Validated Gingr base URL/subdomain, API-key wrapper, redacted request capture, mock transport; live `HttpTransport` is not implemented in this slice. | Build redacted request diagnostics and fixture-safe tests without printing secrets or claiming live access. | `source_import_runs`, adapter mode/readiness, audit metadata, safe error classes. |
| Reservation and stay reads | `reservations`, `reservation_widget_data`, `reservations_by_animal`, `reservations_by_owner`, `reservation_types`, `get_services_by_type`, `back_of_house`; 30-day reservation range and API-user/location-scope caveats are documented. | Import reservation/stay observations with endpoint/window/location provenance and typed ambiguity. | Owned reservation/stay evidence, `source_quality_issues`, occupancy/service-demand read models, review packet inputs. |
| Owner, animal, forms, and care reads | `owner`, `owners`, `animals`, `forms_get_form`, `custom_field_search`, `get_feeding_info`, `get_medication_info`; phone/email/custom-field search values are sensitive. | Produce customer/pet/care source candidates and data-quality issues while quarantining high-PII payloads. | Owned customer/pet/care resources, affected entity refs, source-quality backlog rows, medical/care review queues. |
| Reference data | Locations, species, breeds, vets, temperaments, immunization types, animal immunizations. | Support normalization of location/service/care/vaccine context when mapping is source-backed. | Owned reference dimensions and review caveats, not provider vocabulary as policy. |
| Commerce, retail, revenue, and subscriptions | Retail items, transactions, invoices, subscriptions; transaction details are payment-sensitive; transaction/invoice cutover behavior is encoded. | Preserve revenue/payment/subscription evidence under quarantine and create reconciliation issues where semantics are unclear. | Revenue/payment/deposit projections and checkout/revenue read models only after approved field semantics and safety gates. |
| Labor and report-card files | `timeclock_report` and `report_card_files` request builders. | Use as labor/file activity evidence when a source ref and redaction posture are explicit. | Labor outcome/read-model inputs and care-update evidence; not payroll authority or customer-send authority. |
| Webhooks | Parsed envelopes, HMAC verification, event/entity normalization, acknowledgement policy, and sanitized fixtures. | Accept only verified event evidence; reconcile webhook facts against pulls before driving workflow packets. | Workflow event inputs, import freshness, audit lineage, and stale/gap detection. |
| Narrow mappings | Owner contact candidate, pet name candidate, retail product candidate; unsupported grooming/training/service DTOs are explicit provider-surface gaps. | Promote only fields with narrow mappers and typed errors. | Product-owned resources and review candidates with source refs and caveats. |

## Unsupported or not-yet-authoritative surfaces

Keep these out of public API schemas and BI truth until real access, fixtures, or owner decisions prove their grain and semantics:

- Undocumented or partial provider response fields beyond current `response::*` and DTO coverage.
- Grooming/training/service DTOs not modeled by source-backed provider schemas; current docs intentionally mark them as gaps.
- `quick_checkin` and `receive_call`, which are excluded from the read-only SDK surface because they have side effects.
- Customer password authorization flows (`authorize_owner`) in normal data ingestion; they are sensitive credential checks, not source-import primitives.
- Raw owner/animal/custom-field/payment payloads in broad logs, BI exports, prompts, or public API DTOs.
- Provider status strings as final lifecycle states unless an explicit mapping documents the NVA meaning.
- Provider IDs as canonical NVA IDs without source refs, identity reconciliation, and review caveats.
- BI tables, exports, or existing ETL code as truth before their grain, refresh behavior, redaction posture, and metric definitions are inspected.
- Live writes, customer sends, provider/PMS changes, schedule/capacity changes, payment movement, medical/safety approvals, or production deployment paths.

## Adapter quarantine rules

The migration should quarantine Gingr in the adapter/source layers so owned API consumers do not couple to provider shapes.

1. Raw Gingr records stay under `integrations/gingr` or restricted source snapshot storage. Normal operator/API responses should expose source refs, not raw payloads.
2. Every imported observation carries source system, endpoint/event/report name, provider record ID, request scope, observed/pulled/received time, adapter/schema version, payload hash/ref when retained, and redaction posture.
3. Unknown, missing, conflicting, stale, high-sensitivity, or unsupported fields become `source_quality_issues`, `sync_gaps`, typed assumptions, or review blockers. They do not become silent defaults.
4. Mapping functions promote narrow facts into candidates only when required provider fields and domain validations pass. A candidate is review evidence, not final customer/pet/reservation truth by itself.
5. Webhooks must pass signature verification and then reconcile with source pulls or snapshots before they affect workflow state.
6. Payment-sensitive, medical/care-sensitive, owner/animal custom-field, phone/email lookup, password, and raw document/file payloads are restricted by default.
7. App/API/storage code consumes source-agnostic contracts, workflow packets, outcomes, and read models. It should not import provider DTOs directly.
8. Any future live action path must be separate from the read adapter: workflow packet -> review gate -> approval record -> outbox record -> approved adapter execution -> outcome/audit/reconciliation.

## Migration phases

### Phase 0: Inventory and boundary proof

Goal: preserve evidence and make unknowns visible.

- Keep public/current Gingr docs, fixtures, endpoint catalog, webhook catalog, and crate tests as the evidence base.
- Maintain `integrations/gingr` as a provider adapter: config, request builders, transport redaction, raw response wrappers, webhook verification, DTO gaps, and narrow mappings.
- Do not use live credentials or production data in repo docs/tests.
- Output for presentation packet: “Gingr can be read as evidence; it is not the target product model.”

### Phase 1: Source import and snapshot spine

Goal: make source observations replayable and auditable.

- Add/import through `source_import_runs`, optional `source_record_snapshots`, and source refs with adapter version, endpoint/event, request window, location scope, observed time, payload hash/ref, and redaction status.
- Start with fixture/synthetic data, then use approved read-only access later if granted.
- Record provider gaps as first-class issues: missing required field, unknown status, location-scope ambiguity, payment-state conflict, sensitive payload quarantined, stale import, and sync gap.
- Owned API surface: source/import metadata and source-quality backlog, not raw Gingr mirror tables.

### Phase 2: Owned Data-Quality Hygiene v0

Goal: replace one BI cleanup loop with an owned workflow/read model.

- Use Data-Quality Hygiene as the first slice because it directly turns BI pain into source-quality issues, cleanup drafts, reviewed outcomes, labor minutes, and audit lineage.
- Persist workflow events, review packets, approval/outcome rows, audit events, and internal/stubbed outbox posture behind the existing `WorkflowRepository` seam.
- Publish/read `source_quality_backlog`, `data_quality_hygiene_labor_outcomes`, `audit_lineage`, and `import_freshness` views or API read-model endpoints.
- Keep provider repair, destructive merge/delete, customer sends, payment/schedule/medical/safety actions disabled.

### Phase 3: Operator workflow expansion

Goal: move from source-data cleanup to useful operations queues.

- Add Manager Daily Brief, checkout exception review, inquiry intake, vaccine/document review, and care-update drafts only where owned workflow packets and review gates exist.
- Feed these workflows from source refs, normalized operational resources, and read models; do not interpret raw Gingr status strings in validators.
- Capture reviewed outcomes, actual labor minutes, source-fact correctness, and blocked-action reasons.
- BI consumes read models for review queue aging, labor outcomes, occupancy/service demand, outbox posture, and audit lineage.

### Phase 4: Dual-run with the existing BI workaround

Goal: prove owned read models answer BI/operator questions better than raw extracts.

- Run Gingr imports/read models alongside the existing BI path once NVA grants safe read access or redacted exports.
- Compare row counts, freshness, location scope, status mapping, duplicate/merge behavior, revenue/payment caveats, service-line categorization, and issue visibility.
- Keep reconciliation reports and caveats explicit. A mismatch is a data-quality issue or owner question, not an excuse to guess.
- Output for presentation packet: “BI can warehouse owned read models with lineage instead of reverse-engineering raw provider tables.”

### Phase 5: Scoped workflow replacement

Goal: reduce Gingr dependency for one operational job at a time.

- For a scoped workflow, staff uses the owned API/UI queue as the primary review/outcome surface while Gingr remains read/source evidence or approved system of record where required.
- Cutover requires owner-approved success criteria, role/location authorization, retention/redaction policy, monitoring, fallback, and a no-live-side-effect or approved-live-adapter posture.
- Provider shrinkage happens only after legal/ops/security/business review and after an approved path exists for any necessary writeback or alternate system-of-record update.

## BI replacement path

The replacement claim should be narrow at first: the owned API can replace the BI workaround for one workflow once it exposes supported read models with lineage and caveats. It should not claim that Gingr is fully replaced.

| Existing BI pain | First owned read-model answer | Cutover evidence needed |
| --- | --- | --- |
| Analysts infer dirty source records from raw Gingr pulls. | `source_quality_backlog` with source refs, issue kind, severity, freshness, sensitivity, owner/reviewer role, workflow-blocking status, resolution status, latest outcome, and caveats. | Approved source-import feed, issue taxonomy, and comparison against current BI defects. |
| Labor savings are hard to prove. | `data_quality_hygiene_labor_outcomes` and later shared `labor_outcomes` with reviewed dispositions, action kinds, estimated/actual minutes, source-fact correctness, and reporting groups. | Outcome definitions and pilot data validated by operations/BI owners. |
| Queue urgency is recomputed in spreadsheets or reports. | `review_queue-aging` and workflow-specific review queues with gate, status, age, affected entity, issue count, and blocked actions. | Review-gate ownership, SLA/aging definitions, and role/location auth. |
| Audit trail requires stitching logs/exports/staff memory. | `audit_lineage` by correlation/request/workflow/review/approval/outcome/outbox/audit IDs. | Durable correlation fields and append-only audit coverage. |
| Import freshness and missing coverage are unclear. | `import_freshness` with last successful import, failed/rejected counts, record counts, stale age, gap counts, adapter version, and redaction posture. | Real import schedule, retention policy, and expected coverage by location/source surface. |
| Occupancy/service/labor questions need normalized operations meaning. | `occupancy-service-demand`, Manager Daily Brief read models, and later revenue/labor projections over owned resources plus source refs. | Validated reservation/status/service-line mappings and owner-approved metric definitions. |

## First fields and workflows to validate with real NVA access

When NVA provides approved read-only access, validate these before expanding mapping or claiming BI replacement:

1. Reservation/stay grain: provider reservation ID stability, owner/animal/location/reservation type/service IDs, check-in/check-out/start/end/cancel/complete timestamps, raw status values, checked-in/cancelled/confirmed/completed flags, and back-of-house run/area/event-time fields.
2. Location scope: which endpoints are constrained by API-user logged-in location, which accept `location_id`, and how multi-location tenants expose or hide records.
3. Owner/pet identity: duplicate/merged/recreated owner and animal records, owner-pet relationship stability, phone/email availability, custom-field payload sensitivity, and redaction needs.
4. Service-line mapping: reservation types, service types, add-ons, grooming/training/daycare/boarding meanings, retail item status, and location-specific naming differences.
5. Payment/revenue caveats: invoice/estimate/transaction/subscription references, 2019-08-01 transaction/invoice cutover behavior, deposit/payment state semantics, and payment-sensitive retention rules.
6. Care/vaccine/document facts: immunization records, feeding/medication fields, report-card file metadata, form/custom-field structures, and which fields require medical/safety review.
7. Timeclock/labor facts: timeclock report grain, user/role/location mapping, deleted/currently-clocked-in behavior, and whether it can support labor-outcome validation without becoming payroll authority.
8. Webhook freshness: which events are enabled, signature behavior, retry behavior, entity ID consistency, whether webhooks update BI freshness today, and how webhook events reconcile to pull snapshots.
9. Existing BI path: database engine/warehouse, source endpoints/reports/exports used, raw vs normalized vs fact/dimension tables, refresh cadence, late-arriving changes, rejected rows, and code that constructs current BI-facing types.
10. Owner decisions: which BI/operator outputs matter first, who owns KPI definitions, which review gates are required, retention/redaction expectations, and what counts as successful replacement for a pilot workflow.

## Open questions for the final presentation packet

- Which current BI questions are highest-value for the first owned read model: source-quality backlog, labor outcomes, review queue aging, occupancy/service demand, revenue/payment reconciliation, import freshness, or audit lineage?
- What exact Gingr surfaces feed the existing BI database: API endpoints, reports, exports, webhooks, manual exports, or mixed jobs?
- What is the current BI database/warehouse engine, refresh cadence, failure mode, and retention/redaction posture?
- Which provider IDs are stable across endpoints, webhooks, historical backfills, invoices/payments, duplicates, and merged records?
- Which statuses, service names, reservation types, payment states, custom fields, and location names have local or overloaded meanings?
- Who owns approval for data-quality cleanup, customer/pet profile corrections, care/vaccine review, payment/revenue interpretation, schedule/capacity decisions, and customer sends?
- Which live actions must remain impossible in v0, which can become reviewed outbox candidates later, and which will never belong in the owned API?
- What redacted fixture/export can be used to validate the first Data-Quality Hygiene import and read-model projection without exposing real customer/provider data?

## Handoff summary for downstream cards

- Do not add Gingr DTOs to owned public API schemas. Add source refs, source import metadata, issue records, workflow packets, outcome records, audit lineage, and read models.
- First implementation target: Data-Quality Hygiene persistence/read models behind owned API contracts, using Gingr only as source evidence.
- First read models to wire: `source_quality_backlog`, `data_quality_hygiene_labor_outcomes`, `audit_lineage`, and `import_freshness`.
- First real-access validation: reservation/stay grain, location scope, owner/pet identity, service-line mapping, payment/revenue caveats, care/vaccine/doc fields, timeclock grain, webhook freshness, and current BI construction path.
- Honest current limit: repo evidence supports the migration architecture and fixture-safe adapter boundary, not production BI replacement or live Gingr operation.

## Verification

For this planning artifact:

```sh
./scripts/check_docs.sh
python scripts/check_markdown_links.py --repo-root .
```
