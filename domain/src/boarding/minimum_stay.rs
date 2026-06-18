//! Boarding minimum-stay policy for standard stays, holiday peaks, and multi-pet buffers.
//!
//! Minimum stay contracts keep holiday capacity and staffing assumptions explicit instead of letting
//! an agent silently shorten a stay below local policy.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Minimum-night rule applied before a boarding reservation is accepted.
pub struct Policy {
    nights: StayNights,
    /// Operational reason the minimum-night rule exists.
    pub reason: Reason,
}

impl Policy {
    /// Creates a minimum-stay policy from validated nights and the policy reason.
    pub const fn new(nights: StayNights, reason: Reason) -> Self {
        Self { nights, reason }
    }

    /// Returns the minimum nights required before this boarding reservation can be accepted.
    pub const fn nights(&self) -> StayNights {
        self.nights
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons a boarding reservation may require a minimum stay.
pub enum Reason {
    /// Baseline local policy requires the configured stay length.
    StandardPolicy,
    /// Holiday demand or staffing pressure requires a longer stay commitment.
    HolidayPeak,
    /// Multi-pet handling or room-turn complexity requires an operational buffer.
    MultiPetOperationalBuffer,
}
