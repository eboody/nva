use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::fmt;

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 1000),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct StaffNote(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 80),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct BehaviorObservationLabel(String);

impl fmt::Debug for StaffNote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("StaffNote(<redacted>)")
    }
}

impl fmt::Debug for BehaviorObservationLabel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BehaviorObservationLabel(<redacted>)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GroupPlayObservation {
    #[default]
    NotYetObserved,
    ComfortableInObservedGroup,
    StressedInGroupSetting,
    NeedsIntroAssessment,
}

impl GroupPlayObservation {
    pub fn needs_staff_evaluation(self) -> bool {
        matches!(self, Self::NeedsIntroAssessment)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PeopleOrientation {
    PeopleSeeking,
    Neutral,
    PeopleAvoidant,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Rating {
    Easygoing,
    Moderate,
    NeedsStructure,
    ReviewRequired,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BehaviorObservation {
    Anxiety,
    BiteHistory,
    DogSelective,
    HumanSelective,
    EscapeRisk,
    FoodGuarding,
    RequiresManagerReview,
    Extension(BehaviorObservationLabel),
}

impl BehaviorObservation {
    pub fn indicates_behavior_review_evidence(&self) -> bool {
        matches!(
            self,
            Self::BiteHistory | Self::RequiresManagerReview | Self::HumanSelective
        )
    }
}
