//! Daycare recurring-attendance materialization for predictable front-desk work queues.
//!
//! ```
//! use chrono::{NaiveDate, Weekday};
//! use domain::daycare;
//!
//! let recurrence = daycare::attendance::Recurrence::new(
//!     daycare::attendance::DateRange::new(
//!         NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
//!         NaiveDate::from_ymd_opt(2026, 6, 19).unwrap(),
//!     )
//!     .unwrap(),
//!     daycare::attendance::Days::try_new(vec![Weekday::Mon, Weekday::Wed, Weekday::Fri]).unwrap(),
//! );
//!
//! let visits = daycare::attendance::Materializer.materialize(&recurrence, &[
//!     NaiveDate::from_ymd_opt(2026, 6, 17).unwrap(),
//! ]);
//! assert_eq!(visits.len(), 2);
//! ```

use super::*;
use chrono::Datelike;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Inclusive date range for recurring daycare attendance materialization.
pub struct DateRange {
    start: NaiveDate,
    end: NaiveDate,
}

impl DateRange {
    /// Creates an attendance date range, rejecting ranges whose end precedes the start.
    pub fn new(start: NaiveDate, end: NaiveDate) -> std::result::Result<Self, DateRangeError> {
        if end < start {
            return Err(DateRangeError::EndBeforeStart);
        }
        Ok(Self { start, end })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation errors for recurring daycare attendance date ranges.
pub enum DateRangeError {
    #[error("daycare attendance recurrence end date must not precede start date")]
    /// The recurrence end date was earlier than the start date.
    EndBeforeStart,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Weekdays on which a recurring daycare package or reservation should materialize visits.
pub struct Days(Vec<chrono::Weekday>);

impl Days {
    /// Creates a non-empty weekday set for recurring daycare attendance.
    pub fn try_new(days: Vec<chrono::Weekday>) -> std::result::Result<Self, DaysError> {
        if days.is_empty() {
            return Err(DaysError::Empty);
        }
        Ok(Self(days))
    }

    /// Reports whether the recurrence includes the supplied weekday.
    pub fn contains(&self, day: chrono::Weekday) -> bool {
        self.0.contains(&day)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation errors for recurring daycare attendance weekdays.
pub enum DaysError {
    #[error("daycare attendance recurrence requires at least one weekday")]
    /// No weekdays were supplied, so no attendance could be materialized.
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Recurring daycare attendance rule used to pre-build front-desk work queues.
pub struct Recurrence {
    /// Inclusive date window in which visits may be materialized.
    pub date_range: DateRange,
    /// Weekdays that should generate visits inside the date range.
    pub days: Days,
}

impl Recurrence {
    /// Creates an attendance date range, rejecting ranges whose end precedes the start.
    pub const fn new(date_range: DateRange, days: Days) -> Self {
        Self { date_range, days }
    }
}

#[derive(Debug, Clone, Default)]
/// Service that expands recurrence rules into concrete daycare visit dates.
pub struct Materializer;

impl Materializer {
    /// Materializes concrete visit dates while excluding source-system exceptions and closures.
    pub fn materialize(&self, recurrence: &Recurrence, exceptions: &[NaiveDate]) -> Vec<NaiveDate> {
        let mut dates = Vec::new();
        let mut current = recurrence.date_range.start;
        while current <= recurrence.date_range.end {
            if recurrence.days.contains(current.weekday()) && !exceptions.contains(&current) {
                dates.push(current);
            }
            current = current
                .succ_opt()
                .expect("bounded date range should have next date");
        }
        dates
    }
}
