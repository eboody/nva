//! POS models for attaching retail sales to staff transactions or reservation checkout while preserving approval gates.

use bon::Builder;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{entities, policy};

use super::product::LocationOffering;

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

    /// Returns the quantity for checkout mapping, inventory checks, and audit records.
    pub const fn get(self) -> u32 {
        self.0
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

impl Policy {
    /// Evaluates sale eligibility from offering status, inventory, source, and price-exception policy.
    pub fn evaluate(&self, request: &Request) -> Decision {
        if !request.offering.can_be_sold_to_customer() {
            return Decision::Denied {
                reason: DenialReason::OfferingNotSellable,
            };
        }
        if !request.offering.has_available_sale_units(request.quantity) {
            return Decision::Denied {
                reason: DenialReason::InventoryUnavailable,
            };
        }
        if request.price_adjustment.requires_manager_approval()
            || matches!(self, Self::ManagerOnlyComp)
        {
            return Decision::ReviewRequired {
                reason: ReviewReason::PriceException,
                gate: policy::ReviewGate::ManagerApproval,
            };
        }
        match (self, &request.source) {
            (Self::StandaloneSale, Source::StandaloneStaffSale { .. }) => Decision::DraftAllowed,
            (Self::IntegratedWithReservationCheckout, Source::ReservationCheckout { .. }) => {
                Decision::ReviewRequired {
                    reason: ReviewReason::ReservationCheckoutAttachment,
                    gate: policy::ReviewGate::CustomerMessageApproval,
                }
            }
            _ => Decision::Denied {
                reason: DenialReason::SourceNotAllowed,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Retail sale request combining offering, quantity, source, and price-adjustment context.
pub struct Request {
    /// Location offering being checked for sellability, usage policy, and available units.
    pub offering: LocationOffering,
    /// Positive unit count staff want to sell or attach to checkout.
    pub quantity: Quantity,
    /// Sale origin used to block unsupported POS or reservation mutations.
    pub source: Source,
    /// Discount, comp, refund, or reversal context that may require manager approval.
    pub price_adjustment: PriceAdjustment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Source of the retail sale attempt, used to prevent unsupported POS or reservation mutations.
pub enum Source {
    /// Staff-originated counter sale that may draft when policy and inventory allow it.
    StandaloneStaffSale {
        /// Staff member accountable for the standalone sale draft.
        staff_id: entities::StaffId,
    },
    /// Reservation checkout context that can propose an attachment but cannot send customer copy without approval.
    ReservationCheckout {
        /// Reservation receiving a proposed retail attachment after customer-message approval.
        reservation_id: entities::reservation::Id,
    },
    /// Imported POS reconciliation source that is recorded for review instead of mutating checkout from domain code.
    ExternalPosReconciliation,
}

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

impl PriceAdjustment {
    /// Reports whether this price action must stop for manager approval before checkout changes.
    pub const fn requires_manager_approval(self) -> bool {
        !matches!(self, Self::None)
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// POS decision describing whether a sale draft is allowed, needs review, or is denied.
pub enum Decision {
    /// Sale may be drafted internally because product, inventory, source, and price policy all passed.
    DraftAllowed,
    /// Sale must pause for the named approval gate before POS, reservation, payment, refund, or discount action.
    ReviewRequired {
        /// Approval reason shown to staff or managers before checkout work proceeds.
        reason: ReviewReason,
        /// Approval gate that must be satisfied before the retail workflow can proceed.
        gate: policy::ReviewGate,
    },
    /// Sale is blocked before checkout because product, inventory, or source policy failed.
    Denied {
        /// Denial reason explaining why checkout work must not proceed.
        reason: DenialReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reason a POS draft must be reviewed before checkout action.
pub enum ReviewReason {
    /// Price adjustment requires manager approval before any discount, comp, refund, or reversal.
    PriceException,
    /// Reservation attachment requires customer-message approval before staff can proceed.
    ReservationCheckoutAttachment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reason a retail sale is denied before reaching checkout.
pub enum DenialReason {
    /// Product is inactive, discontinued, or not customer-sellable at this location.
    OfferingNotSellable,
    /// Available units cannot satisfy the requested quantity, so checkout must not promise the item.
    InventoryUnavailable,
    /// Sale origin is not allowed by this POS policy, preventing unsupported POS or reservation writes.
    SourceNotAllowed,
}
