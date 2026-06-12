use serde::{Deserialize, Deserializer, Serialize};

use crate::operations::{Error, Result, StorageField};

/// Storage shape for a migrated grooming service contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContractRecord(pub domain::service::grooming::Contract);

impl From<domain::service::grooming::Contract> for ContractRecord {
    fn from(value: domain::service::grooming::Contract) -> Self {
        Self(value)
    }
}

impl From<ContractRecord> for domain::service::grooming::Contract {
    fn from(record: ContractRecord) -> Self {
        record.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceCode {
    MiniGroom,
    FullGroom,
    ExitBath,
    FullBath,
    PremiumBath,
    NailTrim,
    NailDremel,
    EarCleaning,
    CoatSkinSpecificProduct,
    FirstTimeGroomingOffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct StoredCadenceWeeks(u8);

impl StoredCadenceWeeks {
    pub const fn try_new(value: u8) -> std::result::Result<Self, StoredCadenceWeeksError> {
        if value == 0 {
            return Err(StoredCadenceWeeksError::ZeroWeeks);
        }
        Ok(Self(value))
    }

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
pub enum StoredCadenceWeeksError {
    #[error("stored grooming cadence requires at least one week")]
    ZeroWeeks,
}

impl TryFrom<domain::service::grooming::CadenceWeeks> for StoredCadenceWeeks {
    type Error = Error;

    fn try_from(value: domain::service::grooming::CadenceWeeks) -> Result<Self> {
        Self::try_new(value.get()).map_err(|err| Error::InvalidDomainValue {
            field: StorageField::GroomingCadenceWeeks,
            reason: err.to_string(),
        })
    }
}

impl TryFrom<StoredCadenceWeeks> for domain::service::grooming::CadenceWeeks {
    type Error = Error;

    fn try_from(value: StoredCadenceWeeks) -> Result<Self> {
        domain::service::grooming::CadenceWeeks::try_new(value.get()).map_err(|err| {
            Error::InvalidDomainValue {
                field: StorageField::GroomingCadenceWeeks,
                reason: err.to_string(),
            }
        })
    }
}

impl From<ServiceCode> for domain::service::grooming::Service {
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

impl From<domain::service::grooming::Service> for ServiceCode {
    fn from(value: domain::service::grooming::Service) -> Self {
        match value {
            domain::service::grooming::Service::MiniGroom => Self::MiniGroom,
            domain::service::grooming::Service::FullGroom => Self::FullGroom,
            domain::service::grooming::Service::ExitBath => Self::ExitBath,
            domain::service::grooming::Service::FullBath => Self::FullBath,
            domain::service::grooming::Service::PremiumBath => Self::PremiumBath,
            domain::service::grooming::Service::NailTrim => Self::NailTrim,
            domain::service::grooming::Service::NailDremel => Self::NailDremel,
            domain::service::grooming::Service::EarCleaning => Self::EarCleaning,
            domain::service::grooming::Service::CoatSkinSpecificProduct => {
                Self::CoatSkinSpecificProduct
            }
            domain::service::grooming::Service::FirstTimeGroomingOffer => {
                Self::FirstTimeGroomingOffer
            }
        }
    }
}
