use super::*;
use crate::{entities, policy};

#[derive(Debug, Clone, Default)]
pub struct Policy;

impl Policy {
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
pub struct Plan {
    pet_id: PetId,
    gates: Vec<ReviewGate>,
}

impl Plan {
    pub const fn pet_id(&self) -> PetId {
        self.pet_id
    }

    pub fn gates(&self) -> &[ReviewGate] {
        &self.gates
    }

    pub fn readiness(&self) -> Readiness {
        if self.gates.is_empty() {
            Readiness::ReadyForCheckIn
        } else {
            Readiness::Blocked
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Readiness {
    ReadyForCheckIn,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewGate {
    pub reason: GateReason,
    pub gate: policy::ReviewGate,
}

impl ReviewGate {
    pub const fn new(reason: GateReason, gate: policy::ReviewGate) -> Self {
        Self { reason, gate }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateReason {
    MissingFeedingInstruction,
    MedicationRequiresReview,
}
