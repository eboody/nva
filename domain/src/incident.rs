use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for category decisions in incident workflows.
pub enum Category {
    /// Injury incident category for safety and customer follow-up.
    Injury,
    /// Altercation incident category for safety and customer follow-up.
    Altercation,
    /// Behavior incident category for safety and customer follow-up.
    Behavior,
    /// Medication incident category for safety and customer follow-up.
    Medication,
    /// Escape incident category for safety and customer follow-up.
    Escape,
    /// Property incident category for safety and customer follow-up.
    Property,
    /// Customer service incident category for safety and customer follow-up.
    CustomerService,
    /// Non-dog, non-cat pet handled by exception policy.
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for severity decisions in incident workflows.
pub enum Severity {
    /// Estimate is uncertain and may require staff confirmation.
    Low,
    /// Estimate is usable but should be treated with moderate uncertainty.
    Medium,
    /// Estimate is reliable enough for normal scheduling.
    High,
    /// Critical incident category for safety and customer follow-up.
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Normalized reservation states observed during source-data ingestion.
pub enum Status {
    /// Reported incident category for safety and customer follow-up.
    Reported,
    /// Needs manager review incident category for safety and customer follow-up.
    NeedsManagerReview,
    /// Investigation open incident category for safety and customer follow-up.
    InvestigationOpen,
    /// Customer message review incident category for safety and customer follow-up.
    CustomerMessageReview,
    /// Owner-pet relationship was matched to a single confident record.
    Resolved,
    /// Closed incident category for safety and customer follow-up.
    Closed,
    /// Reopened incident category for safety and customer follow-up.
    Reopened,
    /// Legal hold incident category for safety and customer follow-up.
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
pub struct Summary(String);
