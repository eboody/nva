//! Boarding care-plan readiness for feeding, medication, and medical-document review gates.
//!
//! The policy turns pet care profiles into check-in readiness evidence, keeping automation focused
//! on surfacing missing instructions and review requirements rather than making medical judgments.

use super::*;
use crate::{entities, policy};

#[derive(Debug, Clone, Default)]
/// Boarding care-readiness policy for feeding instructions and medication review.
pub struct Policy;

impl Policy {
    /// Builds the pet-specific check-in care plan and review gates from the source care profile.
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
/// Pet-specific boarding care plan used by staff before check-in.
pub struct Plan {
    pet_id: PetId,
    gates: Vec<ReviewGate>,
}

impl Plan {
    /// Returns the pet whose boarding care readiness is represented by this plan.
    pub const fn pet_id(&self) -> PetId {
        self.pet_id
    }

    /// Returns unresolved care gates staff must clear before check-in.
    pub fn gates(&self) -> &[ReviewGate] {
        &self.gates
    }

    /// Summarizes whether the plan is ready for check-in or blocked by review gates.
    pub fn readiness(&self) -> Readiness {
        if self.gates.is_empty() {
            Readiness::ReadyForCheckIn
        } else {
            Readiness::Blocked
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Check-in readiness outcome for a boarding care plan.
pub enum Readiness {
    /// Feeding and medication evidence is sufficient for staff to proceed with check-in.
    ReadyForCheckIn,
    /// One or more care gates must be resolved before check-in proceeds.
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Specific care-review gate created from missing or sensitive boarding profile evidence.
pub struct ReviewGate {
    /// Care-profile reason that triggered the gate.
    pub reason: GateReason,
    /// Human review category required to clear this care issue.
    pub gate: policy::ReviewGate,
}

impl ReviewGate {
    /// Creates a care-review gate with the operational reason and required review category.
    pub const fn new(reason: GateReason, gate: policy::ReviewGate) -> Self {
        Self { reason, gate }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons a boarding care plan requires staff or medical-document review.
pub enum GateReason {
    /// The source care profile lacks feeding instructions for the boarding stay.
    MissingFeedingInstruction,
    /// At least one medication instruction requires staff or medical-document review.
    MedicationRequiresReview,
}
