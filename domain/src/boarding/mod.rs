use bon::Builder;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{LocationId, PetId};
use crate::money;

macro_rules! positive_scalar {
    ($name:ident, $primitive:ty, $error:ident, $message:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        /// Human-readable name used in boarding workflows.
        pub struct $name($primitive);

        impl $name {
            /// Promotes boundary input into a validated boarding domain value.
            pub const fn try_new(value: $primitive) -> std::result::Result<Self, $error> {
                if value == 0 {
                    return Err($error::Zero);
                }
                Ok(Self(value))
            }

            /// Exposes the validated scalar for serialization and adapter boundaries.
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
        /// Validation failures returned by boarding domain constructors.
        pub enum $error {
            #[error($message)]
            /// Rejects zero where the pet-resort workflow requires a positive quantity.
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
/// Typed hour of day domain value that keeps raw primitives out of boarding workflows.
pub struct HourOfDay(u8);

impl HourOfDay {
    /// Promotes boundary input into a validated boarding domain value.
    pub const fn try_new(value: u8) -> std::result::Result<Self, HourOfDayError> {
        if value > 23 {
            return Err(HourOfDayError::OutsideClockDay);
        }
        Ok(Self(value))
    }

    /// Exposes the validated scalar for serialization and adapter boundaries.
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
/// Domain vocabulary for hour of day error decisions in boarding workflows.
pub enum HourOfDayError {
    #[error("boarding service-window hour must be between 0 and 23")]
    /// Outside clock day boarding policy, stay, capacity, or upsell signal.
    OutsideClockDay,
}

/// Accommodation boundary for boarding contracts.
pub mod accommodation;

pub mod capacity;

pub mod deposit;

/// Care boundary for boarding contracts.
pub mod care;

/// Upsell boundary for boarding contracts.
pub mod upsell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for room availability decisions in boarding workflows.
pub enum RoomAvailability {
    /// Open boarding policy, stay, capacity, or upsell signal.
    Open,
    /// Limited boarding policy, stay, capacity, or upsell signal.
    Limited,
    /// Waitlist only boarding policy, stay, capacity, or upsell signal.
    WaitlistOnly,
    /// Closed boarding policy, stay, capacity, or upsell signal.
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Typed capacity plan domain value that keeps raw primitives out of boarding workflows.
pub struct CapacityPlan {
    room_inventory: RoomInventory,
    /// Availability fact promoted into this boarding contract.
    pub availability: RoomAvailability,
}

impl CapacityPlan {
    /// Assembles this boarding value from already-validated domain parts.
    pub const fn new(room_inventory: RoomInventory, availability: RoomAvailability) -> Self {
        Self {
            room_inventory,
            availability,
        }
    }
    /// Returns this boarding value's room inventory.
    pub const fn room_inventory(&self) -> RoomInventory {
        self.room_inventory
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Typed service window domain value that keeps raw primitives out of boarding workflows.
pub struct ServiceWindow {
    start: HourOfDay,
    end: HourOfDay,
}

impl ServiceWindow {
    /// Assembles this boarding value from already-validated domain parts.
    pub const fn new(
        start: HourOfDay,
        end: HourOfDay,
    ) -> std::result::Result<Self, ServiceWindowError> {
        if start.get() >= end.get() {
            return Err(ServiceWindowError::EndMustFollowStart);
        }
        Ok(Self { start, end })
    }
    /// Returns this boarding value's start.
    pub const fn start(&self) -> HourOfDay {
        self.start
    }
    /// Returns this boarding value's end.
    pub const fn end(&self) -> HourOfDay {
        self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Domain vocabulary for service window error decisions in boarding workflows.
pub enum ServiceWindowError {
    #[error("boarding service window end must follow start")]
    /// End must follow start boarding policy, stay, capacity, or upsell signal.
    EndMustFollowStart,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for deposit rule decisions in boarding workflows.
pub enum DepositRule {
    /// No deposit or review is needed for this reservation path.
    NotRequired,
    /// Amount fact promoted into this boarding contract.
    Required {
        /// Amount carried by this variant.
        amount: money::Money,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for payment timing decisions in boarding workflows.
pub enum PaymentTiming {
    /// Due at booking boarding policy, stay, capacity, or upsell signal.
    DueAtBooking,
    /// Due at check in boarding policy, stay, capacity, or upsell signal.
    DueAtCheckIn,
    /// Due at checkout boarding policy, stay, capacity, or upsell signal.
    DueAtCheckout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for upsell decisions in boarding workflows.
pub enum Upsell {
    /// Bath offered before departure from boarding.
    ExitBath,
    /// Training session boarding policy, stay, capacity, or upsell signal.
    TrainingSession,
    /// Enrichment play boarding policy, stay, capacity, or upsell signal.
    EnrichmentPlay,
    /// Premium bedding boarding policy, stay, capacity, or upsell signal.
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
/// Typed contract domain value that keeps raw primitives out of boarding workflows.
pub struct Contract {
    /// Capacity fact promoted into this boarding contract.
    pub capacity: CapacityPlan,
    /// Arrival window fact promoted into this boarding contract.
    pub arrival_window: ServiceWindow,
    /// Departure window fact promoted into this boarding contract.
    pub departure_window: ServiceWindow,
    /// Minimum stay fact promoted into this boarding contract.
    pub minimum_stay: minimum_stay::Policy,
    /// Cancellation fact promoted into this boarding contract.
    pub cancellation: cancellation::Policy,
    /// Deposit fact promoted into this boarding contract.
    pub deposit: DepositRule,
    /// Payment fact promoted into this boarding contract.
    pub payment: PaymentTiming,
    /// Housekeeping fact promoted into this boarding contract.
    pub housekeeping: housekeeping::Cadence,
    /// Handoff fact promoted into this boarding contract.
    pub handoff: handoff::Requirement,
    #[builder(default)]
    /// Upsells fact promoted into this boarding contract.
    pub upsells: Vec<Upsell>,
}

impl Contract {
    /// Returns the requires deposit collection for this boarding value.
    pub fn requires_deposit_collection(&self) -> bool {
        matches!(self.deposit, DepositRule::Required { .. })
    }
    /// Returns the standard petsuites for this boarding value.
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
            .minimum_stay(minimum_stay::Policy::new(
                StayNights::try_new(1).unwrap(),
                minimum_stay::Reason::StandardPolicy,
            ))
            .cancellation(cancellation::Policy::new(
                NoticeHours::try_new(24).unwrap(),
                cancellation::Penalty::ForfeitDeposit,
            ))
            .deposit(DepositRule::Required {
                amount: money::Money::new(
                    money::MinorUnits::try_new(1).unwrap(),
                    money::Currency::Usd,
                ),
            })
            .payment(PaymentTiming::DueAtCheckout)
            .housekeeping(housekeeping::Cadence::DailyRoomReset)
            .handoff(handoff::Requirement::ArrivalCareReview)
            .upsells(vec![Upsell::ExitBath, Upsell::TrainingSession])
            .build()
    }
}
