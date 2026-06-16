use serde::{Deserialize, Serialize};

use crate::source;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceField {
    CustomerRecordId,
    PetRecordId,
    LocationRecordId,
    ServiceTypeRecordId,
    ReservationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    MissingRequiredField {
        field: SourceField,
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
        Self {
            kind,
            severity,
            provenance,
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
        self.provenance.source_system()
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
