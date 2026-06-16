use super::*;
use crate::{entities, policy};

#[derive(Debug, Clone, Default)]
pub struct Policy;

impl Policy {
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
pub struct Recommendation {
    pub reservation_id: entities::reservation::Id,
    pub pet_id: PetId,
    pub opportunity: Opportunity,
    pub eligibility: Eligibility,
}

impl Recommendation {
    pub fn customer_offer_gate(&self) -> Option<policy::ReviewGate> {
        Some(policy::ReviewGate::CustomerMessageApproval)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opportunity {
    ExitBath,
    Playtime,
    Grooming,
    PremiumSuite,
    Training,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Eligibility {
    Eligible,
    Suppressed {
        reason: SuppressionReason,
    },
    NeedsStaffReview {
        gate: policy::ReviewGate,
        reason: ReviewReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuppressionReason {
    DuplicateRecentOffer,
    PaymentOrComplaintContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewReason {
    CareSafetyAmbiguity,
    BehaviorContextSensitive,
    CapacityEvidenceRequired,
}
