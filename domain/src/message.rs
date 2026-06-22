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
    /// Customer or staff message received by the resort and available as source context.
    InboundReceived,
    /// Outbound content exists only as a draft and must pass approval before queueing.
    OutboundDraft,
    /// Approved outbound message is queued for delivery through the configured channel.
    OutboundQueued,
    /// Outbound message was sent and should have delivery evidence recorded separately.
    OutboundSent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Message channel used for customer, staff, portal, phone-note, or internal communication.
pub enum Channel {
    /// Email channel, subject to email consent, address validation, and approval rules.
    Email,
    /// SMS channel, subject to texting consent, quiet-hour, and approval rules.
    Sms,
    /// Staff note from a call or voicemail, not an automated outbound send.
    PhoneNote,
    /// Customer portal channel controlled by portal delivery and consent settings.
    Portal,
    /// Internal staff communication that should not be treated as customer-facing delivery.
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Normalized lifecycle states used to reconcile source-system data with domain workflows.
pub enum Status {
    /// Draft exists but has not yet requested approval or entered a send queue.
    DraftCreated,
    /// Draft is waiting for the required human or policy approval.
    ApprovalRequested,
    /// Approval was granted and the message may be queued through its channel.
    ApprovedToQueue,
    /// Message is queued for sending but has no delivery result yet.
    Queued,
    /// Delivery attempt was made and needs success/failure evidence.
    SendAttempted,
    /// Delivery evidence says the message reached the channel or provider.
    Delivered,
    /// Send failed and the message needs retry, suppression, or staff review before customer contact.
    Failed,
    /// Message was intentionally suppressed because consent, policy, duplicate, or review rules blocked sending.
    Suppressed,
    /// Message send was cancelled before delivery and must not be retried without a new approval path.
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Review state for customer-message draft material before it can be queued or shown.
pub enum ReviewState {
    /// Human approval has been requested and the material must remain draft-only.
    ApprovalRequested,
    /// Human/system-of-record approval says this material may be used in the approved channel path.
    Approved,
    /// Reviewer rejected the material; it must not be used in customer-facing output.
    Rejected,
    /// The material was suppressed before review because policy, sensitivity, or source quality blocked it.
    Suppressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reason a draft fact, media/document reference, or send attempt is suppressed before delivery.
pub enum SuppressionReason {
    /// Raw internal staff content is not customer-safe.
    InternalOnly,
    /// Medical, medication, allergy, or care-plan content requires specialist review.
    SensitiveCareFact,
    /// Behavior or temperament content requires behavior/manager review.
    BehaviorReviewRequired,
    /// Incident, safety, complaint, legal, or liability content requires manager review.
    IncidentOrSafetyReview,
    /// Payment, refund, deposit, discount, waiver, or forfeiture content requires billing review.
    PaymentOrBillingReview,
    /// Source evidence is ambiguous, conflicting, unverified, wrong-subject, stale, or incomplete.
    SourceAmbiguity,
    /// Media or document evidence is not approved for customer-facing use.
    MediaReviewRequired,
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
