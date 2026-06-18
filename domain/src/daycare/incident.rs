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
/// Groomer-assignment policies used when booking grooming work.
pub enum Policy {
    /// Staff note only daycare attendance, eligibility, coverage, or package signal.
    StaffNoteOnly,
    /// Manager review and customer notice daycare attendance, eligibility, coverage, or package signal.
    ManagerReviewAndCustomerNotice,
    /// Suspend group play pending review daycare attendance, eligibility, coverage, or package signal.
    SuspendGroupPlayPendingReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for severity decisions in daycare workflows.
pub enum Severity {
    /// Staff note only daycare attendance, eligibility, coverage, or package signal.
    StaffNoteOnly,
    /// Owner notice daycare attendance, eligibility, coverage, or package signal.
    OwnerNotice,
    /// Manager review daycare attendance, eligibility, coverage, or package signal.
    ManagerReview,
    /// Suspend group play daycare attendance, eligibility, coverage, or package signal.
    SuspendGroupPlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for restriction decisions in daycare workflows.
pub enum Restriction {
    /// No additional workflow gate is required.
    None,
    /// Pet receiving the grooming or care service.
    SuspendedPendingManagerReview {
        /// Pet id carried by this variant.
        pet_id: PetId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed disposition domain value that keeps raw primitives out of daycare workflows.
pub struct Disposition {
    /// Pet receiving the grooming or care service.
    pub pet_id: PetId,
    /// Severity fact promoted into this daycare contract.
    pub severity: Severity,
    restriction: Restriction,
    required_gate: Option<policy::ReviewGate>,
}

impl Disposition {
    /// Returns this daycare value's restriction.
    pub const fn restriction(&self) -> Restriction {
        self.restriction
    }

    /// Returns the required gate for this daycare value.
    pub fn required_gate(&self) -> Option<policy::ReviewGate> {
        self.required_gate.clone()
    }
}

#[derive(Debug, Clone, Default)]
/// Typed classifier domain value that keeps raw primitives out of daycare workflows.
pub struct Classifier;

impl Classifier {
    /// Returns the classify for this daycare value.
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
