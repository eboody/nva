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

impl Progress {}
