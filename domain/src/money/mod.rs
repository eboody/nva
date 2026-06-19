use serde::{Deserialize, Deserializer, Serialize};

/// Result type returned by fallible money operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures returned by money domain constructors.
pub enum Error {
    #[error("money amount must contain at least one minor unit")]
    /// Zero minor units would hide a charge, refund, deposit, or revenue value, so money construction fails.
    EmptyAmount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Positive minor-unit amount used so checkout, deposit, refund, and revenue math cannot silently use zero.
pub struct MinorUnits(u32);

impl MinorUnits {
    /// Promotes a positive minor-unit amount into a money value for resort charges.
    pub fn try_new(value: u32) -> Result<Self> {
        if value == 0 {
            return Err(Error::EmptyAmount);
        }
        Ok(Self(value))
    }

    /// Returns minor units for checkout, deposit, refund, or ledger adapters.
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
/// Currency vocabulary for resort money amounts before checkout or reporting adapters serialize them.
pub enum Currency {
    /// US dollars, the supported currency for resort charges.
    Usd,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Money amount for resort charges and reports; payment, refund, or discount movement still requires the app workflow gate.
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

    /// Returns the minor units used for checkout, deposit, refund, or reporting calculations.
    pub const fn minor_units(&self) -> MinorUnits {
        self.minor_units
    }

    /// Returns the currency used for checkout, deposit, refund, or reporting calculations.
    pub const fn currency(&self) -> Currency {
        self.currency
    }
}
