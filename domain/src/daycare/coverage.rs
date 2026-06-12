use super::*;
use crate::policy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterSnapshot {
    scheduled_staff: StaffCount,
    checked_in_pets: PetCount,
}

impl RosterSnapshot {
    pub const fn new(scheduled_staff: StaffCount, checked_in_pets: PetCount) -> Self {
        Self {
            scheduled_staff,
            checked_in_pets,
        }
    }

    pub const fn scheduled_staff(&self) -> StaffCount {
        self.scheduled_staff
    }

    pub const fn checked_in_pets(&self) -> PetCount {
        self.checked_in_pets
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Sufficient,
    Insufficient {
        reason: InsufficiencyReason,
        gate: policy::ReviewGate,
    },
    Unknown {
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsufficiencyReason {
    RatioExceeded,
}

#[derive(Debug, Clone, Default)]
pub struct Policy;

impl Policy {
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
