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
/// Typed attendance visit count domain value that keeps raw primitives out of daycare workflows.
pub struct AttendanceVisitCount(u16);

impl AttendanceVisitCount {
    /// Assembles this daycare value from already-validated domain parts.
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// Exposes the validated scalar for serialization and adapter boundaries.
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for care eligibility decisions in daycare workflows.
pub enum CareEligibility {
    /// Cleared daycare attendance, eligibility, coverage, or package signal.
    Cleared,
    /// Blocked by safety review daycare attendance, eligibility, coverage, or package signal.
    BlockedBySafetyReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for package state decisions in daycare workflows.
pub enum PackageState {
    /// Pay per visit daycare attendance, eligibility, coverage, or package signal.
    PayPerVisit,
    /// Already covered daycare attendance, eligibility, coverage, or package signal.
    AlreadyCovered,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for payment state decisions in daycare workflows.
pub enum PaymentState {
    /// Current daycare attendance, eligibility, coverage, or package signal.
    Current,
    /// Needs billing review daycare attendance, eligibility, coverage, or package signal.
    NeedsBillingReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Typed evidence domain value that keeps raw primitives out of daycare workflows.
pub struct Evidence {
    /// Customer id fact promoted into this daycare contract.
    pub customer_id: CustomerId,
    /// Pet receiving the grooming or care service.
    pub pet_id: PetId,
    /// Attendance visits fact promoted into this daycare contract.
    pub attendance_visits: AttendanceVisitCount,
    /// Eligibility fact promoted into this daycare contract.
    pub eligibility: CareEligibility,
    /// Package state fact promoted into this daycare contract.
    pub package_state: PackageState,
    /// Payment state fact promoted into this daycare contract.
    pub payment_state: PaymentState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for decision decisions in daycare workflows.
pub enum Decision {
    /// Recommend staff review daycare attendance, eligibility, coverage, or package signal.
    RecommendStaffReview {
        /// Score fact promoted into this daycare contract.
        score: OpportunityScore,
        /// Gate fact promoted into this daycare contract.
        gate: policy::ReviewGate,
    },
    /// Suppressed daycare attendance, eligibility, coverage, or package signal.
    Suppressed {
        /// Business reason staff should review before proceeding.
        reason: SuppressionReason,
        /// Gate fact promoted into this daycare contract.
        gate: policy::ReviewGate,
    },
    /// No opportunity daycare attendance, eligibility, coverage, or package signal.
    NoOpportunity {
        /// Business reason staff should review before proceeding.
        reason: NoOpportunityReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for opportunity score decisions in daycare workflows.
pub enum OpportunityScore {
    /// Moderate daycare attendance, eligibility, coverage, or package signal.
    Moderate,
    /// Strong daycare attendance, eligibility, coverage, or package signal.
    Strong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for suppression reason decisions in daycare workflows.
pub enum SuppressionReason {
    /// Safety or care review required daycare attendance, eligibility, coverage, or package signal.
    SafetyOrCareReviewRequired,
    /// Payment or billing review required daycare attendance, eligibility, coverage, or package signal.
    PaymentOrBillingReviewRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for no opportunity reason decisions in daycare workflows.
pub enum NoOpportunityReason {
    /// Already covered daycare attendance, eligibility, coverage, or package signal.
    AlreadyCovered,
    /// Not enough attendance history daycare attendance, eligibility, coverage, or package signal.
    NotEnoughAttendanceHistory,
}

#[derive(Debug, Clone, Default)]
/// Typed policy domain value that keeps raw primitives out of daycare workflows.
pub struct Policy;

impl Policy {
    /// Returns the classify for this daycare value.
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
