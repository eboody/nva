//! Consent purpose and evidence contracts for reviewed communication use.
//!
//! Consent evidence is source-backed and exact to a channel/purpose pair. It can support a draft or
//! reviewed workflow decision, but it does not queue or send customer-visible messages without the
//! message approval and delivery lifecycle gates.

use serde::{Deserialize, Serialize};

use crate::{message, source};

/// Canonical message/contact channel used for communication evidence or draft outreach.
pub use message::Channel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Consent purpose that determines whether a channel can be used.
pub enum Purpose {
    /// Transactional response to a fresh lead or inquiry.
    TransactionalLeadResponse,
    /// Marketing or winback outreach.
    MarketingRetention,
    /// Internal service personalization only.
    InternalServiceContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Consent state for a channel/purpose pair.
pub enum ConsentStatus {
    /// Source evidence grants the use.
    Granted,
    /// Consent evidence is missing.
    Missing,
    /// Customer opted out or suppressed the channel/purpose.
    OptedOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Source-backed consent evidence for one channel and purpose.
pub struct ConsentEvidence {
    channel: Channel,
    purpose: Purpose,
    status: ConsentStatus,
    source: source::System,
}

impl ConsentEvidence {
    /// Returns whether this consent evidence allows use.
    pub const fn is_granted(&self) -> bool {
        matches!(self.status, ConsentStatus::Granted)
    }

    /// Returns whether this evidence grants the exact channel/purpose pair.
    pub fn permits(&self, channel: Channel, purpose: Purpose) -> bool {
        self.channel == channel && self.purpose == purpose && self.is_granted()
    }

    /// Channel covered by this evidence.
    pub const fn channel(&self) -> Channel {
        self.channel
    }
}
