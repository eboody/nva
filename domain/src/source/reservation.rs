use serde::{Deserialize, Serialize};

use crate::source;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Confidence in the provider relationship between an owner and pet.
pub enum OwnerPetRelationship {
    /// Owner-pet relationship was matched to a single confident record.
    Resolved,
    /// Multiple provider owner/pet records could match, blocking confident promotion until reviewed.
    Ambiguous {
        /// Number of possible provider matches reviewers must reconcile before promotion.
        candidate_count: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Normalized lifecycle states used to reconcile source-system data with domain workflows.
pub enum Status {
    /// Reservation has been requested but not yet confirmed.
    Requested,
    /// Reservation has been accepted by the resort.
    Confirmed,
    /// Pet has arrived and is in care.
    CheckedIn,
    /// Pet has left care and the stay is complete.
    CheckedOut,
    /// Provider cancellation or void status blocks active booking workflows while preserving source status for reconciliation and review.
    Cancelled,
    /// Provider status was not recognized; retain observed text as a promotion blocker.
    Unknown {
        /// Raw provider status text reviewers must map or reject before workflow promotion.
        observed: source::ObservedStatus,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Explicit ingestion assumptions made while normalizing provider data.
pub enum Assumption {
    /// Provider row grain is interpreted as a reservation snapshot for normalization.
    GrainTreatedAsReservation,
    /// Customer provider record id is assumed stable enough for reconciliation.
    CustomerRecordIdTreatedAsStableJoinKey,
    /// Pet provider record id is assumed stable enough for reconciliation.
    PetRecordIdTreatedAsStableJoinKey,
    /// Status mapping is provisional and should stay visible to reviewer/data-quality workflows.
    ProviderStatusMappingIsProvisional,
    /// Raw-payload retention policy is unknown and should be treated as a data-quality warning.
    RawPayloadRetentionUnknown,
    /// Provider refresh mutation behavior is unknown and can block confident promotion.
    RefreshMutationPolicyUnknown,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Point-in-time source-data view used before promotion into core domain records.
pub struct Snapshot {
    provenance: source::Provenance,
    customer_record_id: Option<source::record::Id>,
    pet_record_id: Option<source::record::Id>,
    location_record_id: Option<source::record::Id>,
    service_type_record_id: Option<source::record::Id>,
    status: Option<Status>,
    relationship: OwnerPetRelationship,
    assumptions: Vec<Assumption>,
}

impl Snapshot {
    /// Source-system evidence for this snapshot.
    pub const fn provenance(&self) -> &source::Provenance {
        &self.provenance
    }

    /// Provider/read-model customer identifier retained for reconciliation.
    pub const fn customer_record_id(&self) -> Option<&source::record::Id> {
        self.customer_record_id.as_ref()
    }

    /// Provider/read-model pet identifier retained for reconciliation.
    pub const fn pet_record_id(&self) -> Option<&source::record::Id> {
        self.pet_record_id.as_ref()
    }

    /// Provider/read-model location identifier retained for reconciliation.
    pub const fn location_record_id(&self) -> Option<&source::record::Id> {
        self.location_record_id.as_ref()
    }

    /// Provider/read-model service-type identifier retained for reconciliation.
    pub const fn service_type_record_id(&self) -> Option<&source::record::Id> {
        self.service_type_record_id.as_ref()
    }

    /// Normalized reservation lifecycle status preserved for booking promotion or exception review.
    pub fn status(&self) -> Option<Status> {
        self.status.clone()
    }

    /// Owner/pet relationship confidence that can block promotion when ambiguous.
    pub const fn relationship(&self) -> &OwnerPetRelationship {
        &self.relationship
    }

    /// Ingestion assumptions reviewers must accept, reject, or keep visible before promotion.
    pub fn assumptions(&self) -> &[Assumption] {
        &self.assumptions
    }
}
