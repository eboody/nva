//! Daycare front-desk readiness decisions for faster, safer check-in lanes.
//!
//! ```
//! use domain::{daycare, entities};
//! use uuid::Uuid;
//!
//! let context = daycare::front_desk::ReadinessContext::builder()
//!     .reservation_id(entities::reservation::Id(Uuid::nil()))
//!     .service(daycare::ServiceVariant::DayBoarding)
//!     .eligibility(daycare::front_desk::EligibilityReadiness::IndividualCareReady)
//!     .coverage(daycare::coverage::Decision::Sufficient)
//!     .care(daycare::front_desk::CareReadiness::Ready)
//!     .package(daycare::front_desk::PackageReadiness::NeedsFrontDeskCollection)
//!     .customer_message(daycare::front_desk::CustomerMessageReadiness::NoMessageNeeded)
//!     .build();
//! let decision = daycare::front_desk::ThroughputPolicy.evaluate(&context);
//! let ticket = daycare::front_desk::QueueTicket::new(
//!     daycare::front_desk::QueuePosition::try_new(1).unwrap(),
//!     decision,
//! );
//!
//! assert_eq!(ticket.lane(), daycare::front_desk::QueueLane::CollectionLane);
//! ```

use super::*;
use crate::{entities, policy};

positive_scalar!(
    QueuePosition,
    u16,
    QueuePositionError,
    "front-desk queue position requires at least one ticket position"
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct ReadinessContext {
    pub reservation_id: entities::reservation::Id,
    pub service: ServiceVariant,
    pub eligibility: EligibilityReadiness,
    pub coverage: coverage::Decision,
    pub care: CareReadiness,
    pub package: PackageReadiness,
    pub customer_message: CustomerMessageReadiness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EligibilityReadiness {
    GroupPlay(eligibility::GroupPlayDecision),
    IndividualCareReady,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CareReadiness {
    Ready,
    NeedsCareTeamReview { gate: policy::ReviewGate },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageReadiness {
    Ready,
    NeedsFrontDeskCollection,
    NeedsManagerReview { gate: policy::ReviewGate },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomerMessageReadiness {
    NoMessageNeeded,
    DraftNeedsApproval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadinessDecision {
    ReadyToCheckIn,
    NeedsFrontDeskCollection,
    NeedsCareTeamReview { gate: policy::ReviewGate },
    NeedsManagerReview { gate: policy::ReviewGate },
    BlockedForSafetyOrPolicy { gate: policy::ReviewGate },
}

impl ReadinessDecision {
    pub fn customer_message_gate(&self) -> Option<policy::ReviewGate> {
        match self {
            Self::ReadyToCheckIn
            | Self::NeedsFrontDeskCollection
            | Self::NeedsCareTeamReview { .. }
            | Self::NeedsManagerReview { .. }
            | Self::BlockedForSafetyOrPolicy { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueLane {
    FastLane,
    CollectionLane,
    CareTeamReviewLane,
    ManagerReviewLane,
    WaitlistOrPolicyLane,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueTicket {
    position: QueuePosition,
    decision: ReadinessDecision,
}

impl QueueTicket {
    pub const fn new(position: QueuePosition, decision: ReadinessDecision) -> Self {
        Self { position, decision }
    }

    pub const fn lane(&self) -> QueueLane {
        match self.decision {
            ReadinessDecision::ReadyToCheckIn => QueueLane::FastLane,
            ReadinessDecision::NeedsFrontDeskCollection => QueueLane::CollectionLane,
            ReadinessDecision::NeedsCareTeamReview { .. } => QueueLane::CareTeamReviewLane,
            ReadinessDecision::NeedsManagerReview { .. } => QueueLane::ManagerReviewLane,
            ReadinessDecision::BlockedForSafetyOrPolicy { .. } => QueueLane::WaitlistOrPolicyLane,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ThroughputPolicy;

impl ThroughputPolicy {
    pub fn evaluate(&self, context: &ReadinessContext) -> ReadinessDecision {
        if let CustomerMessageReadiness::DraftNeedsApproval = context.customer_message {
            return ReadinessDecision::NeedsManagerReview {
                gate: policy::ReviewGate::CustomerMessageApproval,
            };
        }
        if let PackageReadiness::NeedsManagerReview { gate } = &context.package {
            return ReadinessDecision::NeedsManagerReview { gate: gate.clone() };
        }
        if let PackageReadiness::NeedsFrontDeskCollection = context.package {
            return ReadinessDecision::NeedsFrontDeskCollection;
        }
        if let CareReadiness::NeedsCareTeamReview { gate } = &context.care {
            return ReadinessDecision::NeedsCareTeamReview { gate: gate.clone() };
        }
        if let coverage::Decision::Insufficient { gate, .. }
        | coverage::Decision::Unknown { gate } = &context.coverage
        {
            return ReadinessDecision::NeedsManagerReview { gate: gate.clone() };
        }
        match &context.eligibility {
            EligibilityReadiness::IndividualCareReady => ReadinessDecision::ReadyToCheckIn,
            EligibilityReadiness::GroupPlay(eligibility::GroupPlayDecision::Eligible {
                ..
            }) => ReadinessDecision::ReadyToCheckIn,
            EligibilityReadiness::GroupPlay(eligibility::GroupPlayDecision::NeedsStaffReview {
                gate,
                ..
            }) => ReadinessDecision::NeedsCareTeamReview { gate: gate.clone() },
            EligibilityReadiness::GroupPlay(
                eligibility::GroupPlayDecision::TemporarilySuspended { gate, .. },
            ) => ReadinessDecision::NeedsManagerReview { gate: gate.clone() },
            EligibilityReadiness::GroupPlay(eligibility::GroupPlayDecision::Ineligible {
                ..
            }) => ReadinessDecision::BlockedForSafetyOrPolicy {
                gate: policy::ReviewGate::ManagerApproval,
            },
        }
    }
}
