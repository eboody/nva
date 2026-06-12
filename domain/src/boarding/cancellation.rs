use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    pub notice: NoticeHours,
    pub penalty: Penalty,
}

impl Policy {
    pub const fn new(notice: NoticeHours, penalty: Penalty) -> Self {
        Self { notice, penalty }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Penalty {
    None,
    ForfeitDeposit,
    ManagerReview,
}
