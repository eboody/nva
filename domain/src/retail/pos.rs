use bon::Builder;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{entities, policy};

use super::product::LocationOffering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Typed quantity domain value that keeps raw primitives out of retail workflows.
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
/// Domain vocabulary for quantity error decisions in retail workflows.
pub enum QuantityError {
    #[error("retail sale quantity requires at least one unit")]
    /// Rejects zero where the pet-resort workflow requires a positive quantity.
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Groomer-assignment policies used when booking grooming work.
pub enum Policy {
    /// Standalone sale retail inventory, POS, reorder, or recommendation signal.
    StandaloneSale,
    /// Integrated with reservation checkout retail inventory, POS, reorder, or recommendation signal.
    IntegratedWithReservationCheckout,
    /// Manager only comp retail inventory, POS, reorder, or recommendation signal.
    ManagerOnlyComp,
}

impl Policy {
    /// Returns the evaluate for this retail value.
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
/// Typed request domain value that keeps raw primitives out of retail workflows.
pub struct Request {
    /// Offering fact promoted into this retail contract.
    pub offering: LocationOffering,
    /// Quantity fact promoted into this retail contract.
    pub quantity: Quantity,
    /// Source fact promoted into this retail contract.
    pub source: Source,
    /// Price adjustment fact promoted into this retail contract.
    pub price_adjustment: PriceAdjustment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for source decisions in retail workflows.
pub enum Source {
    /// Standalone staff sale retail inventory, POS, reorder, or recommendation signal.
    StandaloneStaffSale {
        /// Staff id fact promoted into this retail contract.
        staff_id: entities::StaffId,
    },
    /// Reservation checkout retail inventory, POS, reorder, or recommendation signal.
    ReservationCheckout {
        /// Reservation id fact promoted into this retail contract.
        reservation_id: entities::reservation::Id,
    },
    /// External pos reconciliation retail inventory, POS, reorder, or recommendation signal.
    ExternalPosReconciliation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for price adjustment decisions in retail workflows.
pub enum PriceAdjustment {
    /// No additional workflow gate is required.
    None,
    /// Business reason staff should review before proceeding.
    PolicyDiscount {
        /// Reason carried by this variant.
        reason: PriceExceptionReason,
    },
    /// Business reason staff should review before proceeding.
    ManagerComp {
        /// Reason carried by this variant.
        reason: PriceExceptionReason,
    },
    /// Business reason staff should review before proceeding.
    RefundOrReversal {
        /// Reason carried by this variant.
        reason: PriceExceptionReason,
    },
}

impl PriceAdjustment {
    /// Returns this retail value's requires manager approval.
    pub const fn requires_manager_approval(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for price exception reason decisions in retail workflows.
pub enum PriceExceptionReason {
    /// Complaint recovery retail inventory, POS, reorder, or recommendation signal.
    ComplaintRecovery,
    /// Staff courtesy retail inventory, POS, reorder, or recommendation signal.
    StaffCourtesy,
    /// Refund correction retail inventory, POS, reorder, or recommendation signal.
    RefundCorrection,
    /// Manager override retail inventory, POS, reorder, or recommendation signal.
    ManagerOverride,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for decision decisions in retail workflows.
pub enum Decision {
    /// Draft allowed retail inventory, POS, reorder, or recommendation signal.
    DraftAllowed,
    /// Review required retail inventory, POS, reorder, or recommendation signal.
    ReviewRequired {
        /// Business reason staff should review before proceeding.
        reason: ReviewReason,
        /// Gate fact promoted into this retail contract.
        gate: policy::ReviewGate,
    },
    /// Denied retail inventory, POS, reorder, or recommendation signal.
    Denied {
        /// Business reason staff should review before proceeding.
        reason: DenialReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for review reason decisions in retail workflows.
pub enum ReviewReason {
    /// Price exception retail inventory, POS, reorder, or recommendation signal.
    PriceException,
    /// Reservation checkout attachment retail inventory, POS, reorder, or recommendation signal.
    ReservationCheckoutAttachment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for denial reason decisions in retail workflows.
pub enum DenialReason {
    /// Offering not sellable retail inventory, POS, reorder, or recommendation signal.
    OfferingNotSellable,
    /// Inventory unavailable retail inventory, POS, reorder, or recommendation signal.
    InventoryUnavailable,
    /// Source not allowed retail inventory, POS, reorder, or recommendation signal.
    SourceNotAllowed,
}
