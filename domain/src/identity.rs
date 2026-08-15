//! Identity matching contracts for source-to-domain reconciliation.
//!
//! Identity confidence and match results preserve candidate/ambiguous source evidence before a
//! workflow treats a source record as a customer, pet, or household fact. Low or ambiguous matches
//! should stay review-visible instead of silently granting automation authority.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashSet;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
/// Minimum-two collection of distinct customer candidates that truthfully represents ambiguity.
pub struct AmbiguousCandidates(Vec<entities::CustomerId>);

impl AmbiguousCandidates {
    /// Validates that ambiguity contains at least two unique customer identities.
    pub fn try_new(candidates: Vec<entities::CustomerId>) -> Result<Self, Error> {
        if candidates.len() < 2 {
            return Err(Error::AtLeastTwoCandidatesRequired);
        }
        let unique: HashSet<_> = candidates.iter().copied().collect();
        if unique.len() != candidates.len() {
            return Err(Error::CandidatesMustBeUnique);
        }
        Ok(Self(candidates))
    }

    /// Returns the distinct candidates in source ranking order.
    pub fn as_slice(&self) -> &[entities::CustomerId] {
        &self.0
    }
}

impl<'de> Deserialize<'de> for AmbiguousCandidates {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(Vec::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Identity-resolution invariant failures.
pub enum Error {
    #[error("ambiguous identity requires at least two candidates")]
    /// Zero or one candidate cannot represent ambiguity.
    AtLeastTwoCandidatesRequired,
    #[error("ambiguous identity candidates must be unique")]
    /// Duplicate identities do not create distinct alternatives.
    CandidatesMustBeUnique,
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
        /// At least two distinct candidate customer ids, preserved in source ranking order.
        candidates: AmbiguousCandidates,
    },
}

impl Match {
    /// Constructs an ambiguous result only from at least two distinct customer identities.
    pub fn ambiguous(candidates: Vec<entities::CustomerId>) -> Result<Self, Error> {
        Ok(Self::Ambiguous {
            candidates: AmbiguousCandidates::try_new(candidates)?,
        })
    }
}
