//! Boarding cancellation policy for notice windows and deposit/refund review.
//!
//! Cancellation penalties are represented as deterministic policy values so staff-facing agents can
//! explain forfeits or manager-review needs without waiving fees or promising exceptions.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Cancellation policy applied to a boarding reservation before staff waive or charge penalties.
pub struct Policy {
    /// Minimum notice the guest must provide before cancellation avoids the configured penalty.
    pub notice: NoticeHours,
    /// Operational consequence when the notice window is missed.
    pub penalty: Penalty,
}

impl Policy {
    /// Creates a cancellation policy from validated notice and penalty values.
    pub const fn new(notice: NoticeHours, penalty: Penalty) -> Self {
        Self { notice, penalty }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Penalties a resort may apply when a boarding cancellation misses the notice window.
pub enum Penalty {
    /// No fee, forfeiture, or manager review is required by this cancellation path.
    None,
    /// The booking deposit should be forfeited unless an approved exception overrides policy.
    ForfeitDeposit,
    /// A manager must review the cancellation before staff promise a fee outcome.
    ManagerReview,
}
