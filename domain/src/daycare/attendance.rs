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
/// Typed date range domain value that keeps raw primitives out of daycare workflows.
pub struct DateRange {
    start: NaiveDate,
    end: NaiveDate,
}

impl DateRange {
    /// Assembles this daycare value from already-validated domain parts.
    pub fn new(start: NaiveDate, end: NaiveDate) -> std::result::Result<Self, DateRangeError> {
        if end < start {
            return Err(DateRangeError::EndBeforeStart);
        }
        Ok(Self { start, end })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Domain vocabulary for date range error decisions in daycare workflows.
pub enum DateRangeError {
    #[error("daycare attendance recurrence end date must not precede start date")]
    /// End before start daycare attendance, eligibility, coverage, or package signal.
    EndBeforeStart,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed days domain value that keeps raw primitives out of daycare workflows.
pub struct Days(Vec<chrono::Weekday>);

impl Days {
    /// Validates and creates the daycare value.
    pub fn try_new(days: Vec<chrono::Weekday>) -> std::result::Result<Self, DaysError> {
        if days.is_empty() {
            return Err(DaysError::Empty);
        }
        Ok(Self(days))
    }

    /// Returns the contains for this daycare value.
    pub fn contains(&self, day: chrono::Weekday) -> bool {
        self.0.contains(&day)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Domain vocabulary for days error decisions in daycare workflows.
pub enum DaysError {
    #[error("daycare attendance recurrence requires at least one weekday")]
    /// Empty daycare attendance, eligibility, coverage, or package signal.
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed recurrence domain value that keeps raw primitives out of daycare workflows.
pub struct Recurrence {
    /// Date range fact promoted into this daycare contract.
    pub date_range: DateRange,
    /// Days fact promoted into this daycare contract.
    pub days: Days,
}

impl Recurrence {
    /// Assembles this daycare value from already-validated domain parts.
    pub const fn new(date_range: DateRange, days: Days) -> Self {
        Self { date_range, days }
    }
}

#[derive(Debug, Clone, Default)]
/// Typed materializer domain value that keeps raw primitives out of daycare workflows.
pub struct Materializer;

impl Materializer {
    /// Returns the materialize for this daycare value.
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
