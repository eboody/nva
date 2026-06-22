//! Reorder models for stock-threshold decisions and manager/vendor workflow creation.

use serde::{Deserialize, Serialize};

use crate::entities::LocationId;
use crate::policy;

use super::inventory::Position;
use super::product::Sku;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reorder policy deciding whether low stock creates a manager review, staff task, or vendor notice.
pub enum Policy {
    /// Low-stock findings stop for manager review instead of creating automatic vendor action.
    ManualReview,
    /// Low-stock findings create a staff/manager task for human ordering follow-up.
    AutoCreateManagerTask,
    /// Low-stock findings create a vendor-managed notice but do not place an order.
    VendorManaged,
}

impl Policy {
    /// Evaluates an inventory position against the policy and emits only threshold-backed reorder actions.
    pub fn evaluate(&self, position: &Position) -> Decision {
        if !position.is_at_or_below_reorder_threshold() {
            return Decision::NoAction;
        }
        match self {
            Self::ManualReview => Decision::ManagerReviewRequired {
                reason: Reason::AtOrBelowThreshold,
                gate: policy::ReviewGate::ManagerApproval,
            },
            Self::AutoCreateManagerTask => Decision::CreateStaffTask {
                location_id: position.location_id,
                sku: position.sku().clone(),
                reason: Reason::AtOrBelowThreshold,
                gate: policy::ReviewGate::ManagerApproval,
            },
            Self::VendorManaged => Decision::VendorManagedNotice {
                sku: position.sku().clone(),
                reason: Reason::AtOrBelowThreshold,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Reorder decision produced when available units are at or below threshold.
pub enum Decision {
    /// Available stock remains above threshold, so no staff or vendor follow-up is needed.
    NoAction,
    /// Creates a human task with location, SKU, reason, and manager approval gate for reorder follow-up.
    CreateStaffTask {
        /// Location where staff should inspect stock or complete the reorder task.
        location_id: LocationId,
        /// SKU that fell to or below the reorder threshold.
        sku: Sku,
        /// Reorder reason shown to staff or managers before purchasing or vendor follow-up.
        reason: Reason,
        /// Manager approval gate required before staff treat the reorder task as authorized.
        gate: policy::ReviewGate,
    },
    /// Low-stock finding requires manager approval before any purchasing or vendor action.
    ManagerReviewRequired {
        /// Reorder reason shown to staff or managers before purchasing or vendor follow-up.
        reason: Reason,
        /// Manager approval gate required before staff treat the reorder task as authorized.
        gate: policy::ReviewGate,
    },
    /// Vendor-managed item produces a notice for review rather than an automatic order.
    VendorManagedNotice {
        /// SKU that fell to or below the reorder threshold.
        sku: Sku,
        /// Reorder reason shown to staff or managers before purchasing or vendor follow-up.
        reason: Reason,
    },
}

impl Decision {
    /// Returns the human review gate that must clear before purchase, POS, or inventory action.
    pub fn review_gate(&self) -> Option<policy::ReviewGate> {
        match self {
            Self::NoAction => None,
            Self::CreateStaffTask { gate, .. } | Self::ManagerReviewRequired { gate, .. } => {
                Some(gate.clone())
            }
            Self::VendorManagedNotice { .. } => Some(policy::ReviewGate::ManagerApproval),
        }
    }

    /// Returns live side effects blocked by a reorder decision until staff/system-of-record action.
    pub const fn blocked_actions(&self) -> &'static [BlockedAction] {
        match self {
            Self::NoAction => &[],
            Self::CreateStaffTask { .. }
            | Self::ManagerReviewRequired { .. }
            | Self::VendorManagedNotice { .. } => REORDER_BLOCKED_ACTIONS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Live retail/POS/vendor side effects blocked by reorder review decisions.
pub enum BlockedAction {
    /// Do not place a vendor purchase order from threshold evidence alone.
    PlaceVendorPurchaseOrder,
    /// Do not change POS stock, inventory counts, reservations, or item saleability autonomously.
    MutateInventoryOrPos,
    /// Do not charge, discount, refund, or invoice from reorder evidence.
    MutatePaymentOrInvoice,
}

const REORDER_BLOCKED_ACTIONS: &[BlockedAction] = &[
    BlockedAction::PlaceVendorPurchaseOrder,
    BlockedAction::MutateInventoryOrPos,
    BlockedAction::MutatePaymentOrInvoice,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Business reason explaining why reorder action or vendor notice is needed.
pub enum Reason {
    /// Available stock is at or below the configured threshold for this SKU.
    AtOrBelowThreshold,
    /// Vendor reports the SKU is backordered, so staff can plan substitutes or waitlist notes.
    VendorBackorder,
    /// Upcoming boarding diet demand may deplete stock and should prompt manager review.
    ForecastedBoardingDietDepletion,
}
