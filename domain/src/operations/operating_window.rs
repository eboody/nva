use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Local resort operating-date window for capacity, labor, and manager-brief reporting.
pub struct Local {
    location_id: LocationId,
    timezone: location::Timezone,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

impl Local {
    /// Builds a local operating window only when the end date follows the start date.
    pub fn new(
        location_id: LocationId,
        timezone: location::Timezone,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Self> {
        if end_date <= start_date {
            return Err(Error::EndMustFollowStart);
        }
        Ok(Self {
            location_id,
            timezone,
            start_date,
            end_date,
        })
    }

    /// Resort location whose local business dates define the window.
    pub const fn location_id(&self) -> LocationId {
        self.location_id
    }

    /// Validated IANA timezone used to interpret the local dates.
    pub const fn timezone(&self) -> &location::Timezone {
        &self.timezone
    }

    /// First local operating date included in the window.
    pub const fn start_date(&self) -> NaiveDate {
        self.start_date
    }

    /// Exclusive local operating date ending the window.
    pub const fn end_date(&self) -> NaiveDate {
        self.end_date
    }
}

impl<'de> Deserialize<'de> for Local {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawLocal {
            location_id: LocationId,
            timezone: location::Timezone,
            start_date: NaiveDate,
            end_date: NaiveDate,
        }

        let raw = RawLocal::deserialize(deserializer)?;
        Self::new(raw.location_id, raw.timezone, raw.start_date, raw.end_date)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Local operating-window validation failures.
pub enum Error {
    #[error("local operating window end date must follow start date")]
    /// The end date did not follow the start date.
    EndMustFollowStart,
}

/// Result type returned by local operating-window constructors.
pub type Result<T> = std::result::Result<T, Error>;
