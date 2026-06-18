//! Customer and staff message delivery state.
//!
//! Message values distinguish drafts, approvals, queued sends, delivery evidence, and suppression
//! reasons so automation can assist high-volume pet-parent communication without sending outside the
//! configured channel, consent, and manager-review boundaries.

use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Direction of a message relative to the resort operation: inbound, outbound draft, queued, or sent.
pub enum Direction {
    /// Inbound received message direction, channel, approval state, or delivery state.
    InboundReceived,
    /// Outbound draft message direction, channel, approval state, or delivery state.
    OutboundDraft,
    /// Outbound queued message direction, channel, approval state, or delivery state.
    OutboundQueued,
    /// Outbound sent message direction, channel, approval state, or delivery state.
    OutboundSent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Message channel used for customer, staff, portal, phone-note, or internal communication.
pub enum Channel {
    /// Email message direction, channel, approval state, or delivery state.
    Email,
    /// Sms message direction, channel, approval state, or delivery state.
    Sms,
    /// Phone note message direction, channel, approval state, or delivery state.
    PhoneNote,
    /// Portal message direction, channel, approval state, or delivery state.
    Portal,
    /// Internal message direction, channel, approval state, or delivery state.
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Normalized lifecycle states used to reconcile source-system data with domain workflows.
pub enum Status {
    /// Draft created message direction, channel, approval state, or delivery state.
    DraftCreated,
    /// Approval requested message direction, channel, approval state, or delivery state.
    ApprovalRequested,
    /// Approved to queue message direction, channel, approval state, or delivery state.
    ApprovedToQueue,
    /// Queued message direction, channel, approval state, or delivery state.
    Queued,
    /// Send attempted message direction, channel, approval state, or delivery state.
    SendAttempted,
    /// Delivered message direction, channel, approval state, or delivery state.
    Delivered,
    /// Deposit collection was attempted but did not succeed.
    Failed,
    /// Suppressed message direction, channel, approval state, or delivery state.
    Suppressed,
    /// Reservation is no longer active.
    Cancelled,
}

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
pub struct BodyRef(String);
