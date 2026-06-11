mod error;

use serde::{Deserialize, Deserializer, Serialize};

pub use error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct MinimumAgeWeeks(u8);

impl MinimumAgeWeeks {
    pub fn try_new(value: u8) -> Result<Self> {
        if value == 0 {
            return Err(Error::EmptyMinimumAge);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl<'de> Deserialize<'de> for MinimumAgeWeeks {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgePolicyReason {
    BoardingMinimum,
    DayPlayMinimum,
    DaycareMinimum,
    ServiceSpecificMinimum,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgeThreshold {
    minimum: MinimumAgeWeeks,
    reason: AgePolicyReason,
}

impl AgeThreshold {
    pub const fn new(minimum: MinimumAgeWeeks, reason: AgePolicyReason) -> Self {
        Self { minimum, reason }
    }

    pub const fn minimum(&self) -> MinimumAgeWeeks {
        self.minimum
    }

    pub const fn reason(&self) -> AgePolicyReason {
        self.reason
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct AddOnLabel(String);

impl AddOnLabel {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(Error::EmptyAddOnLabel);
        }
        if value.chars().count() > 120 {
            return Err(Error::AddOnLabelTooLong);
        }
        Ok(Self(value))
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<'de> Deserialize<'de> for AddOnLabel {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionReason {
    CustomerRequested,
    CapacityUnavailable,
    PolicyHardStop,
    MissingRequiredInformation,
    StaffOverride,
}
