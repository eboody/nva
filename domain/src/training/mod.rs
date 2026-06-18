//! Training service-line contracts for enrollment readiness, trainer capacity, curriculum progress, package sessions, and parent-facing follow-up.
//!
//! Training programs are high-value operational upsells with stronger evidence requirements than simple appointment notes. This module keeps trainer availability, package balances, progress reports, outcome claims, and customer/parent-facing summaries as typed contracts so automation can draft assignments and follow-ups while trainer, manager, payment, and customer-message gates prevent unsupported live claims.

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

/// Training-program duration boundary for single-session and multi-week offerings.
pub mod program {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Positive number of weeks in a Stay-and-Study or other multi-week training program.
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
    /// Decision vocabulary for duration weeks error in training workflows.
    pub enum DurationWeeksError {
        #[error("training program duration requires at least one week")]
        /// Zero weeks training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        ZeroWeeks,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Program duration shape used to plan trainer labor and customer expectations.
    pub enum Duration {
        /// Single session training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        SingleSession,
        /// Weeks training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        Weeks(DurationWeeks),
    }
}

/// Enrollment readiness boundary for deciding whether a training assignment can be drafted.
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
        /// Source-derived gate carried by this training contract.
        TrainerReviewRequired {
            /// Gate value carried by this review or workflow variant.
            gate: policy::ReviewGate,
        },
        /// Source-derived gate carried by this training contract.
        BehaviorOrCareReviewRequired {
            /// Gate value carried by this review or workflow variant.
            gate: policy::ReviewGate,
        },
        /// Source-derived gate carried by this training contract.
        PackageOrPaymentReviewRequired {
            /// Gate value carried by this review or workflow variant.
            gate: policy::ReviewGate,
        },
    }

    impl Readiness {
        /// Returns the blocking review gate recorded on this training contract.
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

/// Curriculum boundary for program units, milestones, and evidence-backed progress tracking.
pub mod curriculum {
    use super::*;

    /// Milestone boundary for normalized trainer-observed progress states.
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
            /// Not started training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
            NotStarted,
            /// Introduced training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
            Introduced,
            /// Practicing training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
            Practicing,
            /// Generalized training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
            Generalized,
            /// Completed training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
            Completed,
            /// Deferred needs trainer note training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
            DeferredNeedsTrainerNote,
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Curriculum unit that defines what trainers should work on and report against.
    pub enum Unit {
        /// Puppy manners training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        PuppyManners,
        /// Loose leash walking training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        LooseLeashWalking,
        /// Recall training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        Recall,
        /// Confidence building training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        ConfidenceBuilding,
        /// Canine good citizen prep training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        CanineGoodCitizenPrep,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Evidence-backed milestone progress entry included in internal and parent-facing reports.
    pub struct Progress {
        /// Source-derived milestone id carried by this training contract.
        pub milestone_id: milestone::Id,
        /// Source-derived status carried by this training contract.
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

/// Trainer assignment boundary for matching programs to certified, named, or program-qualified trainers.
pub mod trainer {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Trainer availability posture used to draft assignments or waitlists without inventing capacity.
    pub enum Availability {
        /// Any certified trainer training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        AnyCertifiedTrainer,
        /// Named trainer required training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        NamedTrainerRequired,
        /// Waitlist until trainer available training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        WaitlistUntilTrainerAvailable,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Trainer requirement that constrains who may deliver a program or session.
    pub enum Requirement {
        /// Any certified trainer training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        AnyCertifiedTrainer,
        /// Source-derived trainer id carried by this training contract.
        NamedTrainer {
            /// Trainer id value carried by this review or workflow variant.
            trainer_id: StaffId,
        },
        /// Source-derived program carried by this training contract.
        ProgramQualified {
            /// Program value carried by this review or workflow variant.
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
        /// Certified trainer training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        CertifiedTrainer,
        /// Program specialist training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        ProgramSpecialist,
        /// Manager approved exception training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
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
    /// Progress evidence required training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    ProgressEvidenceRequired,
    #[error("training outcome claim requires evidence for achieved/readiness claims")]
    /// Outcome evidence required training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    OutcomeEvidenceRequired,
    #[error("training outcome documentation requires at least one claim")]
    /// Outcome claim required training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    OutcomeClaimRequired,
    #[error("training package policy does not define a reusable session balance")]
    /// Package has no reusable balance training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    PackageHasNoReusableBalance,
}

/// Result type returned by fallible training operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Training program sold or fulfilled by the resort, used for capacity, package, and outcome planning.
pub enum Program {
    /// Source-derived duration carried by this training contract.
    StayAndStudy {
        /// Duration value carried by this review or workflow variant.
        duration: program::DurationWeeks,
    },
    /// Tutor session training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    TutorSession,
    /// Group class training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    GroupClass,
    /// Puppy kindergarten training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    PuppyKindergarten,
    /// Private lesson training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    PrivateLesson,
    /// Akc canine good citizen prep training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    AkcCanineGoodCitizenPrep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Required progress-recording depth for a training program.
pub enum ProgressTracking {
    /// Attendance only training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    AttendanceOnly,
    /// Session notes and milestones training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    SessionNotesAndMilestones,
    /// Trainer scorecard training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    TrainerScorecard,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Outcome claim vocabulary that must be backed by trainer evidence before customer-facing use.
pub enum Outcome {
    /// Basic manners training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    BasicManners,
    /// Reduced reactivity training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    ReducedReactivity,
    /// Canine good citizen readiness training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    CanineGoodCitizenReadiness,
    /// Owner handling plan training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    OwnerHandlingPlan,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Follow-up cadence that determines whether a progress/homework/re-enrollment message is due.
pub enum FollowUpCadence {
    /// No additional workflow gate is required.
    None,
    /// After each session training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    AfterEachSession,
    /// After program completion training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    AfterProgramCompletion,
    /// Thirty days after completion training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    ThirtyDaysAfterCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Source evidence attached to progress reports and outcome claims.
pub enum ProgressEvidence {
    /// Trainer note training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    TrainerNote {
        /// Source-derived evidence id carried by this training contract.
        evidence_id: EvidenceId,
        /// Source-derived note carried by this training contract.
        note: ProgressNote,
    },
    /// Milestone observed training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    MilestoneObserved {
        /// Source-derived evidence id carried by this training contract.
        evidence_id: EvidenceId,
        /// Source-derived milestone id carried by this training contract.
        milestone_id: curriculum::milestone::Id,
        /// Source-derived status carried by this training contract.
        status: curriculum::milestone::Status,
    },
    /// Session completed training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    SessionCompleted {
        /// Source-derived evidence id carried by this training contract.
        evidence_id: EvidenceId,
        /// Source-derived session id carried by this training contract.
        session_id: SessionId,
    },
    /// Outcome candidate training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    OutcomeCandidate {
        /// Source-derived evidence id carried by this training contract.
        evidence_id: EvidenceId,
        /// Source-derived outcome carried by this training contract.
        outcome: Outcome,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Approval state for progress reports before they become parent-facing summaries.
pub enum ApprovalState {
    /// Draft training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    Draft,
    /// Trainer approved training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    TrainerApproved {
        /// Source-derived trainer id carried by this training contract.
        trainer_id: StaffId,
    },
    /// Manager approved training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    ManagerApproved {
        /// Source-derived manager id carried by this training contract.
        manager_id: crate::entities::ManagerId,
    },
    /// Rejected training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    Rejected {
        /// Source-derived gate carried by this training contract.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Review state for outcome documentation before achievements are exposed to customers.
pub enum OutcomeReviewState {
    /// Draft training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    Draft,
    /// Source-derived trainer id carried by this training contract.
    TrainerApproved {
        /// Trainer id value carried by this review or workflow variant.
        trainer_id: StaffId,
    },
    /// Source-derived approved by carried by this training contract.
    ApprovedForMemberFacingUse {
        /// Approved by value carried by this review or workflow variant.
        approved_by: StaffId,
    },
    /// Source-derived gate carried by this training contract.
    Rejected {
        /// Gate value carried by this review or workflow variant.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Boundary for whether a training report or outcome may be shown to a pet parent.
pub enum MemberFacingBoundary {
    /// Internal only training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    InternalOnly,
    /// Source-derived gate carried by this training contract.
    DraftRequiresApproval {
        /// Gate value carried by this review or workflow variant.
        gate: policy::ReviewGate,
    },
    /// Approved for member facing use training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
    ApprovedForMemberFacingUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Remaining reusable session balance for a multi-session training package.
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
    /// Returns the remaining evidence recorded on this training contract.
    pub const fn remaining(self) -> Self {
        self
    }
    /// Returns the reserve one evidence recorded on this training contract.
    pub const fn reserve_one(self) -> Self {
        Self(self.0.saturating_sub(1))
    }
}

/// Trainer availability evaluation boundary for assignment drafting and waitlisting.
pub mod availability {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for capacity outcomes in training workflows.
    pub enum CapacityDecision {
        /// Available training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        Available,
        /// Unavailable training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        Unavailable,
        /// Estimate confidence is unknown and must be reviewed.
        UnknownRequiresReview,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    /// Assignment request combining enrollment readiness, trainer requirement, capacity evidence, and program details.
    pub struct Request {
        /// Source-derived enrollment id carried by this training contract.
        pub enrollment_id: enrollment::Id,
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// Source-derived program carried by this training contract.
        pub program: Program,
        /// Source-derived requirement carried by this training contract.
        pub requirement: trainer::Requirement,
        /// Source-derived capacity carried by this training contract.
        pub capacity: CapacityDecision,
        /// Source-derived readiness carried by this training contract.
        pub readiness: enrollment::Readiness,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Assignment decision showing whether to draft, waitlist, or require review before mutating provider schedules.
    pub enum Decision {
        /// Assignment drafted training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        AssignmentDrafted,
        /// Waitlist training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        Waitlist {
            /// Business reason staff should review before proceeding.
            reason: WaitlistReason,
            /// Source-derived gate carried by this training contract.
            gate: policy::ReviewGate,
        },
        /// Review required training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        ReviewRequired {
            /// Business reason staff should review before proceeding.
            reason: ReviewReason,
            /// Source-derived gate carried by this training contract.
            gate: policy::ReviewGate,
        },
    }

    impl Decision {
        /// Returns the provider mutation review gate recorded on this training contract.
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
    /// Decision vocabulary for waitlist reason in training workflows.
    pub enum WaitlistReason {
        /// Requested trainer unavailable training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        RequestedTrainerUnavailable,
        /// Capacity snapshot unavailable training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        CapacitySnapshotUnavailable,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for review reason in training workflows.
    pub enum ReviewReason {
        /// Enrollment not ready training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        EnrollmentNotReady,
        /// Capacity unknown training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
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

/// Progress-report boundary for evidence-backed trainer updates and parent-facing approval gates.
pub mod progress {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Training progress report carrying session evidence, milestones, and approval state.
    pub struct Report {
        /// Source-derived report id carried by this training contract.
        pub report_id: ProgressReportId,
        /// Source-derived enrollment id carried by this training contract.
        pub enrollment_id: enrollment::Id,
        /// Source-derived session ref carried by this training contract.
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
        /// Returns the milestones evidence recorded on this training contract.
        pub fn milestones(&self) -> &[curriculum::Progress] {
            &self.milestones
        }
        /// Returns the approval evidence recorded on this training contract.
        pub fn approval(&self) -> &ApprovalState {
            &self.approval
        }
        /// Returns the parent facing boundary evidence recorded on this training contract.
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
        /// Sets the report id value on this training builder.
        pub fn report_id(mut self, value: ProgressReportId) -> Self {
            self.report_id = Some(value);
            self
        }
        /// Sets the enrollment id value on this training builder.
        pub fn enrollment_id(mut self, value: enrollment::Id) -> Self {
            self.enrollment_id = Some(value);
            self
        }
        /// Sets the session ref value on this training builder.
        pub fn session_ref(mut self, value: SessionRef) -> Self {
            self.session_ref = Some(value);
            self
        }
        /// Returns the evidence recorded on this training contract.
        pub fn evidence(mut self, value: Vec<ProgressEvidence>) -> Self {
            self.evidence = value;
            self
        }
        /// Sets the milestones value on this training builder.
        pub fn milestones(mut self, value: Vec<curriculum::Progress>) -> Self {
            self.milestones = value;
            self
        }
        /// Sets the approval value on this training builder.
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

/// Outcome-documentation boundary for claims like manners readiness or CGC readiness.
pub mod outcome {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for claim status in training workflows.
    pub enum ClaimStatus {
        /// Achieved training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        Achieved,
        /// Readiness training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        Readiness,
        /// Deferred training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        Deferred,
        /// Not assessed training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        NotAssessed,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Evidence bundle used to promote an outcome claim into reviewed documentation.
    pub struct ClaimEvidence {
        /// Source-derived outcome carried by this training contract.
        pub outcome: Outcome,
        /// Source-derived status carried by this training contract.
        pub status: ClaimStatus,
        /// Source-derived evidence carried by this training contract.
        pub evidence: Vec<EvidenceId>,
        /// Source-derived milestones carried by this training contract.
        pub milestones: Vec<curriculum::milestone::Id>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Outcome claim whose achieved/readiness status cannot exist without supporting evidence.
    pub struct Claim {
        /// Source-derived outcome carried by this training contract.
        pub outcome: Outcome,
        /// Source-derived status carried by this training contract.
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
        /// Returns the evidence recorded on this training contract.
        pub fn evidence(&self) -> &[EvidenceId] {
            &self.evidence
        }
        /// Returns the milestones evidence recorded on this training contract.
        pub fn milestones(&self) -> &[curriculum::milestone::Id] {
            &self.milestones
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Training outcome documentation packet for customer/account history and manager review.
    pub struct Documentation {
        /// Source-derived documentation id carried by this training contract.
        pub documentation_id: OutcomeDocumentationId,
        /// Source-derived enrollment id carried by this training contract.
        pub enrollment_id: enrollment::Id,
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// Source-derived location id carried by this training contract.
        pub location_id: LocationId,
        claims: Vec<Claim>,
        review: OutcomeReviewState,
    }

    impl Documentation {
        /// Starts a validated builder for this training documentation or progress packet.
        pub fn builder() -> DocumentationBuilder {
            DocumentationBuilder::default()
        }
        /// Returns the claims evidence recorded on this training contract.
        pub fn claims(&self) -> &[Claim] {
            &self.claims
        }
        /// Returns the review evidence recorded on this training contract.
        pub fn review(&self) -> &OutcomeReviewState {
            &self.review
        }
        /// Returns the member facing boundary evidence recorded on this training contract.
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
        /// Sets the documentation id value on this training builder.
        pub fn documentation_id(mut self, value: OutcomeDocumentationId) -> Self {
            self.documentation_id = Some(value);
            self
        }
        /// Sets the enrollment id value on this training builder.
        pub fn enrollment_id(mut self, value: enrollment::Id) -> Self {
            self.enrollment_id = Some(value);
            self
        }
        /// Sets the pet id value on this training builder.
        pub fn pet_id(mut self, value: PetId) -> Self {
            self.pet_id = Some(value);
            self
        }
        /// Sets the location id value on this training builder.
        pub fn location_id(mut self, value: LocationId) -> Self {
            self.location_id = Some(value);
            self
        }
        /// Sets the claims value on this training builder.
        pub fn claims(mut self, value: Vec<Claim>) -> Self {
            self.claims = value;
            self
        }
        /// Sets the review value on this training builder.
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

/// Package and session-ledger boundary for reserving, consuming, and reconciling training sessions.
pub mod package {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Groomer-assignment policies used when booking grooming work.
    pub enum Policy {
        /// Pay per session training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        PayPerSession,
        /// Source-derived sessions carried by this training contract.
        MultiSessionPackage {
            /// Sessions value carried by this review or workflow variant.
            sessions: SessionCount,
        },
        /// Board and train bundle training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
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
    /// Decision vocabulary for ledger entries in training workflows.
    pub enum LedgerEntry {
        /// Source-derived sessions carried by this training contract.
        Purchased {
            /// Sessions value carried by this review or workflow variant.
            sessions: SessionCount,
        },
        /// Source-derived session id carried by this training contract.
        Reserved {
            /// Session id value carried by this review or workflow variant.
            session_id: SessionId,
        },
        /// Source-derived session id carried by this training contract.
        Consumed {
            /// Session id value carried by this review or workflow variant.
            session_id: SessionId,
        },
        /// Source-derived session id carried by this training contract.
        Released {
            /// Session id value carried by this review or workflow variant.
            session_id: SessionId,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Source-derived opening ledger used to create a reusable multi-session package ledger.
    pub struct OpeningLedger {
        /// Source-derived package id carried by this training contract.
        pub package_id: Id,
        /// Source-derived customer id carried by this training contract.
        pub customer_id: CustomerId,
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// Source-derived policy carried by this training contract.
        pub policy: Policy,
        /// Source-derived entries carried by this training contract.
        pub entries: Vec<LedgerEntry>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Training package ledger used to compute remaining reusable sessions without raw counters.
    pub struct Ledger {
        package_id: Id,
        /// Source-derived customer id carried by this training contract.
        pub customer_id: CustomerId,
        /// Pet receiving the grooming or care service.
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
        /// Returns the package id evidence recorded on this training contract.
        pub fn package_id(&self) -> &Id {
            &self.package_id
        }
        /// Returns the entries evidence recorded on this training contract.
        pub fn entries(&self) -> &[LedgerEntry] {
            &self.entries
        }
        /// Returns the balance evidence recorded on this training contract.
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
        /// Reserve next session training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        ReserveNextSession {
            /// Source-derived package id carried by this training contract.
            package_id: Id,
            /// Source-derived remaining after reservation carried by this training contract.
            remaining_after_reservation: SessionBalance,
        },
        /// No remaining sessions training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        NoRemainingSessions {
            /// Source-derived package id carried by this training contract.
            package_id: Id,
            /// Source-derived gate carried by this training contract.
            gate: policy::ReviewGate,
        },
        /// Reconciliation required training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        ReconciliationRequired {
            /// Source-derived package id carried by this training contract.
            package_id: Id,
            /// Source-derived gate carried by this training contract.
            gate: policy::ReviewGate,
        },
    }

    #[derive(Debug, Clone, Default)]
    /// Represents the usage policy concept as a typed training operational contract instead of a raw primitive.
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

/// Follow-up boundary for progress updates, homework coaching, completion summaries, and re-enrollment prompts.
pub mod follow_up {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for trigger in training workflows.
    pub enum Trigger {
        /// Source-derived session id carried by this training contract.
        SessionCompleted {
            /// Session id value carried by this review or workflow variant.
            session_id: SessionId,
        },
        /// Source-derived enrollment id carried by this training contract.
        ProgramCompleted {
            /// Enrollment id value carried by this review or workflow variant.
            enrollment_id: enrollment::Id,
        },
        /// Source-derived enrollment id carried by this training contract.
        LaterCadenceCheckpoint {
            /// Enrollment id value carried by this review or workflow variant.
            enrollment_id: enrollment::Id,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for purpose in training workflows.
    pub enum Purpose {
        /// Progress update training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        ProgressUpdate,
        /// Homework coaching training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        HomeworkCoaching,
        /// Program completion summary training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        ProgramCompletionSummary,
        /// Re enrollment prompt training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        ReEnrollmentPrompt,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for evidence readiness in training workflows.
    pub enum EvidenceReadiness {
        /// Progress and homework ready training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        ProgressAndHomeworkReady,
        /// Needs trainer evidence training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        NeedsTrainerEvidence,
        /// Outcome disputed or ambiguous training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        OutcomeDisputedOrAmbiguous,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for state in training workflows.
    pub enum State {
        /// Not due training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        NotDue,
        /// Source-derived gate carried by this training contract.
        TrainerEvidenceRequired {
            /// Gate value carried by this review or workflow variant.
            gate: policy::ReviewGate,
        },
        /// Source-derived gate carried by this training contract.
        DraftRequiresApproval {
            /// Gate value carried by this review or workflow variant.
            gate: policy::ReviewGate,
        },
        /// Suppressed training operational signal for enrollment, curriculum, progress, package, or follow-up handling.
        Suppressed,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Follow-up plan that separates due/not-due state from approval-gated customer messaging.
    pub struct Plan {
        /// Source-derived trigger carried by this training contract.
        pub trigger: Trigger,
        purpose: Purpose,
        state: State,
    }

    impl Plan {
        /// Returns the purpose evidence recorded on this training contract.
        pub const fn purpose(&self) -> Purpose {
            self.purpose
        }
        /// Returns the state evidence recorded on this training contract.
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
/// Location training contract tying program duration, curriculum, progress depth, outcomes, trainer availability, package policy, and follow-up cadence together.
pub struct Contract {
    /// Source-derived program duration carried by this training contract.
    pub program_duration: program::Duration,
    #[builder(default)]
    /// Source-derived curriculum carried by this training contract.
    pub curriculum: Vec<curriculum::Unit>,
    /// Source-derived progress carried by this training contract.
    pub progress: ProgressTracking,
    #[builder(default)]
    /// Source-derived outcomes carried by this training contract.
    pub outcomes: Vec<Outcome>,
    /// Source-derived trainer availability carried by this training contract.
    pub trainer_availability: trainer::Availability,
    /// Source-derived package carried by this training contract.
    pub package: package::Policy,
    /// Source-derived follow up carried by this training contract.
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
    /// Reports whether the training contract includes the requested outcome claim.
    pub fn has_outcome(&self, outcome: &Outcome) -> bool {
        self.outcomes.contains(outcome)
    }
    /// Builds a representative PetSuites-style training contract for docs/tests without claiming it is live policy.
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
