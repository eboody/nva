//! Source-system provenance and record references for app-owned operational facts.
//!
//! Provenance travels with facts so an agent draft can cite the app-owned source evidence it used:
//!
//! ```
//! use domain::source;
//!
//! let provenance = source::Provenance::builder()
//!     .system(source::System::Gingr)
//!     .endpoint(source::Endpoint::try_new("/reservations").unwrap())
//!     .record_id(source::record::Id::try_new("reservation-123").unwrap())
//!     .extraction_batch(source::ExtractionBatchId::try_new("batch-2026-06-18").unwrap())
//!     .pulled_at(source::Timestamp::try_new("2026-06-18T13:00:00Z").unwrap())
//!     .request_scope(source::RequestScope::try_new("manager-daily-brief:loc-1").unwrap())
//!     .schema_version(source::SchemaVersion::try_new("gingr-reservations-v1").unwrap())
//!     .payload_hash(source::PayloadHash::try_new("sha256:fixture").unwrap())
//!     .raw_payload_ref(source::RawPayloadRef::try_new("minio://fixtures/reservation-123.json").unwrap())
//!     .build();
//!
//! let record_ref = source::RecordRef::from_provenance(&provenance);
//! assert_eq!(record_ref.system(), source::System::Gingr);
//! assert_eq!(record_ref.record_id().as_str(), "reservation-123");
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum System {
    Gingr,
    BusinessIntelligence,
    LaborScheduling,
    Timeclock,
    Payroll,
    CapacityInventory,
    PointOfSale,
    ManualImport,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn try_new(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref().trim();
        if value.is_empty() {
            return Err(Error::EmptyTimestamp);
        }
        let parsed = value
            .parse::<DateTime<Utc>>()
            .map_err(|_| Error::InvalidTimestamp)?;
        Ok(Self(parsed))
    }

    pub const fn get(&self) -> &DateTime<Utc> {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Endpoint(String);

impl Endpoint {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyEndpoint).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ExtractionBatchId(String);

impl ExtractionBatchId {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyExtractionBatch).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RequestScope(String);

impl RequestScope {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyRequestScope).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SchemaVersion(String);

impl SchemaVersion {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptySchemaVersion).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PayloadHash(String);

impl PayloadHash {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyPayloadHash).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RawPayloadRef(String);

impl RawPayloadRef {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyRawPayloadRef).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObservedStatus(String);

impl ObservedStatus {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyObservedStatus).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub mod record {
    use serde::{Deserialize, Serialize};

    use crate::source;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Id(String);

    impl Id {
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyRecordId).map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Role {
        Customer,
        Pet,
        Location,
        ReservationType,
        Invoice,
        Payment,
        Service,
        Staff,
        Unknown,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct RelatedId {
        role: Role,
        id: Id,
    }

    impl RelatedId {
        pub const fn new(role: Role, id: Id) -> Self {
            Self { role, id }
        }

        pub const fn role(&self) -> Role {
            self.role
        }

        pub const fn id(&self) -> &Id {
            &self.id
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordRef {
    system: System,
    record_id: record::Id,
}

impl RecordRef {
    pub const fn new(system: System, record_id: record::Id) -> Self {
        Self { system, record_id }
    }

    pub fn from_provenance(provenance: &Provenance) -> Self {
        Self::new(provenance.system(), provenance.record_id().clone())
    }

    pub const fn system(&self) -> System {
        self.system
    }

    pub const fn record_id(&self) -> &record::Id {
        &self.record_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct Provenance {
    system: System,
    endpoint: Endpoint,
    record_id: record::Id,
    #[builder(default)]
    related_record_ids: Vec<record::RelatedId>,
    extraction_batch: ExtractionBatchId,
    pulled_at: Timestamp,
    request_scope: RequestScope,
    schema_version: SchemaVersion,
    payload_hash: PayloadHash,
    raw_payload_ref: RawPayloadRef,
}

impl Provenance {
    pub const fn system(&self) -> System {
        self.system
    }

    pub const fn source_system(&self) -> System {
        self.system
    }

    pub const fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    pub const fn record_id(&self) -> &record::Id {
        &self.record_id
    }

    pub fn related_record_ids(&self) -> &[record::RelatedId] {
        &self.related_record_ids
    }

    pub const fn extraction_batch(&self) -> &ExtractionBatchId {
        &self.extraction_batch
    }

    pub const fn pulled_at(&self) -> &Timestamp {
        &self.pulled_at
    }

    pub const fn request_scope(&self) -> &RequestScope {
        &self.request_scope
    }

    pub const fn schema_version(&self) -> &SchemaVersion {
        &self.schema_version
    }

    pub const fn payload_hash(&self) -> &PayloadHash {
        &self.payload_hash
    }

    pub const fn raw_payload_ref(&self) -> &RawPayloadRef {
        &self.raw_payload_ref
    }
}

pub mod reservation {
    use serde::{Deserialize, Serialize};

    use crate::{data_quality, source};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum OwnerPetRelationship {
        Resolved,
        Ambiguous { candidate_count: u16 },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Status {
        Requested,
        Confirmed,
        CheckedIn,
        CheckedOut,
        Cancelled,
        Unknown { observed: source::ObservedStatus },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Assumption {
        GrainTreatedAsReservation,
        CustomerRecordIdTreatedAsStableJoinKey,
        PetRecordIdTreatedAsStableJoinKey,
        ProviderStatusMappingIsProvisional,
        RawPayloadRetentionUnknown,
        RefreshMutationPolicyUnknown,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        pub const fn builder() -> SnapshotBuilder {
            SnapshotBuilder::new()
        }

        pub const fn provenance(&self) -> &source::Provenance {
            &self.provenance
        }

        pub const fn customer_record_id(&self) -> Option<&source::record::Id> {
            self.customer_record_id.as_ref()
        }

        pub const fn pet_record_id(&self) -> Option<&source::record::Id> {
            self.pet_record_id.as_ref()
        }

        pub const fn location_record_id(&self) -> Option<&source::record::Id> {
            self.location_record_id.as_ref()
        }

        pub const fn service_type_record_id(&self) -> Option<&source::record::Id> {
            self.service_type_record_id.as_ref()
        }

        pub fn status(&self) -> Option<Status> {
            self.status.clone()
        }

        pub const fn relationship(&self) -> &OwnerPetRelationship {
            &self.relationship
        }

        pub fn assumptions(&self) -> &[Assumption] {
            &self.assumptions
        }

        pub fn data_quality_issues(
            &self,
            detected_at: source::Timestamp,
        ) -> Vec<data_quality::Issue> {
            let mut issues = Vec::new();
            self.push_missing_issue(
                &mut issues,
                self.customer_record_id.is_none(),
                data_quality::FieldPath::reservation(
                    data_quality::ReservationField::CustomerRecordId,
                ),
                detected_at.clone(),
            );
            self.push_missing_issue(
                &mut issues,
                self.pet_record_id.is_none(),
                data_quality::FieldPath::reservation(data_quality::ReservationField::PetRecordId),
                detected_at.clone(),
            );
            self.push_missing_issue(
                &mut issues,
                self.location_record_id.is_none(),
                data_quality::FieldPath::reservation(
                    data_quality::ReservationField::LocationRecordId,
                ),
                detected_at.clone(),
            );
            self.push_missing_issue(
                &mut issues,
                self.service_type_record_id.is_none(),
                data_quality::FieldPath::reservation(
                    data_quality::ReservationField::ServiceTypeRecordId,
                ),
                detected_at.clone(),
            );
            if self.status.is_none() {
                self.push_missing_issue(
                    &mut issues,
                    true,
                    data_quality::FieldPath::reservation(data_quality::ReservationField::Status),
                    detected_at.clone(),
                );
                issues.push(data_quality::Issue::new(
                    data_quality::Kind::AssumptionInForce {
                        assumption: Assumption::RefreshMutationPolicyUnknown,
                    },
                    data_quality::Severity::Blocking,
                    self.provenance.clone(),
                    detected_at.clone(),
                    true,
                ));
            }
            if let Some(Status::Unknown { observed }) = &self.status {
                issues.push(data_quality::Issue::new(
                    data_quality::Kind::UnknownSourceStatus {
                        observed: observed.clone(),
                    },
                    data_quality::Severity::Blocking,
                    self.provenance.clone(),
                    detected_at.clone(),
                    true,
                ));
            }
            if matches!(self.relationship, OwnerPetRelationship::Ambiguous { .. }) {
                issues.push(data_quality::Issue::new(
                    data_quality::Kind::AmbiguousOwnerPetRelationship,
                    data_quality::Severity::Blocking,
                    self.provenance.clone(),
                    detected_at.clone(),
                    true,
                ));
            }
            for assumption in &self.assumptions {
                if matches!(
                    assumption,
                    Assumption::RawPayloadRetentionUnknown
                        | Assumption::RefreshMutationPolicyUnknown
                ) {
                    issues.push(data_quality::Issue::new(
                        data_quality::Kind::AssumptionInForce {
                            assumption: *assumption,
                        },
                        data_quality::Severity::Warning,
                        self.provenance.clone(),
                        detected_at.clone(),
                        false,
                    ));
                }
            }
            issues
        }

        fn push_missing_issue(
            &self,
            issues: &mut Vec<data_quality::Issue>,
            missing: bool,
            field: data_quality::FieldPath,
            detected_at: source::Timestamp,
        ) {
            if missing {
                issues.push(data_quality::Issue::new(
                    data_quality::Kind::MissingRequiredField { field },
                    data_quality::Severity::Blocking,
                    self.provenance.clone(),
                    detected_at,
                    true,
                ));
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct SnapshotBuilder {
        provenance: Option<source::Provenance>,
        customer_record_id: Option<source::record::Id>,
        pet_record_id: Option<source::record::Id>,
        location_record_id: Option<source::record::Id>,
        service_type_record_id: Option<source::record::Id>,
        status: Option<Status>,
        relationship: Option<OwnerPetRelationship>,
        assumptions: Vec<Assumption>,
    }

    impl Default for SnapshotBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    impl SnapshotBuilder {
        pub const fn new() -> Self {
            Self {
                provenance: None,
                customer_record_id: None,
                pet_record_id: None,
                location_record_id: None,
                service_type_record_id: None,
                status: None,
                relationship: None,
                assumptions: Vec::new(),
            }
        }

        pub fn provenance(mut self, provenance: source::Provenance) -> Self {
            self.provenance = Some(provenance);
            self
        }

        pub fn customer_record_id(mut self, id: impl Into<Option<source::record::Id>>) -> Self {
            self.customer_record_id = id.into();
            self
        }

        pub fn pet_record_id(mut self, id: impl Into<Option<source::record::Id>>) -> Self {
            self.pet_record_id = id.into();
            self
        }

        pub fn location_record_id(mut self, id: impl Into<Option<source::record::Id>>) -> Self {
            self.location_record_id = id.into();
            self
        }

        pub fn service_type_record_id(mut self, id: impl Into<Option<source::record::Id>>) -> Self {
            self.service_type_record_id = id.into();
            self
        }

        pub fn status(mut self, status: impl Into<Option<Status>>) -> Self {
            self.status = status.into();
            self
        }

        pub fn relationship(mut self, relationship: OwnerPetRelationship) -> Self {
            self.relationship = Some(relationship);
            self
        }

        pub fn assumptions(mut self, assumptions: Vec<Assumption>) -> Self {
            self.assumptions = assumptions;
            self
        }

        pub fn build(self) -> Snapshot {
            Snapshot {
                provenance: self.provenance.expect("snapshot provenance is required"),
                customer_record_id: self.customer_record_id,
                pet_record_id: self.pet_record_id,
                location_record_id: self.location_record_id,
                service_type_record_id: self.service_type_record_id,
                status: self.status,
                relationship: self
                    .relationship
                    .expect("snapshot relationship is required"),
                assumptions: self.assumptions,
            }
        }
    }
}

pub mod gingr {
    use bon::Builder;
    use serde::{Deserialize, Serialize};

    use crate::source;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Endpoint(String);

    impl Endpoint {
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyGingrEndpoint).map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<Endpoint> for source::Endpoint {
        fn from(value: Endpoint) -> Self {
            source::Endpoint::try_new(value.0).expect("Gingr endpoint was already validated")
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct ProviderRecordId(String);

    impl ProviderRecordId {
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyProviderRecordId).map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<ProviderRecordId> for source::record::Id {
        fn from(value: ProviderRecordId) -> Self {
            source::record::Id::try_new(value.0).expect("Gingr provider id was already validated")
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RelatedProviderId {
        Owner(ProviderRecordId),
        Animal(ProviderRecordId),
        Location(ProviderRecordId),
        ReservationType(ProviderRecordId),
        Invoice(ProviderRecordId),
        Payment(ProviderRecordId),
        Service(ProviderRecordId),
    }

    impl RelatedProviderId {
        pub const fn owner(id: ProviderRecordId) -> Self {
            Self::Owner(id)
        }

        pub const fn animal(id: ProviderRecordId) -> Self {
            Self::Animal(id)
        }

        fn promote(self) -> source::record::RelatedId {
            match self {
                Self::Owner(id) => {
                    source::record::RelatedId::new(source::record::Role::Customer, id.into())
                }
                Self::Animal(id) => {
                    source::record::RelatedId::new(source::record::Role::Pet, id.into())
                }
                Self::Location(id) => {
                    source::record::RelatedId::new(source::record::Role::Location, id.into())
                }
                Self::ReservationType(id) => {
                    source::record::RelatedId::new(source::record::Role::ReservationType, id.into())
                }
                Self::Invoice(id) => {
                    source::record::RelatedId::new(source::record::Role::Invoice, id.into())
                }
                Self::Payment(id) => {
                    source::record::RelatedId::new(source::record::Role::Payment, id.into())
                }
                Self::Service(id) => {
                    source::record::RelatedId::new(source::record::Role::Service, id.into())
                }
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct ExtractionBatchId(String);

    impl ExtractionBatchId {
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyExtractionBatch).map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<ExtractionBatchId> for source::ExtractionBatchId {
        fn from(value: ExtractionBatchId) -> Self {
            source::ExtractionBatchId::try_new(value.0)
                .expect("Gingr extraction batch id was already validated")
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct RequestScope(String);

    impl RequestScope {
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyRequestScope).map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<RequestScope> for source::RequestScope {
        fn from(value: RequestScope) -> Self {
            source::RequestScope::try_new(value.0)
                .expect("Gingr request scope was already validated")
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct ProviderSchemaVersion(String);

    impl ProviderSchemaVersion {
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyProviderSchemaVersion).map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<ProviderSchemaVersion> for source::SchemaVersion {
        fn from(value: ProviderSchemaVersion) -> Self {
            source::SchemaVersion::try_new(value.0)
                .expect("Gingr provider schema version was already validated")
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct ProviderStatus(String);

    impl ProviderStatus {
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyProviderStatus).map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }

        fn promote(self) -> source::reservation::Status {
            match self.0.trim().to_ascii_lowercase().as_str() {
                "requested" | "request" | "pending" => source::reservation::Status::Requested,
                "confirmed" | "booked" => source::reservation::Status::Confirmed,
                "checked_in" | "checked-in" | "in_house" => source::reservation::Status::CheckedIn,
                "checked_out" | "checked-out" | "complete" => {
                    source::reservation::Status::CheckedOut
                }
                "cancelled" | "canceled" => source::reservation::Status::Cancelled,
                _ => source::reservation::Status::Unknown {
                    observed: source::ObservedStatus::try_new(self.0)
                        .expect("Gingr provider status was already validated"),
                },
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    pub struct Provenance {
        endpoint: Endpoint,
        provider_record_id: ProviderRecordId,
        #[builder(default)]
        related_provider_ids: Vec<RelatedProviderId>,
        extraction_batch: ExtractionBatchId,
        pulled_at: source::Timestamp,
        request_scope: RequestScope,
        provider_schema_version: ProviderSchemaVersion,
        source_payload_hash: source::PayloadHash,
        raw_payload_ref: source::RawPayloadRef,
    }

    impl Provenance {
        pub const fn source_system(&self) -> source::System {
            source::System::Gingr
        }

        pub const fn endpoint(&self) -> &Endpoint {
            &self.endpoint
        }

        pub const fn provider_record_id(&self) -> &ProviderRecordId {
            &self.provider_record_id
        }

        pub fn related_provider_ids(&self) -> &[RelatedProviderId] {
            &self.related_provider_ids
        }

        pub const fn extraction_batch(&self) -> &ExtractionBatchId {
            &self.extraction_batch
        }

        pub const fn pulled_at(&self) -> &source::Timestamp {
            &self.pulled_at
        }

        pub const fn raw_payload_ref(&self) -> &source::RawPayloadRef {
            &self.raw_payload_ref
        }

        pub fn promote(self) -> source::Provenance {
            source::Provenance::builder()
                .system(source::System::Gingr)
                .endpoint(self.endpoint.into())
                .record_id(self.provider_record_id.into())
                .related_record_ids(
                    self.related_provider_ids
                        .into_iter()
                        .map(RelatedProviderId::promote)
                        .collect(),
                )
                .extraction_batch(self.extraction_batch.into())
                .pulled_at(self.pulled_at)
                .request_scope(self.request_scope.into())
                .schema_version(self.provider_schema_version.into())
                .payload_hash(self.source_payload_hash)
                .raw_payload_ref(self.raw_payload_ref)
                .build()
        }
    }

    pub mod reservation {
        use serde::{Deserialize, Serialize};

        use super::{Provenance, ProviderRecordId, ProviderStatus};
        use crate::source;

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum OwnerPetRelationship {
            Resolved,
            Ambiguous { candidate_count: u16 },
        }

        impl From<OwnerPetRelationship> for source::reservation::OwnerPetRelationship {
            fn from(value: OwnerPetRelationship) -> Self {
                match value {
                    OwnerPetRelationship::Resolved => Self::Resolved,
                    OwnerPetRelationship::Ambiguous { candidate_count } => {
                        Self::Ambiguous { candidate_count }
                    }
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Snapshot {
            provenance: Provenance,
            owner_provider_id: Option<ProviderRecordId>,
            animal_provider_id: Option<ProviderRecordId>,
            location_provider_id: Option<ProviderRecordId>,
            service_type_provider_id: Option<ProviderRecordId>,
            provider_status: Option<ProviderStatus>,
            relationship: OwnerPetRelationship,
        }

        impl Snapshot {
            pub const fn builder() -> SnapshotBuilder {
                SnapshotBuilder::new()
            }

            pub const fn provenance(&self) -> &Provenance {
                &self.provenance
            }

            pub const fn owner_provider_id(&self) -> Option<&ProviderRecordId> {
                self.owner_provider_id.as_ref()
            }

            pub const fn animal_provider_id(&self) -> Option<&ProviderRecordId> {
                self.animal_provider_id.as_ref()
            }

            pub const fn location_provider_id(&self) -> Option<&ProviderRecordId> {
                self.location_provider_id.as_ref()
            }

            pub const fn service_type_provider_id(&self) -> Option<&ProviderRecordId> {
                self.service_type_provider_id.as_ref()
            }

            pub const fn provider_status(&self) -> Option<&ProviderStatus> {
                self.provider_status.as_ref()
            }

            pub const fn relationship(&self) -> &OwnerPetRelationship {
                &self.relationship
            }

            pub fn promote(self) -> source::reservation::Snapshot {
                let status = self.provider_status.map(ProviderStatus::promote);
                let mut assumptions = vec![
                    source::reservation::Assumption::GrainTreatedAsReservation,
                    source::reservation::Assumption::CustomerRecordIdTreatedAsStableJoinKey,
                    source::reservation::Assumption::PetRecordIdTreatedAsStableJoinKey,
                    source::reservation::Assumption::ProviderStatusMappingIsProvisional,
                ];
                if status.is_none() {
                    assumptions.push(source::reservation::Assumption::RefreshMutationPolicyUnknown);
                }

                source::reservation::Snapshot::builder()
                    .provenance(self.provenance.promote())
                    .customer_record_id(self.owner_provider_id.map(Into::into))
                    .pet_record_id(self.animal_provider_id.map(Into::into))
                    .location_record_id(self.location_provider_id.map(Into::into))
                    .service_type_record_id(self.service_type_provider_id.map(Into::into))
                    .status(status)
                    .relationship(self.relationship.into())
                    .assumptions(assumptions)
                    .build()
            }
        }

        #[derive(Debug, Clone)]
        pub struct SnapshotBuilder {
            provenance: Option<Provenance>,
            owner_provider_id: Option<ProviderRecordId>,
            animal_provider_id: Option<ProviderRecordId>,
            location_provider_id: Option<ProviderRecordId>,
            service_type_provider_id: Option<ProviderRecordId>,
            provider_status: Option<ProviderStatus>,
            relationship: Option<OwnerPetRelationship>,
        }

        impl Default for SnapshotBuilder {
            fn default() -> Self {
                Self::new()
            }
        }

        impl SnapshotBuilder {
            pub const fn new() -> Self {
                Self {
                    provenance: None,
                    owner_provider_id: None,
                    animal_provider_id: None,
                    location_provider_id: None,
                    service_type_provider_id: None,
                    provider_status: None,
                    relationship: None,
                }
            }

            pub fn provenance(mut self, provenance: Provenance) -> Self {
                self.provenance = Some(provenance);
                self
            }

            pub fn owner_provider_id(mut self, id: impl Into<Option<ProviderRecordId>>) -> Self {
                self.owner_provider_id = id.into();
                self
            }

            pub fn animal_provider_id(mut self, id: impl Into<Option<ProviderRecordId>>) -> Self {
                self.animal_provider_id = id.into();
                self
            }

            pub fn location_provider_id(mut self, id: impl Into<Option<ProviderRecordId>>) -> Self {
                self.location_provider_id = id.into();
                self
            }

            pub fn service_type_provider_id(
                mut self,
                id: impl Into<Option<ProviderRecordId>>,
            ) -> Self {
                self.service_type_provider_id = id.into();
                self
            }

            pub fn provider_status(mut self, status: impl Into<Option<ProviderStatus>>) -> Self {
                self.provider_status = status.into();
                self
            }

            pub fn relationship(mut self, relationship: OwnerPetRelationship) -> Self {
                self.relationship = Some(relationship);
                self
            }

            pub fn build(self) -> Snapshot {
                Snapshot {
                    provenance: self.provenance.expect("snapshot provenance is required"),
                    owner_provider_id: self.owner_provider_id,
                    animal_provider_id: self.animal_provider_id,
                    location_provider_id: self.location_provider_id,
                    service_type_provider_id: self.service_type_provider_id,
                    provider_status: self.provider_status,
                    relationship: self
                        .relationship
                        .expect("snapshot relationship is required"),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("timestamp must not be empty")]
    EmptyTimestamp,
    #[error("timestamp must be RFC3339 UTC-compatible text")]
    InvalidTimestamp,
    #[error("source endpoint must not be empty")]
    EmptyEndpoint,
    #[error("Gingr endpoint must not be empty")]
    EmptyGingrEndpoint,
    #[error("source record id must not be empty")]
    EmptyRecordId,
    #[error("provider record id must not be empty")]
    EmptyProviderRecordId,
    #[error("extraction batch id must not be empty")]
    EmptyExtractionBatch,
    #[error("request scope must not be empty")]
    EmptyRequestScope,
    #[error("schema version must not be empty")]
    EmptySchemaVersion,
    #[error("provider schema version must not be empty")]
    EmptyProviderSchemaVersion,
    #[error("source payload hash must not be empty")]
    EmptyPayloadHash,
    #[error("raw payload reference must not be empty")]
    EmptyRawPayloadRef,
    #[error("observed status must not be empty")]
    EmptyObservedStatus,
    #[error("provider status must not be empty")]
    EmptyProviderStatus,
}

pub type Result<T> = std::result::Result<T, Error>;

fn trimmed_non_empty(value: impl Into<String>, empty_error: Error) -> Result<String> {
    let value = value.into().trim().to_string();
    if value.is_empty() {
        return Err(empty_error);
    }
    Ok(value)
}
