//! Training storage projection codes and validated program-duration quantities.
//!
//! Training records preserve program choices such as stay-and-study, tutor
//! sessions, and AKC prep. Duration is validated before persistence so runtime
//! workflows cannot report impossible zero-week programs as source evidence.

use serde::{Deserialize, Deserializer, Serialize};

use domain::training::program;

use crate::operations::{self, StorageField};

/// Storage shape for a migrated training service rules.
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
/// Storage-facing training program code, including duration for stay-and-study programs.
pub enum ProgramRecord {
    /// Stable storage code for stay and study.
    StayAndStudy {
        /// Training duration in weeks for stay-and-study programs.
        duration_weeks: StoredProgramDurationWeeks,
    },
    /// Stable storage code for tutor session.
    TutorSession,
    /// Stable storage code for group class.
    GroupClass,
    /// Stable storage code for puppy kindergarten.
    PuppyKindergarten,
    /// Stable storage code for private lesson.
    PrivateLesson,
    /// Stable storage code for akc canine good citizen prep.
    AkcCanineGoodCitizenPrep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Positive training duration persisted in weeks for stay-and-study offerings.
pub struct StoredProgramDurationWeeks(u8);

impl StoredProgramDurationWeeks {
    /// Validates and wraps a positive quantity before it is persisted.
    pub const fn try_new(value: u8) -> std::result::Result<Self, StoredProgramDurationWeeksError> {
        if value == 0 {
            return Err(StoredProgramDurationWeeksError::ZeroWeeks);
        }
        Ok(Self(value))
    }

    /// Returns the provider numeric identifier kept on this wrapper.
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
/// Validation failures for persisted training-program durations.
pub enum StoredProgramDurationWeeksError {
    #[error("stored training program duration requires at least one week")]
    /// Stable storage code for zero weeks.
    ZeroWeeks,
}

impl TryFrom<program::DurationWeeks> for StoredProgramDurationWeeks {
    type Error = operations::Error;

    fn try_from(value: program::DurationWeeks) -> operations::Result<Self> {
        Self::try_new(value.get()).map_err(|err| operations::Error::InvalidDomainValue {
            field: StorageField::TrainingProgramDurationWeeks,
            reason: err.to_string(),
        })
    }
}

impl TryFrom<StoredProgramDurationWeeks> for program::DurationWeeks {
    type Error = operations::Error;

    fn try_from(value: StoredProgramDurationWeeks) -> operations::Result<Self> {
        program::DurationWeeks::try_new(value.get()).map_err(|err| {
            operations::Error::InvalidDomainValue {
                field: StorageField::TrainingProgramDurationWeeks,
                reason: err.to_string(),
            }
        })
    }
}

impl TryFrom<domain::training::Program> for ProgramRecord {
    type Error = operations::Error;

    fn try_from(value: domain::training::Program) -> operations::Result<Self> {
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
    type Error = operations::Error;

    fn try_from(value: ProgramRecord) -> operations::Result<Self> {
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
