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
/// Daycare incident handling policy that determines notes, customer notice, and group-play suspension.
pub enum Policy {
    /// Incident is recorded as a staff note without extra customer or manager workflow.
    StaffNoteOnly,
    /// Incident requires manager review and customer-message approval before follow-up.
    ManagerReviewAndCustomerNotice,
    /// Incident should suspend group play until manager review clears the pet.
    SuspendGroupPlayPendingReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Severity classification supplied to the incident classifier.
pub enum Severity {
    /// Incident is recorded as a staff note without extra customer or manager workflow.
    StaffNoteOnly,
    /// Incident requires customer notice but does not itself suspend group play.
    OwnerNotice,
    /// Incident requires manager review before staff close the disposition.
    ManagerReview,
    /// Incident should suspend group play pending manager review.
    SuspendGroupPlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Operational restriction created by a daycare incident disposition.
pub enum Restriction {
    /// Incident creates no active restriction on future daycare attendance.
    None,
    /// Pet whose group-play access is suspended pending manager review.
    SuspendedPendingManagerReview {
        /// Pet id carried by this variant.
        pet_id: PetId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Classified incident outcome carrying restriction and review-gate evidence.
pub struct Disposition {
    /// Pet whose group-play access is suspended pending manager review.
    pub pet_id: PetId,
    /// Severity used to choose restriction and review gates.
    pub severity: Severity,
    restriction: Restriction,
    required_gate: Option<policy::ReviewGate>,
}

impl Disposition {
    /// Returns the operational restriction that eligibility policy must honor.
    pub const fn restriction(&self) -> Restriction {
        self.restriction
    }

    /// Returns the human review gate required before the incident disposition can be cleared.
    pub fn required_gate(&self) -> Option<policy::ReviewGate> {
        self.required_gate.clone()
    }
}

#[derive(Debug, Clone, Default)]
/// Deterministic classifier that maps incident severity to restrictions and review gates.
pub struct Classifier;

impl Classifier {
    /// Classifies an incident severity for a pet into a disposition staff can act on.
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
