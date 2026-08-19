use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Financial and attribution period with explicit local-date or UTC-instant semantics.
pub enum Period {
    /// Local operating-date period for resort manager and finance reporting.
    LocalOperatingDates {
        /// Resort location whose timezone owns the local date boundary.
        location_id: LocationId,
        /// IANA timezone used to interpret the local dates.
        timezone: location::Timezone,
        /// First local operating date included in the period.
        start_date: NaiveDate,
        /// Exclusive local operating date ending the period.
        end_date: NaiveDate,
    },
    /// UTC instant period for SLA, source ingestion, and attribution timestamps.
    UtcInstants {
        /// Resort location associated with the reporting window.
        location_id: LocationId,
        /// Start instant included in the period.
        start: DateTime<Utc>,
        /// End instant excluded from the period.
        end: DateTime<Utc>,
    },
}

impl Period {
    /// Creates a local operating-date reporting period.
    pub fn local_operating_dates(
        location_id: LocationId,
        timezone: location::Timezone,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Self> {
        if end_date <= start_date {
            return Err(Error::EndMustFollowStart);
        }
        Ok(Self::LocalOperatingDates {
            location_id,
            timezone,
            start_date,
            end_date,
        })
    }

    /// Creates a UTC-instant reporting or attribution period.
    pub fn utc_instants(
        location_id: LocationId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Self> {
        if end <= start {
            return Err(Error::EndMustFollowStart);
        }
        Ok(Self::UtcInstants {
            location_id,
            start,
            end,
        })
    }
}

impl<'de> Deserialize<'de> for Period {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        enum RawPeriod {
            LocalOperatingDates {
                location_id: LocationId,
                timezone: location::Timezone,
                start_date: NaiveDate,
                end_date: NaiveDate,
            },
            UtcInstants {
                location_id: LocationId,
                start: DateTime<Utc>,
                end: DateTime<Utc>,
            },
        }

        match RawPeriod::deserialize(deserializer)? {
            RawPeriod::LocalOperatingDates {
                location_id,
                timezone,
                start_date,
                end_date,
            } => Self::local_operating_dates(location_id, timezone, start_date, end_date),
            RawPeriod::UtcInstants {
                location_id,
                start,
                end,
            } => Self::utc_instants(location_id, start, end),
        }
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Reporting-period validation failures.
pub enum Error {
    #[error("reporting period end must follow start")]
    /// The end date or instant did not follow the start.
    EndMustFollowStart,
}

/// Result type returned by reporting-period constructors.
pub type Result<T> = std::result::Result<T, Error>;
