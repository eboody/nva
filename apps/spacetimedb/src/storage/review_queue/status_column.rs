//! Small SpacetimeDB value columns for review-queue storage rows.
//!
//! These enums are adapter storage values, not domain enums. Codecs convert them
//! explicitly at the boundary before app/domain logic runs.

/// Reviewed feedback outcome encoded in review-queue storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, spacetimedb::SpacetimeType)]
pub enum FeedbackOutcomeColumn {
    /// Cleanup was completed.
    Completed,
    /// Reviewer deferred the work.
    Deferred,
    /// Manager suppressed the recommendation.
    SuppressedByManager,
    /// Reviewer determined the source fact was wrong.
    SourceFactWasWrong,
    /// Reviewer determined the action was not actionable.
    NotActionable,
}

/// Reviewed resolution status encoded in review-queue storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, spacetimedb::SpacetimeType)]
pub enum ResolutionStatusColumn {
    /// Open issue.
    Open,
    /// Acknowledged issue.
    Acknowledged,
    /// Ignored issue.
    Ignored,
    /// Repaired issue.
    Repaired,
    /// Superseded issue.
    Superseded,
}

/// Blocked capture reason encoded in review-queue storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, spacetimedb::SpacetimeType)]
pub enum BlockedActionReasonColumn {
    /// Actor id did not resolve.
    ActorNotFound,
    /// Review queue item did not resolve.
    ReviewQueueItemNotFound,
    /// Actor lacked the required review gate.
    ActorLacksReviewGate,
}
