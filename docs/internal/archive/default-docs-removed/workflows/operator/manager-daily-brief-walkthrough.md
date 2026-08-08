# Manager Daily Brief end-to-end walkthrough

This walkthrough follows one resort morning from source evidence to reviewed outcome proof. It is deliberately narrow: the Manager Daily Brief saves manager/front-desk reconciliation time by preparing a source-grounded review packet, but it does not change schedules, write Gingr/PMS records, send customer messages, move money, hide source issues, or prove production NVA labor savings by itself.

Use it when a cold reader needs the complete proof chain without opening ten pages first. For the full workflow contract, read [Manager Daily Brief](manager-daily-brief.md). For the labor-driver map, read [labor-cost reduction crosswalk](../../design/labor-cost-reduction-crosswalk.md).

## One morning in the flow

Scenario: before 8 a.m., a general manager wants to know what needs attention today. The app receives source-backed operating-day context for one location: a boarding demand fact above the attention threshold, an open checkout exception, an eligible retention follow-up, and one service-demand caveat with source-quality warning.

1. Source/read-model evidence enters.
   - Concrete proof paths: `domain::analytics::service_demand::Fact`, `domain::operations::operating_day::Key`, and `domain::source::RecordRef` appear in `app/src/manager_daily_brief.rs` and `app/tests/manager_daily_brief_workflow_contracts.rs`.
   - Provider/read-model boundary: Gingr reservation, back-of-house, timeclock, or BI/read-model rows are evidence receipts. They must be normalized and source-referenced before the brief treats them as app/domain facts.
   - Caveat: BI/read models measure and expose visibility; they do not authorize staffing changes or live provider writes.

2. Provenance and data-quality caveats stay attached.
   - Each valid brief action needs non-empty source refs. Data-quality issues are preserved as `SourceFactKind::SourceDataQualityIssue` instead of being hidden.
   - Concrete proof paths: `domain/src/source.rs`, `domain/src/data_quality.rs`, `app/src/manager_daily_brief.rs`, and the `manager_daily_brief_contract_preserves_review_boundaries_and_data_quality_visibility` test.
   - Reader rule: provenance proves traceability, not truth, freshness, or approval.

3. The app builds a Manager Daily Brief packet.
   - The packet is scoped to one `location_id` and one `operating_day`, then ranks reviewable actions such as `ReviewDemandAgainstStaffingPlan`, `ResolveCheckoutException`, `ApproveRetentionFollowUpDraft`, and `InvestigateSourceDataQualityIssue`.
   - Concrete proof paths: `app::manager_daily_brief::{Request, Packet, BriefAction, BriefActionKind, SourceFact, LaborImpactEstimate}` in `app/src/manager_daily_brief.rs`; location/day filtering tests in `app/tests/manager_daily_brief_workflow_contracts.rs`.
   - Adjacent packet inputs: checkout context comes from `app/src/checkout_completion.rs`; retention context comes from `app/src/crm_retention.rs`.

4. Agent-safe work becomes a summary, ranking, or draft recommendation.
   - Allowed work: summarize source evidence, rank manager actions, draft internal review notes/tasks, estimate labor minutes saved, and prepare outcome feedback when a human disposition exists.
   - Blocked work: change staff schedule, mutate provider/PMS records, send a customer message, move refund/discount/payment, hide source data-quality issues, or treat a local example as production ROI.
   - Concrete proof paths: `SafeAgentAction`, `BlockedAction`, and `requested_side_effect_rejection_reason` in `app/src/manager_daily_brief.rs`; side-effect rejection tests in `app/tests/manager_daily_brief_workflow_contracts.rs`.

5. A manager or front-desk lead reviews the work.
   - The brief can say “review demand against staffing plan first” or “approve/reject this retention follow-up draft,” but the human owns the live decision.
   - Review gates remain attached: manager approval for staffing/source-quality/checkout exceptions and customer-message approval for retention follow-up drafts.
   - Concrete proof paths: `domain::policy::ReviewGate`, `app/tests/manager_daily_brief_workflow_contracts.rs`, and the [relationship adjacency flow](../../entity-atlas/contract-crosswalk/relationship-adjacency.md#flow-3-workflow-packet-into-draft-review-outcome-and-storage-proof).

6. Outcome and labor minutes are recorded only after review.
   - The app outcome records action id, actor, disposition, before minutes, actual minutes, source refs, and actual minutes saved.
   - Example local contract evidence: demand + retention actions produce 75 minutes before, 25 minutes after, and 50 estimated minutes saved; a reviewed demand outcome with 45 before and 12 actual minutes saves 33 actual minutes.
   - Concrete proof paths: `app::manager_daily_brief::OutcomeRecord`, `FeedbackOutcome`, and `LaborMinutes` in `app/src/manager_daily_brief.rs`; outcome tests in `app/tests/manager_daily_brief_workflow_contracts.rs`.

7. Storage/runtime/reporting expose proof without becoming authority.
   - Storage-shaped records keep reporting dimensions such as location, operating day, action kind, owner persona, source refs, estimated minutes, and actual minutes saved.
   - Concrete proof paths: `storage::operations::ManagerDailyBriefOutcomeRecord`, `StoredManagerDailyBriefLaborMinutes`, and `StoredSourceRecordRef` in `storage/src/operations.rs`; `storage/tests/manager_daily_brief_outcome_storage.rs`.
   - Caveat: API route state, local smoke scripts, static staff web, and generated docs are local/demo/proof surfaces unless a production database, outbox, and live integration contract are explicitly proven.

## Chain summary

| Chain step | What the reader should trust | What remains out of scope |
| --- | --- | --- |
| Source/read-model evidence | Source refs, normalized service-demand/checkout/retention facts, and visible caveats. | Raw provider rows as automatic business truth. |
| Provenance/data quality | Traceability and explicit ambiguity. | Hiding, auto-fixing, or silently overriding source problems. |
| App workflow packet | Ranked, scoped, source-grounded manager actions. | Schedule/PMS/provider/customer/payment side effects. |
| Agent-safe draft/ranking | Summaries, internal notes, review recommendations, and labor estimates. | Autonomous live decisions. |
| Human review gate | Manager/front-desk/customer-message approval before action. | Treating review gates as optional paperwork. |
| Outcome/labor minutes | Reviewed disposition and before/actual minutes. | Production NVA ROI claims without production outcome evidence. |
| Storage/runtime/reporting proof | Durable-shaped local proof, tests, Rustdoc, smoke/API inspection. | Runtime shell or reporting DB as workflow authority. |

## Where to verify next

- Workflow page: [Manager Daily Brief](manager-daily-brief.md)
- Labor measurement contract: [Manager Daily Brief measurable labor loop](../../design/manager-daily-brief-measurable-labor-loop.md)
- Labor-driver hub: [Labor-cost reduction crosswalk](../../design/labor-cost-reduction-crosswalk.md)
- Relationship proof chain: [Relationship adjacency and flow diagrams](../../entity-atlas/contract-crosswalk/relationship-adjacency.md)
- Workflow packet proof: [Workflow packets contract crosswalk](../../entity-atlas/contract-crosswalk/workflow-packets.md)
- Storage proof: [Storage/persistence crosswalk](../../entity-atlas/contract-crosswalk/storage-persistence.md)
- Runtime proof and caveats: [Runtime exposure crosswalk](../../entity-atlas/contract-crosswalk/runtime-exposure.md)
- Source/Rust/test paths: `app/src/manager_daily_brief.rs`, `app/tests/manager_daily_brief_workflow_contracts.rs`, `storage/tests/manager_daily_brief_outcome_storage.rs`, and `storage/src/operations.rs`
