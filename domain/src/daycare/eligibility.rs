//! Daycare group-play eligibility policy for source-grounded staff review.
//!
//! ## Operator-summary
//!
//! This module supports the daycare group-play eligibility queue: it combines species,
//! requested care mode, current temperament assessment, vaccine readiness, spay/neuter
//! status, active incident restriction, and staffing coverage before a pet enters group play.
//! It can reduce labor by producing a deterministic staff-review reason instead of asking
//! front-desk or play-yard staff to manually reconcile every source note and policy gate.
//!
//! It must not automate live admission to group play, vaccine acceptance, behavior clearance,
//! staffing overrides, or customer promises. Authoritative facts remain the pet profile,
//! reviewed temperament/vaccine records, local play policy, incident disposition, and
//! coverage decision. Review gates protect pets, customers, and staff by routing missing
//! temperament, uncertain vaccine proof, spay/neuter concerns, insufficient staffing, or
//! incident suspension to behavior, medical-document, manager, or staff review before use.
//!
//! ```
//! use domain::{daycare, entities, policy};
//! use uuid::Uuid;
//!
//! let evidence = daycare::eligibility::Evidence::builder()
//!     .pet_id(entities::PetId::new(uuid::Uuid::from_u128(1)))
//!     .species(entities::Species::Dog)
//!     .service(daycare::ServiceVariant::AllDayPlay)
//!     .temperament(daycare::eligibility::TemperamentAssessmentFreshness::Missing)
//!     .vaccines(daycare::eligibility::VaccineReadiness::Current)
//!     .spay_neuter(entities::SpayNeuterStatus::Neutered)
//!     .incident(daycare::incident::Restriction::None)
//!     .staff_coverage(daycare::coverage::Decision::Sufficient)
//!     .build();
//!
//! assert_eq!(
//!     daycare::eligibility::GroupPlayPolicy.evaluate(&evidence),
//!     daycare::eligibility::GroupPlayDecision::NeedsStaffReview {
//!         reason: daycare::eligibility::ReviewReason::MissingCurrentTemperamentAssessment,
//!         gate: policy::ReviewGate::BehaviorReview,
//!     },
//! );
//! ```

use super::*;
use crate::policy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Freshness state of the temperament assessment required for daycare group play.
pub enum TemperamentAssessmentFreshness {
    /// Source evidence is current and can be used without additional review.
    Current,
    /// Evidence exists but is stale, so staff must review before group play.
    Stale,
    /// Required source evidence is missing and must be collected or reviewed.
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Vaccine-proof readiness state for daycare eligibility decisions.
pub enum VaccineReadiness {
    /// Source evidence is current and can be used without additional review.
    Current,
    /// Vaccine documentation is absent and requires medical-document review.
    MissingProof,
    /// Vaccine status could not be mapped confidently and must be reviewed.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Eligibility outcome for admitting a pet to daycare group play.
pub enum GroupPlayDecision {
    /// Pet has sufficient current evidence for group-play admission.
    Eligible {
        /// Evidence basis that justified the eligible outcome.
        basis: EligibleBasis,
    },
    /// Staff must review missing, stale, or sensitive evidence before group play.
    NeedsStaffReview {
        /// Operational reason the pet cannot be auto-cleared for group play.
        reason: ReviewReason,
        /// Human review gate required to clear the eligibility issue.
        gate: policy::ReviewGate,
    },
    /// The requested service or care mode is not eligible for group play.
    Ineligible {
        /// Operational reason the pet cannot be auto-cleared for group play.
        reason: DenialReason,
    },
    /// An incident restriction suspends group play pending manager review.
    TemporarilySuspended {
        /// Pet whose daycare eligibility is being evaluated.
        pet_id: PetId,
        /// Human review gate required to clear the eligibility issue.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Evidence basis proving a pet is eligible for group play.
pub enum EligibleBasis {
    /// Current source evidence satisfies all configured group-play gates.
    CurrentEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons daycare group-play eligibility requires staff review.
pub enum ReviewReason {
    /// Temperament assessment is missing or stale and requires behavior review.
    MissingCurrentTemperamentAssessment,
    /// Vaccine proof is missing or uncertain and requires medical-document review.
    VaccineProofRequiresReview,
    /// Spay/neuter status requires staff review before group play.
    SpayNeuterStatusRequiresReview,
    /// Staffing coverage is insufficient or unknown for safe group play.
    StaffCoverageRequiresReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons a pet is not eligible for the requested group-play care mode.
pub enum DenialReason {
    /// Requested service or care mode does not support group play for this species.
    ServiceUnavailableForSpeciesOrCareMode,
}

#[derive(Debug, Clone, Default)]
/// Deterministic policy that converts source evidence into daycare group-play eligibility.
pub struct GroupPlayPolicy;

impl GroupPlayPolicy {}
