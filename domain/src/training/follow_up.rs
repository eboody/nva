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

impl Policy {}
