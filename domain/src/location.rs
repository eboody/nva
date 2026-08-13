//! Resort-location values used to scope policies, schedules, labor, and source data.
//!
//! Location labels and time zones are source-of-truth facts for multi-site automation. They keep
//! manager briefings, reservation windows, and labor-cost reports tied to the correct resort rather
//! than relying on loose strings from individual systems.

use nutype::nutype;
use serde::{Deserialize, Deserializer, Serialize};

/// Display name for a resort location or brand-specific site.
///
/// This is the human-readable location label used in manager briefings, audit events, and staff
/// workflows across the 170-location portfolio.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct Name(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// IANA timezone identifier for a resort location.
///
/// Timezone is part of the safety boundary for reminders, check-in windows, daily briefings, and
/// labor reporting; unknown provider labels must be normalized before they can drive local
/// operational windows or reporting periods.
///
/// ```
/// let timezone = domain::location::Timezone::try_new(" America/New_York ").unwrap();
/// assert_eq!(timezone.as_str(), "America/New_York");
/// assert!(domain::location::Timezone::try_new("America/Not_A_Zone").is_err());
/// ```
pub struct Timezone(String);

impl Timezone {
    /// Validates and trims a location timezone before local operating dates rely on it.
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(Error::EmptyTimezone);
        }
        if value.len() > 80 {
            return Err(Error::TimezoneTooLong);
        }
        if !is_supported_iana_timezone(&value) {
            return Err(Error::UnknownTimezone);
        }
        Ok(Self(value))
    }

    /// Returns the timezone identifier used by adapters and reporting windows.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the timezone into its validated string identifier.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<'de> Deserialize<'de> for Timezone {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Location timezone validation failures.
pub enum Error {
    #[error("location timezone must not be empty")]
    /// Empty timezone labels cannot safely anchor local operating dates.
    EmptyTimezone,
    #[error("location timezone is too long")]
    /// Timezone label exceeded the supported storage/display length.
    TimezoneTooLong,
    #[error("location timezone must be a known IANA timezone")]
    /// Unknown provider timezone labels must be normalized before they drive operations.
    UnknownTimezone,
}

/// Result type returned by location value constructors.
pub type Result<T> = std::result::Result<T, Error>;

fn is_supported_iana_timezone(value: &str) -> bool {
    matches!(
        value,
        "UTC"
            | "Etc/UTC"
            | "America/New_York"
            | "America/Chicago"
            | "America/Denver"
            | "America/Phoenix"
            | "America/Los_Angeles"
            | "America/Anchorage"
            | "Pacific/Honolulu"
    )
}
