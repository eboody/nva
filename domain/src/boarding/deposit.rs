use super::*;
use crate::{payment, policy};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    rule: DepositRule,
    timing: PaymentTiming,
}

impl Policy {
    pub const fn new(rule: DepositRule, timing: PaymentTiming) -> Self {
        Self { rule, timing }
    }

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
pub enum ConfirmationReadiness {
    Ready,
    Blocked {
        blocker: Blocker,
        review_gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Blocker {
    DepositRequired,
    ReferenceMissing,
}
