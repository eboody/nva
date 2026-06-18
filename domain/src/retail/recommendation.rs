use bon::Builder;
use serde::{Deserialize, Serialize};

use crate::entities::{CustomerId, LocationId, PetId};
use crate::policy;

use super::inventory::Availability;
use super::product::Product;

/// Rationale boundary for retail recommendation contracts.
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
/// Domain vocabulary for rule decisions in retail workflows.
pub enum Rule {
    /// No additional workflow gate is required.
    None,
    /// Anxiety support after boarding retail inventory, POS, reorder, or recommendation signal.
    AnxietySupportAfterBoarding,
    /// Diet support after boarding retail inventory, POS, reorder, or recommendation signal.
    DietSupportAfterBoarding,
    /// Coat care after grooming retail inventory, POS, reorder, or recommendation signal.
    CoatCareAfterGrooming,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Typed candidate domain value that keeps raw primitives out of retail workflows.
pub struct Candidate {
    /// Customer id fact promoted into this retail contract.
    pub customer_id: CustomerId,
    /// Pet receiving the grooming or care service.
    pub pet_id: PetId,
    /// Location id fact promoted into this retail contract.
    pub location_id: LocationId,
    /// Product fact promoted into this retail contract.
    pub product: Product,
    /// Business reason staff should review before proceeding.
    pub reason: Reason,
    /// Rationale fact promoted into this retail contract.
    pub rationale: rationale::Text,
    /// Care sensitivity fact promoted into this retail contract.
    pub care_sensitivity: CareSensitivity,
    /// Inventory fact promoted into this retail contract.
    pub inventory: Availability,
    /// Customer preference fact promoted into this retail contract.
    pub customer_preference: Preference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for reason decisions in retail workflows.
pub enum Reason {
    /// Anxiety or stress support retail inventory, POS, reorder, or recommendation signal.
    AnxietyOrStressSupport,
    /// Boarding diet continuity retail inventory, POS, reorder, or recommendation signal.
    BoardingDietContinuity,
    /// Coat or skin care after grooming retail inventory, POS, reorder, or recommendation signal.
    CoatOrSkinCareAfterGrooming,
    /// Prior purchase replenishment retail inventory, POS, reorder, or recommendation signal.
    PriorPurchaseReplenishment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for care sensitivity decisions in retail workflows.
pub enum CareSensitivity {
    /// No known care conflict retail inventory, POS, reorder, or recommendation signal.
    NoKnownCareConflict,
    /// Supplement or diet review required retail inventory, POS, reorder, or recommendation signal.
    SupplementOrDietReviewRequired,
    /// Care plan conflict retail inventory, POS, reorder, or recommendation signal.
    CarePlanConflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for preference decisions in retail workflows.
pub enum Preference {
    /// Allows retail recommendations retail inventory, POS, reorder, or recommendation signal.
    AllowsRetailRecommendations,
    /// Opted out retail inventory, POS, reorder, or recommendation signal.
    OptedOut,
    /// Estimate confidence is unknown and must be reviewed.
    UnknownRequiresReview,
}

#[derive(Debug, Clone, Default)]
/// Typed policy domain value that keeps raw primitives out of retail workflows.
pub struct Policy;

impl Policy {
    /// Returns the evaluate for this retail value.
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
/// Domain vocabulary for decision decisions in retail workflows.
pub enum Decision {
    /// Draft internal candidate retail inventory, POS, reorder, or recommendation signal.
    DraftInternalCandidate,
    /// Staff review required retail inventory, POS, reorder, or recommendation signal.
    StaffReviewRequired {
        /// Business reason staff should review before proceeding.
        reason: ReviewReason,
        /// Gate fact promoted into this retail contract.
        gate: policy::ReviewGate,
    },
    /// Manager review required retail inventory, POS, reorder, or recommendation signal.
    ManagerReviewRequired {
        /// Business reason staff should review before proceeding.
        reason: ReviewReason,
        /// Gate fact promoted into this retail contract.
        gate: policy::ReviewGate,
    },
    /// Suppressed retail inventory, POS, reorder, or recommendation signal.
    Suppressed {
        /// Business reason staff should review before proceeding.
        reason: SuppressionReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for review reason decisions in retail workflows.
pub enum ReviewReason {
    /// Care sensitive product retail inventory, POS, reorder, or recommendation signal.
    CareSensitiveProduct,
    /// Care plan conflict retail inventory, POS, reorder, or recommendation signal.
    CarePlanConflict,
    /// Unknown customer preference retail inventory, POS, reorder, or recommendation signal.
    UnknownCustomerPreference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for suppression reason decisions in retail workflows.
pub enum SuppressionReason {
    /// Customer opted out retail inventory, POS, reorder, or recommendation signal.
    CustomerOptedOut,
    /// Inventory unavailable retail inventory, POS, reorder, or recommendation signal.
    InventoryUnavailable,
}

/// Customer copy boundary for retail recommendation contracts.
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
    /// Typed policy domain value that keeps raw primitives out of retail workflows.
    pub struct Policy;

    impl Policy {
        /// Returns the evaluate for this retail value.
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
    /// Domain vocabulary for decision decisions in retail workflows.
    pub enum Decision {
        /// Draft requires approval retail inventory, POS, reorder, or recommendation signal.
        DraftRequiresApproval {
            /// Gate fact promoted into this retail contract.
            gate: policy::ReviewGate,
        },
        /// Rejected retail inventory, POS, reorder, or recommendation signal.
        Rejected {
            /// Business reason staff should review before proceeding.
            reason: RejectionReason,
            /// Gate fact promoted into this retail contract.
            gate: policy::ReviewGate,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for rejection reason decisions in retail workflows.
    pub enum RejectionReason {
        /// Medical claim retail inventory, POS, reorder, or recommendation signal.
        MedicalClaim,
        /// Unsupported promise retail inventory, POS, reorder, or recommendation signal.
        UnsupportedPromise,
    }
}
