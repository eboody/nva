//! Crate-owned contracts shared by storage projection boundaries.
//!
//! Operations repositories and service-line codecs both depend on these error
//! and diagnostic values. Keeping them at their narrowest common storage owner
//! prevents either sibling boundary from depending back on the other.

/// Result type returned by fallible storage projection and codec operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
/// Errors raised while validating storage records, codecs, or domain-to-storage projection.
pub enum Error {
    #[error("storage codec error: {0}")]
    /// Wraps a storage JSON codec failure without losing the underlying source error.
    Codec(#[from] CodecError),
    #[error("domain value rejected storage field {field:?}: {reason}")]
    /// Signals that a domain value cannot be represented safely in storage.
    InvalidDomainValue {
        /// Storage field whose value failed projection or validation.
        field: StorageField,
        /// Human-readable reason explaining why the storage projection was unsafe.
        reason: String,
    },
    #[error("current reviewer capability cannot authorize approval outbox admission: {reason:?}")]
    /// The current reviewer, approved row, or bound internal handoff did not match.
    ApprovalOutboxAuthority {
        /// Stable mismatch category without leaking actor ids or payloads.
        reason: ApprovalOutboxAuthorityMismatch,
    },
    #[error("approval already has a pending outbox admission")]
    /// One projection attempted to consume more than one admission authority.
    ApprovalOutboxAlreadyAdmitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Stable reasons an approved persistence row cannot issue outbox admission authority.
pub enum ApprovalOutboxAuthorityMismatch {
    /// The supplied current role is not allowed to decide the approval gate.
    CurrentRole,
    /// The current actor is not the actor retained by the approval decision.
    CurrentActor,
    /// The persisted approval row is not approved.
    ApprovalStatus,
    /// Approval id, gate, or target differs across the approval and binding rows.
    ApprovalRelation,
    /// The requested closed topic or payload differs from the reviewed binding.
    InternalHandoff,
}

#[derive(Debug, thiserror::Error)]
/// JSON codec failures at the storage gate.
pub enum CodecError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Reasons a flattened record cannot represent the requested domain variant.
pub enum ShapeMismatchReason {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Persisted fields that can reject invalid domain values during storage conversion.
pub enum StorageField {
    /// Resort-count field promoted into a positive domain count.
    ResortCount,
    /// Freeform brand-name field preserved for non-enumerated pet-resort banners.
    BrandName,
    /// Manager daily-brief labor-minute field used for before/after evidence.
    ManagerDailyBriefLaborMinutes,
    /// Data-quality hygiene labor-minute field used for before/after evidence.
    DataQualityHygieneLaborMinutes,
}
