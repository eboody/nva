//! Retail recommendation models for personalized upsell candidates, review gates, and safe customer copy.

use bon::Builder;
use serde::{Deserialize, Serialize};

use crate::entities::{CustomerId, LocationId, PetId};
use crate::policy;

use super::inventory::Availability;
use super::product::Product;

/// Rationale text keeps the staff-readable evidence for why a retail recommendation exists.
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
    /// No recommendation rule is active, so no upsell candidate should be produced from this rule alone.
    None,
    /// Boarding stay may justify an internal anxiety-support upsell candidate after inventory and care checks.
    AnxietySupportAfterBoarding,
    /// Boarding diet history may justify a continuity recommendation when stock and care policy allow it.
    DietSupportAfterBoarding,
    /// Grooming outcome may justify a coat-care upsell candidate after staff review gates are satisfied.
    CoatCareAfterGrooming,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Retail recommendation candidate containing customer/pet context, product, rationale, inventory, preference, and care-safety signals.
pub struct Candidate {
    /// Customer whose preferences and opt-out status control whether recommendation work may proceed.
    pub customer_id: CustomerId,
    /// Pet receiving the grooming or care service.
    pub pet_id: PetId,
    /// Location where inventory and staff review capacity are evaluated for this candidate.
    pub location_id: LocationId,
    /// Product being considered for an internal upsell candidate, not an automatic customer send.
    pub product: Product,
    /// Internal reason staff see when deciding whether the recommendation is useful and safe.
    pub reason: Reason,
    /// Staff-readable rationale explaining the source event or care context behind the candidate.
    pub rationale: rationale::Text,
    /// Care-safety state that can require medical-document or manager review before use.
    pub care_sensitivity: CareSensitivity,
    /// Availability state that suppresses candidates when stock cannot support the recommendation.
    pub inventory: Availability,
    /// Preference or opt-out state that prevents unwanted recommendation drafts.
    pub customer_preference: Preference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Business reason for recommending a product, kept separate from customer-facing copy.
pub enum Reason {
    /// Reason tied to boarding stress notes or staff-observed anxiety support needs.
    AnxietyOrStressSupport,
    /// Reason tied to preserving diet continuity after a boarding stay.
    BoardingDietContinuity,
    /// Reason tied to groomer-observed coat or skin care follow-up.
    CoatOrSkinCareAfterGrooming,
    /// Reason tied to replenishing a product the customer previously bought.
    PriorPurchaseReplenishment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Care-sensitivity status that decides whether supplement/diet/product recommendations need care or manager review.
pub enum CareSensitivity {
    /// No care conflict is known, so the candidate can remain an internal draft if other gates pass.
    NoKnownCareConflict,
    /// Supplement or diet item must pause for medical-document review before staff use or customer copy.
    SupplementOrDietReviewRequired,
    /// Pet care plan conflicts with the suggested product and requires manager review.
    CarePlanConflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Customer preference state used to suppress opted-out recommendations and review unknown consent.
pub enum Preference {
    /// Customer preference allows retail recommendation drafts when inventory and care gates pass.
    AllowsRetailRecommendations,
    /// Customer opted out, so the candidate is suppressed before staff or customer use.
    OptedOut,
    /// Customer recommendation preference is unknown, so staff must confirm consent before use.
    UnknownRequiresReview,
}

#[derive(Debug, Clone, Default)]
/// Evaluates retail recommendation safety so staff see useful candidates without bypassing care, inventory, or preference gates.
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
    /// Candidate may be shown internally to staff but is not approved customer copy or a sale action.
    DraftInternalCandidate,
    /// Candidate must pause for staff/medical-document review before use.
    StaffReviewRequired {
        /// Internal reason staff see when deciding whether the recommendation is useful and safe.
        reason: ReviewReason,
        /// Approval gate that must be satisfied before recommendation or customer-copy workflow proceeds.
        gate: policy::ReviewGate,
    },
    /// Candidate must pause for manager approval because care-plan conflict or policy risk is present.
    ManagerReviewRequired {
        /// Internal reason staff see when deciding whether the recommendation is useful and safe.
        reason: ReviewReason,
        /// Approval gate that must be satisfied before recommendation or customer-copy workflow proceeds.
        gate: policy::ReviewGate,
    },
    /// Candidate is hidden from staff/customer workflows because preference, inventory, or safety policy failed.
    Suppressed {
        /// Internal reason staff see when deciding whether the recommendation is useful and safe.
        reason: SuppressionReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Review reasons that explain whether care facts, care-plan conflicts, or consent uncertainty paused the candidate.
pub enum ReviewReason {
    /// Product category or care facts require medical-document review before staff recommend it.
    CareSensitiveProduct,
    /// Pet care plan conflicts with the suggested product and requires manager review.
    CarePlanConflict,
    /// Customer preference is unknown, so staff must verify consent before recommendation use.
    UnknownCustomerPreference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reason a recommendation is suppressed before staff/customer use.
pub enum SuppressionReason {
    /// Customer opted out, so the recommendation must be suppressed.
    CustomerOptedOut,
    /// Product is unavailable, so the recommendation must not promise or draft a sale.
    InventoryUnavailable,
}

/// Customer-copy policy keeps retail recommendation text in draft/review state until approval and rejects unsafe claims.
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
    /// Evaluates customer-facing retail copy so recommendations stay draft-only until customer-message approval.
    pub struct Policy;

    impl Policy {
        /// Rejects unsafe claim language and otherwise keeps copy behind the customer-message approval gate.
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
    /// Customer-copy decision that either remains approval-gated or is rejected for rewrite.
    pub enum Decision {
        /// Customer-facing copy is only a draft until the customer-message approval gate is satisfied.
        DraftRequiresApproval {
            /// Approval gate that must be satisfied before recommendation or customer-copy workflow proceeds.
            gate: policy::ReviewGate,
        },
        /// Customer-facing copy is rejected because it contains an unsafe claim or unsupported promise.
        Rejected {
            /// Rejection reason staff must address before this copy can be approved.
            reason: RejectionReason,
            /// Approval gate that must be satisfied before recommendation or customer-copy workflow proceeds.
            gate: policy::ReviewGate,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Customer-copy rejection reasons that explain what staff must rewrite before approval.
    pub enum RejectionReason {
        /// Copy suggests treatment, diagnosis, cure, prescribing, or medical benefit and cannot be sent as-is.
        MedicalClaim,
        /// Copy promises an outcome staff cannot verify, so it must be rewritten and approved before use.
        UnsupportedPromise,
    }
}
