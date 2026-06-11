use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    InboundReceived,
    OutboundDraft,
    OutboundQueued,
    OutboundSent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Channel {
    Email,
    Sms,
    PhoneNote,
    Portal,
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    DraftCreated,
    ApprovalRequested,
    ApprovedToQueue,
    Queued,
    SendAttempted,
    Delivered,
    Failed,
    Suppressed,
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
