use super::*;
use crate::operations::time_bucket as time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
/// Positive demand quantity.
pub struct Quantity(u32);

impl Quantity {
    /// Creates a nonzero quantity.
    pub const fn try_new(value: u32) -> std::result::Result<Self, Error> {
        if value == 0 {
            return Err(Error::ZeroQuantity);
        }
        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for Quantity {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        Self::try_new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Capacity or scheduling constraint.
pub enum Constraint {
    /// Room or suite inventory constraint.
    RoomOrSuiteAvailability,
    /// Play-yard capacity.
    PlayYardAvailability,
    /// Staff ratio.
    StaffRatio,
    /// Groomer slot capacity.
    GroomerSlotAvailability,
    /// Trainer capacity.
    TrainerAvailability,
    /// Check-in/check-out bottleneck.
    CheckInCheckoutBottleneck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Time-bucketed service demand and labor standard.
pub struct DemandUnit {
    location_id: entities::LocationId,
    service: entities::ServiceKind,
    bucket: time::Window,
    quantity: Quantity,
    labor_minutes_per_unit: labor::Minutes,
    #[builder(default)]
    constraints: Vec<Constraint>,
}

impl DemandUnit {
    /// Resort location whose demand is being evaluated.
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    /// Service line whose demand is being evaluated.
    pub fn service(&self) -> entities::ServiceKind {
        self.service.clone()
    }

    /// Time bucket covered by this demand fact.
    pub const fn bucket(&self) -> time::Window {
        self.bucket
    }

    /// Unit demand quantity.
    pub const fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Labor standard applied to each demand unit.
    pub const fn labor_minutes_per_unit(&self) -> labor::Minutes {
        self.labor_minutes_per_unit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Optimization objective.
pub enum OptimizationObjective {
    /// Reduce front-desk peak queue risk.
    ReduceFrontDeskBottleneck,
    /// Improve utilization without overbooking.
    MaximizeSafeUtilization,
    /// Reduce overstaffing cost.
    ReduceOverstaffing,
    /// Protect service quality during demand peaks.
    ProtectServiceQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Solver result for the capacity/labor recommendation input set.
pub enum SolverStatus {
    /// Inputs produced a feasible manager-review recommendation.
    Feasible,
    /// Inputs could not produce a safe recommendation and should be surfaced as review evidence.
    Infeasible {
        /// Reason the optimizer could not produce a safe recommendation.
        reason: InfeasibilityReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reason a capacity/labor input set is infeasible.
pub enum InfeasibilityReason {
    /// Not enough qualified labor exists for the requested service/window.
    InsufficientQualifiedStaff,
    /// Capacity constraints conflict with the requested demand.
    CapacityConstraintConflict,
    /// Source facts are too incomplete for recommendation authority.
    IncompleteSourceFacts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reviewable action recommended by the optimizer.
pub enum RecommendedAction {
    /// Add role coverage in minutes.
    AddRoleCoverage {
        /// Role to add.
        role: labor::Role,
        /// Minutes to add.
        minutes: labor::Minutes,
    },
    /// Remove role coverage in minutes.
    RemoveRoleCoverage {
        /// Role to remove.
        role: labor::Role,
        /// Minutes to remove.
        minutes: labor::Minutes,
    },
    /// Reassign role coverage.
    ReassignCoverage {
        /// Role to reassign.
        role: labor::Role,
        /// Minutes to reassign.
        minutes: labor::Minutes,
    },
    /// Reassign coverage from one role to another.
    ReassignRoleCoverage {
        /// Role coverage moves away from.
        from_role: labor::Role,
        /// Role coverage moves toward.
        to_role: labor::Role,
        /// Minutes to reassign.
        minutes: labor::Minutes,
    },
    /// Keep plan but flag review.
    ManagerReviewOnly,
}

impl RecommendedAction {
    /// Returns the minutes implied by this action, if it changes coverage.
    pub const fn minutes(&self) -> Option<labor::Minutes> {
        match self {
            Self::AddRoleCoverage { minutes, .. }
            | Self::RemoveRoleCoverage { minutes, .. }
            | Self::ReassignCoverage { minutes, .. }
            | Self::ReassignRoleCoverage { minutes, .. } => Some(*minutes),
            Self::ManagerReviewOnly => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Review-gated capacity/labor optimization recommendation.
pub struct OptimizationRecommendation {
    objective: OptimizationObjective,
    demand: DemandUnit,
    coverage: labor::ScheduledCoverage,
    source_evidence: Vec<crate::source::Provenance>,
    solver_status: SolverStatus,
    alternatives: Vec<RecommendedAction>,
    action: RecommendedAction,
    expected_labor_delta_minutes: labor::SignedMinutes,
    review_gate: policy::ReviewGate,
}

impl std::fmt::Debug for OptimizationRecommendation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("OptimizationRecommendation([REDACTED])")
    }
}

impl OptimizationRecommendation {
    /// Required review gate before staffing or scheduling changes.
    pub fn review_gate(&self) -> policy::ReviewGate {
        self.review_gate.clone()
    }

    /// Service demand fact that was relationship-checked against coverage before the recommendation could exist.
    pub const fn demand(&self) -> &DemandUnit {
        &self.demand
    }

    /// Scheduled coverage fact that was compared with the demand before recommendation.
    pub const fn coverage(&self) -> &labor::ScheduledCoverage {
        &self.coverage
    }

    /// Selected manager-review action; it is evidence for review, not schedule mutation authority.
    pub const fn action(&self) -> RecommendedAction {
        self.action
    }

    /// Source evidence used by the recommendation.
    pub fn source_evidence(&self) -> &[crate::source::Provenance] {
        &self.source_evidence
    }

    /// Solver status accepted for this manager-review recommendation.
    pub const fn solver_status(&self) -> SolverStatus {
        self.solver_status
    }

    /// Alternative recommendations retained for manager review.
    pub fn alternatives(&self) -> &[RecommendedAction] {
        &self.alternatives
    }

    /// Derived labor-minute delta implied by demand standard and current coverage.
    pub const fn expected_labor_delta_minutes(&self) -> labor::SignedMinutes {
        self.expected_labor_delta_minutes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Capacity validation failure.
pub enum Error {
    #[error("demand quantity must be greater than zero")]
    /// Zero demand would erase the optimization target.
    ZeroQuantity,
}
