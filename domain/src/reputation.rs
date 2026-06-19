//! Reputation-review triage for customer-trust, safety-theme, and response workflows.
//!
//! Review signals and escalation decisions cut across service lines. A provider
//! review becomes a validated reputation signal, then drives manager/reputation
//! workflow only through explicit escalation states so customer-facing responses,
//! injury/safety themes, and legal-sensitive cases stay human-gated.

use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use crate::entities::LocationId;
use crate::operations;

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
/// Validated external review platform name used as reputation-source evidence.
pub struct PlatformName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
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
/// Stable provider review id retained for traceability and deduplication.
pub struct Id(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Review signal that preserves platform evidence for manager triage and response drafting.
pub struct Signal {
    /// Resort location connected to the review or escalation.
    pub location_id: LocationId,
    /// External review platform where staff can verify the source post.
    pub platform: PlatformName,
    /// Provider review id retained for traceability and duplicate checks.
    pub review_id: Id,
    /// Sentiment classification used to rank manager or reputation follow-up.
    pub sentiment: Sentiment,
    /// Topics staff should review before drafting or routing a response.
    pub themes: Vec<Theme>,
    /// Required review path before customer-facing response or legal-sensitive handling.
    pub escalation: Escalation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Review sentiment used to triage service recovery and reputation follow-up.
pub enum Sentiment {
    /// Positive sentiment that may feed recognition, marketing, or low-risk thank-you drafting.
    Positive,
    /// Neutral sentiment that may need monitoring but not automatic escalation by itself.
    Neutral,
    /// Negative sentiment that should trigger service-recovery review before any public response.
    Negative,
    /// Mixed sentiment with both praise and concerns, requiring staff interpretation before response.
    Mixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Review theme that links customer feedback to service, staffing, facility, or pricing evidence.
pub enum Theme {
    /// Feedback about staff interactions that should be verified against service context before response.
    StaffExperience,
    /// Cleanliness theme that may connect to facility checks or manager follow-up.
    Cleanliness,
    /// Pricing or billing theme requiring invoice/payment evidence before customer response.
    PricingOrBilling,
    /// Booking-experience theme tied to availability, scheduling, or intake workflow evidence.
    BookingExperience,
    /// Grooming-outcome theme requiring service evidence before apology, refund, or corrective promises.
    GroomingOutcome,
    /// Pet injury or safety theme that must route through manager/safety review before public reply.
    PetInjuryOrSafety,
    /// Communication theme that can point to missed messages, unclear updates, or follow-up gaps.
    Communication,
    /// Wait-time theme that may need staffing, front-desk, or scheduling evidence.
    WaitTime,
    /// Non-dog, non-cat pet handled by exception policy.
    Other(operations::operational::Observation),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Review-response gate that decides whether automation may draft or must escalate.
pub enum Escalation {
    /// No additional workflow gate is required.
    None,
    /// A public response may be drafted, but staff must verify service evidence before approval.
    DraftPublicResponse,
    /// Manager must review the signal before customer-facing or operational follow-up proceeds.
    ManagerReviewRequired,
    /// Safety/legal-sensitive signal blocks public response until authorized review approves wording.
    SafetyOrLegalReviewRequired,
}
