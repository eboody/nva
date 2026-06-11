mod error;

use serde::{Deserialize, Deserializer, Serialize};

pub use error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct MinorUnits(u32);

impl MinorUnits {
    pub fn try_new(value: u32) -> Result<Self> {
        if value == 0 {
            return Err(Error::EmptyAmount);
        }
        Ok(Self(value))
    }

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
pub enum Currency {
    Usd,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    minor_units: MinorUnits,
    currency: Currency,
}

impl Money {
    pub const fn new(minor_units: MinorUnits, currency: Currency) -> Self {
        Self {
            minor_units,
            currency,
        }
    }

    pub const fn minor_units(&self) -> MinorUnits {
        self.minor_units
    }

    pub const fn currency(&self) -> Currency {
        self.currency
    }
}
