use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for requirement decisions in boarding workflows.
pub enum Requirement {
    /// Arrival care review boarding policy, stay, capacity, or upsell signal.
    ArrivalCareReview,
    /// Medication double check boarding policy, stay, capacity, or upsell signal.
    MedicationDoubleCheck,
    /// Departure belongings review boarding policy, stay, capacity, or upsell signal.
    DepartureBelongingsReview,
}
