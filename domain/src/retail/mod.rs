use bon::Builder;
use serde::{Deserialize, Serialize};

/// Inventory boundary for retail contracts.
pub mod inventory;
/// Pos boundary for retail contracts.
pub mod pos;
/// Product boundary for retail contracts.
pub mod product;
/// Recommendation boundary for retail contracts.
pub mod recommendation;
/// Reorder boundary for retail contracts.
pub mod reorder;
/// Vendor boundary for retail contracts.
pub mod vendor;

pub use product::{LocationOffering, OfferingStatus, Product, Sku, SkuError};
pub use vendor::Partner;

/// Result type returned by fallible retail operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
/// Validation failures returned by retail domain constructors.
pub enum Error {
    #[error("retail inventory position cannot reserve more units than are on hand")]
    /// Reserved units exceed on hand retail inventory, POS, reorder, or recommendation signal.
    ReservedUnitsExceedOnHand,
    #[error("retail recommendation rationale is required")]
    /// Missing rationale retail inventory, POS, reorder, or recommendation signal.
    MissingRationale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Typed contract domain value that keeps raw primitives out of retail workflows.
pub struct Contract {
    /// Product fact promoted into this retail contract.
    pub product: Product,
    /// Pos fact promoted into this retail contract.
    pub pos: pos::Policy,
    /// Inventory fact promoted into this retail contract.
    pub inventory: inventory::Policy,
    /// Recommendation fact promoted into this retail contract.
    pub recommendation: recommendation::Rule,
    /// Reorder fact promoted into this retail contract.
    pub reorder: reorder::Policy,
}

impl Contract {
    /// Returns the should reorder for this retail value.
    pub fn should_reorder(&self) -> bool {
        matches!(self.inventory, inventory::Policy::Tracked { on_hand, reorder_at } if on_hand.get() <= reorder_at.get())
    }

    /// Returns the standard petsuites for this retail value.
    pub fn standard_petsuites() -> Self {
        Self::builder()
            .product(Product::new(
                Sku::try_new("PETSUITES-RETAIL").unwrap(),
                product::Category::PersonalizedUpsell,
            ))
            .pos(pos::Policy::IntegratedWithReservationCheckout)
            .inventory(inventory::Policy::Tracked {
                on_hand: inventory::UnitCount::try_new(1).unwrap(),
                reorder_at: inventory::UnitCount::try_new(10).unwrap(),
            })
            .recommendation(recommendation::Rule::AnxietySupportAfterBoarding)
            .reorder(reorder::Policy::AutoCreateManagerTask)
            .build()
    }
}
