use serde::{Deserialize, Deserializer, Serialize};

use crate::operations::{Error, Result, StorageField};

/// Storage shape for a migrated training service contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContractRecord(pub domain::training::Contract);

impl From<domain::training::Contract> for ContractRecord {
    fn from(value: domain::training::Contract) -> Self {
        Self(value)
    }
}

impl From<ContractRecord> for domain::training::Contract {
    fn from(record: ContractRecord) -> Self {
        record.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramRecord {
    StayAndStudy {
        duration_weeks: StoredProgramDurationWeeks,
    },
    TutorSession,
    GroupClass,
    PuppyKindergarten,
    PrivateLesson,
    AkcCanineGoodCitizenPrep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct StoredProgramDurationWeeks(u8);

impl StoredProgramDurationWeeks {
    pub const fn try_new(value: u8) -> std::result::Result<Self, StoredProgramDurationWeeksError> {
        if value == 0 {
            return Err(StoredProgramDurationWeeksError::ZeroWeeks);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl<'de> Deserialize<'de> for StoredProgramDurationWeeks {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum StoredProgramDurationWeeksError {
    #[error("stored training program duration requires at least one week")]
    ZeroWeeks,
}

impl TryFrom<domain::training::DurationWeeks> for StoredProgramDurationWeeks {
    type Error = Error;

    fn try_from(value: domain::training::DurationWeeks) -> Result<Self> {
        Self::try_new(value.get()).map_err(|err| Error::InvalidDomainValue {
            field: StorageField::TrainingProgramDurationWeeks,
            reason: err.to_string(),
        })
    }
}

impl TryFrom<StoredProgramDurationWeeks> for domain::training::DurationWeeks {
    type Error = Error;

    fn try_from(value: StoredProgramDurationWeeks) -> Result<Self> {
        domain::training::DurationWeeks::try_new(value.get()).map_err(|err| {
            Error::InvalidDomainValue {
                field: StorageField::TrainingProgramDurationWeeks,
                reason: err.to_string(),
            }
        })
    }
}

impl TryFrom<domain::training::Program> for ProgramRecord {
    type Error = Error;

    fn try_from(value: domain::training::Program) -> Result<Self> {
        Ok(match value {
            domain::training::Program::StayAndStudy { duration } => Self::StayAndStudy {
                duration_weeks: duration.try_into()?,
            },
            domain::training::Program::TutorSession => Self::TutorSession,
            domain::training::Program::GroupClass => Self::GroupClass,
            domain::training::Program::PuppyKindergarten => Self::PuppyKindergarten,
            domain::training::Program::PrivateLesson => Self::PrivateLesson,
            domain::training::Program::AkcCanineGoodCitizenPrep => Self::AkcCanineGoodCitizenPrep,
        })
    }
}

impl TryFrom<ProgramRecord> for domain::training::Program {
    type Error = Error;

    fn try_from(value: ProgramRecord) -> Result<Self> {
        Ok(match value {
            ProgramRecord::StayAndStudy { duration_weeks } => Self::StayAndStudy {
                duration: duration_weeks.try_into()?,
            },
            ProgramRecord::TutorSession => Self::TutorSession,
            ProgramRecord::GroupClass => Self::GroupClass,
            ProgramRecord::PuppyKindergarten => Self::PuppyKindergarten,
            ProgramRecord::PrivateLesson => Self::PrivateLesson,
            ProgramRecord::AkcCanineGoodCitizenPrep => Self::AkcCanineGoodCitizenPrep,
        })
    }
}
