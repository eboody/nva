use bon::Builder;
use nutype::nutype;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{CustomerId, LocationId, PetId, StaffId};
use crate::policy;

macro_rules! positive_scalar {
    ($name:ident, $primitive:ty, $error:ident, $message:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        /// Human-readable name used in training workflows.
        pub struct $name($primitive);

        impl $name {
            /// Promotes boundary input into a validated training domain value.
            pub const fn try_new(value: $primitive) -> std::result::Result<Self, $error> {
                if value == 0 {
                    return Err($error::Zero);
                }
                Ok(Self(value))
            }

            /// Exposes the validated scalar for serialization and adapter boundaries.
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
        /// Validation failures returned by training domain constructors.
        pub enum $error {
            #[error($message)]
            /// Rejects zero where the pet-resort workflow requires a positive quantity.
            Zero,
        }
    };
}

positive_scalar!(
    SessionCount,
    u16,
    SessionCountError,
    "training package requires at least one session"
);

/// Program boundary for training contracts.
pub mod program {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Typed duration weeks domain value that keeps raw primitives out of training workflows.
    pub struct DurationWeeks(u8);

    impl DurationWeeks {
        /// Promotes boundary input into a validated training domain value.
        pub const fn try_new(value: u8) -> std::result::Result<Self, DurationWeeksError> {
            if value == 0 {
                return Err(DurationWeeksError::ZeroWeeks);
            }
            Ok(Self(value))
        }

        /// Exposes the validated scalar for serialization and adapter boundaries.
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
    /// Domain vocabulary for duration weeks error decisions in training workflows.
    pub enum DurationWeeksError {
        #[error("training program duration requires at least one week")]
        /// Zero weeks training enrollment, curriculum, or coach follow-up signal.
        ZeroWeeks,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for duration decisions in training workflows.
    pub enum Duration {
        /// Single session training enrollment, curriculum, or coach follow-up signal.
        SingleSession,
        /// Weeks training enrollment, curriculum, or coach follow-up signal.
        Weeks(DurationWeeks),
    }
}

/// Enrollment boundary for training contracts.
pub mod enrollment {
    use super::*;

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
    /// Domain vocabulary for readiness decisions in training workflows.
    pub enum Readiness {
        /// Ready training enrollment, curriculum, or coach follow-up signal.
        Ready,
        /// Gate fact promoted into this training contract.
        TrainerReviewRequired {
            /// Gate carried by this variant.
            gate: policy::ReviewGate,
        },
        /// Gate fact promoted into this training contract.
        BehaviorOrCareReviewRequired {
            /// Gate carried by this variant.
            gate: policy::ReviewGate,
        },
        /// Gate fact promoted into this training contract.
        PackageOrPaymentReviewRequired {
            /// Gate carried by this variant.
            gate: policy::ReviewGate,
        },
    }

    impl Readiness {
        /// Returns the blocking gate for this training value.
        pub fn blocking_gate(&self) -> Option<policy::ReviewGate> {
            match self {
                Self::Ready => None,
                Self::TrainerReviewRequired { gate }
                | Self::BehaviorOrCareReviewRequired { gate }
                | Self::PackageOrPaymentReviewRequired { gate } => Some(gate.clone()),
            }
        }
    }
}

/// Curriculum boundary for training contracts.
pub mod curriculum {
    use super::*;

    /// Milestone boundary for training contracts.
    pub mod milestone {
        use super::*;

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

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Normalized reservation states observed during source-data ingestion.
        pub enum Status {
            /// Not started training enrollment, curriculum, or coach follow-up signal.
            NotStarted,
            /// Introduced training enrollment, curriculum, or coach follow-up signal.
            Introduced,
            /// Practicing training enrollment, curriculum, or coach follow-up signal.
            Practicing,
            /// Generalized training enrollment, curriculum, or coach follow-up signal.
            Generalized,
            /// Completed training enrollment, curriculum, or coach follow-up signal.
            Completed,
            /// Deferred needs trainer note training enrollment, curriculum, or coach follow-up signal.
            DeferredNeedsTrainerNote,
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for unit decisions in training workflows.
    pub enum Unit {
        /// Puppy manners training enrollment, curriculum, or coach follow-up signal.
        PuppyManners,
        /// Loose leash walking training enrollment, curriculum, or coach follow-up signal.
        LooseLeashWalking,
        /// Recall training enrollment, curriculum, or coach follow-up signal.
        Recall,
        /// Confidence building training enrollment, curriculum, or coach follow-up signal.
        ConfidenceBuilding,
        /// Canine good citizen prep training enrollment, curriculum, or coach follow-up signal.
        CanineGoodCitizenPrep,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed progress domain value that keeps raw primitives out of training workflows.
    pub struct Progress {
        /// Milestone id fact promoted into this training contract.
        pub milestone_id: milestone::Id,
        /// Status fact promoted into this training contract.
        pub status: milestone::Status,
    }

    impl Progress {
        /// Assembles this training value from already-validated domain parts.
        pub const fn new(milestone_id: milestone::Id, status: milestone::Status) -> Self {
            Self {
                milestone_id,
                status,
            }
        }
    }
}

/// Trainer boundary for training contracts.
pub mod trainer {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for availability decisions in training workflows.
    pub enum Availability {
        /// Any certified trainer training enrollment, curriculum, or coach follow-up signal.
        AnyCertifiedTrainer,
        /// Named trainer required training enrollment, curriculum, or coach follow-up signal.
        NamedTrainerRequired,
        /// Waitlist until trainer available training enrollment, curriculum, or coach follow-up signal.
        WaitlistUntilTrainerAvailable,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for requirement decisions in training workflows.
    pub enum Requirement {
        /// Any certified trainer training enrollment, curriculum, or coach follow-up signal.
        AnyCertifiedTrainer,
        /// Trainer id fact promoted into this training contract.
        NamedTrainer {
            /// Trainer id carried by this variant.
            trainer_id: StaffId,
        },
        /// Program fact promoted into this training contract.
        ProgramQualified {
            /// Program carried by this variant.
            program: Program,
        },
    }

    impl Requirement {
        /// Returns this training value's requires named trainer.
        pub const fn requires_named_trainer(&self) -> bool {
            matches!(self, Self::NamedTrainer { .. })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for qualification decisions in training workflows.
    pub enum Qualification {
        /// Certified trainer training enrollment, curriculum, or coach follow-up signal.
        CertifiedTrainer,
        /// Program specialist training enrollment, curriculum, or coach follow-up signal.
        ProgramSpecialist,
        /// Manager approved exception training enrollment, curriculum, or coach follow-up signal.
        ManagerApprovedException,
    }
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
/// Validation failures returned by training domain constructors.
pub enum Error {
    #[error("training progress report requires evidence before it can be reviewed")]
    /// Progress evidence required training enrollment, curriculum, or coach follow-up signal.
    ProgressEvidenceRequired,
    #[error("training outcome claim requires evidence for achieved/readiness claims")]
    /// Outcome evidence required training enrollment, curriculum, or coach follow-up signal.
    OutcomeEvidenceRequired,
    #[error("training outcome documentation requires at least one claim")]
    /// Outcome claim required training enrollment, curriculum, or coach follow-up signal.
    OutcomeClaimRequired,
    #[error("training package policy does not define a reusable session balance")]
    /// Package has no reusable balance training enrollment, curriculum, or coach follow-up signal.
    PackageHasNoReusableBalance,
}

/// Result type returned by fallible training operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for program decisions in training workflows.
pub enum Program {
    /// Duration fact promoted into this training contract.
    StayAndStudy {
        /// Duration carried by this variant.
        duration: program::DurationWeeks,
    },
    /// Tutor session training enrollment, curriculum, or coach follow-up signal.
    TutorSession,
    /// Group class training enrollment, curriculum, or coach follow-up signal.
    GroupClass,
    /// Puppy kindergarten training enrollment, curriculum, or coach follow-up signal.
    PuppyKindergarten,
    /// Private lesson training enrollment, curriculum, or coach follow-up signal.
    PrivateLesson,
    /// Akc canine good citizen prep training enrollment, curriculum, or coach follow-up signal.
    AkcCanineGoodCitizenPrep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for progress tracking decisions in training workflows.
pub enum ProgressTracking {
    /// Attendance only training enrollment, curriculum, or coach follow-up signal.
    AttendanceOnly,
    /// Session notes and milestones training enrollment, curriculum, or coach follow-up signal.
    SessionNotesAndMilestones,
    /// Trainer scorecard training enrollment, curriculum, or coach follow-up signal.
    TrainerScorecard,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for outcome decisions in training workflows.
pub enum Outcome {
    /// Basic manners training enrollment, curriculum, or coach follow-up signal.
    BasicManners,
    /// Reduced reactivity training enrollment, curriculum, or coach follow-up signal.
    ReducedReactivity,
    /// Canine good citizen readiness training enrollment, curriculum, or coach follow-up signal.
    CanineGoodCitizenReadiness,
    /// Owner handling plan training enrollment, curriculum, or coach follow-up signal.
    OwnerHandlingPlan,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for follow up cadence decisions in training workflows.
pub enum FollowUpCadence {
    /// No additional workflow gate is required.
    None,
    /// After each session training enrollment, curriculum, or coach follow-up signal.
    AfterEachSession,
    /// After program completion training enrollment, curriculum, or coach follow-up signal.
    AfterProgramCompletion,
    /// Thirty days after completion training enrollment, curriculum, or coach follow-up signal.
    ThirtyDaysAfterCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for progress evidence decisions in training workflows.
pub enum ProgressEvidence {
    /// Trainer note training enrollment, curriculum, or coach follow-up signal.
    TrainerNote {
        /// Evidence id fact promoted into this training contract.
        evidence_id: EvidenceId,
        /// Note fact promoted into this training contract.
        note: ProgressNote,
    },
    /// Milestone observed training enrollment, curriculum, or coach follow-up signal.
    MilestoneObserved {
        /// Evidence id fact promoted into this training contract.
        evidence_id: EvidenceId,
        /// Milestone id fact promoted into this training contract.
        milestone_id: curriculum::milestone::Id,
        /// Status fact promoted into this training contract.
        status: curriculum::milestone::Status,
    },
    /// Session completed training enrollment, curriculum, or coach follow-up signal.
    SessionCompleted {
        /// Evidence id fact promoted into this training contract.
        evidence_id: EvidenceId,
        /// Session id fact promoted into this training contract.
        session_id: SessionId,
    },
    /// Outcome candidate training enrollment, curriculum, or coach follow-up signal.
    OutcomeCandidate {
        /// Evidence id fact promoted into this training contract.
        evidence_id: EvidenceId,
        /// Outcome fact promoted into this training contract.
        outcome: Outcome,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for approval state decisions in training workflows.
pub enum ApprovalState {
    /// Draft training enrollment, curriculum, or coach follow-up signal.
    Draft,
    /// Trainer approved training enrollment, curriculum, or coach follow-up signal.
    TrainerApproved {
        /// Trainer id fact promoted into this training contract.
        trainer_id: StaffId,
    },
    /// Manager approved training enrollment, curriculum, or coach follow-up signal.
    ManagerApproved {
        /// Manager id fact promoted into this training contract.
        manager_id: crate::entities::ManagerId,
    },
    /// Rejected training enrollment, curriculum, or coach follow-up signal.
    Rejected {
        /// Gate fact promoted into this training contract.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for outcome review state decisions in training workflows.
pub enum OutcomeReviewState {
    /// Draft training enrollment, curriculum, or coach follow-up signal.
    Draft,
    /// Trainer id fact promoted into this training contract.
    TrainerApproved {
        /// Trainer id carried by this variant.
        trainer_id: StaffId,
    },
    /// Approved by fact promoted into this training contract.
    ApprovedForMemberFacingUse {
        /// Approved by carried by this variant.
        approved_by: StaffId,
    },
    /// Gate fact promoted into this training contract.
    Rejected {
        /// Gate carried by this variant.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for member facing boundary decisions in training workflows.
pub enum MemberFacingBoundary {
    /// Internal only training enrollment, curriculum, or coach follow-up signal.
    InternalOnly,
    /// Gate fact promoted into this training contract.
    DraftRequiresApproval {
        /// Gate carried by this variant.
        gate: policy::ReviewGate,
    },
    /// Approved for member facing use training enrollment, curriculum, or coach follow-up signal.
    ApprovedForMemberFacingUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Typed session balance domain value that keeps raw primitives out of training workflows.
pub struct SessionBalance(u16);

impl SessionBalance {
    /// Assembles this training value from already-validated domain parts.
    pub const fn new(value: u16) -> Self {
        Self(value)
    }
    /// Exposes the validated scalar for serialization and adapter boundaries.
    pub const fn get(self) -> u16 {
        self.0
    }
    /// Returns this training value's remaining.
    pub const fn remaining(self) -> Self {
        self
    }
    /// Returns this training value's reserve one.
    pub const fn reserve_one(self) -> Self {
        Self(self.0.saturating_sub(1))
    }
}

/// Availability boundary for training contracts.
pub mod availability {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for capacity decision decisions in training workflows.
    pub enum CapacityDecision {
        /// Available training enrollment, curriculum, or coach follow-up signal.
        Available,
        /// Unavailable training enrollment, curriculum, or coach follow-up signal.
        Unavailable,
        /// Estimate confidence is unknown and must be reviewed.
        UnknownRequiresReview,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    /// Typed request domain value that keeps raw primitives out of training workflows.
    pub struct Request {
        /// Enrollment id fact promoted into this training contract.
        pub enrollment_id: enrollment::Id,
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// Program fact promoted into this training contract.
        pub program: Program,
        /// Requirement fact promoted into this training contract.
        pub requirement: trainer::Requirement,
        /// Capacity fact promoted into this training contract.
        pub capacity: CapacityDecision,
        /// Readiness fact promoted into this training contract.
        pub readiness: enrollment::Readiness,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for decision decisions in training workflows.
    pub enum Decision {
        /// Assignment drafted training enrollment, curriculum, or coach follow-up signal.
        AssignmentDrafted,
        /// Waitlist training enrollment, curriculum, or coach follow-up signal.
        Waitlist {
            /// Business reason staff should review before proceeding.
            reason: WaitlistReason,
            /// Gate fact promoted into this training contract.
            gate: policy::ReviewGate,
        },
        /// Review required training enrollment, curriculum, or coach follow-up signal.
        ReviewRequired {
            /// Business reason staff should review before proceeding.
            reason: ReviewReason,
            /// Gate fact promoted into this training contract.
            gate: policy::ReviewGate,
        },
    }

    impl Decision {
        /// Returns the provider mutation gate for this training value.
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
    /// Domain vocabulary for waitlist reason decisions in training workflows.
    pub enum WaitlistReason {
        /// Requested trainer unavailable training enrollment, curriculum, or coach follow-up signal.
        RequestedTrainerUnavailable,
        /// Capacity snapshot unavailable training enrollment, curriculum, or coach follow-up signal.
        CapacitySnapshotUnavailable,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for review reason decisions in training workflows.
    pub enum ReviewReason {
        /// Enrollment not ready training enrollment, curriculum, or coach follow-up signal.
        EnrollmentNotReady,
        /// Capacity unknown training enrollment, curriculum, or coach follow-up signal.
        CapacityUnknown,
    }

    #[derive(Debug, Clone, Default)]
    /// Typed policy domain value that keeps raw primitives out of training workflows.
    pub struct Policy;

    impl Policy {
        /// Returns the evaluate for this training value.
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

/// Progress boundary for training contracts.
pub mod progress {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed report domain value that keeps raw primitives out of training workflows.
    pub struct Report {
        /// Report id fact promoted into this training contract.
        pub report_id: ProgressReportId,
        /// Enrollment id fact promoted into this training contract.
        pub enrollment_id: enrollment::Id,
        /// Session ref fact promoted into this training contract.
        pub session_ref: SessionRef,
        evidence: Vec<ProgressEvidence>,
        milestones: Vec<curriculum::Progress>,
        approval: ApprovalState,
    }

    impl Report {
        /// Returns the builder for this training value.
        pub fn builder() -> ReportBuilder {
            ReportBuilder::default()
        }
        /// Returns the has evidence for this training value.
        pub fn has_evidence(&self) -> bool {
            !self.evidence.is_empty()
        }
        /// Returns the milestones for this training value.
        pub fn milestones(&self) -> &[curriculum::Progress] {
            &self.milestones
        }
        /// Returns the approval for this training value.
        pub fn approval(&self) -> &ApprovalState {
            &self.approval
        }
        /// Returns the parent facing boundary for this training value.
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
    /// Typed report builder domain value that keeps raw primitives out of training workflows.
    pub struct ReportBuilder {
        report_id: Option<ProgressReportId>,
        enrollment_id: Option<enrollment::Id>,
        session_ref: Option<SessionRef>,
        evidence: Vec<ProgressEvidence>,
        milestones: Vec<curriculum::Progress>,
        approval: Option<ApprovalState>,
    }

    impl ReportBuilder {
        /// Returns the report id for this training value.
        pub fn report_id(mut self, value: ProgressReportId) -> Self {
            self.report_id = Some(value);
            self
        }
        /// Returns the enrollment id for this training value.
        pub fn enrollment_id(mut self, value: enrollment::Id) -> Self {
            self.enrollment_id = Some(value);
            self
        }
        /// Returns the session ref for this training value.
        pub fn session_ref(mut self, value: SessionRef) -> Self {
            self.session_ref = Some(value);
            self
        }
        /// Returns the evidence for this training value.
        pub fn evidence(mut self, value: Vec<ProgressEvidence>) -> Self {
            self.evidence = value;
            self
        }
        /// Returns the milestones for this training value.
        pub fn milestones(mut self, value: Vec<curriculum::Progress>) -> Self {
            self.milestones = value;
            self
        }
        /// Returns the approval for this training value.
        pub fn approval(mut self, value: ApprovalState) -> Self {
            self.approval = Some(value);
            self
        }
        /// Builds the validated training value.
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

/// Outcome boundary for training contracts.
pub mod outcome {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for claim status decisions in training workflows.
    pub enum ClaimStatus {
        /// Achieved training enrollment, curriculum, or coach follow-up signal.
        Achieved,
        /// Readiness training enrollment, curriculum, or coach follow-up signal.
        Readiness,
        /// Deferred training enrollment, curriculum, or coach follow-up signal.
        Deferred,
        /// Not assessed training enrollment, curriculum, or coach follow-up signal.
        NotAssessed,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed claim evidence domain value that keeps raw primitives out of training workflows.
    pub struct ClaimEvidence {
        /// Outcome fact promoted into this training contract.
        pub outcome: Outcome,
        /// Status fact promoted into this training contract.
        pub status: ClaimStatus,
        /// Evidence fact promoted into this training contract.
        pub evidence: Vec<EvidenceId>,
        /// Milestones fact promoted into this training contract.
        pub milestones: Vec<curriculum::milestone::Id>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed claim domain value that keeps raw primitives out of training workflows.
    pub struct Claim {
        /// Outcome fact promoted into this training contract.
        pub outcome: Outcome,
        /// Status fact promoted into this training contract.
        pub status: ClaimStatus,
        evidence: Vec<EvidenceId>,
        milestones: Vec<curriculum::milestone::Id>,
    }

    impl Claim {
        /// Builds this training value from evidence data.
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
        /// Returns the evidence for this training value.
        pub fn evidence(&self) -> &[EvidenceId] {
            &self.evidence
        }
        /// Returns the milestones for this training value.
        pub fn milestones(&self) -> &[curriculum::milestone::Id] {
            &self.milestones
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed documentation domain value that keeps raw primitives out of training workflows.
    pub struct Documentation {
        /// Documentation id fact promoted into this training contract.
        pub documentation_id: OutcomeDocumentationId,
        /// Enrollment id fact promoted into this training contract.
        pub enrollment_id: enrollment::Id,
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// Location id fact promoted into this training contract.
        pub location_id: LocationId,
        claims: Vec<Claim>,
        review: OutcomeReviewState,
    }

    impl Documentation {
        /// Returns the builder for this training value.
        pub fn builder() -> DocumentationBuilder {
            DocumentationBuilder::default()
        }
        /// Returns the claims for this training value.
        pub fn claims(&self) -> &[Claim] {
            &self.claims
        }
        /// Returns the review for this training value.
        pub fn review(&self) -> &OutcomeReviewState {
            &self.review
        }
        /// Returns the member facing boundary for this training value.
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
    /// Typed documentation builder domain value that keeps raw primitives out of training workflows.
    pub struct DocumentationBuilder {
        documentation_id: Option<OutcomeDocumentationId>,
        enrollment_id: Option<enrollment::Id>,
        pet_id: Option<PetId>,
        location_id: Option<LocationId>,
        claims: Vec<Claim>,
        review: Option<OutcomeReviewState>,
    }

    impl DocumentationBuilder {
        /// Returns the documentation id for this training value.
        pub fn documentation_id(mut self, value: OutcomeDocumentationId) -> Self {
            self.documentation_id = Some(value);
            self
        }
        /// Returns the enrollment id for this training value.
        pub fn enrollment_id(mut self, value: enrollment::Id) -> Self {
            self.enrollment_id = Some(value);
            self
        }
        /// Returns the pet id for this training value.
        pub fn pet_id(mut self, value: PetId) -> Self {
            self.pet_id = Some(value);
            self
        }
        /// Returns the location id for this training value.
        pub fn location_id(mut self, value: LocationId) -> Self {
            self.location_id = Some(value);
            self
        }
        /// Returns the claims for this training value.
        pub fn claims(mut self, value: Vec<Claim>) -> Self {
            self.claims = value;
            self
        }
        /// Returns the review for this training value.
        pub fn review(mut self, value: OutcomeReviewState) -> Self {
            self.review = Some(value);
            self
        }
        /// Builds the validated training value.
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

/// Package boundary for training contracts.
pub mod package {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Groomer-assignment policies used when booking grooming work.
    pub enum Policy {
        /// Pay per session training enrollment, curriculum, or coach follow-up signal.
        PayPerSession,
        /// Sessions fact promoted into this training contract.
        MultiSessionPackage {
            /// Sessions carried by this variant.
            sessions: SessionCount,
        },
        /// Board and train bundle training enrollment, curriculum, or coach follow-up signal.
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
    /// Domain vocabulary for ledger entry decisions in training workflows.
    pub enum LedgerEntry {
        /// Sessions fact promoted into this training contract.
        Purchased {
            /// Sessions carried by this variant.
            sessions: SessionCount,
        },
        /// Session id fact promoted into this training contract.
        Reserved {
            /// Session id carried by this variant.
            session_id: SessionId,
        },
        /// Session id fact promoted into this training contract.
        Consumed {
            /// Session id carried by this variant.
            session_id: SessionId,
        },
        /// Session id fact promoted into this training contract.
        Released {
            /// Session id carried by this variant.
            session_id: SessionId,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed opening ledger domain value that keeps raw primitives out of training workflows.
    pub struct OpeningLedger {
        /// Package id fact promoted into this training contract.
        pub package_id: Id,
        /// Customer id fact promoted into this training contract.
        pub customer_id: CustomerId,
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// Policy fact promoted into this training contract.
        pub policy: Policy,
        /// Entries fact promoted into this training contract.
        pub entries: Vec<LedgerEntry>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed ledger domain value that keeps raw primitives out of training workflows.
    pub struct Ledger {
        package_id: Id,
        /// Customer id fact promoted into this training contract.
        pub customer_id: CustomerId,
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        policy: Policy,
        entries: Vec<LedgerEntry>,
    }

    impl Ledger {
        /// Returns the open for this training value.
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
        /// Returns the package id for this training value.
        pub fn package_id(&self) -> &Id {
            &self.package_id
        }
        /// Returns the entries for this training value.
        pub fn entries(&self) -> &[LedgerEntry] {
            &self.entries
        }
        /// Returns the balance for this training value.
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
    /// Domain vocabulary for usage decision decisions in training workflows.
    pub enum UsageDecision {
        /// Reserve next session training enrollment, curriculum, or coach follow-up signal.
        ReserveNextSession {
            /// Package id fact promoted into this training contract.
            package_id: Id,
            /// Remaining after reservation fact promoted into this training contract.
            remaining_after_reservation: SessionBalance,
        },
        /// No remaining sessions training enrollment, curriculum, or coach follow-up signal.
        NoRemainingSessions {
            /// Package id fact promoted into this training contract.
            package_id: Id,
            /// Gate fact promoted into this training contract.
            gate: policy::ReviewGate,
        },
        /// Reconciliation required training enrollment, curriculum, or coach follow-up signal.
        ReconciliationRequired {
            /// Package id fact promoted into this training contract.
            package_id: Id,
            /// Gate fact promoted into this training contract.
            gate: policy::ReviewGate,
        },
    }

    #[derive(Debug, Clone, Default)]
    /// Typed usage policy domain value that keeps raw primitives out of training workflows.
    pub struct UsagePolicy;

    impl UsagePolicy {
        /// Returns the decide usage for this training value.
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

/// Follow up boundary for training contracts.
pub mod follow_up {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for trigger decisions in training workflows.
    pub enum Trigger {
        /// Session id fact promoted into this training contract.
        SessionCompleted {
            /// Session id carried by this variant.
            session_id: SessionId,
        },
        /// Enrollment id fact promoted into this training contract.
        ProgramCompleted {
            /// Enrollment id carried by this variant.
            enrollment_id: enrollment::Id,
        },
        /// Enrollment id fact promoted into this training contract.
        LaterCadenceCheckpoint {
            /// Enrollment id carried by this variant.
            enrollment_id: enrollment::Id,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for purpose decisions in training workflows.
    pub enum Purpose {
        /// Progress update training enrollment, curriculum, or coach follow-up signal.
        ProgressUpdate,
        /// Homework coaching training enrollment, curriculum, or coach follow-up signal.
        HomeworkCoaching,
        /// Program completion summary training enrollment, curriculum, or coach follow-up signal.
        ProgramCompletionSummary,
        /// Re enrollment prompt training enrollment, curriculum, or coach follow-up signal.
        ReEnrollmentPrompt,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for evidence readiness decisions in training workflows.
    pub enum EvidenceReadiness {
        /// Progress and homework ready training enrollment, curriculum, or coach follow-up signal.
        ProgressAndHomeworkReady,
        /// Needs trainer evidence training enrollment, curriculum, or coach follow-up signal.
        NeedsTrainerEvidence,
        /// Outcome disputed or ambiguous training enrollment, curriculum, or coach follow-up signal.
        OutcomeDisputedOrAmbiguous,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for state decisions in training workflows.
    pub enum State {
        /// Not due training enrollment, curriculum, or coach follow-up signal.
        NotDue,
        /// Gate fact promoted into this training contract.
        TrainerEvidenceRequired {
            /// Gate carried by this variant.
            gate: policy::ReviewGate,
        },
        /// Gate fact promoted into this training contract.
        DraftRequiresApproval {
            /// Gate carried by this variant.
            gate: policy::ReviewGate,
        },
        /// Suppressed training enrollment, curriculum, or coach follow-up signal.
        Suppressed,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed plan domain value that keeps raw primitives out of training workflows.
    pub struct Plan {
        /// Trigger fact promoted into this training contract.
        pub trigger: Trigger,
        purpose: Purpose,
        state: State,
    }

    impl Plan {
        /// Returns this training value's purpose.
        pub const fn purpose(&self) -> Purpose {
            self.purpose
        }
        /// Returns the state for this training value.
        pub fn state(&self) -> State {
            self.state.clone()
        }
    }

    #[derive(Debug, Clone, Default)]
    /// Typed policy domain value that keeps raw primitives out of training workflows.
    pub struct Policy;

    impl Policy {
        /// Returns this training value's plan.
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
/// Typed contract domain value that keeps raw primitives out of training workflows.
pub struct Contract {
    /// Program duration fact promoted into this training contract.
    pub program_duration: program::Duration,
    #[builder(default)]
    /// Curriculum fact promoted into this training contract.
    pub curriculum: Vec<curriculum::Unit>,
    /// Progress fact promoted into this training contract.
    pub progress: ProgressTracking,
    #[builder(default)]
    /// Outcomes fact promoted into this training contract.
    pub outcomes: Vec<Outcome>,
    /// Trainer availability fact promoted into this training contract.
    pub trainer_availability: trainer::Availability,
    /// Package fact promoted into this training contract.
    pub package: package::Policy,
    /// Follow up fact promoted into this training contract.
    pub follow_up: FollowUpCadence,
}

impl Contract {
    /// Returns the requires named trainer for this training value.
    pub fn requires_named_trainer(&self) -> bool {
        matches!(
            self.trainer_availability,
            trainer::Availability::NamedTrainerRequired
                | trainer::Availability::WaitlistUntilTrainerAvailable
        )
    }
    /// Returns the has outcome for this training value.
    pub fn has_outcome(&self, outcome: &Outcome) -> bool {
        self.outcomes.contains(outcome)
    }
    /// Returns the standard petsuites for this training value.
    pub fn standard_petsuites() -> Self {
        Self::builder()
            .program_duration(program::Duration::Weeks(
                program::DurationWeeks::try_new(3).unwrap(),
            ))
            .curriculum(vec![
                curriculum::Unit::LooseLeashWalking,
                curriculum::Unit::Recall,
            ])
            .progress(ProgressTracking::SessionNotesAndMilestones)
            .outcomes(vec![Outcome::CanineGoodCitizenReadiness])
            .trainer_availability(trainer::Availability::NamedTrainerRequired)
            .package(package::Policy::MultiSessionPackage {
                sessions: SessionCount::try_new(6).unwrap(),
            })
            .follow_up(FollowUpCadence::AfterProgramCompletion)
            .build()
    }
}
