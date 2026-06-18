use serde::{Deserialize, Serialize};

use crate::entities::LocationId;
use crate::policy;

use super::inventory::Position;
use super::product::Sku;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Groomer-assignment policies used when booking grooming work.
pub enum Policy {
    /// Manual review retail inventory, POS, reorder, or recommendation signal.
    ManualReview,
    /// Auto create manager task retail inventory, POS, reorder, or recommendation signal.
    AutoCreateManagerTask,
    /// Vendor managed retail inventory, POS, reorder, or recommendation signal.
    VendorManaged,
}

impl Policy {
    /// Returns the evaluate for this retail value.
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
/// Domain vocabulary for decision decisions in retail workflows.
pub enum Decision {
    /// No action retail inventory, POS, reorder, or recommendation signal.
    NoAction,
    /// Create staff task retail inventory, POS, reorder, or recommendation signal.
    CreateStaffTask {
        /// Location id fact promoted into this retail contract.
        location_id: LocationId,
        /// Sku fact promoted into this retail contract.
        sku: Sku,
        /// Business reason staff should review before proceeding.
        reason: Reason,
        /// Gate fact promoted into this retail contract.
        gate: policy::ReviewGate,
    },
    /// Manager review required retail inventory, POS, reorder, or recommendation signal.
    ManagerReviewRequired {
        /// Business reason staff should review before proceeding.
        reason: Reason,
        /// Gate fact promoted into this retail contract.
        gate: policy::ReviewGate,
    },
    /// Vendor managed notice retail inventory, POS, reorder, or recommendation signal.
    VendorManagedNotice {
        /// Sku fact promoted into this retail contract.
        sku: Sku,
        /// Business reason staff should review before proceeding.
        reason: Reason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for reason decisions in retail workflows.
pub enum Reason {
    /// At or below threshold retail inventory, POS, reorder, or recommendation signal.
    AtOrBelowThreshold,
    /// Vendor backorder retail inventory, POS, reorder, or recommendation signal.
    VendorBackorder,
    /// Forecasted boarding diet depletion retail inventory, POS, reorder, or recommendation signal.
    ForecastedBoardingDietDepletion,
}
