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

impl Requirement {}
