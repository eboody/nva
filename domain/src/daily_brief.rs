//! Manager-facing daily brief values for cross-service resort operations.
//!
//! Brief sections, occupancy, labor, revenue, watchlists, and recommended manager actions
//! are owned here rather than hidden behind broader operations vocabulary. A daily brief
//! is the manager-facing read model in the source-fact → validated-domain → workflow
//! chain: it exposes labor-cost levers such as scheduled staff count, utilization,
//! over/understaffing, follow-up queues, safety watchlists, and revenue opportunities
//! without taking live customer or staffing action on its own.
//!
//! ```
//! use domain::{daily_brief, entities};
//!
//! let brief = daily_brief::Resort {
//!     operating_day: daily_brief::ResortOperatingDay {
//!         location_id: entities::LocationId(uuid::Uuid::nil()),
//!         date: chrono::NaiveDate::from_ymd_opt(2026, 6, 18).unwrap(),
//!         snapshot_id: daily_brief::snapshot::Id::try_new("loc-1-2026-06-18").unwrap(),
//!     },
//!     sections: vec![daily_brief::Section::Labor(daily_brief::LaborSnapshot {
//!         scheduled_staff_count: daily_brief::ScheduledStaffCount::new(4),
//!         labor_risk: daily_brief::LaborRisk::Understaffed,
//!     })],
//!     recommended_actions: vec![daily_brief::Action::SuggestScheduleReview {
//!         risk: daily_brief::LaborRisk::Understaffed,
//!     }],
//!     risks: vec![daily_brief::Risk::LaborMismatch {
//!         risk: daily_brief::LaborRisk::Understaffed,
//!     }],
//! };
//!
//! assert!(brief.has_manager_attention_required());
//! assert!(brief.recommended_actions[0].requires_manager_approval());
//! ```

use chrono::{DateTime, NaiveDate, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{self, CustomerId, LocationId, PetId, ServiceKind};
use crate::operations;

pub use snapshot::Id as Snapshot;

/// Snapshot identifiers used to tie a daily brief back to the source/read-model extract.
pub mod snapshot {
    use super::*;

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 120),
        derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize
        )
    )]
    pub struct Id(String);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Source snapshot key for one resort's manager brief on an operating day.
pub struct ResortOperatingDay {
    /// Resort location whose manager owns this operating-day brief.
    pub location_id: LocationId,
    /// Operating day the brief summarizes for staffing, arrivals, care, and follow-up decisions.
    pub date: NaiveDate,
    /// Source snapshot id retained so every brief item can be traced back to the read-model extract.
    pub snapshot_id: snapshot::Id,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Manager-facing daily brief assembled from validated operational read models.
pub struct Resort {
    /// Location/date/snapshot key that scopes the entire manager brief.
    pub operating_day: ResortOperatingDay,
    /// Manager-facing sections that organize occupancy, labor, customer, care, and revenue evidence.
    pub sections: Vec<Section>,
    /// Draft recommendations that may create tasks, messages, escalations, schedule reviews, or revenue follow-up.
    pub recommended_actions: Vec<Action>,
    /// Risks requiring manager attention before staff rely on the brief for operational decisions.
    pub risks: Vec<Risk>,
}

impl Resort {
    /// Returns whether risks or proposed actions require manager attention before work starts.
    pub fn has_manager_attention_required(&self) -> bool {
        self.risks.iter().any(Risk::requires_manager_attention)
            || self
                .recommended_actions
                .iter()
                .any(Action::requires_manager_approval)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Section of the daily brief that turns source/read-model evidence into manager focus.
pub enum Section {
    /// Booked-vs-capacity view showing where demand may exceed service limits.
    Occupancy(OccupancySnapshot),
    /// Check-in/check-out workload that helps front desk and care teams plan the day.
    ArrivalsAndDepartures(ArrivalDepartureSnapshot),
    /// Staffing snapshot comparing scheduled labor with expected demand.
    Labor(LaborSnapshot),
    /// Customer follow-up queue for missing proof, changes, reviews, or service recovery.
    CustomerFollowUps(Vec<CustomerFollowUp>),
    /// Pet-care watchlist for medication, feeding, behavior, or incident attention.
    PetCareWatchlist(Vec<PetCareWatch>),
    /// Revenue opportunities that need staff review before customer follow-up.
    RevenueOpportunities(Vec<RevenueOpportunity>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Occupancy/utilization snapshot used to compare booked demand with service capacity.
pub struct OccupancySnapshot {
    /// Boarding booked-vs-capacity metric used to identify occupancy pressure.
    pub boarding_capacity: capacity::Metric,
    /// Daycare booked-vs-capacity metric used for playgroup and staffing review.
    pub daycare_capacity: capacity::Metric,
    /// Grooming utilization metric used to spot schedule pressure or rebooking capacity.
    pub grooming_utilization: capacity::Metric,
    /// Training utilization metric used to plan trainer workload and consult capacity.
    pub training_utilization: capacity::Metric,
}

/// Capacity metrics used by daily briefs to expose utilization and labor-pressure signals.
pub mod capacity {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Number of booked units contributing to a capacity metric.
    pub struct Booked(u32);

    impl Booked {
        /// Assembles this daily brief value from already-validated domain parts.
        pub const fn new(value: u32) -> Self {
            Self(value)
        }

        /// Returns the checked value for storage, reporting, or adapter output.
        pub const fn get(self) -> u32 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Nonzero service capacity limit used as the denominator for utilization.
    pub struct Limit(u32);

    impl Limit {
        /// Rejects unusable daily-brief input before managers see capacity or labor metrics.
        pub const fn try_new(value: u32) -> Result<Self, LimitError> {
            if value == 0 {
                return Err(LimitError::ZeroCapacity);
            }
            Ok(Self(value))
        }

        /// Returns the checked value for storage, reporting, or adapter output.
        pub const fn get(self) -> u32 {
            self.0
        }
    }

    impl<'de> Deserialize<'de> for Limit {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Self::try_new(u32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Limit validation error that protects daily-brief ranking from empty or oversized queues.
    pub enum LimitError {
        #[error("capacity metrics require an explicit non-zero capacity limit")]
        /// Zero capacity would make utilization meaningless and blocks the metric before manager display.
        ZeroCapacity,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Capacity saturation expressed in basis points for stable BI/reporting comparisons.
    pub struct SaturationBasisPoints(u32);

    impl SaturationBasisPoints {
        /// Assembles this daily brief value from already-validated domain parts.
        pub const fn new(value: u32) -> Self {
            Self(value)
        }

        /// Returns the checked value for storage, reporting, or adapter output.
        pub const fn get(self) -> u32 {
            self.0
        }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Booked-vs-capacity metric that makes service utilization visible to managers.
    pub struct Metric {
        booked: Booked,
        capacity: Limit,
    }

    impl Metric {
        /// Assembles this daily brief value from already-validated domain parts.
        pub const fn new(booked: Booked, capacity: Limit) -> Self {
            Self { booked, capacity }
        }

        /// Returns booked units so occupancy pressure can be compared with capacity.
        pub const fn booked(&self) -> Booked {
            self.booked
        }

        /// Returns available capacity used to rank overbooking risk and labor pressure.
        pub const fn capacity(&self) -> Limit {
            self.capacity
        }

        /// Returns the saturation basis points for this daily brief value.
        pub fn saturation_basis_points(&self) -> SaturationBasisPoints {
            SaturationBasisPoints::new(
                self.booked.get().saturating_mul(10_000) / self.capacity.get(),
            )
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Count of scheduled staff used to reason about over/understaffing labor risk.
pub struct ScheduledStaffCount(u16);

impl ScheduledStaffCount {
    /// Assembles this daily brief value from already-validated domain parts.
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// Returns the checked value for storage, reporting, or adapter output.
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Check-in/check-out workload snapshot for front-desk and care-team planning.
pub struct ArrivalDepartureSnapshot {
    /// Reservations expected to arrive, driving front-desk preparation and care-team intake labor.
    pub check_ins: Vec<entities::reservation::Id>,
    /// Reservations expected to depart, driving pickup, billing, belongings, and room turnover work.
    pub check_outs: Vec<entities::reservation::Id>,
    /// Departures at risk of running late and affecting capacity, labor, or customer communication.
    pub late_departure_risk: Vec<entities::reservation::Id>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Labor summary comparing scheduled staff against expected demand and risk.
pub struct LaborSnapshot {
    /// Number of scheduled staff used to compare labor coverage with expected demand.
    pub scheduled_staff_count: ScheduledStaffCount,
    /// Staffing posture that tells managers whether to review coverage before the day starts.
    pub labor_risk: LaborRisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Staffing posture surfaced as a labor-cost and service-quality lever.
pub enum LaborRisk {
    /// Expected demand exceeds scheduled coverage and should trigger staffing review.
    Understaffed,
    /// Scheduled coverage appears aligned with expected demand.
    OnPlan,
    /// Scheduled coverage may exceed demand and can inform cost review or reassignment.
    Overstaffed,
    /// Labor coverage evidence is missing or unclear, so managers should verify staffing before acting.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Customer follow-up item generated from validated operational evidence.
pub struct CustomerFollowUp {
    /// Customer whose follow-up or revenue item needs staff-owned review.
    pub customer_id: CustomerId,
    /// Business reason staff should review before proceeding.
    pub reason: FollowUpReason,
    /// Deadline for completing or escalating the follow-up before it becomes stale.
    pub due_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Follow-up reason that tells managers why the daily brief surfaced a pet, customer, or revenue task.
pub enum FollowUpReason {
    /// Vaccine proof is missing and must be collected or verified before care eligibility is trusted.
    MissingVaccineProof,
    /// Deposit is unpaid, requiring billing review or customer follow-up before relying on the booking.
    DepositNotPaid,
    /// Customer requested a reservation change that staff must confirm before schedule or capacity changes.
    ReservationChangeRequested,
    /// Sales/intake lead is waiting for response and may affect conversion or capacity planning.
    LeadNeedsResponse,
    /// Post-stay check-in is due for customer experience or service recovery.
    PostStayCheckIn,
    /// Reputation response is needed but must follow review and approval boundaries.
    ReviewResponseNeeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Pet care/safety watch item that protects staff handoff and manager review.
pub struct PetCareWatch {
    /// Pet whose care/safety watch item needs staff attention.
    pub pet_id: PetId,
    /// Business reason staff should review before proceeding.
    pub reason: PetCareWatchReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Pet-care watch reason used to flag vaccination, incident, medication, temperament, or feeding attention.
pub enum PetCareWatchReason {
    /// Medication task is due and needs care-team completion evidence.
    MedicationDue,
    /// Feeding instructions or exceptions need care-team attention before normal workflow proceeds.
    FeedingException,
    /// Anxiety or stress evidence may affect handling, staffing, and customer updates.
    AnxietyOrStressFlag,
    /// Behavior evidence requires review before playgroup, handling, or customer messaging changes.
    BehaviorReview,
    /// Incident follow-up is due and may affect safety review or customer communication.
    IncidentFollowUp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Revenue opportunity that may justify staff follow-up without bypassing approval gates.
pub struct RevenueOpportunity {
    /// Customer connected to a possible revenue follow-up, if known.
    pub customer_id: Option<CustomerId>,
    /// Pet connected to the revenue opportunity, if known.
    pub pet_id: Option<PetId>,
    /// Requested service that drives scheduling and labor estimates.
    pub service: ServiceKind,
    /// Opportunity category staff should review before drafting or making a customer offer.
    pub opportunity: RevenueOpportunityKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Revenue opportunity kind used to separate package, add-on, retail, reactivation, and training work.
pub enum RevenueOpportunityKind {
    /// Boarding stay may be eligible for an exit bath offer after staff confirm service fit and timing.
    ExitBathAfterBoarding,
    /// Grooming customer may be due for rebooking, subject to schedule and customer preference review.
    GroomingRebookingDue,
    /// Daycare usage suggests package discussion, but staff must verify attendance and payment context.
    DaycarePackageCandidate,
    /// Care or behavior evidence suggests a training consult may be useful after staff review.
    TrainingConsultCandidate,
    /// Waitlist or cancellation gap may allow boarding revenue after capacity and policy review.
    HolidayBoardingWaitlistFill,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Manager-visible risk derived from occupancy, labor, customer, care, or revenue evidence.
pub enum Risk {
    /// Demand may exceed capacity for the service and should interrupt normal planning.
    CapacityConstraint {
        /// Requested service that drives scheduling and labor estimates.
        service: ServiceKind,
    },
    /// Staffing coverage may not match demand and should drive schedule review.
    LaborMismatch {
        /// Labor-risk value that explains why schedule review is recommended.
        risk: LaborRisk,
    },
    /// Customer-experience signal needing manager review before follow-up.
    CustomerExperienceRisk {
        /// Source-backed observation explaining the risk for reviewers.
        observation: operations::operational::Observation,
    },
    /// Pet safety or care signal that should route to care/manager review.
    PetSafetyOrCareRisk {
        /// Source-backed care observation explaining the safety concern.
        observation: operations::operational::Observation,
    },
    /// Revenue signal that may deserve follow-up but stays review-gated.
    RevenueLeakage {
        /// Source-backed revenue observation for manager review.
        observation: operations::operational::Observation,
    },
}

impl Risk {
    /// Returns whether this risk should interrupt normal staff workflow for manager review.
    pub fn requires_manager_attention(&self) -> bool {
        matches!(
            self,
            Self::CapacityConstraint { .. }
                | Self::LaborMismatch {
                    risk: LaborRisk::Understaffed
                }
                | Self::CustomerExperienceRisk { .. }
                | Self::PetSafetyOrCareRisk { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Proposed action that remains draft/recommendation until the workflow gate approves it.
pub enum Action {
    /// Create a staff task from brief evidence without taking external customer action.
    CreateInternalTask {
        /// Recommendation text/evidence used to create the internal task.
        recommendation: operations::operational::Recommendation,
    },
    /// Draft a customer message only; approval and channel rules still control sending.
    DraftCustomerMessage {
        /// Customer who would receive the drafted follow-up after approval.
        customer_id: CustomerId,
        /// Business reason staff should review before proceeding.
        reason: FollowUpReason,
    },
    /// Escalate source-backed concern to a manager before staff act.
    EscalateToManager {
        /// Business reason staff should review before proceeding.
        reason: operations::operational::Observation,
    },
    /// Suggest manager review of staffing or schedule coverage.
    SuggestScheduleReview {
        /// Labor risk that justifies schedule review.
        risk: LaborRisk,
    },
    /// Suggest review-gated revenue follow-up rather than direct sales outreach.
    SuggestRevenueFollowUp {
        /// Revenue opportunity to verify before any customer-facing offer.
        opportunity: RevenueOpportunityKind,
    },
}

impl Action {
    /// Returns whether this action affects staffing/escalation enough to require approval.
    pub fn requires_manager_approval(&self) -> bool {
        matches!(
            self,
            Self::EscalateToManager { .. } | Self::SuggestScheduleReview { .. }
        )
    }
}
