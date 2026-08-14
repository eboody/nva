//! Fallible promotion of raw database columns into validated persistence values.
//!
//! SQL adapters own raw rows. Values leave those adapters only after this module
//! has validated syntax, version identity, and relationships between columns.

use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

/// Result returned by persistence-column promotion.
pub type Result<T> = std::result::Result<T, Error>;

/// Typed failures raised before raw database values can reach application code.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A required text column was empty after trimming.
    #[error("required persistence field {field} is empty")]
    /// Stores the empty component of this boundary value.
    Empty {
        /// Stable field name safe to include in boundary diagnostics.
        field: &'static str,
    },
    /// A database identifier was not a UUID.
    #[error("persistence id is not a valid UUID")]
    InvalidId,
    /// A timestamp was not a valid UTC instant.
    #[error("persistence timestamp is invalid")]
    InvalidTimestamp,
    /// A period ended at or before its start.
    #[error("persistence period must end after it starts")]
    InvalidPeriod,
    /// A currency code is unsupported at this boundary.
    #[error("unsupported persistence currency code")]
    UnsupportedCurrency,
    /// A money amount was negative.
    #[error("persistence money amount cannot be negative")]
    NegativeMoney,
    /// A stable code contained characters outside its storage grammar.
    #[error("persistence field {field} has an invalid stable code")]
    /// Stores the invalid code component of this boundary value.
    InvalidCode {
        /// Stable field name safe to include in boundary diagnostics.
        field: &'static str,
    },
    /// An outbox topic was not confined to the internal namespace.
    #[error("persistence outbox topic must use the internal namespace")]
    UnsafeTopic,
    /// A source-system code was unknown.
    #[error("persistence source system is unknown")]
    UnknownSourceSystem,
    /// A versioned payload was not a JSON object.
    #[error("versioned persistence payload must be a JSON object")]
    PayloadNotObject,
    /// A versioned payload omitted its schema-version member.
    #[error("versioned persistence payload is missing schema_version")]
    PayloadVersionMissing,
    /// The payload's embedded version disagreed with its row version.
    #[error("versioned persistence payload disagrees with its row version")]
    PayloadVersionMismatch,
}

/// UUID identity decoded from a SQL UUID/text column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(Uuid);

impl Id {
    /// Parses a UUID at the persistence boundary.
    pub fn try_new(raw: impl AsRef<str>) -> Result<Self> {
        Uuid::parse_str(raw.as_ref().trim())
            .map(Self)
            .map_err(|_| Error::InvalidId)
    }

    /// Returns the validated UUID.
    pub const fn get(self) -> Uuid {
        self.0
    }
}

/// UTC instant decoded from a SQL timestamp/text column.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Parses an RFC 3339 UTC instant.
    pub fn try_new(raw: impl AsRef<str>) -> Result<Self> {
        raw.as_ref()
            .trim()
            .parse::<DateTime<Utc>>()
            .map(Self)
            .map_err(|_| Error::InvalidTimestamp)
    }

    /// Returns the validated UTC instant.
    pub const fn get(&self) -> &DateTime<Utc> {
        &self.0
    }
}

/// Half-open reporting period whose end is strictly after its start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Period {
    start: Timestamp,
    end: Timestamp,
}

impl Period {
    /// Promotes two timestamp columns after checking their ordering.
    pub fn try_new(start: impl AsRef<str>, end: impl AsRef<str>) -> Result<Self> {
        let start = Timestamp::try_new(start)?;
        let end = Timestamp::try_new(end)?;
        if end <= start {
            return Err(Error::InvalidPeriod);
        }
        Ok(Self { start, end })
    }

    /// Returns the inclusive period start.
    pub const fn start(&self) -> &Timestamp {
        &self.start
    }

    /// Returns the exclusive period end.
    pub const fn end(&self) -> &Timestamp {
        &self.end
    }
}

/// Currency codes accepted by the owned SQL schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrencyCode {
    /// United States dollars.
    Usd,
}

impl CurrencyCode {
    /// Decodes the stable lowercase SQL currency code.
    pub fn try_new(raw: impl AsRef<str>) -> Result<Self> {
        match raw.as_ref().trim() {
            "usd" => Ok(Self::Usd),
            _ => Err(Error::UnsupportedCurrency),
        }
    }
}

/// Relationship-checked money columns promoted as one value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoneyColumns {
    minor_units: u64,
    currency: CurrencyCode,
}

impl MoneyColumns {
    /// Promotes amount and currency together so neither can escape independently.
    pub fn try_new(minor_units: i64, currency: impl AsRef<str>) -> Result<Self> {
        let minor_units = u64::try_from(minor_units).map_err(|_| Error::NegativeMoney)?;
        Ok(Self {
            minor_units,
            currency: CurrencyCode::try_new(currency)?,
        })
    }

    /// Returns the non-negative amount in minor units.
    pub const fn minor_units(self) -> u64 {
        self.minor_units
    }

    /// Returns the validated currency authority.
    pub const fn currency(self) -> CurrencyCode {
        self.currency
    }
}

macro_rules! stable_text {
    ($name:ident, $field:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Validates and normalizes a raw SQL text column.
            pub fn try_new(raw: impl Into<String>) -> Result<Self> {
                stable_code(raw, $field).map(Self)
            }

            /// Returns the validated stable representation.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

stable_text!(
    WorkflowName,
    "workflow_name",
    "Stable workflow code persisted with workflow events."
);
stable_text!(
    Version,
    "version",
    "Stable schema or adapter version label."
);
stable_text!(
    RecordType,
    "record_type",
    "Stable source-record family code."
);

/// Non-empty bounded replay key. Debug intentionally redacts its value.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Validates a replay key before query or insert use.
    pub fn try_new(raw: impl Into<String>) -> Result<Self> {
        required_text(raw, "idempotency_key", 240).map(Self)
    }

    /// Returns the validated key for a bound SQL parameter.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for IdempotencyKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("IdempotencyKey([REDACTED])")
    }
}

/// Internal outbox topic validated against the no-live-side-effect namespace.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Topic(String);

impl Topic {
    /// Accepts only stable topics beneath `internal.`.
    pub fn try_new(raw: impl Into<String>) -> Result<Self> {
        let value = stable_code(raw, "topic")?;
        if !value.starts_with("internal.") {
            return Err(Error::UnsafeTopic);
        }
        Ok(Self(value))
    }

    /// Returns the validated internal topic.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Complete source-reference columns promoted as one relationship-checked value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRefColumns {
    record_ref: domain::source::RecordRef,
    record_type: RecordType,
    observed_at: domain::source::Timestamp,
    adapter_version: Version,
}

impl SourceRefColumns {
    /// Promotes source identity, observation time, and adapter version atomically.
    pub fn try_new(
        system: impl AsRef<str>,
        record_type: impl Into<String>,
        record_id: impl Into<String>,
        observed_at: impl AsRef<str>,
        adapter_version: impl Into<String>,
    ) -> Result<Self> {
        let system = system
            .as_ref()
            .trim()
            .parse::<domain::source::System>()
            .map_err(|_| Error::UnknownSourceSystem)?;
        let record_id = domain::source::record::Id::try_new(record_id)
            .map_err(|_| Error::InvalidCode { field: "record_id" })?;
        let observed_at =
            domain::source::Timestamp::try_new(observed_at).map_err(|_| Error::InvalidTimestamp)?;
        Ok(Self {
            record_ref: domain::source::RecordRef::new(system, record_id),
            record_type: RecordType::try_new(record_type)?,
            observed_at,
            adapter_version: Version::try_new(adapter_version)?,
        })
    }

    /// Returns the validated source-record identity.
    pub const fn record_ref(&self) -> &domain::source::RecordRef {
        &self.record_ref
    }

    /// Returns the validated provider-native record id.
    pub const fn record_id(&self) -> &domain::source::record::Id {
        self.record_ref.record_id()
    }

    /// Returns the validated source-record family.
    pub const fn record_type(&self) -> &RecordType {
        &self.record_type
    }

    /// Returns the source observation instant.
    pub const fn observed_at(&self) -> &domain::source::Timestamp {
        &self.observed_at
    }

    /// Returns the adapter version that interpreted the source record.
    pub const fn adapter_version(&self) -> &Version {
        &self.adapter_version
    }
}

/// JSON payload bound to an explicit and matching schema version.
#[derive(Debug, Clone, PartialEq)]
pub struct VersionedPayload {
    version: Version,
    value: Value,
}

impl VersionedPayload {
    /// Validates an object payload and requires its embedded version to match the row version.
    pub fn try_new(version: impl Into<String>, value: Value) -> Result<Self> {
        let version = Version::try_new(version)?;
        let object = value.as_object().ok_or(Error::PayloadNotObject)?;
        let embedded = object
            .get("schema_version")
            .and_then(Value::as_str)
            .ok_or(Error::PayloadVersionMissing)?;
        if embedded != version.as_str() {
            return Err(Error::PayloadVersionMismatch);
        }
        Ok(Self { version, value })
    }

    /// Returns the validated schema version.
    pub const fn version(&self) -> &Version {
        &self.version
    }

    /// Returns the relationship-checked JSON object.
    pub const fn value(&self) -> &Value {
        &self.value
    }
}

fn required_text(raw: impl Into<String>, field: &'static str, max: usize) -> Result<String> {
    let value = raw.into().trim().to_owned();
    if value.is_empty() {
        return Err(Error::Empty { field });
    }
    if value.chars().count() > max {
        return Err(Error::InvalidCode { field });
    }
    Ok(value)
}

fn stable_code(raw: impl Into<String>, field: &'static str) -> Result<String> {
    let value = required_text(raw, field, 160)?;
    if !value.chars().all(|character| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || matches!(character, '_' | '-' | '.' | ':')
    }) {
        return Err(Error::InvalidCode { field });
    }
    Ok(value)
}
