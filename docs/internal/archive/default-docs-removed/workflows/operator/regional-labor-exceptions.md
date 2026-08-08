# Regional Labor Exceptions / Future Portfolio View

This planned regional portfolio view would save regional operations leaders from manually scanning dashboards and collecting one-off GM narratives when a resort is off plan. It would use source-backed read models, manager daily brief outcomes, and data-quality outcomes to rank regional exceptions while a regional leader keeps approval over follow-up, staffing, policy, personnel, pricing, schedule, provider, BI, and customer-facing actions.

Status: planned/future. The repo does not yet contain a dedicated `app::regional_labor_exceptions` module, workflow packet, API endpoint, or durable regional exception outcome record.

Navigation: start with the [operator workflow index](README.md). Entity-first backlinks: [outcome/labor atlas](../../design/entity-atlas-outcomes-operations-money.md), [runtime/storage/API surfaces](../../design/entity-atlas-runtime-storage-api-surfaces.md), [source/provenance/data quality](../../design/source-provenance-data-quality-atlas.md), and [review gates and blocked actions](../../design/entity-atlas-review-safety-boundaries.md).

Example future workflow: a regional operator opens one queue showing three sites with labor-risk exceptions, the source-backed reason for each variance, related manager daily brief/data-quality outcomes, and a draft GM follow-up question for human review.

## 1. What problem does this solve?

Regional leaders need to know which sites are off plan, why, and what follow-up should be reviewed without manually reconciling BI dashboards, manager notes, demand/staffing changes, incidents, reviews, and recurring source-data caveats.

The planned page is not an automation claim. It is a regional review queue concept over existing labor, outcome, source, and data-quality vocabulary. Until a dedicated regional packet exists, the strongest evidence is that current contracts can identify local manager outcomes and portfolio/operations concepts that a future regional workflow could aggregate.

## 2. Whose time does it save?

- Regional operations leaders who otherwise scan portfolio dashboards and ask GMs for narrative explanations.
- Operations analysts who prepare exception lists, reconcile source caveats, and compare sites by reporting group.
- Portfolio leaders who need comparable, source-backed explanations across locations instead of one-off stories.
- General managers, secondarily, because follow-up questions can be narrower and tied to source evidence.

## 3. What source data does it need?

It would need portfolio or regional [read models](../../glossary-architecture-terms.md#read-model) grouped by location, period, reporting group, and metric; labor-risk and demand/staffing variance; utilization/capacity; incidents/reviews; [data-quality issues](../../glossary-source-data-terms.md#domaindata_qualityissue-data-quality-issue); manager daily brief outcomes; [source refs](../../glossary-architecture-terms.md#source-ref-domainsourcerecordref) with [provenance](../../glossary-architecture-terms.md#provenance-domainsourceprovenance); and comparable peer/trend context.

The future regional packet should cite derived read models, storage outcome records, and source-of-record facts. It should not treat raw BI screenshots, model memory, or AI-generated summaries as business truth.

| Entity or source fact | Why the workflow needs it | Source of record / authority | Evidence citation |
| --- | --- | --- | --- |
| Location, region, portfolio, reporting group | Groups exceptions by who can act and which sites are comparable. | Approved portfolio/read-model source plus storage/domain projection; future regional owner reviews grouping. | `storage/src/operations.rs` `PetResortPortfolioRecord`; `domain/src/operations.rs` `OperatingFunction::RegionalOperations`; Rustdoc `target/doc/storage/operations/struct.PetResortPortfolioRecord.html` and `target/doc/domain/operations/enum.OperatingFunction.html`. |
| Off-plan metric and labor/demand variance | Defines what is exceptional: labor risk, staffing variance, utilization/capacity, demand trend, or reporting anomaly. | Derived BI/read model plus domain operations vocabulary; not agent inference alone. | `domain/src/operations.rs` `AiUseCase::{RegionalOpsExceptionReporting, RegionalPerformanceBenchmarking}` and `OptimizationOpportunity`; `integrations/gingr/src/endpoint/labor_ops.rs` `TimeclockReport` as provider request evidence. |
| Manager daily brief outcomes | Seeds regional view with site-reviewed actions and before/after labor minutes. | App/storage manager brief outcome record; GM/front-desk disposition remains human evidence. | `app/src/manager_daily_brief.rs` `OutcomeRecord`, `LaborImpactEstimate`; `storage/src/operations.rs` `ManagerDailyBriefOutcomeRecord`; tests `app/tests/manager_daily_brief_workflow_contracts.rs`. |
| Data-quality outcomes and caveats | Explains when an exception may be wrong-source, stale, conflicting, or not comparable. | Data-quality workflow/outcome records and source refs. | `app/src/data_quality_hygiene.rs` `OutcomeRecord`; `storage/src/operations.rs` `DataQualityHygieneOutcomeRecord`; `domain/src/data_quality.rs` issue vocabulary; tests `app/tests/data_quality_hygiene_workflow_contracts.rs`. |
| Incidents, reviews, and reputation context | Gives regional leaders optional context for labor or performance exceptions without making reputation the only explanation. | Source-backed incident/review/reputation records and human review. | `domain/src/reputation.rs`; `domain/src/operations.rs`; source evidence map docs. |

## 4. Which entities are featured?

Featured entities:

- Regional/portfolio exception candidate: the future queue item that says a site, metric, period, and reason are off plan.
- Off-plan site/metric/reporting group: the comparable unit a regional leader reviews.
- Manager daily brief outcome: the local reviewed action and labor estimate/actual that can explain what the GM already handled.
- Data-quality outcome or issue: the caveat that says whether the exception is trustworthy, stale, ambiguous, or wrong-source.
- Source ref/provenance: the audit trail back to the source record or read model.

Related entities to mention without making them the page center:

- Reservation/back-of-house/timeclock provider records: source evidence for demand, occupancy, and labor; they are not regional approval authority by themselves.
- Incidents/reviews/reputation signals: optional context for the variance, not a substitute for source-backed labor or operations facts.
- Staff schedules, prices, policies, provider/PMS records, and customer messages: downstream action surfaces that remain human-approved and blocked by default.
- Agent prompt packets and workflow packets: future app-layer packaging; today they are indirect evidence through manager daily brief and data-quality workflows, not a dedicated regional packet.

## 5. Which featured contracts are listed across app/domain/storage/provider?

| Layer | Contract | What it authorizes | What it does not authorize |
| --- | --- | --- | --- |
| `app` | No dedicated `app::regional_labor_exceptions` contract exists yet. Existing seed evidence comes from `app::manager_daily_brief::{Packet, BriefAction, LaborImpactEstimate, OutcomeRecord}` and `app::data_quality_hygiene::{Packet, Candidate, OutcomeRecord}`. | Local review packets and outcome records that can later be aggregated into a regional queue. | A shipped regional agent, autonomous GM follow-up, live staffing decisions, or provider/BI mutation. |
| `domain` | `domain::operations::{AiUseCase::RegionalOpsExceptionReporting, AiUseCase::RegionalPerformanceBenchmarking, OperatingFunction::RegionalOperations, DataQualityIssue, OptimizationOpportunity}` plus `domain::daily_brief` and `domain::reputation`. | Business vocabulary for regional exception concepts, labor/capacity/reporting lanes, optional reputation context, and operating-day facts. | Source-of-record truth by itself; domain enums do not prove a workflow has been shipped. |
| `storage` | `storage::operations::{PetResortPortfolioRecord, ManagerDailyBriefOutcomeRecord, DataQualityHygieneOutcomeRecord}`. | Portfolio seed facts and durable local outcome/labor evidence that a future regional view could read. | A dedicated durable regional exception outcome projection; that remains a gap. |
| `integrations/gingr` | `gingr::endpoint::labor_ops::TimeclockReport`; `gingr::endpoint::reservations::{BackOfHouse, Reservations, GetServicesByType}`; mapping/source inventory docs. | Typed provider request surfaces for timeclock, back-of-house, reservation, and service evidence. | Domain truth, policy approval, source mutation, or live regional decision authority. |

## 6. Who or what is authoritative for each fact or decision?

- Source-of-record authority: provider/read-model/storage sources own the raw observed facts, exposed through source refs and provenance.
- Domain authority: `domain::operations`, `domain::daily_brief`, `domain::data_quality`, and `domain::reputation` name the business vocabulary and review concepts.
- App authority: existing manager daily brief and data-quality app packets decide what can be ranked, summarized, drafted, or recorded inside those local workflows.
- Storage authority: storage outcome records prove reviewed dispositions and labor-minute estimates/actuals for existing workflows.
- Human authority: regional leaders, approved GMs, operations analysts, HR/personnel owners, pricing owners, and approved systems of record approve any real-world follow-up or side effect.

The future regional page should use [source-of-record](../../glossary-source-data-terms.md#source-of-record) language whenever it names a fact. An agent-ranked exception is a recommendation, not a source of truth.

## 7. What does the agent draft, rank, recommend, validate, summarize, or record?

In the planned version, an agent could:

- Rank off-plan sites and exception groups by labor risk, source confidence, recency, and review urgency.
- Summarize source-backed evidence and trends, including whether a variance is supported by manager daily brief outcomes or blocked by data-quality caveats.
- Draft an internal regional review queue item or GM follow-up question for review.
- Recommend the next internal review action, such as “ask GM to explain staffing variance,” “wait for source cleanup,” or “compare to peer trend.”
- Record a draft recommendation or proposed disposition only after the future packet/outcome contract exists.

Existing app evidence is indirect: manager daily brief outcomes, data-quality hygiene outcomes, and operations-domain vocabulary point toward the portfolio queue. There is no dedicated `app::regional_labor_exceptions` [workflow packet](../../glossary-workflow-state-terms.md#workflow-packet) yet.

## 8. What must a human approve?

A regional leader or other approved human must approve:

- GM follow-up wording and any escalation outside an internal review queue.
- Staffing-plan or schedule changes.
- Discipline, coaching, termination, or other personnel action.
- Policy exceptions, local-rule overrides, pricing, discounts, refunds, payment movement, or customer credits.
- Customer communications or public/reputation responses.
- Provider/PMS writes, BI hiding, metric reclassification, source-data suppression, or source cleanup.

These are [review gates](../../glossary-workflow-state-terms.md#review-gate). The agent can prepare evidence and draft internal questions; it cannot approve the regional action.

## 9. What actions are blocked or human-reviewed by default?

Blocked by default:

- Sending messages to customers, GMs, staff, or vendors without approval.
- Mutating provider/PMS records, schedules, staffing plans, timecards, prices, payments, refunds, discounts, or BI dashboards.
- Hiding, deleting, merging, or reclassifying source facts and data-quality caveats.
- Making or recommending personnel discipline as an automated decision.
- Approving policy/local-rule exceptions or overriding resort/region authority.
- Publishing regional performance claims without source and human review.

These remain [blocked actions](../../glossary-workflow-state-terms.md#blocked-action) even if the future queue ranks an exception confidently.

## 10. What outcome or labor value gets measured?

Intended measured outcomes:

- Regional leader or operations analyst minutes avoided per exception queue review.
- Count of off-plan sites reviewed with a source-backed next action.
- Completed, deferred, suppressed, wrong-source, not-comparable, and needs-source-cleanup dispositions by metric/reporting group.
- Number of GM follow-up requests narrowed by source evidence rather than broad narrative collection.
- Recurring data-quality or local-rule caveats that explain regional comparison gaps.

Current evidence status: planned/future for regional outcome capture. Existing code can cite manager daily brief and data-quality hygiene outcome records for local labor evidence, but there is no durable regional exception outcome record yet. Any labor-value statement on this page should be framed as intended [outcome capture](../../glossary-workflow-state-terms.md#outcome-capture), not measured production savings.

## 11. What code/Rustdoc/test evidence backs this up?

Operator evidence and design:

- [Entity-driven workflow page template and evidence matrix](../../design/entity-driven-workflow-page-template.md#seven-workflow-pages-expected-featured-entities-and-contracts) for the required regional page contract and gap language.
- [Operator workflow page inventory](../../design/operator-workflow-page-inventory.md#intended-page-set) for regional audience, source data, approvals, and outcome targets.
- [Workflow source/Rustdoc backing map](../../design/workflow-page-source-rustdoc-map.md#regional-labor-exceptions-future-portfolio-view) for current evidence and gaps.
- [Labor-cost reduction crosswalk](../../design/labor-cost-reduction-crosswalk.md) and [labor-cost platform readiness audit](../../audits/2026-06-18-labor-cost-platform-readiness.md) for broader regional/labor-loop context.

Source and Rustdoc evidence:

- Source: [domain/src/operations.rs](../../../domain/src/operations.rs) (`domain::operations::{AiUseCase::RegionalOpsExceptionReporting, AiUseCase::RegionalPerformanceBenchmarking, OperatingFunction::RegionalOperations, DataQualityIssue, OptimizationOpportunity}`); Rustdoc `target/doc/domain/operations/index.html`, `target/doc/domain/operations/enum.AiUseCase.html`, and `target/doc/domain/operations/enum.OperatingFunction.html`; status: planned/future vocabulary proof, not a shipped workflow.
- Source: [domain/src/daily_brief.rs](../../../domain/src/daily_brief.rs) (`domain::daily_brief` operating-day facts); Rustdoc `target/doc/domain/daily_brief/index.html`; status: source vocabulary that can feed aggregate reporting.
- Source: [domain/src/reputation.rs](../../../domain/src/reputation.rs) (`domain::reputation` review/reputation signals); Rustdoc `target/doc/domain/reputation/index.html`; status: optional context, not authority for labor exceptions by itself.
- Source: [app/src/manager_daily_brief.rs](../../../app/src/manager_daily_brief.rs) (`app::manager_daily_brief::{Packet, BriefAction, LaborImpactEstimate, OutcomeRecord}`); Rustdoc `target/doc/app/manager_daily_brief/index.html`; tests [app/tests/manager_daily_brief_workflow_contracts.rs](../../../app/tests/manager_daily_brief_workflow_contracts.rs); status: supported local manager workflow/outcome evidence that could seed a future regional queue.
- Source: [app/src/data_quality_hygiene.rs](../../../app/src/data_quality_hygiene.rs) (`app::data_quality_hygiene::{Packet, Candidate, OutcomeRecord}`); Rustdoc `target/doc/app/data_quality_hygiene/index.html`; tests [app/tests/data_quality_hygiene_workflow_contracts.rs](../../../app/tests/data_quality_hygiene_workflow_contracts.rs); status: supported local data-quality outcome evidence that can explain unreliable regional comparisons.
- Source: [storage/src/operations.rs](../../../storage/src/operations.rs) (`storage::operations::{PetResortPortfolioRecord, ManagerDailyBriefOutcomeRecord, DataQualityHygieneOutcomeRecord}`); Rustdoc `target/doc/storage/operations/index.html`, `target/doc/storage/operations/struct.PetResortPortfolioRecord.html`, `target/doc/storage/operations/struct.ManagerDailyBriefOutcomeRecord.html`, and `target/doc/storage/operations/struct.DataQualityHygieneOutcomeRecord.html`; status: portfolio seed and existing outcome evidence, with no dedicated regional exception outcome record.
- Source: [integrations/gingr/src/endpoint/labor_ops.rs](../../../integrations/gingr/src/endpoint/labor_ops.rs) (`gingr::endpoint::labor_ops::TimeclockReport`) and [integrations/gingr/src/endpoint/reservations.rs](../../../integrations/gingr/src/endpoint/reservations.rs) (`BackOfHouse`, `Reservations`, `GetServicesByType`); Rustdoc `target/doc/gingr/endpoint/labor_ops/index.html` and `target/doc/gingr/endpoint/reservations/index.html`; status: provider request evidence only, not approval or side-effect authority.

Generated Rustdoc backing is local under `target/doc/` after running `cargo doc --no-deps --workspace`.

## 12. Caveats and future source need

- Preserve the planned/future label until code adds a dedicated regional exception app module, packet, API endpoint, and durable regional exception outcome record.
- Do not claim production-verified NVA regional labor savings, live provider write access, autonomous GM messaging, BI mutation, or personnel/staffing authority.
- Region/resort/local-rule exceptions are likely page angles, but current code evidence is portfolio/operations vocabulary and existing local outcome records, not a complete regional policy engine.
- Future implementation should add explicit region/resort/policy entities, source authority for regional/local rules, a regional packet with allowed and blocked actions, reviewable GM follow-up drafts, regional outcome storage, and tests proving the human-approval boundaries.

## Contract crosswalk links

Use the [workflow packet row](../../entity-atlas/contract-crosswalk/workflow-packets.md#workflow-by-workflow-entity-map) for the bidirectional path from this workflow page back to the entities it consumes and produces. Use [surface inventory](../../entity-atlas/contract-crosswalk/surface-inventory.md) for source/Rustdoc/test proof, [source/provider flows](../../entity-atlas/contract-crosswalk/source-provider-flows.md) for source-entry and normalization evidence, [storage/persistence](../../entity-atlas/contract-crosswalk/storage-persistence.md) for `future regional outcome storage gap`, and [runtime exposure](../../entity-atlas/contract-crosswalk/runtime-exposure.md) for API/worker/CLI/web/script exposure. Rustdoc/module path: `future app::regional_labor_exceptions gap`; operator-facing entity family: `Regional labor exception portfolio signal`.
