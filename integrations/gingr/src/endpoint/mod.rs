pub mod catalog;
pub mod commerce_retail;
pub mod labor_ops;
pub mod owners_animals;
pub mod reference_data;
pub mod report_cards_files;
pub mod reservations;

use crate::transport;
use chrono::NaiveDate;
use std::fmt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("invalid Gingr date {value:?}: expected YYYY-MM-DD")]
    InvalidDate { value: String },
    #[error("invalid Gingr ISO date {value:?}: expected YYYY-MM-DD")]
    InvalidIsoDate { value: String },
    #[error("invalid Gingr date range: start {start} must not be after end {end}")]
    ReversedDateRange { start: Date, end: Date },
    #[error("invalid Gingr date range: reservations range may not exceed 30 days")]
    DateRangeTooLong,
    #[error("invalid Gingr positive integer {value}: expected non-zero value")]
    InvalidPositiveInteger { value: u64 },
    #[error("invalid Gingr text value: expected non-empty text")]
    EmptyText,
    #[error("missing required Gingr endpoint parameter {parameter}")]
    MissingRequiredParameter { parameter: &'static str },
    #[error("invalid Gingr legacy date boundary for {date}: {boundary}")]
    LegacyDateBoundary {
        date: String,
        boundary: &'static str,
    },
    #[error("invalid Gingr pagination: {reason}")]
    InvalidPagination { reason: &'static str },
    #[error("invalid Gingr subscription bill day {value}: expected 1..=31")]
    InvalidBillDayOfMonth { value: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date(NaiveDate);

impl Date {
    pub fn parse(raw: impl AsRef<str>) -> Result<Self> {
        let raw = raw.as_ref();
        NaiveDate::parse_from_str(raw, "%Y-%m-%d")
            .map(Self)
            .map_err(|_| Error::InvalidDate {
                value: raw.to_owned(),
            })
    }

    pub const fn inner(self) -> NaiveDate {
        self.0
    }
}

impl fmt::Display for Date {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.format("%Y-%m-%d"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IsoDate(NaiveDate);

impl IsoDate {
    pub fn parse(raw: impl AsRef<str>) -> Result<Self> {
        let raw = raw.as_ref();
        NaiveDate::parse_from_str(raw, "%Y-%m-%d")
            .map(Self)
            .map_err(|_| Error::InvalidIsoDate {
                value: raw.to_owned(),
            })
    }
}

impl fmt::Display for IsoDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.format("%Y-%m-%d"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateRange {
    start: Date,
    end: Date,
}

impl DateRange {
    pub fn new(start: Date, end: Date) -> Result<Self> {
        if start > end {
            return Err(Error::ReversedDateRange { start, end });
        }
        if (end.inner() - start.inner()).num_days() > 29 {
            return Err(Error::DateRangeTooLong);
        }
        Ok(Self { start, end })
    }

    pub const fn start(self) -> Date {
        self.start
    }

    pub const fn end(self) -> Date {
        self.end
    }
}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u64);

        impl $name {
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            pub const fn get(self) -> u64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}", self.0)
            }
        }
    };
}

id_type!(AnimalId);
id_type!(OwnerId);
id_type!(ReservationId);
id_type!(LocationId);
id_type!(SpeciesId);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limit(u64);

impl Limit {
    pub fn new(value: u64) -> Result<Self> {
        if value == 0 {
            return Err(Error::InvalidPositiveInteger { value });
        }
        Ok(Self(value))
    }
}

impl fmt::Display for Limit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

pub trait Request {
    fn method(&self) -> Method;
    fn path(&self) -> &'static str;
    fn parameters(&self) -> Vec<(String, String)>;
    fn sensitive_parameter_names(&self) -> &'static [&'static str] {
        &[]
    }

    fn request_parts(&self) -> transport::RequestParts {
        transport::RequestParts::new(
            self.method(),
            self.path(),
            self.parameters(),
            self.sensitive_parameter_names(),
        )
    }
}

pub(crate) fn non_empty_text(value: impl Into<String>) -> Result<String> {
    let value = value.into().trim().to_owned();
    if value.is_empty() {
        return Err(Error::EmptyText);
    }
    Ok(value)
}
