//! Training service-line rules for enrollment readiness, trainer capacity, curriculum progress, package sessions, and parent-facing follow-up.
//!
//! Operator summary: training helps resort staff decide which program requests can be drafted, which trainer/package/progress/outcome queues need review, and which parent follow-up must stay internal. It can reduce repeated trainer-capacity checks, package/session reconciliation, evidence lookup, and graduation or re-enrollment follow-up by producing typed assignment, report, outcome, package, and follow-up decisions.
//!
//! This module is not permission for live automation. It does not assign trainers in a provider system, move waitlists, send customer messages, adjust packages or payments, or publish outcome/graduation claims. Source facts remain authoritative in `domain::entities`, `domain::care`, `domain::temperament`, `domain::payment`, `domain::policy`, `storage::service_line::training`, and provider/integration mappings; training values carry review gates so trainer, manager, payment, behavior/care, and member-facing approval boundaries protect pets, customers, and staff.

use bon::Builder;
use nonempty::NonEmpty;
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
pub mod program;

/// Enrollment readiness gate for deciding whether a training assignment can be drafted.
pub mod enrollment;

/// Curriculum vocabulary for program units, milestones, and evidence-backed progress tracking.
pub mod curriculum;

/// Trainer assignment policy for matching programs to certified, named, or program-qualified trainers.
pub mod trainer;

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

mod error;
pub use error::{Error, Result};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Remaining reusable session balance for a multi-session training package.
pub struct SessionBalance(u16);

/// Progress-report workflow for evidence-backed trainer updates and parent-facing approval gates.
pub mod progress;

/// Outcome-documentation workflow for claims like manners readiness or CGC readiness.
pub mod outcome;

/// Package and session-ledger workflow for reserving, consuming, and reconciling training sessions.
pub mod package;

/// Follow-up workflow for progress updates, homework coaching, completion summaries, and re-enrollment prompts.
pub mod follow_up;

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
}
