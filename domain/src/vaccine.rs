//! Vaccination-document review states for compliance-gated resort workflows.
//!
//! Vaccine facts move from uploaded/source documents into explicit review states before
//! daycare, boarding, or customer-response workflows can rely on them. This keeps staff
//! labor for missing/expired proofs visible and prevents automation from inventing
//! compliance clearance.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Review lifecycle for vaccination evidence before it can satisfy care requirements.
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
