use serde::{Deserialize, Serialize};

/// Storage shape for a migrated daycare service contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContractRecord(pub domain::daycare::Contract);

impl From<domain::daycare::Contract> for ContractRecord {
    fn from(value: domain::daycare::Contract) -> Self {
        Self(value)
    }
}

impl From<ContractRecord> for domain::daycare::Contract {
    fn from(record: ContractRecord) -> Self {
        record.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Storage-facing daycare format code used by service-offering records.
pub enum FormatCode {
    /// Stable storage code for all day play.
    AllDayPlay,
    /// Stable storage code for half day play.
    HalfDayPlay,
    /// Stable storage code for day boarding.
    DayBoarding,
    /// Stable storage code for day play plus room.
    DayPlayPlusRoom,
    /// Stable storage code for cat individual playtime.
    CatIndividualPlaytime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Storage-facing daycare eligibility rule code.
pub enum EligibilityRuleCode {
    /// Stable storage code for temperament review required.
    TemperamentReviewRequired,
    /// Stable storage code for spay neuter required for group play.
    SpayNeuterRequiredForGroupPlay,
    /// Stable storage code for vaccine proof required.
    VaccineProofRequired,
    /// Stable storage code for staff to pet ratio required.
    StaffToPetRatioRequired,
}

impl From<FormatCode> for domain::operations::DaycareFormat {
    fn from(value: FormatCode) -> Self {
        match value {
            FormatCode::AllDayPlay => Self::AllDayPlay,
            FormatCode::HalfDayPlay => Self::HalfDayPlay,
            FormatCode::DayBoarding => Self::DayBoarding,
            FormatCode::DayPlayPlusRoom => Self::DayPlayPlusRoom,
            FormatCode::CatIndividualPlaytime => Self::CatIndividualPlaytime,
        }
    }
}

impl From<domain::operations::DaycareFormat> for FormatCode {
    fn from(value: domain::operations::DaycareFormat) -> Self {
        match value {
            domain::operations::DaycareFormat::AllDayPlay => Self::AllDayPlay,
            domain::operations::DaycareFormat::HalfDayPlay => Self::HalfDayPlay,
            domain::operations::DaycareFormat::DayBoarding => Self::DayBoarding,
            domain::operations::DaycareFormat::DayPlayPlusRoom => Self::DayPlayPlusRoom,
            domain::operations::DaycareFormat::CatIndividualPlaytime => Self::CatIndividualPlaytime,
        }
    }
}

impl From<EligibilityRuleCode> for domain::operations::DaycareEligibilityRule {
    fn from(value: EligibilityRuleCode) -> Self {
        match value {
            EligibilityRuleCode::TemperamentReviewRequired => Self::TemperamentReviewRequired,
            EligibilityRuleCode::SpayNeuterRequiredForGroupPlay => {
                Self::SpayNeuterRequiredForGroupPlay
            }
            EligibilityRuleCode::VaccineProofRequired => Self::VaccineProofRequired,
            EligibilityRuleCode::StaffToPetRatioRequired => Self::StaffToPetRatioRequired,
        }
    }
}

impl From<domain::operations::DaycareEligibilityRule> for EligibilityRuleCode {
    fn from(value: domain::operations::DaycareEligibilityRule) -> Self {
        match value {
            domain::operations::DaycareEligibilityRule::TemperamentReviewRequired => {
                Self::TemperamentReviewRequired
            }
            domain::operations::DaycareEligibilityRule::SpayNeuterRequiredForGroupPlay => {
                Self::SpayNeuterRequiredForGroupPlay
            }
            domain::operations::DaycareEligibilityRule::VaccineProofRequired => {
                Self::VaccineProofRequired
            }
            domain::operations::DaycareEligibilityRule::StaffToPetRatioRequired => {
                Self::StaffToPetRatioRequired
            }
        }
    }
}
