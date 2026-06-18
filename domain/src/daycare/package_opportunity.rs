//! Daycare package-opportunity policy for review-gated membership/pass recommendations.
//!
//! ```
//! use domain::{daycare, entities, policy};
//! use uuid::Uuid;
//!
//! let evidence = daycare::package_opportunity::Evidence::builder()
//!     .customer_id(entities::CustomerId(Uuid::nil()))
//!     .pet_id(entities::PetId(Uuid::nil()))
//!     .attendance_visits(daycare::package_opportunity::AttendanceVisitCount::new(8))
//!     .eligibility(daycare::package_opportunity::CareEligibility::Cleared)
//!     .package_state(daycare::package_opportunity::PackageState::PayPerVisit)
//!     .payment_state(daycare::package_opportunity::PaymentState::Current)
//!     .build();
//!
//! assert_eq!(
//!     daycare::package_opportunity::Policy.classify(&evidence),
//!     daycare::package_opportunity::Decision::RecommendStaffReview {
//!         score: daycare::package_opportunity::OpportunityScore::Strong,
//!         gate: policy::ReviewGate::CustomerMessageApproval,
//!     },
//! );
//! ```

use super::*;
use crate::policy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AttendanceVisitCount(u16);

impl AttendanceVisitCount {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CareEligibility {
    Cleared,
    BlockedBySafetyReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageState {
    PayPerVisit,
    AlreadyCovered,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentState {
    Current,
    NeedsBillingReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Evidence {
    pub customer_id: CustomerId,
    pub pet_id: PetId,
    pub attendance_visits: AttendanceVisitCount,
    pub eligibility: CareEligibility,
    pub package_state: PackageState,
    pub payment_state: PaymentState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    RecommendStaffReview {
        score: OpportunityScore,
        gate: policy::ReviewGate,
    },
    Suppressed {
        reason: SuppressionReason,
        gate: policy::ReviewGate,
    },
    NoOpportunity {
        reason: NoOpportunityReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpportunityScore {
    Moderate,
    Strong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuppressionReason {
    SafetyOrCareReviewRequired,
    PaymentOrBillingReviewRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoOpportunityReason {
    AlreadyCovered,
    NotEnoughAttendanceHistory,
}

#[derive(Debug, Clone, Default)]
pub struct Policy;

impl Policy {
    pub fn classify(&self, evidence: &Evidence) -> Decision {
        if matches!(evidence.eligibility, CareEligibility::BlockedBySafetyReview) {
            return Decision::Suppressed {
                reason: SuppressionReason::SafetyOrCareReviewRequired,
                gate: policy::ReviewGate::BehaviorReview,
            };
        }
        if matches!(evidence.payment_state, PaymentState::NeedsBillingReview) {
            return Decision::Suppressed {
                reason: SuppressionReason::PaymentOrBillingReviewRequired,
                gate: policy::ReviewGate::RefundOrDepositException,
            };
        }
        if matches!(evidence.package_state, PackageState::AlreadyCovered) {
            return Decision::NoOpportunity {
                reason: NoOpportunityReason::AlreadyCovered,
            };
        }
        if evidence.attendance_visits.get() >= 8 {
            Decision::RecommendStaffReview {
                score: OpportunityScore::Strong,
                gate: policy::ReviewGate::CustomerMessageApproval,
            }
        } else {
            Decision::NoOpportunity {
                reason: NoOpportunityReason::NotEnoughAttendanceHistory,
            }
        }
    }
}
