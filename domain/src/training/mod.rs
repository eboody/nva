//! Training service-line rules for enrollment readiness, trainer capacity, curriculum progress, package sessions, and parent-facing follow-up.
//!
//! Operator summary: training helps resort staff decide which program requests can be drafted, which trainer/package/progress/outcome queues need review, and which parent follow-up must stay internal. It can reduce repeated trainer-capacity checks, package/session reconciliation, evidence lookup, and graduation or re-enrollment follow-up by producing typed assignment, report, outcome, package, and follow-up decisions.
//!
//! This module is not permission for live automation. It does not assign trainers in a provider system, move waitlists, send customer messages, adjust packages or payments, or publish outcome/graduation claims. Source facts remain authoritative in `domain::entities`, `domain::care`, `domain::temperament`, `domain::payment`, `domain::policy`, `storage::service_line::training`, and provider/integration mappings; training values carry review gates so trainer, manager, payment, behavior/care, and member-facing approval boundaries protect pets, customers, and staff.

use bon::Builder;
use nutype::nutype;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{CustomerId, LocationId, PetId, StaffId};
use crate::policy;

macro_rules! positive_scalar {
    ($name:ident, $primitive:ty, $error:ident, $message:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        /// Positive training quantity used for package/session counts where zero would invalidate labor and revenue tracking.
        pub struct $name($primitive);

        impl $name {
            /// Rejects zero or unsupported training values before they affect package balances, trainer scheduling, progress reports, or parent summaries.
            pub const fn try_new(value: $primitive) -> std::result::Result<Self, $error> {
                if value == 0 {
                    return Err($error::Zero);
                }
                Ok(Self(value))
            }

            /// Returns the training number used by package balances, scheduling, progress reports, or parent summaries.
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
        /// Training-domain validation failures that prevent unsupported reports, outcomes, or package usage from entering workflow state.
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

/// Training-program duration policy for single-session and multi-week offerings.
pub mod program {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Positive number of weeks in a Stay-and-Study or other multi-week training program.
    pub struct DurationWeeks(u8);

    impl DurationWeeks {
        /// Rejects zero or unsupported training values before they affect package balances, trainer scheduling, progress reports, or parent summaries.
        pub const fn try_new(value: u8) -> std::result::Result<Self, DurationWeeksError> {
            if value == 0 {
                return Err(DurationWeeksError::ZeroWeeks);
            }
            Ok(Self(value))
        }

        /// Returns the training number used by package balances, scheduling, progress reports, or parent summaries.
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
    /// Duration validation error for multi-week training programs that cannot use zero weeks.
    pub enum DurationWeeksError {
        #[error("training program duration requires at least one week")]
        /// Staff can see the zero weeks training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        ZeroWeeks,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Program duration shape used to plan trainer labor and customer expectations.
    pub enum Duration {
        /// Staff can see the single session training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        SingleSession,
        /// Staff can see the weeks training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        Weeks(DurationWeeks),
    }
}

/// Enrollment readiness gate for deciding whether a training assignment can be drafted.
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
    /// Enrollment readiness state and the review gate that blocks assignment when data, behavior, care, or payment facts are incomplete.
    pub enum Readiness {
        /// Enrollment has enough source facts to draft trainer assignment.
        Ready,
        /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
        TrainerReviewRequired {
            /// Approval gate staff must clear before acting on this variant.
            gate: policy::ReviewGate,
        },
        /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
        BehaviorOrCareReviewRequired {
            /// Approval gate staff must clear before acting on this variant.
            gate: policy::ReviewGate,
        },
        /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
        PackageOrPaymentReviewRequired {
            /// Approval gate staff must clear before acting on this variant.
            gate: policy::ReviewGate,
        },
    }

    impl Readiness {
        /// Returns the review gate that blocks trainer assignment until staff clear it.
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

/// Curriculum vocabulary for program units, milestones, and evidence-backed progress tracking.
pub mod curriculum {
    use super::*;

    /// Milestone vocabulary for normalized trainer-observed progress states.
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
        /// Normalized training milestone status observed from trainer notes or source-data ingestion.
        pub enum Status {
            /// Staff can see the not started training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
            NotStarted,
            /// Staff can see the introduced training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
            Introduced,
            /// Staff can see the practicing training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
            Practicing,
            /// Staff can see the generalized training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
            Generalized,
            /// Staff can see the completed training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
            Completed,
            /// Staff can see the deferred needs trainer note training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
            DeferredNeedsTrainerNote,
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Curriculum unit that defines what trainers should work on and report against.
    pub enum Unit {
        /// Staff can see the puppy manners training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        PuppyManners,
        /// Staff can see the loose leash walking training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        LooseLeashWalking,
        /// Staff can see the recall training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        Recall,
        /// Staff can see the confidence building training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        ConfidenceBuilding,
        /// Staff can see the canine good citizen prep training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        CanineGoodCitizenPrep,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Evidence-backed milestone progress entry included in internal and parent-facing reports.
    pub struct Progress {
        /// Milestone identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub milestone_id: milestone::Id,
        /// Status used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub status: milestone::Status,
    }

    impl Progress {
        /// Creates this training value from already-checked enrollment, progress, or package inputs.
        pub const fn new(milestone_id: milestone::Id, status: milestone::Status) -> Self {
            Self {
                milestone_id,
                status,
            }
        }
    }
}

/// Trainer assignment policy for matching programs to certified, named, or program-qualified trainers.
pub mod trainer {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Trainer availability posture used to draft assignments or waitlists without inventing capacity.
    pub enum Availability {
        /// Staff can see the any certified trainer training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        AnyCertifiedTrainer,
        /// Staff can see the named trainer required training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        NamedTrainerRequired,
        /// Staff can see the waitlist until trainer available training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        WaitlistUntilTrainerAvailable,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Trainer requirement that constrains who may deliver a program or session.
    pub enum Requirement {
        /// Staff can see the any certified trainer training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        AnyCertifiedTrainer,
        /// Trainer identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        NamedTrainer {
            /// Trainer whose approval or requirement is tied to this state.
            trainer_id: StaffId,
        },
        /// Program used by staff to prepare training assignment, package, progress, or parent-summary review.
        ProgramQualified {
            /// Training program that the trainer must be qualified to deliver.
            program: Program,
        },
    }

    impl Requirement {
        /// Reports whether trainer assignment must use a named or waitlisted trainer.
        pub const fn requires_named_trainer(&self) -> bool {
            matches!(self, Self::NamedTrainer { .. })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Qualification evidence used to explain why a trainer may own a program.
    pub enum Qualification {
        /// Staff can see the certified trainer training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        CertifiedTrainer,
        /// Staff can see the program specialist training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        ProgramSpecialist,
        /// Staff can see the manager approved exception training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
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
/// Training-domain validation failures that prevent unsupported reports, outcomes, or package usage from entering workflow state.
pub enum Error {
    #[error("training progress report requires evidence before it can be reviewed")]
    /// Staff can see the progress evidence required training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    ProgressEvidenceRequired,
    #[error("training outcome claim requires evidence for achieved/readiness claims")]
    /// Staff can see the outcome evidence required training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    OutcomeEvidenceRequired,
    #[error("training outcome documentation requires at least one claim")]
    /// Staff can see the outcome claim required training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    OutcomeClaimRequired,
    #[error("training package policy does not define a reusable session balance")]
    /// Staff can see the package has no reusable balance training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    PackageHasNoReusableBalance,
}

/// Result type returned by fallible training operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Training program sold or fulfilled by the resort, used for capacity, package, and outcome planning.
pub enum Program {
    /// Duration used by staff to prepare training assignment, package, progress, or parent-summary review.
    StayAndStudy {
        /// Stay-and-study duration staff should use for package and schedule planning.
        duration: program::DurationWeeks,
    },
    /// Staff can see the tutor session training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    TutorSession,
    /// Staff can see the group class training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    GroupClass,
    /// Staff can see the puppy kindergarten training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    PuppyKindergarten,
    /// Staff can see the private lesson training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    PrivateLesson,
    /// Staff can see the AKC canine good citizen prep training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    AkcCanineGoodCitizenPrep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Required progress-recording depth for a training program.
pub enum ProgressTracking {
    /// Staff can see the attendance only training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    AttendanceOnly,
    /// Staff can see the session notes and milestones training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    SessionNotesAndMilestones,
    /// Staff can see the trainer scorecard training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    TrainerScorecard,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Outcome claim vocabulary that must be backed by trainer evidence before customer-facing use.
pub enum Outcome {
    /// Staff can see the basic manners training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    BasicManners,
    /// Staff can see the reduced reactivity training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    ReducedReactivity,
    /// Staff can see the canine good citizen readiness training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    CanineGoodCitizenReadiness,
    /// Staff can see the owner handling plan training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    OwnerHandlingPlan,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Follow-up cadence that determines whether a progress/homework/re-enrollment message is due.
pub enum FollowUpCadence {
    /// No additional workflow gate is required.
    None,
    /// Staff can see the after each session training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    AfterEachSession,
    /// Staff can see the after program completion training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    AfterProgramCompletion,
    /// Staff can see the thirty days after completion training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    ThirtyDaysAfterCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Source evidence attached to progress reports and outcome claims.
pub enum ProgressEvidence {
    /// Staff can see the trainer note training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    TrainerNote {
        /// Evidence identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        evidence_id: EvidenceId,
        /// Note used by staff to prepare training assignment, package, progress, or parent-summary review.
        note: ProgressNote,
    },
    /// Staff can see the milestone observed training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    MilestoneObserved {
        /// Evidence identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        evidence_id: EvidenceId,
        /// Milestone identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        milestone_id: curriculum::milestone::Id,
        /// Status used by staff to prepare training assignment, package, progress, or parent-summary review.
        status: curriculum::milestone::Status,
    },
    /// Staff can see the session completed training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    SessionCompleted {
        /// Evidence identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        evidence_id: EvidenceId,
        /// Session identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        session_id: SessionId,
    },
    /// Staff can see the outcome candidate training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    OutcomeCandidate {
        /// Evidence identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        evidence_id: EvidenceId,
        /// Outcome used by staff to prepare training assignment, package, progress, or parent-summary review.
        outcome: Outcome,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Approval state for progress reports before they become parent-facing summaries.
pub enum ApprovalState {
    /// Staff can see the draft training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    Draft,
    /// Staff can see the trainer approved training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    TrainerApproved {
        /// Trainer identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        trainer_id: StaffId,
    },
    /// Staff can see the manager approved training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    ManagerApproved {
        /// Manager identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        manager_id: crate::entities::ManagerId,
    },
    /// Staff can see the rejected training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    Rejected {
        /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Review state for outcome documentation before achievements are exposed to customers.
pub enum OutcomeReviewState {
    /// Staff can see the draft training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    Draft,
    /// Trainer identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
    TrainerApproved {
        /// Trainer whose approval or requirement is tied to this state.
        trainer_id: StaffId,
    },
    /// Approved by used by staff to prepare training assignment, package, progress, or parent-summary review.
    ApprovedForMemberFacingUse {
        /// Staff member who approved the outcome for parent-facing use.
        approved_by: StaffId,
    },
    /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
    Rejected {
        /// Approval gate staff must clear before acting on this variant.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Parent-facing visibility state for a training report or outcome.
pub enum MemberFacingBoundary {
    /// Staff can see the internal only training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    InternalOnly,
    /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
    DraftRequiresApproval {
        /// Approval gate staff must clear before acting on this variant.
        gate: policy::ReviewGate,
    },
    /// Staff can see the approved for member facing use training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    ApprovedForMemberFacingUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Remaining reusable session balance for a multi-session training package.
pub struct SessionBalance(u16);

impl SessionBalance {
    /// Creates this training value from already-checked enrollment, progress, or package inputs.
    pub const fn new(value: u16) -> Self {
        Self(value)
    }
    /// Returns the training number used by package balances, scheduling, progress reports, or parent summaries.
    pub const fn get(self) -> u16 {
        self.0
    }
    /// Returns the remaining value used by training assignment, progress, package, or parent-summary review.
    pub const fn remaining(self) -> Self {
        self
    }
    /// Returns the reserve one value used by training assignment, progress, package, or parent-summary review.
    pub const fn reserve_one(self) -> Self {
        Self(self.0.saturating_sub(1))
    }
}

/// Trainer availability evaluation for assignment drafting and waitlisting.
pub mod availability {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Trainer-capacity outcome used when drafting assignments or waitlists.
    pub enum CapacityDecision {
        /// Staff can see the available training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        Available,
        /// Staff can see the unavailable training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        Unavailable,
        /// Estimate confidence is unknown and must be reviewed.
        UnknownRequiresReview,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    /// Assignment request combining enrollment readiness, trainer requirement, capacity evidence, and program details.
    pub struct Request {
        /// Enrollment identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub enrollment_id: enrollment::Id,
        /// Pet receiving the training service or parent-facing progress update.
        pub pet_id: PetId,
        /// Program used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub program: Program,
        /// Requirement used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub requirement: trainer::Requirement,
        /// Capacity used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub capacity: CapacityDecision,
        /// Readiness used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub readiness: enrollment::Readiness,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Assignment decision showing whether to draft, waitlist, or require review before mutating provider schedules.
    pub enum Decision {
        /// Staff can see the assignment drafted training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        AssignmentDrafted,
        /// Staff can see the waitlist training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        Waitlist {
            /// Business reason staff should review before proceeding.
            reason: WaitlistReason,
            /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
            gate: policy::ReviewGate,
        },
        /// Staff can see the review required training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        ReviewRequired {
            /// Business reason staff should review before proceeding.
            reason: ReviewReason,
            /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
            gate: policy::ReviewGate,
        },
    }

    impl Decision {
        /// Returns the approval gate required before staff mutate provider trainer assignments.
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
    /// Reason staff should waitlist a training assignment instead of drafting it.
    pub enum WaitlistReason {
        /// Staff can see the requested trainer unavailable training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        RequestedTrainerUnavailable,
        /// Staff can see the capacity snapshot unavailable training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        CapacitySnapshotUnavailable,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Reason staff must review a training assignment before it can be drafted.
    pub enum ReviewReason {
        /// Staff can see the enrollment not ready training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        EnrollmentNotReady,
        /// Staff can see the capacity unknown training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        CapacityUnknown,
    }

    #[derive(Debug, Clone, Default)]
    /// Training policy object that converts source facts into assignment, report, package, or follow-up decisions.
    pub struct Policy;

    impl Policy {
        /// Evaluates the request into a draft assignment, waitlist, or review gate without inventing trainer capacity.
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

/// Progress-report workflow for evidence-backed trainer updates and parent-facing approval gates.
pub mod progress {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Training progress report carrying session evidence, milestones, and approval state.
    pub struct Report {
        /// Report identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub report_id: ProgressReportId,
        /// Enrollment identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub enrollment_id: enrollment::Id,
        /// Session ref used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub session_ref: SessionRef,
        evidence: Vec<ProgressEvidence>,
        milestones: Vec<curriculum::Progress>,
        approval: ApprovalState,
    }

    impl Report {
        /// Starts a validated builder for this training documentation or progress packet.
        pub fn builder() -> ReportBuilder {
            ReportBuilder::default()
        }
        /// Reports whether the progress report includes trainer/source evidence.
        pub fn has_evidence(&self) -> bool {
            !self.evidence.is_empty()
        }
        /// Returns the milestones value used by training assignment, progress, package, or parent-summary review.
        pub fn milestones(&self) -> &[curriculum::Progress] {
            &self.milestones
        }
        /// Returns the approval value used by training assignment, progress, package, or parent-summary review.
        pub fn approval(&self) -> &ApprovalState {
            &self.approval
        }
        /// Returns the parent-facing approval gate value used by training assignment, progress, package, or parent-summary review.
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
    /// Builder for progress reports that rejects reports without trainer/source evidence.
    pub struct ReportBuilder {
        report_id: Option<ProgressReportId>,
        enrollment_id: Option<enrollment::Id>,
        session_ref: Option<SessionRef>,
        evidence: Vec<ProgressEvidence>,
        milestones: Vec<curriculum::Progress>,
        approval: Option<ApprovalState>,
    }

    impl ReportBuilder {
        /// Sets the progress report identifier for the trainer update packet.
        pub fn report_id(mut self, value: ProgressReportId) -> Self {
            self.report_id = Some(value);
            self
        }
        /// Sets the enrollment identifier that anchors this training packet.
        pub fn enrollment_id(mut self, value: enrollment::Id) -> Self {
            self.enrollment_id = Some(value);
            self
        }
        /// Sets the session reference tied to the trainer evidence.
        pub fn session_ref(mut self, value: SessionRef) -> Self {
            self.session_ref = Some(value);
            self
        }
        /// Adds trainer/source evidence that must be present before a progress report can be reviewed.
        pub fn evidence(mut self, value: Vec<ProgressEvidence>) -> Self {
            self.evidence = value;
            self
        }
        /// Sets the milestone progress entries included in the trainer report.
        pub fn milestones(mut self, value: Vec<curriculum::Progress>) -> Self {
            self.milestones = value;
            self
        }
        /// Sets the approval state before a progress report can become parent-facing.
        pub fn approval(mut self, value: ApprovalState) -> Self {
            self.approval = Some(value);
            self
        }
        /// Builds the report only when required evidence exists; missing IDs still indicate programmer misuse in tests/fixtures.
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

/// Outcome-documentation workflow for claims like manners readiness or CGC readiness.
pub mod outcome {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Outcome-claim status used in trainer evidence and parent-facing documentation review.
    pub enum ClaimStatus {
        /// Staff can see the achieved training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        Achieved,
        /// Staff can see the readiness training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        Readiness,
        /// Staff can see the deferred training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        Deferred,
        /// Staff can see the not assessed training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        NotAssessed,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Evidence bundle used to promote an outcome claim into reviewed documentation.
    pub struct ClaimEvidence {
        /// Outcome used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub outcome: Outcome,
        /// Status used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub status: ClaimStatus,
        /// Evidence used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub evidence: Vec<EvidenceId>,
        /// Milestones used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub milestones: Vec<curriculum::milestone::Id>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Outcome claim whose achieved/readiness status cannot exist without supporting evidence.
    pub struct Claim {
        /// Outcome used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub outcome: Outcome,
        /// Status used by staff to prepare training assignment, package, progress, or parent-summary review.
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
        /// Returns the trainer/source evidence that supports this outcome claim.
        pub fn evidence(&self) -> &[EvidenceId] {
            &self.evidence
        }
        /// Returns the milestones value used by training assignment, progress, package, or parent-summary review.
        pub fn milestones(&self) -> &[curriculum::milestone::Id] {
            &self.milestones
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Training outcome documentation packet for customer/account history and manager review.
    pub struct Documentation {
        /// Documentation identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub documentation_id: OutcomeDocumentationId,
        /// Enrollment identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub enrollment_id: enrollment::Id,
        /// Pet receiving the training service or parent-facing progress update.
        pub pet_id: PetId,
        /// Location identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub location_id: LocationId,
        claims: Vec<Claim>,
        review: OutcomeReviewState,
    }

    impl Documentation {
        /// Starts a validated builder for this training documentation or progress packet.
        pub fn builder() -> DocumentationBuilder {
            DocumentationBuilder::default()
        }
        /// Returns the claims value used by training assignment, progress, package, or parent-summary review.
        pub fn claims(&self) -> &[Claim] {
            &self.claims
        }
        /// Returns the review value used by training assignment, progress, package, or parent-summary review.
        pub fn review(&self) -> &OutcomeReviewState {
            &self.review
        }
        /// Returns whether this training outcome can appear in parent-facing copy or must remain internal.
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
    /// Builder for outcome documentation that requires at least one evidence-backed claim.
    pub struct DocumentationBuilder {
        documentation_id: Option<OutcomeDocumentationId>,
        enrollment_id: Option<enrollment::Id>,
        pet_id: Option<PetId>,
        location_id: Option<LocationId>,
        claims: Vec<Claim>,
        review: Option<OutcomeReviewState>,
    }

    impl DocumentationBuilder {
        /// Sets the outcome-documentation identifier for the trainer evidence packet.
        pub fn documentation_id(mut self, value: OutcomeDocumentationId) -> Self {
            self.documentation_id = Some(value);
            self
        }
        /// Sets the enrollment identifier that anchors this training packet.
        pub fn enrollment_id(mut self, value: enrollment::Id) -> Self {
            self.enrollment_id = Some(value);
            self
        }
        /// Sets the pet whose training outcome documentation is being prepared.
        pub fn pet_id(mut self, value: PetId) -> Self {
            self.pet_id = Some(value);
            self
        }
        /// Sets the resort location tied to the training outcome evidence.
        pub fn location_id(mut self, value: LocationId) -> Self {
            self.location_id = Some(value);
            self
        }
        /// Sets the evidence-backed outcome claims for trainer or manager review.
        pub fn claims(mut self, value: Vec<Claim>) -> Self {
            self.claims = value;
            self
        }
        /// Sets the review state controlling parent-facing use of outcome claims.
        pub fn review(mut self, value: OutcomeReviewState) -> Self {
            self.review = Some(value);
            self
        }
        /// Builds the report only when required evidence exists; missing IDs still indicate programmer misuse in tests/fixtures.
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

/// Package and session-ledger workflow for reserving, consuming, and reconciling training sessions.
pub mod package {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Groomer-assignment policies used when booking grooming work.
    pub enum Policy {
        /// Staff can see the pay per session training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        PayPerSession,
        /// Sessions used by staff to prepare training assignment, package, progress, or parent-summary review.
        MultiSessionPackage {
            /// Session count that sets the purchased or reusable package balance.
            sessions: SessionCount,
        },
        /// Staff can see the board and train bundle training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
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
    /// Training package ledger event for purchases, reservations, consumption, and releases.
    pub enum LedgerEntry {
        /// Sessions used by staff to prepare training assignment, package, progress, or parent-summary review.
        Purchased {
            /// Session count that sets the purchased or reusable package balance.
            sessions: SessionCount,
        },
        /// Session identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        Reserved {
            /// Training session tied to the package ledger or follow-up trigger.
            session_id: SessionId,
        },
        /// Session identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        Consumed {
            /// Training session tied to the package ledger or follow-up trigger.
            session_id: SessionId,
        },
        /// Session identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        Released {
            /// Training session tied to the package ledger or follow-up trigger.
            session_id: SessionId,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Opening package ledger assembled from purchased, reserved, consumed, and released session facts.
    pub struct OpeningLedger {
        /// Package identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub package_id: Id,
        /// Customer identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub customer_id: CustomerId,
        /// Pet receiving the training service or parent-facing progress update.
        pub pet_id: PetId,
        /// Policy used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub policy: Policy,
        /// Entries used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub entries: Vec<LedgerEntry>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Training package ledger used to compute remaining reusable sessions without raw counters.
    pub struct Ledger {
        package_id: Id,
        /// Customer identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub customer_id: CustomerId,
        /// Pet receiving the training service or parent-facing progress update.
        pub pet_id: PetId,
        policy: Policy,
        entries: Vec<LedgerEntry>,
    }

    impl Ledger {
        /// Opens a reusable package ledger after confirming the package policy has a session balance.
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
        /// Returns the package id value used by training assignment, progress, package, or parent-summary review.
        pub fn package_id(&self) -> &Id {
            &self.package_id
        }
        /// Returns the entries value used by training assignment, progress, package, or parent-summary review.
        pub fn entries(&self) -> &[LedgerEntry] {
            &self.entries
        }
        /// Returns the balance value used by training assignment, progress, package, or parent-summary review.
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
    /// Package usage decision for reserving the next session or escalating balance/reconciliation issues.
    pub enum UsageDecision {
        /// Staff can see the reserve next session training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        ReserveNextSession {
            /// Package identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
            package_id: Id,
            /// Remaining after reservation used by staff to prepare training assignment, package, progress, or parent-summary review.
            remaining_after_reservation: SessionBalance,
        },
        /// Staff can see the no remaining sessions training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        NoRemainingSessions {
            /// Package identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
            package_id: Id,
            /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
            gate: policy::ReviewGate,
        },
        /// Staff can see the reconciliation required training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        ReconciliationRequired {
            /// Package identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
            package_id: Id,
            /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
            gate: policy::ReviewGate,
        },
    }

    #[derive(Debug, Clone, Default)]
    /// Training usage policy that reserves the next session or escalates package balance issues.
    pub struct UsagePolicy;

    impl UsagePolicy {
        /// Decides whether the next training session can be reserved or needs payment/reconciliation review.
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

/// Follow-up workflow for progress updates, homework coaching, completion summaries, and re-enrollment prompts.
pub mod follow_up {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Follow-up trigger for session completion, program completion, or later cadence checks.
    pub enum Trigger {
        /// Session identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        SessionCompleted {
            /// Training session tied to the package ledger or follow-up trigger.
            session_id: SessionId,
        },
        /// Enrollment identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        ProgramCompleted {
            /// Training enrollment that completed or needs later follow-up.
            enrollment_id: enrollment::Id,
        },
        /// Enrollment identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        LaterCadenceCheckpoint {
            /// Training enrollment that completed or needs later follow-up.
            enrollment_id: enrollment::Id,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Follow-up purpose staff review before progress, homework, completion, or re-enrollment copy is drafted.
    pub enum Purpose {
        /// Staff can see the progress update training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        ProgressUpdate,
        /// Staff can see the homework coaching training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        HomeworkCoaching,
        /// Staff can see the program completion summary training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        ProgramCompletionSummary,
        /// Staff can see the re-enrollment prompt training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        ReEnrollmentPrompt,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Evidence-readiness state that decides whether training follow-up can be drafted or needs trainer input.
    pub enum EvidenceReadiness {
        /// Staff can see the progress and homework ready training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        ProgressAndHomeworkReady,
        /// Staff can see the needs trainer evidence training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        NeedsTrainerEvidence,
        /// Staff can see the outcome disputed or ambiguous training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        OutcomeDisputedOrAmbiguous,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Follow-up state that keeps due/not-due, trainer-evidence, approval, and suppression decisions explicit.
    pub enum State {
        /// Staff can see the not due training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        NotDue,
        /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
        TrainerEvidenceRequired {
            /// Approval gate staff must clear before acting on this variant.
            gate: policy::ReviewGate,
        },
        /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
        DraftRequiresApproval {
            /// Approval gate staff must clear before acting on this variant.
            gate: policy::ReviewGate,
        },
        /// Staff can see the suppressed training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
        Suppressed,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Follow-up plan that separates due/not-due state from approval-gated customer messaging.
    pub struct Plan {
        /// Trigger used by staff to prepare training assignment, package, progress, or parent-summary review.
        pub trigger: Trigger,
        purpose: Purpose,
        state: State,
    }

    impl Plan {
        /// Returns the purpose value used by training assignment, progress, package, or parent-summary review.
        pub const fn purpose(&self) -> Purpose {
            self.purpose
        }
        /// Returns the state value used by training assignment, progress, package, or parent-summary review.
        pub fn state(&self) -> State {
            self.state.clone()
        }
    }

    #[derive(Debug, Clone, Default)]
    /// Training policy object that converts source facts into assignment, report, package, or follow-up decisions.
    pub struct Policy;

    impl Policy {
        /// Builds a training follow-up plan from trigger, cadence, and evidence readiness.
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
/// Location training ruleset tying program duration, curriculum, progress depth, outcomes, trainer availability, package policy, and follow-up cadence together.
pub struct Contract {
    /// Program duration used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub program_duration: program::Duration,
    #[builder(default)]
    /// Curriculum used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub curriculum: Vec<curriculum::Unit>,
    /// Progress used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub progress: ProgressTracking,
    #[builder(default)]
    /// Outcomes used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub outcomes: Vec<Outcome>,
    /// Trainer availability used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub trainer_availability: trainer::Availability,
    /// Package used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub package: package::Policy,
    /// Follow up used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub follow_up: FollowUpCadence,
}

impl Contract {
    /// Reports whether trainer assignment must use a named or waitlisted trainer.
    pub fn requires_named_trainer(&self) -> bool {
        matches!(
            self.trainer_availability,
            trainer::Availability::NamedTrainerRequired
                | trainer::Availability::WaitlistUntilTrainerAvailable
        )
    }
    /// Reports whether the location training rules include the requested outcome claim.
    pub fn has_outcome(&self, outcome: &Outcome) -> bool {
        self.outcomes.contains(outcome)
    }
    /// Builds representative PetSuites-style training rules for docs/tests without claiming they are live policy.
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
