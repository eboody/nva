//! Promotion helpers from quarantined Gingr records into source-agnostic candidates.
//!
//! Mapping code may read Gingr-shaped DTOs, but the values it returns are domain
//! source-backed candidates. Provider ids stay inside this adapter; the domain
//! does not learn Gingr vocabulary, and callers cannot detach a promoted value
//! from the exact source record and mapper version that established it.
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use std::collections::BTreeMap;
//!
//! use domain::source;
//! use gingr::{endpoint, mapping, response};
//!
//! let provider_record = response::OwnerRecord {
//!     id: endpoint::OwnerId::new(501),
//!     first_name: Some("Sam".to_owned()),
//!     last_name: Some("Rivera".to_owned()),
//!     email: Some(response::provider::Email::new("sam@example.test")),
//!     cell_phone: None,
//!     unknown: BTreeMap::new(),
//! };
//!
//! let provenance = source::Provenance::builder()
//!     .system(source::System::Gingr)
//!     .endpoint(source::Endpoint::try_new("opaque://fixture/owner")?)
//!     .record_id(source::record::Id::try_new(provider_record.id.to_string())?)
//!     .extraction_batch(source::ExtractionBatchId::try_new("fixture-batch")?)
//!     .pulled_at(source::Timestamp::try_new("2026-08-13T21:30:00Z")?)
//!     .request_scope(source::RequestScope::try_new("customer-contact")?)
//!     .schema_version(source::SchemaVersion::try_new("unverified-owner-fixture")?)
//!     .payload_hash(source::PayloadHash::try_new("sha256:fixture")?)
//!     .raw_payload_ref(source::RawPayloadRef::try_new("fixture://gingr/owner-501")?)
//!     .build();
//! let promoted = mapping::customer::contact_candidate(&provider_record, provenance)?;
//!
//! assert_eq!(promoted.record_ref().system(), source::System::Gingr);
//! assert_eq!(promoted.record_ref().record_id().as_str(), "501");
//! assert_eq!(promoted.mapping_version(), mapping::Version::CustomerContactV1);
//! assert_eq!(promoted.candidate().provider_owner_id, endpoint::OwnerId::new(501));
//! assert!(promoted.candidate().email.is_some());
//! # Ok(())
//! # }
//! ```

/// Customer mapper that turns a Gingr owner record into a reviewable domain contact candidate.
pub mod customer;
/// Pet mapper that turns a Gingr animal record into a reviewable domain pet-name candidate.
pub mod pet;
/// Retail mapper that turns a Gingr item DTO into a reviewable domain product candidate.
pub mod retail;

use domain::source;

/// Result type returned by fallible mapping operations.
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Stable identity of an NVA-local mapping implementation.
///
/// These variants do not assert provider endpoint or schema knowledge. The observed
/// endpoint and source-shape labels remain opaque in `source::Provenance` until an
/// authoritative provider contract is available.
pub enum Version {
    /// First local owner-shaped observation to customer-contact candidate mapper.
    CustomerContactV1,
    /// First local animal-shaped observation to pet-name candidate mapper.
    PetNameV1,
    /// First local retail-item-shaped observation to product candidate mapper.
    RetailProductV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Candidate atomically bound to the source evidence and mapping contract that produced it.
pub struct Promoted<T> {
    candidate: T,
    record_ref: source::RecordRef,
    provenance: source::Provenance,
    mapping_version: Version,
}

impl<T> Promoted<T> {
    fn from_gingr_record(
        candidate: T,
        provider_record_id: impl Into<String>,
        provenance: source::Provenance,
        mapping_version: Version,
    ) -> Result<Self> {
        if provenance.system() != source::System::Gingr {
            return Err(Error::UnexpectedSourceSystem {
                actual: provenance.system(),
            });
        }

        let provider_record_id =
            source::record::Id::try_new(provider_record_id.into()).map_err(|error| {
                Error::InvalidSourceRecordId {
                    reason: error.to_string(),
                }
            })?;
        if provenance.record_id() != &provider_record_id {
            return Err(Error::SourceRecordMismatch {
                provider_record_id,
                provenance_record_id: provenance.record_id().clone(),
            });
        }

        Ok(Self {
            candidate,
            record_ref: source::RecordRef::from_provenance(&provenance),
            provenance,
            mapping_version,
        })
    }

    /// Candidate values validated from the quarantined provider record.
    pub const fn candidate(&self) -> &T {
        &self.candidate
    }

    /// Stable source-record pointer bound to this candidate.
    pub const fn record_ref(&self) -> &source::RecordRef {
        &self.record_ref
    }

    /// Complete extraction evidence bound to this candidate.
    pub const fn provenance(&self) -> &source::Provenance {
        &self.provenance
    }

    /// NVA-local mapper used to derive the candidate without claiming a verified provider contract.
    pub const fn mapping_version(&self) -> Version {
        self.mapping_version
    }

    /// Decomposes the atomic value only when an owning app boundary is ready to store all evidence together.
    pub fn into_parts(self) -> (T, source::RecordRef, source::Provenance, Version) {
        (
            self.candidate,
            self.record_ref,
            self.provenance,
            self.mapping_version,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, strum::Display)]
/// Gingr fields required by DTO-to-domain mapping routines.
pub enum ProviderField {
    #[strum(to_string = "owner name")]
    /// Required Gingr owner-name field for customer mapping.
    OwnerName,
    #[strum(to_string = "animal name")]
    /// Required Gingr animal-name field for pet mapping.
    AnimalName,
    #[strum(to_string = "retail item name")]
    /// Required Gingr retail item name for product mapping.
    RetailItemName,
    #[strum(to_string = "retail item sku")]
    /// Required Gingr retail SKU for product matching.
    RetailItemSku,
    #[strum(to_string = "retail item category")]
    /// Required Gingr retail category for merchandising mapping.
    RetailItemCategory,
    #[strum(to_string = "retail item active flag")]
    /// Required Gingr retail active flag for product availability mapping.
    RetailItemActive,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
/// Errors raised when Gingr records are missing fields or fail domain validation during mapping.
pub enum Error {
    #[error("missing required Gingr provider field: {field}")]
    /// DTO mapping cannot proceed because Gingr omitted a required field.
    MissingRequiredProviderField {
        /// Provider field required before this mapping can create a source-backed candidate.
        field: ProviderField,
    },
    #[error("invalid domain value promoted from Gingr provider field {field}: {reason}")]
    /// Signals that a domain value cannot be represented safely in storage.
    InvalidDomainValue {
        /// Provider field whose promoted value failed NVA domain validation.
        field: ProviderField,
        /// Validation reason returned by the downstream domain type or mapper.
        reason: String,
    },
    #[error("Gingr promotion requires Gingr provenance, received {actual}")]
    /// Provenance names a source system other than Gingr at this provider boundary.
    UnexpectedSourceSystem {
        /// Source system supplied by the caller.
        actual: source::System,
    },
    #[error("invalid Gingr source record id: {reason}")]
    /// Provider identity could not be represented by the source-record identity contract.
    InvalidSourceRecordId {
        /// Validation reason from the source-record identity type.
        reason: String,
    },
    #[error("Gingr provider record does not match the supplied provenance record")]
    /// Candidate and provenance refer to different provider records and cannot be promoted atomically.
    SourceRecordMismatch {
        /// Identity carried by the quarantined provider DTO.
        provider_record_id: source::record::Id,
        /// Identity carried by the supplied provenance.
        provenance_record_id: source::record::Id,
    },
}
