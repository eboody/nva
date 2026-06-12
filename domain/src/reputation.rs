//! Canonical domain contracts for reputation-review triage.
//!
//! Review signals and escalation decisions cut across service lines;
//! `operations` retains legacy compatibility re-exports.

use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use crate::entities::LocationId;
use crate::operations::OperationalObservation;

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
pub struct ReviewPlatformName(String);

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
pub struct ReviewId(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReputationSignal {
    pub location_id: LocationId,
    pub platform: ReviewPlatformName,
    pub review_id: ReviewId,
    pub sentiment: ReviewSentiment,
    pub themes: Vec<ReviewTheme>,
    pub escalation: ReviewEscalation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewSentiment {
    Positive,
    Neutral,
    Negative,
    Mixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewTheme {
    StaffExperience,
    Cleanliness,
    PricingOrBilling,
    BookingExperience,
    GroomingOutcome,
    PetInjuryOrSafety,
    Communication,
    WaitTime,
    Other(OperationalObservation),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewEscalation {
    None,
    DraftPublicResponse,
    ManagerReviewRequired,
    SafetyOrLegalReviewRequired,
}
