//! Vaccination-document review states for compliance-gated resort workflows.
//!
//! ## Operator-summary
//!
//! This module supports the vaccine/requirements queue that decides whether a pet's
//! proof is suggested, pending review, current, expired, rejected, exception-requested,
//! exception-approved, or superseded before daycare, boarding, or customer follow-up uses
//! it. It can reduce labor by making missing or expired proof, extraction suggestions, and
//! exception requests explicit instead of burying them in uploaded documents or staff notes.
//!
//! It must not automate live vaccine acceptance, policy exceptions, group-play clearance,
//! booking confirmation, or customer messaging. Authoritative facts remain the reviewed
//! vaccine document, local location policy, vaccine name/requirement, effective/expiration
//! dates, staff reviewer decision, and audit trail. Review gates protect pets, customers,
//! and staff by requiring medical-document or manager review before uncertain, expired,
//! rejected, or exception-based evidence can satisfy compliance.
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
