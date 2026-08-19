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
//!     .system(source::System::ProviderOrPms)
//!     .endpoint(source::Endpoint::try_new("/reservations").unwrap())
//!     .record_id(source::record::Id::try_new("reservation-123").unwrap())
//!     .extraction_batch(source::ExtractionBatchId::try_new("batch-2026-06-18").unwrap())
//!     .pulled_at(source::Timestamp::try_new("2026-06-18T13:00:00Z").unwrap())
//!     .request_scope(source::RequestScope::try_new("manager-daily-brief:loc-1").unwrap())
//!     .schema_version(source::SchemaVersion::try_new("provider-reservations-v1").unwrap())
//!     .payload_hash(source::PayloadHash::try_new("sha256:fixture").unwrap())
//!     .raw_payload_ref(source::RawPayloadRef::try_new("minio://fixtures/reservation-123.json").unwrap())
//!     .build();
//!
//! let record_ref = source::RecordRef::from_provenance(&provenance);
//! assert_eq!(record_ref.system(), source::System::ProviderOrPms);
//! assert_eq!(record_ref.record_id().as_str(), "reservation-123");
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

macro_rules! deserialize_via_try_new {
    ($type:ty) => {
        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = <String as Deserialize>::deserialize(deserializer)?;
                <$type>::try_new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

macro_rules! redacted_debug {
    ($($type:ty => $name:literal),+ $(,)?) => {
        $(
            impl std::fmt::Debug for $type {
                fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str(concat!($name, "([REDACTED])"))
                }
            }
        )+
    };
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
/// Upstream systems that can supply operational, POS, labor, or import data.
pub enum System {
    /// Telephony or call-center system carrying missed calls, voicemails, and call outcomes.
    Telephony,
    /// SMS provider carrying inbound/outbound text evidence and opt-out events.
    SmsProvider,
    /// Email inbox or transactional-email provider.
    Email,
    /// Web chat or website-assistant transcript source.
    WebChat,
    /// Website lead/intake form source.
    WebsiteForms,
    /// Marketing automation or campaign platform.
    MarketingAutomation,
    /// CRM/customer profile source.
    Crm,
    /// Finance/accounting source for site revenue, discounts, refunds, and costs.
    FinanceAccounting,
    /// HRIS, scheduling, or timekeeping source for labor evidence.
    WorkforceManagement,
    /// Document/SOP/vendor knowledge base.
    KnowledgeBase,
    /// Existing pet-resort operating system or PMS evidence.
    ProviderOrPms,
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

redacted_debug!(
    Timestamp => "Timestamp",
    Endpoint => "Endpoint",
    ExtractionBatchId => "ExtractionBatchId",
    RequestScope => "RequestScope",
    SchemaVersion => "SchemaVersion",
    PayloadHash => "PayloadHash",
    RawPayloadRef => "RawPayloadRef",
    ObservedStatus => "ObservedStatus",
);

/// Source-record identity and relationship vocabulary used for provenance joins.
pub mod record;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

redacted_debug!(RecordRef => "RecordRef");

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
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

impl std::fmt::Debug for Provenance {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Provenance([REDACTED])")
    }
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
pub mod reservation;

redacted_debug!(
    reservation::Snapshot => "source::reservation::Snapshot",
);

deserialize_via_try_new!(Timestamp);
deserialize_via_try_new!(Endpoint);
deserialize_via_try_new!(ExtractionBatchId);
deserialize_via_try_new!(RequestScope);
deserialize_via_try_new!(SchemaVersion);
deserialize_via_try_new!(PayloadHash);
deserialize_via_try_new!(RawPayloadRef);
deserialize_via_try_new!(ObservedStatus);
mod error;
pub use error::{Error, Result};

fn trimmed_non_empty(value: impl Into<String>, empty_error: Error) -> Result<String> {
    let value = value.into().trim().to_string();
    if value.is_empty() {
        return Err(empty_error);
    }
    Ok(value)
}
