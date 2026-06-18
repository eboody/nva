use serde::{Deserialize, Deserializer, Serialize};

/// Result type returned by fallible money operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures returned by money domain constructors.
pub enum Error {
    #[error("money amount must contain at least one minor unit")]
    /// Signals that amount was blank or missing during money validation.
    EmptyAmount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Typed minor units domain value that keeps raw primitives out of money workflows.
pub struct MinorUnits(u32);

impl MinorUnits {
    /// Promotes a positive minor-unit amount into a money value for resort charges.
    pub fn try_new(value: u32) -> Result<Self> {
        if value == 0 {
            return Err(Error::EmptyAmount);
        }
        Ok(Self(value))
    }

    /// Exposes the validated scalar for serialization and adapter boundaries.
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for MinorUnits {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Domain vocabulary for currency decisions in money workflows.
pub enum Currency {
    /// US dollars, the supported currency for resort charges.
    Usd,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed money domain value that keeps raw primitives out of money workflows.
pub struct Money {
    minor_units: MinorUnits,
    currency: Currency,
}

impl Money {
    /// Assembles a resort money amount from validated minor units and currency.
    pub const fn new(minor_units: MinorUnits, currency: Currency) -> Self {
        Self {
            minor_units,
            currency,
        }
    }

    /// Returns the minor units carried by this money amount.
    pub const fn minor_units(&self) -> MinorUnits {
        self.minor_units
    }

    /// Returns the currency carried by this money amount.
    pub const fn currency(&self) -> Currency {
        self.currency
    }
}
