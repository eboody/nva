use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for direction decisions in message workflows.
pub enum Direction {
    /// Inbound received customer-message channel, approval, or delivery state.
    InboundReceived,
    /// Outbound draft customer-message channel, approval, or delivery state.
    OutboundDraft,
    /// Outbound queued customer-message channel, approval, or delivery state.
    OutboundQueued,
    /// Outbound sent customer-message channel, approval, or delivery state.
    OutboundSent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for channel decisions in message workflows.
pub enum Channel {
    /// Email customer-message channel, approval, or delivery state.
    Email,
    /// Sms customer-message channel, approval, or delivery state.
    Sms,
    /// Phone note customer-message channel, approval, or delivery state.
    PhoneNote,
    /// Portal customer-message channel, approval, or delivery state.
    Portal,
    /// Internal customer-message channel, approval, or delivery state.
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Normalized reservation states observed during source-data ingestion.
pub enum Status {
    /// Draft created customer-message channel, approval, or delivery state.
    DraftCreated,
    /// Approval requested customer-message channel, approval, or delivery state.
    ApprovalRequested,
    /// Approved to queue customer-message channel, approval, or delivery state.
    ApprovedToQueue,
    /// Queued customer-message channel, approval, or delivery state.
    Queued,
    /// Send attempted customer-message channel, approval, or delivery state.
    SendAttempted,
    /// Delivered customer-message channel, approval, or delivery state.
    Delivered,
    /// Deposit collection was attempted but did not succeed.
    Failed,
    /// Suppressed customer-message channel, approval, or delivery state.
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
