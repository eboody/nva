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
pub struct Evidence {
    pub pet_id: PetId,
    pub species: entities::Species,
    pub service: ServiceVariant,
    pub temperament: TemperamentAssessmentFreshness,
    pub vaccines: VaccineReadiness,
    pub spay_neuter: entities::SpayNeuterStatus,
    pub incident: incident::Restriction,
    pub staff_coverage: coverage::Decision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemperamentAssessmentFreshness {
    Current,
    Stale,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VaccineReadiness {
    Current,
    MissingProof,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupPlayDecision {
    Eligible {
        basis: EligibleBasis,
    },
    NeedsStaffReview {
        reason: ReviewReason,
        gate: policy::ReviewGate,
    },
    Ineligible {
        reason: DenialReason,
    },
    TemporarilySuspended {
        pet_id: PetId,
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EligibleBasis {
    CurrentEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewReason {
    MissingCurrentTemperamentAssessment,
    VaccineProofRequiresReview,
    SpayNeuterStatusRequiresReview,
    StaffCoverageRequiresReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DenialReason {
    ServiceUnavailableForSpeciesOrCareMode,
}

#[derive(Debug, Clone, Default)]
pub struct GroupPlayPolicy;

impl GroupPlayPolicy {
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
