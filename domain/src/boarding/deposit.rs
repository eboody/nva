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
//!         blocker: boarding::deposit::Blocker::DepositRequired,
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
//!     boarding::deposit::ConfirmationReadiness::Ready,
//! );
//! ```

use super::*;
use crate::{payment, policy};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed policy domain value that keeps raw primitives out of boarding workflows.
pub struct Policy {
    rule: DepositRule,
    timing: PaymentTiming,
}

impl Policy {
    /// Assembles this boarding value from already-validated domain parts.
    pub const fn new(rule: DepositRule, timing: PaymentTiming) -> Self {
        Self { rule, timing }
    }

    /// Returns the readiness for confirmation for this boarding value.
    pub fn readiness_for_confirmation(
        &self,
        deposit: Option<&payment::Deposit>,
    ) -> ConfirmationReadiness {
        match (
            &self.rule,
            self.timing,
            deposit.map(payment::Deposit::status),
        ) {
            (DepositRule::NotRequired, _, _) => ConfirmationReadiness::Ready,
            (
                DepositRule::Required { .. },
                _,
                Some(payment::DepositStatus::Paid | payment::DepositStatus::WaivedByManager),
            ) => ConfirmationReadiness::Ready,
            (DepositRule::Required { .. }, PaymentTiming::DueAtBooking, _) => {
                ConfirmationReadiness::Blocked {
                    blocker: Blocker::DepositRequired,
                    review_gate: policy::ReviewGate::RefundOrDepositException,
                }
            }
            (DepositRule::Required { .. }, _, _) => ConfirmationReadiness::Ready,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for confirmation readiness decisions in boarding workflows.
pub enum ConfirmationReadiness {
    /// Ready boarding policy, stay, capacity, or upsell signal.
    Ready,
    /// Blocked boarding policy, stay, capacity, or upsell signal.
    Blocked {
        /// Blocker fact promoted into this boarding contract.
        blocker: Blocker,
        /// Review gate fact promoted into this boarding contract.
        review_gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for blocker decisions in boarding workflows.
pub enum Blocker {
    /// Deposit must be collected before the booking is secure.
    DepositRequired,
    /// Reference missing boarding policy, stay, capacity, or upsell signal.
    ReferenceMissing,
}
