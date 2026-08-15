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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Stable discriminator preserving why a grooming cadence does or does not carry weeks.
pub enum StoredCadenceKind {
    /// A positive interval is stored separately in `grooming_cadence_weeks`.
    EveryWeeks,
    /// Rebooking is driven by need rather than a fixed interval.
    AsNeeded,
    /// A groomer must recommend the next interval.
    GroomerRecommended,
    /// Source evidence did not establish a cadence kind.
    Unknown,
}

impl StoredCadenceKind {
    /// Converts a domain cadence into its lossless stable kind and optional interval columns.
    pub fn from_domain(
        cadence: rebooking::Cadence,
    ) -> operations::Result<(Self, Option<StoredCadenceWeeks>)> {
        match cadence {
            rebooking::Cadence::EveryWeeks(weeks) => {
                Ok((Self::EveryWeeks, Some(weeks.try_into()?)))
            }
            rebooking::Cadence::AsNeeded => Ok((Self::AsNeeded, None)),
            rebooking::Cadence::GroomerRecommended => Ok((Self::GroomerRecommended, None)),
            rebooking::Cadence::Unknown => Ok((Self::Unknown, None)),
        }
    }

    /// Rehydrates a domain cadence only when the discriminator and interval columns agree.
    pub fn into_domain(
        self,
        weeks: Option<StoredCadenceWeeks>,
    ) -> operations::Result<rebooking::Cadence> {
        match (self, weeks) {
            (Self::EveryWeeks, Some(weeks)) => {
                Ok(rebooking::Cadence::EveryWeeks(weeks.try_into()?))
            }
            (Self::AsNeeded, None) => Ok(rebooking::Cadence::AsNeeded),
            (Self::GroomerRecommended, None) => Ok(rebooking::Cadence::GroomerRecommended),
            (Self::Unknown, None) => Ok(rebooking::Cadence::Unknown),
            _ => Err(operations::Error::StorageShapeMismatch {
                record: operations::RecordKind::ServiceOffering,
                reason: operations::ShapeMismatchReason::FieldBelongsToDifferentVariant,
            }),
        }
    }
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
