//! Accountable customer, staff, manager, system, and agent actors.

use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use super::identifiers::CustomerId;
use crate::agent;

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
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
pub struct StaffId(String);

/// Manager identifier used when approvals, overrides, or escalations require accountable leadership.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
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
pub struct ManagerId(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Actor that performed or is accountable for an audited action.
pub enum ActorRef {
    /// Customer record participating in the workflow.
    Customer(CustomerId),
    /// Staff id retained from source records for staff review, safety gates, and workflow joins.
    Staff {
        /// Staff id attached to this variant for reviewers and adapters.
        staff_id: StaffId,
    },
    /// Manager id retained from source records for staff review, safety gates, and workflow joins.
    Manager {
        /// Manager id attached to this variant for reviewers and adapters.
        manager_id: ManagerId,
    },
    /// System state or source category preserved for normalized resort records.
    System,
    /// Workflow retained from source records for staff review, safety gates, and workflow joins.
    Agent {
        /// Workflow attached to this variant for reviewers and adapters.
        workflow: agent::Name,
    },
}
