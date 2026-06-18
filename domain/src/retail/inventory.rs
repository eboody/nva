use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::LocationId;

use super::product::Sku;
use super::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Typed unit count domain value that keeps raw primitives out of retail workflows.
pub struct UnitCount(u32);

impl UnitCount {
    /// Promotes boundary input into a validated retail domain value.
    pub const fn try_new(value: u32) -> std::result::Result<Self, UnitCountError> {
        if value == 0 {
            return Err(UnitCountError::Zero);
        }
        Ok(Self(value))
    }

    /// Exposes the validated scalar for serialization and adapter boundaries.
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for UnitCount {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Domain vocabulary for unit count error decisions in retail workflows.
pub enum UnitCountError {
    #[error("retail inventory count requires at least one unit")]
    /// Rejects zero where the pet-resort workflow requires a positive quantity.
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Typed on hand units domain value that keeps raw primitives out of retail workflows.
pub struct OnHandUnits(u32);

impl OnHandUnits {
    /// Assembles this retail value from already-validated domain parts.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Exposes the validated scalar for serialization and adapter boundaries.
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Typed reserved units domain value that keeps raw primitives out of retail workflows.
pub struct ReservedUnits(u32);

impl ReservedUnits {
    /// Assembles this retail value from already-validated domain parts.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Exposes the validated scalar for serialization and adapter boundaries.
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Typed available units domain value that keeps raw primitives out of retail workflows.
pub struct AvailableUnits(u32);

impl AvailableUnits {
    /// Assembles this retail value from already-validated domain parts.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Exposes the validated scalar for serialization and adapter boundaries.
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed stock domain value that keeps raw primitives out of retail workflows.
pub struct Stock {
    /// Location id fact promoted into this retail contract.
    pub location_id: LocationId,
    /// Sku fact promoted into this retail contract.
    pub sku: Sku,
    /// On hand fact promoted into this retail contract.
    pub on_hand: OnHandUnits,
    /// Reserved fact promoted into this retail contract.
    pub reserved: ReservedUnits,
    /// Reorder at fact promoted into this retail contract.
    pub reorder_at: UnitCount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed position domain value that keeps raw primitives out of retail workflows.
pub struct Position {
    /// Location id fact promoted into this retail contract.
    pub location_id: LocationId,
    sku: Sku,
    on_hand: OnHandUnits,
    reserved: ReservedUnits,
    reorder_at: UnitCount,
}

impl Position {
    /// Returns the record for this retail value.
    pub fn record(stock: Stock) -> Result<Self> {
        if stock.reserved.get() > stock.on_hand.get() {
            return Err(Error::ReservedUnitsExceedOnHand);
        }
        Ok(Self {
            location_id: stock.location_id,
            sku: stock.sku,
            on_hand: stock.on_hand,
            reserved: stock.reserved,
            reorder_at: stock.reorder_at,
        })
    }

    /// Returns the sku for this retail value.
    pub fn sku(&self) -> &Sku {
        &self.sku
    }

    /// Returns this retail value's on hand.
    pub const fn on_hand(&self) -> OnHandUnits {
        self.on_hand
    }

    /// Returns this retail value's reserved.
    pub const fn reserved(&self) -> ReservedUnits {
        self.reserved
    }

    /// Returns this retail value's reorder at.
    pub const fn reorder_at(&self) -> UnitCount {
        self.reorder_at
    }

    /// Returns this retail value's available units.
    pub const fn available_units(&self) -> AvailableUnits {
        AvailableUnits(self.on_hand.get() - self.reserved.get())
    }

    /// Returns this retail value's is at or below reorder threshold.
    pub const fn is_at_or_below_reorder_threshold(&self) -> bool {
        self.available_units().get() <= self.reorder_at.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Groomer-assignment policies used when booking grooming work.
pub enum Policy {
    /// Not tracked retail inventory, POS, reorder, or recommendation signal.
    NotTracked,
    /// Tracked retail inventory, POS, reorder, or recommendation signal.
    Tracked {
        /// On hand fact promoted into this retail contract.
        on_hand: UnitCount,
        /// Reorder at fact promoted into this retail contract.
        reorder_at: UnitCount,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for availability decisions in retail workflows.
pub enum Availability {
    /// Available retail inventory, POS, reorder, or recommendation signal.
    Available,
    /// Out of stock retail inventory, POS, reorder, or recommendation signal.
    OutOfStock,
    /// Backordered retail inventory, POS, reorder, or recommendation signal.
    Backordered,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}
