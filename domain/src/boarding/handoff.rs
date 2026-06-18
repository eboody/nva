//! Boarding staff handoff requirements for arrival, medication, and departure review.
//!
//! Handoff values document which operational checklist must be completed when automation prepares
//! a stay packet for the resort team.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Staff handoff checklist requirement for a boarding stay.
pub enum Requirement {
    /// Arrival team must review care instructions before taking custody of the pet.
    ArrivalCareReview,
    /// Two-person or qualified-staff check is required for medication instructions.
    MedicationDoubleCheck,
    /// Departure team must verify belongings, notes, and checkout handoff before release.
    DepartureBelongingsReview,
}
