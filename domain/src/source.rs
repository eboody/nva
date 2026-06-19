//! Source-system provenance and record references for app-owned operational facts.
//!
//! Provenance travels with facts so an agent draft can cite the app-owned source evidence it used:
//!
//! Crosswalk navigation: provenance is the source-entry receipt used by entity
//! pages, workflow packets, storage records, and runtime shells. See
//! `docs/entity-atlas/contract-crosswalk/source-provider-flows.md` for entry and
//! normalization, `workflow-packets.md` for workflow use,
//! `storage-persistence.md` for stored source refs, and `runtime-exposure.md`
//! for API/script exposure.
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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[strum(serialize_all = "snake_case")]
/// Upstream systems that can supply operational, POS, labor, or import data.
pub enum System {
    /// Gingr reservation and pet-care operating system.
    Gingr,
    /// Reporting or BI data source.
    BusinessIntelligence,
    /// Labor scheduling source for staffing plans.
    LaborScheduling,
    /// Timeclock source for worked-hour data.
    Timeclock,
    /// Payroll source for labor-cost reconciliation.
    Payroll,
    /// Capacity inventory source for available accommodation counts.
    CapacityInventory,
    /// Point-of-sale source for retail and payment activity.
    PointOfSale,
    /// Manually supplied import data.
    ManualImport,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// UTC instant reported by an upstream system for source-data lineage.
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Validates an upstream UTC timestamp before it can anchor source-data freshness.
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

    /// UTC extraction instant exposed for freshness checks, replay windows, and audit trails.
    pub const fn get(&self) -> &DateTime<Utc> {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Provider API endpoint or import route that produced source data.
pub struct Endpoint(String);

impl Endpoint {
    /// Validates the provider endpoint or import route before it can label source evidence.
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyEndpoint).map(Self)
    }

    /// Endpoint or import-route text retained for adapter calls and provenance displays.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Identifier that groups records from the same provider extraction run.
pub struct ExtractionBatchId(String);

impl ExtractionBatchId {
    /// Validates the extraction-batch id that ties provider records to the same pull.
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyExtractionBatch).map(Self)
    }

    /// Extraction-batch id exposed for replay, freshness, and audit comparison.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Import or API scope requested from the provider during extraction.
pub struct RequestScope(String);

impl RequestScope {
    /// Validates the request scope that explains why a provider payload was imported.
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyRequestScope).map(Self)
    }

    /// Request-scope text retained for source review and adapter diagnostics.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Version tag for the source payload schema used during mapping.
pub struct SchemaVersion(String);

impl SchemaVersion {
    /// Validates the schema-version label used to choose and review source mappers.
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptySchemaVersion).map(Self)
    }

    /// Schema-version label exposed for mapper selection and drift review.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Hash of the provider payload used for idempotency and drift checks.
pub struct PayloadHash(String);

impl PayloadHash {
    /// Validates the payload hash used to detect replay, duplicates, and source drift.
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyPayloadHash).map(Self)
    }

    /// Payload hash exposed for idempotency, drift detection, and audit comparison.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Storage reference for the unnormalized provider payload.
pub struct RawPayloadRef(String);

impl RawPayloadRef {
    /// Validates the storage reference that lets reviewers inspect the raw payload.
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyRawPayloadRef).map(Self)
    }

    /// Raw-payload location exposed for reviewer lookup and source audit trails.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Status text observed directly from the provider before normalization.
pub struct ObservedStatus(String);

impl ObservedStatus {
    /// Validates provider status text before an unknown mapping is retained for review.
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyObservedStatus).map(Self)
    }

    /// Provider status text exposed so reviewers can map or reject the unknown state.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Source-record identity and relationship vocabulary used for provenance joins.
pub mod record {
    use nutype::nutype;
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 120),
        derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            AsRef,
            Serialize,
            Deserialize
        )
    )]
    /// Provider or source identifier retained as the stable join key.
    pub struct Id(String);

    impl Id {
        /// Provider/read-model record id exposed for reconciliation joins.
        pub fn as_str(&self) -> &str {
            self.as_ref()
        }
    }

    #[derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        Serialize,
        Deserialize,
        strum::Display,
        strum::EnumString,
        strum::VariantArray,
    )]
    #[strum(serialize_all = "snake_case")]
    /// Kinds of related records that may be attached to source-data lineage.
    pub enum Role {
        /// Customer record participating in the workflow.
        Customer,
        /// Pet record participating in the workflow.
        Pet,
        /// Resort location record participating in the workflow.
        Location,
        /// Gingr reservation-type identifier used for service reconciliation.
        ReservationType,
        /// Gingr invoice identifier tied to reservation/payment reconciliation.
        Invoice,
        /// Gingr payment identifier tied to deposit or checkout reconciliation.
        Payment,
        /// Gingr service identifier used when mapping provider service types.
        Service,
        /// Staff provider id retained for labor-source reconciliation.
        Staff,
        /// Related record role is unknown, so reconciliation should not assume the link's business meaning.
        Unknown,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Link from a source record to another related provider record.
    pub struct RelatedId {
        role: Role,
        id: Id,
    }

    impl RelatedId {
        /// Assembles source-lineage data from already validated domain parts without reinterpreting authority.
        pub const fn new(role: Role, id: Id) -> Self {
            Self { role, id }
        }

        /// Related-record role explaining how this source id participates in reconciliation.
        pub const fn role(&self) -> Role {
            self.role
        }

        /// Provider/read-model identifier retained for reconciliation.
        pub const fn id(&self) -> &Id {
            &self.id
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Stable pointer to an upstream record and the system that owns it.
pub struct RecordRef {
    system: System,
    record_id: record::Id,
}

impl RecordRef {
    /// Assembles source-lineage data from already validated domain parts without reinterpreting authority.
    pub const fn new(system: System, record_id: record::Id) -> Self {
        Self { system, record_id }
    }

    /// Builds this source value from provenance data.
    pub fn from_provenance(provenance: &Provenance) -> Self {
        Self::new(provenance.system(), provenance.record_id().clone())
    }

    /// Source system that owns the referenced record.
    pub const fn system(&self) -> System {
        self.system
    }

    /// Provider/read-model identifier retained for reconciliation.
    pub const fn record_id(&self) -> &record::Id {
        &self.record_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Lineage metadata that ties normalized data back to its provider record.
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
    /// Upstream system that supplied this source evidence.
    pub const fn system(&self) -> System {
        self.system
    }

    /// Upstream system label preserved for source evidence and adapter routing.
    pub const fn source_system(&self) -> System {
        self.system
    }

    /// Provider endpoint or import route that produced this payload.
    pub const fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// Primary provider/read-model record id for this source fact.
    pub const fn record_id(&self) -> &record::Id {
        &self.record_id
    }

    /// Related source records that explain joins behind this source fact.
    pub fn related_record_ids(&self) -> &[record::RelatedId] {
        &self.related_record_ids
    }

    /// Extraction batch that groups records from the same provider pull.
    pub const fn extraction_batch(&self) -> &ExtractionBatchId {
        &self.extraction_batch
    }

    /// UTC extraction timestamp used to reason about freshness and replay.
    pub const fn pulled_at(&self) -> &Timestamp {
        &self.pulled_at
    }

    /// Provider request scope that explains why this record was imported.
    pub const fn request_scope(&self) -> &RequestScope {
        &self.request_scope
    }

    /// Provider schema version used by mappers and drift review.
    pub const fn schema_version(&self) -> &SchemaVersion {
        &self.schema_version
    }

    /// Payload hash used for idempotency, drift detection, and audit comparison.
    pub const fn payload_hash(&self) -> &PayloadHash {
        &self.payload_hash
    }

    /// Raw payload storage reference kept as reviewer-facing source evidence.
    pub const fn raw_payload_ref(&self) -> &RawPayloadRef {
        &self.raw_payload_ref
    }
}

/// Reservation source snapshots and assumptions retained for booking/review workflows.
pub mod reservation {
    use serde::{Deserialize, Serialize};

    use crate::{data_quality, source};

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

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        /// Starts a Gingr reservation source snapshot builder.
        pub const fn builder() -> SnapshotBuilder {
            SnapshotBuilder::new()
        }

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

        /// Data-quality blockers and warnings derived from missing source evidence or ambiguous promotion state.
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
    /// Builder for assembling a source snapshot with validated provider identifiers.
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
        /// Assembles source-lineage data from already validated domain parts without reinterpreting authority.
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

        /// Sets source-system evidence for the reservation snapshot's audit trail.
        pub fn provenance(mut self, provenance: source::Provenance) -> Self {
            self.provenance = Some(provenance);
            self
        }

        /// Attaches the optional customer provider id retained for owner reconciliation.
        pub fn customer_record_id(mut self, id: impl Into<Option<source::record::Id>>) -> Self {
            self.customer_record_id = id.into();
            self
        }

        /// Attaches the optional pet provider id retained for animal reconciliation.
        pub fn pet_record_id(mut self, id: impl Into<Option<source::record::Id>>) -> Self {
            self.pet_record_id = id.into();
            self
        }

        /// Attaches the optional location provider id used for resort-level reconciliation.
        pub fn location_record_id(mut self, id: impl Into<Option<source::record::Id>>) -> Self {
            self.location_record_id = id.into();
            self
        }

        /// Attaches the optional service-type provider id before domain-service promotion.
        pub fn service_type_record_id(mut self, id: impl Into<Option<source::record::Id>>) -> Self {
            self.service_type_record_id = id.into();
            self
        }

        /// Records normalized reservation status evidence for promotion or review routing.
        pub fn status(mut self, status: impl Into<Option<Status>>) -> Self {
            self.status = status.into();
            self
        }

        /// Records whether owner/pet linkage is resolved or needs reviewer reconciliation.
        pub fn relationship(mut self, relationship: OwnerPetRelationship) -> Self {
            self.relationship = Some(relationship);
            self
        }

        /// Records normalization assumptions reviewers may need to accept or reject.
        pub fn assumptions(mut self, assumptions: Vec<Assumption>) -> Self {
            self.assumptions = assumptions;
            self
        }

        /// Builds the source snapshot only after provenance and relationship evidence are present.
        ///
        /// Returning typed domain errors instead of panicking keeps imported reservation evidence
        /// out of automation and labor-review queues until reviewers can see exactly which source
        /// invariant is missing.
        pub fn build(self) -> source::Result<Snapshot> {
            Ok(Snapshot {
                provenance: self
                    .provenance
                    .ok_or(source::Error::ReservationSnapshotProvenanceRequired)?,
                customer_record_id: self.customer_record_id,
                pet_record_id: self.pet_record_id,
                location_record_id: self.location_record_id,
                service_type_record_id: self.service_type_record_id,
                status: self.status,
                relationship: self
                    .relationship
                    .ok_or(source::Error::ReservationSnapshotRelationshipRequired)?,
                assumptions: self.assumptions,
            })
        }
    }
}

/// Gingr provider mapping vocabulary kept separate from app-owned policy decisions.
pub mod gingr {
    use bon::Builder;
    use serde::{Deserialize, Serialize};

    use crate::source;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Provider API endpoint or import route that produced source data.
    pub struct Endpoint(String);

    impl Endpoint {
        /// Validates the Gingr endpoint before it can anchor provider-source evidence.
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyGingrEndpoint).map(Self)
        }

        /// Gingr endpoint text exposed for adapter calls and source-evidence review.
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
    /// Provider-native identifier for a source record.
    pub struct ProviderRecordId(String);

    impl ProviderRecordId {
        /// Validates the Gingr record id before it can be promoted into reconciliation evidence.
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyProviderRecordId).map(Self)
        }

        /// Gingr record id exposed for owner, pet, reservation, and payment reconciliation.
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
    /// Provider identifier relationship captured from source evidence for reconciliation and audit trails.
    pub enum RelatedProviderId {
        /// Gingr owner identifier related to the reservation snapshot.
        Owner(ProviderRecordId),
        /// Gingr animal identifier related to the reservation snapshot.
        Animal(ProviderRecordId),
        /// Resort location record participating in the workflow.
        Location(ProviderRecordId),
        /// Gingr reservation-type id retained for service-line reconciliation.
        ReservationType(ProviderRecordId),
        /// Gingr invoice id retained for payment and folio reconciliation.
        Invoice(ProviderRecordId),
        /// Gingr payment id retained for deposit and checkout reconciliation.
        Payment(ProviderRecordId),
        /// Gingr service id retained while mapping provider service types.
        Service(ProviderRecordId),
    }

    impl RelatedProviderId {
        /// Builds an owner related-provider id from Gingr source evidence.
        pub const fn owner(id: ProviderRecordId) -> Self {
            Self::Owner(id)
        }

        /// Builds an animal related-provider id from Gingr source evidence.
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
    /// Identifier that groups records from the same provider extraction run.
    pub struct ExtractionBatchId(String);

    impl ExtractionBatchId {
        /// Validates the Gingr extraction-batch id that groups records from one pull.
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyExtractionBatch).map(Self)
        }

        /// Gingr batch id exposed for replay, freshness, and adapter diagnostics.
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
    /// Import or API scope requested from the provider during extraction.
    pub struct RequestScope(String);

    impl RequestScope {
        /// Validates the Gingr request scope that explains the provider import boundary.
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyRequestScope).map(Self)
        }

        /// Gingr request scope exposed for source review and adapter diagnostics.
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
    /// Provider schema version observed for an imported payload.
    pub struct ProviderSchemaVersion(String);

    impl ProviderSchemaVersion {
        /// Validates the Gingr schema-version label before mapper promotion.
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyProviderSchemaVersion).map(Self)
        }

        /// Gingr schema-version label exposed for mapper selection and drift review.
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
    /// Provider-native status before mapping to a reservation workflow state.
    pub struct ProviderStatus(String);

    impl ProviderStatus {
        /// Validates Gingr status text before normalization into reservation workflow state.
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyProviderStatus).map(Self)
        }

        /// Gingr status text exposed for reviewer mapping when promotion is uncertain.
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
    /// Lineage metadata that ties normalized data back to its provider record.
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
        /// Source system for this Gingr provenance, always Gingr.
        pub const fn source_system(&self) -> source::System {
            source::System::Gingr
        }

        /// Gingr endpoint that produced the provider payload.
        pub const fn endpoint(&self) -> &Endpoint {
            &self.endpoint
        }

        /// Gingr provider-native record id retained before domain promotion.
        pub const fn provider_record_id(&self) -> &ProviderRecordId {
            &self.provider_record_id
        }

        /// Gingr provider ids retained for reconciliation and promotion decisions.
        pub fn related_provider_ids(&self) -> &[RelatedProviderId] {
            &self.related_provider_ids
        }

        /// Gingr extraction batch for freshness and replay review.
        pub const fn extraction_batch(&self) -> &ExtractionBatchId {
            &self.extraction_batch
        }

        /// UTC timestamp when the Gingr payload was pulled.
        pub const fn pulled_at(&self) -> &source::Timestamp {
            &self.pulled_at
        }

        /// Raw Gingr payload reference kept for reviewer/source-evidence lookup.
        pub const fn raw_payload_ref(&self) -> &source::RawPayloadRef {
            &self.raw_payload_ref
        }

        /// Promotes provider source data into the normalized domain snapshot.
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

    /// Reservation source snapshots and assumptions retained for booking/review workflows.
    pub mod reservation {
        use serde::{Deserialize, Serialize};

        use super::{Provenance, ProviderRecordId, ProviderStatus};
        use crate::source;

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Confidence in the provider relationship between an owner and pet.
        pub enum OwnerPetRelationship {
            /// Owner-pet relationship was matched to a single confident record.
            Resolved,
            /// Multiple Gingr owner/animal records could match, blocking confident promotion until reviewed.
            Ambiguous {
                /// Number of possible provider owner/pet matches reviewers must reconcile before promotion.
                candidate_count: u16,
            },
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
        /// Point-in-time source-data view used before promotion into core domain records.
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
            /// Starts a Gingr reservation source snapshot builder.
            pub const fn builder() -> SnapshotBuilder {
                SnapshotBuilder::new()
            }

            /// Gingr source-system evidence for this snapshot.
            pub const fn provenance(&self) -> &Provenance {
                &self.provenance
            }

            /// Gingr owner id retained for customer reconciliation.
            pub const fn owner_provider_id(&self) -> Option<&ProviderRecordId> {
                self.owner_provider_id.as_ref()
            }

            /// Gingr animal id retained for pet reconciliation.
            pub const fn animal_provider_id(&self) -> Option<&ProviderRecordId> {
                self.animal_provider_id.as_ref()
            }

            /// Gingr location id retained for location reconciliation.
            pub const fn location_provider_id(&self) -> Option<&ProviderRecordId> {
                self.location_provider_id.as_ref()
            }

            /// Gingr service-type id retained for service reconciliation.
            pub const fn service_type_provider_id(&self) -> Option<&ProviderRecordId> {
                self.service_type_provider_id.as_ref()
            }

            /// Gingr status text retained until it is mapped into normalized reservation status.
            pub const fn provider_status(&self) -> Option<&ProviderStatus> {
                self.provider_status.as_ref()
            }

            /// Gingr owner/pet relationship confidence used during promotion.
            pub const fn relationship(&self) -> &OwnerPetRelationship {
                &self.relationship
            }

            /// Promotes provider source data into the normalized domain snapshot.
            pub fn promote(self) -> source::Result<source::reservation::Snapshot> {
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
        /// Builder for assembling a source snapshot with validated provider identifiers.
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
            /// Assembles source-lineage data from already validated domain parts without reinterpreting authority.
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

            /// Sets Gingr source-system evidence for the reservation snapshot's audit trail.
            pub fn provenance(mut self, provenance: Provenance) -> Self {
                self.provenance = Some(provenance);
                self
            }

            /// Attaches the optional Gingr owner id retained for customer reconciliation.
            pub fn owner_provider_id(mut self, id: impl Into<Option<ProviderRecordId>>) -> Self {
                self.owner_provider_id = id.into();
                self
            }

            /// Attaches the optional Gingr animal id retained for pet reconciliation.
            pub fn animal_provider_id(mut self, id: impl Into<Option<ProviderRecordId>>) -> Self {
                self.animal_provider_id = id.into();
                self
            }

            /// Attaches the optional Gingr location id used for resort-level reconciliation.
            pub fn location_provider_id(mut self, id: impl Into<Option<ProviderRecordId>>) -> Self {
                self.location_provider_id = id.into();
                self
            }

            /// Attaches the optional Gingr service-type id before domain-service promotion.
            pub fn service_type_provider_id(
                mut self,
                id: impl Into<Option<ProviderRecordId>>,
            ) -> Self {
                self.service_type_provider_id = id.into();
                self
            }

            /// Records observed Gingr status evidence that drives promotion or exception review.
            pub fn provider_status(mut self, status: impl Into<Option<ProviderStatus>>) -> Self {
                self.provider_status = status.into();
                self
            }

            /// Records whether Gingr owner/animal linkage is resolved or reviewer-ambiguous.
            pub fn relationship(mut self, relationship: OwnerPetRelationship) -> Self {
                self.relationship = Some(relationship);
                self
            }

            /// Builds the Gingr source snapshot only after provenance and relationship evidence are present.
            ///
            /// Returning typed domain errors instead of panicking keeps provider-native reservation
            /// evidence out of automation and labor-review queues until reviewers can see exactly
            /// which Gingr source invariant is missing.
            pub fn build(self) -> source::Result<Snapshot> {
                Ok(Snapshot {
                    provenance: self
                        .provenance
                        .ok_or(source::Error::GingrReservationSnapshotProvenanceRequired)?,
                    owner_provider_id: self.owner_provider_id,
                    animal_provider_id: self.animal_provider_id,
                    location_provider_id: self.location_provider_id,
                    service_type_provider_id: self.service_type_provider_id,
                    provider_status: self.provider_status,
                    relationship: self
                        .relationship
                        .ok_or(source::Error::GingrReservationSnapshotRelationshipRequired)?,
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures returned by source domain constructors.
pub enum Error {
    #[error("timestamp must not be empty")]
    /// Signals that timestamp was blank or missing during source validation.
    EmptyTimestamp,
    #[error("timestamp must be RFC3339 UTC-compatible text")]
    /// Signals that timestamp could not be parsed or accepted during source validation.
    InvalidTimestamp,
    #[error("source endpoint must not be empty")]
    /// Signals that endpoint was blank or missing during source validation.
    EmptyEndpoint,
    #[error("Gingr endpoint must not be empty")]
    /// Signals that gingr endpoint was blank or missing during source validation.
    EmptyGingrEndpoint,
    #[error("source record id must not be empty")]
    /// Signals that record id was blank or missing during source validation.
    EmptyRecordId,
    #[error("provider record id must not be empty")]
    /// Signals that provider record id was blank or missing during source validation.
    EmptyProviderRecordId,
    #[error("extraction batch id must not be empty")]
    /// Signals that extraction batch was blank or missing during source validation.
    EmptyExtractionBatch,
    #[error("request scope must not be empty")]
    /// Signals that request scope was blank or missing during source validation.
    EmptyRequestScope,
    #[error("schema version must not be empty")]
    /// Signals that schema version was blank or missing during source validation.
    EmptySchemaVersion,
    #[error("provider schema version must not be empty")]
    /// Signals that provider schema version was blank or missing during source validation.
    EmptyProviderSchemaVersion,
    #[error("source payload hash must not be empty")]
    /// Signals that payload hash was blank or missing during source validation.
    EmptyPayloadHash,
    #[error("raw payload reference must not be empty")]
    /// Signals that raw payload ref was blank or missing during source validation.
    EmptyRawPayloadRef,
    #[error("observed status must not be empty")]
    /// Signals that observed status was blank or missing during source validation.
    EmptyObservedStatus,
    #[error("provider status must not be empty")]
    /// Signals that provider status was blank or missing during source validation.
    EmptyProviderStatus,
    #[error("reservation snapshot requires provenance before source evidence can enter review")]
    /// Signals that normalized reservation source snapshots were built without audit provenance.
    ReservationSnapshotProvenanceRequired,
    #[error("reservation snapshot requires owner/pet relationship evidence before review")]
    /// Signals that normalized reservation source snapshots were built without owner/pet relationship evidence.
    ReservationSnapshotRelationshipRequired,
    #[error(
        "Gingr reservation snapshot requires provenance before source evidence can enter review"
    )]
    /// Signals that Gingr-native reservation snapshots were built without provider provenance.
    GingrReservationSnapshotProvenanceRequired,
    #[error("Gingr reservation snapshot requires owner/animal relationship evidence before review")]
    /// Signals that Gingr-native reservation snapshots were built without owner/animal relationship evidence.
    GingrReservationSnapshotRelationshipRequired,
}

/// Result type returned by fallible source operations.
pub type Result<T> = std::result::Result<T, Error>;

fn trimmed_non_empty(value: impl Into<String>, empty_error: Error) -> Result<String> {
    let value = value.into().trim().to_string();
    if value.is_empty() {
        return Err(empty_error);
    }
    Ok(value)
}
