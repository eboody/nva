//! Reservation support facts used by booking triage and checkout handoff workflows.
//!
//! ## Operator summary
//!
//! Staff use reservation facts to decide which booking or checkout queue owns the next review:
//! routine front-desk collection, vaccine/document follow-up, care or behavior review, payment
//! review, manager exception, or read-only handoff to checkout/retention workflows. The module
//! reduces labor by naming reusable policy facts—minimum-age thresholds, add-on labels, and
//! transition reasons—so application packets can surface the same evidence without staff retyping
//! provider notes or reconciling free-text labels.
//!
//! This module must not book, confirm, cancel, check in/out, hold capacity, change pricing,
//! move money, mutate Gingr/provider/PMS records, or send customer messages. It is source
//! vocabulary only. Live authority stays with the provider/PMS ledger, approved location policy,
//! verified payment/deposit records, customer/pet/reservation source snapshots, and accountable
//! staff/manager approvals.
//!
//! Review gates protect pets, customers, and staff whenever reservation facts touch medical or
//! vaccine evidence, temperament/incident handling, special-care acceptance, capacity or staffing
//! exceptions, payment/deposit closeout, customer-sensitive copy, or provider mutation. Unknown,
//! stale, conflicting, or unmapped source facts should remain review work; they must not become
//! inferred readiness.

mod error;

use serde::{Deserialize, Deserializer, Serialize};

pub use error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Minimum pet age, in weeks, required before a service can be booked.
pub struct MinimumAgeWeeks(u8);

impl MinimumAgeWeeks {
    /// Promotes reservation policy input after enforcing the domain validation rule.
    pub fn try_new(value: u8) -> Result<Self> {
        if value == 0 {
            return Err(Error::EmptyMinimumAge);
        }
        Ok(Self(value))
    }

    /// Returns the minimum age threshold used by booking and policy adapters.
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl<'de> Deserialize<'de> for MinimumAgeWeeks {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons a pet may be blocked by age rules for a service.
pub enum AgePolicyReason {
    /// Minimum-age rule for boarding reservations.
    BoardingMinimum,
    /// Minimum-age rule for day-play reservations.
    DayPlayMinimum,
    /// Minimum-age rule for daycare reservations.
    DaycareMinimum,
    /// Minimum-age rule configured for a specific service.
    ServiceSpecificMinimum,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Age gate applied when determining whether a pet may book a service.
pub struct AgeThreshold {
    minimum: MinimumAgeWeeks,
    reason: AgePolicyReason,
}

impl AgeThreshold {
    /// Assembles a reservation policy value from validated age and reason parts.
    pub const fn new(minimum: MinimumAgeWeeks, reason: AgePolicyReason) -> Self {
        Self { minimum, reason }
    }

    /// Returns the reservation minimum used by the policy gate.
    pub const fn minimum(&self) -> MinimumAgeWeeks {
        self.minimum
    }

    /// Returns the reservation reason used by the policy gate.
    pub const fn reason(&self) -> AgePolicyReason {
        self.reason
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Display label for an optional reservation add-on offered to the customer.
pub struct AddOnLabel(String);

impl AddOnLabel {
    /// Promotes reservation policy input after enforcing the domain validation rule.
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(Error::EmptyAddOnLabel);
        }
        if value.chars().count() > 120 {
            return Err(Error::AddOnLabelTooLong);
        }
        Ok(Self(value))
    }

    /// Returns the owned inner string for storage or outbound mapping.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<'de> Deserialize<'de> for AddOnLabel {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Source exception retained when checkout handoff evidence cannot be trusted until provider/PMS reconciliation.
pub enum CheckoutSourceException {
    /// Provider/PMS record conflicts with the staff handoff or checkout packet.
    ProviderRecordConflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Human/system-of-record disposition for checkout completion before labor savings can be claimed.
pub enum CheckoutCompletionDisposition {
    /// Handoff and source evidence support staff-verified checkout, with outbound communication still approval-gated.
    StaffVerified,
    /// Manager or lead review is required before final closeout.
    ManagerReviewRequired,
    /// Provider/PMS source reconciliation is required before checkout can be trusted.
    SourceReconciliationRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Business reasons for moving or rejecting a reservation workflow transition.
pub enum TransitionReason {
    /// Transition was initiated by a customer request.
    CustomerRequested,
    /// Transition was blocked because the requested capacity is unavailable.
    CapacityUnavailable,
    /// Transition is blocked by a non-overridable policy.
    PolicyHardStop,
    /// Transition is blocked until required customer or pet details are supplied.
    MissingRequiredInformation,
    /// Staff manually approved a workflow transition.
    StaffOverride,
}
