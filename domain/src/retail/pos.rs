//! POS models for attaching retail sales to staff transactions or reservation checkout while preserving approval gates.

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Positive sale quantity used to ensure retail checkout never drafts zero-unit line items.
pub struct Quantity(u32);

impl Quantity {
    /// Accepts a positive sale quantity so POS drafts never create zero-unit retail line items.
    pub const fn try_new(value: u32) -> std::result::Result<Self, QuantityError> {
        if value == 0 {
            return Err(QuantityError::Zero);
        }
        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for Quantity {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Quantity validation errors that prevent unusable POS sale drafts.
pub enum QuantityError {
    #[error("retail sale quantity requires at least one unit")]
    /// Rejects zero where the pet-resort workflow requires a positive quantity.
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// POS policy deciding which retail sources are allowed and which price actions need manager review.
pub enum Policy {
    /// Allows staff to draft an in-person retail sale that remains separate from reservation checkout.
    StandaloneSale,
    /// Allows a sale to be proposed during reservation checkout but still requires customer-message approval.
    IntegratedWithReservationCheckout,
    /// Forces manager approval before any comp, refund, or discount reaches POS workflow.
    ManagerOnlyComp,
}

impl Policy {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Price adjustment or comp that triggers manager review before checkout mutation.
pub enum PriceAdjustment {
    /// No discount, comp, refund, or reversal is requested.
    None,
    /// Approval reason shown to staff or managers before checkout work proceeds.
    PolicyDiscount {
        /// Manager-readable reason for the discount, comp, refund, or reversal request.
        reason: PriceExceptionReason,
    },
    /// Approval reason shown to staff or managers before checkout work proceeds.
    ManagerComp {
        /// Manager-readable reason for the discount, comp, refund, or reversal request.
        reason: PriceExceptionReason,
    },
    /// Approval reason shown to staff or managers before checkout work proceeds.
    RefundOrReversal {
        /// Manager-readable reason for the discount, comp, refund, or reversal request.
        reason: PriceExceptionReason,
    },
}

impl PriceAdjustment {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Price-exception reasons that explain why a manager must approve the checkout change.
pub enum PriceExceptionReason {
    /// Discount or comp requested to recover from a customer complaint, requiring manager review.
    ComplaintRecovery,
    /// Courtesy adjustment requested by staff, requiring manager review before POS action.
    StaffCourtesy,
    /// Refund or reversal correction that must be approved before money movement.
    RefundCorrection,
    /// Manager override reason documenting why an exception may proceed after approval.
    ManagerOverride,
}
