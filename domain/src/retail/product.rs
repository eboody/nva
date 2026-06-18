use bon::Builder;
use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::entities::LocationId;

use super::{inventory, pos, reorder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for category decisions in retail workflows.
pub enum Category {
    /// Supplement retail inventory, POS, reorder, or recommendation signal.
    Supplement,
    /// In house diet retail inventory, POS, reorder, or recommendation signal.
    InHouseDiet,
    /// Personalized upsell retail inventory, POS, reorder, or recommendation signal.
    PersonalizedUpsell,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Typed sku domain value that keeps raw primitives out of retail workflows.
pub struct Sku(String);

impl Sku {
    /// Validates and creates the retail value.
    pub fn try_new(value: impl Into<String>) -> std::result::Result<Self, SkuError> {
        let value = value.into().trim().to_owned();
        if value.is_empty() {
            return Err(SkuError::Empty);
        }
        Ok(Self(value))
    }

    /// Returns the owned inner string for storage or outbound mapping.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Returns the provider or domain identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Sku {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::try_new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Domain vocabulary for sku error decisions in retail workflows.
pub enum SkuError {
    #[error("retail SKU cannot be empty")]
    /// Empty retail inventory, POS, reorder, or recommendation signal.
    Empty,
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed product domain value that keeps raw primitives out of retail workflows.
pub struct Product {
    sku: Sku,
    /// Category fact promoted into this retail contract.
    pub category: Category,
}

impl Product {
    /// Assembles this retail value from already-validated domain parts.
    pub fn new(sku: Sku, category: Category) -> Self {
        Self { sku, category }
    }

    /// Returns the sku for this retail value.
    pub fn sku(&self) -> &Sku {
        &self.sku
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for offering status decisions in retail workflows.
pub enum OfferingStatus {
    /// Active retail inventory, POS, reorder, or recommendation signal.
    Active,
    /// Inactive retail inventory, POS, reorder, or recommendation signal.
    Inactive,
    /// Discontinued retail inventory, POS, reorder, or recommendation signal.
    Discontinued,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for usage decisions in retail workflows.
pub enum Usage {
    /// Customer sellable retail inventory, POS, reorder, or recommendation signal.
    CustomerSellable,
    /// In house consumable retail inventory, POS, reorder, or recommendation signal.
    InHouseConsumable,
    /// Sellable and in house consumable retail inventory, POS, reorder, or recommendation signal.
    SellableAndInHouseConsumable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Typed location offering domain value that keeps raw primitives out of retail workflows.
pub struct LocationOffering {
    /// Location id fact promoted into this retail contract.
    pub location_id: LocationId,
    /// Product fact promoted into this retail contract.
    pub product: Product,
    /// Status fact promoted into this retail contract.
    pub status: OfferingStatus,
    /// Usage fact promoted into this retail contract.
    pub usage: Usage,
    /// Pos fact promoted into this retail contract.
    pub pos: pos::Policy,
    /// Inventory fact promoted into this retail contract.
    pub inventory: inventory::Policy,
    /// Reorder fact promoted into this retail contract.
    pub reorder: reorder::Policy,
}

impl LocationOffering {
    /// Returns the can be sold to customer for this retail value.
    pub fn can_be_sold_to_customer(&self) -> bool {
        matches!(self.status, OfferingStatus::Active)
            && matches!(
                self.usage,
                Usage::CustomerSellable | Usage::SellableAndInHouseConsumable
            )
    }

    /// Returns the has available sale units for this retail value.
    pub fn has_available_sale_units(&self, quantity: pos::Quantity) -> bool {
        match self.inventory {
            inventory::Policy::NotTracked => true,
            inventory::Policy::Tracked { on_hand, .. } => on_hand.get() >= quantity.get(),
        }
    }
}
