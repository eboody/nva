use crate::entities;
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

/// Provider-supplied status reason text preserved as review evidence.
pub mod reason {
    use super::*;

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 500),
        derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize
        )
    )]
    pub struct Reason(String);
}

pub use reason::Reason;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Intended reservation transition requested by a workflow before policy and review checks are applied.
pub enum TransitionIntent {
    /// Request medical review workflow state, command, or review outcome.
    RequestMedicalReview,
    /// Apply capacity decision workflow state, command, or review outcome.
    ApplyCapacityDecision,
    /// Confirm accepted offer workflow state, command, or review outcome.
    ConfirmAcceptedOffer,
    /// Cancel reservation workflow state, command, or review outcome.
    CancelReservation,
    /// Reject by policy workflow state, command, or review outcome.
    RejectByPolicy,
    /// Complete checkout workflow state, command, or review outcome.
    CompleteCheckout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Workflow-scoped reservation transition request with target state, reason, and review intent.
pub struct Reservation {
    /// Workflow status value preserved for staff review and audit evidence.
    pub status: entities::reservation::Status,
    /// Workflow intent value preserved for staff review and audit evidence.
    pub intent: TransitionIntent,
    /// Business reason staff should review before proceeding.
    pub reason: Reason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Workflow target that a task, event, or recommended action is about.
pub enum Target {
    /// Reservation record participating in the workflow.
    Reservation(Reservation),
}
