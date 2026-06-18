//! Retail recommendation contracts for personalized upsell candidates, review gates, and safe customer copy.

use bon::Builder;
use serde::{Deserialize, Serialize};

use crate::entities::{CustomerId, LocationId, PetId};
use crate::policy;

use super::inventory::Availability;
use super::product::Product;

/// Rationale boundary for human-readable evidence explaining why a retail recommendation exists.
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
/// Recommendation rule that names the operational event that can produce an upsell candidate.
pub enum Rule {
    /// No additional workflow gate is required.
    None,
    /// Anxiety support after boarding retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    AnxietySupportAfterBoarding,
    /// Diet support after boarding retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    DietSupportAfterBoarding,
    /// Coat care after grooming retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    CoatCareAfterGrooming,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Retail recommendation candidate containing customer/pet context, product, rationale, inventory, preference, and care-safety signals.
pub struct Candidate {
    /// Source-derived customer id carried by this retail contract.
    pub customer_id: CustomerId,
    /// Pet receiving the grooming or care service.
    pub pet_id: PetId,
    /// Source-derived location id carried by this retail contract.
    pub location_id: LocationId,
    /// Source-derived product carried by this retail contract.
    pub product: Product,
    /// Business reason staff should review before proceeding.
    pub reason: Reason,
    /// Source-derived rationale carried by this retail contract.
    pub rationale: rationale::Text,
    /// Source-derived care sensitivity carried by this retail contract.
    pub care_sensitivity: CareSensitivity,
    /// Source-derived inventory carried by this retail contract.
    pub inventory: Availability,
    /// Source-derived customer preference carried by this retail contract.
    pub customer_preference: Preference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Business reason for recommending a product, kept separate from customer-facing copy.
pub enum Reason {
    /// Anxiety or stress support retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    AnxietyOrStressSupport,
    /// Boarding diet continuity retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    BoardingDietContinuity,
    /// Coat or skin care after grooming retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    CoatOrSkinCareAfterGrooming,
    /// Prior purchase replenishment retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    PriorPurchaseReplenishment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Care-sensitivity status that decides whether supplement/diet/product recommendations need care or manager review.
pub enum CareSensitivity {
    /// No known care conflict retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    NoKnownCareConflict,
    /// Supplement or diet review required retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    SupplementOrDietReviewRequired,
    /// Care plan conflict retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    CarePlanConflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Customer preference state used to suppress opted-out recommendations and review unknown consent.
pub enum Preference {
    /// Allows retail recommendations retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    AllowsRetailRecommendations,
    /// Opted out retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    OptedOut,
    /// Estimate confidence is unknown and must be reviewed.
    UnknownRequiresReview,
}

#[derive(Debug, Clone, Default)]
/// Represents the policy concept as a typed retail operational contract instead of a raw primitive.
pub struct Policy;

impl Policy {
    /// Evaluates recommendation or customer copy safety without bypassing preference, inventory, or care-review gates.
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
/// Recommendation decision that either drafts an internal candidate, requires review, or suppresses the upsell.
pub enum Decision {
    /// Draft internal candidate retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    DraftInternalCandidate,
    /// Staff review required retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    StaffReviewRequired {
        /// Business reason staff should review before proceeding.
        reason: ReviewReason,
        /// Source-derived gate carried by this retail contract.
        gate: policy::ReviewGate,
    },
    /// Manager review required retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    ManagerReviewRequired {
        /// Business reason staff should review before proceeding.
        reason: ReviewReason,
        /// Source-derived gate carried by this retail contract.
        gate: policy::ReviewGate,
    },
    /// Suppressed retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    Suppressed {
        /// Business reason staff should review before proceeding.
        reason: SuppressionReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Decision vocabulary for review reason in retail workflows.
pub enum ReviewReason {
    /// Care sensitive product retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    CareSensitiveProduct,
    /// Care plan conflict retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    CarePlanConflict,
    /// Unknown customer preference retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    UnknownCustomerPreference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reason a recommendation is suppressed before staff/customer use.
pub enum SuppressionReason {
    /// Customer opted out retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    CustomerOptedOut,
    /// Inventory unavailable retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    InventoryUnavailable,
}

/// Customer-copy boundary that prevents unsafe retail claims from leaving draft/review state.
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
    /// Represents the policy concept as a typed retail operational contract instead of a raw primitive.
    pub struct Policy;

    impl Policy {
        /// Evaluates recommendation or customer copy safety without bypassing preference, inventory, or care-review gates.
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
    /// Recommendation decision that either drafts an internal candidate, requires review, or suppresses the upsell.
    pub enum Decision {
        /// Draft requires approval retail operational signal for inventory, POS, reorder, recommendation, or review handling.
        DraftRequiresApproval {
            /// Source-derived gate carried by this retail contract.
            gate: policy::ReviewGate,
        },
        /// Rejected retail operational signal for inventory, POS, reorder, recommendation, or review handling.
        Rejected {
            /// Business reason staff should review before proceeding.
            reason: RejectionReason,
            /// Source-derived gate carried by this retail contract.
            gate: policy::ReviewGate,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for rejection reason in retail workflows.
    pub enum RejectionReason {
        /// Medical claim retail operational signal for inventory, POS, reorder, recommendation, or review handling.
        MedicalClaim,
        /// Unsupported promise retail operational signal for inventory, POS, reorder, recommendation, or review handling.
        UnsupportedPromise,
    }
}
