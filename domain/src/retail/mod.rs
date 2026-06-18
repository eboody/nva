//! Retail service-line contracts for POS sale eligibility, inventory position, reorder workflows, vendor partnerships, and care-safe product recommendations.
//!
//! Retail products such as supplements, boarding diets, and coat-care items can lift revenue and guest experience, but customer-facing copy and product recommendations must remain evidence-backed. This module keeps SKU/catalog facts, stock thresholds, checkout attachment, reorder tasks, and safe upsell recommendations behind review gates so automation drafts opportunities without promising medical outcomes or bypassing inventory/POS policy.

use bon::Builder;
use serde::{Deserialize, Serialize};

/// Inventory boundary for stock position, availability, reorder thresholds, and sellable-unit checks.
pub mod inventory;
/// POS boundary for standalone sales, reservation-checkout attachments, price exceptions, and comps.
pub mod pos;
/// Product catalog boundary for SKUs, location offerings, sellability, and in-house consumable use.
pub mod product;
/// Recommendation boundary for personalized retail upsells with inventory, preference, and care-safety gates.
pub mod recommendation;
/// Reorder boundary for manager tasks, vendor-managed notices, and no-action threshold decisions.
pub mod reorder;
/// Vendor boundary for partner-product catalog relationships and externally managed assortments.
pub mod vendor;

pub use product::{LocationOffering, OfferingStatus, Product, Sku, SkuError};
pub use vendor::Partner;

/// Result type returned by fallible retail operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
/// Retail validation failures that prevent impossible stock math or unsupported recommendations from becoming workflow facts.
pub enum Error {
    #[error("retail inventory position cannot reserve more units than are on hand")]
    /// Reserved units exceed on hand retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    ReservedUnitsExceedOnHand,
    #[error("retail recommendation rationale is required")]
    /// Missing rationale retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    MissingRationale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Location retail contract tying catalog product, POS policy, inventory policy, recommendation rule, and reorder policy together.
pub struct Contract {
    /// Source-derived product carried by this retail contract.
    pub product: Product,
    /// Source-derived pos carried by this retail contract.
    pub pos: pos::Policy,
    /// Source-derived inventory carried by this retail contract.
    pub inventory: inventory::Policy,
    /// Source-derived recommendation carried by this retail contract.
    pub recommendation: recommendation::Rule,
    /// Source-derived reorder carried by this retail contract.
    pub reorder: reorder::Policy,
}

impl Contract {
    /// Reports whether the contracted inventory threshold indicates manager/vendor reorder attention is due.
    pub fn should_reorder(&self) -> bool {
        matches!(self.inventory, inventory::Policy::Tracked { on_hand, reorder_at } if on_hand.get() <= reorder_at.get())
    }

    /// Builds a representative PetSuites-style retail contract for docs/tests without claiming it is live policy.
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
