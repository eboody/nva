use serde::{Deserialize, Serialize};

use crate::entities::LocationId;
use crate::policy;

use super::inventory::Position;
use super::product::Sku;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Policy {
    ManualReview,
    AutoCreateManagerTask,
    VendorManaged,
}

impl Policy {
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
pub enum Decision {
    NoAction,
    CreateStaffTask {
        location_id: LocationId,
        sku: Sku,
        reason: Reason,
        gate: policy::ReviewGate,
    },
    ManagerReviewRequired {
        reason: Reason,
        gate: policy::ReviewGate,
    },
    VendorManagedNotice {
        sku: Sku,
        reason: Reason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reason {
    AtOrBelowThreshold,
    VendorBackorder,
    ForecastedBoardingDietDepletion,
}
