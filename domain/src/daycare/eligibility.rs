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
/// Typed evidence domain value that keeps raw primitives out of daycare workflows.
pub struct Evidence {
    /// Pet receiving the grooming or care service.
    pub pet_id: PetId,
    /// Species fact promoted into this daycare contract.
    pub species: entities::Species,
    /// Requested service that drives scheduling and labor estimates.
    pub service: ServiceVariant,
    /// Temperament fact promoted into this daycare contract.
    pub temperament: TemperamentAssessmentFreshness,
    /// Vaccines fact promoted into this daycare contract.
    pub vaccines: VaccineReadiness,
    /// Spay neuter fact promoted into this daycare contract.
    pub spay_neuter: entities::SpayNeuterStatus,
    /// Incident fact promoted into this daycare contract.
    pub incident: incident::Restriction,
    /// Staff coverage fact promoted into this daycare contract.
    pub staff_coverage: coverage::Decision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for temperament assessment freshness decisions in daycare workflows.
pub enum TemperamentAssessmentFreshness {
    /// Current daycare attendance, eligibility, coverage, or package signal.
    Current,
    /// Stale daycare attendance, eligibility, coverage, or package signal.
    Stale,
    /// Missing daycare attendance, eligibility, coverage, or package signal.
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for vaccine readiness decisions in daycare workflows.
pub enum VaccineReadiness {
    /// Current daycare attendance, eligibility, coverage, or package signal.
    Current,
    /// Missing proof daycare attendance, eligibility, coverage, or package signal.
    MissingProof,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for group play decision decisions in daycare workflows.
pub enum GroupPlayDecision {
    /// Eligible daycare attendance, eligibility, coverage, or package signal.
    Eligible {
        /// Basis fact promoted into this daycare contract.
        basis: EligibleBasis,
    },
    /// Needs staff review daycare attendance, eligibility, coverage, or package signal.
    NeedsStaffReview {
        /// Business reason staff should review before proceeding.
        reason: ReviewReason,
        /// Gate fact promoted into this daycare contract.
        gate: policy::ReviewGate,
    },
    /// Ineligible daycare attendance, eligibility, coverage, or package signal.
    Ineligible {
        /// Business reason staff should review before proceeding.
        reason: DenialReason,
    },
    /// Temporarily suspended daycare attendance, eligibility, coverage, or package signal.
    TemporarilySuspended {
        /// Pet receiving the grooming or care service.
        pet_id: PetId,
        /// Gate fact promoted into this daycare contract.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for eligible basis decisions in daycare workflows.
pub enum EligibleBasis {
    /// Current evidence daycare attendance, eligibility, coverage, or package signal.
    CurrentEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for review reason decisions in daycare workflows.
pub enum ReviewReason {
    /// Missing current temperament assessment daycare attendance, eligibility, coverage, or package signal.
    MissingCurrentTemperamentAssessment,
    /// Vaccine proof requires review daycare attendance, eligibility, coverage, or package signal.
    VaccineProofRequiresReview,
    /// Spay neuter status requires review daycare attendance, eligibility, coverage, or package signal.
    SpayNeuterStatusRequiresReview,
    /// Staff coverage requires review daycare attendance, eligibility, coverage, or package signal.
    StaffCoverageRequiresReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for denial reason decisions in daycare workflows.
pub enum DenialReason {
    /// Service unavailable for species or care mode daycare attendance, eligibility, coverage, or package signal.
    ServiceUnavailableForSpeciesOrCareMode,
}

#[derive(Debug, Clone, Default)]
/// Typed group play policy domain value that keeps raw primitives out of daycare workflows.
pub struct GroupPlayPolicy;

impl GroupPlayPolicy {
    /// Returns the evaluate for this daycare value.
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
