//! Retail service-line models for POS sale eligibility, inventory position, reorder workflows, vendor partnerships, and care-safe product recommendations.
//!
//! Operator summary: this module supports staff queues for retail sale drafts, checkout attachments, customer-safe recommendation drafts, low-stock reorder work, and vendor-managed notices. It reduces front-desk and manager labor by turning SKU/catalog facts, stock counts, customer preference, care sensitivity, checkout source, price exceptions, and reorder thresholds into typed decisions instead of ad hoc manual review.
//!
//! It is not a live automation layer. `domain::retail` does not send customer messages, promise medical outcomes, place vendor orders, mutate POS/Gingr transactions, reconcile payments, approve comps/refunds, or attach products to reservations. Provider DTOs/endpoints, storage records/codes, and source/provenance facts remain authoritative in their own layers; this module only evaluates promoted domain facts.
//!
//! Review gates protect pets, customers, and staff: unavailable or non-sellable items are denied, opted-out customers and unavailable products suppress recommendations, supplement/diet and care-plan conflicts require staff or manager review, medical-claim customer copy is rejected or approval-gated, reservation-checkout attachments require customer-message approval, price exceptions require manager approval, impossible stock math is rejected, and reorder actions become threshold-backed manager tasks or vendor notices rather than automatic purchases.

use bon::Builder;
use serde::{Deserialize, Serialize};

/// Inventory models stock position, available units, reorder thresholds, and oversell checks for retail staff workflows.
pub mod inventory;
/// POS models standalone sales, reservation-checkout attachments, price exceptions, and comps so checkout writes stay approval-gated.
pub mod pos;
/// Product catalog models SKUs, location offerings, sellability, and in-house consumable use for staff-facing retail decisions.
pub mod product;
/// Recommendation models personalized upsells with inventory, preference, and care-safety gates before any customer copy is approved.
pub mod recommendation;
/// Reorder models manager tasks, vendor-managed notices, and no-action outcomes from location stock thresholds.
pub mod reorder;
/// Vendor models partner-product catalog relationships and externally managed assortments without placing orders.
pub mod vendor;

pub use product::{LocationOffering, OfferingStatus, Product, Sku, SkuError};
pub use vendor::Partner;

/// Result type returned by fallible retail operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
/// Retail validation failures that prevent impossible stock math or unsupported recommendations from becoming workflow facts.
pub enum Error {
    #[error("retail inventory position cannot reserve more units than are on hand")]
    /// Blocks impossible stock math before POS drafts, recommendations, or reorder tasks use the inventory count.
    ReservedUnitsExceedOnHand,
    #[error("retail recommendation rationale is required")]
    /// Blocks recommendation candidates that lack a staff-readable reason for the suggested product.
    MissingRationale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Location retail policy bundle tying catalog product, POS policy, inventory policy, recommendation rule, and reorder policy together.
pub struct Contract {
    /// Product SKU/category facts used by sale, recommendation, inventory, and reorder decisions.
    pub product: Product,
    /// POS policy that decides which sale sources are allowed and which price actions require manager approval.
    pub pos: pos::Policy,
    /// Inventory policy that tells staff whether stock is tracked and where reorder attention begins.
    pub inventory: inventory::Policy,
    /// Recommendation rule that can create an internal upsell candidate only after safety gates pass.
    pub recommendation: recommendation::Rule,
    /// Reorder policy that routes threshold findings to manager review, staff tasks, or vendor notices.
    pub reorder: reorder::Policy,
}

impl Contract {
    /// Reports whether the contracted inventory threshold indicates manager/vendor reorder attention is due.
    pub fn should_reorder(&self) -> bool {
        matches!(self.inventory, inventory::Policy::Tracked { on_hand, reorder_at } if on_hand.get() <= reorder_at.get())
    }

    /// Builds a representative PetSuites-style retail policy bundle for docs/tests without claiming it is live policy.
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
