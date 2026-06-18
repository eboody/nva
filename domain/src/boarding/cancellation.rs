use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Typed policy domain value that keeps raw primitives out of boarding workflows.
pub struct Policy {
    /// Notice fact promoted into this boarding contract.
    pub notice: NoticeHours,
    /// Penalty fact promoted into this boarding contract.
    pub penalty: Penalty,
}

impl Policy {
    /// Assembles this boarding value from already-validated domain parts.
    pub const fn new(notice: NoticeHours, penalty: Penalty) -> Self {
        Self { notice, penalty }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for penalty decisions in boarding workflows.
pub enum Penalty {
    /// No additional workflow gate is required.
    None,
    /// Forfeit deposit boarding policy, stay, capacity, or upsell signal.
    ForfeitDeposit,
    /// Manager review boarding policy, stay, capacity, or upsell signal.
    ManagerReview,
}
