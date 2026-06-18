//! Canonical domain contracts for staff tasking and assignment.
//!
//! Staff work is shared across service lines, so `operations` re-exports these
//! types only as a deprecated compatibility shim.

use bon::Builder;
use chrono::{DateTime, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use crate::daily_brief::{self, FollowUpReason};
use crate::entities::{self, CustomerId, LocationId, PetId, StaffId};
use crate::workflow::task as workflow_task;

/// Completion evidence boundary for staff contracts.
pub mod completion_evidence {
    use super::*;

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 500),
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
    pub struct Evidence(String);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Typed task domain value that keeps raw primitives out of staff workflows.
pub struct Task {
    /// Location id fact promoted into this staff contract.
    pub location_id: LocationId,
    /// Kind fact promoted into this staff contract.
    pub kind: task::Kind,
    /// Title fact promoted into this staff contract.
    pub title: workflow_task::Title,
    /// Status fact promoted into this staff contract.
    pub status: task::Status,
    /// Priority fact promoted into this staff contract.
    pub priority: task::Priority,
    /// Due at fact promoted into this staff contract.
    pub due_at: DateTime<Utc>,
    /// Assignment fact promoted into this staff contract.
    pub assignment: task::Assignment,
    /// Source fact promoted into this staff contract.
    pub source: task::Source,
    /// Completion evidence fact promoted into this staff contract.
    pub completion_evidence: Option<completion_evidence::Evidence>,
}

impl Task {
    /// Returns the requires manager attention for this staff value.
    pub fn requires_manager_attention(&self) -> bool {
        matches!(
            self.status,
            task::Status::Blocked | task::Status::NeedsManagerReview
        ) || matches!(
            self.priority,
            task::Priority::High | task::Priority::Critical
        ) || matches!(
            self.kind,
            task::Kind::IncidentFollowUp { .. }
                | task::Kind::MedicationAdministration { .. }
                | task::Kind::DocumentReview { .. }
        )
    }

    /// Returns the complete with for this staff value.
    pub fn complete_with(mut self, evidence: completion_evidence::Evidence) -> Self {
        self.status = task::Status::Completed;
        self.completion_evidence = Some(evidence);
        self
    }
}

/// Task boundary for staff contracts.
pub mod task {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for kind decisions in staff workflows.
    pub enum Kind {
        /// Check in prep staff role, schedule, or labor-planning signal.
        CheckInPrep {
            /// Reservation id fact promoted into this staff contract.
            reservation_id: entities::reservation::Id,
        },
        /// Check out prep staff role, schedule, or labor-planning signal.
        CheckOutPrep {
            /// Reservation id fact promoted into this staff contract.
            reservation_id: entities::reservation::Id,
        },
        /// Feeding staff role, schedule, or labor-planning signal.
        Feeding {
            /// Pet receiving the grooming or care service.
            pet_id: PetId,
        },
        /// Medication service that requires care instructions.
        MedicationAdministration {
            /// Pet receiving the grooming or care service.
            pet_id: PetId,
        },
        /// Playgroup assessment staff role, schedule, or labor-planning signal.
        PlaygroupAssessment {
            /// Pet receiving the grooming or care service.
            pet_id: PetId,
        },
        /// Cleaning turnover staff role, schedule, or labor-planning signal.
        CleaningTurnover {
            /// Reservation id fact promoted into this staff contract.
            reservation_id: entities::reservation::Id,
        },
        /// Daily update draft staff role, schedule, or labor-planning signal.
        DailyUpdateDraft {
            /// Reservation id fact promoted into this staff contract.
            reservation_id: entities::reservation::Id,
        },
        /// Document review staff role, schedule, or labor-planning signal.
        DocumentReview {
            /// Pet receiving the grooming or care service.
            pet_id: PetId,
        },
        /// Incident follow up staff role, schedule, or labor-planning signal.
        IncidentFollowUp {
            /// Pet receiving the grooming or care service.
            pet_id: PetId,
        },
        /// Customer follow up staff role, schedule, or labor-planning signal.
        CustomerFollowUp {
            /// Customer id fact promoted into this staff contract.
            customer_id: CustomerId,
            /// Business reason staff should review before proceeding.
            reason: FollowUpReason,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Normalized reservation states observed during source-data ingestion.
    pub enum Status {
        /// Open staff role, schedule, or labor-planning signal.
        Open,
        /// In progress staff role, schedule, or labor-planning signal.
        InProgress,
        /// Blocked staff role, schedule, or labor-planning signal.
        Blocked,
        /// Needs manager review staff role, schedule, or labor-planning signal.
        NeedsManagerReview,
        /// Completed staff role, schedule, or labor-planning signal.
        Completed,
        /// Reservation is no longer active.
        Cancelled,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    /// Domain vocabulary for priority decisions in staff workflows.
    pub enum Priority {
        /// Estimate is uncertain and may require staff confirmation.
        Low,
        /// Normal staff role, schedule, or labor-planning signal.
        Normal,
        /// Estimate is reliable enough for normal scheduling.
        High,
        /// Critical staff role, schedule, or labor-planning signal.
        Critical,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for assignment decisions in staff workflows.
    pub enum Assignment {
        /// Unassigned staff role, schedule, or labor-planning signal.
        Unassigned,
        /// Staff staff role, schedule, or labor-planning signal.
        Staff(StaffId),
        /// Role staff role, schedule, or labor-planning signal.
        Role(super::Role),
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for source decisions in staff workflows.
    pub enum Source {
        /// Reservation record participating in the workflow.
        Reservation(entities::reservation::Id),
        /// Pet record participating in the workflow.
        Pet(PetId),
        /// Customer record participating in the workflow.
        Customer(CustomerId),
        /// Daily brief staff role, schedule, or labor-planning signal.
        DailyBrief(daily_brief::snapshot::Id),
        /// Workflow event staff role, schedule, or labor-planning signal.
        WorkflowEvent(crate::workflow::EventId),
        /// Staff created staff role, schedule, or labor-planning signal.
        StaffCreated,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Kinds of related records that may be attached to source-data lineage.
pub enum Role {
    /// Front desk staff role, schedule, or labor-planning signal.
    FrontDesk,
    /// Kennel technician staff role, schedule, or labor-planning signal.
    KennelTechnician,
    /// Groomer staff role, schedule, or labor-planning signal.
    Groomer,
    /// Trainer staff role, schedule, or labor-planning signal.
    Trainer,
    /// Lead staff staff role, schedule, or labor-planning signal.
    LeadStaff,
    /// Manager staff role, schedule, or labor-planning signal.
    Manager,
}
