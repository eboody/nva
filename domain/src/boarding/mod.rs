use bon::Builder;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{LocationId, PetId, ReservationId};
use crate::money;

macro_rules! positive_scalar {
    ($name:ident, $primitive:ty, $error:ident, $message:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        pub struct $name($primitive);

        impl $name {
            pub const fn try_new(value: $primitive) -> std::result::Result<Self, $error> {
                if value == 0 {
                    return Err($error::Zero);
                }
                Ok(Self(value))
            }

            pub const fn get(self) -> $primitive {
                self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Self::try_new(<$primitive>::deserialize(deserializer)?)
                    .map_err(serde::de::Error::custom)
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
        pub enum $error {
            #[error($message)]
            Zero,
        }
    };
}

positive_scalar!(
    RoomInventory,
    u16,
    RoomInventoryError,
    "boarding room inventory requires at least one room"
);
positive_scalar!(
    StayNights,
    u16,
    StayNightsError,
    "boarding minimum stay requires at least one night"
);
positive_scalar!(
    NoticeHours,
    u16,
    NoticeHoursError,
    "boarding cancellation notice requires at least one hour"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct HourOfDay(u8);

impl HourOfDay {
    pub const fn try_new(value: u8) -> std::result::Result<Self, HourOfDayError> {
        if value > 23 {
            return Err(HourOfDayError::OutsideClockDay);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl<'de> Deserialize<'de> for HourOfDay {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum HourOfDayError {
    #[error("boarding service-window hour must be between 0 and 23")]
    OutsideClockDay,
}

pub mod accommodation;

pub mod capacity;

pub mod deposit;

pub mod care;

pub mod upsell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomAvailability {
    Open,
    Limited,
    WaitlistOnly,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacityPlan {
    room_inventory: RoomInventory,
    pub availability: RoomAvailability,
}

impl CapacityPlan {
    pub const fn new(room_inventory: RoomInventory, availability: RoomAvailability) -> Self {
        Self {
            room_inventory,
            availability,
        }
    }
    pub const fn room_inventory(&self) -> RoomInventory {
        self.room_inventory
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceWindow {
    start: HourOfDay,
    end: HourOfDay,
}

impl ServiceWindow {
    pub const fn new(
        start: HourOfDay,
        end: HourOfDay,
    ) -> std::result::Result<Self, ServiceWindowError> {
        if start.get() >= end.get() {
            return Err(ServiceWindowError::EndMustFollowStart);
        }
        Ok(Self { start, end })
    }
    pub const fn start(&self) -> HourOfDay {
        self.start
    }
    pub const fn end(&self) -> HourOfDay {
        self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ServiceWindowError {
    #[error("boarding service window end must follow start")]
    EndMustFollowStart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinimumStay {
    nights: StayNights,
    pub reason: MinimumStayReason,
}

impl MinimumStay {
    pub const fn new(nights: StayNights, reason: MinimumStayReason) -> Self {
        Self { nights, reason }
    }
    pub const fn nights(&self) -> StayNights {
        self.nights
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinimumStayReason {
    StandardPolicy,
    HolidayPeak,
    MultiPetOperationalBuffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancellationPolicy {
    pub notice: NoticeHours,
    pub penalty: CancellationPenalty,
}

impl CancellationPolicy {
    pub const fn new(notice: NoticeHours, penalty: CancellationPenalty) -> Self {
        Self { notice, penalty }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CancellationPenalty {
    None,
    ForfeitDeposit,
    ManagerReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepositRule {
    NotRequired,
    Required { amount: money::Money },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentTiming {
    DueAtBooking,
    DueAtCheckIn,
    DueAtCheckout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HousekeepingCadence {
    DailyRoomReset,
    TwiceDailyForExtendedStay,
    TurnoverOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandoffRequirement {
    ArrivalCareReview,
    MedicationDoubleCheck,
    DepartureBelongingsReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Upsell {
    ExitBath,
    TrainingSession,
    EnrichmentPlay,
    PremiumBedding,
}

/// Housekeeping policies for boarded pets and room turns.
pub mod housekeeping;

/// Check-in/check-out windows and staff handoff requirements.
pub mod handoff;

/// Minimum-stay rules for holidays, multi-pet buffers, and standard stays.
pub mod minimum_stay;

/// Cancellation notice and penalty rules for boarding reservations.
pub mod cancellation;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Contract {
    pub capacity: CapacityPlan,
    pub arrival_window: ServiceWindow,
    pub departure_window: ServiceWindow,
    pub minimum_stay: MinimumStay,
    pub cancellation: CancellationPolicy,
    pub deposit: DepositRule,
    pub payment: PaymentTiming,
    pub housekeeping: HousekeepingCadence,
    pub handoff: HandoffRequirement,
    #[builder(default)]
    pub upsells: Vec<Upsell>,
}

impl Contract {
    pub fn requires_deposit_collection(&self) -> bool {
        matches!(self.deposit, DepositRule::Required { .. })
    }
    pub fn standard_petsuites() -> Self {
        Self::builder()
            .capacity(CapacityPlan::new(
                RoomInventory::try_new(1).unwrap(),
                RoomAvailability::Limited,
            ))
            .arrival_window(
                ServiceWindow::new(
                    HourOfDay::try_new(7).unwrap(),
                    HourOfDay::try_new(18).unwrap(),
                )
                .unwrap(),
            )
            .departure_window(
                ServiceWindow::new(
                    HourOfDay::try_new(7).unwrap(),
                    HourOfDay::try_new(12).unwrap(),
                )
                .unwrap(),
            )
            .minimum_stay(MinimumStay::new(
                StayNights::try_new(1).unwrap(),
                MinimumStayReason::StandardPolicy,
            ))
            .cancellation(CancellationPolicy::new(
                NoticeHours::try_new(24).unwrap(),
                CancellationPenalty::ForfeitDeposit,
            ))
            .deposit(DepositRule::Required {
                amount: money::Money::new(
                    money::MinorUnits::try_new(1).unwrap(),
                    money::Currency::Usd,
                ),
            })
            .payment(PaymentTiming::DueAtCheckout)
            .housekeeping(HousekeepingCadence::DailyRoomReset)
            .handoff(HandoffRequirement::ArrivalCareReview)
            .upsells(vec![Upsell::ExitBath, Upsell::TrainingSession])
            .build()
    }
}
