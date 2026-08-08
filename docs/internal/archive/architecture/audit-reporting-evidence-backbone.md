# Audit, reporting, and evidence backbone

Status: architecture decision and job-contact explanation for the enterprise evidence/export backbone behind the realtime Data-Quality Hygiene demo. This page keeps the demo honest: SpacetimeDB can prove live review queues and subscriptions, but enterprise audit, history, reporting, export, and document/media evidence need explicit durable jobs before the architecture is production-shaped. It does not claim live NVA/Gingr/PMS access, production data handling, provider writes, member/customer sends, payment movement, schedule/capacity changes, medical/safety decisions, or production deployment.

Source context: the prior Postgres vs SpacetimeDB decision memo recommended a hybrid runtime: SpacetimeDB for realtime operations and Postgres/S3-compatible object storage for durable enterprise audit/reporting/reconciliation/evidence. The Data-Quality Hygiene realtime queue currently has private SpacetimeDB actor/role/scope, review/audit/outcome, and blocked-action rows plus public staff/manager/outcome/blocked-action read models. See also [owned operations API contract families](owned-operations-api-contract.md), [owned API storage and BI read-model cutline](owned-api-storage-read-model-cutline.md), [local demo walkthrough](../demo/local-demo-walkthrough.md), [`apps/spacetimedb/src/storage/review_queue/row.rs`](../../apps/spacetimedb/src/storage/review_queue/row.rs), and [`apps/spacetimedb/src/read_model/`](../../apps/spacetimedb/src/read_model/).

## Recommendation

Use a hybrid evidence backbone unless and until SpacetimeDB-only durability/export proofs exist.

The split is not a sunk-cost argument for Postgres. Every retained layer has a distinct enterprise job:

| Layer | Unique job | What it must not become |
| --- | --- | --- |
| SpacetimeDB | Live operational runtime: reducers, authorization-scoped queue mutations, subscriptions, staff/manager/public read models, realtime blocked-action notices, and low-latency workflow coordination. | The only long-term audit ledger, the BI warehouse, the raw document/media store, or the place where raw provider facts become product authority. |
| Postgres | Durable enterprise ledger and query backbone: append-only audit history, workflow/review/approval/outcome/outbox rows, source-import/freshness rows, reconciliation state, BI-safe read models/views, export checkpoints, and point-in-time support/debug queries. | A raw Gingr mirror that analysts reverse-engineer, a replacement for app/domain semantics, or a justification for keeping old tables because they already exist. |
| S3/MinIO-compatible object storage | Immutable evidence object store: source payload snapshots, redacted document/media artifacts, OCR/extraction inputs/outputs, export bundles, hashes/manifests, and large payloads referenced by audit/source rows. | A query database, broad prompt/log sink, or place to expose raw provider/customer payloads without redaction/access policy. |

The core rule is: SpacetimeDB owns live coordination; Postgres owns durable enterprise facts and exports; S3/MinIO owns immutable evidence blobs. If a retained persistence layer cannot name one of those unique jobs for a row/object, do not keep that row/object there.

## Realtime-to-durable flow

```text
source/provider observation, staff input, or fixture evidence
  -> app/domain workflow packet with source refs, review gates, and blocked actions
  -> SpacetimeDB reducer / subscription lane for realtime staff and manager work
  -> durable Postgres append-only events and read-model projections
  -> S3/MinIO object refs for large/raw/redacted evidence where needed
  -> BI/export surfaces that carry lineage, freshness, caveats, and review status
```

SpacetimeDB rows can be the live operational surface. Postgres rows are the enterprise record for audit, historical reporting, replay, export, reconciliation, and support. Object refs bridge both: runtime rows and audit rows may carry safe `object_ref`/`evidence_ref` pointers, but ordinary queue/read-model clients should not receive raw object payloads.

## SpacetimeDB-only proof gates before removing Postgres

If the architecture later recommends SpacetimeDB-only persistence, it must first prove all of these. Until then, removing Postgres would weaken enterprise audit/export credibility.

1. Audit retention proof
   - Append-only audit events with immutable event ids, actor/scope/command/decision/source refs, blocked/allowed status, correlation/causation ids, and schema version.
   - Retention and legal-hold policy for years-scale history.
   - Point-in-time lookup by correlation id, actor id, location id, action id, source ref, and review/outcome ids.
   - Tamper-evidence or independent immutable export path for audit rows.
2. History/replay proof
   - Deterministic replay or snapshot/export strategy across reducer/schema versions.
   - Backfill/migration process that preserves old event semantics and provenance.
   - Operator recovery flow for bad deploys, partial writes, and projection rebuilds.
3. BI/reporting/export proof
   - Supported SQL or equivalent ad-hoc query/export path for analytics without asking BI to subscribe to live operational tables directly.
   - Stable read-model contracts for source-quality backlog, labor outcomes, review aging, audit lineage, import freshness, and outbox posture.
   - Scheduled exports with manifests, row counts, hashes, watermarks, caveats, and failure/dead-letter reporting.
4. Source/evidence proof
   - Safe object storage or equivalent for large/raw/redacted provider payloads, uploaded documents, media, OCR outputs, and export bundles.
   - Evidence refs linked from audit/outcome/source rows without exposing raw payloads through broad subscriptions or logs.
5. Reconciliation proof
   - Source import/freshness/gap tracking: import runs, source record snapshots/refs, sync gaps, rejected counts, adapter version, redaction posture, and last-success watermarks.
   - Durable comparison between source observations, owned workflow decisions, reviewed outcomes, and any future live adapter execution.
6. Operations proof
   - Backup/restore drills, access-control model, retention/redaction tooling, operational dashboards, queue/dead-letter metrics, and incident/debug queries.
   - Clear ownership for export schemas and KPI definitions.

## Audit event shape

Audit events should be append-only, queryable, and safe to export. They should record the decision boundary, not raw secrets or broad provider payloads.

Recommended shape:

| Field | Purpose |
| --- | --- |
| `audit_event_id` | Stable immutable id for the audit row/event. |
| `occurred_at` | Timestamp from the trusted runtime path. |
| `schema_version` | Additive event-shape version. |
| `tenant_id` / `organization_id` | Optional enterprise/tenant scope when present. |
| `location_id` / `location_scope` | Location or region the action was allowed to see/affect. |
| `actor` | Actor id plus display label if safe. For service actors, include service name and credential class, not secrets. |
| `actor_kind` | Staff, manager, regional operator, service account, worker, system, or fixture/demo actor. |
| `actor_role` | Review role used for authorization, such as front-desk lead, general manager, regional operator, operations analyst, or system worker. |
| `command` | The submitted command/reducer/API action: claim queue item, draft recommendation, approve review, record outcome, attempt side effect, export read model, etc. |
| `subject_ref` | Workflow/action/entity being acted on: `action_id`, `workflow_event_id`, `review_packet_id`, `outcome_id`, customer/pet/reservation/doc refs when safe. |
| `decision` | App/domain decision: allowed, blocked, queued-for-review, recorded, suppressed, not-actionable, source-wrong, exported, failed-closed. |
| `decision_reason` | Short policy/result reason safe for audit and support. |
| `blocked_or_allowed` | Explicit status so fail-closed behavior is reportable. |
| `review_gate` | Required/applied gate and approval linkage when relevant. |
| `source_refs` | Stable source/evidence pointers, not raw provider payloads. Include system, record type/id, observed/imported time, adapter/schema version, and hash/ref if available. |
| `object_refs` | Optional document/media/source-snapshot/export object refs with hash, redaction class, object type, and retention class. |
| `correlation_id` | End-to-end request/workflow correlation id for support, BI lineage, and demo explanation. |
| `causation_id` | Parent event/command id that directly caused this event. Use it to rebuild `request -> workflow -> review -> approval -> outcome -> outbox/audit`. |
| `idempotency_key` | Optional client/app key for safely retryable commands. |
| `request_ref` | API request id/session id/user-agent class where appropriate and safe. |
| `runtime` | Runtime/source of the audit event: API, SpacetimeDB reducer, worker, import job, export job. |
| `safe_payload_summary` | Redacted summary only; never raw secrets, customer messages, payments, provider JSON, raw OCR text, or medical/safety payloads in broad audit exports. |

Current SpacetimeDB `HygieneAuditEventRow` and `BlockedActionAttemptRow` already prove a small realtime slice: action id, actor id, blocked actions/attempted side effect, location id, reason, and created time. Enterprise audit needs the additional correlation/causation, role/scope, source/object refs, command, decision, and export-safe metadata above.

## Outcome and labor evidence shape

Outcome/labor evidence should measure reviewed work, not claim that a provider record changed. It must preserve the difference between estimated value, actual staff time, disposition, and source-quality posture.

Recommended shape:

| Field | Purpose |
| --- | --- |
| `outcome_id` | Stable id for the reviewed outcome row. |
| `workflow_name` | `data-quality-hygiene`, `manager-daily-brief`, etc. |
| `action_id` / `issue_refs` | Links outcome to the recommended/reviewed work and data-quality issues. |
| `location_id` / `operating_day` | Reporting dimensions. |
| `actor` / `actor_role` | Person/system that recorded or reviewed the outcome. |
| `review_status` | Pending review, manager approved, completed, deferred, suppressed, rejected, source-wrong, not-actionable, blocked. |
| `review_packet_id` / `approval_record_id` | Review-gate evidence proving the outcome was reviewed when required. |
| `estimated_minutes` | Pre-action estimate from app/workflow packet. This is forecast evidence, not proof of saved labor. |
| `actual_minutes_spent` | Reviewed time spent by staff/manager. |
| `actual_minutes_saved` | Reviewed saved/avoided minutes, if the workflow records that measure separately from time spent. |
| `disposition` | Completed, deferred, duplicate, source-wrong, stale, not-actionable, suppressed, needs-provider-follow-up, etc. |
| `source_quality_finding` | Whether the source fact was current, stale, conflicting, missing, duplicate, mapping-uncertain, provider-wrong, or fixture-only. |
| `source_refs` | Source records/snapshots that supported the recommendation and outcome. |
| `object_refs` | Optional documents/media/source snapshots used as evidence. |
| `caveats` | Redaction/freshness/review caveats, including `live_side_effects_allowed=false` for the local proof. |
| `correlation_id` / `causation_id` | Lineage back to request/workflow/review/audit events. |

Current SpacetimeDB `HygieneOutcomeRow` and `HygieneOutcomeCardRow` prove part of this shape: action id, recorded-by label/payload, outcome label, before/actual minutes, source refs, issue refs, reviewed resolution status, and `live_delivery_allowed=false`. The enterprise backbone should persist the complete outcome evidence in Postgres and project safe realtime cards from it.

## Export and BI shape

BI can report product-owned operational facts and caveated source-quality evidence. It must not treat raw provider facts as product authority.

Safe BI/export surfaces:

| Surface | Safe questions it answers | Required caveats/lineage |
| --- | --- | --- |
| `source_quality_backlog` | What source defects are open/reviewed by location, severity, issue kind, workflow-blocking posture, owner role, and age? | Source refs, import freshness, mapping status, sensitivity/redaction status, review status, and whether the row came from fixture/demo data. |
| `labor_outcomes` / `data_quality_hygiene_labor_outcomes` | How many reviewed actions were completed/deferred/suppressed/source-wrong/not-actionable, and what estimated vs actual minutes were recorded? | Review/approval linkage, source refs, workflow name, location/day, caveats that outcomes measure reviewed work rather than provider mutation. |
| `review_queue_aging` | Which gates/roles/locations are waiting on review and how long? | Review packet ids, gate/status, actor role/scope, blocked action families, no raw payloads. |
| `audit_lineage` | For a correlation id, what request/workflow/review/approval/outcome/outbox/audit events happened? | Ordered causation chain, safe event summaries, source/object refs, actor role/location scope. |
| `import_freshness` | Which source imports are current, stale, rejected, or gap-prone? | Import run ids, adapter version, last success/failure, redaction posture, sync gaps, row counts, hashes/manifests. |
| `outbox_posture` | Which side-effect candidates are approved, blocked, stubbed, retried, or dead-lettered? | Approval refs, blocked/allowed status, live adapter mode, safe error class, no live-send claim unless separately approved. |
| `export_manifest` | What exact data was exported and under which schema/watermark? | Export id, generated_at, projection version, filters, watermarks, row counts, object refs, hashes, caveats, access class. |

Unsafe BI/export claims:

- Do not report raw Gingr/provider rows as NVA-owned truth without an owned mapping/review/read-model status.
- Do not count estimated minutes as actual savings without reviewed outcome evidence.
- Do not hide stale/conflicting/missing source posture when reporting operational progress.
- Do not export raw provider payloads, raw documents, raw OCR text, messages, payment payloads, or medical/safety content through ordinary BI surfaces.
- Do not imply customer/member messages, provider writes, schedule changes, payments, or medical/safety decisions happened when the row only proves a draft, blocked attempt, review packet, or disabled outbox candidate.

## Object and evidence refs

Use object refs when evidence is large, sensitive, binary, or raw enough that it should not live directly in queue/audit/read-model rows.

Recommended `evidence_ref` / `object_ref` shape:

| Field | Purpose |
| --- | --- |
| `evidence_ref_id` | Stable id used by audit/outcome/source rows. |
| `object_uri` | S3/MinIO key or URI, not a broad public URL. |
| `object_type` | Source payload snapshot, uploaded document, media, OCR/extraction output, redacted derivative, export bundle, manifest, or screenshot. |
| `subject_ref` | Customer/pet/reservation/document/workflow/source record the object supports, when safe. |
| `source_system` / `source_record_ref` | Provider/source lineage for payload snapshots. |
| `content_sha256` | Integrity hash for tamper/replay/export verification. |
| `size_bytes` / `content_type` | Storage and validation metadata. |
| `redaction_status` | Raw, redacted, tokenized, derived-summary-only, fixture-only, or restricted. |
| `retention_class` | Operational, audit, legal-hold, demo-fixture, export-temporary, etc. |
| `created_at` / `observed_at` | Object creation and source observation times. |
| `access_class` | Who/what can read it; ordinary BI/read-model clients should usually get metadata only. |
| `manifest_ref` | Optional export/import manifest linkage. |

Object refs are relevant for provider payload snapshots, vaccine/document uploads, media, OCR/extraction artifacts, redacted derivatives, export bundles, and import/export manifests. They are optional for small safe source refs, but mandatory when the raw evidence would be too large or sensitive for Postgres/SpacetimeDB rows.

## Concrete implementation cutline

For the current Data-Quality Hygiene slice, the concrete enterprise cutline is:

1. SpacetimeDB keeps live queue and subscription state:
   - private rows: actor/role/location scope, review queue item, source-quality issue summary, workflow transition, hygiene outcome, hygiene audit event, blocked action attempt;
   - public rows: staff queue item, manager queue item, hygiene outcome card, blocked action notice;
   - reducer responsibility: enforce app-owned authorization, update live rows, publish safe realtime views, and fail closed on blocked side effects.
2. Postgres stores durable enterprise history:
   - append-only audit events with full command/decision/source/object/correlation/causation shape;
   - workflow/review/approval/outcome/outbox rows for replay, support, and point-in-time reporting;
   - source import runs, source record snapshots/refs, source quality issues, sync gaps, and export manifests;
   - BI views/materialized projections for source backlog, labor outcomes, review aging, audit lineage, import freshness, and outbox posture.
3. S3/MinIO stores evidence objects:
   - raw or redacted provider snapshots, uploaded documents/media, OCR/extraction artifacts, export bundles/manifests, and large payloads;
   - every object has hash, retention/redaction/access metadata and is referenced by Postgres audit/source/outcome/export rows;
   - SpacetimeDB/public clients see only safe refs/summaries needed for review and traceability.

## Job-contact explanation

The simple explanation is:

> The realtime demo is not just a queue toy. SpacetimeDB shows the live operating loop: staff and managers see scoped cleanup work immediately, blocked actions fail closed, and reviewed outcomes update realtime cards. The enterprise backbone is the durable proof layer behind that loop: Postgres keeps the audit/history/reporting/export ledger, and S3/MinIO keeps immutable document/source evidence. BI reports from reviewed, source-caveated read models instead of treating raw provider tables as authority.

If asked why two databases are justified:

> They have different jobs. SpacetimeDB is for live reducers and subscriptions. Postgres is for durable audit, history, SQL reporting, reconciliation, and export contracts. Object storage is for large immutable evidence. I would remove Postgres only after SpacetimeDB proves years-scale audit retention, point-in-time historical queries, BI-safe exports, source/import reconciliation, object evidence handling, backup/restore, and operational tooling.

## Verification path

For this planning artifact:

```sh
./scripts/check_docs.sh
python scripts/check_markdown_links.py --repo-root .
```

For downstream implementation, require tests/proofs that:

- a reviewed Data-Quality Hygiene outcome creates live SpacetimeDB rows and durable Postgres audit/outcome/source/read-model rows with the same correlation id;
- a blocked side-effect attempt creates a realtime public notice and a durable audit event with blocked status, actor role/scope, command, decision reason, and causation id;
- source/object refs survive export and can be verified by hashes/manifests without exposing raw payloads to ordinary BI/read-model clients;
- BI projections include freshness, review status, caveats, and source-quality findings; and
- fixture/demo rows remain clearly labeled so no live production data access is invented.
