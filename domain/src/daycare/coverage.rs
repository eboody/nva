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
/// Typed roster snapshot domain value that keeps raw primitives out of daycare workflows.
pub struct RosterSnapshot {
    scheduled_staff: StaffCount,
    checked_in_pets: PetCount,
}

impl RosterSnapshot {
    /// Assembles this daycare value from already-validated domain parts.
    pub const fn new(scheduled_staff: StaffCount, checked_in_pets: PetCount) -> Self {
        Self {
            scheduled_staff,
            checked_in_pets,
        }
    }

    /// Returns this daycare value's scheduled staff.
    pub const fn scheduled_staff(&self) -> StaffCount {
        self.scheduled_staff
    }

    /// Returns this daycare value's checked in pets.
    pub const fn checked_in_pets(&self) -> PetCount {
        self.checked_in_pets
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for decision decisions in daycare workflows.
pub enum Decision {
    /// Sufficient daycare attendance, eligibility, coverage, or package signal.
    Sufficient,
    /// Insufficient daycare attendance, eligibility, coverage, or package signal.
    Insufficient {
        /// Business reason staff should review before proceeding.
        reason: InsufficiencyReason,
        /// Gate fact promoted into this daycare contract.
        gate: policy::ReviewGate,
    },
    /// Provider role or status could not be mapped confidently.
    Unknown {
        /// Gate fact promoted into this daycare contract.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for insufficiency reason decisions in daycare workflows.
pub enum InsufficiencyReason {
    /// Ratio exceeded daycare attendance, eligibility, coverage, or package signal.
    RatioExceeded,
}

#[derive(Debug, Clone, Default)]
/// Typed policy domain value that keeps raw primitives out of daycare workflows.
pub struct Policy;

impl Policy {
    /// Returns the evaluate for this daycare value.
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
