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
