//! POS contracts for attaching retail sales to staff transactions or reservation checkout while preserving approval gates.

use bon::Builder;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{entities, policy};

use super::product::LocationOffering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Positive sale quantity used to ensure retail checkout never drafts zero-unit line items.
pub struct Quantity(u32);

impl Quantity {
    /// Promotes boundary input into a validated retail domain value.
    pub const fn try_new(value: u32) -> std::result::Result<Self, QuantityError> {
        if value == 0 {
            return Err(QuantityError::Zero);
        }
        Ok(Self(value))
    }

    /// Exposes the validated scalar for serialization and adapter boundaries.
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
/// Decision vocabulary for quantity error in retail workflows.
pub enum QuantityError {
    #[error("retail sale quantity requires at least one unit")]
    /// Rejects zero where the pet-resort workflow requires a positive quantity.
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// POS policy deciding which retail sources are allowed and which price actions need manager review.
pub enum Policy {
    /// Standalone sale retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    StandaloneSale,
    /// Integrated with reservation checkout retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    IntegratedWithReservationCheckout,
    /// Manager only comp retail operational signal for inventory, POS, reorder, recommendation, or review handling.
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
    /// Source-derived offering carried by this retail contract.
    pub offering: LocationOffering,
    /// Source-derived quantity carried by this retail contract.
    pub quantity: Quantity,
    /// Source-derived source carried by this retail contract.
    pub source: Source,
    /// Source-derived price adjustment carried by this retail contract.
    pub price_adjustment: PriceAdjustment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Source of the retail sale attempt, used to prevent unsupported POS or reservation mutations.
pub enum Source {
    /// Standalone staff sale retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    StandaloneStaffSale {
        /// Source-derived staff id carried by this retail contract.
        staff_id: entities::StaffId,
    },
    /// Reservation checkout retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    ReservationCheckout {
        /// Source-derived reservation id carried by this retail contract.
        reservation_id: entities::reservation::Id,
    },
    /// External pos reconciliation retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    ExternalPosReconciliation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Price adjustment or comp that triggers manager review before checkout mutation.
pub enum PriceAdjustment {
    /// No additional workflow gate is required.
    None,
    /// Business reason staff should review before proceeding.
    PolicyDiscount {
        /// Reason value carried by this review or workflow variant.
        reason: PriceExceptionReason,
    },
    /// Business reason staff should review before proceeding.
    ManagerComp {
        /// Reason value carried by this review or workflow variant.
        reason: PriceExceptionReason,
    },
    /// Business reason staff should review before proceeding.
    RefundOrReversal {
        /// Reason value carried by this review or workflow variant.
        reason: PriceExceptionReason,
    },
}

impl PriceAdjustment {
    /// Returns the requires manager approval evidence recorded on this retail contract.
    pub const fn requires_manager_approval(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Decision vocabulary for price exception reason in retail workflows.
pub enum PriceExceptionReason {
    /// Complaint recovery retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    ComplaintRecovery,
    /// Staff courtesy retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    StaffCourtesy,
    /// Refund correction retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    RefundCorrection,
    /// Manager override retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    ManagerOverride,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// POS decision describing whether a sale draft is allowed, needs review, or is denied.
pub enum Decision {
    /// Draft allowed retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    DraftAllowed,
    /// Review required retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    ReviewRequired {
        /// Business reason staff should review before proceeding.
        reason: ReviewReason,
        /// Source-derived gate carried by this retail contract.
        gate: policy::ReviewGate,
    },
    /// Denied retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    Denied {
        /// Business reason staff should review before proceeding.
        reason: DenialReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reason a POS draft must be reviewed before checkout action.
pub enum ReviewReason {
    /// Price exception retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    PriceException,
    /// Reservation checkout attachment retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    ReservationCheckoutAttachment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reason a retail sale is denied before reaching checkout.
pub enum DenialReason {
    /// Offering not sellable retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    OfferingNotSellable,
    /// Inventory unavailable retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    InventoryUnavailable,
    /// Source not allowed retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    SourceNotAllowed,
}
