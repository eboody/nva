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
/// Combined daycare check-in evidence used to route a front-desk queue ticket.
pub struct ReadinessContext {
    /// Reservation being prepared for check-in or staff review.
    pub reservation_id: entities::reservation::Id,
    /// Requested service that drives scheduling and labor estimates.
    pub service: ServiceVariant,
    /// Eligibility readiness for the requested daycare care mode.
    pub eligibility: EligibilityReadiness,
    /// Staffing coverage state that may block or route check-in.
    pub coverage: coverage::Decision,
    /// Care-team readiness for special handling, medical, or behavior review.
    pub care: CareReadiness,
    /// Package/payment readiness controlling collection or manager review at check-in.
    pub package: PackageReadiness,
    /// Customer-message approval state for any drafted daycare communication.
    pub customer_message: CustomerMessageReadiness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Eligibility readiness category used by front-desk routing.
pub enum EligibilityReadiness {
    /// Group-play add-on or accommodation feature.
    GroupPlay(eligibility::GroupPlayDecision),
    /// Individual-care service does not need group-play clearance for check-in.
    IndividualCareReady,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Care-team readiness state for daycare check-in.
pub enum CareReadiness {
    /// Care evidence is clear enough for check-in to proceed.
    Ready,
    /// Human review gate required before this readiness state can proceed.
    NeedsCareTeamReview {
        /// Gate carried by this variant.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Package/payment readiness state used to route daycare front-desk work.
pub enum PackageReadiness {
    /// Care evidence is clear enough for check-in to proceed.
    Ready,
    /// Front desk must collect payment, package visits, or missing account information.
    NeedsFrontDeskCollection,
    /// Human review gate required before this readiness state can proceed.
    NeedsManagerReview {
        /// Gate carried by this variant.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Approval status for daycare customer-message drafts.
pub enum CustomerMessageReadiness {
    /// No customer-facing message is required for this check-in path.
    NoMessageNeeded,
    /// A drafted customer message must be approved before it is sent or used.
    DraftNeedsApproval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Front-desk routing outcome for a daycare check-in ticket.
pub enum ReadinessDecision {
    /// Ticket can move through the fast lane without extra collection or review.
    ReadyToCheckIn,
    /// Front desk must collect payment, package visits, or missing account information.
    NeedsFrontDeskCollection,
    /// Human review gate required before this readiness state can proceed.
    NeedsCareTeamReview {
        /// Gate carried by this variant.
        gate: policy::ReviewGate,
    },
    /// Human review gate required before this readiness state can proceed.
    NeedsManagerReview {
        /// Gate carried by this variant.
        gate: policy::ReviewGate,
    },
    /// Human review gate required before this readiness state can proceed.
    BlockedForSafetyOrPolicy {
        /// Gate carried by this variant.
        gate: policy::ReviewGate,
    },
}

impl ReadinessDecision {
    /// Returns any approval gate tied specifically to customer-message handling for this decision.
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
/// Physical or operational lane used to sort daycare front-desk work.
pub enum QueueLane {
    /// Queue lane for check-ins with all readiness gates clear.
    FastLane,
    /// Queue lane for tickets needing payment or package collection.
    CollectionLane,
    /// Queue lane for tickets needing care-team or behavior review.
    CareTeamReviewLane,
    /// Queue lane for manager approval before check-in proceeds.
    ManagerReviewLane,
    /// Queue lane for blocked, ineligible, or policy-limited attendance paths.
    WaitlistOrPolicyLane,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Front-desk queue ticket pairing a position with the readiness decision that determines lane routing.
pub struct QueueTicket {
    position: QueuePosition,
    decision: ReadinessDecision,
}

impl QueueTicket {
    /// Creates a queue ticket from a validated position and readiness decision.
    pub const fn new(position: QueuePosition, decision: ReadinessDecision) -> Self {
        Self { position, decision }
    }

    /// Maps the readiness decision to the operational queue lane staff should use.
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
/// Deterministic policy for turning daycare readiness evidence into front-desk routing.
pub struct ThroughputPolicy;

impl ThroughputPolicy {
    /// Evaluates message, package, care, coverage, and eligibility gates in front-desk priority order.
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
