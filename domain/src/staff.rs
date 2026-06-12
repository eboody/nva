//! Canonical domain contracts for staff tasking and assignment.
//!
//! Staff work is shared across service lines, so `operations` re-exports these
//! types only as a deprecated compatibility shim.

use bon::Builder;
use chrono::{DateTime, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use crate::daily_brief::{FollowUpReason, SnapshotId};
use crate::entities::{self, CustomerId, LocationId, PetId, StaffId};
use crate::workflow::task;

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
pub struct CompletionEvidence(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Task {
    pub location_id: LocationId,
    pub kind: TaskKind,
    pub title: task::Title,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub due_at: DateTime<Utc>,
    pub assignment: TaskAssignment,
    pub source: TaskSource,
    pub completion_evidence: Option<CompletionEvidence>,
}

impl Task {
    pub fn requires_manager_attention(&self) -> bool {
        matches!(
            self.status,
            TaskStatus::Blocked | TaskStatus::NeedsManagerReview
        ) || matches!(self.priority, TaskPriority::High | TaskPriority::Critical)
            || matches!(
                self.kind,
                TaskKind::IncidentFollowUp { .. }
                    | TaskKind::MedicationAdministration { .. }
                    | TaskKind::DocumentReview { .. }
            )
    }

    pub fn complete_with(mut self, evidence: CompletionEvidence) -> Self {
        self.status = TaskStatus::Completed;
        self.completion_evidence = Some(evidence);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskKind {
    CheckInPrep {
        reservation_id: entities::ReservationId,
    },
    CheckOutPrep {
        reservation_id: entities::ReservationId,
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
        reservation_id: entities::ReservationId,
    },
    DailyUpdateDraft {
        reservation_id: entities::ReservationId,
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
pub enum TaskStatus {
    Open,
    InProgress,
    Blocked,
    NeedsManagerReview,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskAssignment {
    Unassigned,
    Staff(StaffId),
    Role(Role),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskSource {
    Reservation(entities::ReservationId),
    Pet(PetId),
    Customer(CustomerId),
    DailyBrief(SnapshotId),
    WorkflowEvent(crate::workflow::EventId),
    StaffCreated,
}
