use bon::Builder;
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for classification decisions in document workflows.
pub enum Classification {
    /// Vaccine proof document type, review state, or retention signal.
    VaccineProof,
    /// Waiver document type, review state, or retention signal.
    Waiver,
    /// Photo document type, review state, or retention signal.
    Photo,
    /// Medical record document type, review state, or retention signal.
    MedicalRecord,
    /// Incident evidence document type, review state, or retention signal.
    IncidentEvidence,
    /// Non-dog, non-cat pet handled by exception policy.
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for source decisions in document workflows.
pub enum Source {
    /// Customer upload document type, review state, or retention signal.
    CustomerUpload,
    /// Staff scan document type, review state, or retention signal.
    StaffScan,
    /// Staff upload document type, review state, or retention signal.
    StaffUpload,
    /// Email ingest document type, review state, or retention signal.
    EmailIngest,
    /// Provider poll document type, review state, or retention signal.
    ProviderPoll,
    /// Provider webhook document type, review state, or retention signal.
    ProviderWebhook,
    /// Migration import document type, review state, or retention signal.
    MigrationImport,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Normalized reservation states observed during source-data ingestion.
pub enum Status {
    /// Received document type, review state, or retention signal.
    Received,
    /// Quarantined rejected document type, review state, or retention signal.
    QuarantinedRejected,
    /// Extracting document type, review state, or retention signal.
    Extracting,
    /// Extraction failed document type, review state, or retention signal.
    ExtractionFailed,
    /// Awaiting review document type, review state, or retention signal.
    AwaitingReview,
    /// Verified document type, review state, or retention signal.
    Verified,
    /// Rejected document type, review state, or retention signal.
    Rejected,
    /// Superseded document type, review state, or retention signal.
    Superseded,
    /// Archived document type, review state, or retention signal.
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for virus scan status decisions in document workflows.
pub enum VirusScanStatus {
    /// Pending document type, review state, or retention signal.
    Pending,
    /// Passed document type, review state, or retention signal.
    Passed,
    /// Deposit collection was attempted but did not succeed.
    Failed,
    /// Unsupported document type, review state, or retention signal.
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for pii redaction status decisions in document workflows.
pub enum PiiRedactionStatus {
    /// No deposit or review is needed for this reservation path.
    NotRequired,
    /// Pending document type, review state, or retention signal.
    Pending,
    /// Redacted document type, review state, or retention signal.
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
/// Typed content length bytes domain value that keeps raw primitives out of document workflows.
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
/// Domain vocabulary for content length error decisions in document workflows.
pub enum ContentLengthError {
    #[error("document storage evidence must not point at an empty object")]
    /// Signals that object was blank or missing during document validation.
    EmptyObject,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Typed sha 256 digest domain value that keeps raw primitives out of document workflows.
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
/// Domain vocabulary for sha 256 digest error decisions in document workflows.
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
/// Typed storage ref domain value that keeps raw primitives out of document workflows.
pub struct StorageRef {
    /// Bucket fact promoted into this document contract.
    pub bucket: StorageBucket,
    /// Key fact promoted into this document contract.
    pub key: StorageKey,
    /// Version fact promoted into this document contract.
    pub version: StorageVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Typed original file domain value that keeps raw primitives out of document workflows.
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
