//! Data-quality findings that guard the source-fact → domain → workflow chain.
//!
//! These types keep messy provider facts visible instead of silently normalizing them:
//! blocking issues stop unsafe automation, reviewable issues remain attached to analytics
//! facts and manager briefs, and BI-visible status lets labor-cost reports explain when
//! data hygiene—not actual demand or staffing—is driving an apparent exception.

use serde::{Deserialize, Serialize};

use crate::source;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Path segment vocabulary for source fields that can fail validation.
pub enum FieldSegment {
    /// Reservation record participating in the workflow.
    Reservation,
    /// Stay projection path used when read-model evidence is incomplete or inconsistent.
    Stay,
    /// Source metadata path used when provenance or payload evidence is missing.
    Source,
    /// Customer record id is missing or ambiguous, blocking safe owner communication and merge decisions.
    CustomerRecordId,
    /// Pet record id is missing or ambiguous, blocking care, vaccine, and temperament confidence.
    PetRecordId,
    /// Location record id is missing or ambiguous, blocking resort-specific labor and capacity reporting.
    LocationRecordId,
    /// Service type record id is missing or unmapped, blocking correct service-line grouping.
    ServiceTypeRecordId,
    /// Status value is missing, conflicting, or unmapped, so workflow state must be reviewed.
    Status,
    /// Owner-pet relationship is ambiguous, blocking customer communication and profile cleanup automation.
    OwnerPetRelationship,
    /// Source record id is missing, so the issue cannot be traced back for repair.
    RecordId,
    /// Source endpoint is missing, so adapter evidence cannot be audited confidently.
    Endpoint,
    /// Payload hash is missing, weakening replay and tamper-evidence for source review.
    PayloadHash,
    /// Raw payload reference is missing or quarantined, limiting audit and repair evidence.
    RawPayloadRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reservation fields whose absence or ambiguity can block workflow projections.
pub enum ReservationField {
    /// Customer record id is missing or ambiguous, blocking safe owner communication and merge decisions.
    CustomerRecordId,
    /// Pet record id is missing or ambiguous, blocking care, vaccine, and temperament confidence.
    PetRecordId,
    /// Location record id is missing or ambiguous, blocking resort-specific labor and capacity reporting.
    LocationRecordId,
    /// Service type record id is missing or unmapped, blocking correct service-line grouping.
    ServiceTypeRecordId,
    /// Status value is missing, conflicting, or unmapped, so workflow state must be reviewed.
    Status,
    /// Owner-pet relationship is ambiguous, blocking customer communication and profile cleanup automation.
    OwnerPetRelationship,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Stay/read-model fields checked before analytics and labor views trust a record.
pub enum StayField {
    /// Stay id is missing, so analytics cannot safely join or deduplicate the projected fact.
    Id,
    /// Pet record id is missing or ambiguous, blocking care, vaccine, and temperament confidence.
    PetRecordId,
    /// Location record id is missing or ambiguous, blocking resort-specific labor and capacity reporting.
    LocationRecordId,
    /// Status value is missing, conflicting, or unmapped, so workflow state must be reviewed.
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Source-ingestion metadata fields needed to preserve provenance and auditability.
pub enum SourceField {
    /// Source record id is missing, so the issue cannot be traced back for repair.
    RecordId,
    /// Source endpoint is missing, so adapter evidence cannot be audited confidently.
    Endpoint,
    /// Payload hash is missing, weakening replay and tamper-evidence for source review.
    PayloadHash,
    /// Raw payload reference is missing or quarantined, limiting audit and repair evidence.
    RawPayloadRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Semantic path to the source field that produced a data-quality issue.
pub enum FieldPath {
    /// Reservation record participating in the workflow.
    Reservation(ReservationField),
    /// Stay projection path used when read-model evidence is incomplete or inconsistent.
    Stay(StayField),
    /// Source metadata path used when provenance or payload evidence is missing.
    Source(SourceField),
}

impl FieldPath {
    /// Builds a reservation-field path for a source-data-quality issue.
    pub const fn reservation(field: ReservationField) -> Self {
        Self::Reservation(field)
    }

    /// Builds a stay/read-model-field path for a source-data-quality issue.
    pub const fn stay(field: StayField) -> Self {
        Self::Stay(field)
    }

    /// Builds a source-metadata-field path for a source-data-quality issue.
    pub const fn source(field: SourceField) -> Self {
        Self::Source(field)
    }

    /// Returns stable path segments for repair queues, BI dimensions, and manager review.
    pub const fn segments(&self) -> &'static [FieldSegment] {
        match self {
            Self::Reservation(ReservationField::CustomerRecordId) => {
                &[FieldSegment::Reservation, FieldSegment::CustomerRecordId]
            }
            Self::Reservation(ReservationField::PetRecordId) => {
                &[FieldSegment::Reservation, FieldSegment::PetRecordId]
            }
            Self::Reservation(ReservationField::LocationRecordId) => {
                &[FieldSegment::Reservation, FieldSegment::LocationRecordId]
            }
            Self::Reservation(ReservationField::ServiceTypeRecordId) => {
                &[FieldSegment::Reservation, FieldSegment::ServiceTypeRecordId]
            }
            Self::Reservation(ReservationField::Status) => {
                &[FieldSegment::Reservation, FieldSegment::Status]
            }
            Self::Reservation(ReservationField::OwnerPetRelationship) => &[
                FieldSegment::Reservation,
                FieldSegment::OwnerPetRelationship,
            ],
            Self::Stay(StayField::Id) => &[FieldSegment::Stay, FieldSegment::RecordId],
            Self::Stay(StayField::PetRecordId) => &[FieldSegment::Stay, FieldSegment::PetRecordId],
            Self::Stay(StayField::LocationRecordId) => {
                &[FieldSegment::Stay, FieldSegment::LocationRecordId]
            }
            Self::Stay(StayField::Status) => &[FieldSegment::Stay, FieldSegment::Status],
            Self::Source(SourceField::RecordId) => &[FieldSegment::Source, FieldSegment::RecordId],
            Self::Source(SourceField::Endpoint) => &[FieldSegment::Source, FieldSegment::Endpoint],
            Self::Source(SourceField::PayloadHash) => {
                &[FieldSegment::Source, FieldSegment::PayloadHash]
            }
            Self::Source(SourceField::RawPayloadRef) => {
                &[FieldSegment::Source, FieldSegment::RawPayloadRef]
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Kind of hygiene defect found while validating source facts.
pub enum Kind {
    /// Required source field is absent, so projection or automation must stop until staff/source repair happens.
    MissingRequiredField {
        /// Source field whose missing value or conflict created the data-quality issue.
        field: FieldPath,
    },
    /// Source mapper used an assumption that must remain visible before managers trust derived workflow data.
    AssumptionInForce {
        /// Assumption staff or managers must review before the workflow trusts derived data.
        assumption: source::reservation::Assumption,
    },
    /// Provider status was not mapped to known workflow vocabulary and needs review before automation acts.
    UnknownSourceStatus {
        /// Provider status text retained before normalization.
        observed: source::ObservedStatus,
    },
    /// Source timestamps conflict, so ordering-sensitive stay or checkout decisions need human review.
    ConflictingTimestamps,
    /// Duplicate source record may inflate demand, revenue, or workload until a human verifies the duplicate.
    DuplicateSourceRecord,
    /// Owner-pet relationship ambiguity blocks safe customer messaging and profile merge work.
    AmbiguousOwnerPetRelationship,
    /// Service type cannot be mapped to boarding, daycare, grooming, training, or retail without review.
    UnmappedServiceType,
    /// Location scope is unclear, so regional or local labor reporting may point at the wrong resort.
    LocationScopeAmbiguity,
    /// Payment state conflicts must be reviewed before checkout, invoice, refund, or discount work proceeds.
    PaymentStateConflict,
    /// Checkout evidence is missing, so stay completion and billing summaries cannot be trusted automatically.
    CheckoutEvidenceMissing,
    /// Unclosed reservation may distort occupancy, labor, and checkout queues until staff closes or explains it.
    UnclosedReservation,
    /// Incomplete pet profile blocks reliable care, eligibility, and customer-update drafts.
    IncompletePetProfile,
    /// Missing vaccination record blocks safety-sensitive eligibility claims until documentation is reviewed.
    MissingVaccinationRecord,
    /// Sensitive payload was quarantined, so automation may preserve audit evidence but must not expose secrets or PII.
    SensitivePayloadQuarantined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Severity of a hygiene defect and its effect on read-model or workflow safety.
pub enum Severity {
    /// Informational issue stays visible for audit and trend reporting without blocking workflow use.
    Informational,
    /// Warning issue allows read-model use only with manager-visible context.
    Warning,
    /// Blocking issue stops projection or automation until source evidence is repaired or reviewed.
    Blocking,
    /// Critical issue signals source safety or trust failure that must be escalated before workflow use.
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Lifecycle state for a data-quality issue as staff acknowledge or repair it.
pub enum ResolutionStatus {
    /// Open issue still needs staff or source-system action.
    Open,
    /// Acknowledged issue has been seen by staff but remains part of evidence shown to workflows.
    Acknowledged,
    /// Ignored issue is intentionally bypassed for a workflow, preserving audit context for why.
    Ignored,
    /// Repaired issue records that source or profile cleanup resolved the defect.
    Repaired,
    /// Superseded issue was replaced by fresher evidence or a more precise finding.
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Evidence-backed data-quality issue attached to source records and derived facts.
pub struct Issue {
    kind: Kind,
    severity: Severity,
    provenance: source::Provenance,
    source_record_ref: source::RecordRef,
    detected_at: source::Timestamp,
    resolution_status: ResolutionStatus,
    visible_to_bi: bool,
    workflow_blocking: bool,
}

impl Issue {
    /// Creates an open data-quality issue from provenance and validated issue metadata.
    pub fn new(
        kind: Kind,
        severity: Severity,
        provenance: source::Provenance,
        detected_at: source::Timestamp,
        workflow_blocking: bool,
    ) -> Self {
        let source_record_ref = source::RecordRef::from_provenance(&provenance);
        Self {
            kind,
            severity,
            provenance,
            source_record_ref,
            detected_at,
            resolution_status: ResolutionStatus::Open,
            visible_to_bi: true,
            workflow_blocking,
        }
    }

    /// Returns the concrete hygiene defect that explains why a source fact is blocked or reviewable.
    pub fn kind(&self) -> Kind {
        self.kind.clone()
    }

    /// Returns how strongly the issue should affect projection, manager review, or escalation.
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns the provider system that produced the record needing cleanup or review.
    pub const fn source_system(&self) -> source::System {
        self.source_record_ref.system()
    }

    /// Returns the auditable source record reference staff can use to repair the defect.
    pub const fn source_record_ref(&self) -> &source::RecordRef {
        &self.source_record_ref
    }

    /// Returns the provenance chain that keeps cleanup evidence tied to provider ingestion.
    pub const fn provenance(&self) -> &source::Provenance {
        &self.provenance
    }

    /// Returns when the issue was detected so stale hygiene can be prioritized.
    pub const fn detected_at(&self) -> &source::Timestamp {
        &self.detected_at
    }

    /// Returns whether the hygiene issue is open, acknowledged, ignored, repaired, or superseded.
    pub const fn resolution_status(&self) -> ResolutionStatus {
        self.resolution_status
    }

    /// Returns whether BI/labor reports should show the issue as context for apparent exceptions.
    pub const fn visible_to_bi(&self) -> bool {
        self.visible_to_bi
    }

    /// Returns whether this issue must stop automation or projection into labor workflows.
    pub const fn workflow_blocking(&self) -> bool {
        self.workflow_blocking
    }
}
