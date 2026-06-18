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
/// Typed readiness context domain value that keeps raw primitives out of daycare workflows.
pub struct ReadinessContext {
    /// Reservation id fact promoted into this daycare contract.
    pub reservation_id: entities::reservation::Id,
    /// Requested service that drives scheduling and labor estimates.
    pub service: ServiceVariant,
    /// Eligibility fact promoted into this daycare contract.
    pub eligibility: EligibilityReadiness,
    /// Coverage fact promoted into this daycare contract.
    pub coverage: coverage::Decision,
    /// Care fact promoted into this daycare contract.
    pub care: CareReadiness,
    /// Package fact promoted into this daycare contract.
    pub package: PackageReadiness,
    /// Customer message fact promoted into this daycare contract.
    pub customer_message: CustomerMessageReadiness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for eligibility readiness decisions in daycare workflows.
pub enum EligibilityReadiness {
    /// Group-play add-on or accommodation feature.
    GroupPlay(eligibility::GroupPlayDecision),
    /// Individual care ready daycare attendance, eligibility, coverage, or package signal.
    IndividualCareReady,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for care readiness decisions in daycare workflows.
pub enum CareReadiness {
    /// Ready daycare attendance, eligibility, coverage, or package signal.
    Ready,
    /// Gate fact promoted into this daycare contract.
    NeedsCareTeamReview {
        /// Gate carried by this variant.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for package readiness decisions in daycare workflows.
pub enum PackageReadiness {
    /// Ready daycare attendance, eligibility, coverage, or package signal.
    Ready,
    /// Needs front desk collection daycare attendance, eligibility, coverage, or package signal.
    NeedsFrontDeskCollection,
    /// Gate fact promoted into this daycare contract.
    NeedsManagerReview {
        /// Gate carried by this variant.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for customer message readiness decisions in daycare workflows.
pub enum CustomerMessageReadiness {
    /// No message needed daycare attendance, eligibility, coverage, or package signal.
    NoMessageNeeded,
    /// Draft needs approval daycare attendance, eligibility, coverage, or package signal.
    DraftNeedsApproval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for readiness decision decisions in daycare workflows.
pub enum ReadinessDecision {
    /// Ready to check in daycare attendance, eligibility, coverage, or package signal.
    ReadyToCheckIn,
    /// Needs front desk collection daycare attendance, eligibility, coverage, or package signal.
    NeedsFrontDeskCollection,
    /// Gate fact promoted into this daycare contract.
    NeedsCareTeamReview {
        /// Gate carried by this variant.
        gate: policy::ReviewGate,
    },
    /// Gate fact promoted into this daycare contract.
    NeedsManagerReview {
        /// Gate carried by this variant.
        gate: policy::ReviewGate,
    },
    /// Gate fact promoted into this daycare contract.
    BlockedForSafetyOrPolicy {
        /// Gate carried by this variant.
        gate: policy::ReviewGate,
    },
}

impl ReadinessDecision {
    /// Returns the customer message gate for this daycare value.
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
/// Domain vocabulary for queue lane decisions in daycare workflows.
pub enum QueueLane {
    /// Fast lane daycare attendance, eligibility, coverage, or package signal.
    FastLane,
    /// Collection lane daycare attendance, eligibility, coverage, or package signal.
    CollectionLane,
    /// Care team review lane daycare attendance, eligibility, coverage, or package signal.
    CareTeamReviewLane,
    /// Manager review lane daycare attendance, eligibility, coverage, or package signal.
    ManagerReviewLane,
    /// Waitlist or policy lane daycare attendance, eligibility, coverage, or package signal.
    WaitlistOrPolicyLane,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed queue ticket domain value that keeps raw primitives out of daycare workflows.
pub struct QueueTicket {
    position: QueuePosition,
    decision: ReadinessDecision,
}

impl QueueTicket {
    /// Assembles this daycare value from already-validated domain parts.
    pub const fn new(position: QueuePosition, decision: ReadinessDecision) -> Self {
        Self { position, decision }
    }

    /// Returns this daycare value's lane.
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
/// Typed throughput policy domain value that keeps raw primitives out of daycare workflows.
pub struct ThroughputPolicy;

impl ThroughputPolicy {
    /// Returns the evaluate for this daycare value.
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
