//! Document and vaccine-proof aggregates with checked review state.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use super::{
    actor::ActorRef,
    identifiers::{CustomerId, DocumentId, IncidentId, LocationId, PetId, VaccineRecordId},
    reservation,
};
use crate::{document, policy, vaccine};

/// Document aggregate construction and rehydration failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DocumentError {
    #[error("verified document requires passed virus scan")]
    /// Represents the `VerifiedRequiresPassedVirusScan` semantic case.
    VerifiedRequiresPassedVirusScan,
    #[error("verified document requires safe PII redaction status")]
    /// Represents the `VerifiedRequiresSafePiiRedactionStatus` semantic case.
    VerifiedRequiresSafePiiRedactionStatus,
}

#[derive(Clone, PartialEq, Eq, Serialize)]
/// Document record tying storage, classification, source, scan, redaction, and review status together.
pub struct Document {
    id: DocumentId,
    location_id: LocationId,
    subject: DocumentSubject,
    classification: document::Classification,
    source: document::Source,
    uploaded_by_actor: ActorRef,
    uploaded_at: DateTime<Utc>,
    original_file: document::OriginalFile,
    storage_ref: document::StorageRef,
    virus_scan_status: document::VirusScanStatus,
    pii_redaction_status: document::PiiRedactionStatus,
    verification_status: document::Status,
    audit_refs: Vec<crate::audit::EventId>,
}

#[derive(Deserialize)]
struct RawDocument {
    id: DocumentId,
    location_id: LocationId,
    subject: DocumentSubject,
    classification: document::Classification,
    source: document::Source,
    uploaded_by_actor: ActorRef,
    uploaded_at: DateTime<Utc>,
    original_file: document::OriginalFile,
    storage_ref: document::StorageRef,
    virus_scan_status: document::VirusScanStatus,
    pii_redaction_status: document::PiiRedactionStatus,
    verification_status: document::Status,
    audit_refs: Vec<crate::audit::EventId>,
}

impl RawDocument {
    fn try_into_document(self) -> std::result::Result<Document, DocumentError> {
        if matches!(self.verification_status, document::Status::Verified)
            && self.virus_scan_status != document::VirusScanStatus::Passed
        {
            return Err(DocumentError::VerifiedRequiresPassedVirusScan);
        }
        if matches!(self.verification_status, document::Status::Verified)
            && !matches!(
                self.pii_redaction_status,
                document::PiiRedactionStatus::NotRequired | document::PiiRedactionStatus::Redacted
            )
        {
            return Err(DocumentError::VerifiedRequiresSafePiiRedactionStatus);
        }
        Ok(Document {
            id: self.id,
            location_id: self.location_id,
            subject: self.subject,
            classification: self.classification,
            source: self.source,
            uploaded_by_actor: self.uploaded_by_actor,
            uploaded_at: self.uploaded_at,
            original_file: self.original_file,
            storage_ref: self.storage_ref,
            virus_scan_status: self.virus_scan_status,
            pii_redaction_status: self.pii_redaction_status,
            verification_status: self.verification_status,
            audit_refs: self.audit_refs,
        })
    }
}

impl<'de> Deserialize<'de> for Document {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RawDocument::deserialize(deserializer)?
            .try_into_document()
            .map_err(serde::de::Error::custom)
    }
}

impl Document {
    /// Returns the aggregate id.
    pub fn id(&self) -> DocumentId {
        self.id
    }
    /// Returns the aggregate location id.
    pub fn location_id(&self) -> LocationId {
        self.location_id
    }
    /// Returns the aggregate subject.
    pub fn subject(&self) -> &DocumentSubject {
        &self.subject
    }
    /// Returns the aggregate classification.
    pub fn classification(&self) -> document::Classification {
        self.classification
    }
    /// Returns the aggregate source.
    pub fn source(&self) -> document::Source {
        self.source
    }
    /// Returns the aggregate uploaded by actor.
    pub fn uploaded_by_actor(&self) -> &ActorRef {
        &self.uploaded_by_actor
    }
    /// Returns the aggregate uploaded at.
    pub fn uploaded_at(&self) -> DateTime<Utc> {
        self.uploaded_at
    }
    /// Returns the aggregate original file.
    pub fn original_file(&self) -> &document::OriginalFile {
        &self.original_file
    }
    /// Returns the aggregate storage ref.
    pub fn storage_ref(&self) -> &document::StorageRef {
        &self.storage_ref
    }
    /// Returns the aggregate virus scan status.
    pub fn virus_scan_status(&self) -> document::VirusScanStatus {
        self.virus_scan_status
    }
    /// Returns the aggregate pii redaction status.
    pub fn pii_redaction_status(&self) -> document::PiiRedactionStatus {
        self.pii_redaction_status
    }
    /// Returns the aggregate verification status.
    pub fn verification_status(&self) -> document::Status {
        self.verification_status
    }
    /// Returns the aggregate audit refs.
    pub fn audit_refs(&self) -> &[crate::audit::EventId] {
        &self.audit_refs
    }
    /// Returns the aggregate requires human review before use.
    pub fn requires_human_review_before_use(&self) -> bool {
        matches!(
            self.verification_status,
            document::Status::Received
                | document::Status::Extracting
                | document::Status::ExtractionFailed
                | document::Status::AwaitingReview
                | document::Status::QuarantinedRejected
        ) || !matches!(self.virus_scan_status, document::VirusScanStatus::Passed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Entity or workflow subject a document is evidence for.
pub enum DocumentSubject {
    /// Customer record participating in the workflow.
    Customer(CustomerId),
    /// Pet record participating in the workflow.
    Pet(PetId),
    /// Reservation record participating in the workflow.
    Reservation(reservation::Id),
    /// Incident record participating in the workflow.
    Incident(IncidentId),
}

/// Vaccine-record aggregate construction and rehydration failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VaccineRecordError {
    #[error("vaccine record id is required")]
    /// Represents the `IdRequired` semantic case.
    IdRequired,
    #[error("vaccine record pet id is required")]
    /// Represents the `PetIdRequired` semantic case.
    PetIdRequired,
    #[error("vaccine name is required")]
    /// Represents the `VaccineNameRequired` semantic case.
    VaccineNameRequired,
    #[error("vaccine source document id is required")]
    /// Represents the `SourceDocumentIdRequired` semantic case.
    SourceDocumentIdRequired,
    #[error("vaccine status is required")]
    /// Represents the `StatusRequired` semantic case.
    StatusRequired,
    #[error("vaccine effective date is required")]
    /// Represents the `EffectiveOnRequired` semantic case.
    EffectiveOnRequired,
    #[error("vaccine review gate is required")]
    /// Represents the `ReviewGateRequired` semantic case.
    ReviewGateRequired,
    #[error("vaccine expiration date must be after effective date")]
    /// Represents the `ExpirationMustBeAfterEffectiveDate` semantic case.
    ExpirationMustBeAfterEffectiveDate,
    #[error("expired vaccine status requires an expiration date")]
    /// Represents the `ExpiredStatusRequiresExpirationDate` semantic case.
    ExpiredStatusRequiresExpirationDate,
    #[error("vaccine exception status requires manager approval review gate")]
    /// Represents the `ExceptionStatusRequiresManagerApprovalReviewGate` semantic case.
    ExceptionStatusRequiresManagerApprovalReviewGate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Vaccine compliance record linking pet, vaccine name, expiration, proof document, and review status.
pub struct VaccineRecord {
    id: VaccineRecordId,
    pet_id: PetId,
    vaccine_name: policy::VaccineName,
    source_document_id: DocumentId,
    status: vaccine::Status,
    effective_on: NaiveDate,
    expires_on: Option<NaiveDate>,
    review_gate: policy::ReviewGate,
    audit_refs: Vec<crate::audit::EventId>,
}

#[derive(Deserialize)]
struct RawVaccineRecord {
    id: VaccineRecordId,
    pet_id: PetId,
    vaccine_name: policy::VaccineName,
    source_document_id: DocumentId,
    status: vaccine::Status,
    effective_on: NaiveDate,
    expires_on: Option<NaiveDate>,
    review_gate: policy::ReviewGate,
    audit_refs: Vec<crate::audit::EventId>,
}

impl RawVaccineRecord {
    fn try_into_record(self) -> std::result::Result<VaccineRecord, VaccineRecordError> {
        if self
            .expires_on
            .is_some_and(|expires_on| expires_on <= self.effective_on)
        {
            return Err(VaccineRecordError::ExpirationMustBeAfterEffectiveDate);
        }
        if matches!(self.status, vaccine::Status::VerifiedExpired) && self.expires_on.is_none() {
            return Err(VaccineRecordError::ExpiredStatusRequiresExpirationDate);
        }
        if matches!(
            self.status,
            vaccine::Status::ExceptionApproved | vaccine::Status::ExceptionRequested
        ) && self.review_gate != policy::ReviewGate::ManagerApproval
        {
            return Err(VaccineRecordError::ExceptionStatusRequiresManagerApprovalReviewGate);
        }
        Ok(VaccineRecord {
            id: self.id,
            pet_id: self.pet_id,
            vaccine_name: self.vaccine_name,
            source_document_id: self.source_document_id,
            status: self.status,
            effective_on: self.effective_on,
            expires_on: self.expires_on,
            review_gate: self.review_gate,
            audit_refs: self.audit_refs,
        })
    }
}

impl<'de> Deserialize<'de> for VaccineRecord {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RawVaccineRecord::deserialize(deserializer)?
            .try_into_record()
            .map_err(serde::de::Error::custom)
    }
}

impl VaccineRecord {
    /// Returns the aggregate id.
    pub fn id(&self) -> VaccineRecordId {
        self.id
    }
    /// Returns the aggregate pet id.
    pub fn pet_id(&self) -> PetId {
        self.pet_id
    }
    /// Returns the aggregate vaccine name.
    pub fn vaccine_name(&self) -> &policy::VaccineName {
        &self.vaccine_name
    }
    /// Returns the aggregate source document id.
    pub fn source_document_id(&self) -> DocumentId {
        self.source_document_id
    }
    /// Returns the aggregate status.
    pub fn status(&self) -> vaccine::Status {
        self.status
    }
    /// Returns the aggregate effective on.
    pub fn effective_on(&self) -> NaiveDate {
        self.effective_on
    }
    /// Returns the aggregate expires on.
    pub fn expires_on(&self) -> Option<NaiveDate> {
        self.expires_on
    }
    /// Promotes the stored review gate into the semantic application value.
    pub fn review_gate(&self) -> policy::ReviewGate {
        self.review_gate.clone()
    }
    /// Returns the aggregate audit refs.
    pub fn audit_refs(&self) -> &[crate::audit::EventId] {
        &self.audit_refs
    }
    /// Reports whether vaccine proof is still unverified, rejected, or otherwise unsafe for compliance automation.
    pub fn requires_human_review_before_compliance(&self) -> bool {
        matches!(
            self.status,
            vaccine::Status::SuggestedExtracted
                | vaccine::Status::PendingReview
                | vaccine::Status::Rejected
                | vaccine::Status::ExceptionRequested
        )
    }
}
