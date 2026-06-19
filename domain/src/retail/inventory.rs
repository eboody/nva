//! Inventory models for retail stock counts, available units, and reorder threshold decisions.

use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::LocationId;

use super::product::Sku;
use super::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Positive unit threshold used for reorder-at quantities and tracked inventory policy.
pub struct UnitCount(u32);

impl UnitCount {
    /// Accepts a positive reorder threshold so staff tasks are not created from zero-unit policy values.
    pub const fn try_new(value: u32) -> std::result::Result<Self, UnitCountError> {
        if value == 0 {
            return Err(UnitCountError::Zero);
        }
        Ok(Self(value))
    }

    /// Returns the unit count for storage records, POS mappings, and threshold comparisons.
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
/// Unit-count validation errors that keep reorder thresholds and tracked inventory policy usable by staff.
pub enum UnitCountError {
    #[error("retail inventory count requires at least one unit")]
    /// Rejects zero where the pet-resort workflow requires a positive quantity.
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Units physically on hand at a location before reservations or holds are subtracted.
pub struct OnHandUnits(u32);

impl OnHandUnits {
    /// Records the count reported by the inventory source before availability math is applied.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the unit count for storage records, POS mappings, and threshold comparisons.
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Units already reserved for checkout, service bundles, or staff-held transactions.
pub struct ReservedUnits(u32);

impl ReservedUnits {
    /// Records the count reported by the inventory source before availability math is applied.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the unit count for storage records, POS mappings, and threshold comparisons.
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Units available to sell after reserved units are subtracted from on-hand stock.
pub struct AvailableUnits(u32);

impl AvailableUnits {
    /// Records the count reported by the inventory source before availability math is applied.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the unit count for storage records, POS mappings, and threshold comparisons.
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Source stock record promoted from POS/inventory data before invariant checks are applied.
pub struct Stock {
    /// Location whose shelves or retail stockroom own this inventory count.
    pub location_id: LocationId,
    /// SKU whose on-hand, reserved, and reorder quantities are being evaluated.
    pub sku: Sku,
    /// Physical units counted at the location before reservations or holds are subtracted.
    pub on_hand: OnHandUnits,
    /// Units held for checkout, service bundles, or staff workflows before sale availability is calculated.
    pub reserved: ReservedUnits,
    /// Minimum available units that should prompt manager or vendor reorder attention.
    pub reorder_at: UnitCount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Validated inventory position that guarantees reserved units do not exceed on-hand units.
pub struct Position {
    /// Location whose shelves or retail stockroom own this inventory count.
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

    /// Returns the SKU used to connect this position to catalog, POS, recommendation, and reorder workflows.
    pub fn sku(&self) -> &Sku {
        &self.sku
    }

    /// Returns the physical stock count that anchors oversell and reorder decisions.
    pub const fn on_hand(&self) -> OnHandUnits {
        self.on_hand
    }

    /// Returns held units so staff drafts do not promise inventory already reserved elsewhere.
    pub const fn reserved(&self) -> ReservedUnits {
        self.reserved
    }

    /// Returns the low-stock threshold used to decide whether manager/vendor follow-up is due.
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
/// Inventory policy used by POS, recommendation, and reorder workflows to decide whether stock checks apply.
pub enum Policy {
    /// Treats inventory as untracked, allowing sale drafts while leaving stock verification to staff/POS workflow.
    NotTracked,
    /// Uses explicit on-hand and reorder-at counts to prevent oversells and surface low-stock work.
    Tracked {
        /// Physical units counted at the location before reservations or holds are subtracted.
        on_hand: UnitCount,
        /// Minimum available units that should prompt manager or vendor reorder attention.
        reorder_at: UnitCount,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Availability status used by recommendation policy to suppress unavailable products.
pub enum Availability {
    /// Product is available for sale drafts and recommendation candidates.
    Available,
    /// Product is unavailable, suppressing POS sale drafts and customer-facing recommendations.
    OutOfStock,
    /// Product is on backorder, so staff can see demand but automation must not promise fulfillment.
    Backordered,
    /// Inventory source did not provide a confident availability status, so staff should verify before promising stock.
    Unknown,
}
