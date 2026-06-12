use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::LocationId;

use super::product::Sku;
use super::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct UnitCount(u32);

impl UnitCount {
    pub const fn try_new(value: u32) -> std::result::Result<Self, UnitCountError> {
        if value == 0 {
            return Err(UnitCountError::Zero);
        }
        Ok(Self(value))
    }

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
pub enum UnitCountError {
    #[error("retail inventory count requires at least one unit")]
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OnHandUnits(u32);

impl OnHandUnits {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ReservedUnits(u32);

impl ReservedUnits {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AvailableUnits(u32);

impl AvailableUnits {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stock {
    pub location_id: LocationId,
    pub sku: Sku,
    pub on_hand: OnHandUnits,
    pub reserved: ReservedUnits,
    pub reorder_at: UnitCount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub location_id: LocationId,
    sku: Sku,
    on_hand: OnHandUnits,
    reserved: ReservedUnits,
    reorder_at: UnitCount,
}

impl Position {
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

    pub fn sku(&self) -> &Sku {
        &self.sku
    }

    pub const fn on_hand(&self) -> OnHandUnits {
        self.on_hand
    }

    pub const fn reserved(&self) -> ReservedUnits {
        self.reserved
    }

    pub const fn reorder_at(&self) -> UnitCount {
        self.reorder_at
    }

    pub const fn available_units(&self) -> AvailableUnits {
        AvailableUnits(self.on_hand.get() - self.reserved.get())
    }

    pub const fn is_at_or_below_reorder_threshold(&self) -> bool {
        self.available_units().get() <= self.reorder_at.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Policy {
    NotTracked,
    Tracked {
        on_hand: UnitCount,
        reorder_at: UnitCount,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Availability {
    Available,
    OutOfStock,
    Backordered,
    Unknown,
}
