# Owned operations API replacement thesis

Status: canonical strategy and architecture artifact for the owned Pet Resorts operations API direction. This page frames the business thesis and migration problem; it does not design every endpoint, claim live NVA/Gingr access, claim production readiness, authorize provider/PMS writes, send customer/member messages, move payments/refunds/discounts, change schedules/capacity, approve medical/safety decisions, or deploy anything.

## Executive thesis

NVA should not build a literal Gingr clone. The product opportunity is an owned Pet Resorts operations API that captures the operating contracts NVA actually needs and lets Gingr become one source adapter during migration.

The current BI extraction workaround is a product signal: teams already need operational answers and read models that Gingr does not provide cleanly enough. Pulling Gingr into a separate BI database helps reporting, but it leaves the underlying workflow authority, source provenance, review gates, labor outcomes, and audit posture outside a product-owned contract. The replacement path is to move those contracts into an owned operations API and use provider systems only as evidence sources until a workflow no longer needs them.

Five-point replacement thesis:

1. Gingr/provider data is source evidence, not the target product model.
2. The owned API should speak NVA pet-resort operations: customers, pets, reservations, service lines, tasks, review packets, outcomes, audit, and read models.
3. BI extraction pain should become first-class read-model and metric requirements rather than a mandate to mirror provider tables.
4. Gingr should shrink into a source adapter that imports verified facts, exposes provenance, and highlights gaps; it should not own workflow authority.
5. V0 should prove one or two review-gated workflows end-to-end before any live replacement claim: source evidence -> owned contract -> review gate -> outcome/labor/audit/read model.

## Why this is not a Gingr clone

A clone would preserve the incumbent problem: NVA would still inherit provider-shaped IDs, endpoint quirks, optional fields, undocumented surfaces, and reporting gaps as if those were the business. The repository already points in a different direction:

- The README's front-door principle says provider systems provide source evidence while app/domain contracts, review gates, outcomes, storage projections, and runtime proof carry the labor-saving operating model ([`../../README.md`](../../README.md#canonical-docs-path)).
- Runtime boundaries say provider DTOs and raw responses are quarantined evidence, while product-owned contracts live in domain/app/API/storage ([`runtime-contract-boundaries.md`](runtime-contract-boundaries.md)).
- Gingr integration docs explicitly keep provider IDs, raw responses, DTOs, webhooks, and endpoint shapes separate from normalized domain meaning ([`../../integrations/gingr/README.md`](../../integrations/gingr/README.md)).
- Storage docs define persisted projections and read models as reporting/review views, not live decision authority or raw provider mirrors ([`../../storage/README.md`](../../storage/README.md)).

The API replacement should therefore target NVA-owned outcomes:

- faster staff review queues;
- fewer repeated source-data checks;
- explicit source lineage for every operational claim;
- safer automation boundaries;
- measurable labor outcomes;
- BI/read models that answer operational questions without scraping around provider limitations.

## Owned API capability families

The owned operations API should be organized around capability families that preserve business meaning. These families are product contracts, even when a v0 route is still local/demo-only.

| Capability family | What the owned API should own | Current proof paths | Not owned by provider DTOs |
| --- | --- | --- | --- |
| Source, provenance, and data quality | Source systems, source refs, adapter/schema versions, observed-at timestamps, source freshness, unknown/unsupported provider surfaces, data-quality issues, redaction/sensitivity boundaries, and reconciliation status. | [`runtime-contract-boundaries.md`](runtime-contract-boundaries.md), [`../workflows/operator/data-quality-hygiene.md`](../workflows/operator/data-quality-hygiene.md), [`../../domain/src/source.rs`](../../domain/src/source.rs), [`../../domain/src/data_quality.rs`](../../domain/src/data_quality.rs), [`../../integrations/gingr/README.md`](../../integrations/gingr/README.md). | A provider payload can show what Gingr returned; it cannot decide whether a fact is clean, current, sensitive, or usable by an NVA workflow. |
| Operational entities | Product-owned customers, pets, reservations, service offerings, documents, vaccine facts, care notes, incidents, messages, payment/deposit projections, tasks, locations, staff actors, and external refs. | [`pet-resort-data-model.md`](pet-resort-data-model.md), [`../../domain/README.md`](../../domain/README.md), [`../../migrations/0001_mvp_foundation.sql`](../../migrations/0001_mvp_foundation.sql), [`../../storage/src/operations.rs`](../../storage/src/operations.rs). | Gingr IDs and records remain external references. They should not replace semantic NVA IDs, lifecycle states, review requirements, or policy concepts. |
| Workflow packets and review gates | Reviewable packets, draft artifacts, deterministic checks, allowed/blocked actions, approval requirements, review packet state, and human/system-of-record gates. | [`pet-resort-workflow-events.md`](pet-resort-workflow-events.md), [`../../app/README.md`](../../app/README.md), [`../../app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs), [`../../app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs), [`../../apps/api/src/http.rs`](../../apps/api/src/http.rs). | A provider record can trigger or inform a workflow; it cannot approve customer sends, provider writes, refunds, booking confirmations, eligibility, or safety decisions. |
| Labor outcomes and business value | Reviewed dispositions, before/after minutes, actual minutes spent/saved, source-fact correctness, owner persona, reporting groups, outcome status, and workflow-to-outcome linkage. | [`../workflows/operator/data-quality-hygiene.md#8-outcome-and-labor-value`](../workflows/operator/data-quality-hygiene.md#8-outcome-and-labor-value), [`../presentation/job-presentation-walkthrough.md#local-demo-slice-data-quality-hygiene-in-five-minutes`](../presentation/job-presentation-walkthrough.md#local-demo-slice-data-quality-hygiene-in-five-minutes), [`../../storage/src/operations.rs`](../../storage/src/operations.rs). | Provider activity logs or BI extracts do not by themselves prove reviewed labor reduction or distinguish completed, deferred, suppressed, wrong-source, and not-actionable outcomes. |
| Logging, metrics, audit, and read models | Request/correlation/workflow/job IDs, safe error classes, append-only audit events, workflow/outbox/dead-letter posture, operational metrics, BI-friendly read models, and dashboard summaries. | [`runtime-contract-boundaries.md`](runtime-contract-boundaries.md), [`../audits/dto-api-db-observability-readiness-gap-map.md`](../audits/dto-api-db-observability-readiness-gap-map.md), [`../roadmap/pet-resort-mvp-stack.md#12-observability-audit-and-operations`](../roadmap/pet-resort-mvp-stack.md#12-observability-audit-and-operations), [`../../apps/api/README.md`](../../apps/api/README.md). | A provider table dump cannot explain why an action was recommended, which review gate applied, what was blocked, whether an outbox item was approved, or how a BI number ties back to source and outcome evidence. |
| Adapter and migration contracts | Explicit source adapters, import windows, mapping errors, unsupported-surface markers, source snapshots, fixture-backed DTO expansion, dual-run comparison, cutover gates, and fallback/read-only posture. | [`../../integrations/gingr/README.md`](../../integrations/gingr/README.md), [`../integrations/gingr/README.md`](../integrations/gingr/README.md), [`../entity-atlas/contract-crosswalk/source-provider-flows.md`](../entity-atlas/contract-crosswalk/source-provider-flows.md). | Provider adapters should not shape the owned resources. They should feed evidence into owned contracts and make gaps visible. |

## Provider DTOs vs product-owned API resources

Keep these layers distinct during design and implementation:

```text
provider/source evidence
  - Gingr endpoint requests, response envelopes, DTOs, webhooks, raw/fixture records
  - provider-scoped ids, optional fields, unknown fields, unsupported surfaces
  - authority: what the source said and when it was observed
        |
        v
source refs + provenance
  - source system, external record ref, adapter/schema version, observed timestamp, sensitivity/redaction posture
  - authority: lineage and trust boundary
        |
        v
product-owned resources and workflows
  - NVA customer, pet, reservation, document, vaccine, task, review packet, outcome, audit event, read model
  - authority: validated operational meaning, review requirements, workflow state, labor evidence
        |
        v
API DTOs and read models
  - staff/reviewer request/response contracts, queue views, outcome summaries, dashboard/BI projections
  - authority: what NVA exposes as a supported operational contract
```

Provider DTOs should be kept close to `integrations/gingr` and only promoted through explicit mapping. Product-owned resources should be named from NVA operations and should remain usable if Gingr is replaced, augmented, or reduced to a historical source. Storage/read-model shapes should be projections of owned meaning plus source refs; they should not become a second raw Gingr database with friendlier names.

Design rule: if a field exists only because Gingr has it, keep it in the adapter/source-evidence layer until an NVA workflow, review gate, read model, or metric can explain the business meaning.

## BI workaround as a pain map

The BI extraction workaround should be read as evidence of missing owned contracts. Instead of treating BI's separate database as an endpoint, convert each recurring reporting need into an API/read-model requirement:

| BI/workaround pain | Owned API/read-model response | Product implication |
| --- | --- | --- |
| Raw provider tables need cleanup before analysts trust them. | Source/provenance/data-quality read models expose issue category, source refs, freshness, sensitivity, resolution status, and reviewer disposition. | Data-quality hygiene becomes a core operating loop, not a backstage spreadsheet cleanup. |
| Reports need customer/pet/reservation/service facts normalized across provider quirks. | Operational entity resources expose semantic IDs, external refs, validated lifecycle states, service-line contracts, and source evidence. | Provider adapters can change without forcing BI and agents to relearn provider semantics. |
| Labor savings are hard to prove from provider activity alone. | Outcome records and read models capture estimated/actual minutes, staff persona, action kind, reporting group, source-fact correctness, and reviewed disposition. | ROI moves from anecdote to reviewed workflow evidence. |
| Exception queues live across many provider screens or exports. | Workflow packet/read models expose review queues, blockers, required gates, and next safe staff actions. | Staff and analysts inspect one product-owned queue contract rather than recomputing urgency in BI. |
| Audit and traceability require stitching logs, exports, and staff memory. | Append-only audit, request/correlation IDs, workflow events/results, review packets, approvals, outbox, and safe logs provide an auditable chain. | BI can consume trusted projections instead of reverse-engineering action history. |
| Provider fields do not map neatly to desired portfolio questions. | Purpose-built BI projections answer location/day/service/workflow/labor/source-quality questions with explicit caveats. | The API can produce NVA operating views even while Gingr remains a partial source. |

This does not make BI less important. It gives BI a cleaner upstream contract: product-owned operational read models with lineage, review status, and labor/audit context.

## Gingr as a source adapter during migration

In the replacement path, Gingr should have a narrower and clearer role:

1. Read provider/source evidence where documentation, fixtures, or approved access make the shape trustworthy.
2. Preserve provider IDs, endpoint names, unknown fields, and unsupported surfaces as source facts.
3. Verify signatures or transport authenticity where applicable.
4. Map provider records into domain candidates only through explicit conversion functions and typed errors.
5. Emit source refs, provenance, freshness, and data-quality issues into the owned API.
6. Feed workflow events/read models without granting live action authority.
7. Support dual-run reconciliation until a workflow can operate from owned resources and approved system-of-record paths.

Gingr should not:

- define NVA's canonical resource names or lifecycle states;
- become the API's public DTO vocabulary;
- justify unreviewed customer sends, schedule changes, provider writes, payment moves, or medical/safety decisions;
- hide missing provider surfaces behind guessed fields;
- be mirrored into storage unless a normalized projection or source snapshot has a product purpose.

## First v0 workflow candidates

V0 should pick workflows that already demonstrate the owned-contract shape and convert them into durable API/read-model proof. Good candidates:

1. Data-quality hygiene. Strongest bridge from BI pain to product-owned API. It already ranks source-quality issues, preserves source refs/issue refs, validates draft cleanup work, records reviewed outcomes, and summarizes labor evidence. Next slice: persist API -> Postgres workflow/review/audit/outcome rows and expose a BI-ready cleanup/labor read model.
2. Manager daily brief. Strong labor-value story: source-grounded action packets, reviewed manager outcomes, reporting groups, and labor minutes. Next slice: tie brief actions to durable workflow events and outcome/read-model projections.
3. Inquiry intake. Clear API ownership surface for lead normalization, draft-only customer response, staff review queue, and audit events. Next slice: move from in-memory API state to `workflow_events`, `operational_tasks`, review packets, message draft records, and audit.
4. Vaccine/document review. Strong safety and evidence boundary: upload metadata, extraction suggestions, review packets, approval/rejection, eligibility projections, and audit without autonomous medical acceptance. Next slice: durable object metadata/evidence storage and review packet persistence.
5. Checkout completion / review-request eligibility. Useful later BI and revenue-read-model candidate, but keep payment, final checkout, review-request sends, and provider status writes gated until approvals/outbox are implemented.

Recommended first proof: Data-quality hygiene, because it directly turns the BI workaround into a product thesis: bad source facts become source-backed cleanup work, reviewed outcomes, labor metrics, and read models rather than opaque ETL cleanup.

## Current proof paths and honest gaps

Current proof already supports the thesis:

- Strategy/front door: [`../../README.md`](../../README.md), [`../presentation/job-presentation-walkthrough.md`](../presentation/job-presentation-walkthrough.md), [`runtime-contract-boundaries.md`](runtime-contract-boundaries.md).
- Board plan: [`../plans/2026-06-25-owned-operations-api-replacement-kanban.md`](../plans/2026-06-25-owned-operations-api-replacement-kanban.md).
- API shell and safe route contracts: [`../../apps/api/README.md`](../../apps/api/README.md), [`../../apps/api/src/http.rs`](../../apps/api/src/http.rs).
- Data-quality proof: [`../workflows/operator/data-quality-hygiene.md`](../workflows/operator/data-quality-hygiene.md), [`../../app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs).
- Storage and migration proof: [`../../storage/README.md`](../../storage/README.md), [`../../storage/src/operations.rs`](../../storage/src/operations.rs), [`../../migrations/0001_mvp_foundation.sql`](../../migrations/0001_mvp_foundation.sql).
- Gap inventory: [`../audits/dto-api-db-observability-readiness-gap-map.md`](../audits/dto-api-db-observability-readiness-gap-map.md), [`../roadmap/pet-resort-mvp-stack.md`](../roadmap/pet-resort-mvp-stack.md).
- Gingr/source adapter proof: [`../../integrations/gingr/README.md`](../../integrations/gingr/README.md), [`../integrations/gingr/README.md`](../integrations/gingr/README.md).

Gaps to keep explicit:

1. The API currently proves owned route shape locally, but much state is still in-memory and not connected to the Postgres migration spine.
2. API DTOs are not yet published as OpenAPI/client schemas.
3. Request/job/workflow correlation is present in pieces, not yet an end-to-end durable observability model.
4. Worker leasing, dead-letter/replay, outbox execution, and dashboard metrics remain shells or plans.
5. Object storage and durable evidence handling are modeled but not fully wired for document workflows.
6. There is no live NVA/Gingr credential use, production data, provider writeback, live customer messaging, payment movement, schedule/capacity mutation, medical/safety decision automation, or production deployment proof.
7. BI pain is strategically clear, but detailed BI stakeholder query inventory and production KPI definitions still need owner/input validation.

## Implications for downstream API/schema cards

Downstream cards should avoid endpoint-first cloning and instead derive schemas from owned resources and read-model questions.

1. Define product-owned resource names first: `source_ref`, `data_quality_issue`, `workflow_event`, `review_packet`, `approval_record`, `outcome`, `audit_event`, `outbox_candidate`, and workflow-specific resources such as `data_quality_hygiene_action` or `manager_daily_brief_action`.
2. Keep adapter DTOs out of public API schemas except as source-evidence references or explicitly marked raw/source snapshots.
3. Publish v0 OpenAPI around review-safe workflows and read models, with live side effects marked disabled/stubbed unless a later approved gate changes that.
4. Add read-model endpoints that answer BI/operator questions directly: source-quality backlog, reviewed cleanup outcomes, labor minutes by location/day/persona/action, review queue aging, outbox posture, and audit/correlation lookup.
5. Wire one vertical slice through durable persistence before expanding breadth: API request -> workflow event -> review packet -> approval/outcome/audit/outbox candidate -> metrics/read model.
6. Make migration/cutover explicit: each route should state whether it reads owned storage, provider adapter evidence, fixture/local state, or a future dual-run comparison.

## Replacement readiness ladder

Use this ladder when describing progress:

1. Architecture/demo-ready: owned contracts and local proofs exist; no live side effects or production claims.
2. Durable v0 proof: one workflow persists through owned Postgres workflow/review/audit/outcome/read models with safe local/stubbed side effects.
3. BI-read-model pilot: selected BI/operator questions consume owned read models with lineage and caveats.
4. Adapter dual-run: Gingr imports feed the owned API alongside existing BI workarounds, with reconciliation reports and no unapproved writes.
5. Workflow replacement: staff uses the owned workflow for a scoped operational job while Gingr remains read/source or approved system-of-record as needed.
6. Provider shrinkage: after legal/ops/security approval, approved write paths or alternative systems reduce the operational need for Gingr workflow screens.

Anything beyond the first two rungs requires owner decisions, real access, production data handling, security/privacy review, and explicit approval for any live operational side effect.
