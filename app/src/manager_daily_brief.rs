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
pub struct LaborMinutes(u16);

impl LaborMinutes {
    pub const fn try_new(value: u16) -> Result<Self> {
        if value == 0 {
            return Err(Error::ZeroLaborMinutes);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AggregateLaborMinutes(u16);

impl AggregateLaborMinutes {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DemandThresholdUnits(u32);

impl DemandThresholdUnits {
    pub const fn try_new(value: u32) -> Result<Self> {
        if value == 0 {
            return Err(Error::ZeroDemandThresholdUnits);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ManagerBriefPersona {
    GeneralManager,
    AssistantGeneralManager,
    FrontDeskLead,
    FrontDeskAgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RemovedManualWork {
    MorningDashboardReconciliation,
    DemandVersusStaffingScan,
    CheckoutExceptionAudit,
    RetentionFollowUpQueuePrioritization,
    DataQualityExceptionTriage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SourceFactKind {
    ServiceDemandForecast,
    CheckoutCompletionStatus,
    RetentionFollowUpEligibility,
    SourceDataQualityIssue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct SourceFact {
    kind: SourceFactKind,
    summary: BriefSummary,
    #[builder(default)]
    source_record_refs: Vec<source::RecordRef>,
}

impl SourceFact {
    pub const fn kind(&self) -> SourceFactKind {
        self.kind
    }

    pub const fn summary(&self) -> &BriefSummary {
        &self.summary
    }

    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }

    pub fn has_source_evidence(&self) -> bool {
        !self.source_record_refs.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BriefActionKind {
    ReviewDemandAgainstStaffingPlan,
    ResolveCheckoutException,
    ApproveRetentionFollowUpDraft,
    InvestigateSourceDataQualityIssue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BriefActionPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SafeAgentAction {
    SummarizeSourceEvidence,
    RankManagerActions,
    DraftInternalTaskForReview,
    RecordManagerFeedback,
    EstimateLaborMinutesSaved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BlockedAction {
    ChangeStaffSchedule,
    MutateProviderOrPmsRecord,
    SendCustomerMessage,
    MoveRefundDiscountOrPayment,
    HideSourceDataQualityIssue,
}

impl BlockedAction {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ChangeStaffSchedule => "change_staff_schedule",
            Self::MutateProviderOrPmsRecord => "mutate_provider_or_pms_record",
            Self::SendCustomerMessage => "send_customer_message",
            Self::MoveRefundDiscountOrPayment => "move_refund_discount_or_payment",
            Self::HideSourceDataQualityIssue => "hide_source_data_quality_issue",
        }
    }

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

pub fn requested_side_effect_rejection_reason(side_effect: &str) -> String {
    if BlockedAction::from_requested_side_effect_code(side_effect).is_some() {
        format!("blocked_side_effect:{side_effect}")
    } else {
        format!("unsupported_side_effect:{side_effect}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaborImpactEstimate {
    before_minutes: LaborMinutes,
    after_minutes: LaborMinutes,
}

impl LaborImpactEstimate {
    pub const fn new(before_minutes: LaborMinutes, after_minutes: LaborMinutes) -> Self {
        Self {
            before_minutes,
            after_minutes,
        }
    }

    pub const fn before_minutes(&self) -> LaborMinutes {
        self.before_minutes
    }

    pub const fn after_minutes(&self) -> LaborMinutes {
        self.after_minutes
    }

    pub const fn minutes_saved(&self) -> u16 {
        self.before_minutes.0.saturating_sub(self.after_minutes.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
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
    pub const fn id(&self) -> &ActionId {
        &self.id
    }

    pub const fn kind(&self) -> BriefActionKind {
        self.kind
    }

    pub const fn priority(&self) -> BriefActionPriority {
        self.priority
    }

    pub const fn owner_persona(&self) -> ManagerBriefPersona {
        self.owner_persona
    }

    pub const fn removed_manual_work(&self) -> RemovedManualWork {
        self.removed_manual_work
    }

    pub const fn rationale(&self) -> &ActionRationale {
        &self.rationale
    }

    pub fn source_facts(&self) -> &[SourceFact] {
        &self.source_facts
    }

    pub const fn labor_impact(&self) -> &LaborImpactEstimate {
        &self.labor_impact
    }

    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }

    pub fn is_source_grounded(&self) -> bool {
        !self.source_facts.is_empty()
            && self
                .source_facts
                .iter()
                .all(SourceFact::has_source_evidence)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct ScopedCheckoutPacket {
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    packet: checkout_completion::Packet,
}

impl ScopedCheckoutPacket {
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    pub const fn operating_day(&self) -> operations::operating_day::Date {
        self.operating_day
    }

    pub const fn packet(&self) -> &checkout_completion::Packet {
        &self.packet
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct ScopedRetentionPacket {
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    packet: crm_retention::Packet,
}

impl ScopedRetentionPacket {
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    pub const fn operating_day(&self) -> operations::operating_day::Date {
        self.operating_day
    }

    pub const fn packet(&self) -> &crm_retention::Packet {
        &self.packet
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
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
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    pub const fn operating_day(&self) -> operations::operating_day::Date {
        self.operating_day
    }

    pub const fn prepared_for(&self) -> ManagerBriefPersona {
        self.prepared_for
    }

    pub const fn demand_attention_threshold(&self) -> DemandThresholdUnits {
        self.demand_attention_threshold
    }

    pub fn service_demand_facts(&self) -> &[analytics::service_demand::Fact] {
        &self.service_demand_facts
    }

    pub fn checkout_packets(&self) -> &[ScopedCheckoutPacket] {
        &self.checkout_packets
    }

    pub fn retention_packets(&self) -> &[ScopedRetentionPacket] {
        &self.retention_packets
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    pub const fn operating_day(&self) -> operations::operating_day::Date {
        self.operating_day
    }

    pub const fn prepared_for(&self) -> ManagerBriefPersona {
        self.prepared_for
    }

    pub fn actions(&self) -> &[BriefAction] {
        &self.actions
    }

    pub fn safe_agent_actions(&self) -> &[SafeAgentAction] {
        &self.safe_agent_actions
    }

    pub fn blocked_actions(&self) -> &[BlockedAction] {
        &self.blocked_actions
    }

    pub const fn before_minutes(&self) -> AggregateLaborMinutes {
        self.before_minutes
    }

    pub const fn after_minutes(&self) -> AggregateLaborMinutes {
        self.after_minutes
    }

    pub const fn minutes_saved(&self) -> u16 {
        self.before_minutes.0.saturating_sub(self.after_minutes.0)
    }

    pub fn all_actions_are_source_grounded(&self) -> bool {
        self.actions.iter().all(BriefAction::is_source_grounded)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FeedbackOutcome {
    Completed,
    Deferred,
    SuppressedByManager,
    SourceFactWasWrong,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
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
    pub const fn action_id(&self) -> &ActionId {
        &self.action_id
    }

    pub const fn recorded_by(&self) -> &entities::ActorRef {
        &self.recorded_by
    }

    pub const fn outcome(&self) -> FeedbackOutcome {
        self.outcome
    }

    pub const fn before_minutes(&self) -> LaborMinutes {
        self.before_minutes
    }

    pub const fn actual_minutes(&self) -> LaborMinutes {
        self.actual_minutes
    }

    pub const fn actual_minutes_saved(&self) -> u16 {
        self.before_minutes.0.saturating_sub(self.actual_minutes.0)
    }

    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }

    pub fn records_feedback_without_external_mutation(&self) -> bool {
        true
    }

    pub fn blocked_actions(&self) -> Vec<BlockedAction> {
        blocked_actions_for()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("labor minutes must be greater than zero")]
    ZeroLaborMinutes,
    #[error("demand threshold units must be greater than zero")]
    ZeroDemandThresholdUnits,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Workflow;

impl Workflow {
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
