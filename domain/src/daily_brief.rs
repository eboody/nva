//! Canonical domain contracts for cross-service resort daily briefs.
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

/// Snapshot boundary for daily brief contracts.
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
    /// Location id fact promoted into this daily brief contract.
    pub location_id: LocationId,
    /// Date fact promoted into this daily brief contract.
    pub date: NaiveDate,
    /// Snapshot id fact promoted into this daily brief contract.
    pub snapshot_id: snapshot::Id,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Manager-facing daily brief assembled from validated operational read models.
pub struct Resort {
    /// Operating day fact promoted into this daily brief contract.
    pub operating_day: ResortOperatingDay,
    /// Sections fact promoted into this daily brief contract.
    pub sections: Vec<Section>,
    /// Recommended actions fact promoted into this daily brief contract.
    pub recommended_actions: Vec<Action>,
    /// Risks fact promoted into this daily brief contract.
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
    /// Occupancy item surfaced for manager daily-brief triage.
    Occupancy(OccupancySnapshot),
    /// Arrivals and departures item surfaced for manager daily-brief triage.
    ArrivalsAndDepartures(ArrivalDepartureSnapshot),
    /// Labor item surfaced for manager daily-brief triage.
    Labor(LaborSnapshot),
    /// Customer follow ups item surfaced for manager daily-brief triage.
    CustomerFollowUps(Vec<CustomerFollowUp>),
    /// Pet care watchlist item surfaced for manager daily-brief triage.
    PetCareWatchlist(Vec<PetCareWatch>),
    /// Revenue opportunities item surfaced for manager daily-brief triage.
    RevenueOpportunities(Vec<RevenueOpportunity>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Occupancy/utilization snapshot used to compare booked demand with service capacity.
pub struct OccupancySnapshot {
    /// Boarding capacity fact promoted into this daily brief contract.
    pub boarding_capacity: capacity::Metric,
    /// Daycare capacity fact promoted into this daily brief contract.
    pub daycare_capacity: capacity::Metric,
    /// Grooming utilization fact promoted into this daily brief contract.
    pub grooming_utilization: capacity::Metric,
    /// Training utilization fact promoted into this daily brief contract.
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

        /// Exposes the validated scalar for serialization and adapter boundaries.
        pub const fn get(self) -> u32 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Nonzero service capacity limit used as the denominator for utilization.
    pub struct Limit(u32);

    impl Limit {
        /// Promotes boundary input into a validated daily brief domain value.
        pub const fn try_new(value: u32) -> Result<Self, LimitError> {
            if value == 0 {
                return Err(LimitError::ZeroCapacity);
            }
            Ok(Self(value))
        }

        /// Exposes the validated scalar for serialization and adapter boundaries.
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
    /// Domain vocabulary for limit error decisions in daily brief workflows.
    pub enum LimitError {
        #[error("capacity metrics require an explicit non-zero capacity limit")]
        /// Zero capacity item surfaced for manager daily-brief triage.
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

        /// Exposes the validated scalar for serialization and adapter boundaries.
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

        /// Returns this daily brief value's booked.
        pub const fn booked(&self) -> Booked {
            self.booked
        }

        /// Returns this daily brief value's capacity.
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

    /// Exposes the validated scalar for serialization and adapter boundaries.
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Check-in/check-out workload snapshot for front-desk and care-team planning.
pub struct ArrivalDepartureSnapshot {
    /// Check ins fact promoted into this daily brief contract.
    pub check_ins: Vec<entities::reservation::Id>,
    /// Check outs fact promoted into this daily brief contract.
    pub check_outs: Vec<entities::reservation::Id>,
    /// Late departure risk fact promoted into this daily brief contract.
    pub late_departure_risk: Vec<entities::reservation::Id>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Labor summary comparing scheduled staff against expected demand and risk.
pub struct LaborSnapshot {
    /// Scheduled staff count fact promoted into this daily brief contract.
    pub scheduled_staff_count: ScheduledStaffCount,
    /// Labor risk fact promoted into this daily brief contract.
    pub labor_risk: LaborRisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Staffing posture surfaced as a labor-cost and service-quality lever.
pub enum LaborRisk {
    /// Understaffed item surfaced for manager daily-brief triage.
    Understaffed,
    /// On plan item surfaced for manager daily-brief triage.
    OnPlan,
    /// Overstaffed item surfaced for manager daily-brief triage.
    Overstaffed,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Customer follow-up item generated from validated operational evidence.
pub struct CustomerFollowUp {
    /// Customer id fact promoted into this daily brief contract.
    pub customer_id: CustomerId,
    /// Business reason staff should review before proceeding.
    pub reason: FollowUpReason,
    /// Due at fact promoted into this daily brief contract.
    pub due_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for follow up reason decisions in daily brief workflows.
pub enum FollowUpReason {
    /// Missing vaccine proof item surfaced for manager daily-brief triage.
    MissingVaccineProof,
    /// Deposit not paid item surfaced for manager daily-brief triage.
    DepositNotPaid,
    /// Reservation change requested item surfaced for manager daily-brief triage.
    ReservationChangeRequested,
    /// Lead needs response item surfaced for manager daily-brief triage.
    LeadNeedsResponse,
    /// Post stay check in item surfaced for manager daily-brief triage.
    PostStayCheckIn,
    /// Review response needed item surfaced for manager daily-brief triage.
    ReviewResponseNeeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Pet care/safety watch item that protects staff handoff and manager review.
pub struct PetCareWatch {
    /// Pet receiving the grooming or care service.
    pub pet_id: PetId,
    /// Business reason staff should review before proceeding.
    pub reason: PetCareWatchReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for pet care watch reason decisions in daily brief workflows.
pub enum PetCareWatchReason {
    /// Medication due item surfaced for manager daily-brief triage.
    MedicationDue,
    /// Feeding exception item surfaced for manager daily-brief triage.
    FeedingException,
    /// Anxiety or stress flag item surfaced for manager daily-brief triage.
    AnxietyOrStressFlag,
    /// Behavior review item surfaced for manager daily-brief triage.
    BehaviorReview,
    /// Incident follow up item surfaced for manager daily-brief triage.
    IncidentFollowUp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Revenue opportunity that may justify staff follow-up without bypassing approval gates.
pub struct RevenueOpportunity {
    /// Customer id fact promoted into this daily brief contract.
    pub customer_id: Option<CustomerId>,
    /// Pet receiving the grooming or care service.
    pub pet_id: Option<PetId>,
    /// Requested service that drives scheduling and labor estimates.
    pub service: ServiceKind,
    /// Opportunity fact promoted into this daily brief contract.
    pub opportunity: RevenueOpportunityKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for revenue opportunity kind decisions in daily brief workflows.
pub enum RevenueOpportunityKind {
    /// Exit bath after boarding item surfaced for manager daily-brief triage.
    ExitBathAfterBoarding,
    /// Grooming rebooking due item surfaced for manager daily-brief triage.
    GroomingRebookingDue,
    /// Daycare package candidate item surfaced for manager daily-brief triage.
    DaycarePackageCandidate,
    /// Training consult candidate item surfaced for manager daily-brief triage.
    TrainingConsultCandidate,
    /// Holiday boarding waitlist fill item surfaced for manager daily-brief triage.
    HolidayBoardingWaitlistFill,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Manager-visible risk derived from occupancy, labor, customer, care, or revenue evidence.
pub enum Risk {
    /// Capacity constraint item surfaced for manager daily-brief triage.
    CapacityConstraint {
        /// Requested service that drives scheduling and labor estimates.
        service: ServiceKind,
    },
    /// Labor mismatch item surfaced for manager daily-brief triage.
    LaborMismatch {
        /// Risk fact promoted into this daily brief contract.
        risk: LaborRisk,
    },
    /// Customer experience risk item surfaced for manager daily-brief triage.
    CustomerExperienceRisk {
        /// Observation fact promoted into this daily brief contract.
        observation: operations::operational::Observation,
    },
    /// Pet safety or care risk item surfaced for manager daily-brief triage.
    PetSafetyOrCareRisk {
        /// Observation fact promoted into this daily brief contract.
        observation: operations::operational::Observation,
    },
    /// Revenue leakage item surfaced for manager daily-brief triage.
    RevenueLeakage {
        /// Observation fact promoted into this daily brief contract.
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
    /// Create internal task item surfaced for manager daily-brief triage.
    CreateInternalTask {
        /// Recommendation fact promoted into this daily brief contract.
        recommendation: operations::operational::Recommendation,
    },
    /// Draft customer message item surfaced for manager daily-brief triage.
    DraftCustomerMessage {
        /// Customer id fact promoted into this daily brief contract.
        customer_id: CustomerId,
        /// Business reason staff should review before proceeding.
        reason: FollowUpReason,
    },
    /// Escalate to manager item surfaced for manager daily-brief triage.
    EscalateToManager {
        /// Business reason staff should review before proceeding.
        reason: operations::operational::Observation,
    },
    /// Suggest schedule review item surfaced for manager daily-brief triage.
    SuggestScheduleReview {
        /// Risk fact promoted into this daily brief contract.
        risk: LaborRisk,
    },
    /// Suggest revenue follow up item surfaced for manager daily-brief triage.
    SuggestRevenueFollowUp {
        /// Opportunity fact promoted into this daily brief contract.
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
