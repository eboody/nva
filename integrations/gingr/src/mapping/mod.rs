pub mod customer;
pub mod pet;
pub mod retail;

use std::fmt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderField {
    OwnerName,
    AnimalName,
    RetailItemName,
    RetailItemSku,
    RetailItemCategory,
}

impl fmt::Display for ProviderField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnerName => formatter.write_str("owner name"),
            Self::AnimalName => formatter.write_str("animal name"),
            Self::RetailItemName => formatter.write_str("retail item name"),
            Self::RetailItemSku => formatter.write_str("retail item sku"),
            Self::RetailItemCategory => formatter.write_str("retail item category"),
        }
    }
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
