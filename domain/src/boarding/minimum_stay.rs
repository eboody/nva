use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Typed policy domain value that keeps raw primitives out of boarding workflows.
pub struct Policy {
    nights: StayNights,
    /// Business reason staff should review before proceeding.
    pub reason: Reason,
}

impl Policy {
    /// Assembles this boarding value from already-validated domain parts.
    pub const fn new(nights: StayNights, reason: Reason) -> Self {
        Self { nights, reason }
    }

    /// Returns this boarding value's nights.
    pub const fn nights(&self) -> StayNights {
        self.nights
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for reason decisions in boarding workflows.
pub enum Reason {
    /// Standard policy boarding policy, stay, capacity, or upsell signal.
    StandardPolicy,
    /// Holiday peak boarding policy, stay, capacity, or upsell signal.
    HolidayPeak,
    /// Multi pet operational buffer boarding policy, stay, capacity, or upsell signal.
    MultiPetOperationalBuffer,
}
