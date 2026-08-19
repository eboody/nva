use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Operating-day date used when manager briefs compare booked demand against staffing and room capacity.
pub struct Date(NaiveDate);

impl Date {
    /// Accepts a source/read-model operating date after the adapter has already chosen the resort business day.
    pub const fn try_new(value: NaiveDate) -> Result<Self> {
        Ok(Self(value))
    }

    /// Returns the operating-day date for storage records, analytics projections, or adapter output.
    pub const fn get(self) -> NaiveDate {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Location/service/date key that groups the demand and staffing facts a manager brief can rank.
pub struct Key {
    location_id: LocationId,
    service_line: super::service_core::ServiceLine,
    date: Date,
}

impl Key {
    /// Assembles the resort, service line, and operating day used before analytics can compare labor to demand.
    pub const fn new(
        location_id: LocationId,
        service_line: super::service_core::ServiceLine,
        date: Date,
    ) -> Self {
        Self {
            location_id,
            service_line,
            date,
        }
    }

    /// Returns the resort/location whose staffing or capacity queue is being evaluated.
    pub const fn location_id(&self) -> LocationId {
        self.location_id
    }

    /// Returns the service line whose boarding, daycare, grooming, training, or retail demand is being grouped.
    pub const fn service_line(&self) -> super::service_core::ServiceLine {
        self.service_line
    }

    /// Returns the business day for the manager or regional reporting workflow.
    pub const fn date(&self) -> Date {
        self.date
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures returned by operations domain constructors.
pub enum Error {}

/// Result type for operations values that must reject impossible reporting keys before automation sees them.
pub type Result<T> = std::result::Result<T, Error>;
