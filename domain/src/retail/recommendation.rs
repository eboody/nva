//! Retail recommendation models for personalized upsell candidates, review gates, and safe customer copy.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Recommendation rule that names the operational event that can produce an upsell candidate.
pub enum Rule {
    /// No recommendation rule is active, so no upsell candidate should be produced from this rule alone.
    None,
    /// Boarding stay may justify an internal anxiety-support upsell candidate after inventory and care checks.
    AnxietySupportAfterBoarding,
    /// Boarding diet history may justify a continuity recommendation when stock and care policy allow it.
    DietSupportAfterBoarding,
    /// Grooming outcome may justify a coat-care upsell candidate after staff review gates are satisfied.
    CoatCareAfterGrooming,
}
