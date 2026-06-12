//! Canonical domain contracts for cross-service resort daily briefs.
//!
//! `operations` keeps deprecated compatibility aliases, but brief sections, occupancy,
//! labor, revenue, watchlists, and recommended manager actions are owned here.

use chrono::{DateTime, NaiveDate, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{self, CustomerId, LocationId, PetId, ServiceKind};
use crate::operations;

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
pub struct SnapshotId(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResortOperatingDay {
    pub location_id: LocationId,
    pub date: NaiveDate,
    pub snapshot_id: SnapshotId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resort {
    pub operating_day: ResortOperatingDay,
    pub sections: Vec<Section>,
    pub recommended_actions: Vec<Action>,
    pub risks: Vec<Risk>,
}

impl Resort {
    pub fn has_manager_attention_required(&self) -> bool {
        self.risks.iter().any(Risk::requires_manager_attention)
            || self
                .recommended_actions
                .iter()
                .any(Action::requires_manager_approval)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Section {
    Occupancy(OccupancySnapshot),
    ArrivalsAndDepartures(ArrivalDepartureSnapshot),
    Labor(LaborSnapshot),
    CustomerFollowUps(Vec<CustomerFollowUp>),
    PetCareWatchlist(Vec<PetCareWatch>),
    RevenueOpportunities(Vec<RevenueOpportunity>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OccupancySnapshot {
    pub boarding_capacity: CapacityMetric,
    pub daycare_capacity: CapacityMetric,
    pub grooming_utilization: CapacityMetric,
    pub training_utilization: CapacityMetric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CapacityBooked(u32);

impl CapacityBooked {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct CapacityLimit(u32);

impl CapacityLimit {
    pub const fn try_new(value: u32) -> Result<Self, CapacityLimitError> {
        if value == 0 {
            return Err(CapacityLimitError::ZeroCapacity);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for CapacityLimit {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CapacityLimitError {
    #[error("capacity metrics require an explicit non-zero capacity limit")]
    ZeroCapacity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CapacitySaturationBasisPoints(u32);

impl CapacitySaturationBasisPoints {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ScheduledStaffCount(u16);

impl ScheduledStaffCount {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacityMetric {
    booked: CapacityBooked,
    capacity: CapacityLimit,
}

impl CapacityMetric {
    pub const fn new(booked: CapacityBooked, capacity: CapacityLimit) -> Self {
        Self { booked, capacity }
    }

    pub const fn booked(&self) -> CapacityBooked {
        self.booked
    }

    pub const fn capacity(&self) -> CapacityLimit {
        self.capacity
    }

    pub fn saturation_basis_points(&self) -> CapacitySaturationBasisPoints {
        CapacitySaturationBasisPoints::new(
            self.booked.get().saturating_mul(10_000) / self.capacity.get(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrivalDepartureSnapshot {
    pub check_ins: Vec<entities::ReservationId>,
    pub check_outs: Vec<entities::ReservationId>,
    pub late_departure_risk: Vec<entities::ReservationId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaborSnapshot {
    pub scheduled_staff_count: ScheduledStaffCount,
    pub labor_risk: LaborRisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaborRisk {
    Understaffed,
    OnPlan,
    Overstaffed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomerFollowUp {
    pub customer_id: CustomerId,
    pub reason: FollowUpReason,
    pub due_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FollowUpReason {
    MissingVaccineProof,
    DepositNotPaid,
    ReservationChangeRequested,
    LeadNeedsResponse,
    PostStayCheckIn,
    ReviewResponseNeeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PetCareWatch {
    pub pet_id: PetId,
    pub reason: PetCareWatchReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PetCareWatchReason {
    MedicationDue,
    FeedingException,
    AnxietyOrStressFlag,
    BehaviorReview,
    IncidentFollowUp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevenueOpportunity {
    pub customer_id: Option<CustomerId>,
    pub pet_id: Option<PetId>,
    pub service: ServiceKind,
    pub opportunity: RevenueOpportunityKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevenueOpportunityKind {
    ExitBathAfterBoarding,
    GroomingRebookingDue,
    DaycarePackageCandidate,
    TrainingConsultCandidate,
    HolidayBoardingWaitlistFill,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Risk {
    CapacityConstraint {
        service: ServiceKind,
    },
    LaborMismatch {
        risk: LaborRisk,
    },
    CustomerExperienceRisk {
        observation: operations::OperationalObservation,
    },
    PetSafetyOrCareRisk {
        observation: operations::OperationalObservation,
    },
    RevenueLeakage {
        observation: operations::OperationalObservation,
    },
}

impl Risk {
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
pub enum Action {
    CreateInternalTask {
        recommendation: operations::OperationalRecommendation,
    },
    DraftCustomerMessage {
        customer_id: CustomerId,
        reason: FollowUpReason,
    },
    EscalateToManager {
        reason: operations::OperationalObservation,
    },
    SuggestScheduleReview {
        risk: LaborRisk,
    },
    SuggestRevenueFollowUp {
        opportunity: RevenueOpportunityKind,
    },
}

impl Action {
    pub fn requires_manager_approval(&self) -> bool {
        matches!(
            self,
            Self::EscalateToManager { .. } | Self::SuggestScheduleReview { .. }
        )
    }
}
