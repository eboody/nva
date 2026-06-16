use serde::{Deserialize, Serialize};

use crate::source;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldSegment {
    Reservation,
    Stay,
    Source,
    CustomerRecordId,
    PetRecordId,
    LocationRecordId,
    ServiceTypeRecordId,
    Status,
    OwnerPetRelationship,
    RecordId,
    Endpoint,
    PayloadHash,
    RawPayloadRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReservationField {
    CustomerRecordId,
    PetRecordId,
    LocationRecordId,
    ServiceTypeRecordId,
    Status,
    OwnerPetRelationship,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StayField {
    Id,
    PetRecordId,
    LocationRecordId,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceField {
    RecordId,
    Endpoint,
    PayloadHash,
    RawPayloadRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldPath {
    Reservation(ReservationField),
    Stay(StayField),
    Source(SourceField),
}

impl FieldPath {
    pub const fn reservation(field: ReservationField) -> Self {
        Self::Reservation(field)
    }

    pub const fn stay(field: StayField) -> Self {
        Self::Stay(field)
    }

    pub const fn source(field: SourceField) -> Self {
        Self::Source(field)
    }

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
pub enum Kind {
    MissingRequiredField {
        field: FieldPath,
    },
    AssumptionInForce {
        assumption: source::reservation::Assumption,
    },
    UnknownSourceStatus {
        observed: source::ObservedStatus,
    },
    ConflictingTimestamps,
    DuplicateSourceRecord,
    AmbiguousOwnerPetRelationship,
    UnmappedServiceType,
    LocationScopeAmbiguity,
    PaymentStateConflict,
    CheckoutEvidenceMissing,
    UnclosedReservation,
    IncompletePetProfile,
    MissingVaccinationRecord,
    SensitivePayloadQuarantined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Informational,
    Warning,
    Blocking,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStatus {
    Open,
    Acknowledged,
    Ignored,
    Repaired,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn kind(&self) -> Kind {
        self.kind.clone()
    }

    pub const fn severity(&self) -> Severity {
        self.severity
    }

    pub const fn source_system(&self) -> source::System {
        self.source_record_ref.system()
    }

    pub const fn source_record_ref(&self) -> &source::RecordRef {
        &self.source_record_ref
    }

    pub const fn provenance(&self) -> &source::Provenance {
        &self.provenance
    }

    pub const fn detected_at(&self) -> &source::Timestamp {
        &self.detected_at
    }

    pub const fn resolution_status(&self) -> ResolutionStatus {
        self.resolution_status
    }

    pub const fn visible_to_bi(&self) -> bool {
        self.visible_to_bi
    }

    pub const fn workflow_blocking(&self) -> bool {
        self.workflow_blocking
    }
}
