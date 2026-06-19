//! Boarding service-line rules for capacity, stay policy, care handoffs, and upsells.
//!
//! # Operator summary
//!
//! Boarding supports the queue where front-desk staff and managers decide whether an overnight
//! stay can be confirmed, waitlisted, held for care review, or turned into a safe follow-up offer.
//! The useful labor reduction is deterministic triage: capacity snapshots, care profiles, deposit
//! status, cancellation notice, housekeeping cadence, handoff checklists, and exit-bath evidence
//! become typed decisions instead of scattered note reading and ad hoc manager pings.
//!
//! This module is not permission to automate live boarding operations. It does not book or cancel
//! reservations, mutate room inventory, collect deposits, issue refunds, send customer messages,
//! make medical judgments, or override staff. Source systems and promoted domain facts remain
//! authoritative: room counts and reservation status from the provider/read model, payment/deposit
//! evidence from `domain::payment`, care instructions and medication-review requirements from the
//! pet care profile, and policy values captured in the boarding location rules.
//!
//! Review gates protect pets, customers, and staff at the unsafe edges: manager approval for
//! denied or exception capacity decisions, medical-document review for missing feeding or medication
//! ambiguity, refund/deposit exception review for payment edge cases, and customer-message approval
//! before any upsell recommendation becomes customer-facing.
//!
//! Next step: start with the location ruleset to see the policy, then open the child module that
//! matches the business question: `capacity` for confirm/waitlist decisions, `accommodation`
//! for room fit, `deposit` for payment gates, `care` for feeding or medication handoff,
//! `housekeeping` for stay execution, or `upsell` for review-gated add-on recommendations.
//!
//! The rest of the module documents the externally visible boarding rules that labor-saving agents
//! may use when drafting staff packets, manager briefs, and customer-response recommendations.

use bon::Builder;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{LocationId, PetId};
use crate::money;

macro_rules! positive_scalar {
    ($name:ident, $primitive:ty, $error:ident, $message:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        /// Positive boarding quantity used where zero would erase a real labor or stay requirement.
        pub struct $name($primitive);

        impl $name {
            /// Rejects impossible boarding values before they affect capacity, minimum-stay, deposit, service-window, or checkout calculations.
            pub const fn try_new(value: $primitive) -> std::result::Result<Self, $error> {
                if value == 0 {
                    return Err($error::Zero);
                }
                Ok(Self(value))
            }

            /// Returns the boarding number used by capacity, service-window, minimum-stay, deposit, or checkout calculations.
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
        /// Validation failure returned when a required positive boarding scalar is zero.
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
/// Hour within a resort service day used for boarding arrival and departure windows.
pub struct HourOfDay(u8);

impl HourOfDay {
    /// Rejects impossible boarding values before they affect capacity, minimum-stay, deposit, service-window, or checkout calculations.
    pub const fn try_new(value: u8) -> std::result::Result<Self, HourOfDayError> {
        if value > 23 {
            return Err(HourOfDayError::OutsideClockDay);
        }
        Ok(Self(value))
    }

    /// Returns the boarding number used by capacity, service-window, minimum-stay, deposit, or checkout calculations.
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
/// Validation errors for boarding service-window hours.
pub enum HourOfDayError {
    #[error("boarding service-window hour must be between 0 and 23")]
    /// Hour was outside the 0–23 clock range and cannot define a service window.
    OutsideClockDay,
}

/// Accommodation policy for room/suite fit, species safety, capacity, and booking recommendations.
pub mod accommodation;

/// Room and suite capacity policy for confirm, waitlist, and denial decisions.
pub mod capacity;

/// Deposit readiness policy for boarding confirmation gates.
pub mod deposit;

/// Care policy for feeding, medication review, and kennel-staff handoff decisions during a stay.
pub mod care;

/// Upsell policy for review-gated boarding add-ons and checkout recommendations.
pub mod upsell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Coarse availability status used in boarding rules and manager briefs.
pub enum RoomAvailability {
    /// Rooms are generally available for this reservation path.
    Open,
    /// Inventory is constrained and staff should treat capacity as a labor/care watch item.
    Limited,
    /// New reservations should be routed to waitlist unless a manager approves otherwise.
    WaitlistOnly,
    /// Reservations should not be accepted from this reservation path.
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Capacity posture for boarding rules, pairing inventory with availability status.
pub struct CapacityPlan {
    room_inventory: RoomInventory,
    /// Staff-facing availability status derived from resort capacity evidence.
    pub availability: RoomAvailability,
}

impl CapacityPlan {
    /// Creates the boarding value from validated domain parts without re-reading source systems.
    pub const fn new(room_inventory: RoomInventory, availability: RoomAvailability) -> Self {
        Self {
            room_inventory,
            availability,
        }
    }
    /// Returns the inventory count represented by this capacity plan.
    pub const fn room_inventory(&self) -> RoomInventory {
        self.room_inventory
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Check-in or check-out window that constrains front-desk staffing and guest promises.
pub struct ServiceWindow {
    start: HourOfDay,
    end: HourOfDay,
}

impl ServiceWindow {
    /// Creates the boarding value from validated domain parts without re-reading source systems.
    pub const fn new(
        start: HourOfDay,
        end: HourOfDay,
    ) -> std::result::Result<Self, ServiceWindowError> {
        if start.get() >= end.get() {
            return Err(ServiceWindowError::EndMustFollowStart);
        }
        Ok(Self { start, end })
    }
    /// Returns the inclusive start hour staff may use for this service window.
    pub const fn start(&self) -> HourOfDay {
        self.start
    }
    /// Returns the exclusive end hour after which this service window is closed.
    pub const fn end(&self) -> HourOfDay {
        self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation errors for boarding arrival or departure windows.
pub enum ServiceWindowError {
    #[error("boarding service window end must follow start")]
    /// The end hour did not follow the start hour, so the window cannot be offered.
    EndMustFollowStart,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Deposit rule used to determine whether a boarding reservation can be confirmed.
pub enum DepositRule {
    /// Reservation path is secured without a deposit requirement.
    NotRequired,
    /// Required deposit amount sourced from policy or booking evidence.
    Required {
        /// Money amount staff must collect or have waived before this deposit rule is satisfied.
        amount: money::Money,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Payment timing that controls when staff must collect boarding charges or deposits.
pub enum PaymentTiming {
    /// Payment is required before the reservation is considered secured.
    DueAtBooking,
    /// Payment is collected when the pet arrives for the stay.
    DueAtCheckIn,
    /// Payment can be collected during departure checkout.
    DueAtCheckout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Optional boarding-adjacent services that may appear in staff offer recommendations.
pub enum Upsell {
    /// Bath offered before departure from boarding.
    ExitBath,
    /// Training add-on that can be bundled with a boarding stay after staff review.
    TrainingSession,
    /// Additional play or enrichment add-on during the stay.
    EnrichmentPlay,
    /// Premium comfort add-on for the boarding room or suite.
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
/// Boarding service-line ruleset combining capacity, stay, payment, handoff, and upsell policy.
pub struct Contract {
    /// Capacity posture staff and automation must honor before confirming stays.
    pub capacity: CapacityPlan,
    /// Guest arrival window used for front-desk staffing and check-in promises.
    pub arrival_window: ServiceWindow,
    /// Guest departure window used for checkout staffing and Pawgress/report timing.
    pub departure_window: ServiceWindow,
    /// Minimum-stay rule for standard, holiday, or multi-pet boarding demand.
    pub minimum_stay: minimum_stay::Policy,
    /// Cancellation policy that governs notice, deposit forfeiture, and manager review.
    pub cancellation: cancellation::Policy,
    /// Deposit requirement used before staff or automation treats the booking as secured.
    pub deposit: DepositRule,
    /// Payment timing that constrains collection workflow and front-desk labor.
    pub payment: PaymentTiming,
    /// Room-cleaning cadence that feeds labor planning for the stay.
    pub housekeeping: housekeeping::Cadence,
    /// Staff handoff checklist required at arrival, medication review, or departure.
    pub handoff: handoff::Requirement,
    #[builder(default)]
    /// Optional services that can be offered only through the review-gated recommendation flow.
    pub upsells: Vec<Upsell>,
}

impl Contract {
    /// Reports whether these boarding rules require deposit collection before confirmation.
    pub fn requires_deposit_collection(&self) -> bool {
        matches!(self.deposit, DepositRule::Required { .. })
    }
    /// Builds the baseline PetSuites-style boarding rules used by examples and tests.
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
