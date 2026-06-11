use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    SuggestedExtracted,
    PendingReview,
    VerifiedCurrent,
    VerifiedExpired,
    Rejected,
    ExceptionRequested,
    ExceptionApproved,
    Superseded,
}
