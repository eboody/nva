use serde::{Deserialize, Serialize};

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

impl From<AccommodationCode> for domain::operations::BoardingAccommodation {
    fn from(value: AccommodationCode) -> Self {
        match value {
            AccommodationCode::ClassicSuite => Self::ClassicSuite,
            AccommodationCode::LuxurySuite => Self::LuxurySuite,
            AccommodationCode::CatCondo => Self::CatCondo,
        }
    }
}

impl From<domain::operations::BoardingAccommodation> for AccommodationCode {
    fn from(value: domain::operations::BoardingAccommodation) -> Self {
        match value {
            domain::operations::BoardingAccommodation::ClassicSuite => Self::ClassicSuite,
            domain::operations::BoardingAccommodation::LuxurySuite => Self::LuxurySuite,
            domain::operations::BoardingAccommodation::CatCondo => Self::CatCondo,
        }
    }
}

impl From<CareFeatureCode> for domain::operations::BoardingCareFeature {
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

impl From<domain::operations::BoardingCareFeature> for CareFeatureCode {
    fn from(value: domain::operations::BoardingCareFeature) -> Self {
        match value {
            domain::operations::BoardingCareFeature::DailyHousekeeping => Self::DailyHousekeeping,
            domain::operations::BoardingCareFeature::PottyWalks => Self::PottyWalks,
            domain::operations::BoardingCareFeature::Bedding => Self::Bedding,
            domain::operations::BoardingCareFeature::PawgressReport => Self::PawgressReport,
            domain::operations::BoardingCareFeature::FeedingSupport => Self::FeedingSupport,
            domain::operations::BoardingCareFeature::MedicationSupport => Self::MedicationSupport,
        }
    }
}

impl From<AddOnCode> for domain::operations::BoardingAddOn {
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

impl From<domain::operations::BoardingAddOn> for AddOnCode {
    fn from(value: domain::operations::BoardingAddOn) -> Self {
        match value {
            domain::operations::BoardingAddOn::Playtime => Self::Playtime,
            domain::operations::BoardingAddOn::ExitBath => Self::ExitBath,
            domain::operations::BoardingAddOn::PremiumSuite => Self::PremiumSuite,
            domain::operations::BoardingAddOn::Grooming => Self::Grooming,
            domain::operations::BoardingAddOn::TrainingSession => Self::TrainingSession,
        }
    }
}
