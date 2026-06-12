use super::*;
use crate::{entities, policy};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    StaffNoteOnly,
    OwnerNotice,
    ManagerReview,
    SuspendGroupPlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Restriction {
    None,
    SuspendedPendingManagerReview { pet_id: PetId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Disposition {
    pub pet_id: PetId,
    pub severity: Severity,
    restriction: Restriction,
    required_gate: Option<policy::ReviewGate>,
}

impl Disposition {
    pub const fn restriction(&self) -> Restriction {
        self.restriction
    }

    pub fn required_gate(&self) -> Option<policy::ReviewGate> {
        self.required_gate.clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Policy;

impl Policy {
    pub fn classify(&self, pet_id: entities::PetId, severity: Severity) -> Disposition {
        let (restriction, required_gate) = match severity {
            Severity::StaffNoteOnly => (Restriction::None, None),
            Severity::OwnerNotice => (
                Restriction::None,
                Some(policy::ReviewGate::CustomerMessageApproval),
            ),
            Severity::ManagerReview => {
                (Restriction::None, Some(policy::ReviewGate::ManagerApproval))
            }
            Severity::SuspendGroupPlay => (
                Restriction::SuspendedPendingManagerReview { pet_id },
                Some(policy::ReviewGate::ManagerApproval),
            ),
        };
        Disposition {
            pet_id,
            severity,
            restriction,
            required_gate,
        }
    }
}
