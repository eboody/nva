use bon::Builder;
use nutype::nutype;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{CustomerId, LocationId, PetId, StaffId};
use crate::policy;

macro_rules! positive_scalar {
    ($name:ident, $primitive:ty, $error:ident, $message:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        pub struct $name($primitive);

        impl $name {
            pub const fn try_new(value: $primitive) -> std::result::Result<Self, $error> {
                if value == 0 {
                    return Err($error::Zero);
                }
                Ok(Self(value))
            }

            pub const fn get(self) -> $primitive {
                self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Self::try_new(<$primitive>::deserialize(deserializer)?)
                    .map_err(serde::de::Error::custom)
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
        pub enum $error {
            #[error($message)]
            Zero,
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct DurationWeeks(u8);

impl DurationWeeks {
    pub const fn try_new(value: u8) -> std::result::Result<Self, DurationWeeksError> {
        if value == 0 {
            return Err(DurationWeeksError::ZeroWeeks);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl<'de> Deserialize<'de> for DurationWeeks {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DurationWeeksError {
    #[error("training program duration requires at least one week")]
    ZeroWeeks,
}

positive_scalar!(
    SessionCount,
    u16,
    SessionCountError,
    "training package requires at least one session"
);

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
pub struct EnrollmentId(String);

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
pub struct SessionId(String);

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
pub struct SessionRef(String);

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
pub struct ProgressReportId(String);

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
pub struct EvidenceId(String);

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
pub struct MilestoneId(String);

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
pub struct OutcomeDocumentationId(String);

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
pub struct ProgressNote(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("training progress report requires evidence before it can be reviewed")]
    ProgressEvidenceRequired,
    #[error("training outcome claim requires evidence for achieved/readiness claims")]
    OutcomeEvidenceRequired,
    #[error("training outcome documentation requires at least one claim")]
    OutcomeClaimRequired,
    #[error("training package policy does not define a reusable session balance")]
    PackageHasNoReusableBalance,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Program {
    StayAndStudy { duration: DurationWeeks },
    TutorSession,
    GroupClass,
    PuppyKindergarten,
    PrivateLesson,
    AkcCanineGoodCitizenPrep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgramDuration {
    SingleSession,
    Weeks(DurationWeeks),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurriculumUnit {
    PuppyManners,
    LooseLeashWalking,
    Recall,
    ConfidenceBuilding,
    CanineGoodCitizenPrep,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressTracking {
    AttendanceOnly,
    SessionNotesAndMilestones,
    TrainerScorecard,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    BasicManners,
    ReducedReactivity,
    CanineGoodCitizenReadiness,
    OwnerHandlingPlan,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrainerAvailability {
    AnyCertifiedTrainer,
    NamedTrainerRequired,
    WaitlistUntilTrainerAvailable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FollowUpCadence {
    None,
    AfterEachSession,
    AfterProgramCompletion,
    ThirtyDaysAfterCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnrollmentReadiness {
    Ready,
    TrainerReviewRequired { gate: policy::ReviewGate },
    BehaviorOrCareReviewRequired { gate: policy::ReviewGate },
    PackageOrPaymentReviewRequired { gate: policy::ReviewGate },
}

impl EnrollmentReadiness {
    pub fn blocking_gate(&self) -> Option<policy::ReviewGate> {
        match self {
            Self::Ready => None,
            Self::TrainerReviewRequired { gate }
            | Self::BehaviorOrCareReviewRequired { gate }
            | Self::PackageOrPaymentReviewRequired { gate } => Some(gate.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrainerRequirement {
    AnyCertifiedTrainer,
    NamedTrainer { trainer_id: StaffId },
    ProgramQualified { program: Program },
}

impl TrainerRequirement {
    pub const fn requires_named_trainer(&self) -> bool {
        matches!(self, Self::NamedTrainer { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MilestoneStatus {
    NotStarted,
    Introduced,
    Practicing,
    Generalized,
    Completed,
    DeferredNeedsTrainerNote,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurriculumProgress {
    pub milestone_id: MilestoneId,
    pub status: MilestoneStatus,
}

impl CurriculumProgress {
    pub const fn new(milestone_id: MilestoneId, status: MilestoneStatus) -> Self {
        Self {
            milestone_id,
            status,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressEvidence {
    TrainerNote {
        evidence_id: EvidenceId,
        note: ProgressNote,
    },
    MilestoneObserved {
        evidence_id: EvidenceId,
        milestone_id: MilestoneId,
        status: MilestoneStatus,
    },
    SessionCompleted {
        evidence_id: EvidenceId,
        session_id: SessionId,
    },
    OutcomeCandidate {
        evidence_id: EvidenceId,
        outcome: Outcome,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalState {
    Draft,
    TrainerApproved {
        trainer_id: StaffId,
    },
    ManagerApproved {
        manager_id: crate::entities::ManagerId,
    },
    Rejected {
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeReviewState {
    Draft,
    TrainerApproved { trainer_id: StaffId },
    ApprovedForMemberFacingUse { approved_by: StaffId },
    Rejected { gate: policy::ReviewGate },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberFacingBoundary {
    InternalOnly,
    DraftRequiresApproval { gate: policy::ReviewGate },
    ApprovedForMemberFacingUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SessionBalance(u16);

pub mod enrollment {
    pub use super::{EnrollmentId as Id, EnrollmentReadiness as Readiness};
}

pub mod program {
    pub use super::{DurationWeeks, DurationWeeksError, Program, ProgramDuration};
}

pub mod curriculum {
    pub use super::{
        CurriculumProgress as Progress, CurriculumUnit as Unit, MilestoneId, MilestoneStatus,
    };
}

pub mod trainer {
    pub use super::{TrainerAvailability as Availability, TrainerRequirement as Requirement};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub enum Qualification {
        CertifiedTrainer,
        ProgramSpecialist,
        ManagerApprovedException,
    }
}

impl SessionBalance {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }
    pub const fn get(self) -> u16 {
        self.0
    }
    pub const fn remaining(self) -> Self {
        self
    }
    pub const fn reserve_one(self) -> Self {
        Self(self.0.saturating_sub(1))
    }
}

pub mod availability {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CapacityDecision {
        Available,
        Unavailable,
        UnknownRequiresReview,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    pub struct Request {
        pub enrollment_id: EnrollmentId,
        pub pet_id: PetId,
        pub program: Program,
        pub requirement: TrainerRequirement,
        pub capacity: CapacityDecision,
        pub readiness: EnrollmentReadiness,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Decision {
        AssignmentDrafted,
        Waitlist {
            reason: WaitlistReason,
            gate: policy::ReviewGate,
        },
        ReviewRequired {
            reason: ReviewReason,
            gate: policy::ReviewGate,
        },
    }

    impl Decision {
        pub fn provider_mutation_gate(&self) -> Option<policy::ReviewGate> {
            match self {
                Self::AssignmentDrafted => None,
                Self::Waitlist { gate, .. } | Self::ReviewRequired { gate, .. } => {
                    Some(gate.clone())
                }
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum WaitlistReason {
        RequestedTrainerUnavailable,
        CapacitySnapshotUnavailable,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ReviewReason {
        EnrollmentNotReady,
        CapacityUnknown,
    }

    #[derive(Debug, Clone, Default)]
    pub struct Policy;

    impl Policy {
        pub fn evaluate(&self, request: &Request) -> Decision {
            if let Some(gate) = request.readiness.blocking_gate() {
                return Decision::ReviewRequired {
                    reason: ReviewReason::EnrollmentNotReady,
                    gate,
                };
            }
            match request.capacity {
                CapacityDecision::Available => Decision::AssignmentDrafted,
                CapacityDecision::Unavailable => Decision::Waitlist {
                    reason: if request.requirement.requires_named_trainer() {
                        WaitlistReason::RequestedTrainerUnavailable
                    } else {
                        WaitlistReason::CapacitySnapshotUnavailable
                    },
                    gate: policy::ReviewGate::ManagerApproval,
                },
                CapacityDecision::UnknownRequiresReview => Decision::ReviewRequired {
                    reason: ReviewReason::CapacityUnknown,
                    gate: policy::ReviewGate::ManagerApproval,
                },
            }
        }
    }
}

pub mod progress {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Report {
        pub report_id: ProgressReportId,
        pub enrollment_id: EnrollmentId,
        pub session_ref: SessionRef,
        evidence: Vec<ProgressEvidence>,
        milestones: Vec<CurriculumProgress>,
        approval: ApprovalState,
    }

    impl Report {
        pub fn builder() -> ReportBuilder {
            ReportBuilder::default()
        }
        pub fn has_evidence(&self) -> bool {
            !self.evidence.is_empty()
        }
        pub fn milestones(&self) -> &[CurriculumProgress] {
            &self.milestones
        }
        pub fn approval(&self) -> &ApprovalState {
            &self.approval
        }
        pub fn parent_facing_boundary(&self) -> MemberFacingBoundary {
            match &self.approval {
                ApprovalState::Draft | ApprovalState::TrainerApproved { .. } => {
                    MemberFacingBoundary::DraftRequiresApproval {
                        gate: policy::ReviewGate::CustomerMessageApproval,
                    }
                }
                ApprovalState::ManagerApproved { .. } => {
                    MemberFacingBoundary::ApprovedForMemberFacingUse
                }
                ApprovalState::Rejected { .. } => MemberFacingBoundary::InternalOnly,
            }
        }
    }

    #[derive(Default)]
    pub struct ReportBuilder {
        report_id: Option<ProgressReportId>,
        enrollment_id: Option<EnrollmentId>,
        session_ref: Option<SessionRef>,
        evidence: Vec<ProgressEvidence>,
        milestones: Vec<CurriculumProgress>,
        approval: Option<ApprovalState>,
    }

    impl ReportBuilder {
        pub fn report_id(mut self, value: ProgressReportId) -> Self {
            self.report_id = Some(value);
            self
        }
        pub fn enrollment_id(mut self, value: EnrollmentId) -> Self {
            self.enrollment_id = Some(value);
            self
        }
        pub fn session_ref(mut self, value: SessionRef) -> Self {
            self.session_ref = Some(value);
            self
        }
        pub fn evidence(mut self, value: Vec<ProgressEvidence>) -> Self {
            self.evidence = value;
            self
        }
        pub fn milestones(mut self, value: Vec<CurriculumProgress>) -> Self {
            self.milestones = value;
            self
        }
        pub fn approval(mut self, value: ApprovalState) -> Self {
            self.approval = Some(value);
            self
        }
        pub fn build(self) -> Result<Report> {
            if self.evidence.is_empty() {
                return Err(Error::ProgressEvidenceRequired);
            }
            Ok(Report {
                report_id: self.report_id.expect("report_id is required"),
                enrollment_id: self.enrollment_id.expect("enrollment_id is required"),
                session_ref: self.session_ref.expect("session_ref is required"),
                evidence: self.evidence,
                milestones: self.milestones,
                approval: self.approval.unwrap_or(ApprovalState::Draft),
            })
        }
    }
}

pub mod outcome {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ClaimStatus {
        Achieved,
        Readiness,
        Deferred,
        NotAssessed,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ClaimEvidence {
        pub outcome: Outcome,
        pub status: ClaimStatus,
        pub evidence: Vec<EvidenceId>,
        pub milestones: Vec<MilestoneId>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Claim {
        pub outcome: Outcome,
        pub status: ClaimStatus,
        evidence: Vec<EvidenceId>,
        milestones: Vec<MilestoneId>,
    }

    impl Claim {
        pub fn from_evidence(value: ClaimEvidence) -> Result<Self> {
            if matches!(value.status, ClaimStatus::Achieved | ClaimStatus::Readiness)
                && value.evidence.is_empty()
            {
                return Err(Error::OutcomeEvidenceRequired);
            }
            Ok(Self {
                outcome: value.outcome,
                status: value.status,
                evidence: value.evidence,
                milestones: value.milestones,
            })
        }
        pub fn evidence(&self) -> &[EvidenceId] {
            &self.evidence
        }
        pub fn milestones(&self) -> &[MilestoneId] {
            &self.milestones
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Documentation {
        pub documentation_id: OutcomeDocumentationId,
        pub enrollment_id: EnrollmentId,
        pub pet_id: PetId,
        pub location_id: LocationId,
        claims: Vec<Claim>,
        review: OutcomeReviewState,
    }

    impl Documentation {
        pub fn builder() -> DocumentationBuilder {
            DocumentationBuilder::default()
        }
        pub fn claims(&self) -> &[Claim] {
            &self.claims
        }
        pub fn review(&self) -> &OutcomeReviewState {
            &self.review
        }
        pub fn member_facing_boundary(&self) -> MemberFacingBoundary {
            match &self.review {
                OutcomeReviewState::ApprovedForMemberFacingUse { .. } => {
                    MemberFacingBoundary::ApprovedForMemberFacingUse
                }
                OutcomeReviewState::Draft | OutcomeReviewState::TrainerApproved { .. } => {
                    MemberFacingBoundary::DraftRequiresApproval {
                        gate: policy::ReviewGate::CustomerMessageApproval,
                    }
                }
                OutcomeReviewState::Rejected { .. } => MemberFacingBoundary::InternalOnly,
            }
        }
    }

    #[derive(Default)]
    pub struct DocumentationBuilder {
        documentation_id: Option<OutcomeDocumentationId>,
        enrollment_id: Option<EnrollmentId>,
        pet_id: Option<PetId>,
        location_id: Option<LocationId>,
        claims: Vec<Claim>,
        review: Option<OutcomeReviewState>,
    }

    impl DocumentationBuilder {
        pub fn documentation_id(mut self, value: OutcomeDocumentationId) -> Self {
            self.documentation_id = Some(value);
            self
        }
        pub fn enrollment_id(mut self, value: EnrollmentId) -> Self {
            self.enrollment_id = Some(value);
            self
        }
        pub fn pet_id(mut self, value: PetId) -> Self {
            self.pet_id = Some(value);
            self
        }
        pub fn location_id(mut self, value: LocationId) -> Self {
            self.location_id = Some(value);
            self
        }
        pub fn claims(mut self, value: Vec<Claim>) -> Self {
            self.claims = value;
            self
        }
        pub fn review(mut self, value: OutcomeReviewState) -> Self {
            self.review = Some(value);
            self
        }
        pub fn build(self) -> Result<Documentation> {
            if self.claims.is_empty() {
                return Err(Error::OutcomeClaimRequired);
            }
            Ok(Documentation {
                documentation_id: self.documentation_id.expect("documentation_id is required"),
                enrollment_id: self.enrollment_id.expect("enrollment_id is required"),
                pet_id: self.pet_id.expect("pet_id is required"),
                location_id: self.location_id.expect("location_id is required"),
                claims: self.claims,
                review: self.review.unwrap_or(OutcomeReviewState::Draft),
            })
        }
    }
}

pub mod package {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Policy {
        PayPerSession,
        MultiSessionPackage { sessions: SessionCount },
        BoardAndTrainBundle,
    }

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
    pub struct Id(String);

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum LedgerEntry {
        Purchased { sessions: SessionCount },
        Reserved { session_id: SessionId },
        Consumed { session_id: SessionId },
        Released { session_id: SessionId },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct OpeningLedger {
        pub package_id: Id,
        pub customer_id: CustomerId,
        pub pet_id: PetId,
        pub policy: Policy,
        pub entries: Vec<LedgerEntry>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Ledger {
        package_id: Id,
        pub customer_id: CustomerId,
        pub pet_id: PetId,
        policy: Policy,
        entries: Vec<LedgerEntry>,
    }

    impl Ledger {
        pub fn open(opening: OpeningLedger) -> Result<Self> {
            if !matches!(opening.policy, Policy::MultiSessionPackage { .. }) {
                return Err(Error::PackageHasNoReusableBalance);
            }
            Ok(Self {
                package_id: opening.package_id,
                customer_id: opening.customer_id,
                pet_id: opening.pet_id,
                policy: opening.policy,
                entries: opening.entries,
            })
        }
        pub fn package_id(&self) -> &Id {
            &self.package_id
        }
        pub fn entries(&self) -> &[LedgerEntry] {
            &self.entries
        }
        pub fn balance(&self) -> SessionBalance {
            let Policy::MultiSessionPackage { sessions } = self.policy else {
                return SessionBalance::new(0);
            };
            let used = self.entries.iter().fold(0u16, |used, entry| match entry {
                LedgerEntry::Reserved { .. } | LedgerEntry::Consumed { .. } => {
                    used.saturating_add(1)
                }
                LedgerEntry::Released { .. } => used.saturating_sub(1),
                LedgerEntry::Purchased { .. } => used,
            });
            SessionBalance::new(sessions.get().saturating_sub(used))
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum UsageDecision {
        ReserveNextSession {
            package_id: Id,
            remaining_after_reservation: SessionBalance,
        },
        NoRemainingSessions {
            package_id: Id,
            gate: policy::ReviewGate,
        },
        ReconciliationRequired {
            package_id: Id,
            gate: policy::ReviewGate,
        },
    }

    #[derive(Debug, Clone, Default)]
    pub struct UsagePolicy;

    impl UsagePolicy {
        pub fn decide_usage(&self, ledger: &Ledger) -> UsageDecision {
            let balance = ledger.balance();
            if balance.get() == 0 {
                UsageDecision::NoRemainingSessions {
                    package_id: ledger.package_id().clone(),
                    gate: policy::ReviewGate::RefundOrDepositException,
                }
            } else {
                UsageDecision::ReserveNextSession {
                    package_id: ledger.package_id().clone(),
                    remaining_after_reservation: balance.reserve_one(),
                }
            }
        }
    }
}

pub mod follow_up {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Trigger {
        SessionCompleted { session_id: SessionId },
        ProgramCompleted { enrollment_id: EnrollmentId },
        LaterCadenceCheckpoint { enrollment_id: EnrollmentId },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Purpose {
        ProgressUpdate,
        HomeworkCoaching,
        ProgramCompletionSummary,
        ReEnrollmentPrompt,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum EvidenceReadiness {
        ProgressAndHomeworkReady,
        NeedsTrainerEvidence,
        OutcomeDisputedOrAmbiguous,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum State {
        NotDue,
        TrainerEvidenceRequired { gate: policy::ReviewGate },
        DraftRequiresApproval { gate: policy::ReviewGate },
        Suppressed,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Plan {
        pub trigger: Trigger,
        purpose: Purpose,
        state: State,
    }

    impl Plan {
        pub const fn purpose(&self) -> Purpose {
            self.purpose
        }
        pub fn state(&self) -> State {
            self.state.clone()
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct Policy;

    impl Policy {
        pub const fn plan(
            &self,
            trigger: Trigger,
            cadence: FollowUpCadence,
            evidence: EvidenceReadiness,
        ) -> Plan {
            let purpose = match trigger {
                Trigger::SessionCompleted { .. } => Purpose::ProgressUpdate,
                Trigger::ProgramCompleted { .. } => Purpose::ProgramCompletionSummary,
                Trigger::LaterCadenceCheckpoint { .. } => Purpose::ReEnrollmentPrompt,
            };
            let cadence_matches = matches!(
                (&trigger, cadence),
                (
                    Trigger::SessionCompleted { .. },
                    FollowUpCadence::AfterEachSession
                ) | (
                    Trigger::ProgramCompleted { .. },
                    FollowUpCadence::AfterProgramCompletion
                ) | (
                    Trigger::LaterCadenceCheckpoint { .. },
                    FollowUpCadence::ThirtyDaysAfterCompletion
                )
            );
            let state = if !cadence_matches || matches!(cadence, FollowUpCadence::None) {
                State::NotDue
            } else {
                match evidence {
                    EvidenceReadiness::ProgressAndHomeworkReady => State::DraftRequiresApproval {
                        gate: policy::ReviewGate::CustomerMessageApproval,
                    },
                    EvidenceReadiness::NeedsTrainerEvidence => State::TrainerEvidenceRequired {
                        gate: policy::ReviewGate::ManagerApproval,
                    },
                    EvidenceReadiness::OutcomeDisputedOrAmbiguous => {
                        State::TrainerEvidenceRequired {
                            gate: policy::ReviewGate::ManagerApproval,
                        }
                    }
                }
            };
            Plan {
                trigger,
                purpose,
                state,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Contract {
    pub program_duration: ProgramDuration,
    #[builder(default)]
    pub curriculum: Vec<CurriculumUnit>,
    pub progress: ProgressTracking,
    #[builder(default)]
    pub outcomes: Vec<Outcome>,
    pub trainer_availability: TrainerAvailability,
    pub package: package::Policy,
    pub follow_up: FollowUpCadence,
}

impl Contract {
    pub fn requires_named_trainer(&self) -> bool {
        matches!(
            self.trainer_availability,
            TrainerAvailability::NamedTrainerRequired
                | TrainerAvailability::WaitlistUntilTrainerAvailable
        )
    }
    pub fn has_outcome(&self, outcome: &Outcome) -> bool {
        self.outcomes.contains(outcome)
    }
    pub fn standard_petsuites() -> Self {
        Self::builder()
            .program_duration(ProgramDuration::Weeks(DurationWeeks::try_new(3).unwrap()))
            .curriculum(vec![
                CurriculumUnit::LooseLeashWalking,
                CurriculumUnit::Recall,
            ])
            .progress(ProgressTracking::SessionNotesAndMilestones)
            .outcomes(vec![Outcome::CanineGoodCitizenReadiness])
            .trainer_availability(TrainerAvailability::NamedTrainerRequired)
            .package(package::Policy::MultiSessionPackage {
                sessions: SessionCount::try_new(6).unwrap(),
            })
            .follow_up(FollowUpCadence::AfterProgramCompletion)
            .build()
    }
}
