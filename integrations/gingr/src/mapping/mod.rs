//! Promotion helpers from quarantined Gingr records into source-agnostic candidates.
//!
//! Mapping code may read Gingr-shaped DTOs, but the values it returns are domain
//! candidates plus caller-owned source refs. Provider ids stay inside this adapter;
//! the domain does not learn Gingr vocabulary.
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
//! let source_ref = source::RecordRef::new(
//!     source::System::Gingr,
//!     source::record::Id::try_new(provider_record.id.to_string())?,
//! );
//! let promoted = mapping::customer::contact_candidate(&provider_record)?;
//!
//! assert_eq!(source_ref.system(), source::System::Gingr);
//! assert_eq!(source_ref.record_id().as_str(), "501");
//! assert_eq!(promoted.provider_owner_id, endpoint::OwnerId::new(501));
//! assert!(promoted.email.is_some());
//! # Ok(())
//! # }
//! ```

/// Customer mapper that turns a Gingr owner record into a reviewable domain contact candidate.
pub mod customer;
/// Pet mapper that turns a Gingr animal record into a reviewable domain pet-name candidate.
pub mod pet;
/// Retail mapper that turns a Gingr item DTO into a reviewable domain product candidate.
pub mod retail;

/// Result type returned by fallible mapping operations.
pub type Result<T> = core::result::Result<T, Error>;

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
}
