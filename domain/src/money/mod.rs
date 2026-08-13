use serde::{Deserialize, Deserializer, Serialize};

/// Result type returned by fallible money operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
/// Validation failures returned by money domain constructors.
pub enum Error {
    #[error("money amount cannot be negative")]
    /// Financial facts and charges cannot encode negative minor units.
    NegativeAmount,
    #[error("money amount exceeds supported minor-unit range")]
    /// Minor-unit value is too large for checked financial arithmetic.
    AmountOverflow,
    #[error("cannot combine {left:?} money with {right:?} money")]
    /// Checked arithmetic refuses to mix different currency authorities.
    CurrencyMismatch {
        /// Left-hand currency authority.
        left: Currency,
        /// Right-hand currency authority.
        right: Currency,
    },
    #[error("money addition overflowed minor-unit range")]
    /// Adding two money values would exceed the supported range.
    AdditionOverflow,
    #[error("money subtraction would produce a negative amount")]
    /// Deduction values exceed the gross/source amount.
    SubtractionWouldBeNegative,
    #[error("basis points must be between 0 and 10000")]
    /// Basis-points values above 100% are invalid for confidence/utilization percentages.
    BasisPointsOutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Non-negative minor-unit amount used so checkout, deposit, refund, and revenue math can represent true zero while rejecting negative values.
pub struct MinorUnits(u64);

impl MinorUnits {
    /// Promotes a non-negative minor-unit amount into a money value for resort charges or reports.
    pub const fn try_new(value: u64) -> Result<Self> {
        Ok(Self(value))
    }

    /// Returns minor units for checkout, deposit, refund, or ledger adapters.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for MinorUnits {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u64::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Currency vocabulary for resort money amounts before checkout or reporting adapters serialize them.
pub enum Currency {
    /// US dollars, the supported currency for resort charges.
    Usd,
    /// Temporary compatibility currency for source systems that report a named accounting currency before canonical mapping.
    AccountingSystem(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Percentage-like value expressed in basis points for confidence, utilization, and reporting comparisons.
pub struct BasisPoints(u16);

impl BasisPoints {
    /// Creates a basis-points value in the inclusive 0..=10_000 range.
    pub const fn try_new(value: u16) -> Result<Self> {
        if value > 10_000 {
            return Err(Error::BasisPointsOutOfRange);
        }
        Ok(Self(value))
    }

    /// Returns the basis-points value for stable reporting and serde boundaries.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for BasisPoints {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u16::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Money amount for resort charges and reports; payment, refund, or discount movement still requires the app workflow gate.
pub struct Money {
    minor_units: MinorUnits,
    currency: Currency,
}

impl Money {
    /// Builds a money value from an untrusted signed minor-unit amount and currency.
    pub fn try_new(minor_units: i128, currency: Currency) -> Result<Self> {
        if minor_units < 0 {
            return Err(Error::NegativeAmount);
        }
        let minor_units = u64::try_from(minor_units).map_err(|_| Error::AmountOverflow)?;
        Ok(Self::new(MinorUnits::try_new(minor_units)?, currency))
    }

    /// Builds a USD money value from non-negative minor units.
    pub fn usd(minor_units: u64) -> Result<Self> {
        Self::try_new(i128::from(minor_units), Currency::Usd)
    }

    /// Builds a zero money value in the named currency.
    pub fn zero(currency: Currency) -> Self {
        Self::new(MinorUnits(0), currency)
    }

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
    pub fn currency(&self) -> Currency {
        self.currency.clone()
    }

    /// Adds money only when both values share the same currency and the sum fits.
    pub fn checked_add(&self, other: Self) -> Result<Self> {
        self.ensure_same_currency(&other)?;
        let amount = self
            .minor_units
            .get()
            .checked_add(other.minor_units.get())
            .ok_or(Error::AdditionOverflow)?;
        Ok(Self::new(MinorUnits(amount), self.currency.clone()))
    }

    /// Subtracts money only when both values share the same currency and the result is non-negative.
    pub fn checked_sub(&self, other: Self) -> Result<Self> {
        self.ensure_same_currency(&other)?;
        let amount = self
            .minor_units
            .get()
            .checked_sub(other.minor_units.get())
            .ok_or(Error::SubtractionWouldBeNegative)?;
        Ok(Self::new(MinorUnits(amount), self.currency.clone()))
    }

    fn ensure_same_currency(&self, other: &Self) -> Result<()> {
        if self.currency != other.currency {
            return Err(Error::CurrencyMismatch {
                left: self.currency.clone(),
                right: other.currency.clone(),
            });
        }
        Ok(())
    }
}
