//! Reorder contracts for stock-threshold decisions and manager/vendor workflow creation.

use serde::{Deserialize, Serialize};

use crate::entities::LocationId;
use crate::policy;

use super::inventory::Position;
use super::product::Sku;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reorder policy deciding whether low stock creates a manager review, staff task, or vendor notice.
pub enum Policy {
    /// Manual review retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    ManualReview,
    /// Auto create manager task retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    AutoCreateManagerTask,
    /// Vendor managed retail operational signal for inventory, POS, reorder, recommendation, or review handling.
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
    /// No action retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    NoAction,
    /// Create staff task retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    CreateStaffTask {
        /// Source-derived location id carried by this retail contract.
        location_id: LocationId,
        /// Source-derived sku carried by this retail contract.
        sku: Sku,
        /// Business reason staff should review before proceeding.
        reason: Reason,
        /// Source-derived gate carried by this retail contract.
        gate: policy::ReviewGate,
    },
    /// Manager review required retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    ManagerReviewRequired {
        /// Business reason staff should review before proceeding.
        reason: Reason,
        /// Source-derived gate carried by this retail contract.
        gate: policy::ReviewGate,
    },
    /// Vendor managed notice retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    VendorManagedNotice {
        /// Source-derived sku carried by this retail contract.
        sku: Sku,
        /// Business reason staff should review before proceeding.
        reason: Reason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Business reason explaining why reorder action or vendor notice is needed.
pub enum Reason {
    /// At or below threshold retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    AtOrBelowThreshold,
    /// Vendor backorder retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    VendorBackorder,
    /// Forecasted boarding diet depletion retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    ForecastedBoardingDietDepletion,
}
