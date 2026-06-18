//! Promotion helpers from quarantined Gingr records into source-agnostic candidates.
//!
//! Mapping code may read Gingr-shaped DTOs, but the values it returns are domain
//! candidates plus caller-owned source refs. Provider ids stay at this boundary;
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

pub mod customer;
pub mod pet;
pub mod retail;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, strum::Display)]
pub enum ProviderField {
    #[strum(to_string = "owner name")]
    OwnerName,
    #[strum(to_string = "animal name")]
    AnimalName,
    #[strum(to_string = "retail item name")]
    RetailItemName,
    #[strum(to_string = "retail item sku")]
    RetailItemSku,
    #[strum(to_string = "retail item category")]
    RetailItemCategory,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("missing required Gingr provider field: {field}")]
    MissingRequiredProviderField { field: ProviderField },
    #[error("invalid domain value promoted from Gingr provider field {field}: {reason}")]
    InvalidDomainValue {
        field: ProviderField,
        reason: String,
    },
}
