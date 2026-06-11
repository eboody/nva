mod error;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use crate::money::Money;

pub use error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct PaymentReference(String);

impl PaymentReference {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(Error::EmptyPaymentReference);
        }
        if value.chars().count() > 160 {
            return Err(Error::PaymentReferenceTooLong);
        }
        Ok(Self(value))
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<'de> Deserialize<'de> for PaymentReference {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepositStatus {
    NotRequired,
    Required,
    Paid,
    Refunded,
    Failed,
    WaivedByManager,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deposit {
    amount: Money,
    refundable_until: Option<DateTime<Utc>>,
    status: DepositStatus,
    payment_reference: Option<PaymentReference>,
}

impl Deposit {
    pub const fn required(amount: Money) -> Self {
        Self {
            amount,
            refundable_until: None,
            status: DepositStatus::Required,
            payment_reference: None,
        }
    }

    pub fn paid(amount: Money, payment_reference: PaymentReference) -> Self {
        Self::required(amount).mark_paid(payment_reference)
    }

    pub const fn waived(amount: Money) -> Self {
        Self {
            amount,
            refundable_until: None,
            status: DepositStatus::WaivedByManager,
            payment_reference: None,
        }
    }

    pub fn with_refundable_until(mut self, refundable_until: DateTime<Utc>) -> Self {
        self.refundable_until = Some(refundable_until);
        self
    }

    pub fn mark_paid(mut self, payment_reference: PaymentReference) -> Self {
        self.status = DepositStatus::Paid;
        self.payment_reference = Some(payment_reference);
        self
    }

    pub const fn amount(&self) -> &Money {
        &self.amount
    }

    pub const fn refundable_until(&self) -> Option<DateTime<Utc>> {
        self.refundable_until
    }

    pub const fn status(&self) -> DepositStatus {
        self.status
    }

    pub const fn payment_reference(&self) -> Option<&PaymentReference> {
        self.payment_reference.as_ref()
    }

    pub const fn requires_collection(&self) -> bool {
        matches!(self.status, DepositStatus::Required | DepositStatus::Failed)
    }
}
