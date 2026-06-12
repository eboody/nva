use bon::Builder;
use serde::{Deserialize, Serialize};

pub mod inventory;
pub mod pos;
pub mod product;
pub mod recommendation;
pub mod reorder;
pub mod vendor;

pub use product::{LocationOffering, OfferingStatus, Product, Sku, SkuError};
pub use vendor::Partner;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("retail inventory position cannot reserve more units than are on hand")]
    ReservedUnitsExceedOnHand,
    #[error("retail recommendation rationale is required")]
    MissingRationale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Contract {
    pub product: Product,
    pub pos: pos::Policy,
    pub inventory: inventory::Policy,
    pub recommendation: recommendation::Rule,
    pub reorder: reorder::Policy,
}

impl Contract {
    pub fn should_reorder(&self) -> bool {
        matches!(self.inventory, inventory::Policy::Tracked { on_hand, reorder_at } if on_hand.get() <= reorder_at.get())
    }

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
