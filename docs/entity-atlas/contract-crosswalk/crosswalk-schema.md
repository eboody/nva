# Contract crosswalk schema and evidence rules

Purpose: this is the reusable row contract for every entity/workflow/source/storage crosswalk table that follows the entity atlas. A crosswalk row must let a non-coder reviewer trace one business concept from source entry, through normalization and workflow use, into persistence/exposure, while seeing who owns authority, what automation may do, what stays reviewed, how value is measured, and where source/Rustdoc/test proof lives.

Use this schema with the [entity atlas inventory](../../design/entity-atlas-inventory.md), [entity atlas page template](../../design/entity-atlas-page-template.md), [relationship map](../../design/entity-atlas-relationships.md), [workflow source/Rustdoc backing map](../../design/workflow-source-rustdoc-backing-map.md), and [documentation style guide](../../quality/nva-documentation-style-guide.md). Markdown rows explain the contract; source, tests, and Rustdoc/module paths remain the behavioral authority.

Crosswalk siblings: [surface inventory](surface-inventory.md) finds the evidence surfaces, [source/provider flows](source-provider-flows.md) shows entry and normalization, [workflow packets](workflow-packets.md) shows workflow use and human-review boundaries, [storage/persistence](storage-persistence.md) shows persisted or intentionally non-persisted projections, and [runtime exposure](runtime-exposure.md) shows API/worker/CLI/web/script exposure. The [README](../../../README.md#entity-navigation-map), [entity index](../../design/entity-index.md#contract-crosswalk-proof-paths), and [operator workflow index](../../workflows/operator/README.md) link back here so readers can navigate from business entity -> source proof and from source/test proof -> business meaning.

## Required crosswalk row fields


Glossary bridge: when a crosswalk row uses repo terms that can change a non-coder's interpretation of authority or safety, link to the nearest glossary entry on first use: [domain](../../glossary-architecture-terms.md#domain), [app](../../glossary-architecture-terms.md#app), [storage](../../glossary-architecture-terms.md#storage), [DTO](../../glossary-architecture-terms.md#dto), [adapter](../../glossary-architecture-terms.md#adapter), [source-of-record](../../glossary-source-data-terms.md#source-of-record), [source refs/provenance](../../glossary-source-data-terms.md#domainsourceprovenance-and-domainsourcerecordref-as-data-evidence), [workflow packet](../../glossary-workflow-state-terms.md#workflow-packet), [review gate](../../glossary-workflow-state-terms.md#review-gate), [blocked action](../../glossary-workflow-state-terms.md#blocked-action), and [outcome capture](../../glossary-workflow-state-terms.md#outcome-capture). Keep glossary links sparse; the row's source/test/Rustdoc evidence remains authoritative.

Every later contract-crosswalk table must use these fields in this order. Short tables may keep the same fields as bullets under each entity, but they must not omit a field; write `none known`, `not applicable`, or `gap/TODO` rather than leaving ambiguity.

| Field | Required writer content | Evidence rule |
| --- | --- | --- |
| Entity | The canonical business concept name, using plain words first and code/module names second if helpful. | Must match a candidate or family in the [inventory](../../design/entity-atlas-inventory.md), or be marked `new/gap` with why it belongs. |
| Plain-English meaning | One or two sentences describing the pet-resort meaning for an operator, manager, regional reviewer, or docs writer. | Must not open with code jargon. Code paths can appear after the meaning is clear. |
| Source-entry surfaces | Where the fact first enters review: provider endpoint/DTO/webhook, staff note, uploaded document, import, BI/read model, or local fixture. | Link source files, provider-boundary docs, or workflow maps. If source authority is unknown, say `source-entry gap`. |
| Normalization surfaces | Where raw/provider/source facts become semantic candidates, domain entities, data-quality issues, or explicit mapping errors. | Link domain/mapping/source modules. Do not imply raw provider fields are domain truth. |
| Workflow-use surfaces | Which workflow packets, review queues, agent packets, drafts, or deterministic validators read or produce the entity. | Link app workflow modules, workflow pages, or backing-map sections. Include safe result wording such as `review packet`, `draft`, `ranked queue`, or `validated outcome`. |
| Persistence surfaces | Storage projections, outcome records, audit records, approval records, source refs, raw payload refs, or statement that the entity is not persisted. | Link storage/domain/audit source or tests where present. If planned only, mark `planned/no persisted contract yet`. |
| Exposure surfaces | API routes, worker runtime, CLI/public docs/Rustdoc/operator page surfaces that expose the contract to humans or systems. | Link runtime/source/docs surfaces. Do not claim production UI or live API exposure unless backed. |
| Related entities | Adjacent entities and relationship direction: owns, is evidence for, feeds, blocks, approves, persists, exposes, measures, or reviews. | Include at least one inbound and one outbound relationship when applicable. Use `none known` only for truly isolated support values. |
| Authority owner | Source system or human role that owns a fact/decision, scoped by fact type. | Ask `authority for what?` Separate provider status, policy exception, customer send, safety decision, payment movement, and labor claim owners. |
| Automation allowed/draft/recommend actions | Verbs the app/agent may perform: read, map, validate, summarize, rank, draft, prepare internal task, record reviewed outcome. | Must be backed by workflow `SafeAgentAction`, tool-port, app service, or domain policy evidence. |
| Blocked or human-reviewed actions | Explicit no-go and review-gated actions: customer sends, provider/PMS writes, schedule/capacity changes, money movement, safety/medical/behavior approvals, source hiding, sensitive exposure. | Every row touching customers, pets, safety, money, schedules, provider records, or source ambiguity must name blocked actions. |
| Value/outcome measure | How the entity contributes to labor savings, safety, revenue, quality, or audit confidence; include measurable fields where known. | Labor/revenue claims need outcome records, actual/estimated minutes, disposition, conversion, or `planned metric` wording. |
| Rustdoc/source/test evidence | Source links, Rustdoc/module paths, and tests/doctests that prove the row. | Include at least one source path. Include test paths when behavior is tested. Use module paths when rendered Rustdoc is not present/published. |
| Caveats | Gaps, local/demo-only status, provider-surface uncertainty, data-quality ambiguity, privacy constraints, or future-work warnings. | Must keep docs from overstating live readiness, autonomous authority, or production outcome proof. |

## Copyable Markdown row template

Use this shape for long-form rows. It keeps the table readable when evidence lists are too long for one Markdown row.

```markdown
### Entity: <Plain name> (`module::Path` when useful)

| Field | Crosswalk value |
| --- | --- |
| Entity |  |
| Plain-English meaning |  |
| Source-entry surfaces |  |
| Normalization surfaces |  |
| Workflow-use surfaces |  |
| Persistence surfaces |  |
| Exposure surfaces |  |
| Related entities |  |
| Authority owner |  |
| Automation allowed/draft/recommend actions |  |
| Blocked or human-reviewed actions |  |
| Value/outcome measure |  |
| Rustdoc/source/test evidence |  |
| Caveats |  |
```

For compact multi-row matrices, keep the same column order and move longer evidence to footnotes or a per-entity evidence appendix. Do not collapse `allowed actions` and `blocked actions` into one vague `automation` column.

## Evidence standards

### Source and Rustdoc evidence

- Source paths should be repo-local links such as [`domain/src/source.rs`](../../../domain/src/source.rs), [`app/src/manager_daily_brief.rs`](../../../app/src/manager_daily_brief.rs), or [`storage/src/operations.rs`](../../../storage/src/operations.rs).
- Rustdoc evidence may be a module/type path, for example `domain::source::Provenance`, `app::manager_daily_brief::Packet`, or `storage::operations::ManagerDailyBriefOutcomeRecord`.
- Use rendered Rustdoc links only when the generated/published page is present and link-checkable in the repo or docs site. Otherwise write `Rustdoc/module path` and link source.
- Test evidence should name the exact behavior tested, not merely say `tests exist`.

### Authority and automation evidence

- Separate authority by decision. A provider may own observed reservation status; a manager may own policy exceptions; an approved sender may own customer messages; accounting may own payment movement; a workflow outcome record may own measured labor feedback.
- Automation verbs must be narrow. Prefer `summarize source evidence`, `draft review text`, `rank cleanup candidates`, `validate blocked actions`, and `record reviewed outcome` over vague `handle`, `resolve`, or `process`.
- If no source/test proves an allowed action, classify it as `recommended future action` or `gap/TODO`, not current authority.

### Outcome evidence

- Outcome rows should cite fields that can be reviewed later: actual minutes saved, estimated minutes, disposition, staff persona, source refs, reporting group, conversion/suppression status, approval record, audit event, or data-quality resolution status.
- `Could save labor` is acceptable only as a hypothesis. `Saved labor` requires reviewed outcome evidence.

### Caveat language

Use caveats to keep the docs honest:

- `local/demo contract only; no production connector proven`
- `provider evidence only; not a domain approval`
- `storage projection exists; live writeback remains blocked`
- `outcome path exists; production savings not yet measured`
- `provider surface gap; do not invent DTO fields`

## Representative examples

### Entity: Provenance (`domain::source::Provenance`)

| Field | Crosswalk value |
| --- | --- |
| Entity | Provenance / source receipt |
| Plain-English meaning | The receipt for where a source fact came from: system, endpoint, record id, extraction batch, pull time, schema, payload hash, raw payload ref, and request scope. It lets a reviewer trace why a recommendation or data-quality issue exists. |
| Source-entry surfaces | Gingr endpoints/webhooks, BI/read models, staff/import files, labor/timeclock/POS sources, and other source systems named by [`domain/src/source.rs`](../../../domain/src/source.rs). Provider-boundary examples are summarized in the [Gingr provider boundary atlas](../../integrations/gingr/provider-boundary-atlas.md). |
| Normalization surfaces | `domain::source::{Provenance, RecordRef, System, Endpoint, RawPayloadRef, PayloadHash}` preserve the source receipt before domain/workflow use; Gingr mapping candidates and data-quality issues must keep source refs instead of silently promoting raw fields. |
| Workflow-use surfaces | Manager Daily Brief, Data Quality Hygiene, Booking Triage, Checkout Completion, CRM Retention, and Daily Updates use source refs/provenance to keep packets source-grounded; see the [workflow backing map](../../design/workflow-source-rustdoc-backing-map.md#shared-source-of-record-and-safety-boundaries). |
| Persistence surfaces | Storage-side source references such as `storage::operations::StoredSourceRecordRef` persist reviewed outcome/source evidence in [`storage/src/operations.rs`](../../../storage/src/operations.rs). Raw payload refs and hashes are evidence handles, not permission to expose sensitive payloads. |
| Exposure surfaces | Entity/source atlas pages, workflow pages, Rustdoc/module paths, and storage/API contract tests expose provenance as a traceability requirement. No live provider browser is implied by this row. |
| Related entities | Feeds data-quality issues, workflow packets, outcome records, audit events, and storage records; depends on source system, provider record, endpoint, payload hash, and raw payload reference. |
| Authority owner | The source system owns the observed fact; integration owner owns source extraction/mapping correctness; workflow reviewer owns decisions that use the fact. |
| Automation allowed/draft/recommend actions | Read source refs, summarize source evidence, validate required refs, group issues by source, and record reviewed outcome refs. |
| Blocked or human-reviewed actions | No hiding missing/stale/conflicting source facts, no exposing sensitive raw payloads without review, no treating raw provider status as approval, and no provider/PMS mutation from provenance alone. |
| Value/outcome measure | Reduces reconciliation time and audit ambiguity by letting each recommendation point to source refs; measurable when outcome records preserve source refs and actual/estimated minutes. |
| Rustdoc/source/test evidence | Source: [`domain/src/source.rs`](../../../domain/src/source.rs), [`storage/src/operations.rs`](../../../storage/src/operations.rs). Rustdoc/module paths: `domain::source::{Provenance, RecordRef, System}`, `storage::operations::StoredSourceRecordRef`. Tests: workflow/storage tests that require source refs, including [`app/tests/manager_daily_brief_workflow_contracts.rs`](../../../app/tests/manager_daily_brief_workflow_contracts.rs) and [`app/tests/data_quality_hygiene_workflow_contracts.rs`](../../../app/tests/data_quality_hygiene_workflow_contracts.rs). |
| Caveats | Provenance proves source lineage, not truth, approval, freshness, or authorization. Sensitive/high-PII raw payloads stay behind redaction/access review. |

### Entity: Manager Daily Brief packet (`app::manager_daily_brief::Packet`)

| Field | Crosswalk value |
| --- | --- |
| Entity | Manager Daily Brief packet |
| Plain-English meaning | A manager-facing review bundle of source-backed actions, labor-risk or service-demand facts, safe recommendations, blocked actions, and outcome feedback for a resort operating day. |
| Source-entry surfaces | Domain daily-brief/analytics facts, labor/timeclock/POS/read-model evidence, checkout and retention packets, source refs, and data-quality issues. The workflow backing map lists the source surfaces in [Manager Daily Brief](../../design/workflow-source-rustdoc-backing-map.md#manager-daily-brief). |
| Normalization surfaces | `domain::daily_brief` and `domain::analytics` shape operating-day facts; `app::manager_daily_brief::{Request, SourceFact, LaborImpactEstimate}` compose normalized source evidence into packet actions. |
| Workflow-use surfaces | The packet is the workflow-use surface: it ranks manager actions, embeds scoped checkout/retention packets, estimates labor impact, carries safe/blocked action vocabulary, and accepts reviewed outcome records. |
| Persistence surfaces | `app::manager_daily_brief::OutcomeRecord` captures reviewed feedback; `storage::operations::ManagerDailyBriefOutcomeRecord`, reporting groups, outcome codes, labor-minute values, and stored source refs persist proof in [`storage/src/operations.rs`](../../../storage/src/operations.rs). |
| Exposure surfaces | Local app/Rustdoc/module paths, manager daily brief design docs, local smoke docs, API contract routes where present, and future operator page [`docs/workflows/operator/manager-daily-brief.md`](../../workflows/operator/manager-daily-brief.md). |
| Related entities | Reads provenance/source facts, data-quality issues, checkout packets, CRM retention packets, labor minutes, operations service offerings, review gates, blocked actions, outcome records, and storage records. |
| Authority owner | Manager or regional operator owns management decisions; source systems own observed facts; approved sender owns customer-message decisions; storage/outcome records own measured feedback once reviewed. |
| Automation allowed/draft/recommend actions | Summarize source evidence, rank internal work, draft internal/customer follow-up for review, estimate labor impact, and record reviewed outcome feedback when validation passes. |
| Blocked or human-reviewed actions | No staff schedule changes, provider/PMS writes, customer sends, occupancy invention, refunds/discounts/payments, source ambiguity hiding, or policy/safety exceptions without the named review gate. |
| Value/outcome measure | Estimated and actual labor minutes, feedback outcome, action kind, staff persona/reporting group, source refs, and reviewed disposition. |
| Rustdoc/source/test evidence | Source: [`app/src/manager_daily_brief.rs`](../../../app/src/manager_daily_brief.rs), [`domain/src/daily_brief.rs`](../../../domain/src/daily_brief.rs), [`storage/src/operations.rs`](../../../storage/src/operations.rs). Rustdoc/module paths: `app::manager_daily_brief::{Packet, BriefAction, SourceFact, LaborImpactEstimate, OutcomeRecord}`, `storage::operations::ManagerDailyBriefOutcomeRecord`. Tests: [`app/tests/manager_daily_brief_workflow_contracts.rs`](../../../app/tests/manager_daily_brief_workflow_contracts.rs), [`app/tests/workflow_service_composition_contracts.rs`](../../../app/tests/workflow_service_composition_contracts.rs). |
| Caveats | Local/app contract and outcome-capture path are present; production data connector, operator UI, schedule writeback, and verified production labor savings are not proven by this row. |

### Entity: Data-quality hygiene candidate/action (`app::data_quality_hygiene::Candidate` / `Action`)

| Field | Crosswalk value |
| --- | --- |
| Entity | Data-quality hygiene candidate and action |
| Plain-English meaning | A review item for a duplicate, stale, missing, conflicting, incomplete, sensitive, or suspicious source fact that could waste staff time or make automation unsafe if hidden. |
| Source-entry surfaces | Data-quality issues from domain/source facts, provider mappings, source snapshots, staff/import evidence, and workflow-blocking facts. See [Data Quality Hygiene](../../design/workflow-source-rustdoc-backing-map.md#data-quality-hygiene). |
| Normalization surfaces | `domain::data_quality::{Issue, Kind, FieldPath, Severity, ResolutionStatus}` names the defect; `app::data_quality_hygiene::{Candidate, CandidateKind, SourceFreshness, Sensitivity, Action, ActionKind}` groups issues into reviewable cleanup work. |
| Workflow-use surfaces | Data Quality Hygiene packet ranks candidates, drafts internal cleanup work, validates draft submissions, blocks unsafe side effects, and records reviewed cleanup outcomes. Manager Daily Brief and Regional Exceptions can consume unresolved or high-impact issues as source risk. |
| Persistence surfaces | `app::data_quality_hygiene::OutcomeRecord` captures reviewed disposition; `storage::operations::DataQualityHygieneOutcomeRecord`, outcome codes, reporting groups, resolution status codes, labor minutes, and stored source refs persist proof. |
| Exposure surfaces | Local workflow module/tests, storage/Rustdoc module paths, hygiene design/local-smoke docs, safety/source evidence docs, and future operator page [`docs/workflows/operator/data-quality-hygiene.md`](../../workflows/operator/data-quality-hygiene.md). |
| Related entities | Depends on provenance, field path, source freshness, sensitivity/redaction, workflow packet, review gate, blocked action, labor minutes, and storage outcome record; feeds manager/regional risk views. |
| Authority owner | Front desk lead, manager, regional ops, integration owner, or privacy/security reviewer owns the cleanup decision depending on issue type and sensitivity; the source system owns observed raw facts. |
| Automation allowed/draft/recommend actions | Summarize evidence, group/rank cleanup candidates, draft internal cleanup tasks, estimate labor impact, validate draft context/source refs/review gates, and record reviewed outcomes. |
| Blocked or human-reviewed actions | No provider/PMS mutation, no source ambiguity hiding, no auto-resolving uncertain facts, no exposing sensitive/quarantined payloads, and no treating stale/unknown context as accepted. |
| Value/outcome measure | Actual/estimated labor minutes saved or wasted, cleanup disposition, resolution status, candidate kind, reporting group, source refs, and reviewer persona. |
| Rustdoc/source/test evidence | Source: [`domain/src/data_quality.rs`](../../../domain/src/data_quality.rs), [`app/src/data_quality_hygiene.rs`](../../../app/src/data_quality_hygiene.rs), [`storage/src/operations.rs`](../../../storage/src/operations.rs). Rustdoc/module paths: `domain::data_quality::{Issue, Kind, FieldPath, Severity, ResolutionStatus}`, `app::data_quality_hygiene::{Candidate, Action, DraftValidation, OutcomeRecord}`, `storage::operations::DataQualityHygieneOutcomeRecord`. Tests: [`app/tests/data_quality_hygiene_workflow_contracts.rs`](../../../app/tests/data_quality_hygiene_workflow_contracts.rs). |
| Caveats | This is a ranked/reviewed cleanup workflow, not an automated provider repair engine. Provider repair tooling and final production UI are not proven by this row. |

### Entity: Gingr retail product candidate (`gingr::mapping::retail::ProductCandidate`)

| Field | Crosswalk value |
| --- | --- |
| Entity | Gingr retail product candidate |
| Plain-English meaning | A provider retail item after Gingr-shaped fields have been read and mapped into an explicit product candidate for review, without pretending the provider item is already approved domain inventory. |
| Source-entry surfaces | Gingr commerce/retail endpoints and provider DTOs, including [`integrations/gingr/src/endpoint/commerce_retail.rs`](../../../integrations/gingr/src/endpoint/commerce_retail.rs) and [`integrations/gingr/src/dto/retail.rs`](../../../integrations/gingr/src/dto/retail.rs). |
| Normalization surfaces | [`integrations/gingr/src/mapping/retail.rs`](../../../integrations/gingr/src/mapping/retail.rs) maps provider fields into `ProductCandidate`; [`domain/src/retail/mod.rs`](../../../domain/src/retail/mod.rs) owns the domain retail contract. Missing/invalid fields should become mapping errors or data-quality issues. |
| Workflow-use surfaces | Data Quality Hygiene can review bad/missing product fields; Manager Daily Brief or future retail/reorder workflow can surface reorder/product-review opportunities. Current crosswalks should treat this as provider-boundary evidence unless a workflow test links it to an app packet. |
| Persistence surfaces | Retail/service-line storage records and service offering projections may persist reviewed domain/service-line facts; raw Gingr provider item fields are provider evidence. Use `planned/no persisted contract yet` for any direct candidate persistence not backed by storage source. |
| Exposure surfaces | Gingr provider boundary atlas, integration README/Rustdoc/module paths, retail entity atlas pages, and storage/domain retail docs. |
| Related entities | Depends on Gingr item/provider id, mapping error/provider field, provenance/source ref, retail product, inventory, POS, vendor, reorder decision, data-quality issue, and service offering record. |
| Authority owner | Gingr/provider owns observed retail item fields; integration owner owns mapping correctness; retail/service-line manager owns product approval, reorder, POS, vendor, or inventory actions. |
| Automation allowed/draft/recommend actions | Read provider retail evidence, map to a candidate, flag missing/invalid fields, draft internal review/reorder recommendations only where workflow contracts allow it. |
| Blocked or human-reviewed actions | No POS transaction, inventory adjustment, vendor order, discount/price movement, product approval, source hiding, or provider write without explicit reviewed authority. |
| Value/outcome measure | Future value may be fewer stock exceptions, cleaner retail catalog data, or reviewed reorder opportunities; claim measured value only after outcome records or inventory/reorder dispositions exist. |
| Rustdoc/source/test evidence | Source: [`integrations/gingr/src/dto/retail.rs`](../../../integrations/gingr/src/dto/retail.rs), [`integrations/gingr/src/mapping/retail.rs`](../../../integrations/gingr/src/mapping/retail.rs), [`domain/src/retail/mod.rs`](../../../domain/src/retail/mod.rs). Rustdoc/module paths: `gingr::dto::retail::Item`, `gingr::mapping::retail::ProductCandidate`, `domain::retail::Contract`. Tests should be named when mapping/reorder behavior is added or linked. |
| Caveats | Provider/read evidence and mapping shape are not product approval, POS authority, or vendor-order authority. Workflow-specific retail automation remains future/planned unless linked to app tests/source. |

## Bidirectional link rules

Every entity page, workflow page, Rustdoc/source reference, test reference, and contract map must support navigation in both directions: readers should be able to start from business language and find proof, or start from proof and find the business meaning.

### Entity page -> crosswalk row

- Each entity page should link to its contract-crosswalk row or section using the exact entity name.
- The page's `Contracts and source/Rustdoc links` table should include the same source paths and module paths as the crosswalk row, or explain why the page is narrower.
- The entity page must include related workflow links and blocked-action/review-gate links when the crosswalk row names them.

### Crosswalk row -> entity page

- The crosswalk row should link back to the entity page once the page exists. Before the page exists, link to the inventory/family page and mark `entity page planned` in caveats.
- Crosswalk related-entity cells should prefer links to existing entity/family pages over bare prose.
- If a row uses a concept from another family, link the relationship map or family atlas page rather than duplicating that page's full explanation.

### Workflow page -> crosswalk row

- Each workflow page should include a `Contract crosswalk` or `Entity evidence` section listing the entities it reads, produces, reviews, persists, or exposes.
- Workflow pages must reuse the crosswalk blocked-action language for shared no-go boundaries, then add workflow-local details from `SafeAgentAction`, `BlockedAction`, draft validation, or tests.
- Workflow pages should link to outcome-measure fields when they make labor, revenue, safety, or quality claims.

### Crosswalk row -> workflow page

- The `Workflow-use surfaces` field should link to workflow pages when present and to app source/tests when a final page is still planned.
- The row should say whether the workflow reads the entity, produces it, validates it, drafts with it, records it, or merely references it.
- Do not link a workflow as if it can mutate the entity unless a source/test contract explicitly grants that authority and the human gate is named.

### Rustdoc/source -> docs

- Rustdoc/module paths in crosswalk rows should be stable names that writers can search: `domain::source::Provenance`, `app::data_quality_hygiene::DraftValidation`, `storage::operations::DataQualityHygieneOutcomeRecord`.
- Source files with module docs should link back to operator/entity docs when source comments are intentionally user-facing. If source docs do not link back yet, the crosswalk row remains valid but can note `source back-link TODO`.
- Avoid adding broad docs links to low-level helper comments. Back-links belong on semantic domain modules, workflow modules, storage records, or public type docs that represent a business contract.

### Tests -> docs

- Crosswalk rows should name tests that enforce the behavior being claimed.
- If a test is the only proof of an edge, the row must describe the tested behavior in plain English.
- If a docs page makes a claim with no test/source proof, mark it `docs-only conceptual / TODO` and do not present it as implemented.

### Contract maps -> rows/pages

- Relationship maps should link to crosswalk rows for edge semantics and to entity/workflow pages for operator meaning.
- Crosswalk rows should link to relationship maps for adjacency context rather than re-creating whole operating-model diagrams.
- Contract maps may summarize many rows, but must preserve the same authority/automation/blocked-action outcome boundaries.

## Acceptance checklist for future rows

Before accepting a new crosswalk row, verify:

1. All required fields are present and in the standard order.
2. The plain-English meaning is understandable without reading Rust.
3. Source-entry and normalization are separated; provider evidence is not called domain truth.
4. Workflow-use says what the workflow does with the entity and what safe result it produces.
5. Persistence and exposure claims are backed or explicitly marked planned/gap.
6. Authority owner is scoped by fact/decision, not a vague single owner.
7. Allowed actions use narrow verbs backed by app/domain/tool-port contracts.
8. Blocked/human-reviewed actions are explicit for customer, safety, schedule, money, provider, and source-ambiguity surfaces.
9. Value/outcome language is measured, reviewable, or clearly hypothetical.
10. Rustdoc/source/test evidence includes concrete paths/module names/tests.
11. Caveats prevent overclaiming production readiness or autonomous authority.
12. Links work in both directions or carry a `back-link TODO` caveat.
