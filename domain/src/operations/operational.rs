use super::*;

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
/// Human-readable operational observation attached to evidence-backed workflows.
///
/// Observations describe what a source/read-model chain found—such as labor
/// mismatch, customer-experience risk, or revenue leakage—without granting
/// an agent authority to act without the target workflow gate.
pub struct Observation(String);

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
/// Human-readable recommendation proposed for staff or manager review.
///
/// Recommendations are labor-cost levers only after the surrounding workflow
/// decides whether they remain drafts, become staff tasks, or require manager
/// approval.
pub struct Recommendation(String);
