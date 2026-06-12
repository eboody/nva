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
