use super::*;

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 1200),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct BriefSummary(String);

impl fmt::Debug for BriefSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BriefSummary(<redacted>)")
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct ActionId(String);

impl fmt::Debug for ActionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ActionId([REDACTED])")
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 500),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct ActionRationale(String);

impl fmt::Debug for ActionRationale {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ActionRationale(<redacted>)")
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 500),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
/// Caller-reported manager or front-desk feedback retained as nonclaimable evidence.
///
/// The text authenticates no actor and proves no review, action, completion, source-of-record
/// disposition, measured labor, or realized value. It never grants authority to change schedules,
/// provider records, customer messages, payments, refunds, discounts, or source data.
pub struct ManagerFeedback(String);

impl fmt::Debug for ManagerFeedback {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ManagerFeedback(<redacted>)")
    }
}

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
    /// Retains a caller-reported numeric label without authenticating review, measurement, action, completion, labor reduction, or value.
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
/// Caller-selected manager-brief persona label used for reviewable presentation; it authenticates no person and creates no queue, draft, gate, action, or completion authority.
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
/// Reported categories of manual work associated with manager-brief evidence; serialized labels do not create workflow authority.
pub enum RemovedManualWork {
    /// Selects morning dashboard reconciliation for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    MorningDashboardReconciliation,
    /// Selects demand versus staffing scan for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    DemandVersusStaffingScan,
    /// Selects checkout exception audit for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    CheckoutExceptionAudit,

    /// Selects data quality exception triage for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    DataQualityExceptionTriage,
    /// Selects service capacity labor planning for reviewable staffing/capacity recommendations without mutating schedules.
    ServiceCapacityLaborPlanning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Source-fact labels carried by manager-brief evidence; labels alone cannot create an action, queue, task, draft, completion, or value claim.
pub enum SourceFactKind {
    /// Selects service demand forecast for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    ServiceDemandForecast,
    /// Selects checkout completion status for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    CheckoutCompletionStatus,

    /// Selects source data quality issue for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    SourceDataQualityIssue,
    /// Selects capacity labor recommendation for the manager brief decision model so staffing/capacity evidence stays reviewed and source-cited.
    CapacityLaborRecommendation,
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
/// Manager-brief action labels; caller-created or serialized labels do not prove that the workflow emitted or completed an action.
pub enum BriefActionKind {
    /// Selects review demand against staffing plan for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    ReviewDemandAgainstStaffingPlan,
    /// Selects resolve checkout exception for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    ResolveCheckoutException,

    /// Selects investigate source data quality issue for the manager brief decision model so the app can choose a review, evidence, or draft path without taking live action.
    InvestigateSourceDataQualityIssue,
    /// Selects review capacity labor recommendation for the manager brief decision model without schedule mutation authority.
    ReviewCapacityLaborRecommendation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Caller-visible priority label for reviewable manager-brief evidence; it creates no queue, draft, gate, action, or completion authority.
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
    /// Allows agents to report a labor estimate difference for staff review without claiming realized savings or enabling side effects.
    ReportLaborEstimateDifference,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[strum(serialize_all = "snake_case")]
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
    /// Parses a requested-side-effect label into blocked-action evidence without asserting reviewed source facts or creating a queue, draft, gate, action, or completion authority.
    pub fn from_requested_side_effect_code(code: &str) -> Option<Self> {
        code.parse().ok()
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
    /// Retains a caller-reported numeric label without authenticating review, measurement, action, completion, labor reduction, or value.
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

    /// Returns a caller-reported estimate difference for prioritization, never a realized-savings claim.
    pub const fn reported_estimated_minutes_difference(&self) -> u16 {
        self.before_minutes.0.saturating_sub(self.after_minutes.0)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
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
    capacity_labor_recommendation: Option<operations::capacity::OptimizationRecommendation>,
    #[builder(default)]
    required_review_gates: Vec<policy::ReviewGate>,
}

impl fmt::Debug for BriefAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BriefAction([REDACTED])")
    }
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

    /// Returns the relationship-checked capacity/labor recommendation when this action came from service capacity evidence.
    pub const fn capacity_labor_recommendation(
        &self,
    ) -> Option<&operations::capacity::OptimizationRecommendation> {
        self.capacity_labor_recommendation.as_ref()
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

#[derive(Clone, PartialEq, Eq, Serialize, bon::Builder)]
/// Scoped checkout packet used by the manager daily brief workflow; it assembles reviewable manager brief packets from deterministic context and agent drafts.
pub struct ScopedCheckoutPacket {
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    packet: checkout_completion::ReviewPacket,
}

impl fmt::Debug for ScopedCheckoutPacket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ScopedCheckoutPacket([REDACTED])")
    }
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
    pub const fn packet(&self) -> &checkout_completion::ReviewPacket {
        &self.packet
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, bon::Builder)]
/// Scoped retention packet used by the manager daily brief workflow; it assembles reviewable manager brief packets from deterministic context and agent drafts.
pub struct ScopedRetentionPacket {
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    packet: crm_retention::Packet,
}

impl fmt::Debug for ScopedRetentionPacket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ScopedRetentionPacket([REDACTED])")
    }
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

#[derive(Clone, PartialEq, Eq, Serialize, bon::Builder)]
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
    #[builder(default)]
    capacity_labor_recommendations: Vec<operations::capacity::OptimizationRecommendation>,
}

impl fmt::Debug for Request {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Request([REDACTED])")
    }
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

    /// Returns source-grounded capacity/labor recommendations already validated by domain relationship checks.
    pub fn capacity_labor_recommendations(
        &self,
    ) -> &[operations::capacity::OptimizationRecommendation] {
        &self.capacity_labor_recommendations
    }
}

#[derive(Clone, PartialEq, Eq, Serialize)]
/// Server-issued review packet with deterministic gates already applied.
///
/// It is serializable for presentation but intentionally not caller-deserializable; callers must
/// submit source evidence through [`Request`] so the workflow can issue ranked actions.
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

impl fmt::Debug for Packet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Packet([REDACTED])")
    }
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

    /// Returns a caller-reported estimate difference for prioritization, never a realized-savings claim.
    pub const fn reported_estimated_minutes_difference(&self) -> u16 {
        self.before_minutes.0.saturating_sub(self.after_minutes.0)
    }

    /// Returns the all actions are source grounded evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn all_actions_are_source_grounded(&self) -> bool {
        self.actions.iter().all(BriefAction::is_source_grounded)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Fail-closed validation error for manager-brief evidence; it creates no queue, draft, gate, action, review, or completion authority.
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
    /// Builds a reviewable manager-daily-brief packet from deterministic caller/source evidence while preserving review gates and draft-only side effects. Source correlation authenticates no review, action, completion, measurement, or value.
    pub fn evaluate(request: Request) -> Packet {
        let mut actions = Vec::new();
        actions.extend(service_demand_actions(&request));
        actions.extend(capacity_labor_recommendation_actions(&request));
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
                SafeAgentAction::ReportLaborEstimateDifference,
            ],
            blocked_actions: blocked_actions_for(),
            before_minutes,
            after_minutes,
        }
    }
}

fn capacity_labor_recommendation_actions(request: &Request) -> Vec<BriefAction> {
    request
        .capacity_labor_recommendations
        .iter()
        .filter(|recommendation| {
            scoped_packet_matches_request_scope(
                recommendation.demand().location_id(),
                request.operating_day,
                request,
            )
        })
        .map(|recommendation| {
            let source_record_refs = recommendation
                .source_evidence()
                .iter()
                .map(source::RecordRef::from_provenance)
                .collect::<Vec<_>>();
            let expected_delta = recommendation.expected_labor_delta_minutes().get().unsigned_abs();
            let before_minutes = expected_delta.saturating_sub(22).clamp(1, u32::from(u16::MAX));
            let before_minutes = u16::try_from(before_minutes).expect("clamped to u16");

            BriefAction::builder()
                .id(ActionId::try_new(format!(
                    "capacity-labor-{:?}-{:?}",
                    recommendation.demand().service(),
                    recommendation.coverage().role()
                ))
                .expect("formatted capacity labor action ids are non-empty"))
                .kind(BriefActionKind::ReviewCapacityLaborRecommendation)
                .priority(BriefActionPriority::High)
                .owner_persona(ManagerBriefPersona::GeneralManager)
                .removed_manual_work(RemovedManualWork::ServiceCapacityLaborPlanning)
                .rationale(ActionRationale::try_new("Manager receives a source-cited capacity/labor recommendation with alternatives and feasibility already checked, while the agent remains unable to mutate the schedule.").expect("static rationale is valid"))
                .source_facts(vec![SourceFact::builder()
                    .kind(SourceFactKind::CapacityLaborRecommendation)
                    .summary(BriefSummary::try_new("Capacity, forecast-demand, and scheduled-coverage evidence produced a reviewable labor-recommendation label; no manager review, schedule action, or labor effect is proven.").expect("static brief summary is valid"))
                    .source_record_refs(source_record_refs)
                    .build()])
                .labor_impact(LaborImpactEstimate::new(
                    LaborMinutes::try_new(before_minutes).expect("derived nonzero minutes are valid"),
                    LaborMinutes::try_new(18).expect("static minutes are valid"),
                ))
                .capacity_labor_recommendation(recommendation.clone())
                .required_review_gates(vec![recommendation.review_gate()])
                .build()
        })
        .collect()
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
    let _reported_retention_evidence = &request.retention_packets;
    Vec::new()
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

pub(super) fn blocked_actions_for() -> Vec<BlockedAction> {
    vec![
        BlockedAction::ChangeStaffSchedule,
        BlockedAction::MutateProviderOrPmsRecord,
        BlockedAction::SendCustomerMessage,
        BlockedAction::MoveRefundDiscountOrPayment,
        BlockedAction::HideSourceDataQualityIssue,
    ]
}
