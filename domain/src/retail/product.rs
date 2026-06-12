use bon::Builder;
use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::entities::LocationId;

use super::inventory::InventoryPolicy;
use super::pos::{PointOfSalePolicy, SaleQuantity};
use super::reorder::Policy as ReorderPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Supplement,
    InHouseDiet,
    PersonalizedUpsell,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Sku(String);

impl Sku {
    pub fn try_new(value: impl Into<String>) -> std::result::Result<Self, SkuError> {
        let value = value.into().trim().to_owned();
        if value.is_empty() {
            return Err(SkuError::Empty);
        }
        Ok(Self(value))
    }

    pub fn into_inner(self) -> String {
        self.0
    }

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
pub enum SkuError {
    #[error("retail SKU cannot be empty")]
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
pub struct ProductName(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Product {
    sku: Sku,
    pub category: Category,
}

impl Product {
    pub fn new(sku: Sku, category: Category) -> Self {
        Self { sku, category }
    }

    pub fn sku(&self) -> &Sku {
        &self.sku
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfferingStatus {
    Active,
    Inactive,
    Discontinued,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductUsage {
    CustomerSellable,
    InHouseConsumable,
    SellableAndInHouseConsumable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct LocationOffering {
    pub location_id: LocationId,
    pub product: Product,
    pub status: OfferingStatus,
    pub usage: ProductUsage,
    pub pos: PointOfSalePolicy,
    pub inventory: InventoryPolicy,
    pub reorder: ReorderPolicy,
}

impl LocationOffering {
    pub fn can_be_sold_to_customer(&self) -> bool {
        matches!(self.status, OfferingStatus::Active)
            && matches!(
                self.usage,
                ProductUsage::CustomerSellable | ProductUsage::SellableAndInHouseConsumable
            )
    }

    pub fn has_available_sale_units(&self, quantity: SaleQuantity) -> bool {
        match self.inventory {
            InventoryPolicy::NotTracked => true,
            InventoryPolicy::Tracked { on_hand, .. } => on_hand.get() >= quantity.get(),
        }
    }
}
