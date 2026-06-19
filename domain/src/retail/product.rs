//! Product catalog models for SKUs, categories, location offerings, and sellability rules.

use bon::Builder;
use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::entities::LocationId;

use super::{inventory, pos, reorder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Product category used to distinguish supplements, boarding diets, and personalized upsell items.
pub enum Category {
    /// Supplement item that may need care or medical-document review before recommendation copy is shown.
    Supplement,
    /// Boarding or care diet stocked for in-house use and monitored for depletion.
    InHouseDiet,
    /// Customer-facing upsell candidate whose sale and copy still depend on inventory, preference, and review gates.
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
/// SKU validation errors that keep catalog, stock, and reorder records traceable.
pub enum SkuError {
    #[error("retail SKU cannot be empty")]
    /// Rejects blank catalog/POS SKU values so inventory and reorder facts remain traceable.
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
/// Retail product with a SKU and category used across POS, inventory, and recommendation decisions.
pub struct Product {
    sku: Sku,
    /// Product category used to route supplement, diet, and upsell handling.
    pub category: Category,
}

impl Product {
    /// Pairs a validated SKU with its retail category for catalog, inventory, and recommendation work.
    pub fn new(sku: Sku, category: Category) -> Self {
        Self { sku, category }
    }

    /// Returns the SKU that ties this product to catalog, inventory, POS, and vendor records.
    pub fn sku(&self) -> &Sku {
        &self.sku
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Location-level offering status used to prevent inactive or discontinued products from being sold.
pub enum OfferingStatus {
    /// Offering can be considered for sale or recommendation if usage and inventory also allow it.
    Active,
    /// Offering is disabled at the location and must not be sold or recommended.
    Inactive,
    /// Offering has been retired and should remain out of staff sale and recommendation drafts.
    Discontinued,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Usage policy distinguishing customer-sellable items from in-house consumables such as boarding diets.
pub enum Usage {
    /// Product may be sold to customers when status and inventory permit.
    CustomerSellable,
    /// Product is reserved for resort use, such as boarding diets, and should not become customer-sale copy.
    InHouseConsumable,
    /// Product can support both staff operations and customer sale drafts when other gates pass.
    SellableAndInHouseConsumable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Location-specific product offering with POS, inventory, and reorder policies attached.
pub struct LocationOffering {
    /// Location whose catalog, shelf policy, and inventory counts own this offering.
    pub location_id: LocationId,
    /// SKU and category being evaluated for sale, recommendation, inventory, and reorder work.
    pub product: Product,
    /// Location status that blocks inactive or discontinued items from customer-sale drafts.
    pub status: OfferingStatus,
    /// Usage policy deciding whether the product is customer-sellable, in-house only, or both.
    pub usage: Usage,
    /// POS policy controlling checkout sources and price-exception approval.
    pub pos: pos::Policy,
    /// Inventory policy controlling stock checks and low-stock attention.
    pub inventory: inventory::Policy,
    /// Reorder policy controlling manager tasks, vendor notices, or no-action outcomes.
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
