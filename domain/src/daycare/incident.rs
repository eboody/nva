//! Daycare incident disposition rules that preserve pet-safety review gates.
//!
//! ```
//! use domain::{daycare, entities, policy};
//! use uuid::Uuid;
//!
//! let pet_id = entities::PetId(Uuid::nil());
//! let disposition = daycare::incident::Classifier
//!     .classify(pet_id, daycare::incident::Severity::SuspendGroupPlay);
//!
//! assert_eq!(disposition.required_gate(), Some(policy::ReviewGate::ManagerApproval));
//! assert_eq!(
//!     disposition.restriction(),
//!     daycare::incident::Restriction::SuspendedPendingManagerReview { pet_id },
//! );
//! ```

use super::*;
use crate::{entities, policy};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Policy {
    StaffNoteOnly,
    ManagerReviewAndCustomerNotice,
    SuspendGroupPlayPendingReview,
}

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
pub struct Classifier;

impl Classifier {
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
