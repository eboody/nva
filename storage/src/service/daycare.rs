use serde::{Deserialize, Serialize};

/// Storage shape for a migrated daycare service contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContractRecord(pub domain::service::daycare::Contract);

impl From<domain::service::daycare::Contract> for ContractRecord {
    fn from(value: domain::service::daycare::Contract) -> Self {
        Self(value)
    }
}

impl From<ContractRecord> for domain::service::daycare::Contract {
    fn from(record: ContractRecord) -> Self {
        record.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormatCode {
    AllDayPlay,
    HalfDayPlay,
    DayBoarding,
    DayPlayPlusRoom,
    CatIndividualPlaytime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EligibilityRuleCode {
    TemperamentReviewRequired,
    SpayNeuterRequiredForGroupPlay,
    VaccineProofRequired,
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
