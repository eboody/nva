use serde::{Deserialize, Serialize};

use domain::{data_quality, entities, operations, policy, source};

pub const WORKFLOW_NAME: &str = "data-quality-hygiene";
pub const SCHEMA_VERSION: &str = "data-quality-hygiene-context-v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IssueRef(String);

impl IssueRef {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyIssueRef).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ActionId(String);

impl ActionId {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyActionId).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ContextPacketId(String);

impl ContextPacketId {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyContextPacketId).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CorrelationId(String);

impl CorrelationId {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyCorrelationId).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ActionRationale(String);

impl ActionRationale {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyActionRationale).map(Self)
    }
}

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
pub enum HygienePersona {
    GeneralManager,
    AssistantGeneralManager,
    FrontDeskLead,
    FrontDeskAgent,
    RegionalOperator,
    OperationsAnalyst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CandidateKind {
    SourceIssue,
    DuplicateCandidate,
    ProfileGap,
    ServiceLineMapping,
    SourceFreshness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SourceFreshness {
    Current,
    Stale,
    Conflicting,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Sensitivity {
    StandardOperationalEvidence,
    VaccineEvidence,
    IncidentOrBehaviorEvidence,
    PaymentEvidence,
    QuarantinedSensitivePayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct Candidate {
    id: IssueRef,
    kind: CandidateKind,
    issue: data_quality::Issue,
    #[builder(default)]
    source_record_refs: Vec<source::RecordRef>,
    source_freshness: SourceFreshness,
    sensitivity: Sensitivity,
}

impl Candidate {
    pub const fn id(&self) -> &IssueRef {
        &self.id
    }

    pub const fn kind(&self) -> CandidateKind {
        self.kind
    }

    pub const fn issue(&self) -> &data_quality::Issue {
        &self.issue
    }

    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }

    pub const fn source_freshness(&self) -> SourceFreshness {
        self.source_freshness
    }

    pub const fn sensitivity(&self) -> Sensitivity {
        self.sensitivity
    }

    fn effective_source_record_refs(&self) -> Vec<source::RecordRef> {
        let mut refs = self.source_record_refs.clone();
        let issue_ref = self.issue.source_record_ref().clone();
        if !refs.contains(&issue_ref) {
            refs.push(issue_ref);
        }
        refs
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ActionKind {
    InvestigateMissingSourceEvidence,
    ReconcileDuplicateCustomerOrPetCandidate,
    CompleteMissingPetOrCustomerProfileFields,
    ReviewStaleVaccinationSourceFreshness,
    NormalizeAmbiguousServiceLineNaming,
    ReviewCheckoutOrUnclosedReservationEvidence,
    EscalateSensitiveOrQuarantinedPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ActionPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RemovedManualWork {
    MissingEvidenceInvestigation,
    DuplicateCandidateReconciliation,
    IncompleteProfileCleanupPreparation,
    SourceFreshnessReview,
    ServiceLineNormalizationReview,
    CheckoutEvidenceReview,
    SensitivePayloadEscalation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SafeAgentAction {
    SummarizeSourceEvidence,
    RankHygieneActions,
    DraftInternalCleanupTask,
    PreserveAmbiguityForReview,
    EstimateReconciliationMinutesSaved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BlockedAction {
    SendCustomerMessage,
    MutateProviderOrPmsRecord,
    ChangeStaffSchedule,
    MoveRefundDiscountOrPayment,
    HideOrAutoResolveSourceAmbiguity,
    ExposeQuarantinedSensitivePayload,
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
pub struct Action {
    id: ActionId,
    kind: ActionKind,
    priority: ActionPriority,
    owner_persona: HygienePersona,
    removed_manual_work: RemovedManualWork,
    rationale: ActionRationale,
    #[builder(default)]
    source_record_refs: Vec<source::RecordRef>,
    #[builder(default)]
    issue_refs: Vec<IssueRef>,
    #[builder(default)]
    required_review_gates: Vec<policy::ReviewGate>,
    labor_impact: LaborImpactEstimate,
}

impl Action {
    pub const fn id(&self) -> &ActionId {
        &self.id
    }

    pub const fn kind(&self) -> ActionKind {
        self.kind
    }

    pub const fn priority(&self) -> ActionPriority {
        self.priority
    }

    pub const fn owner_persona(&self) -> HygienePersona {
        self.owner_persona
    }

    pub const fn removed_manual_work(&self) -> RemovedManualWork {
        self.removed_manual_work
    }

    pub const fn rationale(&self) -> &ActionRationale {
        &self.rationale
    }

    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }

    pub fn issue_refs(&self) -> &[IssueRef] {
        &self.issue_refs
    }

    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }

    pub const fn labor_impact(&self) -> &LaborImpactEstimate {
        &self.labor_impact
    }

    pub fn is_source_grounded(&self) -> bool {
        !self.source_record_refs.is_empty() && !self.issue_refs.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct Request {
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    prepared_for: HygienePersona,
    #[builder(default)]
    candidates: Vec<Candidate>,
}

impl Request {
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    pub const fn operating_day(&self) -> operations::operating_day::Date {
        self.operating_day
    }

    pub const fn prepared_for(&self) -> HygienePersona {
        self.prepared_for
    }

    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Packet {
    workflow: &'static str,
    schema_version: &'static str,
    context_packet_id: ContextPacketId,
    correlation_id: CorrelationId,
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    prepared_for: HygienePersona,
    candidates: Vec<Candidate>,
    actions: Vec<Action>,
    safe_agent_actions: Vec<SafeAgentAction>,
    blocked_actions: Vec<BlockedAction>,
    before_minutes: AggregateLaborMinutes,
    after_minutes: AggregateLaborMinutes,
}

impl Packet {
    pub const fn workflow(&self) -> &'static str {
        self.workflow
    }

    pub const fn schema_version(&self) -> &'static str {
        self.schema_version
    }

    pub const fn context_packet_id(&self) -> &ContextPacketId {
        &self.context_packet_id
    }

    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }

    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    pub const fn operating_day(&self) -> operations::operating_day::Date {
        self.operating_day
    }

    pub const fn prepared_for(&self) -> HygienePersona {
        self.prepared_for
    }

    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    pub fn actions(&self) -> &[Action] {
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
        self.actions.iter().all(Action::is_source_grounded)
    }

    pub fn validate_draft(&self, draft: &DraftSubmission) -> DraftValidation {
        let mut rejection_reasons = Vec::new();

        if draft.context_packet_id != self.context_packet_id
            || draft.correlation_id != self.correlation_id
        {
            rejection_reasons.push(DraftRejectionReason::StaleOrUnknownContextPacket);
        }

        for action in &draft.actions {
            validate_draft_action(self, action, &mut rejection_reasons);
        }

        rejection_reasons.dedup();
        DraftValidation { rejection_reasons }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftAction {
    action_id: ActionId,
    kind: ActionKind,
    source_record_refs: Vec<source::RecordRef>,
    issue_refs: Vec<IssueRef>,
    required_review_gates: Vec<policy::ReviewGate>,
    requested_side_effects: Vec<String>,
    attempted_ambiguity_resolution: bool,
}

impl DraftAction {
    pub fn from_action(action: Action) -> Self {
        Self {
            action_id: action.id,
            kind: action.kind,
            source_record_refs: action.source_record_refs,
            issue_refs: action.issue_refs,
            required_review_gates: action.required_review_gates,
            requested_side_effects: Vec::new(),
            attempted_ambiguity_resolution: false,
        }
    }

    pub fn with_requested_side_effect(mut self, side_effect: impl Into<String>) -> Self {
        self.requested_side_effects.push(side_effect.into());
        self
    }

    pub const fn with_attempted_ambiguity_resolution(mut self) -> Self {
        self.attempted_ambiguity_resolution = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct DraftSubmission {
    context_packet_id: ContextPacketId,
    correlation_id: CorrelationId,
    #[builder(default)]
    actions: Vec<DraftAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftValidation {
    rejection_reasons: Vec<DraftRejectionReason>,
}

impl DraftValidation {
    pub fn is_accepted(&self) -> bool {
        self.rejection_reasons.is_empty()
    }

    pub fn rejection_reasons(&self) -> &[DraftRejectionReason] {
        &self.rejection_reasons
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DraftRejectionReason {
    StaleOrUnknownContextPacket,
    UnsupportedActionKind,
    MissingSourceRefs,
    SourceRefsNotPresentInContextPacket,
    MissingDataQualityIssueRefs,
    WrongReviewGate,
    BlockedSideEffectRequested,
    UnsupportedSideEffectRequested,
    AttemptedAmbiguityHiding,
    SensitivePayloadExposureAttempted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FeedbackOutcome {
    Completed,
    Deferred,
    SuppressedByManager,
    SourceFactWasWrong,
    NotActionable,
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
    #[builder(default)]
    issue_refs: Vec<IssueRef>,
    reviewed_resolution_status: Option<data_quality::ResolutionStatus>,
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

    pub fn issue_refs(&self) -> &[IssueRef] {
        &self.issue_refs
    }

    pub const fn reviewed_resolution_status(&self) -> Option<data_quality::ResolutionStatus> {
        self.reviewed_resolution_status
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
    #[error("issue ref cannot be empty")]
    EmptyIssueRef,
    #[error("action id cannot be empty")]
    EmptyActionId,
    #[error("context packet id cannot be empty")]
    EmptyContextPacketId,
    #[error("correlation id cannot be empty")]
    EmptyCorrelationId,
    #[error("action rationale cannot be empty")]
    EmptyActionRationale,
    #[error("labor minutes must be greater than zero")]
    ZeroLaborMinutes,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Workflow;

impl Workflow {
    pub fn evaluate(request: Request) -> Packet {
        let actions = request
            .candidates
            .iter()
            .map(action_for_candidate)
            .collect::<Vec<_>>();
        let before_minutes = total_before_minutes(&actions);
        let after_minutes = total_after_minutes(&actions);

        Packet {
            workflow: WORKFLOW_NAME,
            schema_version: SCHEMA_VERSION,
            context_packet_id: ContextPacketId::try_new(format!(
                "data-quality-hygiene-context:{:?}:{:?}",
                request.location_id, request.operating_day
            ))
            .expect("formatted context packet id is non-empty"),
            correlation_id: CorrelationId::try_new(format!(
                "data-quality-hygiene:{:?}:{:?}",
                request.location_id, request.operating_day
            ))
            .expect("formatted correlation id is non-empty"),
            location_id: request.location_id,
            operating_day: request.operating_day,
            prepared_for: request.prepared_for,
            candidates: request.candidates,
            actions,
            safe_agent_actions: safe_agent_actions_for(),
            blocked_actions: blocked_actions_for(),
            before_minutes,
            after_minutes,
        }
    }
}

fn action_for_candidate(candidate: &Candidate) -> Action {
    let (kind, owner_persona, removed_manual_work, before, after) = action_shape_for(candidate);
    Action::builder()
        .id(
            ActionId::try_new(format!("dq-action-{}", candidate.id().as_str()))
                .expect("candidate ids are non-empty"),
        )
        .kind(kind)
        .priority(priority_for(candidate))
        .owner_persona(owner_persona)
        .removed_manual_work(removed_manual_work)
        .rationale(rationale_for(candidate))
        .source_record_refs(candidate.effective_source_record_refs())
        .issue_refs(vec![candidate.id.clone()])
        .required_review_gates(review_gates_for(candidate))
        .labor_impact(LaborImpactEstimate::new(
            LaborMinutes::try_new(before).expect("static before minutes are valid"),
            LaborMinutes::try_new(after).expect("static after minutes are valid"),
        ))
        .build()
}

fn action_shape_for(
    candidate: &Candidate,
) -> (ActionKind, HygienePersona, RemovedManualWork, u16, u16) {
    if candidate.sensitivity == Sensitivity::QuarantinedSensitivePayload {
        return (
            ActionKind::EscalateSensitiveOrQuarantinedPayload,
            HygienePersona::GeneralManager,
            RemovedManualWork::SensitivePayloadEscalation,
            20,
            8,
        );
    }

    match candidate.issue.kind() {
        data_quality::Kind::MissingVaccinationRecord => (
            ActionKind::ReviewStaleVaccinationSourceFreshness,
            HygienePersona::FrontDeskLead,
            RemovedManualWork::SourceFreshnessReview,
            25,
            10,
        ),
        data_quality::Kind::DuplicateSourceRecord => (
            ActionKind::ReconcileDuplicateCustomerOrPetCandidate,
            HygienePersona::GeneralManager,
            RemovedManualWork::DuplicateCandidateReconciliation,
            30,
            12,
        ),
        data_quality::Kind::IncompletePetProfile
        | data_quality::Kind::AmbiguousOwnerPetRelationship => (
            ActionKind::CompleteMissingPetOrCustomerProfileFields,
            HygienePersona::FrontDeskLead,
            RemovedManualWork::IncompleteProfileCleanupPreparation,
            20,
            7,
        ),
        data_quality::Kind::UnmappedServiceType | data_quality::Kind::LocationScopeAmbiguity => (
            ActionKind::NormalizeAmbiguousServiceLineNaming,
            HygienePersona::GeneralManager,
            RemovedManualWork::ServiceLineNormalizationReview,
            20,
            6,
        ),
        data_quality::Kind::CheckoutEvidenceMissing | data_quality::Kind::UnclosedReservation => (
            ActionKind::ReviewCheckoutOrUnclosedReservationEvidence,
            HygienePersona::FrontDeskLead,
            RemovedManualWork::CheckoutEvidenceReview,
            20,
            8,
        ),
        data_quality::Kind::SensitivePayloadQuarantined => (
            ActionKind::EscalateSensitiveOrQuarantinedPayload,
            HygienePersona::GeneralManager,
            RemovedManualWork::SensitivePayloadEscalation,
            20,
            8,
        ),
        data_quality::Kind::MissingRequiredField { .. }
        | data_quality::Kind::AssumptionInForce { .. }
        | data_quality::Kind::UnknownSourceStatus { .. }
        | data_quality::Kind::ConflictingTimestamps
        | data_quality::Kind::PaymentStateConflict => (
            ActionKind::InvestigateMissingSourceEvidence,
            HygienePersona::FrontDeskLead,
            RemovedManualWork::MissingEvidenceInvestigation,
            25,
            8,
        ),
    }
}

fn priority_for(candidate: &Candidate) -> ActionPriority {
    match candidate.issue.severity() {
        data_quality::Severity::Critical | data_quality::Severity::Blocking => ActionPriority::High,
        data_quality::Severity::Warning => ActionPriority::Medium,
        data_quality::Severity::Informational => ActionPriority::Low,
    }
}

fn rationale_for(candidate: &Candidate) -> ActionRationale {
    let text = match candidate.issue.kind() {
        data_quality::Kind::MissingVaccinationRecord => {
            "Route stale or missing vaccination source evidence to staff review while preserving ambiguity; this workflow does not approve service eligibility or send the customer a message."
        }
        data_quality::Kind::DuplicateSourceRecord => {
            "Prepare a source-grounded duplicate candidate for manager review without merging or mutating provider records."
        }
        data_quality::Kind::UnmappedServiceType => {
            "Prepare ambiguous service-line naming for manager review before reporting or labor automation consumes the source value."
        }
        data_quality::Kind::SensitivePayloadQuarantined => {
            "Escalate quarantined sensitive evidence as metadata only; do not expose raw payload contents to the agent."
        }
        _ => {
            "Prepare a source-grounded internal data-quality hygiene task for human review without hiding ambiguity or mutating source systems."
        }
    };
    ActionRationale::try_new(text).expect("static rationale is valid")
}

fn review_gates_for(candidate: &Candidate) -> Vec<policy::ReviewGate> {
    match candidate.issue.kind() {
        data_quality::Kind::MissingVaccinationRecord => vec![policy::ReviewGate::ManagerApproval],
        data_quality::Kind::SensitivePayloadQuarantined => {
            vec![policy::ReviewGate::ManagerApproval]
        }
        data_quality::Kind::PaymentStateConflict => vec![
            policy::ReviewGate::ManagerApproval,
            policy::ReviewGate::RefundOrDepositException,
        ],
        _ if matches!(
            candidate.issue.severity(),
            data_quality::Severity::Blocking | data_quality::Severity::Critical
        ) =>
        {
            vec![policy::ReviewGate::ManagerApproval]
        }
        _ => vec![policy::ReviewGate::ManagerApproval],
    }
}

fn validate_draft_action(
    packet: &Packet,
    action: &DraftAction,
    rejection_reasons: &mut Vec<DraftRejectionReason>,
) {
    if action.source_record_refs.is_empty() {
        rejection_reasons.push(DraftRejectionReason::MissingSourceRefs);
    }
    if action.issue_refs.is_empty() {
        rejection_reasons.push(DraftRejectionReason::MissingDataQualityIssueRefs);
    }
    if action.attempted_ambiguity_resolution {
        rejection_reasons.push(DraftRejectionReason::AttemptedAmbiguityHiding);
    }

    if action
        .source_record_refs
        .iter()
        .any(|source_ref| !packet_has_source_ref(packet, source_ref))
    {
        rejection_reasons.push(DraftRejectionReason::SourceRefsNotPresentInContextPacket);
    }

    let matching_packet_action = packet.actions.iter().find(|packet_action| {
        packet_action.id == action.action_id && packet_action.kind == action.kind
    });
    match matching_packet_action {
        Some(packet_action)
            if packet_action.required_review_gates != action.required_review_gates =>
        {
            rejection_reasons.push(DraftRejectionReason::WrongReviewGate);
        }
        Some(_) => {}
        None => rejection_reasons.push(DraftRejectionReason::UnsupportedActionKind),
    }

    for side_effect in &action.requested_side_effects {
        match classify_requested_side_effect(side_effect.as_str()) {
            RequestedSideEffect::KnownBlocked => {
                rejection_reasons.push(DraftRejectionReason::BlockedSideEffectRequested)
            }
            RequestedSideEffect::Unsupported => {
                rejection_reasons.push(DraftRejectionReason::UnsupportedSideEffectRequested)
            }
        }
    }
}

fn packet_has_source_ref(packet: &Packet, source_ref: &source::RecordRef) -> bool {
    packet
        .candidates
        .iter()
        .flat_map(Candidate::effective_source_record_refs)
        .any(|packet_ref| packet_ref == *source_ref)
}

enum RequestedSideEffect {
    KnownBlocked,
    Unsupported,
}

fn classify_requested_side_effect(side_effect: &str) -> RequestedSideEffect {
    match side_effect.trim() {
        "send_customer_message"
        | "mutate_provider_or_pms_record"
        | "change_staff_schedule"
        | "move_refund_discount_or_payment"
        | "hide_or_auto_resolve_source_ambiguity"
        | "expose_quarantined_sensitive_payload" => RequestedSideEffect::KnownBlocked,
        _ => RequestedSideEffect::Unsupported,
    }
}

fn total_before_minutes(actions: &[Action]) -> AggregateLaborMinutes {
    AggregateLaborMinutes::new(
        actions
            .iter()
            .map(|action| action.labor_impact.before_minutes().get())
            .sum::<u16>(),
    )
}

fn total_after_minutes(actions: &[Action]) -> AggregateLaborMinutes {
    AggregateLaborMinutes::new(
        actions
            .iter()
            .map(|action| action.labor_impact.after_minutes().get())
            .sum::<u16>(),
    )
}

fn safe_agent_actions_for() -> Vec<SafeAgentAction> {
    vec![
        SafeAgentAction::SummarizeSourceEvidence,
        SafeAgentAction::RankHygieneActions,
        SafeAgentAction::DraftInternalCleanupTask,
        SafeAgentAction::PreserveAmbiguityForReview,
        SafeAgentAction::EstimateReconciliationMinutesSaved,
    ]
}

fn blocked_actions_for() -> Vec<BlockedAction> {
    vec![
        BlockedAction::SendCustomerMessage,
        BlockedAction::MutateProviderOrPmsRecord,
        BlockedAction::ChangeStaffSchedule,
        BlockedAction::MoveRefundDiscountOrPayment,
        BlockedAction::HideOrAutoResolveSourceAmbiguity,
        BlockedAction::ExposeQuarantinedSensitivePayload,
    ]
}

fn trimmed_non_empty(value: impl Into<String>, empty_error: Error) -> Result<String> {
    let value = value.into().trim().to_owned();
    if value.is_empty() {
        Err(empty_error)
    } else {
        Ok(value)
    }
}
