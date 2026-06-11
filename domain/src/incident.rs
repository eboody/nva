use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Injury,
    Altercation,
    Behavior,
    Medication,
    Escape,
    Property,
    CustomerService,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Reported,
    NeedsManagerReview,
    InvestigationOpen,
    CustomerMessageReview,
    Resolved,
    Closed,
    Reopened,
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
