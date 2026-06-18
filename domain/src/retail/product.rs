//! Product catalog contracts for SKUs, categories, location offerings, and sellability rules.

use bon::Builder;
use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::entities::LocationId;

use super::{inventory, pos, reorder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Product category used to distinguish supplements, boarding diets, and personalized upsell items.
pub enum Category {
    /// Supplement retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    Supplement,
    /// In house diet retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    InHouseDiet,
    /// Personalized upsell retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    PersonalizedUpsell,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Non-empty SKU identifier promoted from POS/catalog data for inventory and reorder workflows.
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
/// Decision vocabulary for sku error in retail workflows.
pub enum SkuError {
    #[error("retail SKU cannot be empty")]
    /// Empty retail operational signal for inventory, POS, reorder, recommendation, or review handling.
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
/// Retail product with a SKU and category used across POS, inventory, and recommendation contracts.
pub struct Product {
    sku: Sku,
    /// Source-derived category carried by this retail contract.
    pub category: Category,
}

impl Product {
    /// Assembles this retail value from already-validated domain parts.
    pub fn new(sku: Sku, category: Category) -> Self {
        Self { sku, category }
    }

    /// Returns the sku evidence recorded on this retail contract.
    pub fn sku(&self) -> &Sku {
        &self.sku
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Location-level offering status used to prevent inactive or discontinued products from being sold.
pub enum OfferingStatus {
    /// Active retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    Active,
    /// Inactive retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    Inactive,
    /// Discontinued retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    Discontinued,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Usage policy distinguishing customer-sellable items from in-house consumables such as boarding diets.
pub enum Usage {
    /// Customer sellable retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    CustomerSellable,
    /// In house consumable retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    InHouseConsumable,
    /// Sellable and in house consumable retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    SellableAndInHouseConsumable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Location-specific product offering with POS, inventory, and reorder policies attached.
pub struct LocationOffering {
    /// Source-derived location id carried by this retail contract.
    pub location_id: LocationId,
    /// Source-derived product carried by this retail contract.
    pub product: Product,
    /// Source-derived status carried by this retail contract.
    pub status: OfferingStatus,
    /// Source-derived usage carried by this retail contract.
    pub usage: Usage,
    /// Source-derived pos carried by this retail contract.
    pub pos: pos::Policy,
    /// Source-derived inventory carried by this retail contract.
    pub inventory: inventory::Policy,
    /// Source-derived reorder carried by this retail contract.
    pub reorder: reorder::Policy,
}

impl LocationOffering {
    /// Reports whether the product is active and customer-sellable at this location.
    pub fn can_be_sold_to_customer(&self) -> bool {
        matches!(self.status, OfferingStatus::Active)
            && matches!(
                self.usage,
                Usage::CustomerSellable | Usage::SellableAndInHouseConsumable
            )
    }

    /// Checks tracked inventory before allowing a POS sale draft for the requested quantity.
    pub fn has_available_sale_units(&self, quantity: pos::Quantity) -> bool {
        match self.inventory {
            inventory::Policy::NotTracked => true,
            inventory::Policy::Tracked { on_hand, .. } => on_hand.get() >= quantity.get(),
        }
    }
}
