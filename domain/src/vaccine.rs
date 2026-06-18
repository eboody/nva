use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Normalized reservation states observed during source-data ingestion.
pub enum Status {
    /// Suggested extracted vaccination-document state for compliance review.
    SuggestedExtracted,
    /// Pending review vaccination-document state for compliance review.
    PendingReview,
    /// Verified current vaccination-document state for compliance review.
    VerifiedCurrent,
    /// Verified expired vaccination-document state for compliance review.
    VerifiedExpired,
    /// Rejected vaccination-document state for compliance review.
    Rejected,
    /// Exception requested vaccination-document state for compliance review.
    ExceptionRequested,
    /// Exception approved vaccination-document state for compliance review.
    ExceptionApproved,
    /// Superseded vaccination-document state for compliance review.
    Superseded,
}
