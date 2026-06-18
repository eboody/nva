//! Data-quality findings that guard the source-fact → domain → workflow chain.
//!
//! These contracts keep messy provider facts visible instead of silently normalizing them:
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
    /// Stay data-quality finding for cleanup or review.
    Stay,
    /// Source data-quality finding for cleanup or review.
    Source,
    /// Customer record id data-quality finding for cleanup or review.
    CustomerRecordId,
    /// Pet record id data-quality finding for cleanup or review.
    PetRecordId,
    /// Location record id data-quality finding for cleanup or review.
    LocationRecordId,
    /// Service type record id data-quality finding for cleanup or review.
    ServiceTypeRecordId,
    /// Status data-quality finding for cleanup or review.
    Status,
    /// Owner pet relationship data-quality finding for cleanup or review.
    OwnerPetRelationship,
    /// Record id data-quality finding for cleanup or review.
    RecordId,
    /// Endpoint data-quality finding for cleanup or review.
    Endpoint,
    /// Payload hash data-quality finding for cleanup or review.
    PayloadHash,
    /// Raw payload ref data-quality finding for cleanup or review.
    RawPayloadRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reservation fields whose absence or ambiguity can block workflow projections.
pub enum ReservationField {
    /// Customer record id data-quality finding for cleanup or review.
    CustomerRecordId,
    /// Pet record id data-quality finding for cleanup or review.
    PetRecordId,
    /// Location record id data-quality finding for cleanup or review.
    LocationRecordId,
    /// Service type record id data-quality finding for cleanup or review.
    ServiceTypeRecordId,
    /// Status data-quality finding for cleanup or review.
    Status,
    /// Owner pet relationship data-quality finding for cleanup or review.
    OwnerPetRelationship,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Stay/read-model fields checked before analytics and labor views trust a record.
pub enum StayField {
    /// Id data-quality finding for cleanup or review.
    Id,
    /// Pet record id data-quality finding for cleanup or review.
    PetRecordId,
    /// Location record id data-quality finding for cleanup or review.
    LocationRecordId,
    /// Status data-quality finding for cleanup or review.
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Source-ingestion metadata fields needed to preserve provenance and auditability.
pub enum SourceField {
    /// Record id data-quality finding for cleanup or review.
    RecordId,
    /// Endpoint data-quality finding for cleanup or review.
    Endpoint,
    /// Payload hash data-quality finding for cleanup or review.
    PayloadHash,
    /// Raw payload ref data-quality finding for cleanup or review.
    RawPayloadRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Semantic path to the source field that produced a data-quality issue.
pub enum FieldPath {
    /// Reservation record participating in the workflow.
    Reservation(ReservationField),
    /// Stay data-quality finding for cleanup or review.
    Stay(StayField),
    /// Source data-quality finding for cleanup or review.
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
    /// Missing required field data-quality finding for cleanup or review.
    MissingRequiredField {
        /// Field fact promoted into this data quality contract.
        field: FieldPath,
    },
    /// Assumption in force data-quality finding for cleanup or review.
    AssumptionInForce {
        /// Assumption fact promoted into this data quality contract.
        assumption: source::reservation::Assumption,
    },
    /// Unknown source status data-quality finding for cleanup or review.
    UnknownSourceStatus {
        /// Provider status text retained before normalization.
        observed: source::ObservedStatus,
    },
    /// Conflicting timestamps data-quality finding for cleanup or review.
    ConflictingTimestamps,
    /// Duplicate source record data-quality finding for cleanup or review.
    DuplicateSourceRecord,
    /// Ambiguous owner pet relationship data-quality finding for cleanup or review.
    AmbiguousOwnerPetRelationship,
    /// Unmapped service type data-quality finding for cleanup or review.
    UnmappedServiceType,
    /// Location scope ambiguity data-quality finding for cleanup or review.
    LocationScopeAmbiguity,
    /// Payment state conflict data-quality finding for cleanup or review.
    PaymentStateConflict,
    /// Checkout evidence missing data-quality finding for cleanup or review.
    CheckoutEvidenceMissing,
    /// Unclosed reservation data-quality finding for cleanup or review.
    UnclosedReservation,
    /// Incomplete pet profile data-quality finding for cleanup or review.
    IncompletePetProfile,
    /// Missing vaccination record data-quality finding for cleanup or review.
    MissingVaccinationRecord,
    /// Sensitive payload quarantined data-quality finding for cleanup or review.
    SensitivePayloadQuarantined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Severity of a hygiene defect and its effect on read-model or workflow safety.
pub enum Severity {
    /// Informational data-quality finding for cleanup or review.
    Informational,
    /// Warning data-quality finding for cleanup or review.
    Warning,
    /// Blocking data-quality finding for cleanup or review.
    Blocking,
    /// Critical data-quality finding for cleanup or review.
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Lifecycle state for a data-quality issue as staff acknowledge or repair it.
pub enum ResolutionStatus {
    /// Open data-quality finding for cleanup or review.
    Open,
    /// Acknowledged data-quality finding for cleanup or review.
    Acknowledged,
    /// Ignored data-quality finding for cleanup or review.
    Ignored,
    /// Repaired data-quality finding for cleanup or review.
    Repaired,
    /// Superseded data-quality finding for cleanup or review.
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

    /// Returns the kind for this data quality value.
    pub fn kind(&self) -> Kind {
        self.kind.clone()
    }

    /// Returns this data quality value's severity.
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns this data quality value's source system.
    pub const fn source_system(&self) -> source::System {
        self.source_record_ref.system()
    }

    /// Returns this data quality value's source record ref.
    pub const fn source_record_ref(&self) -> &source::RecordRef {
        &self.source_record_ref
    }

    /// Returns this data quality value's provenance.
    pub const fn provenance(&self) -> &source::Provenance {
        &self.provenance
    }

    /// Returns this data quality value's detected at.
    pub const fn detected_at(&self) -> &source::Timestamp {
        &self.detected_at
    }

    /// Returns this data quality value's resolution status.
    pub const fn resolution_status(&self) -> ResolutionStatus {
        self.resolution_status
    }

    /// Returns this data quality value's visible to bi.
    pub const fn visible_to_bi(&self) -> bool {
        self.visible_to_bi
    }

    /// Returns whether this issue must stop automation or projection into labor workflows.
    pub const fn workflow_blocking(&self) -> bool {
        self.workflow_blocking
    }
}
