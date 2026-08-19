//! Daycare front-desk readiness decisions for faster, safer check-in lanes.
//!
//! ```
//! use domain::{daycare, entities};
//! use uuid::Uuid;
//!
//! let context = daycare::front_desk::ReadinessContext::builder()
//!     .reservation_id(entities::reservation::Id::new(uuid::Uuid::from_u128(1)))
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
use crate::policy;

positive_scalar!(
    QueuePosition,
    u16,
    QueuePositionError,
    "front-desk queue position requires at least one ticket position"
);

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
    /// Care-team medical, behavior, and handling evidence has no unresolved check-in blocker.
    ///
    /// Package/payment and manager policy readiness are represented by `PackageReadiness`.
    Ready,
    /// Care team must clear medical, behavior, or handling evidence before check-in advances.
    ///
    /// This gate is not the package/payment or manager policy readiness gate.
    NeedsCareTeamReview {
        /// Specific care-team gate that must clear before daycare check-in advances.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Package/payment readiness state used to route daycare front-desk work.
pub enum PackageReadiness {
    /// Package and payment state is clear enough for front desk to skip collection.
    Ready,
    /// Front desk must collect payment, package visits, or missing account information.
    NeedsFrontDeskCollection,
    /// Manager must review package, payment, or policy ambiguity before check-in advances.
    NeedsManagerReview {
        /// Specific gate the responsible team must clear before daycare check-in advances.
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
    /// Care team must clear medical, behavior, or handling evidence before check-in advances.
    NeedsCareTeamReview {
        /// Specific gate the responsible team must clear before daycare check-in advances.
        gate: policy::ReviewGate,
    },
    /// Manager must clear operational or package/payment policy before check-in advances.
    NeedsManagerReview {
        /// Specific gate the responsible team must clear before daycare check-in advances.
        gate: policy::ReviewGate,
    },
    /// Safety or policy block prevents check-in until the named gate is resolved.
    BlockedForSafetyOrPolicy {
        /// Specific gate the responsible team must clear before daycare check-in advances.
        gate: policy::ReviewGate,
    },
}

impl ReadinessDecision {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Front-desk queue ticket pairing a position with the readiness decision that determines lane routing.
pub struct QueueTicket {
    position: QueuePosition,
    decision: ReadinessDecision,
}

impl QueueTicket {}

#[derive(Debug, Clone, Default)]
/// Deterministic policy for turning daycare readiness evidence into front-desk routing.
pub struct ThroughputPolicy;

impl ThroughputPolicy {}
