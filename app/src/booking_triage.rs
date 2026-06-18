//! Booking triage contracts for deterministic review before agent drafting.
//!
//! The app evaluates reservation readiness from policy/evidence first. Agents
//! may draft review packets or customer-safe scripts only after the deterministic
//! packet exposes the allowed review boundary; provider mutation, booking
//! confirmation, customer sends, and payment movement remain blocked actions.
//!
//! The typestate request machine models the safe sequence for triage evidence:
//! intake, pet profile attachment, reservation fact attachment, deterministic
//! review, and staff-ready handoff. The machine's generated helper pages are a
//! `statum` implementation detail; this module documents the operational contract
//! here and on the source state variants so external readers understand that the
//! generated `Request`/state APIs enforce evidence order rather than granting live
//! booking authority.
//! ```
//! use app::booking_triage as triage;
//!
//! let vaccine_review = triage::rule::ReviewFinding::builder()
//!     .rule_id(triage::rule::Id::VaccineRequirements)
//!     .failure_code(triage::FailureCode::MissingOrUnverifiedVaccine)
//!     .readiness_bucket(triage::ReadinessBucket::VaccinePending)
//!     .human_approval_required(triage::ApprovalGate::MedicalDocumentReview)
//!     .evidence_refs(vec![triage::EvidenceRef::try_new(
//!         "gingr:reservation:fixture-123:vaccine-expired",
//!     )?])
//!     .build();
//!
//! let deterministic = triage::DeterministicResult::evaluate(vec![
//!     triage::rule::Evaluation::needs_human_approval(vaccine_review),
//! ]);
//!
//! assert_eq!(deterministic.recommended_status(), triage::ReadinessBucket::VaccinePending);
//! assert!(deterministic.requires(triage::ApprovalGate::MedicalDocumentReview));
//! assert_eq!(deterministic.staff_decision_boundary(), triage::StaffDecisionBoundary::ReviewPacketOnly);
//! assert!(deterministic.blocked_actions().contains(&triage::BlockedAction::ConfirmBooking));
//! assert!(deterministic.blocked_actions().contains(&triage::BlockedAction::SendCustomerMessage));
//! assert!(deterministic.blocked_actions().contains(&triage::BlockedAction::MutateProviderRecord));
//!
//! let packet = triage::StaffEvaluationPacket::new(
//!     triage::Reservation::try_new("reservation-fixture-123")?,
//!     deterministic,
//! );
//! let draft = triage::ConfirmationDraft::new(
//!     triage::CustomerMessageDraft::try_new("We can draft this only after staff review.")?,
//! );
//!
//! assert_eq!(
//!     packet.try_with_confirmation_draft(draft).unwrap_err(),
//!     triage::ConfirmationDraftError::DeterministicGateNotReadyForDraft,
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
use nutype::nutype;
use serde::{Deserialize, Serialize};
use statum::{machine, state, transition};

use domain::entities::reservation as reservation_entity;
use domain::{entities, pet};

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 80),
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
pub struct Reservation(String);

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
/// Pet profile carried by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
pub struct PetProfile {
    /// Name preserved as evidence for audit, review, or agent context.
    pub name: pet::Name,
    /// Completeness preserved as evidence for audit, review, or agent context.
    pub completeness: PetProfileCompleteness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Policy attached data carried by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
pub struct PolicyAttachedData {
    /// Pet profile preserved as evidence for audit, review, or agent context.
    pub pet_profile: PetProfile,
    /// Policy snapshot preserved as evidence for audit, review, or agent context.
    pub policy_snapshot: PolicySnapshot,
}

mod request_typestate {
    #![allow(missing_docs)]

    use super::*;

    /// Typestate markers for booking-triage request progress.
    ///
    /// The variants record which source-derived prerequisites are present before
    /// staff or an agent can evaluate booking readiness. The surrounding module
    /// allows missing docs only for undocumented public helper items generated by
    /// `statum` from this documented source enum.
    #[state]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum RequestState {
        /// The intake exists, but no pet profile evidence has been attached yet.
        Intake,
        /// A source-derived pet profile has been attached for policy checks.
        PetProfileAttached(PetProfile),
        /// A policy snapshot has been attached alongside the pet profile.
        PolicyAttached(PolicyAttachedData),
        /// All deterministic inputs required for policy decisioning are present.
        ReadyForPolicyDecision(PolicyAttachedData),
    }

    /// Typestate request machine for booking-triage intake, policy attachment, and decisioning.
    ///
    /// The generated state-specific request types enforce the ordering of evidence
    /// attachment in code: intake first, then pet profile evidence, then policy
    /// evidence, and only then a packet ready for deterministic staff review. The
    /// machine stores source facts but does not confirm bookings, send customer
    /// messages, or mutate a provider/PMS record.
    #[machine]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Request<RequestState> {
        /// Source reservation label or identifier that the typed request evaluates.
        pub(super) reservation: Reservation,
    }

    #[transition]
    impl Request<Intake> {
        /// Attaches pet profile evidence before the request can move to policy decisioning.
        pub fn attach_pet_profile(
            self,
            name: pet::Name,
            completeness: PetProfileCompleteness,
        ) -> Request<PetProfileAttached> {
            self.transition_with(PetProfile { name, completeness })
        }
    }

    #[transition]
    impl Request<PetProfileAttached> {
        /// Attaches policy snapshot evidence before the request can move to policy decisioning.
        pub fn attach_policy_snapshot(
            self,
            policy_snapshot: PolicySnapshot,
        ) -> Request<PolicyAttached> {
            let pet_profile = self.state_data.clone();
            self.transition_with(PolicyAttachedData {
                pet_profile,
                policy_snapshot,
            })
        }
    }

    #[transition]
    impl Request<PolicyAttached> {
        /// Marks the packet as ready for policy decision once required evidence has been attached.
        pub fn mark_ready_for_policy_decision(self) -> Request<ReadyForPolicyDecision> {
            let ready_data = self.state_data.clone();
            self.transition_with(ready_data)
        }
    }
}

pub use request_typestate::{
    Intake, PetProfileAttached, PolicyAttached, ReadyForPolicyDecision, Request, RequestState,
    RequestStateTrait,
};

impl<S: RequestStateTrait> Request<S> {
    /// Returns the reservation carried by this booking-readiness workflow value.
    pub fn reservation(&self) -> &Reservation {
        &self.reservation
    }
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

impl ReadinessBucket {
    const fn priority(self) -> u8 {
        match self {
            Self::Rejected => 95,
            Self::FailedSafely => 90,
            Self::SpecialReview => 80,
            Self::VaccinePending => 70,
            Self::MissingInfo => 60,
            Self::Waitlisted => 50,
            Self::Offered => 40,
            Self::Confirmed => 30,
            Self::ReadyForStaffApproval => 10,
        }
    }
}

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
/// Review-safe agent tasks allowed to save staff time without crossing mutation or send boundaries.
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
    /// Limits the packet to draft confirmation allowed so agents stay inside the approved handoff boundary.
    DraftConfirmationAllowed,
    /// Limits the packet to review packet only so agents stay inside the approved handoff boundary.
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

    use super::{ApprovalGate, EvidenceRef, FailureCode, ReadinessBucket, SafeAgentAction};

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
    /// Review finding carried by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
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
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Evaluation carried by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
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
    }

    impl Evaluation {
        /// Builds or derives pass data for the booking-readiness workflow contract.
        pub fn pass(rule_id: Id, evidence_refs: Vec<EvidenceRef>) -> Self {
            Self {
                rule_id,
                decision: Decision::Pass,
                readiness_bucket: ReadinessBucket::ReadyForStaffApproval,
                evidence_refs,
                failure_code: None,
                human_approval_required: ApprovalGate::None,
                safe_agent_actions: vec![SafeAgentAction::EvidenceSummary],
            }
        }

        /// Builds or derives unknown data for the booking-readiness workflow contract.
        pub fn unknown(finding: ReviewFinding) -> Self {
            Self::blocked_or_review(finding, Decision::Unknown)
        }

        /// Builds or derives needs human approval data for the booking-readiness workflow contract.
        pub fn needs_human_approval(finding: ReviewFinding) -> Self {
            Self::blocked_or_review(finding, Decision::NeedsHumanApproval)
        }

        /// Builds or derives hard block data for the booking-readiness workflow contract.
        pub fn hard_block(finding: ReviewFinding) -> Self {
            Self::blocked_or_review(finding, Decision::HardBlock)
        }

        fn blocked_or_review(finding: ReviewFinding, decision: Decision) -> Self {
            Self {
                rule_id: finding.rule_id,
                decision,
                readiness_bucket: finding.readiness_bucket,
                evidence_refs: finding.evidence_refs,
                failure_code: Some(finding.failure_code),
                human_approval_required: finding.human_approval_required,
                safe_agent_actions: vec![
                    SafeAgentAction::EvidenceSummary,
                    SafeAgentAction::InternalTaskDraft,
                    SafeAgentAction::ManagerPacketDraft,
                ],
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Deterministic result carried by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
pub struct DeterministicResult {
    rule_evaluations: Vec<rule::Evaluation>,
    recommended_status: ReadinessBucket,
    approval_gates: Vec<ApprovalGate>,
    blocked_actions: Vec<BlockedAction>,
}

impl DeterministicResult {
    /// Builds or derives evaluate data for the booking-readiness workflow contract.
    pub fn evaluate(rule_evaluations: Vec<rule::Evaluation>) -> Self {
        let recommended_status = rule_evaluations
            .iter()
            .map(|rule| rule.readiness_bucket)
            .max_by_key(|status| status.priority())
            .unwrap_or(ReadinessBucket::MissingInfo);

        let mut approval_gates: Vec<ApprovalGate> = rule_evaluations
            .iter()
            .map(|rule| rule.human_approval_required)
            .filter(|gate| *gate != ApprovalGate::None)
            .collect();
        approval_gates.sort_unstable();
        approval_gates.dedup();

        let mut blocked_actions = vec![
            BlockedAction::ConfirmBooking,
            BlockedAction::RejectRequest,
            BlockedAction::MutateProviderRecord,
            BlockedAction::SendCustomerMessage,
        ];
        if approval_gates.contains(&ApprovalGate::BehaviorReview) {
            blocked_actions.push(BlockedAction::ApproveBehaviorException);
        }
        if approval_gates.contains(&ApprovalGate::CareTeamApproval) {
            blocked_actions.push(BlockedAction::AcceptSpecialCare);
        }
        if approval_gates.contains(&ApprovalGate::PaymentManagerApproval) {
            blocked_actions.push(BlockedAction::MovePayment);
        }
        blocked_actions.sort_unstable();
        blocked_actions.dedup();

        Self {
            rule_evaluations,
            recommended_status,
            approval_gates,
            blocked_actions,
        }
    }

    /// Returns the recommended status carried by this booking-readiness workflow value.
    pub const fn recommended_status(&self) -> ReadinessBucket {
        self.recommended_status
    }

    /// Reports whether the booking-readiness workflow satisfies the requires safety condition.
    pub fn requires(&self, gate: ApprovalGate) -> bool {
        self.approval_gates.contains(&gate)
    }

    /// Returns the blocked actions carried by this booking-readiness workflow value.
    pub fn blocked_actions(&self) -> &[BlockedAction] {
        &self.blocked_actions
    }

    /// Returns the rule evaluations carried by this booking-readiness workflow value.
    pub fn rule_evaluations(&self) -> &[rule::Evaluation] {
        &self.rule_evaluations
    }

    /// Returns the staff may confirm without human gate carried by this booking-readiness workflow value.
    pub fn staff_may_confirm_without_human_gate(&self) -> bool {
        matches!(
            self.recommended_status,
            ReadinessBucket::ReadyForStaffApproval
        ) && self.approval_gates.is_empty()
    }

    /// Returns the staff decision boundary carried by this booking-readiness workflow value.
    pub const fn staff_decision_boundary(&self) -> StaffDecisionBoundary {
        match self.recommended_status {
            ReadinessBucket::ReadyForStaffApproval | ReadinessBucket::Offered => {
                StaffDecisionBoundary::DraftConfirmationAllowed
            }
            _ => StaffDecisionBoundary::ReviewPacketOnly,
        }
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
/// Ai recommendation carried by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
pub struct AiRecommendation {
    recommended_action: AgentRecommendedAction,
    rationale: RecommendationText,
}

impl AiRecommendation {
    /// Builds the booking-triage service around a read-only reservation evidence repository.
    pub const fn new(
        recommended_action: AgentRecommendedAction,
        rationale: RecommendationText,
    ) -> Self {
        Self {
            recommended_action,
            rationale,
        }
    }

    /// Builds or derives recommend staff confirmation data for the booking-readiness workflow contract.
    pub const fn recommend_staff_confirmation(rationale: RecommendationText) -> Self {
        Self::new(
            AgentRecommendedAction::DraftConfirmationForStaffApproval,
            rationale,
        )
    }

    /// Returns the recommended action carried by this booking-readiness workflow value.
    pub const fn recommended_action(&self) -> AgentRecommendedAction {
        self.recommended_action
    }

    /// Returns the rationale carried by this booking-readiness workflow value.
    pub const fn rationale(&self) -> &RecommendationText {
        &self.rationale
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Confirmation draft carried by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
pub struct ConfirmationDraft {
    body: CustomerMessageDraft,
    approval_gate: ApprovalGate,
}

impl ConfirmationDraft {
    /// Builds the booking-triage service around a read-only reservation evidence repository.
    pub const fn new(body: CustomerMessageDraft) -> Self {
        Self {
            body,
            approval_gate: ApprovalGate::CustomerMessageApproval,
        }
    }

    /// Returns the body carried by this booking-readiness workflow value.
    pub const fn body(&self) -> &CustomerMessageDraft {
        &self.body
    }

    /// Returns the approval gate carried by this booking-readiness workflow value.
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
    /// Routes booking triage work flagged as message approval requested to the right queue, review gate, or agent packet.
    MessageApprovalRequested,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Staff evaluation packet carried by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
pub struct StaffEvaluationPacket {
    reservation: Reservation,
    deterministic_result: DeterministicResult,
    ai_recommendation: Option<AiRecommendation>,
    confirmation_draft: Option<ConfirmationDraft>,
    audit_event_drafts: Vec<AuditEventDraft>,
}

impl StaffEvaluationPacket {
    /// Builds the booking-triage service around a read-only reservation evidence repository.
    pub fn new(reservation: Reservation, deterministic_result: DeterministicResult) -> Self {
        Self {
            reservation,
            deterministic_result,
            ai_recommendation: None,
            confirmation_draft: None,
            audit_event_drafts: vec![AuditEventDraft::PolicyDecisionRecorded],
        }
    }

    /// Returns the with ai recommendation carried by this booking-readiness workflow value.
    pub fn with_ai_recommendation(mut self, ai_recommendation: AiRecommendation) -> Self {
        self.ai_recommendation = Some(ai_recommendation);
        self.audit_event_drafts
            .push(AuditEventDraft::ReservationStatusSuggested);
        self.dedup_audit_event_drafts();
        self
    }

    /// Returns the with confirmation draft carried by this booking-readiness workflow value.
    pub fn with_confirmation_draft(mut self, confirmation_draft: ConfirmationDraft) -> Self {
        self = self
            .try_with_confirmation_draft(confirmation_draft)
            .expect("confirmation drafts require ready/offered deterministic gates");
        self
    }

    /// Attempts to advance the booking-readiness workflow while preserving deterministic safety gates.
    pub fn try_with_confirmation_draft(
        mut self,
        confirmation_draft: ConfirmationDraft,
    ) -> core::result::Result<Self, ConfirmationDraftError> {
        if self.deterministic_result.staff_decision_boundary()
            != StaffDecisionBoundary::DraftConfirmationAllowed
        {
            return Err(ConfirmationDraftError::DeterministicGateNotReadyForDraft);
        }
        self.confirmation_draft = Some(confirmation_draft);
        self.audit_event_drafts
            .push(AuditEventDraft::ConfirmationDraftGenerated);
        self.audit_event_drafts
            .push(AuditEventDraft::MessageApprovalRequested);
        self.dedup_audit_event_drafts();
        Ok(self)
    }

    /// Returns the reservation carried by this booking-readiness workflow value.
    pub const fn reservation(&self) -> &Reservation {
        &self.reservation
    }

    /// Returns the deterministic result carried by this booking-readiness workflow value.
    pub const fn deterministic_result(&self) -> &DeterministicResult {
        &self.deterministic_result
    }

    /// Returns the ai recommendation carried by this booking-readiness workflow value.
    pub fn ai_recommendation(&self) -> &AiRecommendation {
        self.ai_recommendation
            .as_ref()
            .expect("staff evaluation packet should include an AI recommendation")
    }

    /// Returns the confirmation draft carried by this booking-readiness workflow value.
    pub fn confirmation_draft(&self) -> &ConfirmationDraft {
        self.confirmation_draft
            .as_ref()
            .expect("staff evaluation packet should include a confirmation draft")
    }

    /// Returns the audit event drafts carried by this booking-readiness workflow value.
    pub fn audit_event_drafts(&self) -> &[AuditEventDraft] {
        &self.audit_event_drafts
    }

    /// Returns the suggested status carried by this booking-readiness workflow value.
    pub const fn suggested_status(&self) -> reservation_entity::Status {
        match self.deterministic_result.recommended_status {
            ReadinessBucket::ReadyForStaffApproval => reservation_entity::Status::Offered,
            ReadinessBucket::MissingInfo => reservation_entity::Status::MissingInfo,
            ReadinessBucket::VaccinePending => reservation_entity::Status::VaccinePending,
            ReadinessBucket::SpecialReview => reservation_entity::Status::SpecialReview,
            ReadinessBucket::Waitlisted => reservation_entity::Status::Waitlisted,
            ReadinessBucket::Offered => reservation_entity::Status::Offered,
            ReadinessBucket::Confirmed => reservation_entity::Status::Offered,
            ReadinessBucket::Rejected => reservation_entity::Status::SpecialReview,
            ReadinessBucket::FailedSafely => reservation_entity::Status::SpecialReview,
        }
    }

    fn dedup_audit_event_drafts(&mut self) {
        self.audit_event_drafts.sort_unstable();
        self.audit_event_drafts.dedup();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Classifies error values that drive the booking-readiness workflow.
pub enum Error {
    #[error("booking triage reservation repository could not load requested reservation")]
    /// Identifies reservation not found as the reason the workflow must stop, retry, or request review.
    ReservationNotFound,
}

/// Shared app result type used across the booking triage boundary.
pub type AppResult<T> = core::result::Result<T, Error>;

/// Reservation identifiers used by booking-triage packets and review evidence.
pub mod reservation {
    use super::entities;

    /// Read-only reservation repository used to retrieve source facts for booking triage evaluation.
    pub trait Repository {
        /// Fetches the reservation source record by id without confirming, cancelling, messaging, or mutating provider state.
        fn get(&self, id: entities::reservation::Id) -> Option<entities::Reservation>;
    }
}

#[derive(Debug, Clone)]
/// Service carried by the booking-readiness workflow; it keeps booking work grounded in deterministic policy evidence before any agent draft reaches staff.
pub struct Service<R> {
    reservations: R,
}

impl<R> Service<R>
where
    R: reservation::Repository,
{
    /// Builds the booking-triage service around a read-only reservation evidence repository.
    pub const fn new(reservations: R) -> Self {
        Self { reservations }
    }

    /// Evaluates one reservation into a staff review packet using deterministic policy gates before any agent draft is allowed.
    pub fn evaluate(&self, id: entities::reservation::Id) -> AppResult<StaffEvaluationPacket> {
        let reservation = self
            .reservations
            .get(id)
            .ok_or(Error::ReservationNotFound)?;
        let deterministic_result =
            DeterministicResult::evaluate(evaluate_reservation(&reservation));
        Ok(StaffEvaluationPacket::new(
            Reservation::try_new(reservation.id.0.to_string())
                .expect("uuid reservation id should be a non-empty app reservation label"),
            deterministic_result,
        ))
    }
}

fn evaluate_reservation(reservation: &entities::Reservation) -> Vec<rule::Evaluation> {
    if reservation.hard_stops.is_empty() && reservation.deposit_is_satisfied() {
        return vec![rule::Evaluation::pass(
            rule::Id::DateRangeAndServiceSupported,
            vec![
                EvidenceRef::try_new("reservation:requested-without-hard-stops")
                    .expect("static evidence ref is valid"),
            ],
        )];
    }

    let mut evaluations = Vec::new();
    for hard_stop in &reservation.hard_stops {
        evaluations.push(evaluate_hard_stop(hard_stop));
    }
    if !reservation.deposit_is_satisfied() {
        evaluations.push(rule::Evaluation::needs_human_approval(review_finding(
            rule::Id::DepositAndPricingRequirements,
            FailureCode::DepositNotSatisfied,
            ReadinessBucket::SpecialReview,
            ApprovalGate::PaymentManagerApproval,
            "deposit:missing-or-unverified",
        )));
    }
    evaluations
}

trait ReservationDepositReadiness {
    fn deposit_is_satisfied(&self) -> bool;
}

impl ReservationDepositReadiness for entities::Reservation {
    fn deposit_is_satisfied(&self) -> bool {
        self.deposit.as_ref().is_some_and(|deposit| {
            matches!(
                deposit.status(),
                domain::payment::DepositStatus::Paid
                    | domain::payment::DepositStatus::NotRequired
                    | domain::payment::DepositStatus::WaivedByManager
            )
        })
    }
}

fn evaluate_hard_stop(hard_stop: &entities::HardStop) -> rule::Evaluation {
    match hard_stop {
        entities::HardStop::MissingRequiredVaccine(_) => {
            rule::Evaluation::needs_human_approval(review_finding(
                rule::Id::VaccineRequirements,
                FailureCode::MissingOrUnverifiedVaccine,
                ReadinessBucket::VaccinePending,
                ApprovalGate::MedicalDocumentReview,
                "vaccine:missing-required",
            ))
        }
        entities::HardStop::IneligibleForGroupPlay(_)
        | entities::HardStop::BehaviorReviewRequired => {
            rule::Evaluation::needs_human_approval(review_finding(
                rule::Id::BehaviorRestrictions,
                FailureCode::BehaviorExceptionRequiresReview,
                ReadinessBucket::SpecialReview,
                ApprovalGate::BehaviorReview,
                "behavior:review-required",
            ))
        }
        entities::HardStop::MedicalOrMedicationReviewRequired => {
            rule::Evaluation::needs_human_approval(review_finding(
                rule::Id::MedicationSpecialCareLimits,
                FailureCode::SpecialCareRequiresReview,
                ReadinessBucket::SpecialReview,
                ApprovalGate::CareTeamApproval,
                "care:medical-or-medication-review-required",
            ))
        }
        entities::HardStop::DepositRequired => {
            rule::Evaluation::needs_human_approval(review_finding(
                rule::Id::DepositAndPricingRequirements,
                FailureCode::DepositNotSatisfied,
                ReadinessBucket::SpecialReview,
                ApprovalGate::PaymentManagerApproval,
                "deposit:required",
            ))
        }
        entities::HardStop::InHeat | entities::HardStop::AgeBelowMinimumWeeks(_) => {
            rule::Evaluation::hard_block(review_finding(
                rule::Id::DateRangeAndServiceSupported,
                FailureCode::PolicyHardStop,
                ReadinessBucket::Rejected,
                ApprovalGate::ManagerApproval,
                "policy:hard-stop",
            ))
        }
    }
}

fn review_finding(
    rule_id: rule::Id,
    failure_code: FailureCode,
    readiness_bucket: ReadinessBucket,
    human_approval_required: ApprovalGate,
    evidence_ref: &'static str,
) -> rule::ReviewFinding {
    rule::ReviewFinding::builder()
        .rule_id(rule_id)
        .failure_code(failure_code)
        .readiness_bucket(readiness_bucket)
        .human_approval_required(human_approval_required)
        .evidence_refs(vec![
            EvidenceRef::try_new(evidence_ref).expect("static evidence ref is valid"),
        ])
        .build()
}
