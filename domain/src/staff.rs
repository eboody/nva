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
pub struct Task {
    pub location_id: LocationId,
    pub kind: task::Kind,
    pub title: workflow_task::Title,
    pub status: task::Status,
    pub priority: task::Priority,
    pub due_at: DateTime<Utc>,
    pub assignment: task::Assignment,
    pub source: task::Source,
    pub completion_evidence: Option<completion_evidence::Evidence>,
}

impl Task {
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

    pub fn complete_with(mut self, evidence: completion_evidence::Evidence) -> Self {
        self.status = task::Status::Completed;
        self.completion_evidence = Some(evidence);
        self
    }
}

pub mod task {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Kind {
        CheckInPrep {
            reservation_id: entities::reservation::Id,
        },
        CheckOutPrep {
            reservation_id: entities::reservation::Id,
        },
        Feeding {
            pet_id: PetId,
        },
        MedicationAdministration {
            pet_id: PetId,
        },
        PlaygroupAssessment {
            pet_id: PetId,
        },
        CleaningTurnover {
            reservation_id: entities::reservation::Id,
        },
        DailyUpdateDraft {
            reservation_id: entities::reservation::Id,
        },
        DocumentReview {
            pet_id: PetId,
        },
        IncidentFollowUp {
            pet_id: PetId,
        },
        CustomerFollowUp {
            customer_id: CustomerId,
            reason: FollowUpReason,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Status {
        Open,
        InProgress,
        Blocked,
        NeedsManagerReview,
        Completed,
        Cancelled,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub enum Priority {
        Low,
        Normal,
        High,
        Critical,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Assignment {
        Unassigned,
        Staff(StaffId),
        Role(super::Role),
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Source {
        Reservation(entities::reservation::Id),
        Pet(PetId),
        Customer(CustomerId),
        DailyBrief(daily_brief::snapshot::Id),
        WorkflowEvent(crate::workflow::EventId),
        StaffCreated,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    FrontDesk,
    KennelTechnician,
    Groomer,
    Trainer,
    LeadStaff,
    Manager,
}
