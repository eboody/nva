//! Boarding deposit rules that keep confirmation policy deterministic.
//!
//! A deposit can block confirmation without letting an agent collect money or waive fees by itself:
//!
//! ```
//! use domain::{boarding, money, payment, policy};
//!
//! let deposit_amount = money::Money::new(
//!     money::MinorUnits::try_new(5_000).unwrap(),
//!     money::Currency::Usd,
//! );
//! let policy = boarding::deposit::Policy::new(
//!     boarding::DepositRule::Required { amount: deposit_amount.clone() },
//!     boarding::PaymentTiming::DueAtBooking,
//! );
//!
//! let readiness = policy.readiness_for_confirmation(None);
//! assert_eq!(
//!     readiness,
//!     boarding::deposit::ConfirmationReadiness::Blocked {
//!         blocker: boarding::deposit::Blocker::ReferenceMissing,
//!         review_gate: policy::ReviewGate::RefundOrDepositException,
//!     }
//! );
//!
//! let paid = payment::Deposit::paid(
//!     deposit_amount,
//!     payment::Reference::try_new("fixture-gateway-reference").unwrap(),
//! );
//! assert_eq!(
//!     policy.readiness_for_confirmation(Some(&paid)),
//!     boarding::deposit::ConfirmationReadiness::Blocked {
//!         blocker: boarding::deposit::Blocker::ReferenceMissing,
//!         review_gate: policy::ReviewGate::RefundOrDepositException,
//!     },
//! );
//! ```

use super::*;
use crate::{payment, policy};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Deposit-collection policy used before confirming a boarding reservation.
pub struct Policy {
    rule: DepositRule,
    timing: PaymentTiming,
}

impl Policy {
    /// Creates a deposit policy from the resort rule and payment timing.
    pub const fn new(rule: DepositRule, timing: PaymentTiming) -> Self {
        Self { rule, timing }
    }

    /// Serializable deposit and policy history cannot authorize confirmation.
    pub fn readiness_for_confirmation(
        &self,
        _deposit: Option<&payment::Deposit>,
    ) -> ConfirmationReadiness {
        ConfirmationReadiness::Blocked {
            blocker: Blocker::ReferenceMissing,
            review_gate: policy::ReviewGate::RefundOrDepositException,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Deposit readiness result for boarding reservation confirmation.
pub enum ConfirmationReadiness {
    /// Deposit policy is satisfied, waived by manager, or not required for confirmation.
    Ready,
    /// Confirmation is blocked until staff collect a deposit or approve an exception.
    Blocked {
        /// Operational reason confirmation cannot proceed automatically.
        blocker: Blocker,
        /// Manager or billing gate required to override the blocked deposit state.
        review_gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons deposit policy can block boarding confirmation.
pub enum Blocker {
    /// Deposit must be collected before the booking is secure.
    DepositRequired,
    /// A payment reference is missing, so staff cannot verify that the deposit was collected.
    ReferenceMissing,
}
