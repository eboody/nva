//! Payment-domain values for deposits, references, and review-safe money movement state.
//!
//! These types record payment readiness without performing provider writes. That keeps deposit
//! exceptions and front-desk collection work reviewable:
//!
//! ```
//! use domain::{money, payment};
//!
//! let amount = money::Money::new(
//!     money::MinorUnits::try_new(5_000).unwrap(),
//!     money::Currency::Usd,
//! );
//! let deposit = payment::Deposit::paid(
//!     amount,
//!     payment::Reference::try_new("sandbox-receipt-42").unwrap(),
//! );
//!
//! assert_eq!(deposit.status(), payment::DepositStatus::Paid);
//! assert!(!deposit.requires_collection());
//! assert!(deposit.payment_reference().is_some());
//! ```

mod error;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use crate::money::Money;

pub use error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// External payment or POS reference used to reconcile pet-resort charges.
pub struct Reference(String);

impl Reference {
    /// Validates and creates the payment value.
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(Error::EmptyReference);
        }
        if value.chars().count() > 160 {
            return Err(Error::ReferenceTooLong);
        }
        Ok(Self(value))
    }

    /// Returns the owned inner string for storage or outbound mapping.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<'de> Deserialize<'de> for Reference {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Lifecycle states for collecting, waiving, refunding, or failing a reservation deposit.
pub enum DepositStatus {
    /// No deposit or review is needed for this reservation path.
    NotRequired,
    /// A deposit must be collected before the booking is secure.
    Required,
    /// The required deposit has been collected and reconciled.
    Paid,
    /// The collected deposit has been returned to the customer.
    Refunded,
    /// Deposit collection was attempted but did not succeed.
    Failed,
    /// A manager waived the deposit requirement for the booking.
    WaivedByManager,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Deposit amount, status, refund window, and payment reference for a reservation.
pub struct Deposit {
    amount: Money,
    refundable_until: Option<DateTime<Utc>>,
    status: DepositStatus,
    payment_reference: Option<Reference>,
}

impl Deposit {
    /// Starts a deposit record that still needs front-desk collection.
    pub const fn required(amount: Money) -> Self {
        Self {
            amount,
            refundable_until: None,
            status: DepositStatus::Required,
            payment_reference: None,
        }
    }

    /// Creates a deposit already reconciled to a payment reference.
    pub fn paid(amount: Money, payment_reference: Reference) -> Self {
        Self::required(amount).mark_paid(payment_reference)
    }

    /// Starts a deposit record waived by manager exception.
    pub const fn waived(amount: Money) -> Self {
        Self {
            amount,
            refundable_until: None,
            status: DepositStatus::WaivedByManager,
            payment_reference: None,
        }
    }

    /// Sets the deadline through which the deposit remains refundable.
    pub fn with_refundable_until(mut self, refundable_until: DateTime<Utc>) -> Self {
        self.refundable_until = Some(refundable_until);
        self
    }

    /// Marks the deposit as collected and stores the payment reference.
    pub fn mark_paid(mut self, payment_reference: Reference) -> Self {
        self.status = DepositStatus::Paid;
        self.payment_reference = Some(payment_reference);
        self
    }

    /// Returns this payment value's amount.
    pub const fn amount(&self) -> &Money {
        &self.amount
    }

    /// Returns this payment value's refundable until.
    pub const fn refundable_until(&self) -> Option<DateTime<Utc>> {
        self.refundable_until
    }

    /// Returns this payment value's status.
    pub const fn status(&self) -> DepositStatus {
        self.status
    }

    /// Returns this payment value's payment reference.
    pub const fn payment_reference(&self) -> Option<&Reference> {
        self.payment_reference.as_ref()
    }

    /// Reports whether this deposit still requires collection from the customer.
    pub const fn requires_collection(&self) -> bool {
        matches!(self.status, DepositStatus::Required | DepositStatus::Failed)
    }
}
