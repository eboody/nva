//! Identity matching contracts for source-to-domain reconciliation.
//!
//! Identity confidence and match results preserve candidate/ambiguous source evidence before a
//! workflow treats a source record as a customer, pet, or household fact. Low or ambiguous matches
//! should stay review-visible instead of silently granting automation authority.

use serde::{Deserialize, Serialize};

use crate::{entities, money};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Match confidence band for source-to-domain identity promotion.
pub enum Confidence {
    /// Low confidence; human review should be visible.
    Low,
    /// Medium confidence; usable only with caveats or review.
    Medium,
    /// High confidence from stable source evidence.
    High,
}

impl Confidence {
    /// Classifies a validated basis-points confidence score into a review-facing band.
    pub const fn from_basis_points(value: money::BasisPoints) -> Self {
        match value.get() {
            8_000..=10_000 => Self::High,
            5_000..=7_999 => Self::Medium,
            _ => Self::Low,
        }
    }

    /// Returns the review-facing confidence band.
    pub const fn band(self) -> Self {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Customer identity resolution result for lead, CRM, and retention workflows.
pub enum Match {
    /// No matching customer was found.
    None,
    /// One candidate customer exists with explicit confidence.
    Candidate {
        /// Candidate customer id.
        customer_id: entities::CustomerId,
        /// Confidence in the match.
        confidence: Confidence,
    },
    /// Multiple plausible customers require staff review.
    Ambiguous {
        /// Candidate customer ids.
        candidates: Vec<entities::CustomerId>,
    },
}
