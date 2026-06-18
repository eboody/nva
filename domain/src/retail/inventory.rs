//! Inventory contracts for retail stock counts, available units, and reorder threshold decisions.

use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::LocationId;

use super::product::Sku;
use super::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Positive unit threshold used for reorder-at quantities and tracked inventory policy.
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
/// Decision vocabulary for unit count error in retail workflows.
pub enum UnitCountError {
    #[error("retail inventory count requires at least one unit")]
    /// Rejects zero where the pet-resort workflow requires a positive quantity.
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Units physically on hand at a location before reservations or holds are subtracted.
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
/// Units already reserved for checkout, service bundles, or staff-held transactions.
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
/// Units available to sell after reserved units are subtracted from on-hand stock.
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
/// Source stock record promoted from POS/inventory data before invariant checks are applied.
pub struct Stock {
    /// Source-derived location id carried by this retail contract.
    pub location_id: LocationId,
    /// Source-derived sku carried by this retail contract.
    pub sku: Sku,
    /// Source-derived on hand carried by this retail contract.
    pub on_hand: OnHandUnits,
    /// Source-derived reserved carried by this retail contract.
    pub reserved: ReservedUnits,
    /// Source-derived reorder at carried by this retail contract.
    pub reorder_at: UnitCount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Validated inventory position that guarantees reserved units do not exceed on-hand units.
pub struct Position {
    /// Source-derived location id carried by this retail contract.
    pub location_id: LocationId,
    sku: Sku,
    on_hand: OnHandUnits,
    reserved: ReservedUnits,
    reorder_at: UnitCount,
}

impl Position {
    /// Records a stock position after rejecting impossible inventory math.
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

    /// Returns the sku evidence recorded on this retail contract.
    pub fn sku(&self) -> &Sku {
        &self.sku
    }

    /// Returns the on hand evidence recorded on this retail contract.
    pub const fn on_hand(&self) -> OnHandUnits {
        self.on_hand
    }

    /// Returns the reserved evidence recorded on this retail contract.
    pub const fn reserved(&self) -> ReservedUnits {
        self.reserved
    }

    /// Returns the reorder at evidence recorded on this retail contract.
    pub const fn reorder_at(&self) -> UnitCount {
        self.reorder_at
    }

    /// Computes sellable units so POS and recommendation workflows do not oversell stock.
    pub const fn available_units(&self) -> AvailableUnits {
        AvailableUnits(self.on_hand.get() - self.reserved.get())
    }

    /// Reports whether available inventory has fallen to the reorder threshold.
    pub const fn is_at_or_below_reorder_threshold(&self) -> bool {
        self.available_units().get() <= self.reorder_at.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Groomer-assignment policies used when booking grooming work.
pub enum Policy {
    /// Not tracked retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    NotTracked,
    /// Tracked retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    Tracked {
        /// Source-derived on hand carried by this retail contract.
        on_hand: UnitCount,
        /// Source-derived reorder at carried by this retail contract.
        reorder_at: UnitCount,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Availability status used by recommendation policy to suppress unavailable products.
pub enum Availability {
    /// Available retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    Available,
    /// Out of stock retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    OutOfStock,
    /// Backordered retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    Backordered,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}
