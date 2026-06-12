use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    nights: StayNights,
    pub reason: Reason,
}

impl Policy {
    pub const fn new(nights: StayNights, reason: Reason) -> Self {
        Self { nights, reason }
    }

    pub const fn nights(&self) -> StayNights {
        self.nights
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reason {
    StandardPolicy,
    HolidayPeak,
    MultiPetOperationalBuffer,
}
