//! Manager Daily Brief workflow rules for labor-saving internal review.
//!
//! The workflow starts from app-owned, source-grounded context and produces a
//! deterministic packet that an agent may summarize or rank, but not use as
//! authority to mutate schedules, provider/PMS records, customer channels, or
//! money movement. Outcome records then capture whether the reviewed action
//! actually reduced manager/front-desk labor.
//!
//! Crosswalk navigation: operator docs link this workflow to the
//! Manager Daily Brief packet row in
//! `docs/entity-atlas/contract-crosswalk/workflow-packets.md`; storage outcome
//! projection evidence lives in `storage-persistence.md`, runtime/API exposure
//! in `runtime-exposure.md`, and executable proof in
//! `app/tests/manager_daily_brief_workflow_contracts.rs` plus API/storage tests.
//!
//! ```
//! use app::manager_daily_brief as brief;
//! use chrono::NaiveDate;
//! use domain::{analytics, entities, operations, source};
//! use uuid::Uuid;
//!
//! let location_id = entities::LocationId(Uuid::from_u128(0x170));
//! let operating_day = operations::operating_day::Date::try_new(
//!     NaiveDate::from_ymd_opt(2026, 6, 18).expect("fixture date is valid"),
//! )?;
//! let source_ref = source::RecordRef::new(
//!     source::System::BusinessIntelligence,
//!     source::record::Id::try_new("labor-read-model:boarding-demand:2026-06-18")?,
//! );
//! let demand_fact = analytics::service_demand::Fact::try_new(
//!     analytics::service_demand::Id::try_new("boarding-demand-risk")?,
//!     operations::operating_day::Key::new(
//!         location_id,
//!         operations::service_core::ServiceLine::Boarding,
//!         operating_day,
//!     ),
//!     analytics::service_demand::DemandUnits::try_new(42)?,
//!     vec![source_ref.clone()],
//!     analytics::ProjectionVersion::try_new("manager-brief-fixture-v1")?,
//!     vec![],
//! )?;
//!
//! let request = brief::Request::builder()
//!     .location_id(location_id)
//!     .operating_day(operating_day)
//!     .prepared_for(brief::ManagerBriefPersona::GeneralManager)
//!     .demand_attention_threshold(brief::DemandThresholdUnits::try_new(25)?)
//!     .service_demand_facts(vec![demand_fact])
//!     .build();
//!
//! let packet = brief::Workflow::evaluate(request);
//!
//! assert_eq!(packet.actions().len(), 1);
//! assert!(packet.all_actions_are_source_grounded());
//! assert!(packet.safe_agent_actions().contains(&brief::SafeAgentAction::RankManagerActions));
//! assert!(packet.blocked_actions().contains(&brief::BlockedAction::ChangeStaffSchedule));
//! assert!(packet.blocked_actions().contains(&brief::BlockedAction::MutateProviderOrPmsRecord));
//! assert!(packet.minutes_saved() > 0);
//!
//! let outcome = brief::OutcomeRecord::builder()
//!     .action_id(packet.actions()[0].id().clone())
//!     .recorded_by(entities::ActorRef::Manager {
//!         manager_id: entities::ManagerId::try_new("gm-fixture")?,
//!     })
//!     .outcome(brief::FeedbackOutcome::Completed)
//!     .before_minutes(brief::LaborMinutes::try_new(45)?)
//!     .actual_minutes(brief::LaborMinutes::try_new(12)?)
//!     .source_record_refs(vec![source_ref])
//!     .build();
//!
//! assert!(outcome.records_feedback_without_external_mutation());
//! assert!(outcome.blocked_actions().contains(&brief::BlockedAction::SendCustomerMessage));
//! assert_eq!(outcome.actual_minutes_saved(), 33);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
use domain::{analytics, entities, operations, policy, source};
use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::{checkout_completion, crm_retention};

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 1200),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct BriefSummary(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct ActionId(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 500),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct ActionRationale(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Labor minutes used by the manager daily brief workflow; it assembles reviewable manager brief packets from deterministic context and agent drafts.
pub struct LaborMinutes(u16);

impl LaborMinutes {
    /// Validates a non-zero value for the manager daily brief workflow before it can appear in a manager packet or outcome record.
    pub const fn try_new(value: u16) -> Result<Self> {
        if value == 0 {
            return Err(Error::ZeroLaborMinutes);
        }
        Ok(Self(value))
    }

    /// Returns the numeric value available to manager daily brief review without touching provider, customer, payment, or schedule systems.
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Aggregate labor minutes used by the manager daily brief workflow; it assembles reviewable manager brief packets from deterministic context and agent drafts.
pub struct AggregateLaborMinutes(u16);

impl AggregateLaborMinutes {
    /// Stores the reviewed value for the manager daily brief workflow without triggering provider, customer, payment, or schedule side effects.
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// Returns the numeric value available to manager daily brief review without touching provider, customer, payment, or schedule systems.
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Demand threshold units used by the manager daily brief workflow; it assembles reviewable manager brief packets from deterministic context and agent drafts.
pub struct DemandThresholdUnits(u32);

impl DemandThresholdUnits {
    /// Validates a non-zero value for the manager daily brief workflow before it can appear in a manager packet or outcome record.
    pub const fn try_new(value: u32) -> Result<Self> {
        if value == 0 {
            return Err(Error::ZeroDemandThresholdUnits);
        }
        Ok(Self(value))
    }

    /// Returns the numeric value available to manager daily brief review without touching provider, customer, payment, or schedule systems.
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for manager brief persona in the manager daily brief workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum ManagerBriefPersona {
    /// Selects general manager for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    GeneralManager,
    /// Selects assistant general manager for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    AssistantGeneralManager,
    /// Selects front desk lead for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    FrontDeskLead,
    /// Selects front desk agent for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    FrontDeskAgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for removed manual work in the manager daily brief workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum RemovedManualWork {
    /// Selects morning dashboard reconciliation for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    MorningDashboardReconciliation,
    /// Selects demand versus staffing scan for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    DemandVersusStaffingScan,
    /// Selects checkout exception audit for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    CheckoutExceptionAudit,
    /// Selects retention follow up queue prioritization for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    RetentionFollowUpQueuePrioritization,
    /// Selects data quality exception triage for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    DataQualityExceptionTriage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for source fact kind in the manager daily brief workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum SourceFactKind {
    /// Selects service demand forecast for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    ServiceDemandForecast,
    /// Selects checkout completion status for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    CheckoutCompletionStatus,
    /// Selects retention follow up eligibility for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    RetentionFollowUpEligibility,
    /// Selects source data quality issue for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    SourceDataQualityIssue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Source fact used by the manager daily brief workflow; it assembles reviewable manager brief packets from deterministic context and agent drafts.
pub struct SourceFact {
    kind: SourceFactKind,
    summary: BriefSummary,
    #[builder(default)]
    source_record_refs: Vec<source::RecordRef>,
}

impl SourceFact {
    /// Returns the kind evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn kind(&self) -> SourceFactKind {
        self.kind
    }

    /// Returns the summary evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn summary(&self) -> &BriefSummary {
        &self.summary
    }

    /// Returns the source record refs evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }

    /// Reports whether the manager daily brief workflow satisfies the has source evidence safety condition.
    pub fn has_source_evidence(&self) -> bool {
        !self.source_record_refs.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for brief action kind in the manager daily brief workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum BriefActionKind {
    /// Selects review demand against staffing plan for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    ReviewDemandAgainstStaffingPlan,
    /// Selects resolve checkout exception for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    ResolveCheckoutException,
    /// Selects approve retention follow up draft for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    ApproveRetentionFollowUpDraft,
    /// Selects investigate source data quality issue for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    InvestigateSourceDataQualityIssue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for brief action priority in the manager daily brief workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum BriefActionPriority {
    /// Selects high for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    High,
    /// Selects medium for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    Medium,
    /// Selects low for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Review-safe agent tasks allowed to save staff time without crossing mutation or send gates.
pub enum SafeAgentAction {
    /// Allows agents to summarize source evidence for staff review without mutating records or contacting customers.
    SummarizeSourceEvidence,
    /// Allows agents to rank manager actions for staff review without mutating records or contacting customers.
    RankManagerActions,
    /// Allows agents to draft internal task for review for staff review without mutating records or contacting customers.
    DraftInternalTaskForReview,
    /// Allows agents to record manager feedback for staff review without mutating records or contacting customers.
    RecordManagerFeedback,
    /// Allows agents to estimate labor minutes saved for staff review without mutating records or contacting customers.
    EstimateLaborMinutesSaved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Actions the agent must never perform without a human/operator system of record.
pub enum BlockedAction {
    /// Blocks agents from change staff schedule until staff or the system of record performs the action.
    ChangeStaffSchedule,
    /// Blocks agents from mutate provider or pms record until staff or the system of record performs the action.
    MutateProviderOrPmsRecord,
    /// Blocks agents from send customer message until staff or the system of record performs the action.
    SendCustomerMessage,
    /// Blocks agents from move refund discount or payment until staff or the system of record performs the action.
    MoveRefundDiscountOrPayment,
    /// Blocks agents from hide source data quality issue until staff or the system of record performs the action.
    HideSourceDataQualityIssue,
}

impl BlockedAction {
    /// Returns the code evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn code(self) -> &'static str {
        match self {
            Self::ChangeStaffSchedule => "change_staff_schedule",
            Self::MutateProviderOrPmsRecord => "mutate_provider_or_pms_record",
            Self::SendCustomerMessage => "send_customer_message",
            Self::MoveRefundDiscountOrPayment => "move_refund_discount_or_payment",
            Self::HideSourceDataQualityIssue => "hide_source_data_quality_issue",
        }
    }

    /// Builds the from requested side effect code result for the manager daily brief workflow from reviewed source facts while preserving human review gates and draft-only side effects.
    pub fn from_requested_side_effect_code(code: &str) -> Option<Self> {
        match code {
            "change_staff_schedule" => Some(Self::ChangeStaffSchedule),
            "mutate_provider_or_pms_record" => Some(Self::MutateProviderOrPmsRecord),
            "send_customer_message" => Some(Self::SendCustomerMessage),
            "move_refund_discount_or_payment" => Some(Self::MoveRefundDiscountOrPayment),
            "hide_source_data_quality_issue" => Some(Self::HideSourceDataQualityIssue),
            _ => None,
        }
    }
}

/// Produces the requested side effect rejection reason rules for the manager daily brief workflow.
pub fn requested_side_effect_rejection_reason(side_effect: &str) -> String {
    if BlockedAction::from_requested_side_effect_code(side_effect).is_some() {
        format!("blocked_side_effect:{side_effect}")
    } else {
        format!("unsupported_side_effect:{side_effect}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Labor impact estimate used by the manager daily brief workflow; it assembles reviewable manager brief packets from deterministic context and agent drafts.
pub struct LaborImpactEstimate {
    before_minutes: LaborMinutes,
    after_minutes: LaborMinutes,
}

impl LaborImpactEstimate {
    /// Stores the reviewed value for the manager daily brief workflow without triggering provider, customer, payment, or schedule side effects.
    pub const fn new(before_minutes: LaborMinutes, after_minutes: LaborMinutes) -> Self {
        Self {
            before_minutes,
            after_minutes,
        }
    }

    /// Returns the before minutes evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn before_minutes(&self) -> LaborMinutes {
        self.before_minutes
    }

    /// Returns the after minutes evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn after_minutes(&self) -> LaborMinutes {
        self.after_minutes
    }

    /// Returns the minutes saved evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn minutes_saved(&self) -> u16 {
        self.before_minutes.0.saturating_sub(self.after_minutes.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Brief action used by the manager daily brief workflow; it assembles reviewable manager brief packets from deterministic context and agent drafts.
pub struct BriefAction {
    id: ActionId,
    kind: BriefActionKind,
    priority: BriefActionPriority,
    owner_persona: ManagerBriefPersona,
    removed_manual_work: RemovedManualWork,
    rationale: ActionRationale,
    source_facts: Vec<SourceFact>,
    labor_impact: LaborImpactEstimate,
    #[builder(default)]
    required_review_gates: Vec<policy::ReviewGate>,
}

impl BriefAction {
    /// Returns the id evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn id(&self) -> &ActionId {
        &self.id
    }

    /// Returns the kind evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn kind(&self) -> BriefActionKind {
        self.kind
    }

    /// Returns the priority evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn priority(&self) -> BriefActionPriority {
        self.priority
    }

    /// Returns the owner persona evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn owner_persona(&self) -> ManagerBriefPersona {
        self.owner_persona
    }

    /// Returns the removed manual work evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn removed_manual_work(&self) -> RemovedManualWork {
        self.removed_manual_work
    }

    /// Returns the rationale evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn rationale(&self) -> &ActionRationale {
        &self.rationale
    }

    /// Returns the source facts evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn source_facts(&self) -> &[SourceFact] {
        &self.source_facts
    }

    /// Returns the labor impact evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn labor_impact(&self) -> &LaborImpactEstimate {
        &self.labor_impact
    }

    /// Returns the required review gates evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }

    /// Reports whether the manager daily brief workflow satisfies the is source grounded safety condition.
    pub fn is_source_grounded(&self) -> bool {
        !self.source_facts.is_empty()
            && self
                .source_facts
                .iter()
                .all(SourceFact::has_source_evidence)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Scoped checkout packet used by the manager daily brief workflow; it assembles reviewable manager brief packets from deterministic context and agent drafts.
pub struct ScopedCheckoutPacket {
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    packet: checkout_completion::Packet,
}

impl ScopedCheckoutPacket {
    /// Returns the location id evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    /// Returns the operating day evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn operating_day(&self) -> operations::operating_day::Date {
        self.operating_day
    }

    /// Returns the packet evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn packet(&self) -> &checkout_completion::Packet {
        &self.packet
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Scoped retention packet used by the manager daily brief workflow; it assembles reviewable manager brief packets from deterministic context and agent drafts.
pub struct ScopedRetentionPacket {
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    packet: crm_retention::Packet,
}

impl ScopedRetentionPacket {
    /// Returns the location id evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    /// Returns the operating day evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn operating_day(&self) -> operations::operating_day::Date {
        self.operating_day
    }

    /// Returns the packet evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn packet(&self) -> &crm_retention::Packet {
        &self.packet
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Input rules for building the workflow packet from source-grounded records.
pub struct Request {
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    prepared_for: ManagerBriefPersona,
    demand_attention_threshold: DemandThresholdUnits,
    #[builder(default)]
    service_demand_facts: Vec<analytics::service_demand::Fact>,
    #[builder(default)]
    checkout_packets: Vec<ScopedCheckoutPacket>,
    #[builder(default)]
    retention_packets: Vec<ScopedRetentionPacket>,
}

impl Request {
    /// Returns the location id evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    /// Returns the operating day evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn operating_day(&self) -> operations::operating_day::Date {
        self.operating_day
    }

    /// Returns the prepared for evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn prepared_for(&self) -> ManagerBriefPersona {
        self.prepared_for
    }

    /// Returns the demand attention threshold evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn demand_attention_threshold(&self) -> DemandThresholdUnits {
        self.demand_attention_threshold
    }

    /// Returns the service demand facts evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn service_demand_facts(&self) -> &[analytics::service_demand::Fact] {
        &self.service_demand_facts
    }

    /// Returns the checkout packets evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn checkout_packets(&self) -> &[ScopedCheckoutPacket] {
        &self.checkout_packets
    }

    /// Returns the retention packets evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn retention_packets(&self) -> &[ScopedRetentionPacket] {
        &self.retention_packets
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Reviewable packet handed to staff or agents with deterministic gates already applied.
pub struct Packet {
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    prepared_for: ManagerBriefPersona,
    actions: Vec<BriefAction>,
    safe_agent_actions: Vec<SafeAgentAction>,
    blocked_actions: Vec<BlockedAction>,
    before_minutes: AggregateLaborMinutes,
    after_minutes: AggregateLaborMinutes,
}

impl Packet {
    /// Returns the location id evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    /// Returns the operating day evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn operating_day(&self) -> operations::operating_day::Date {
        self.operating_day
    }

    /// Returns the prepared for evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn prepared_for(&self) -> ManagerBriefPersona {
        self.prepared_for
    }

    /// Returns the actions evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn actions(&self) -> &[BriefAction] {
        &self.actions
    }

    /// Returns the safe agent actions evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn safe_agent_actions(&self) -> &[SafeAgentAction] {
        &self.safe_agent_actions
    }

    /// Returns the blocked actions evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn blocked_actions(&self) -> &[BlockedAction] {
        &self.blocked_actions
    }

    /// Returns the before minutes evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn before_minutes(&self) -> AggregateLaborMinutes {
        self.before_minutes
    }

    /// Returns the after minutes evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn after_minutes(&self) -> AggregateLaborMinutes {
        self.after_minutes
    }

    /// Returns the minutes saved evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn minutes_saved(&self) -> u16 {
        self.before_minutes.0.saturating_sub(self.after_minutes.0)
    }

    /// Returns the all actions are source grounded evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn all_actions_are_source_grounded(&self) -> bool {
        self.actions.iter().all(BriefAction::is_source_grounded)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for feedback outcome in the manager daily brief workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum FeedbackOutcome {
    /// Records a completed result so follow-up impact is auditable.
    Completed,
    /// Records a deferred result so follow-up impact is auditable.
    Deferred,
    /// Records a suppressed by manager result so follow-up impact is auditable.
    SuppressedByManager,
    /// Records a source fact was wrong result so follow-up impact is auditable.
    SourceFactWasWrong,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Outcome record used by the manager daily brief workflow; it assembles reviewable manager brief packets from deterministic context and agent drafts.
pub struct OutcomeRecord {
    action_id: ActionId,
    recorded_by: entities::ActorRef,
    outcome: FeedbackOutcome,
    before_minutes: LaborMinutes,
    actual_minutes: LaborMinutes,
    #[builder(default)]
    source_record_refs: Vec<source::RecordRef>,
}

impl OutcomeRecord {
    /// Returns the action id evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn action_id(&self) -> &ActionId {
        &self.action_id
    }

    /// Returns the recorded by evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn recorded_by(&self) -> &entities::ActorRef {
        &self.recorded_by
    }

    /// Returns the outcome evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn outcome(&self) -> FeedbackOutcome {
        self.outcome
    }

    /// Returns the before minutes evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn before_minutes(&self) -> LaborMinutes {
        self.before_minutes
    }

    /// Returns the actual minutes evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn actual_minutes(&self) -> LaborMinutes {
        self.actual_minutes
    }

    /// Returns the actual minutes saved evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn actual_minutes_saved(&self) -> u16 {
        self.before_minutes.0.saturating_sub(self.actual_minutes.0)
    }

    /// Returns the source record refs evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }

    /// Returns the records feedback without external mutation evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn records_feedback_without_external_mutation(&self) -> bool {
        true
    }

    /// Returns the blocked actions evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn blocked_actions(&self) -> Vec<BlockedAction> {
        blocked_actions_for()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Decision choices for error in the manager daily brief workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum Error {
    #[error("labor minutes must be greater than zero")]
    /// Identifies zero labor minutes as the reason the workflow must stop, retry, or request review.
    ZeroLaborMinutes,
    #[error("demand threshold units must be greater than zero")]
    /// Identifies zero demand threshold units as the reason the workflow must stop, retry, or request review.
    ZeroDemandThresholdUnits,
}

/// Result type returned by fallible manager daily brief operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Workflow used by the manager daily brief workflow; it assembles reviewable manager brief packets from deterministic context and agent drafts.
pub struct Workflow;

impl Workflow {
    /// Builds the evaluate result for the manager daily brief workflow from reviewed source facts while preserving human review gates and draft-only side effects.
    pub fn evaluate(request: Request) -> Packet {
        let mut actions = Vec::new();
        actions.extend(service_demand_actions(&request));
        actions.extend(checkout_exception_actions(&request));
        actions.extend(retention_actions(&request));

        let before_minutes = total_before_minutes(&actions);
        let after_minutes = total_after_minutes(&actions);

        Packet {
            location_id: request.location_id,
            operating_day: request.operating_day,
            prepared_for: request.prepared_for,
            actions,
            safe_agent_actions: vec![
                SafeAgentAction::SummarizeSourceEvidence,
                SafeAgentAction::RankManagerActions,
                SafeAgentAction::DraftInternalTaskForReview,
                SafeAgentAction::RecordManagerFeedback,
                SafeAgentAction::EstimateLaborMinutesSaved,
            ],
            blocked_actions: blocked_actions_for(),
            before_minutes,
            after_minutes,
        }
    }
}

fn service_demand_actions(request: &Request) -> Vec<BriefAction> {
    request
        .service_demand_facts
        .iter()
        .filter(|fact| service_demand_fact_matches_request_scope(fact, request))
        .filter(|fact| fact.demand_units().get() >= request.demand_attention_threshold.get())
        .map(|fact| {
            let mut source_facts = vec![SourceFact::builder()
                .kind(SourceFactKind::ServiceDemandForecast)
                .summary(BriefSummary::try_new("Service demand crosses the manager attention threshold for this operating day.").expect("static brief summary is valid"))
                .source_record_refs(fact.source_record_refs().to_vec())
                .build()];

            let mut required_review_gates = Vec::new();
            if matches!(
                fact.data_quality_status(),
                analytics::service_demand::DataQualityStatus::ManagerReviewRequired
            ) {
                source_facts.push(SourceFact::builder()
                    .kind(SourceFactKind::SourceDataQualityIssue)
                    .summary(BriefSummary::try_new("Demand fact carries nonblocking source data-quality issues that should stay visible in the brief.").expect("static brief summary is valid"))
                    .source_record_refs(fact.source_record_refs().to_vec())
                    .build());
                required_review_gates.push(policy::ReviewGate::ManagerApproval);
            }

            BriefAction::builder()
                .id(ActionId::try_new(format!(
                    "demand-staffing-{}",
                    fact.id().as_str()
                ))
                .expect("fact ids are non-empty"))
                .kind(BriefActionKind::ReviewDemandAgainstStaffingPlan)
                .priority(BriefActionPriority::High)
                .owner_persona(ManagerBriefPersona::GeneralManager)
                .removed_manual_work(RemovedManualWork::DemandVersusStaffingScan)
                .rationale(ActionRationale::try_new("Manager starts from a ranked source-grounded staffing risk instead of manually comparing reservation dashboards to the schedule.").expect("static rationale is valid"))
                .source_facts(source_facts)
                .labor_impact(LaborImpactEstimate::new(
                    LaborMinutes::try_new(45).expect("static minutes are valid"),
                    LaborMinutes::try_new(15).expect("static minutes are valid"),
                ))
                .required_review_gates(required_review_gates)
                .build()
        })
        .collect()
}

fn service_demand_fact_matches_request_scope(
    fact: &analytics::service_demand::Fact,
    request: &Request,
) -> bool {
    scoped_packet_matches_request_scope(
        fact.operating_day().location_id(),
        fact.operating_day().date(),
        request,
    )
}

fn scoped_packet_matches_request_scope(
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    request: &Request,
) -> bool {
    location_id == request.location_id && operating_day == request.operating_day
}

fn checkout_exception_actions(request: &Request) -> Vec<BriefAction> {
    request
        .checkout_packets
        .iter()
        .filter(|scoped| scoped_packet_matches_request_scope(scoped.location_id(), scoped.operating_day(), request))
        .map(ScopedCheckoutPacket::packet)
        .filter(|packet| {
            !matches!(
                packet.completion_status(),
                checkout_completion::CompletionStatus::StaffVerifiedCheckout
            )
        })
        .map(|packet| {
            BriefAction::builder()
                .id(ActionId::try_new(format!(
                    "checkout-exception-{:?}",
                    packet.reservation_id()
                ))
                .expect("formatted reservation ids are non-empty"))
                .kind(BriefActionKind::ResolveCheckoutException)
                .priority(BriefActionPriority::High)
                .owner_persona(ManagerBriefPersona::FrontDeskLead)
                .removed_manual_work(RemovedManualWork::CheckoutExceptionAudit)
                .rationale(ActionRationale::try_new("Front desk lead receives the unresolved checkout handoff instead of auditing open reservations one by one.").expect("static rationale is valid"))
                .source_facts(vec![SourceFact::builder()
                    .kind(SourceFactKind::CheckoutCompletionStatus)
                    .summary(BriefSummary::try_new("Checkout/completion contract says this stay still needs staff or manager review.").expect("static brief summary is valid"))
                    .source_record_refs(vec![source::RecordRef::from_provenance(packet.provenance())])
                    .build()])
                .labor_impact(LaborImpactEstimate::new(
                    LaborMinutes::try_new(20).expect("static minutes are valid"),
                    LaborMinutes::try_new(8).expect("static minutes are valid"),
                ))
                .required_review_gates(packet.required_review_gates().to_vec())
                .build()
        })
        .collect()
}

fn retention_actions(request: &Request) -> Vec<BriefAction> {
    request
        .retention_packets
        .iter()
        .filter(|scoped| scoped_packet_matches_request_scope(scoped.location_id(), scoped.operating_day(), request))
        .map(ScopedRetentionPacket::packet)
        .filter(|packet| {
            matches!(
                packet.eligibility(),
                crm_retention::FollowUpEligibility::Eligible { .. }
            )
        })
        .map(|packet| {
            let source_record_refs = packet
                .review_packet()
                .staff_evidence()
                .iter()
                .map(|evidence| source::RecordRef::from_provenance(evidence.provenance()))
                .chain(packet.source_record_refs().iter().cloned())
                .collect::<Vec<_>>();

            BriefAction::builder()
                .id(ActionId::try_new(format!(
                    "retention-follow-up-{:?}",
                    packet.reservation_id()
                ))
                .expect("formatted reservation ids are non-empty"))
                .kind(BriefActionKind::ApproveRetentionFollowUpDraft)
                .priority(BriefActionPriority::Medium)
                .owner_persona(ManagerBriefPersona::FrontDeskLead)
                .removed_manual_work(RemovedManualWork::RetentionFollowUpQueuePrioritization)
                .rationale(ActionRationale::try_new("Front desk lead receives eligible source-grounded retention opportunities instead of manually scanning completed stays for follow-up candidates.").expect("static rationale is valid"))
                .source_facts(vec![SourceFact::builder()
                    .kind(SourceFactKind::RetentionFollowUpEligibility)
                    .summary(BriefSummary::try_new("CRM/retention contract says this stay has an eligible draft-only follow-up opportunity.").expect("static brief summary is valid"))
                    .source_record_refs(source_record_refs)
                    .build()])
                .labor_impact(LaborImpactEstimate::new(
                    LaborMinutes::try_new(30).expect("static minutes are valid"),
                    LaborMinutes::try_new(10).expect("static minutes are valid"),
                ))
                .required_review_gates(packet.required_review_gates().to_vec())
                .build()
        })
        .collect()
}

fn total_before_minutes(actions: &[BriefAction]) -> AggregateLaborMinutes {
    AggregateLaborMinutes::new(
        actions
            .iter()
            .map(|action| action.labor_impact.before_minutes.get())
            .sum::<u16>(),
    )
}

fn total_after_minutes(actions: &[BriefAction]) -> AggregateLaborMinutes {
    AggregateLaborMinutes::new(
        actions
            .iter()
            .map(|action| action.labor_impact.after_minutes.get())
            .sum::<u16>(),
    )
}

fn blocked_actions_for() -> Vec<BlockedAction> {
    vec![
        BlockedAction::ChangeStaffSchedule,
        BlockedAction::MutateProviderOrPmsRecord,
        BlockedAction::SendCustomerMessage,
        BlockedAction::MoveRefundDiscountOrPayment,
        BlockedAction::HideSourceDataQualityIssue,
    ]
}
