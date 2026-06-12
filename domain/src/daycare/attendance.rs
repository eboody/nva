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
pub struct AttendanceDays(Vec<chrono::Weekday>);

impl AttendanceDays {
    pub fn try_new(days: Vec<chrono::Weekday>) -> std::result::Result<Self, AttendanceDaysError> {
        if days.is_empty() {
            return Err(AttendanceDaysError::Empty);
        }
        Ok(Self(days))
    }

    pub fn contains(&self, day: chrono::Weekday) -> bool {
        self.0.contains(&day)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AttendanceDaysError {
    #[error("daycare attendance recurrence requires at least one weekday")]
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recurrence {
    pub date_range: DateRange,
    pub days: AttendanceDays,
}

impl Recurrence {
    pub const fn new(date_range: DateRange, days: AttendanceDays) -> Self {
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
