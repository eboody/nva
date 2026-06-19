//! Grooming storage projection codes and validated cadence quantities.
//!
//! Grooming records can persist service codes and known repeat cadence in weeks
//! for rebooking workflows. Unknown or groomer-recommended cadence remains a
//! domain decision rather than a fabricated storage value.

use serde::{Deserialize, Deserializer, Serialize};

use domain::grooming::rebooking;

use crate::operations::{self, StorageField};

/// Storage shape for a migrated grooming service rules.
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_more::From, derive_more::Into,
)]
#[serde(transparent)]
pub struct ContractRecord(pub domain::grooming::Contract);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Storage-facing grooming service code used by service-offering records.
pub enum ServiceCode {
    /// Stable storage code for mini groom.
    MiniGroom,
    /// Stable storage code for full groom.
    FullGroom,
    /// Stable storage code for exit bath.
    ExitBath,
    /// Stable storage code for full bath.
    FullBath,
    /// Stable storage code for premium bath.
    PremiumBath,
    /// Stable storage code for nail trim.
    NailTrim,
    /// Stable storage code for nail dremel.
    NailDremel,
    /// Stable storage code for ear cleaning.
    EarCleaning,
    /// Stable storage code for coat skin specific product.
    CoatSkinSpecificProduct,
    /// Stable storage code for first time grooming offer.
    FirstTimeGroomingOffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Positive grooming cadence interval persisted in weeks.
pub struct StoredCadenceWeeks(u8);

impl StoredCadenceWeeks {
    /// Validates and wraps a positive quantity before it is persisted.
    pub const fn try_new(value: u8) -> std::result::Result<Self, StoredCadenceWeeksError> {
        if value == 0 {
            return Err(StoredCadenceWeeksError::ZeroWeeks);
        }
        Ok(Self(value))
    }

    /// Returns the provider numeric identifier kept on this wrapper.
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl<'de> Deserialize<'de> for StoredCadenceWeeks {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures for persisted grooming cadence intervals.
pub enum StoredCadenceWeeksError {
    #[error("stored grooming cadence requires at least one week")]
    /// Stable storage code for zero weeks.
    ZeroWeeks,
}

impl TryFrom<rebooking::CadenceWeeks> for StoredCadenceWeeks {
    type Error = operations::Error;

    fn try_from(value: rebooking::CadenceWeeks) -> operations::Result<Self> {
        Self::try_new(value.get()).map_err(|err| operations::Error::InvalidDomainValue {
            field: StorageField::GroomingCadenceWeeks,
            reason: err.to_string(),
        })
    }
}

impl TryFrom<StoredCadenceWeeks> for rebooking::CadenceWeeks {
    type Error = operations::Error;

    fn try_from(value: StoredCadenceWeeks) -> operations::Result<Self> {
        rebooking::CadenceWeeks::try_new(value.get()).map_err(|err| {
            operations::Error::InvalidDomainValue {
                field: StorageField::GroomingCadenceWeeks,
                reason: err.to_string(),
            }
        })
    }
}

impl From<ServiceCode> for domain::grooming::Service {
    fn from(value: ServiceCode) -> Self {
        match value {
            ServiceCode::MiniGroom => Self::MiniGroom,
            ServiceCode::FullGroom => Self::FullGroom,
            ServiceCode::ExitBath => Self::ExitBath,
            ServiceCode::FullBath => Self::FullBath,
            ServiceCode::PremiumBath => Self::PremiumBath,
            ServiceCode::NailTrim => Self::NailTrim,
            ServiceCode::NailDremel => Self::NailDremel,
            ServiceCode::EarCleaning => Self::EarCleaning,
            ServiceCode::CoatSkinSpecificProduct => Self::CoatSkinSpecificProduct,
            ServiceCode::FirstTimeGroomingOffer => Self::FirstTimeGroomingOffer,
        }
    }
}

impl From<domain::grooming::Service> for ServiceCode {
    fn from(value: domain::grooming::Service) -> Self {
        match value {
            domain::grooming::Service::MiniGroom => Self::MiniGroom,
            domain::grooming::Service::FullGroom => Self::FullGroom,
            domain::grooming::Service::ExitBath => Self::ExitBath,
            domain::grooming::Service::FullBath => Self::FullBath,
            domain::grooming::Service::PremiumBath => Self::PremiumBath,
            domain::grooming::Service::NailTrim => Self::NailTrim,
            domain::grooming::Service::NailDremel => Self::NailDremel,
            domain::grooming::Service::EarCleaning => Self::EarCleaning,
            domain::grooming::Service::CoatSkinSpecificProduct => Self::CoatSkinSpecificProduct,
            domain::grooming::Service::FirstTimeGroomingOffer => Self::FirstTimeGroomingOffer,
        }
    }
}
