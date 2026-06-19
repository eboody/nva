//! Incident categories and review states for pet-safety workflows.
//!
//! ## Operator-summary
//!
//! This module supports the incident queue that classifies injuries, altercations, behavior
//! issues, medication errors, escapes, property damage, and customer-service problems by
//! severity and lifecycle. It can reduce labor by making manager follow-up, investigation,
//! customer-message review, legal hold, daily-brief risk, and reputation-response routing
//! visible from the same source-backed incident facts.
//!
//! It must not automate live customer promises, medical/legal conclusions, disciplinary
//! action, refunds, provider writes, or closure of a safety event. Authoritative facts remain
//! the original staff/source report, redacted summary, evidence documents, affected subjects,
//! severity/category review, required gates, and audit history. Review gates protect pets,
//! customers, and staff by keeping high/critical, customer-message, reopened, legal-hold,
//! or manager-review incidents in human review before follow-up, eligibility changes, or
//! external communication occur.
//!
//! Incident facts are source-derived evidence that must stay review-gated: they can create
//! staff follow-up, manager escalation, reputation response, and daily-brief risk entries,
//! but they should not trigger unapproved customer, legal, or medical commitments.

use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Incident category that routes injury, behavior, escape, property, and medication follow-up.
pub enum Category {
    /// Injury report involving a pet, customer, or staff member that needs evidence and care review.
    Injury,
    /// Altercation between pets or people that may affect eligibility, staffing, and customer communication.
    Altercation,
    /// Behavior event that should feed temperament, daycare, or handling review before future care decisions.
    Behavior,
    /// Medication error or concern requiring care review and careful customer communication.
    Medication,
    /// Escape or containment issue that requires safety review and operational follow-up.
    Escape,
    /// Property damage incident preserved for manager review, repair labor, and customer context.
    Property,
    /// Customer-service complaint or service failure that requires manager-reviewed follow-up.
    CustomerService,
    /// Non-dog, non-cat pet handled by exception policy.
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Incident severity used to rank manager attention, customer communication, and safety review.
pub enum Severity {
    /// Low-severity incident that still remains visible for trend and labor follow-up.
    Low,
    /// Medium-severity incident requiring normal manager awareness and documentation.
    Medium,
    /// High-severity incident that should drive manager review and follow-up labor.
    High,
    /// Critical incident requiring immediate manager attention and strict communication review.
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Review lifecycle for incidents as they move from source report to resolution.
pub enum Status {
    /// Source report exists but facts, category, and communication plan are not yet reviewed.
    Reported,
    /// Manager must review the incident before staff close it or use it in customer messaging.
    NeedsManagerReview,
    /// Staff are collecting evidence; downstream actions should wait for investigation context.
    InvestigationOpen,
    /// Customer-facing communication is drafted or needed but must be approved before sending.
    CustomerMessageReview,
    /// Incident has reviewed resolution evidence, but audit history remains available for future care context.
    Resolved,
    /// Incident lifecycle is closed after required review and communication steps are complete.
    Closed,
    /// New evidence or follow-up reopened the incident and should restore manager attention.
    Reopened,
    /// Legal-sensitive incident must not trigger routine automation or unapproved customer commitments.
    LegalHold,
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 1000),
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
/// Redacted incident summary used as evidence for review, not autonomous advice.
pub struct Summary(String);
