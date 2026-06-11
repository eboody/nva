use bon::Builder;
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Classification {
    VaccineProof,
    Waiver,
    Photo,
    MedicalRecord,
    IncidentEvidence,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Source {
    CustomerUpload,
    StaffScan,
    StaffUpload,
    EmailIngest,
    ProviderPoll,
    ProviderWebhook,
    MigrationImport,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Received,
    QuarantinedRejected,
    Extracting,
    ExtractionFailed,
    AwaitingReview,
    Verified,
    Rejected,
    Superseded,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirusScanStatus {
    Pending,
    Passed,
    Failed,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PiiRedactionStatus {
    NotRequired,
    Pending,
    Redacted,
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
pub struct ContentLengthBytes(u64);

impl ContentLengthBytes {
    pub const fn try_new(value: u64) -> Result<Self, ContentLengthError> {
        if value == 0 {
            return Err(ContentLengthError::EmptyObject);
        }
        Ok(Self(value))
    }

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
pub enum ContentLengthError {
    #[error("document storage evidence must not point at an empty object")]
    EmptyObject,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Sha256Digest(String);

impl Sha256Digest {
    pub fn try_new(value: impl Into<String>) -> Result<Self, Sha256DigestError> {
        let value = value.into().trim().to_ascii_lowercase();
        if value.len() != 64 || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Err(Sha256DigestError::InvalidSha256Hex);
        }
        Ok(Self(value))
    }

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
pub enum Sha256DigestError {
    #[error("document content hashes must be 64 lowercase/uppercase hexadecimal sha256 characters")]
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
pub struct StorageRef {
    pub bucket: StorageBucket,
    pub key: StorageKey,
    pub version: StorageVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct OriginalFile {
    pub filename: FileName,
    pub mime_type: MimeType,
    pub content_length: ContentLengthBytes,
    pub sha256: Sha256Digest,
}
