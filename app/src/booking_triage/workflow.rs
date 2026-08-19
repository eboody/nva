use super::*;

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
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
pub struct PolicySnapshot(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Classifies pet profile completeness values that drive the booking-readiness workflow.
pub enum PetProfileCompleteness {
    /// Routes booking triage work flagged as complete to the right queue, review gate, or agent packet.
    Complete,
    /// Routes booking triage work flagged as missing required fields to the right queue, review gate, or agent packet.
    MissingRequiredFields,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Pet profile used by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
pub struct PetProfile {
    /// Name preserved as evidence for audit, review, or agent context.
    pub name: pet::Name,
    /// Completeness preserved as evidence for audit, review, or agent context.
    pub completeness: PetProfileCompleteness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Policy attached data used by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
pub struct PolicyAttachedData {
    /// Pet profile preserved as evidence for audit, review, or agent context.
    pub pet_profile: PetProfile,
    /// Policy snapshot preserved as evidence for audit, review, or agent context.
    pub policy_snapshot: PolicySnapshot,
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 180),
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
pub struct EvidenceRef(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 1000),
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
pub struct RecommendationText(String);

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
pub struct CustomerMessageDraft(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Deterministic booking status bucket used to prioritize staff review.
pub enum ReadinessBucket {
    /// Prioritizes reservations that are ready for staff approval for staff triage queues.
    ReadyForStaffApproval,
    /// Prioritizes reservations that are missing info for staff triage queues.
    MissingInfo,
    /// Prioritizes reservations that are vaccine pending for staff triage queues.
    VaccinePending,
    /// Prioritizes reservations that are special review for staff triage queues.
    SpecialReview,
    /// Prioritizes reservations that are waitlisted for staff triage queues.
    Waitlisted,
    /// Prioritizes reservations that are offered for staff triage queues.
    Offered,
    /// Prioritizes reservations that are confirmed for staff triage queues.
    Confirmed,
    /// Prioritizes reservations that are rejected for staff triage queues.
    Rejected,
    /// Prioritizes reservations that are failed safely for staff triage queues.
    FailedSafely,
}

impl ReadinessBucket {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Human approval checkpoints that must clear before the workflow can advance.
pub enum ApprovalGate {
    /// Requires none before staff can rely on the packet for the next workflow step.
    None,
    /// Requires staff approval before staff can rely on the packet for the next workflow step.
    StaffApproval,
    /// Requires manager approval before staff can rely on the packet for the next workflow step.
    ManagerApproval,
    /// Requires medical document review before staff can rely on the packet for the next workflow step.
    MedicalDocumentReview,
    /// Requires behavior review before staff can rely on the packet for the next workflow step.
    BehaviorReview,
    /// Requires care team approval before staff can rely on the packet for the next workflow step.
    CareTeamApproval,
    /// Requires payment manager approval before staff can rely on the packet for the next workflow step.
    PaymentManagerApproval,
    /// Requires customer message approval before staff can rely on the packet for the next workflow step.
    CustomerMessageApproval,
    /// Requires confirmed booking automation before staff can rely on the packet for the next workflow step.
    ConfirmedBookingAutomation,
    /// Requires rejection approval before staff can rely on the packet for the next workflow step.
    RejectionApproval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Classifies failure code values that drive the booking-readiness workflow.
pub enum FailureCode {
    /// Identifies missing required input as the reason the workflow must stop, retry, or request review.
    MissingRequiredInput,
    /// Identifies stale snapshot as the reason the workflow must stop, retry, or request review.
    StaleSnapshot,
    /// Identifies conflicting source as the reason the workflow must stop, retry, or request review.
    ConflictingSource,
    /// Identifies unmapped provider value as the reason the workflow must stop, retry, or request review.
    UnmappedProviderValue,
    /// Identifies missing policy as the reason the workflow must stop, retry, or request review.
    MissingPolicy,
    /// Identifies capacity unavailable as the reason the workflow must stop, retry, or request review.
    CapacityUnavailable,
    /// Identifies policy hard stop as the reason the workflow must stop, retry, or request review.
    PolicyHardStop,
    /// Identifies missing or unverified vaccine as the reason the workflow must stop, retry, or request review.
    MissingOrUnverifiedVaccine,
    /// Identifies deposit not satisfied as the reason the workflow must stop, retry, or request review.
    DepositNotSatisfied,
    /// Identifies behavior exception requires review as the reason the workflow must stop, retry, or request review.
    BehaviorExceptionRequiresReview,
    /// Identifies special care requires review as the reason the workflow must stop, retry, or request review.
    SpecialCareRequiresReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Source-backed missing-information reason staff can resolve before promising a booking.
pub enum MissingInfoReason {
    /// The requested arrival, departure, or service window is absent or ambiguous.
    RequestedDateWindow,
    /// Pet identity, species, size, age, sex, or ownership facts are absent or ambiguous.
    PetProfile,
    /// Customer contact or preferred reply channel is absent or ambiguous.
    CustomerContact,
    /// Local service, add-on, package, or policy selection is absent or ambiguous.
    ServiceSelection,
    /// Required source or policy evidence is stale, conflicting, unmapped, or unavailable.
    SourceEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Operational blocker family used to summarize why a booking packet cannot advance autonomously.
pub enum BlockerKind {
    /// Missing intake or profile information must be collected or reviewed by staff.
    MissingInfo,
    /// Vaccine or document proof requires medical/document review.
    Vaccine,
    /// Care, medication, medical, allergy, mobility, or handling facts require care-team review.
    Care,
    /// Behavior, incident, group-play, anxiety, aggression, or temperament facts require review.
    Behavior,
    /// Deposit, payment, waiver, refund, or pricing facts require payment/manager review.
    Payment,
    /// Local policy, holiday, minimum-stay, staffing, capacity, or provider-state rules block progress.
    Policy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Evidence summary for one blocker shown in the staff evaluation packet.
pub struct BlockerEvidence {
    /// Blocker family staff should route to the correct review queue.
    pub kind: BlockerKind,
    /// Deterministic failure code that produced the blocker.
    pub failure_code: FailureCode,
    /// Human/system-of-record gate that must clear before related actions can proceed.
    pub approval_gate: ApprovalGate,
    /// Source references proving the blocker came from observed facts, not model inference.
    pub evidence_refs: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Review-safe agent tasks allowed to save staff time without crossing mutation or send gates.
pub enum SafeAgentAction {
    /// Allows agents to evidence summary for staff review without mutating records or contacting customers.
    EvidenceSummary,
    /// Allows agents to internal task draft for staff review without mutating records or contacting customers.
    InternalTaskDraft,
    /// Allows agents to manager packet draft for staff review without mutating records or contacting customers.
    ManagerPacketDraft,
    /// Allows agents to customer safe script draft for staff review without mutating records or contacting customers.
    CustomerSafeScriptDraft,
    /// Allows agents to missing info request draft for staff review without mutating records or contacting customers.
    MissingInfoRequestDraft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Actions the agent must never perform without a human/operator system of record.
pub enum BlockedAction {
    /// Blocks agents from confirm booking until staff or the system of record performs the action.
    ConfirmBooking,
    /// Blocks agents from reject request until staff or the system of record performs the action.
    RejectRequest,
    /// Blocks agents from accept special care until staff or the system of record performs the action.
    AcceptSpecialCare,
    /// Blocks agents from approve behavior exception until staff or the system of record performs the action.
    ApproveBehaviorException,
    /// Blocks agents from mutate provider record until staff or the system of record performs the action.
    MutateProviderRecord,
    /// Blocks agents from send customer message until staff or the system of record performs the action.
    SendCustomerMessage,
    /// Blocks agents from move payment until staff or the system of record performs the action.
    MovePayment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// How far the packet may advance before a staff decision is required.
pub enum StaffDecisionBoundary {
    /// Limits the packet to draft confirmation allowed so agents stay inside the approved handoff gate.
    DraftConfirmationAllowed,
    /// Limits the packet to review packet only so agents stay inside the approved handoff gate.
    ReviewPacketOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Classifies confirmation draft error values that drive the booking-readiness workflow.
pub enum ConfirmationDraftError {
    /// Identifies deterministic gate not ready for draft as the reason the workflow must stop, retry, or request review.
    DeterministicGateNotReadyForDraft,
}

/// Deterministic booking rules that explain readiness findings and safe agent actions.
pub mod rule {
    use bon::Builder;
    use serde::{Deserialize, Serialize};

    use super::{
        ApprovalGate, BlockerKind, EvidenceRef, FailureCode, MissingInfoReason, ReadinessBucket,
        SafeAgentAction,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Classifies id values that drive the booking-readiness workflow.
    pub enum Id {
        /// Routes booking triage work flagged as date range and service supported to the right queue, review gate, or agent packet.
        DateRangeAndServiceSupported,
        /// Routes booking triage work flagged as accommodation availability to the right queue, review gate, or agent packet.
        AccommodationAvailability,
        /// Routes booking triage work flagged as size capacity room or group fit to the right queue, review gate, or agent packet.
        SizeCapacityRoomOrGroupFit,
        /// Routes booking triage work flagged as service capacity and addons to the right queue, review gate, or agent packet.
        ServiceCapacityAndAddons,
        /// Routes booking triage work flagged as vaccine requirements to the right queue, review gate, or agent packet.
        VaccineRequirements,
        /// Routes booking triage work flagged as vaccine pending handling to the right queue, review gate, or agent packet.
        VaccinePendingHandling,
        /// Routes booking triage work flagged as deposit and pricing requirements to the right queue, review gate, or agent packet.
        DepositAndPricingRequirements,
        /// Routes booking triage work flagged as holiday blackout minimum stay to the right queue, review gate, or agent packet.
        HolidayBlackoutMinimumStay,
        /// Routes booking triage work flagged as staff coverage constraints to the right queue, review gate, or agent packet.
        StaffCoverageConstraints,
        /// Routes booking triage work flagged as behavior restrictions to the right queue, review gate, or agent packet.
        BehaviorRestrictions,
        /// Routes booking triage work flagged as anxiety aggression exception handling to the right queue, review gate, or agent packet.
        AnxietyAggressionExceptionHandling,
        /// Routes booking triage work flagged as medication special care limits to the right queue, review gate, or agent packet.
        MedicationSpecialCareLimits,
        /// Routes booking triage work flagged as multi pet constraints to the right queue, review gate, or agent packet.
        MultiPetConstraints,
        /// Routes booking triage work flagged as late pickup checkout impact to the right queue, review gate, or agent packet.
        LatePickupCheckoutImpact,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Classifies decision values that drive the booking-readiness workflow.
    pub enum Decision {
        /// Routes booking triage work flagged as pass to the right queue, review gate, or agent packet.
        Pass,
        /// Routes booking triage work flagged as hard block to the right queue, review gate, or agent packet.
        HardBlock,
        /// Routes booking triage work flagged as needs human approval to the right queue, review gate, or agent packet.
        NeedsHumanApproval,
        /// Routes booking triage work flagged as unknown to the right queue, review gate, or agent packet.
        Unknown,
        /// Routes booking triage work flagged as not applicable to the right queue, review gate, or agent packet.
        NotApplicable,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    /// Review finding used by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
    pub struct ReviewFinding {
        /// Rule id preserved as evidence for audit, review, or agent context.
        pub rule_id: Id,
        /// Failure code preserved as evidence for audit, review, or agent context.
        pub failure_code: FailureCode,
        /// Readiness bucket preserved as evidence for audit, review, or agent context.
        pub readiness_bucket: ReadinessBucket,
        /// Human approval required preserved as evidence for audit, review, or agent context.
        pub human_approval_required: ApprovalGate,
        #[builder(default)]
        /// Evidence refs preserved as evidence for audit, review, or agent context.
        pub evidence_refs: Vec<EvidenceRef>,
        /// Missing-info reason, when this finding represents information staff must collect.
        pub missing_info_reason: Option<MissingInfoReason>,
        /// Operational blocker family for routing staff review without granting action authority.
        pub blocker_kind: Option<BlockerKind>,
    }

    impl ReviewFinding {}

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Evaluation used by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
    pub struct Evaluation {
        /// Rule id preserved as evidence for audit, review, or agent context.
        pub rule_id: Id,
        /// Decision preserved as evidence for audit, review, or agent context.
        pub decision: Decision,
        /// Readiness bucket preserved as evidence for audit, review, or agent context.
        pub readiness_bucket: ReadinessBucket,
        /// Evidence refs preserved as evidence for audit, review, or agent context.
        pub evidence_refs: Vec<EvidenceRef>,
        /// Failure code preserved as evidence for audit, review, or agent context.
        pub failure_code: Option<FailureCode>,
        /// Human approval required preserved as evidence for audit, review, or agent context.
        pub human_approval_required: ApprovalGate,
        /// Safe agent actions preserved as evidence for audit, review, or agent context.
        pub safe_agent_actions: Vec<SafeAgentAction>,
        /// Missing-info reason preserved for staff collection and customer-message draft review.
        pub missing_info_reason: Option<MissingInfoReason>,
        /// Blocker kind preserved for care, vaccine, payment, policy, or behavior routing.
        pub blocker_kind: Option<BlockerKind>,
    }

    impl Evaluation {}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Deterministic result used by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
pub struct DeterministicResult {
    rule_evaluations: Vec<rule::Evaluation>,
    recommended_status: ReadinessBucket,
    approval_gates: Vec<ApprovalGate>,
    blocked_actions: Vec<BlockedAction>,
    missing_info_reasons: Vec<MissingInfoReason>,
    blocker_evidence: Vec<BlockerEvidence>,
}

impl DeterministicResult {
    /// Returns the recommended status value kept on this booking-readiness workflow object for staff review and agent context.
    pub const fn recommended_status(&self) -> ReadinessBucket {
        self.recommended_status
    }

    /// Returns the blocked actions value kept on this booking-readiness workflow object for staff review and agent context.
    pub fn blocked_actions(&self) -> &[BlockedAction] {
        &self.blocked_actions
    }

    /// Returns the rule evaluations value kept on this booking-readiness workflow object for staff review and agent context.
    pub fn rule_evaluations(&self) -> &[rule::Evaluation] {
        &self.rule_evaluations
    }

    /// Returns source-backed reasons staff must resolve before booking readiness can advance.
    pub fn missing_info_reasons(&self) -> &[MissingInfoReason] {
        &self.missing_info_reasons
    }

    /// Returns care, vaccine, payment, behavior, policy, and missing-info blocker evidence.
    pub fn blocker_evidence(&self) -> &[BlockerEvidence] {
        &self.blocker_evidence
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Classifies agent recommended action values that drive the booking-readiness workflow.
pub enum AgentRecommendedAction {
    /// Routes booking triage work flagged as draft confirmation for staff approval to the right queue, review gate, or agent packet.
    DraftConfirmationForStaffApproval,
    /// Routes booking triage work flagged as draft missing info request to the right queue, review gate, or agent packet.
    DraftMissingInfoRequest,
    /// Routes booking triage work flagged as draft review packet to the right queue, review gate, or agent packet.
    DraftReviewPacket,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Ai recommendation used by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
pub struct AiRecommendation {
    recommended_action: AgentRecommendedAction,
    rationale: RecommendationText,
}

impl AiRecommendation {
    /// Returns the recommended action value kept on this booking-readiness workflow object for staff review and agent context.
    pub const fn recommended_action(&self) -> AgentRecommendedAction {
        self.recommended_action
    }

    /// Returns the rationale value kept on this booking-readiness workflow object for staff review and agent context.
    pub const fn rationale(&self) -> &RecommendationText {
        &self.rationale
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Confirmation draft used by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
pub struct ConfirmationDraft {
    body: CustomerMessageDraft,
    approval_gate: ApprovalGate,
}

impl ConfirmationDraft {
    /// Returns the body value kept on this booking-readiness workflow object for staff review and agent context.
    pub const fn body(&self) -> &CustomerMessageDraft {
        &self.body
    }

    /// Returns the approval gate value kept on this booking-readiness workflow object for staff review and agent context.
    pub const fn approval_gate(&self) -> ApprovalGate {
        self.approval_gate
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Missing-information request draft that always requires staff/customer-message approval.
pub struct MissingInfoDraft {
    body: CustomerMessageDraft,
    approval_gate: ApprovalGate,
}

impl MissingInfoDraft {
    /// Returns the customer-facing draft body awaiting staff approval.
    pub const fn body(&self) -> &CustomerMessageDraft {
        &self.body
    }

    /// Returns the approval gate that must clear before this draft can be sent.
    pub const fn approval_gate(&self) -> ApprovalGate {
        self.approval_gate
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Classifies audit event draft values that drive the booking-readiness workflow.
pub enum AuditEventDraft {
    /// Routes booking triage work flagged as policy decision recorded to the right queue, review gate, or agent packet.
    PolicyDecisionRecorded,
    /// Routes booking triage work flagged as reservation status suggested to the right queue, review gate, or agent packet.
    ReservationStatusSuggested,
    /// Routes booking triage work flagged as confirmation draft generated to the right queue, review gate, or agent packet.
    ConfirmationDraftGenerated,
    /// Routes booking triage work flagged as missing information draft generated to the right queue, review gate, or agent packet.
    MissingInfoDraftGenerated,
    /// Routes booking triage work flagged as message approval requested to the right queue, review gate, or agent packet.
    MessageApprovalRequested,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Staff evaluation packet used by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
pub struct StaffEvaluationPacket {
    reservation: reservation_entity::Id,
    deterministic_result: DeterministicResult,
    ai_recommendation: Option<AiRecommendation>,
    confirmation_draft: Option<ConfirmationDraft>,
    missing_info_draft: Option<MissingInfoDraft>,
    audit_event_drafts: Vec<AuditEventDraft>,
}

impl StaffEvaluationPacket {
    /// Returns the reservation value kept on this booking-readiness workflow object for staff review and agent context.
    pub const fn reservation(&self) -> &reservation_entity::Id {
        &self.reservation
    }

    /// Returns the deterministic result value kept on this booking-readiness workflow object for staff review and agent context.
    pub const fn deterministic_result(&self) -> &DeterministicResult {
        &self.deterministic_result
    }

    /// Returns the audit event drafts value kept on this booking-readiness workflow object for staff review and agent context.
    pub fn audit_event_drafts(&self) -> &[AuditEventDraft] {
        &self.audit_event_drafts
    }
}
