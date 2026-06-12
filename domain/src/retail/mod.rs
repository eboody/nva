use bon::Builder;
use serde::{Deserialize, Serialize};

pub mod inventory;
pub mod pos;
pub mod product;
pub mod recommendation;
pub mod reorder;
pub mod vendor;

pub use inventory::{
    AvailableUnits, InventoryAvailability, InventoryPolicy, InventoryPosition, OnHandUnits,
    ReservedUnits, StockPosition, UnitCount, UnitCountError,
};
pub use pos::{
    PointOfSalePolicy, PriceAdjustment, PriceExceptionReason, SaleDenialReason, SaleLineDecision,
    SaleQuantity, SaleQuantityError, SaleRequest, SaleReviewReason, SaleSource,
};
pub use product::{
    Category as ProductCategory, LocationOffering, OfferingStatus, Product, ProductName,
    ProductUsage, Sku, SkuError,
};
pub use recommendation::{
    Candidate as RecommendationCandidate, CareSensitivity, CustomerCopyDecision,
    CustomerCopyPolicy, CustomerCopyRejectionReason, CustomerRetailPreference, CustomerSafeCopy,
    Decision as RecommendationDecision, Policy as RecommendationPolicy,
    Reason as RecommendationReason, RecommendationRationale,
    ReviewReason as RecommendationReviewReason, Rule as RecommendationRule,
    SuppressionReason as RecommendationSuppressionReason,
};
pub use reorder::{Decision as ReorderDecision, Policy as ReorderPolicy, Reason as ReorderReason};
pub use vendor::Partner;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("retail inventory position cannot reserve more units than are on hand")]
    ReservedUnitsExceedOnHand,
    #[error("retail recommendation rationale is required")]
    MissingRecommendationRationale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Contract {
    pub product: Product,
    pub pos: PointOfSalePolicy,
    pub inventory: InventoryPolicy,
    pub recommendation: RecommendationRule,
    pub reorder: ReorderPolicy,
}

impl Contract {
    pub fn should_reorder(&self) -> bool {
        matches!(self.inventory, InventoryPolicy::Tracked { on_hand, reorder_at } if on_hand.get() <= reorder_at.get())
    }

    pub fn standard_petsuites() -> Self {
        Self::builder()
            .product(Product::new(
                Sku::try_new("PETSUITES-RETAIL").unwrap(),
                ProductCategory::PersonalizedUpsell,
            ))
            .pos(PointOfSalePolicy::IntegratedWithReservationCheckout)
            .inventory(InventoryPolicy::Tracked {
                on_hand: UnitCount::try_new(1).unwrap(),
                reorder_at: UnitCount::try_new(10).unwrap(),
            })
            .recommendation(RecommendationRule::AnxietySupportAfterBoarding)
            .reorder(ReorderPolicy::AutoCreateManagerTask)
            .build()
    }
}
