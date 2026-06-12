use bon::Builder;
use serde::{Deserialize, Serialize};

use crate::entities::{CustomerId, LocationId, PetId};
use crate::policy;

use super::inventory::Availability;
use super::product::Product;

pub mod rationale {
    use nutype::nutype;
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};

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
    pub struct Text(String);
}

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
    pub rationale: rationale::Text,
    pub care_sensitivity: CareSensitivity,
    pub inventory: Availability,
    pub customer_preference: Preference,
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
pub enum Preference {
    AllowsRetailRecommendations,
    OptedOut,
    UnknownRequiresReview,
}

#[derive(Debug, Clone, Default)]
pub struct Policy;

impl Policy {
    pub fn evaluate(&self, candidate: &Candidate) -> Decision {
        if matches!(candidate.customer_preference, Preference::OptedOut) {
            return Decision::Suppressed {
                reason: SuppressionReason::CustomerOptedOut,
            };
        }
        if !matches!(candidate.inventory, Availability::Available) {
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

pub mod customer_copy {
    use nutype::nutype;
    use serde::{Deserialize, Serialize};

    use crate::policy;

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
    pub struct SafeCopy(String);

    #[derive(Debug, Clone, Default)]
    pub struct Policy;

    impl Policy {
        pub fn evaluate(&self, copy: &SafeCopy) -> Decision {
            let normalized = copy.clone().into_inner().to_lowercase();
            if ["treat", "diagnos", "cure", "prescrib", "medical"]
                .iter()
                .any(|term| normalized.contains(term))
            {
                Decision::Rejected {
                    reason: RejectionReason::MedicalClaim,
                    gate: policy::ReviewGate::CustomerMessageApproval,
                }
            } else {
                Decision::DraftRequiresApproval {
                    gate: policy::ReviewGate::CustomerMessageApproval,
                }
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Decision {
        DraftRequiresApproval {
            gate: policy::ReviewGate,
        },
        Rejected {
            reason: RejectionReason,
            gate: policy::ReviewGate,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RejectionReason {
        MedicalClaim,
        UnsupportedPromise,
    }
}
