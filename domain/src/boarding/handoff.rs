use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Requirement {
    ArrivalCareReview,
    MedicationDoubleCheck,
    DepartureBelongingsReview,
}
