//! Document intake, safety review, storage, and retention values.
//!
//! ## Operator-summary
//!
//! This module supports the document-review queue for vaccine proofs, waivers, medical
//! records, photos, incident evidence, and provider/customer uploads. It can reduce labor
//! by routing files through classification, virus scan, PII redaction, extraction, storage,
//! supersession, and reviewer status instead of making staff manually inspect each upload
//! before it appears in a workflow.
//!
//! It must not automate live compliance clearance, medical acceptance, incident resolution,
//! customer disclosure, provider writes, or retention/destruction decisions. The authoritative
//! source facts remain the immutable stored object, hash, original metadata, source route,
//! scan/redaction results, extraction evidence, reviewer decision, and audit trail. Review
//! gates protect pets, customers, and staff by keeping unscanned, unredacted, failed,
//! unverified, superseded, or rejected documents out of compliance, messaging, and safety
//! decisions until the appropriate staff review is complete.
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
    /// Immunization proof that can satisfy compliance only after scan and reviewer checks pass.
    VaccineProof,
    /// Signed waiver artifact retained as customer consent evidence for staff review.
    Waiver,
    /// Pet or facility image that may support identity, care notes, or customer communication.
    Photo,
    /// Veterinary or medical record that requires review before influencing care decisions.
    MedicalRecord,
    /// Incident attachment preserved as safety and audit evidence for manager follow-up.
    IncidentEvidence,
    /// Non-dog, non-cat pet handled by exception policy.
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Source route through which a document entered the review and storage pipeline.
pub enum Source {
    /// Customer-submitted file that enters quarantine, scan, and reviewer queues before use.
    CustomerUpload,
    /// Paper document digitized by staff and tied to source metadata for auditability.
    StaffScan,
    /// Staff-uploaded file attached to an operational record with reviewer accountability.
    StaffUpload,
    /// Email attachment captured from an inbox before classification, scan, and review.
    EmailIngest,
    /// File discovered through provider polling and reconciled against source-system authority.
    ProviderPoll,
    /// File announced by provider webhook and retained with webhook provenance for audit.
    ProviderWebhook,
    /// Legacy file imported during migration with provenance preserved for cleanup review.
    MigrationImport,
    /// Source route is unknown, so staff should verify document origin before trusting it in workflows.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Normalized lifecycle states used to reconcile source-system data with domain workflows.
pub enum Status {
    /// Stored but not yet scanned or extracted, so it cannot support compliance decisions.
    Received,
    /// Rejected during quarantine and blocked from staff-visible evidence flows.
    QuarantinedRejected,
    /// OCR or metadata extraction is running before reviewer-ready facts exist.
    Extracting,
    /// Extraction failed and requires staff review before the document can provide facts.
    ExtractionFailed,
    /// Scan and extraction evidence is ready but still needs human approval.
    AwaitingReview,
    /// Reviewer-approved document evidence may now support compliance or care workflows.
    Verified,
    /// Reviewer rejected the file, blocking it from compliance and customer messaging.
    Rejected,
    /// Newer evidence replaced this document while the old audit trail remains retained.
    Superseded,
    /// Retained historical document no longer participates in active workflows.
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Virus-scan outcome used before documents may become staff-visible evidence.
pub enum VirusScanStatus {
    /// Scan request is pending, so the file remains blocked from trusted evidence use.
    Pending,
    /// Virus scan passed, allowing the document to continue toward extraction and review.
    Passed,
    /// Virus scan failed, keeping the document quarantined from staff and automation.
    Failed,
    /// File type cannot be scanned by the supported path and needs manual handling.
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// PII redaction state used before document content is exposed to agents or reports.
pub enum PiiRedactionStatus {
    /// Redaction is unnecessary for this document before staff or agent use.
    NotRequired,
    /// Redaction is pending, so extracted content must stay out of agent/report surfaces.
    Pending,
    /// Sensitive content has been redacted for safe staff, agent, or report use.
    Redacted,
    /// Redaction failed, so document content remains blocked until manual review.
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
    /// Rejects unusable document input before extraction, storage, or reviewer queues use it.
    pub const fn try_new(value: u64) -> Result<Self, ContentLengthError> {
        if value == 0 {
            return Err(ContentLengthError::EmptyObject);
        }
        Ok(Self(value))
    }

    /// Returns the checked value for storage, reporting, or adapter output.
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
    /// Bucket preserved with the stored document so reviewers can audit intake, extraction, and retention.
    pub bucket: StorageBucket,
    /// Key preserved with the stored document so reviewers can audit intake, extraction, and retention.
    pub key: StorageKey,
    /// Version preserved with the stored document so reviewers can audit intake, extraction, and retention.
    pub version: StorageVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Original uploaded file metadata preserved for audit, extraction, and staff review.
pub struct OriginalFile {
    /// Filename preserved with the stored document so reviewers can audit intake, extraction, and retention.
    pub filename: FileName,
    /// Mime type preserved with the stored document so reviewers can audit intake, extraction, and retention.
    pub mime_type: MimeType,
    /// Content length preserved with the stored document so reviewers can audit intake, extraction, and retention.
    pub content_length: ContentLengthBytes,
    /// Sha256 preserved with the stored document so reviewers can audit intake, extraction, and retention.
    pub sha256: Sha256Digest,
}
