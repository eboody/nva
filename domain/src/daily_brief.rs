//! Canonical domain contracts for cross-service resort daily briefs.
//!
//! Brief sections, occupancy, labor, revenue, watchlists, and recommended manager actions
//! are owned here rather than hidden behind broader operations vocabulary.

use chrono::{DateTime, NaiveDate, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{self, CustomerId, LocationId, PetId, ServiceKind};
use crate::operations;

pub use snapshot::Id as Snapshot;

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
pub struct ResortOperatingDay {
    pub location_id: LocationId,
    pub date: NaiveDate,
    pub snapshot_id: snapshot::Id,
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
    pub boarding_capacity: capacity::Metric,
    pub daycare_capacity: capacity::Metric,
    pub grooming_utilization: capacity::Metric,
    pub training_utilization: capacity::Metric,
}

pub mod capacity {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Booked(u32);

    impl Booked {
        pub const fn new(value: u32) -> Self {
            Self(value)
        }

        pub const fn get(self) -> u32 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    pub struct Limit(u32);

    impl Limit {
        pub const fn try_new(value: u32) -> Result<Self, LimitError> {
            if value == 0 {
                return Err(LimitError::ZeroCapacity);
            }
            Ok(Self(value))
        }

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
    pub enum LimitError {
        #[error("capacity metrics require an explicit non-zero capacity limit")]
        ZeroCapacity,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct SaturationBasisPoints(u32);

    impl SaturationBasisPoints {
        pub const fn new(value: u32) -> Self {
            Self(value)
        }

        pub const fn get(self) -> u32 {
            self.0
        }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Metric {
        booked: Booked,
        capacity: Limit,
    }

    impl Metric {
        pub const fn new(booked: Booked, capacity: Limit) -> Self {
            Self { booked, capacity }
        }

        pub const fn booked(&self) -> Booked {
            self.booked
        }

        pub const fn capacity(&self) -> Limit {
            self.capacity
        }

        pub fn saturation_basis_points(&self) -> SaturationBasisPoints {
            SaturationBasisPoints::new(
                self.booked.get().saturating_mul(10_000) / self.capacity.get(),
            )
        }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrivalDepartureSnapshot {
    pub check_ins: Vec<entities::reservation::Id>,
    pub check_outs: Vec<entities::reservation::Id>,
    pub late_departure_risk: Vec<entities::reservation::Id>,
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
        observation: operations::operational::Observation,
    },
    PetSafetyOrCareRisk {
        observation: operations::operational::Observation,
    },
    RevenueLeakage {
        observation: operations::operational::Observation,
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
        recommendation: operations::operational::Recommendation,
    },
    DraftCustomerMessage {
        customer_id: CustomerId,
        reason: FollowUpReason,
    },
    EscalateToManager {
        reason: operations::operational::Observation,
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
