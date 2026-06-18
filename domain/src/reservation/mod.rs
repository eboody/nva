mod error;

use serde::{Deserialize, Deserializer, Serialize};

pub use error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Minimum pet age, in weeks, required before a service can be booked.
pub struct MinimumAgeWeeks(u8);

impl MinimumAgeWeeks {
    /// Validates and creates the reservation value.
    pub fn try_new(value: u8) -> Result<Self> {
        if value == 0 {
            return Err(Error::EmptyMinimumAge);
        }
        Ok(Self(value))
    }

    /// Exposes the validated scalar for serialization and adapter boundaries.
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
/// Reasons a pet may be blocked by age rules for a service.
pub enum AgePolicyReason {
    /// Minimum-age rule for boarding reservations.
    BoardingMinimum,
    /// Minimum-age rule for day-play reservations.
    DayPlayMinimum,
    /// Minimum-age rule for daycare reservations.
    DaycareMinimum,
    /// Minimum-age rule configured for a specific service.
    ServiceSpecificMinimum,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Age gate applied when determining whether a pet may book a service.
pub struct AgeThreshold {
    minimum: MinimumAgeWeeks,
    reason: AgePolicyReason,
}

impl AgeThreshold {
    /// Assembles this reservation value from already-validated domain parts.
    pub const fn new(minimum: MinimumAgeWeeks, reason: AgePolicyReason) -> Self {
        Self { minimum, reason }
    }

    /// Returns this reservation value's minimum.
    pub const fn minimum(&self) -> MinimumAgeWeeks {
        self.minimum
    }

    /// Returns this reservation value's reason.
    pub const fn reason(&self) -> AgePolicyReason {
        self.reason
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Display label for an optional reservation add-on offered to the customer.
pub struct AddOnLabel(String);

impl AddOnLabel {
    /// Validates and creates the reservation value.
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

    /// Returns the owned inner string for storage or outbound mapping.
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
/// Business reasons for moving or rejecting a reservation workflow transition.
pub enum TransitionReason {
    /// Transition was initiated by a customer request.
    CustomerRequested,
    /// Transition was blocked because the requested capacity is unavailable.
    CapacityUnavailable,
    /// Transition is blocked by a non-overridable policy.
    PolicyHardStop,
    /// Transition is blocked until required customer or pet details are supplied.
    MissingRequiredInformation,
    /// Staff manually approved a workflow transition.
    StaffOverride,
}
