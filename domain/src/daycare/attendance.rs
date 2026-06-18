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
pub struct DateRange {
    start: NaiveDate,
    end: NaiveDate,
}

impl DateRange {
    pub fn new(start: NaiveDate, end: NaiveDate) -> std::result::Result<Self, DateRangeError> {
        if end < start {
            return Err(DateRangeError::EndBeforeStart);
        }
        Ok(Self { start, end })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DateRangeError {
    #[error("daycare attendance recurrence end date must not precede start date")]
    EndBeforeStart,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Days(Vec<chrono::Weekday>);

impl Days {
    pub fn try_new(days: Vec<chrono::Weekday>) -> std::result::Result<Self, DaysError> {
        if days.is_empty() {
            return Err(DaysError::Empty);
        }
        Ok(Self(days))
    }

    pub fn contains(&self, day: chrono::Weekday) -> bool {
        self.0.contains(&day)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DaysError {
    #[error("daycare attendance recurrence requires at least one weekday")]
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recurrence {
    pub date_range: DateRange,
    pub days: Days,
}

impl Recurrence {
    pub const fn new(date_range: DateRange, days: Days) -> Self {
        Self { date_range, days }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Materializer;

impl Materializer {
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
