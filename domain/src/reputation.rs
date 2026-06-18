//! Canonical domain contracts for reputation-review triage.
//!
//! Review signals and escalation decisions cut across service lines;
//! `operations` retains deprecated legacy compatibility re-exports.

use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use crate::entities::LocationId;
use crate::operations;

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
pub struct PlatformName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
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
/// Typed signal domain value that keeps raw primitives out of reputation workflows.
pub struct Signal {
    /// Location id fact promoted into this reputation contract.
    pub location_id: LocationId,
    /// Platform fact promoted into this reputation contract.
    pub platform: PlatformName,
    /// Review id fact promoted into this reputation contract.
    pub review_id: Id,
    /// Sentiment fact promoted into this reputation contract.
    pub sentiment: Sentiment,
    /// Themes fact promoted into this reputation contract.
    pub themes: Vec<Theme>,
    /// Escalation fact promoted into this reputation contract.
    pub escalation: Escalation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for sentiment decisions in reputation workflows.
pub enum Sentiment {
    /// Positive review sentiment, topic, or response action for guest-experience follow-up.
    Positive,
    /// Neutral review sentiment, topic, or response action for guest-experience follow-up.
    Neutral,
    /// Negative review sentiment, topic, or response action for guest-experience follow-up.
    Negative,
    /// Mixed review sentiment, topic, or response action for guest-experience follow-up.
    Mixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for theme decisions in reputation workflows.
pub enum Theme {
    /// Staff experience review sentiment, topic, or response action for guest-experience follow-up.
    StaffExperience,
    /// Cleanliness review sentiment, topic, or response action for guest-experience follow-up.
    Cleanliness,
    /// Pricing or billing review sentiment, topic, or response action for guest-experience follow-up.
    PricingOrBilling,
    /// Booking experience review sentiment, topic, or response action for guest-experience follow-up.
    BookingExperience,
    /// Grooming outcome review sentiment, topic, or response action for guest-experience follow-up.
    GroomingOutcome,
    /// Pet injury or safety review sentiment, topic, or response action for guest-experience follow-up.
    PetInjuryOrSafety,
    /// Communication review sentiment, topic, or response action for guest-experience follow-up.
    Communication,
    /// Wait time review sentiment, topic, or response action for guest-experience follow-up.
    WaitTime,
    /// Non-dog, non-cat pet handled by exception policy.
    Other(operations::operational::Observation),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for escalation decisions in reputation workflows.
pub enum Escalation {
    /// No additional workflow gate is required.
    None,
    /// Draft public response review sentiment, topic, or response action for guest-experience follow-up.
    DraftPublicResponse,
    /// Manager review required review sentiment, topic, or response action for guest-experience follow-up.
    ManagerReviewRequired,
    /// Safety or legal review required review sentiment, topic, or response action for guest-experience follow-up.
    SafetyOrLegalReviewRequired,
}
