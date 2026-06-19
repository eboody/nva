//! Temperament and behavior-observation contracts for daycare and care safety.
//!
//! ## Operator-summary
//!
//! This module supports behavior-review and play-assignment queues by naming group-play
//! observations, people orientation, overall rating, and specific behavior evidence such
//! as bite history, human selectivity, escape risk, or food guarding. It can reduce labor
//! by turning staff/provider notes into a consistent watchlist for daycare eligibility,
//! staffing plans, daily briefs, and customer-safe follow-up drafts.
//!
//! It must not automate live group assignment, behavior determinations, training advice,
//! customer blame, or safety exceptions. Authoritative facts remain the reviewed staff
//! observations, source notes, incident history, location play policy, and approval records;
//! these values only preserve redacted signals for downstream review. Review gates protect
//! pets, customers, and staff by routing stale, missing, manager-review, bite-history, or
//! selectivity evidence to behavior/manager review before it changes play access or
//! customer-visible messaging.
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
/// Reviewed group-play status that gates daycare assignment and intro-assessment work.
pub enum GroupPlayObservation {
    #[default]
    /// No reviewed group-play observation exists yet, so staff need fresh behavior evidence before assignment.
    NotYetObserved,
    /// Staff have observed comfortable group play, supporting normal playgroup consideration.
    ComfortableInObservedGroup,
    /// Group setting caused stress, so care staff should consider quieter handling or alternate placement.
    StressedInGroupSetting,
    /// Pet needs an intro assessment before group play can be offered or promised.
    NeedsIntroAssessment,
}

impl GroupPlayObservation {
    /// Returns whether group-play status requires staff evaluation before assignment.
    pub fn needs_staff_evaluation(self) -> bool {
        matches!(self, Self::NeedsIntroAssessment)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Staff-observed people orientation used to plan handling, staffing, and customer follow-up.
pub enum PeopleOrientation {
    /// Pet actively seeks human interaction, which helps staff plan handling and enrichment.
    PeopleSeeking,
    /// Pet shows no strong people-seeking or avoidant signal in reviewed notes.
    Neutral,
    /// Pet avoids people, so staff should use slower handling and review before customer-facing assurances.
    PeopleAvoidant,
    #[default]
    /// People-orientation evidence is missing or unclear and should not drive handling policy by itself.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Overall temperament rating used to rank play eligibility and behavior-review attention.
pub enum Rating {
    /// Reviewed temperament is easygoing enough for normal handling unless other watchlist evidence exists.
    Easygoing,
    /// Temperament needs ordinary staff awareness but does not by itself block care workflows.
    Moderate,
    /// Pet benefits from structured handling or play rules that staff should review before assignment.
    NeedsStructure,
    /// Temperament evidence requires behavior or manager review before it changes play access.
    ReviewRequired,
    #[default]
    /// People-orientation evidence is missing or unclear and should not drive handling policy by itself.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Specific behavior evidence that can block play access or require manager review.
pub enum BehaviorObservation {
    /// Anxiety observed in source notes, signaling care attention and possible lower-stimulation handling.
    Anxiety,
    /// Bite history is safety-sensitive evidence that must route to behavior/manager review.
    BiteHistory,
    /// Dog-selective behavior means group pairing needs staff judgment instead of automatic assignment.
    DogSelective,
    /// Human-selective behavior affects handling plans and should create review evidence.
    HumanSelective,
    /// Escape-risk evidence alerts staff to containment and handoff precautions.
    EscapeRisk,
    /// Food-guarding evidence affects feeding, enrichment, and group-care supervision.
    FoodGuarding,
    /// Source notes explicitly require manager review before changing access or messaging.
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
