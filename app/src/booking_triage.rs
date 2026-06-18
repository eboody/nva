//! Booking triage contracts for deterministic review before agent drafting.
//!
//! The app evaluates reservation readiness from policy/evidence first. Agents
//! may draft review packets or customer-safe scripts only after the deterministic
//! packet exposes the allowed review boundary; provider mutation, booking
//! confirmation, customer sends, and payment movement remain blocked actions.
//!
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
pub enum PetProfileCompleteness {
    Complete,
    MissingRequiredFields,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PetProfile {
    pub name: pet::Name,
    pub completeness: PetProfileCompleteness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyAttachedData {
    pub pet_profile: PetProfile,
    pub policy_snapshot: PolicySnapshot,
}

#[state]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestState {
    Intake,
    PetProfileAttached(PetProfile),
    PolicyAttached(PolicyAttachedData),
    ReadyForPolicyDecision(PolicyAttachedData),
}

#[machine]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request<RequestState> {
    reservation: Reservation,
}

#[transition]
impl Request<Intake> {
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
    pub fn mark_ready_for_policy_decision(self) -> Request<ReadyForPolicyDecision> {
        let ready_data = self.state_data.clone();
        self.transition_with(ready_data)
    }
}

impl<S: RequestStateTrait> Request<S> {
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
pub enum ReadinessBucket {
    ReadyForStaffApproval,
    MissingInfo,
    VaccinePending,
    SpecialReview,
    Waitlisted,
    Offered,
    Confirmed,
    Rejected,
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
pub enum ApprovalGate {
    None,
    StaffApproval,
    ManagerApproval,
    MedicalDocumentReview,
    BehaviorReview,
    CareTeamApproval,
    PaymentManagerApproval,
    CustomerMessageApproval,
    ConfirmedBookingAutomation,
    RejectionApproval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FailureCode {
    MissingRequiredInput,
    StaleSnapshot,
    ConflictingSource,
    UnmappedProviderValue,
    MissingPolicy,
    CapacityUnavailable,
    PolicyHardStop,
    MissingOrUnverifiedVaccine,
    DepositNotSatisfied,
    BehaviorExceptionRequiresReview,
    SpecialCareRequiresReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SafeAgentAction {
    EvidenceSummary,
    InternalTaskDraft,
    ManagerPacketDraft,
    CustomerSafeScriptDraft,
    MissingInfoRequestDraft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BlockedAction {
    ConfirmBooking,
    RejectRequest,
    AcceptSpecialCare,
    ApproveBehaviorException,
    MutateProviderRecord,
    SendCustomerMessage,
    MovePayment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StaffDecisionBoundary {
    DraftConfirmationAllowed,
    ReviewPacketOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfirmationDraftError {
    DeterministicGateNotReadyForDraft,
}

pub mod rule {
    use bon::Builder;
    use serde::{Deserialize, Serialize};

    use super::{ApprovalGate, EvidenceRef, FailureCode, ReadinessBucket, SafeAgentAction};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub enum Id {
        DateRangeAndServiceSupported,
        AccommodationAvailability,
        SizeCapacityRoomOrGroupFit,
        ServiceCapacityAndAddons,
        VaccineRequirements,
        VaccinePendingHandling,
        DepositAndPricingRequirements,
        HolidayBlackoutMinimumStay,
        StaffCoverageConstraints,
        BehaviorRestrictions,
        AnxietyAggressionExceptionHandling,
        MedicationSpecialCareLimits,
        MultiPetConstraints,
        LatePickupCheckoutImpact,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub enum Decision {
        Pass,
        HardBlock,
        NeedsHumanApproval,
        Unknown,
        NotApplicable,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    pub struct ReviewFinding {
        pub rule_id: Id,
        pub failure_code: FailureCode,
        pub readiness_bucket: ReadinessBucket,
        pub human_approval_required: ApprovalGate,
        #[builder(default)]
        pub evidence_refs: Vec<EvidenceRef>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Evaluation {
        pub rule_id: Id,
        pub decision: Decision,
        pub readiness_bucket: ReadinessBucket,
        pub evidence_refs: Vec<EvidenceRef>,
        pub failure_code: Option<FailureCode>,
        pub human_approval_required: ApprovalGate,
        pub safe_agent_actions: Vec<SafeAgentAction>,
    }

    impl Evaluation {
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

        pub fn unknown(finding: ReviewFinding) -> Self {
            Self::blocked_or_review(finding, Decision::Unknown)
        }

        pub fn needs_human_approval(finding: ReviewFinding) -> Self {
            Self::blocked_or_review(finding, Decision::NeedsHumanApproval)
        }

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
pub struct DeterministicResult {
    rule_evaluations: Vec<rule::Evaluation>,
    recommended_status: ReadinessBucket,
    approval_gates: Vec<ApprovalGate>,
    blocked_actions: Vec<BlockedAction>,
}

impl DeterministicResult {
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

    pub const fn recommended_status(&self) -> ReadinessBucket {
        self.recommended_status
    }

    pub fn requires(&self, gate: ApprovalGate) -> bool {
        self.approval_gates.contains(&gate)
    }

    pub fn blocked_actions(&self) -> &[BlockedAction] {
        &self.blocked_actions
    }

    pub fn rule_evaluations(&self) -> &[rule::Evaluation] {
        &self.rule_evaluations
    }

    pub fn staff_may_confirm_without_human_gate(&self) -> bool {
        matches!(
            self.recommended_status,
            ReadinessBucket::ReadyForStaffApproval
        ) && self.approval_gates.is_empty()
    }

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
pub enum AgentRecommendedAction {
    DraftConfirmationForStaffApproval,
    DraftMissingInfoRequest,
    DraftReviewPacket,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiRecommendation {
    recommended_action: AgentRecommendedAction,
    rationale: RecommendationText,
}

impl AiRecommendation {
    pub const fn new(
        recommended_action: AgentRecommendedAction,
        rationale: RecommendationText,
    ) -> Self {
        Self {
            recommended_action,
            rationale,
        }
    }

    pub const fn recommend_staff_confirmation(rationale: RecommendationText) -> Self {
        Self::new(
            AgentRecommendedAction::DraftConfirmationForStaffApproval,
            rationale,
        )
    }

    pub const fn recommended_action(&self) -> AgentRecommendedAction {
        self.recommended_action
    }

    pub const fn rationale(&self) -> &RecommendationText {
        &self.rationale
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmationDraft {
    body: CustomerMessageDraft,
    approval_gate: ApprovalGate,
}

impl ConfirmationDraft {
    pub const fn new(body: CustomerMessageDraft) -> Self {
        Self {
            body,
            approval_gate: ApprovalGate::CustomerMessageApproval,
        }
    }

    pub const fn body(&self) -> &CustomerMessageDraft {
        &self.body
    }

    pub const fn approval_gate(&self) -> ApprovalGate {
        self.approval_gate
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AuditEventDraft {
    PolicyDecisionRecorded,
    ReservationStatusSuggested,
    ConfirmationDraftGenerated,
    MessageApprovalRequested,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffEvaluationPacket {
    reservation: Reservation,
    deterministic_result: DeterministicResult,
    ai_recommendation: Option<AiRecommendation>,
    confirmation_draft: Option<ConfirmationDraft>,
    audit_event_drafts: Vec<AuditEventDraft>,
}

impl StaffEvaluationPacket {
    pub fn new(reservation: Reservation, deterministic_result: DeterministicResult) -> Self {
        Self {
            reservation,
            deterministic_result,
            ai_recommendation: None,
            confirmation_draft: None,
            audit_event_drafts: vec![AuditEventDraft::PolicyDecisionRecorded],
        }
    }

    pub fn with_ai_recommendation(mut self, ai_recommendation: AiRecommendation) -> Self {
        self.ai_recommendation = Some(ai_recommendation);
        self.audit_event_drafts
            .push(AuditEventDraft::ReservationStatusSuggested);
        self.dedup_audit_event_drafts();
        self
    }

    pub fn with_confirmation_draft(mut self, confirmation_draft: ConfirmationDraft) -> Self {
        self = self
            .try_with_confirmation_draft(confirmation_draft)
            .expect("confirmation drafts require ready/offered deterministic gates");
        self
    }

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

    pub const fn reservation(&self) -> &Reservation {
        &self.reservation
    }

    pub const fn deterministic_result(&self) -> &DeterministicResult {
        &self.deterministic_result
    }

    pub fn ai_recommendation(&self) -> &AiRecommendation {
        self.ai_recommendation
            .as_ref()
            .expect("staff evaluation packet should include an AI recommendation")
    }

    pub fn confirmation_draft(&self) -> &ConfirmationDraft {
        self.confirmation_draft
            .as_ref()
            .expect("staff evaluation packet should include a confirmation draft")
    }

    pub fn audit_event_drafts(&self) -> &[AuditEventDraft] {
        &self.audit_event_drafts
    }

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
pub enum Error {
    #[error("booking triage reservation repository could not load requested reservation")]
    ReservationNotFound,
}

pub type AppResult<T> = core::result::Result<T, Error>;

pub mod reservation {
    use super::entities;

    pub trait Repository {
        fn get(&self, id: entities::reservation::Id) -> Option<entities::Reservation>;
    }
}

#[derive(Debug, Clone)]
pub struct Service<R> {
    reservations: R,
}

impl<R> Service<R>
where
    R: reservation::Repository,
{
    pub const fn new(reservations: R) -> Self {
        Self { reservations }
    }

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
