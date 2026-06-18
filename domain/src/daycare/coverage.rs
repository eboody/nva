//! Daycare staff-coverage policy for group-play ratio review.
//!
//! ```
//! use domain::{daycare, policy};
//!
//! let roster = daycare::coverage::RosterSnapshot::new(
//!     daycare::StaffCount::try_new(1).unwrap(),
//!     daycare::PetCount::try_new(18).unwrap(),
//! );
//! let allowed_ratio = daycare::StaffPetRatio::new(
//!     daycare::StaffCount::try_new(1).unwrap(),
//!     daycare::PetCount::try_new(12).unwrap(),
//! );
//!
//! assert_eq!(
//!     daycare::coverage::Policy.evaluate(&roster, allowed_ratio),
//!     daycare::coverage::Decision::Insufficient {
//!         reason: daycare::coverage::InsufficiencyReason::RatioExceeded,
//!         gate: policy::ReviewGate::ManagerApproval,
//!     },
//! );
//! ```

use super::*;
use crate::policy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Point-in-time daycare roster evidence for staffing-ratio decisions.
pub struct RosterSnapshot {
    scheduled_staff: StaffCount,
    checked_in_pets: PetCount,
}

impl RosterSnapshot {
    /// Creates a roster snapshot from scheduled staff and checked-in pet counts.
    pub const fn new(scheduled_staff: StaffCount, checked_in_pets: PetCount) -> Self {
        Self {
            scheduled_staff,
            checked_in_pets,
        }
    }

    /// Returns scheduled staff available to supervise daycare pets.
    pub const fn scheduled_staff(&self) -> StaffCount {
        self.scheduled_staff
    }

    /// Returns pets already checked in and counted against the staffing ratio.
    pub const fn checked_in_pets(&self) -> PetCount {
        self.checked_in_pets
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Staffing coverage decision used by eligibility, assignment, and front-desk routing.
pub enum Decision {
    /// Scheduled staff can cover the checked-in pet count under the configured ratio.
    Sufficient,
    /// Checked-in pet count exceeds allowed coverage and needs manager review.
    Insufficient {
        /// Reason staffing coverage is insufficient.
        reason: InsufficiencyReason,
        /// Human review gate required before staff override coverage policy.
        gate: policy::ReviewGate,
    },
    /// Coverage evidence is unavailable or uncertain, so staff review is required.
    Unknown {
        /// Human review gate required before staff override coverage policy.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons daycare staffing coverage is insufficient.
pub enum InsufficiencyReason {
    /// Pet count exceeds the allowed pets-per-staff ratio.
    RatioExceeded,
}

#[derive(Debug, Clone, Default)]
/// Deterministic staffing-coverage policy for daycare ratio review.
pub struct Policy;

impl Policy {
    /// Evaluates scheduled staff and checked-in pets against the allowed ratio.
    pub fn evaluate(&self, roster: &RosterSnapshot, ratio: StaffPetRatio) -> Decision {
        let allowed = roster
            .scheduled_staff()
            .get()
            .saturating_mul(ratio.pets_per_staff().get());
        if roster.checked_in_pets().get() <= allowed {
            Decision::Sufficient
        } else {
            Decision::Insufficient {
                reason: InsufficiencyReason::RatioExceeded,
                gate: policy::ReviewGate::ManagerApproval,
            }
        }
    }
}
