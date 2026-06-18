//! Secret-free request builders for Gingr endpoint contracts.
//!
//! Endpoint structs describe provider requests without performing network I/O.
//! Callers can inspect the path and parameters, attach source provenance, and only
//! then hand the request to transport code with credentials.
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use gingr::endpoint::{Date, DateRange, LocationId, Request, Reservations};
//!
//! let range = DateRange::new(Date::parse("2026-06-18")?, Date::parse("2026-06-19")?)?;
//! let request = Reservations::for_range(range)
//!     .location(LocationId::new(170))
//!     .build();
//! let parts = request.request_parts();
//!
//! assert_eq!(parts.path(), "/api/v1/reservations");
//! assert!(parts.form_pairs().iter().any(|(key, value)| {
//!     key == "location_id" && value == "170"
//! }));
//! assert!(parts.form_pairs().iter().any(|(key, value)| {
//!     key == "start_date" && value == "2026-06-18"
//! }));
//! # Ok(())
//! # }
//! ```

/// Gingr catalog endpoint boundary with provider parameters kept explicit.
pub mod catalog;
/// Gingr commerce retail endpoint boundary with provider parameters kept explicit.
pub mod commerce_retail;
/// Gingr labor ops endpoint boundary with provider parameters kept explicit.
pub mod labor_ops;
/// Gingr owners animals endpoint boundary with provider parameters kept explicit.
pub mod owners_animals;
/// Gingr reference data endpoint boundary with provider parameters kept explicit.
pub mod reference_data;
/// Gingr report cards files endpoint boundary with provider parameters kept explicit.
pub mod report_cards_files;
/// Gingr reservations endpoint boundary with provider parameters kept explicit.
pub mod reservations;

pub use reservations::Reservations;

use crate::transport;
use chrono::NaiveDate;
use std::fmt;

/// Result type returned by fallible endpoint operations.
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
/// Errors raised when provider values cannot safely cross this Gingr boundary.
pub enum Error {
    #[error("invalid Gingr date {value:?}: expected YYYY-MM-DD")]
    /// Provider date did not match the endpoint date format.
    InvalidDate {
        /// Original provider/caller value rejected before it could become a typed boundary value.
        value: String,
    },
    #[error("invalid Gingr ISO date {value:?}: expected YYYY-MM-DD")]
    /// Provider ISO date could not be parsed for a Gingr request.
    InvalidIsoDate {
        /// Original provider/caller value rejected before it could become a typed boundary value.
        value: String,
    },
    #[error("invalid Gingr date range: start {start} must not be after end {end}")]
    /// Start date is after end date in a Gingr request.
    ReversedDateRange {
        /// Start attached to this Gingr error or DTO.
        start: Date,
        /// End attached to this Gingr error or DTO.
        end: Date,
    },
    #[error("invalid Gingr date range: reservations range may not exceed 30 days")]
    /// Date range exceeds the maximum Gingr endpoint window.
    DateRangeTooLong,
    #[error("invalid Gingr positive integer {value}: expected non-zero value")]
    /// Provider integer wrapper rejected zero or an invalid value.
    InvalidPositiveInteger {
        /// Original provider/caller value rejected before it could become a typed boundary value.
        value: u64,
    },
    #[error("invalid Gingr text value: expected non-empty text")]
    /// Required text parameter was empty after trimming.
    EmptyText,
    #[error("missing required Gingr endpoint parameter {parameter}")]
    /// Typed request builder is missing a required Gingr parameter.
    MissingRequiredParameter {
        /// Name of the provider parameter missing from a typed endpoint builder.
        parameter: &'static str,
    },
    #[error("invalid Gingr legacy date boundary for {date}: {boundary}")]
    /// Request asks Gingr for data before the endpoint-supported cutover date.
    LegacyDateBoundary {
        /// Date carried with this error or record.
        date: String,
        /// Boundary carried with this error or record.
        boundary: &'static str,
    },
    #[error("invalid Gingr pagination: {reason}")]
    /// Pagination parameters would produce an invalid Gingr request.
    InvalidPagination {
        /// Boundary-level reason explaining why this provider request or parse step was rejected.
        reason: &'static str,
    },
    #[error("invalid Gingr subscription bill day {value}: expected 1..=31")]
    /// Subscription bill day was outside Gingr-supported month bounds.
    InvalidBillDayOfMonth {
        /// Original provider/caller value rejected before it could become a typed boundary value.
        value: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// HTTP methods used by typed Gingr endpoint descriptors.
pub enum Method {
    /// Gingr endpoint uses an HTTP GET request.
    Get,
    /// Gingr endpoint uses an HTTP POST request.
    Post,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// Gingr endpoint date formatted as `YYYY-MM-DD` for provider query/form parameters.
pub struct Date(NaiveDate);

impl Date {
    /// Parses provider-sourced text into this boundary type without promoting it to an NVA domain fact.
    pub fn parse(raw: impl AsRef<str>) -> Result<Self> {
        let raw = raw.as_ref();
        NaiveDate::parse_from_str(raw, "%Y-%m-%d")
            .map(Self)
            .map_err(|_| Error::InvalidDate {
                value: raw.to_owned(),
            })
    }

    /// Returns the parsed calendar date used by Gingr endpoint filters.
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
/// Gingr ISO-style date filter formatted as `YYYY-MM-DD` where endpoints use nested params.
pub struct IsoDate(NaiveDate);

impl IsoDate {
    /// Parses provider-sourced text into this boundary type without promoting it to an NVA domain fact.
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
/// Inclusive Gingr date window, capped at the request range this crate explicitly validates.
pub struct DateRange {
    start: Date,
    end: Date,
}

impl DateRange {
    /// Constructs this typed Gingr boundary value after the caller has chosen the provider input to trust.
    pub fn new(start: Date, end: Date) -> Result<Self> {
        if start > end {
            return Err(Error::ReversedDateRange { start, end });
        }
        if (end.inner() - start.inner()).num_days() > 29 {
            return Err(Error::DateRangeTooLong);
        }
        Ok(Self { start, end })
    }

    /// Returns the inclusive start date sent to Gingr.
    pub const fn start(self) -> Date {
        self.start
    }

    /// Returns the inclusive end date sent to Gingr.
    pub const fn end(self) -> Date {
        self.end
    }
}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            serde::Deserialize,
            serde::Serialize,
        )]
        #[serde(transparent)]
        /// Newtype identifier shared by Gingr endpoints that pass numeric provider IDs.
        pub struct $name(u64);

        impl $name {
            /// Wraps an already-observed Gingr identifier without claiming anything beyond provider provenance.
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            /// Returns the provider numeric identifier carried by this wrapper.
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
id_type!(FormId);
id_type!(ReferenceId);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Static Gingr API path emitted by an endpoint descriptor.
pub struct Path(&'static str);

impl Path {
    /// Wraps an already-observed Gingr identifier without claiming anything beyond provider provenance.
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    /// Returns the validated endpoint path segment.
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl PartialEq<&str> for Path {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl fmt::Display for Path {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Positive provider record limit used to bound Gingr list/search responses.
pub struct Limit(u64);

impl Limit {
    /// Constructs this typed Gingr boundary value after the caller has chosen the provider input to trust.
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

/// Defines the behavior required from a request participant in the endpoint workflow.
pub trait Request {
    /// Describes the provider wire contract for this Gingr request.
    fn method(&self) -> Method;
    /// Describes the provider wire contract for this Gingr request.
    fn path(&self) -> &'static str;
    /// Describes the provider wire contract for this Gingr request.
    fn parameters(&self) -> Vec<(String, String)>;
    /// Describes the provider wire contract for this Gingr request.
    fn sensitive_parameter_names(&self) -> &'static [&'static str] {
        &[]
    }

    /// Describes the provider wire contract for this Gingr request.
    fn request_parts(&self) -> transport::RequestParts {
        transport::RequestParts::builder()
            .method(self.method())
            .path(Path::new(self.path()))
            .parameters(self.parameters())
            .sensitive_parameter_names(self.sensitive_parameter_names())
            .build()
    }
}

pub(crate) fn non_empty_text(value: impl Into<String>) -> Result<String> {
    let value = value.into().trim().to_owned();
    if value.is_empty() {
        return Err(Error::EmptyText);
    }
    Ok(value)
}
