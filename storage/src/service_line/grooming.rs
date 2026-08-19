//! Grooming storage projection codes and validated cadence quantities.
//!
//! Grooming records can persist service codes and known repeat cadence in weeks
//! for rebooking workflows. Unknown or groomer-recommended cadence remains a
//! domain decision rather than a fabricated storage value.

use core::num::NonZeroU8;

use serde::{Deserialize, Deserializer, Serialize};

use domain::grooming::rebooking;

use crate::projection::{Error, Result};

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
pub struct StoredCadenceWeeks(NonZeroU8);

impl StoredCadenceWeeks {
    /// Validates and wraps a positive quantity before it is persisted.
    pub const fn try_new(value: u8) -> std::result::Result<Self, StoredCadenceWeeksError> {
        match NonZeroU8::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(StoredCadenceWeeksError::ZeroWeeks),
        }
    }

    const fn from_nonzero(value: NonZeroU8) -> Self {
        Self(value)
    }

    const fn into_nonzero(self) -> NonZeroU8 {
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
/// Canonical persisted grooming cadence; invalid kind/interval combinations are unrepresentable.
pub enum CadenceRecord {
    /// Rebook after a positive number of weeks.
    EveryWeeks(StoredCadenceWeeks),
    /// Rebooking is driven by need rather than a fixed interval.
    AsNeeded,
    /// A groomer must recommend the next interval.
    GroomerRecommended,
    /// Source evidence did not establish a cadence kind.
    Unknown,
}

impl CadenceRecord {
    /// Converts a domain cadence into its lossless persisted representation.
    pub fn from_domain(cadence: rebooking::Cadence) -> Result<Self> {
        match cadence {
            rebooking::Cadence::EveryWeeks(weeks) => Ok(Self::EveryWeeks(weeks.try_into()?)),
            rebooking::Cadence::AsNeeded => Ok(Self::AsNeeded),
            rebooking::Cadence::GroomerRecommended => Ok(Self::GroomerRecommended),
            rebooking::Cadence::Unknown => Ok(Self::Unknown),
        }
    }

    /// Rehydrates the canonical domain cadence.
    pub fn into_domain(self) -> Result<rebooking::Cadence> {
        match self {
            Self::EveryWeeks(weeks) => Ok(rebooking::Cadence::EveryWeeks(weeks.try_into()?)),
            Self::AsNeeded => Ok(rebooking::Cadence::AsNeeded),
            Self::GroomerRecommended => Ok(rebooking::Cadence::GroomerRecommended),
            Self::Unknown => Ok(rebooking::Cadence::Unknown),
        }
    }
}

impl TryFrom<rebooking::CadenceWeeks> for StoredCadenceWeeks {
    type Error = Error;

    fn try_from(value: rebooking::CadenceWeeks) -> Result<Self> {
        Ok(Self::from_nonzero(value.into_nonzero()))
    }
}

impl TryFrom<StoredCadenceWeeks> for rebooking::CadenceWeeks {
    type Error = Error;

    fn try_from(value: StoredCadenceWeeks) -> Result<Self> {
        Ok(rebooking::CadenceWeeks::from_nonzero(value.into_nonzero()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cadence_record_roundtrips_positive_weeks_without_optional_discriminator_drift() {
        let weeks = StoredCadenceWeeks::try_new(6).unwrap();
        assert_eq!(
            CadenceRecord::EveryWeeks(weeks).into_domain().unwrap(),
            rebooking::Cadence::EveryWeeks(weeks.try_into().unwrap())
        );

        let domain_weeks: rebooking::CadenceWeeks = weeks.try_into().unwrap();
        assert_eq!(domain_weeks.get(), 6);
        assert!(serde_json::from_str::<StoredCadenceWeeks>("0").is_err());
    }
}
