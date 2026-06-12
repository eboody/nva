use bon::Builder;
use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::entities::{CustomerId, LocationId, PetId};
use crate::policy;

use super::inventory::InventoryAvailability;
use super::product::Product;

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 500),
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
pub struct RecommendationRationale(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 500),
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
pub struct CustomerSafeCopy(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rule {
    None,
    AnxietySupportAfterBoarding,
    DietSupportAfterBoarding,
    CoatCareAfterGrooming,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Candidate {
    pub customer_id: CustomerId,
    pub pet_id: PetId,
    pub location_id: LocationId,
    pub product: Product,
    pub reason: Reason,
    pub rationale: RecommendationRationale,
    pub care_sensitivity: CareSensitivity,
    pub inventory: InventoryAvailability,
    pub customer_preference: CustomerRetailPreference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reason {
    AnxietyOrStressSupport,
    BoardingDietContinuity,
    CoatOrSkinCareAfterGrooming,
    PriorPurchaseReplenishment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CareSensitivity {
    NoKnownCareConflict,
    SupplementOrDietReviewRequired,
    CarePlanConflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomerRetailPreference {
    AllowsRetailRecommendations,
    OptedOut,
    UnknownRequiresReview,
}

#[derive(Debug, Clone, Default)]
pub struct Policy;

impl Policy {
    pub fn evaluate(&self, candidate: &Candidate) -> Decision {
        if matches!(
            candidate.customer_preference,
            CustomerRetailPreference::OptedOut
        ) {
            return Decision::Suppressed {
                reason: SuppressionReason::CustomerOptedOut,
            };
        }
        if !matches!(candidate.inventory, InventoryAvailability::Available) {
            return Decision::Suppressed {
                reason: SuppressionReason::InventoryUnavailable,
            };
        }
        match candidate.care_sensitivity {
            CareSensitivity::NoKnownCareConflict => Decision::DraftInternalCandidate,
            CareSensitivity::SupplementOrDietReviewRequired => Decision::StaffReviewRequired {
                reason: ReviewReason::CareSensitiveProduct,
                gate: policy::ReviewGate::MedicalDocumentReview,
            },
            CareSensitivity::CarePlanConflict => Decision::ManagerReviewRequired {
                reason: ReviewReason::CarePlanConflict,
                gate: policy::ReviewGate::ManagerApproval,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    DraftInternalCandidate,
    StaffReviewRequired {
        reason: ReviewReason,
        gate: policy::ReviewGate,
    },
    ManagerReviewRequired {
        reason: ReviewReason,
        gate: policy::ReviewGate,
    },
    Suppressed {
        reason: SuppressionReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewReason {
    CareSensitiveProduct,
    CarePlanConflict,
    UnknownCustomerPreference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuppressionReason {
    CustomerOptedOut,
    InventoryUnavailable,
}

#[derive(Debug, Clone, Default)]
pub struct CustomerCopyPolicy;

impl CustomerCopyPolicy {
    pub fn evaluate(&self, copy: &CustomerSafeCopy) -> CustomerCopyDecision {
        let normalized = copy.clone().into_inner().to_lowercase();
        if ["treat", "diagnos", "cure", "prescrib", "medical"]
            .iter()
            .any(|term| normalized.contains(term))
        {
            CustomerCopyDecision::Rejected {
                reason: CustomerCopyRejectionReason::MedicalClaim,
                gate: policy::ReviewGate::CustomerMessageApproval,
            }
        } else {
            CustomerCopyDecision::DraftRequiresApproval {
                gate: policy::ReviewGate::CustomerMessageApproval,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomerCopyDecision {
    DraftRequiresApproval {
        gate: policy::ReviewGate,
    },
    Rejected {
        reason: CustomerCopyRejectionReason,
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomerCopyRejectionReason {
    MedicalClaim,
    UnsupportedPromise,
}
