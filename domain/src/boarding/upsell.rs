use super::*;
use crate::{entities, policy};

#[derive(Debug, Clone, Default)]
/// Typed policy domain value that keeps raw primitives out of boarding workflows.
pub struct Policy;

impl Policy {
    /// Returns the evaluate exit bath for this boarding value.
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
/// Typed recommendation domain value that keeps raw primitives out of boarding workflows.
pub struct Recommendation {
    /// Reservation id fact promoted into this boarding contract.
    pub reservation_id: entities::reservation::Id,
    /// Pet receiving the grooming or care service.
    pub pet_id: PetId,
    /// Opportunity fact promoted into this boarding contract.
    pub opportunity: Opportunity,
    /// Eligibility fact promoted into this boarding contract.
    pub eligibility: Eligibility,
}

impl Recommendation {
    /// Returns the customer offer gate for this boarding value.
    pub fn customer_offer_gate(&self) -> Option<policy::ReviewGate> {
        Some(policy::ReviewGate::CustomerMessageApproval)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for opportunity decisions in boarding workflows.
pub enum Opportunity {
    /// Bath offered before departure from boarding.
    ExitBath,
    /// Playtime boarding policy, stay, capacity, or upsell signal.
    Playtime,
    /// Grooming service line or care-note category.
    Grooming,
    /// Premium suite boarding policy, stay, capacity, or upsell signal.
    PremiumSuite,
    /// Training service line or care-note category.
    Training,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for eligibility decisions in boarding workflows.
pub enum Eligibility {
    /// Eligible boarding policy, stay, capacity, or upsell signal.
    Eligible,
    /// Suppressed boarding policy, stay, capacity, or upsell signal.
    Suppressed {
        /// Business reason staff should review before proceeding.
        reason: SuppressionReason,
    },
    /// Needs staff review boarding policy, stay, capacity, or upsell signal.
    NeedsStaffReview {
        /// Gate fact promoted into this boarding contract.
        gate: policy::ReviewGate,
        /// Business reason staff should review before proceeding.
        reason: ReviewReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for suppression reason decisions in boarding workflows.
pub enum SuppressionReason {
    /// Duplicate recent offer boarding policy, stay, capacity, or upsell signal.
    DuplicateRecentOffer,
    /// Payment or complaint context boarding policy, stay, capacity, or upsell signal.
    PaymentOrComplaintContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for review reason decisions in boarding workflows.
pub enum ReviewReason {
    /// Care safety ambiguity boarding policy, stay, capacity, or upsell signal.
    CareSafetyAmbiguity,
    /// Behavior context sensitive boarding policy, stay, capacity, or upsell signal.
    BehaviorContextSensitive,
    /// Capacity evidence required boarding policy, stay, capacity, or upsell signal.
    CapacityEvidenceRequired,
}
