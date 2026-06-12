use bon::Builder;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{entities, policy};

use super::product::LocationOffering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Quantity(u32);

impl Quantity {
    pub const fn try_new(value: u32) -> std::result::Result<Self, QuantityError> {
        if value == 0 {
            return Err(QuantityError::Zero);
        }
        Ok(Self(value))
    }

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
pub enum QuantityError {
    #[error("retail sale quantity requires at least one unit")]
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Policy {
    StandaloneSale,
    IntegratedWithReservationCheckout,
    ManagerOnlyComp,
}

impl Policy {
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
pub struct Request {
    pub offering: LocationOffering,
    pub quantity: Quantity,
    pub source: Source,
    pub price_adjustment: PriceAdjustment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Source {
    StandaloneStaffSale {
        staff_id: entities::StaffId,
    },
    ReservationCheckout {
        reservation_id: entities::ReservationId,
    },
    ExternalPosReconciliation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriceAdjustment {
    None,
    PolicyDiscount { reason: PriceExceptionReason },
    ManagerComp { reason: PriceExceptionReason },
    RefundOrReversal { reason: PriceExceptionReason },
}

impl PriceAdjustment {
    pub const fn requires_manager_approval(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriceExceptionReason {
    ComplaintRecovery,
    StaffCourtesy,
    RefundCorrection,
    ManagerOverride,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    DraftAllowed,
    ReviewRequired {
        reason: ReviewReason,
        gate: policy::ReviewGate,
    },
    Denied {
        reason: DenialReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewReason {
    PriceException,
    ReservationCheckoutAttachment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DenialReason {
    OfferingNotSellable,
    InventoryUnavailable,
    SourceNotAllowed,
}
