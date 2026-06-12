use bon::Builder;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{ReservationId, StaffId};
use crate::policy;

use super::product::LocationOffering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct SaleQuantity(u32);

impl SaleQuantity {
    pub const fn try_new(value: u32) -> std::result::Result<Self, SaleQuantityError> {
        if value == 0 {
            return Err(SaleQuantityError::Zero);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for SaleQuantity {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SaleQuantityError {
    #[error("retail sale quantity requires at least one unit")]
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointOfSalePolicy {
    StandaloneSale,
    IntegratedWithReservationCheckout,
    ManagerOnlyComp,
}

impl PointOfSalePolicy {
    pub fn evaluate(&self, request: &SaleRequest) -> SaleLineDecision {
        if !request.offering.can_be_sold_to_customer() {
            return SaleLineDecision::Denied {
                reason: SaleDenialReason::OfferingNotSellable,
            };
        }
        if !request.offering.has_available_sale_units(request.quantity) {
            return SaleLineDecision::Denied {
                reason: SaleDenialReason::InventoryUnavailable,
            };
        }
        if request.price_adjustment.requires_manager_approval()
            || matches!(self, Self::ManagerOnlyComp)
        {
            return SaleLineDecision::ReviewRequired {
                reason: SaleReviewReason::PriceException,
                gate: policy::ReviewGate::ManagerApproval,
            };
        }
        match (self, &request.source) {
            (Self::StandaloneSale, SaleSource::StandaloneStaffSale { .. }) => {
                SaleLineDecision::DraftAllowed
            }
            (Self::IntegratedWithReservationCheckout, SaleSource::ReservationCheckout { .. }) => {
                SaleLineDecision::ReviewRequired {
                    reason: SaleReviewReason::ReservationCheckoutAttachment,
                    gate: policy::ReviewGate::CustomerMessageApproval,
                }
            }
            _ => SaleLineDecision::Denied {
                reason: SaleDenialReason::SourceNotAllowed,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct SaleRequest {
    pub offering: LocationOffering,
    pub quantity: SaleQuantity,
    pub source: SaleSource,
    pub price_adjustment: PriceAdjustment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaleSource {
    StandaloneStaffSale { staff_id: StaffId },
    ReservationCheckout { reservation_id: ReservationId },
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
pub enum SaleLineDecision {
    DraftAllowed,
    ReviewRequired {
        reason: SaleReviewReason,
        gate: policy::ReviewGate,
    },
    Denied {
        reason: SaleDenialReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaleReviewReason {
    PriceException,
    ReservationCheckoutAttachment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaleDenialReason {
    OfferingNotSellable,
    InventoryUnavailable,
    SourceNotAllowed,
}
