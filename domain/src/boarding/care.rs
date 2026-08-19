//! Boarding care-plan readiness for feeding, medication, and medical-document review gates.
//!
//! The policy turns pet care profiles into check-in readiness evidence, keeping automation focused
//! on surfacing missing instructions and review requirements rather than making medical judgments.

use super::*;
use crate::policy;

#[derive(Debug, Clone, Default)]
/// Boarding care-readiness policy for feeding instructions and medication review.
pub struct Policy;

impl Policy {}

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Specific care-review gate created from missing or sensitive boarding profile evidence.
pub struct ReviewGate {
    /// Care-profile reason that triggered the gate.
    pub reason: GateReason,
    /// Human review category required to clear this care issue.
    pub gate: policy::ReviewGate,
}

impl ReviewGate {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons a boarding care plan requires staff or medical-document review.
pub enum GateReason {
    /// The source care profile lacks feeding instructions for the boarding stay.
    MissingFeedingInstruction,
    /// At least one medication instruction requires staff or medical-document review.
    MedicationRequiresReview,
}
