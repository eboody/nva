//! Canonical domain contracts for staff tasking and assignment.
//!
//! Staff work is shared across service lines, so `operations` re-exports these
//! types only as a compatibility shim.

use bon::Builder;
use chrono::{DateTime, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use crate::daily_brief::{FollowUpReason, SnapshotId};
use crate::entities::{CustomerId, LocationId, PetId, ReservationId, StaffId};
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
pub struct TaskCompletionEvidence(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct StaffTask {
    pub location_id: LocationId,
    pub kind: StaffTaskKind,
    pub title: task::Title,
    pub status: StaffTaskStatus,
    pub priority: StaffTaskPriority,
    pub due_at: DateTime<Utc>,
    pub assignment: StaffTaskAssignment,
    pub source: StaffTaskSource,
    pub completion_evidence: Option<TaskCompletionEvidence>,
}

impl StaffTask {
    pub fn requires_manager_attention(&self) -> bool {
        matches!(
            self.status,
            StaffTaskStatus::Blocked | StaffTaskStatus::NeedsManagerReview
        ) || matches!(
            self.priority,
            StaffTaskPriority::High | StaffTaskPriority::Critical
        ) || matches!(
            self.kind,
            StaffTaskKind::IncidentFollowUp { .. }
                | StaffTaskKind::MedicationAdministration { .. }
                | StaffTaskKind::DocumentReview { .. }
        )
    }

    pub fn complete_with(mut self, evidence: TaskCompletionEvidence) -> Self {
        self.status = StaffTaskStatus::Completed;
        self.completion_evidence = Some(evidence);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaffTaskKind {
    CheckInPrep {
        reservation_id: ReservationId,
    },
    CheckOutPrep {
        reservation_id: ReservationId,
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
        reservation_id: ReservationId,
    },
    DailyUpdateDraft {
        reservation_id: ReservationId,
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
pub enum StaffTaskStatus {
    Open,
    InProgress,
    Blocked,
    NeedsManagerReview,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StaffTaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaffTaskAssignment {
    Unassigned,
    Staff(StaffId),
    Role(StaffRole),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaffRole {
    FrontDesk,
    KennelTechnician,
    Groomer,
    Trainer,
    LeadStaff,
    Manager,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaffTaskSource {
    Reservation(ReservationId),
    Pet(PetId),
    Customer(CustomerId),
    DailyBrief(SnapshotId),
    WorkflowEvent(crate::workflow::EventId),
    StaffCreated,
}
