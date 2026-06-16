use serde::{Deserialize, Serialize};

use domain::operations::lodging_offer;
/// Storage shape for a migrated boarding service contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContractRecord(pub domain::boarding::Contract);

impl From<domain::boarding::Contract> for ContractRecord {
    fn from(value: domain::boarding::Contract) -> Self {
        Self(value)
    }
}

impl From<ContractRecord> for domain::boarding::Contract {
    fn from(record: ContractRecord) -> Self {
        record.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccommodationCode {
    ClassicSuite,
    LuxurySuite,
    CatCondo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CareFeatureCode {
    DailyHousekeeping,
    PottyWalks,
    Bedding,
    PawgressReport,
    FeedingSupport,
    MedicationSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddOnCode {
    Playtime,
    ExitBath,
    PremiumSuite,
    Grooming,
    TrainingSession,
}

impl From<AccommodationCode> for lodging_offer::Accommodation {
    fn from(value: AccommodationCode) -> Self {
        match value {
            AccommodationCode::ClassicSuite => Self::ClassicSuite,
            AccommodationCode::LuxurySuite => Self::LuxurySuite,
            AccommodationCode::CatCondo => Self::CatCondo,
        }
    }
}

impl From<lodging_offer::Accommodation> for AccommodationCode {
    fn from(value: lodging_offer::Accommodation) -> Self {
        match value {
            lodging_offer::Accommodation::ClassicSuite => Self::ClassicSuite,
            lodging_offer::Accommodation::LuxurySuite => Self::LuxurySuite,
            lodging_offer::Accommodation::CatCondo => Self::CatCondo,
        }
    }
}

impl From<CareFeatureCode> for lodging_offer::CareFeature {
    fn from(value: CareFeatureCode) -> Self {
        match value {
            CareFeatureCode::DailyHousekeeping => Self::DailyHousekeeping,
            CareFeatureCode::PottyWalks => Self::PottyWalks,
            CareFeatureCode::Bedding => Self::Bedding,
            CareFeatureCode::PawgressReport => Self::PawgressReport,
            CareFeatureCode::FeedingSupport => Self::FeedingSupport,
            CareFeatureCode::MedicationSupport => Self::MedicationSupport,
        }
    }
}

impl From<lodging_offer::CareFeature> for CareFeatureCode {
    fn from(value: lodging_offer::CareFeature) -> Self {
        match value {
            lodging_offer::CareFeature::DailyHousekeeping => Self::DailyHousekeeping,
            lodging_offer::CareFeature::PottyWalks => Self::PottyWalks,
            lodging_offer::CareFeature::Bedding => Self::Bedding,
            lodging_offer::CareFeature::PawgressReport => Self::PawgressReport,
            lodging_offer::CareFeature::FeedingSupport => Self::FeedingSupport,
            lodging_offer::CareFeature::MedicationSupport => Self::MedicationSupport,
        }
    }
}

impl From<AddOnCode> for lodging_offer::AddOn {
    fn from(value: AddOnCode) -> Self {
        match value {
            AddOnCode::Playtime => Self::Playtime,
            AddOnCode::ExitBath => Self::ExitBath,
            AddOnCode::PremiumSuite => Self::PremiumSuite,
            AddOnCode::Grooming => Self::Grooming,
            AddOnCode::TrainingSession => Self::TrainingSession,
        }
    }
}

impl From<lodging_offer::AddOn> for AddOnCode {
    fn from(value: lodging_offer::AddOn) -> Self {
        match value {
            lodging_offer::AddOn::Playtime => Self::Playtime,
            lodging_offer::AddOn::ExitBath => Self::ExitBath,
            lodging_offer::AddOn::PremiumSuite => Self::PremiumSuite,
            lodging_offer::AddOn::Grooming => Self::Grooming,
            lodging_offer::AddOn::TrainingSession => Self::TrainingSession,
        }
    }
}
