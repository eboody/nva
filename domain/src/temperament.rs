//! Temperament and behavior-observation contracts for daycare and care safety.
//!
//! These values promote staff/source notes into validated, redacted domain signals before
//! they influence group-play eligibility, daily-brief watchlists, staffing plans, or
//! customer communication. Review evidence remains explicit so automation supports staff
//! judgment instead of overriding safety policy.

use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::fmt;

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 1000),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
/// Redacted staff note containing temperament evidence for review workflows.
pub struct StaffNote(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 80),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
/// Provider-specific behavior label retained when no first-class variant exists.
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
/// Domain vocabulary for group play observation decisions in temperament workflows.
pub enum GroupPlayObservation {
    #[default]
    /// Not yet observed temperament signal for playgroup and handling decisions.
    NotYetObserved,
    /// Comfortable in observed group temperament signal for playgroup and handling decisions.
    ComfortableInObservedGroup,
    /// Stressed in group setting temperament signal for playgroup and handling decisions.
    StressedInGroupSetting,
    /// Needs intro assessment temperament signal for playgroup and handling decisions.
    NeedsIntroAssessment,
}

impl GroupPlayObservation {
    /// Returns whether group-play status requires staff evaluation before assignment.
    pub fn needs_staff_evaluation(self) -> bool {
        matches!(self, Self::NeedsIntroAssessment)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Domain vocabulary for people orientation decisions in temperament workflows.
pub enum PeopleOrientation {
    /// People seeking temperament signal for playgroup and handling decisions.
    PeopleSeeking,
    /// Neutral temperament signal for playgroup and handling decisions.
    Neutral,
    /// People avoidant temperament signal for playgroup and handling decisions.
    PeopleAvoidant,
    #[default]
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Domain vocabulary for rating decisions in temperament workflows.
pub enum Rating {
    /// Easygoing temperament signal for playgroup and handling decisions.
    Easygoing,
    /// Moderate temperament signal for playgroup and handling decisions.
    Moderate,
    /// Needs structure temperament signal for playgroup and handling decisions.
    NeedsStructure,
    /// Review required temperament signal for playgroup and handling decisions.
    ReviewRequired,
    #[default]
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for behavior observation decisions in temperament workflows.
pub enum BehaviorObservation {
    /// Anxiety temperament signal for playgroup and handling decisions.
    Anxiety,
    /// Bite history temperament signal for playgroup and handling decisions.
    BiteHistory,
    /// Dog selective temperament signal for playgroup and handling decisions.
    DogSelective,
    /// Human selective temperament signal for playgroup and handling decisions.
    HumanSelective,
    /// Escape risk temperament signal for playgroup and handling decisions.
    EscapeRisk,
    /// Food guarding temperament signal for playgroup and handling decisions.
    FoodGuarding,
    /// Requires manager review temperament signal for playgroup and handling decisions.
    RequiresManagerReview,
    /// Extension point for provider-specific values not modeled directly.
    Extension(BehaviorObservationLabel),
}

impl BehaviorObservation {
    /// Returns whether the observation should create behavior-review evidence.
    pub fn indicates_behavior_review_evidence(&self) -> bool {
        matches!(
            self,
            Self::BiteHistory | Self::RequiresManagerReview | Self::HumanSelective
        )
    }
}
