//! Daycare group-play eligibility policy for source-grounded staff review.
//!
//! ```
//! use domain::{daycare, entities, policy};
//! use uuid::Uuid;
//!
//! let evidence = daycare::eligibility::Evidence::builder()
//!     .pet_id(entities::PetId(Uuid::nil()))
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
use crate::{entities, policy};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Source-derived evidence used to decide whether a pet may enter daycare group play.
pub struct Evidence {
    /// Pet whose daycare eligibility is being evaluated.
    pub pet_id: PetId,
    /// Pet species used to prevent group-play rules from applying to unsupported care modes.
    pub species: entities::Species,
    /// Requested service that drives scheduling and labor estimates.
    pub service: ServiceVariant,
    /// Freshness of temperament assessment required for safe group assignment.
    pub temperament: TemperamentAssessmentFreshness,
    /// Vaccine proof readiness from source records or staff review.
    pub vaccines: VaccineReadiness,
    /// Spay/neuter status used for group-play policy review.
    pub spay_neuter: entities::SpayNeuterStatus,
    /// Active incident restriction that may suspend group play.
    pub incident: incident::Restriction,
    /// Current staffing coverage decision used before admitting a pet to group play.
    pub staff_coverage: coverage::Decision,
}

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

impl GroupPlayPolicy {
    /// Evaluates species, service, temperament, vaccine, spay/neuter, incident, and coverage gates for group play.
    pub fn evaluate(&self, evidence: &Evidence) -> GroupPlayDecision {
        if let incident::Restriction::SuspendedPendingManagerReview { pet_id } = evidence.incident {
            return GroupPlayDecision::TemporarilySuspended {
                pet_id,
                gate: policy::ReviewGate::ManagerApproval,
            };
        }
        if !matches!(evidence.species, entities::Species::Dog)
            || !matches!(
                evidence.service.care_mode(),
                CareMode::DogGroupPlay | CareMode::DogHybridPlayAndRoom
            )
        {
            return GroupPlayDecision::Ineligible {
                reason: DenialReason::ServiceUnavailableForSpeciesOrCareMode,
            };
        }
        if !matches!(
            evidence.temperament,
            TemperamentAssessmentFreshness::Current
        ) {
            return GroupPlayDecision::NeedsStaffReview {
                reason: ReviewReason::MissingCurrentTemperamentAssessment,
                gate: policy::ReviewGate::BehaviorReview,
            };
        }
        if !matches!(evidence.vaccines, VaccineReadiness::Current) {
            return GroupPlayDecision::NeedsStaffReview {
                reason: ReviewReason::VaccineProofRequiresReview,
                gate: policy::ReviewGate::MedicalDocumentReview,
            };
        }
        if matches!(
            evidence.spay_neuter,
            entities::SpayNeuterStatus::Intact | entities::SpayNeuterStatus::Unknown
        ) {
            return GroupPlayDecision::NeedsStaffReview {
                reason: ReviewReason::SpayNeuterStatusRequiresReview,
                gate: policy::ReviewGate::BehaviorReview,
            };
        }
        if !matches!(evidence.staff_coverage, coverage::Decision::Sufficient) {
            return GroupPlayDecision::NeedsStaffReview {
                reason: ReviewReason::StaffCoverageRequiresReview,
                gate: policy::ReviewGate::ManagerApproval,
            };
        }
        GroupPlayDecision::Eligible {
            basis: EligibleBasis::CurrentEvidence,
        }
    }
}
