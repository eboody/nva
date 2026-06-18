//! Audit identifiers for traceable pet-resort automation events.
//!
//! Audit records provide the evidence trail for agent drafts, policy decisions, manager approvals,
//! customer-message handling, and source-data repairs. They are domain facts, not log formatting.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Stable identifier for an auditable operational event.
///
/// Use this id to correlate the source fact, workflow decision, review gate, and resulting staff or
/// customer action without inventing undocumented authority after the fact.
pub struct EventId(pub Uuid);
