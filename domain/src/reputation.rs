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
pub struct Signal {
    pub location_id: LocationId,
    pub platform: PlatformName,
    pub review_id: Id,
    pub sentiment: Sentiment,
    pub themes: Vec<Theme>,
    pub escalation: Escalation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sentiment {
    Positive,
    Neutral,
    Negative,
    Mixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    StaffExperience,
    Cleanliness,
    PricingOrBilling,
    BookingExperience,
    GroomingOutcome,
    PetInjuryOrSafety,
    Communication,
    WaitTime,
    Other(operations::operational::Observation),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Escalation {
    None,
    DraftPublicResponse,
    ManagerReviewRequired,
    SafetyOrLegalReviewRequired,
}
