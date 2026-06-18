//! Boarding upsell recommendations for exit baths, play, grooming, suite, and training offers.
//!
//! Recommendations carry care-safety gates so labor-saving offer drafting never bypasses staff
//! review for allergies, medications, medical conditions, behavior context, or customer messaging.

use super::*;
use crate::{entities, policy};

#[derive(Debug, Clone, Default)]
/// Boarding upsell policy that classifies offer eligibility from care evidence.
pub struct Policy;

impl Policy {
    /// Evaluates whether an exit-bath offer can be drafted or must be held for care-team review.
    pub fn evaluate_exit_bath(
        &self,
        reservation_id: entities::reservation::Id,
        pet_id: PetId,
        care_profile: &entities::CareProfile,
    ) -> Recommendation {
        let eligibility = if care_profile.allergies.is_empty()
            && care_profile.medical_conditions.is_empty()
            && care_profile.medications.is_empty()
        {
            Eligibility::Eligible
        } else {
            Eligibility::NeedsStaffReview {
                gate: policy::ReviewGate::MedicalDocumentReview,
                reason: ReviewReason::CareSafetyAmbiguity,
            }
        };

        Recommendation {
            reservation_id,
            pet_id,
            opportunity: Opportunity::ExitBath,
            eligibility,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Staff-reviewable boarding upsell recommendation with source reservation, pet, and eligibility evidence.
pub struct Recommendation {
    /// Boarding reservation the offer would attach to.
    pub reservation_id: entities::reservation::Id,
    /// Pet whose stay evidence drives the upsell recommendation.
    pub pet_id: PetId,
    /// Upsell opportunity identified for this stay.
    pub opportunity: Opportunity,
    /// Safety and review state controlling whether staff may offer the upsell.
    pub eligibility: Eligibility,
}

impl Recommendation {
    /// Returns the approval gate required before any recommendation becomes customer-facing.
    pub fn customer_offer_gate(&self) -> Option<policy::ReviewGate> {
        Some(policy::ReviewGate::CustomerMessageApproval)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Boarding-adjacent revenue opportunities that can be recommended from stay evidence.
pub enum Opportunity {
    /// Bath offered before departure from boarding.
    ExitBath,
    /// Additional enrichment or playtime during the boarding stay.
    Playtime,
    /// Grooming service line or care-note category.
    Grooming,
    /// Upgrade opportunity for a higher-tier boarding accommodation.
    PremiumSuite,
    /// Training service line or care-note category.
    Training,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Safety and review status for presenting a boarding upsell recommendation.
pub enum Eligibility {
    /// Source care evidence shows no safety ambiguity, so staff may review the offer normally.
    Eligible,
    /// Recommendation should not be shown because context makes the offer inappropriate.
    Suppressed {
        /// Reason this recommendation is suppressed or must be reviewed before use.
        reason: SuppressionReason,
    },
    /// Care, behavior, or capacity evidence requires staff review before the offer is used.
    NeedsStaffReview {
        /// Review gate staff must clear before presenting this recommendation.
        gate: policy::ReviewGate,
        /// Reason this recommendation is suppressed or must be reviewed before use.
        reason: ReviewReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons an otherwise possible upsell should be hidden from staff or customer drafts.
pub enum SuppressionReason {
    /// A recent similar offer exists, so repeating it would add noise instead of labor savings.
    DuplicateRecentOffer,
    /// Billing, refund, or complaint context makes a sales offer unsafe or inappropriate.
    PaymentOrComplaintContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons staff must review an upsell recommendation before use.
pub enum ReviewReason {
    /// Allergies, medications, or medical conditions make the offer safety-sensitive.
    CareSafetyAmbiguity,
    /// Behavior notes make the offer sensitive enough for staff review.
    BehaviorContextSensitive,
    /// Inventory or staffing evidence is needed before staff can offer the upgrade.
    CapacityEvidenceRequired,
}
