use super::*;
use crate::operations::time_bucket as time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
/// Positive labor minute count.
pub struct Minutes(u16);

impl Minutes {
    /// Creates a nonzero labor minute count.
    pub const fn try_new(value: u16) -> Result<Self, Error> {
        if value == 0 {
            return Err(Error::ZeroMinutes);
        }
        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for Minutes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u16::deserialize(deserializer)?;
        Self::try_new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Signed minute delta for add/remove/reassign recommendations.
pub struct SignedMinutes(i32);

impl SignedMinutes {
    /// Raw signed minute delta.
    pub const fn get(self) -> i32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
/// Positive people count for staffing coverage.
pub struct PeopleCount(u16);

impl PeopleCount {
    /// Creates a nonzero people count.
    pub const fn try_new(value: u16) -> Result<Self, Error> {
        if value == 0 {
            return Err(Error::ZeroPeople);
        }
        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for PeopleCount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u16::deserialize(deserializer)?;
        Self::try_new(value).map_err(serde::de::Error::custom)
    }
}

/// Canonical resort labor role in coverage and staffing recommendations.
pub use crate::staff::role::Role;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Scheduled labor coverage for a role/time bucket.
pub struct ScheduledCoverage {
    location_id: entities::LocationId,
    bucket: time::Window,
    role: Role,
    scheduled_people: PeopleCount,
    scheduled_minutes: Minutes,
    loaded_cost: money::Money,
}

impl ScheduledCoverage {
    /// Resort location covered by this scheduled labor fact.
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    /// Time bucket covered by this scheduled labor fact.
    pub const fn bucket(&self) -> time::Window {
        self.bucket
    }

    /// Labor role covered by this scheduled labor fact.
    pub const fn role(&self) -> Role {
        self.role
    }

    /// Scheduled labor minutes available in the bucket.
    pub const fn scheduled_minutes(&self) -> Minutes {
        self.scheduled_minutes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Labor validation failure.
pub enum Error {
    #[error("labor minutes must be greater than zero")]
    /// Zero labor minutes would erase the work requirement.
    ZeroMinutes,
    #[error("people count must be greater than zero")]
    /// Zero people cannot represent coverage.
    ZeroPeople,
}
