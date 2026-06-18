//! Document intake, safety review, storage, and retention values.
//!
//! Documents carry vaccine proofs, waivers, medical records, incident evidence, and other source
//! artifacts that staff and agents rely on. The domain separates received/extracted facts from
//! verified facts, records virus/PII handling state, and keeps storage references explicit so
//! automation cannot treat unreviewed uploads as compliance truth.

use bon::Builder;
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Document classification used to route vaccine, waiver, medical, photo, and incident evidence.
pub enum Classification {
    /// Vaccine proof document classification or pipeline state used for review and retention.
    VaccineProof,
    /// Waiver document classification or pipeline state used for review and retention.
    Waiver,
    /// Photo document classification or pipeline state used for review and retention.
    Photo,
    /// Medical record document classification or pipeline state used for review and retention.
    MedicalRecord,
    /// Incident evidence document classification or pipeline state used for review and retention.
    IncidentEvidence,
    /// Non-dog, non-cat pet handled by exception policy.
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Source route through which a document entered the review and storage pipeline.
pub enum Source {
    /// Customer upload document classification or pipeline state used for review and retention.
    CustomerUpload,
    /// Staff scan document classification or pipeline state used for review and retention.
    StaffScan,
    /// Staff upload document classification or pipeline state used for review and retention.
    StaffUpload,
    /// Email ingest document classification or pipeline state used for review and retention.
    EmailIngest,
    /// Provider poll document classification or pipeline state used for review and retention.
    ProviderPoll,
    /// Provider webhook document classification or pipeline state used for review and retention.
    ProviderWebhook,
    /// Migration import document classification or pipeline state used for review and retention.
    MigrationImport,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Normalized lifecycle states used to reconcile source-system data with domain workflows.
pub enum Status {
    /// Received document classification or pipeline state used for review and retention.
    Received,
    /// Quarantined rejected document classification or pipeline state used for review and retention.
    QuarantinedRejected,
    /// Extracting document classification or pipeline state used for review and retention.
    Extracting,
    /// Extraction failed document classification or pipeline state used for review and retention.
    ExtractionFailed,
    /// Awaiting review document classification or pipeline state used for review and retention.
    AwaitingReview,
    /// Verified document classification or pipeline state used for review and retention.
    Verified,
    /// Rejected document classification or pipeline state used for review and retention.
    Rejected,
    /// Superseded document classification or pipeline state used for review and retention.
    Superseded,
    /// Archived document classification or pipeline state used for review and retention.
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Virus-scan outcome used before documents may become staff-visible evidence.
pub enum VirusScanStatus {
    /// Pending document classification or pipeline state used for review and retention.
    Pending,
    /// Passed document classification or pipeline state used for review and retention.
    Passed,
    /// Deposit collection was attempted but did not succeed.
    Failed,
    /// Unsupported document classification or pipeline state used for review and retention.
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// PII redaction state used before document content is exposed to agents or reports.
pub enum PiiRedactionStatus {
    /// No deposit or review is needed for this reservation path.
    NotRequired,
    /// Pending document classification or pipeline state used for review and retention.
    Pending,
    /// Redacted document classification or pipeline state used for review and retention.
    Redacted,
    /// Deposit collection was attempted but did not succeed.
    Failed,
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 255),
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
pub struct FileName(String);

/// MIME type reported for a document before extraction, virus scanning, or storage policy decisions.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
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
pub struct MimeType(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Non-zero document size used to reject empty uploads before extraction or review.
pub struct ContentLengthBytes(u64);

impl ContentLengthBytes {
    /// Promotes boundary input into a validated document domain value.
    pub const fn try_new(value: u64) -> Result<Self, ContentLengthError> {
        if value == 0 {
            return Err(ContentLengthError::EmptyObject);
        }
        Ok(Self(value))
    }

    /// Exposes the validated scalar for serialization and adapter boundaries.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ContentLengthBytes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::try_new(u64::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation error for document size constraints.
pub enum ContentLengthError {
    #[error("document storage evidence must not point at an empty object")]
    /// Signals that object was blank or missing during document validation.
    EmptyObject,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// SHA-256 digest used to detect duplicate, tampered, or drifted document payloads.
pub struct Sha256Digest(String);

impl Sha256Digest {
    /// Validates and creates the document value.
    pub fn try_new(value: impl Into<String>) -> Result<Self, Sha256DigestError> {
        let value = value.into().trim().to_ascii_lowercase();
        if value.len() != 64 || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Err(Sha256DigestError::InvalidSha256Hex);
        }
        Ok(Self(value))
    }

    /// Returns the owned inner string for storage or outbound mapping.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Debug for Sha256Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Sha256Digest(<redacted>)")
    }
}

impl<'de> Deserialize<'de> for Sha256Digest {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::try_new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation error for document hash formatting.
pub enum Sha256DigestError {
    #[error("document content hashes must be 64 lowercase/uppercase hexadecimal sha256 characters")]
    /// Signals that sha256 hex could not be parsed or accepted during document validation.
    InvalidSha256Hex,
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
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
pub struct StorageBucket(String);

/// Storage key for the immutable document object used as review or compliance evidence.
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
pub struct StorageKey(String);

/// Optional object-version marker for document retention and supersession audits.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
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
pub struct StorageVersion(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Storage pointer to the immutable object that backs a reviewed or source document.
pub struct StorageRef {
    /// Bucket fact promoted into this document contract.
    pub bucket: StorageBucket,
    /// Key fact promoted into this document contract.
    pub key: StorageKey,
    /// Version fact promoted into this document contract.
    pub version: StorageVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Original uploaded file metadata preserved for audit, extraction, and staff review.
pub struct OriginalFile {
    /// Filename fact promoted into this document contract.
    pub filename: FileName,
    /// Mime type fact promoted into this document contract.
    pub mime_type: MimeType,
    /// Content length fact promoted into this document contract.
    pub content_length: ContentLengthBytes,
    /// Sha 256 fact promoted into this document contract.
    pub sha256: Sha256Digest,
}
