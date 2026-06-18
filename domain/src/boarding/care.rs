use super::*;
use crate::{entities, policy};

#[derive(Debug, Clone, Default)]
/// Typed policy domain value that keeps raw primitives out of boarding workflows.
pub struct Policy;

impl Policy {
    /// Returns the plan for pet for this boarding value.
    pub fn plan_for_pet(&self, pet_id: PetId, profile: &entities::CareProfile) -> Plan {
        let mut gates = Vec::new();
        if profile.feeding_instructions.is_none() {
            gates.push(ReviewGate::new(
                GateReason::MissingFeedingInstruction,
                policy::ReviewGate::MedicalDocumentReview,
            ));
        }
        if profile
            .medications
            .iter()
            .any(|medication| medication.review_requirement.requires_review())
        {
            gates.push(ReviewGate::new(
                GateReason::MedicationRequiresReview,
                policy::ReviewGate::MedicalDocumentReview,
            ));
        }
        Plan { pet_id, gates }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed plan domain value that keeps raw primitives out of boarding workflows.
pub struct Plan {
    pet_id: PetId,
    gates: Vec<ReviewGate>,
}

impl Plan {
    /// Returns this boarding value's pet id.
    pub const fn pet_id(&self) -> PetId {
        self.pet_id
    }

    /// Returns the gates for this boarding value.
    pub fn gates(&self) -> &[ReviewGate] {
        &self.gates
    }

    /// Returns the readiness for this boarding value.
    pub fn readiness(&self) -> Readiness {
        if self.gates.is_empty() {
            Readiness::ReadyForCheckIn
        } else {
            Readiness::Blocked
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for readiness decisions in boarding workflows.
pub enum Readiness {
    /// Ready for check in boarding policy, stay, capacity, or upsell signal.
    ReadyForCheckIn,
    /// Blocked boarding policy, stay, capacity, or upsell signal.
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed review gate domain value that keeps raw primitives out of boarding workflows.
pub struct ReviewGate {
    /// Business reason staff should review before proceeding.
    pub reason: GateReason,
    /// Gate fact promoted into this boarding contract.
    pub gate: policy::ReviewGate,
}

impl ReviewGate {
    /// Assembles this boarding value from already-validated domain parts.
    pub const fn new(reason: GateReason, gate: policy::ReviewGate) -> Self {
        Self { reason, gate }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for gate reason decisions in boarding workflows.
pub enum GateReason {
    /// Missing feeding instruction boarding policy, stay, capacity, or upsell signal.
    MissingFeedingInstruction,
    /// Medication requires review boarding policy, stay, capacity, or upsell signal.
    MedicationRequiresReview,
}
