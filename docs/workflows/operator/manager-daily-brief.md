# Manager Daily Brief

Manager Daily Brief saves the morning scramble for a general manager, assistant GM, or front-desk lead. It turns source-backed operating-day facts into a reviewed priority list while humans keep approval over staffing, schedules, customer messages, provider/PMS changes, payments, policy exceptions, and source cleanup.

Status: implemented local contract with storage outcome evidence; not proof of production NVA labor savings or live provider-write authority.

Navigation: start with the [operator workflow index](README.md). For one concrete source-to-outcome chain, read the [Manager Daily Brief end-to-end walkthrough](manager-daily-brief-walkthrough.md). Entity-first backlinks: [outcome/labor atlas](../../design/entity-atlas-outcomes-operations-money.md), [PetSuites core entities](../../design/entity-atlas-petsuites-core-entities.md), [source/provenance/data quality](../../design/source-provenance-data-quality-atlas.md), [revenue opportunity entities](../../design/entity-atlas-revenue-opportunity-entities.md), and [runtime/storage/API surfaces](../../design/entity-atlas-runtime-storage-api-surfaces.md).

Example: before 8 a.m., the brief can show that boarding demand crossed the attention threshold, one checkout handoff is still open, reported grooming/retention history remains suppressed as ineligible evidence, and one service-demand fact carries a source-quality warning. The manager sees which supported actions to review first, why they appear, which [source refs](../../glossary-architecture-terms.md#source-ref-domainsourcerecordref) back them, and that retention evidence contributes no action or labor estimate.

## Problem solved and reported labor evidence

- Problem solved: managers otherwise reconcile daily demand, occupancy/service mix, staffing pressure, checkout exceptions, prior outcomes, source/data-quality ambiguity, and separately reported retention evidence across several systems before the resort can act. Retention evidence remains visible only as suppressed context and cannot become a brief opportunity or action.
- First roles associated with caller-reported handling-time evidence: general managers and assistant GMs building the morning operating priorities. The report does not establish measured labor.
- Secondary reviewers/operators: front-desk leads reviewing checkout exceptions and reported retention evidence, operations analysts reviewing recurring source blockers, and regional operators comparing outcome evidence later. Current retention packets create no queue.
- Pet-resort example: a source-grounded [workflow packet](../../glossary-workflow-state-terms.md#workflow-packet) ranks “review demand against staffing plan” while retaining serialized CRM history only as ineligible evidence; it cannot add an “approve retention follow-up draft” action.

## Source data and featured entities

The workflow needs normalized app/domain facts, not raw provider payloads or model memory. Gingr/provider reservations, back-of-house, and timeclock records can be evidence, but authority comes after those facts are mapped into source-grounded domain/app values with [provenance](../../glossary-architecture-terms.md#provenance-domainsourceprovenance).

| Entity or source fact | Why the workflow needs it | Source of record / authority | Evidence citation |
| --- | --- | --- | --- |
| Operating day and location | Scopes every action so facts from another site or date do not leak into the brief. | `domain::operations::operating_day` and `domain::entities::LocationId`; app request scope. | `app/src/manager_daily_brief.rs` (`Request`, `ScopedCheckoutPacket`, `ScopedRetentionPacket`); `app/tests/manager_daily_brief_workflow_contracts.rs` location/day filtering tests. |
| Demand, occupancy, and service mix | Shows where boarding/daycare/grooming/training pressure creates daily operating priorities. | Domain/read-model facts; provider evidence only after normalization. | `domain/src/daily_brief.rs` (`OccupancySnapshot`, `Section`, `LaborSnapshot`); `domain/src/analytics.rs` (`service_demand::Fact`); `app/src/manager_daily_brief.rs` (`BriefActionKind::ReviewDemandAgainstStaffingPlan`). |
| Staffing or labor risk | Explains why a manager should review labor coverage without letting the agent change schedules. | Manager/human decision backed by domain daily-brief/labor facts. | `domain/src/daily_brief.rs` (`LaborRisk`, `Action::SuggestScheduleReview`); `app/src/manager_daily_brief.rs` (`BlockedAction::ChangeStaffSchedule`). |
| Checkout packet | Surfaces unresolved open-stay or departure handoff work for the front desk. | Checkout workflow packet and source checkout/PMS provenance. | `app/src/checkout_completion.rs`; `app/src/manager_daily_brief.rs` (`ScopedCheckoutPacket`, `BriefActionKind::ResolveCheckoutException`); `app/tests/manager_daily_brief_workflow_contracts.rs`. |
| Retention packet | Preserves reported CRM history as ineligible evidence. It cannot create a brief action, candidate, ranking, queue item, internal task, customer draft, contact, conversion, completion, or value claim. | CRM retention packet plus reported checkout and contact/consent evidence; no current authority issuer exists. | `app/src/crm_retention.rs`; `app/src/manager_daily_brief.rs` (`ScopedRetentionPacket`, categorical empty `retention_actions`); serialized-authority regressions in `app/tests/crm_retention_workflow_contracts.rs` and `app/tests/manager_daily_brief_workflow_contracts.rs`. |
| Data-quality issue | Keeps stale, unmapped, missing, or conflicting facts visible instead of letting the agent hide ambiguity. | `domain::data_quality::Issue` with source provenance; human cleanup approval. | `domain/src/data_quality.rs`; `app/src/manager_daily_brief.rs` (`SourceFactKind::SourceDataQualityIssue`, `BlockedAction::HideSourceDataQualityIssue`); review-boundary test in `app/tests/manager_daily_brief_workflow_contracts.rs`. |
| Labor-impact estimate and outcome record | Records reported estimate differences, disposition, and actual time spent without calculating realized savings. | App outcome record and storage projection; always nonclaimable evidence. | `app/src/manager_daily_brief.rs` (`LaborImpactEstimate`, `OutcomeRecord`); `storage/src/operations.rs` (`ManagerDailyBriefOutcomeRecord`, `StoredManagerDailyBriefLaborMinutes`). |

Featured entities on this page are the operating day/location, demand/occupancy/labor facts, brief actions, source facts, labor-impact estimates, and outcome records.

Related entities to mention without making them the page center:

- Reservation, checkout, customer, pet, grooming, message, and consent/contact entities: they feed checkout or retention packets, but this page does not become the checkout, rebooking, or customer-message workflow.
- Provider records and Gingr endpoints: they are evidence and mapping inputs, not the agent’s source of authority.
- Data-quality issues and source refs: they explain blockers and auditability; the brief cannot repair or hide them by itself.
- Regional/portfolio views: they may later aggregate manager brief outcomes, but regional exception ranking belongs to the regional workflow page unless a dedicated regional read model lands.

Entity flow: operating day/location + demand/occupancy/labor/source facts + checkout/hygiene packets + ineligible reported retention evidence -> manager brief packet and supported ranked actions -> GM/front-desk/regional review -> outcome record and storage projection. Retention evidence contributes no action or labor estimate.

## Featured contracts

| Layer | Contract | What it authorizes | What it does not authorize |
| --- | --- | --- | --- |
| `app` | `app::manager_daily_brief::{Request, Packet, BriefAction, BriefActionKind, SourceFact, SafeAgentAction, BlockedAction, LaborImpactEstimate, OutcomeRecord, Workflow}` | Build a source-grounded review packet, rank manager actions, expose safe agent work, reject side-effect requests, and retain caller-reported outcome/time evidence. | Proof of review, completion, measured labor/value, live staffing/schedule mutation, provider/PMS writes, customer sends, payment/refund/discount movement, policy exceptions, or source hiding. |
| `app` | `app::checkout_completion` and `app::crm_retention` packets scoped into the request | Feed checkout review evidence and preserve reported retention evidence in request scope. Retention packets contribute no action or labor estimate. | Let the brief establish retention eligibility, rank or enqueue retention work, create a retention task or draft, perform checkout completion, booking mutation, or messaging side effects. |
| `domain` | `domain::daily_brief::{ResortOperatingDay, OccupancySnapshot, LaborRisk, CustomerFollowUp, RevenueOpportunity, Action}` | Define manager-facing operating-day, occupancy, labor, follow-up, revenue, and approval vocabulary. | Provider-specific payload authority or live operational execution. |
| `domain` | `domain::analytics::service_demand::Fact`, `domain::source::{RecordRef, Provenance}`, `domain::data_quality::Issue`, `domain::policy::ReviewGate` | Preserve demand evidence, source lineage, issue visibility, and human-review gates. | Model-invented facts, unsupported source cleanup, or bypassing approvals. |
| `storage` | `storage::operations::{ManagerDailyBriefOutcomeRecord, ManagerDailyBriefOutcomeCode, ManagerDailyBriefPersonaCode, ManagerDailyBriefActionKindCode, StoredManagerDailyBriefLaborMinutes, StoredSourceRecordRef}` | Persist before/after labor minutes, disposition, actor/persona, reporting group, source refs, and derived reported time evidence. | Business policy decisions, production ROI proof, or any provider/customer side effect. |
| `integrations/gingr` | Reservation/back-of-house/timeclock endpoint and mapping surfaces | Supply provider evidence that can be normalized into app/domain facts. | Direct business truth in operator prose or unreviewed live mutation. |

## Authority and source of truth

- Source systems/providers are authority for observed reservation, back-of-house, timeclock, and source-record details, but the brief should cite normalized app/domain facts rather than raw payloads.
- Domain modules are authority for operating-day meaning, service-demand/labor vocabulary, data-quality issue semantics, provenance, and [review gates](../../glossary-workflow-state-terms.md#review-gate).
- The app workflow is authority for packet assembly, ranked action kinds, allowed agent actions, blocked actions, and side-effect rejection.
- Storage is authority only for durable caller-reported outcome/labor evidence, including reported minute, disposition, actor/persona, reporting-group, and source-reference labels. Persistence authenticates no reviewer, review, action, completion, measurement, or value.
- Humans are authority for live decisions: staffing or schedule changes, data cleanup, policy exceptions, customer follow-up approval, payment/discount/refund movement, provider/PMS updates, and customer messages.

## Agent work, approvals, and blocked actions

Agent may:

- Summarize the operating-day packet in non-coder language.
- Rank supported brief actions such as demand-vs-staffing review, checkout exception resolution, capacity review, and source/data-quality investigation. Retention follow-up approval is unavailable.
- Draft internal tasks or review notes for a manager/front-desk queue.
- Calculate a reported time difference from caller-provided estimates in a reviewable workflow packet; do not call it measured labor or savings.
- Retain caller-reported outcome-feedback labels for inspection without asserting that a human disposition or review occurred.

Human must approve:

- Staffing, schedule, coverage, or labor-plan changes.
- Source cleanup, duplicate merge/delete, stale-vaccine/profile fixes, or service-line mapping corrections.
- Retention promotion is unavailable: no human review of the current serialized packet can manufacture a candidate, queue item, internal task, follow-up draft, contact, conversion, completion, or value authority.
- Provider/PMS updates, booking/status movement, checkout finalization, payments, refunds, discounts, or policy/safety exceptions.
- Any regional or personnel implication that uses brief outcomes outside the local manager review loop.

[Blocked actions](../../glossary-workflow-state-terms.md#blocked-action) by default:

- Change staff schedule.
- Mutate provider or PMS record.
- Send customer message.
- Move refund, discount, or payment.
- Hide a source data-quality issue.
- Treat local examples as production NVA labor-savings proof.

## Outcome and labor value

- Reported labor estimate evidence: caller-reported before/after minute estimates per ranked action and operating day; their difference is not realized savings.
- Reported outcome evidence: `app::manager_daily_brief::OutcomeRecord` retains action id, caller-reported actor and disposition labels, before-minute estimates, reported actual minutes, optional feedback, and source refs. The actor label does not authenticate a reviewer, and the record proves no review or completion. Serializable outcome history exposes no realized-savings field or accessor, and no production claim-authority issuer exists. Deferred, suppressed, and wrong-source labels remain visible.
- Current evidence status: nonclaimable local app contract, Rustdoc/doctest example, focused workflow tests, and storage projection evidence.
- Caveat/future source need: this repo evidence does not prove production NVA labor savings, live provider write access, automated staffing changes, or customer-message sends. Any regional aggregation should be described as future/adjacent unless supported by a dedicated regional read model.
- Reported outcome labels: completed-label, deferred-label, suppressed-label, and wrong-source-label. These are nonclaimable caller reports; they authenticate no completion, manager action, source adjudication, labor reduction, or value.

## Contract crosswalk links

Use the [workflow packet row](../../entity-atlas/contract-crosswalk/workflow-packets.md#workflow-by-workflow-entity-map) for the bidirectional path from this workflow page back to the entities it consumes and produces. Use [surface inventory](../../entity-atlas/contract-crosswalk/surface-inventory.md) for source/Rustdoc/test proof, [source/provider flows](../../entity-atlas/contract-crosswalk/source-provider-flows.md) for source-entry and normalization evidence, [storage/persistence](../../entity-atlas/contract-crosswalk/storage-persistence.md) for `manager daily-brief outcome record`, and [runtime exposure](../../entity-atlas/contract-crosswalk/runtime-exposure.md) for API/worker/CLI/web/script exposure. Rustdoc/module path: `app::manager_daily_brief::Packet`; operator-facing entity family: `Manager Daily Brief packet`.

## Evidence citations

- Source: `app/src/manager_daily_brief.rs` (`app::manager_daily_brief::{Request, Packet, BriefAction, BriefActionKind, SourceFact, SafeAgentAction, BlockedAction, requested_side_effect_rejection_reason, LaborImpactEstimate, OutcomeRecord, Workflow}`); status: supported local contract for review packet, ranked actions, safe/blocked agent boundary, labor estimate, and outcome capture.
- Source: `domain/src/daily_brief.rs` (`domain::daily_brief::{ResortOperatingDay, Section, OccupancySnapshot, LaborSnapshot, LaborRisk, CustomerFollowUp, RevenueOpportunity, Action}`); status: supported domain vocabulary for daily operating priorities, occupancy/service mix, labor risk, follow-up queues, and manager attention.
- Source: `domain/src/analytics.rs` (`domain::analytics::service_demand::Fact`), `domain/src/source.rs` (`domain::source::{RecordRef, Provenance}`), `domain/src/data_quality.rs` (`domain::data_quality::Issue`), and `domain/src/policy.rs` (`domain::policy::ReviewGate`); status: supported source/demand/data-quality/review-gate evidence.
- Source: `storage/src/operations.rs` (`storage::operations::{ManagerDailyBriefOutcomeRecord, ManagerDailyBriefOutcomeCode, ManagerDailyBriefPersonaCode, ManagerDailyBriefActionKindCode, StoredManagerDailyBriefLaborMinutes, StoredSourceRecordRef}`); status: supported durable outcome/labor projection, not authority for live side effects.
- Source: `app/src/checkout_completion.rs` and `app/src/crm_retention.rs`; status: checkout packets can support exception review, while retention packets remain ineligible reported evidence and cannot produce a brief action, queue, task, draft, contact, conversion, completion, or value authority.
- Source/provider evidence: `integrations/gingr/src/endpoint/reservations.rs`, `integrations/gingr/src/endpoint/labor_ops.rs`, `integrations/gingr/src/mapping/mod.rs`, and `docs/integrations/gingr/bi-read-model-contract.md`; status: provider/read-model evidence boundary only.
- Tests: `app/tests/manager_daily_brief_workflow_contracts.rs` covers source-grounded ranked actions, location/day scoping, data-quality visibility, blocked side-effect rejection, and outcome capture without external mutation.
- Supporting docs: `docs/design/entity-driven-workflow-page-template.md`, `docs/design/operator-workflow-page-inventory.md`, `docs/design/workflow-page-source-rustdoc-map.md`, `docs/design/manager-daily-brief-measurable-labor-loop.md`, `docs/design/labor-cost-reduction-crosswalk.md`, `docs/ops/manager-daily-brief-local-smoke.md`, and `docs/ops/hermes-manager-daily-brief-bridge.md`.
- Rustdoc: local generated docs live under `target/doc/app/manager_daily_brief/index.html`, `target/doc/domain/daily_brief/index.html`, `target/doc/domain/analytics/index.html`, `target/doc/domain/source/index.html`, and `target/doc/storage/operations/index.html` after running `cargo doc --no-deps --workspace`.
- Evidence status: supported local contract and storage outcome evidence. Known caveat: no production-verified labor savings, no live provider/customer/payment/schedule side effects, and no dedicated regional aggregation claim on this page.
