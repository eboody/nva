//! Staff tasking decisions for resort labor assignment and closeout.
//!
//! Staff work is shared across service lines. These types turn validated daily-brief,
//! reservation, pet, and workflow signals into assignable labor, making the cost levers
//! explicit: what work exists, who/what role owns it, priority, due time, and completion
//! evidence.

use bon::Builder;
use chrono::{DateTime, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use crate::daily_brief::{self, FollowUpReason};
use crate::entities::{self, CustomerId, LocationId, PetId, StaffId};
use crate::workflow::task as workflow_task;

/// Staff-task completion evidence retained for audit and BI reconciliation.
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
    /// Evidence that a staff task was completed, retained for audit and BI reconciliation.
    pub struct Evidence(String);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Staff task assembled from source-backed resort work so managers can route labor without guessing.
pub struct Task {
    /// Resort location whose team owns this task.
    pub location_id: LocationId,
    /// Type of labor staff must perform or review.
    pub kind: task::Kind,
    /// Staff-visible task title used in work queues and manager briefs.
    pub title: workflow_task::Title,
    /// Current workflow state controlling whether staff can act, wait, or review.
    pub status: task::Status,
    /// Urgency used to rank the labor queue for leads and managers.
    pub priority: task::Priority,
    /// Time by which the resort work should be completed or escalated.
    pub due_at: DateTime<Utc>,
    /// Staff member or labor role currently responsible for the work.
    pub assignment: task::Assignment,
    /// Source record or workflow event that explains why this task exists.
    pub source: task::Source,
    /// Optional closeout note proving the task was finished before reports treat it as done.
    pub completion_evidence: Option<completion_evidence::Evidence>,
}

impl Task {
    /// Returns whether priority, status, or safety-sensitive kind should surface to managers.
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

    /// Marks the task completed with auditable evidence for workflow/read-model closeout.
    pub fn complete_with(mut self, evidence: completion_evidence::Evidence) -> Self {
        self.status = task::Status::Completed;
        self.completion_evidence = Some(evidence);
        self
    }
}

/// Staff-task vocabulary for routing work, ranking urgency, and preserving source proof.
pub mod task {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Type of labor a staff task represents across check-in, care, cleanup, and follow-up.
    pub enum Kind {
        /// Labor to prepare a reservation for arrival, documents, room, and front-desk handoff.
        CheckInPrep {
            /// Reservation that requires this staff preparation or closeout work.
            reservation_id: entities::reservation::Id,
        },
        /// Labor to prepare pickup, belongings, invoice, and checkout communication.
        CheckOutPrep {
            /// Reservation that requires this staff preparation or closeout work.
            reservation_id: entities::reservation::Id,
        },
        /// Pet-care labor for feeding instructions or exceptions that staff must complete.
        Feeding {
            /// Pet whose care task needs staff handling.
            pet_id: PetId,
        },
        /// Medication labor that requires reviewed instructions and completion evidence.
        MedicationAdministration {
            /// Pet whose medication task needs reviewed instructions and completion evidence.
            pet_id: PetId,
        },
        /// Labor for temperament or group-play assessment before daycare assignment.
        PlaygroupAssessment {
            /// Pet whose playgroup assessment needs temperament or eligibility review.
            pet_id: PetId,
        },
        /// Labor for kennel, room, or run turnover tied to a reservation.
        CleaningTurnover {
            /// Reservation that requires this staff preparation or closeout work.
            reservation_id: entities::reservation::Id,
        },
        /// Labor to prepare a customer-safe daily update draft from care evidence.
        DailyUpdateDraft {
            /// Reservation that requires this staff preparation or closeout work.
            reservation_id: entities::reservation::Id,
        },
        /// Labor to review document evidence before compliance or care workflows trust it.
        DocumentReview {
            /// Pet whose document evidence must be reviewed before staff trust it.
            pet_id: PetId,
        },
        /// Labor to investigate, document, or communicate about a safety/customer incident.
        IncidentFollowUp {
            /// Pet connected to the incident follow-up labor.
            pet_id: PetId,
        },
        /// Labor to contact a customer for missing proof, changes, review response, or service recovery.
        CustomerFollowUp {
            /// Customer whose follow-up should be routed to staff.
            customer_id: CustomerId,
            /// Business reason staff should review before proceeding.
            reason: FollowUpReason,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Current workflow state for an assignable staff task.
    pub enum Status {
        /// Work is visible in the queue but not yet being handled.
        Open,
        /// A staff member or role is actively handling the work.
        InProgress,
        /// Work cannot proceed until missing proof, policy, or approval is resolved.
        Blocked,
        /// Manager must review the task before staff treat it as complete.
        NeedsManagerReview,
        /// Evidence says the resort work is complete and can feed reports.
        Completed,
        /// Staff task was cancelled or suppressed before completion and should not count as done labor.
        Cancelled,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    /// Priority level used to sequence staff labor and manager attention.
    pub enum Priority {
        /// Low urgency task that can wait behind normal and safety-sensitive labor.
        Low,
        /// Routine resort work that can follow normal queue order.
        Normal,
        /// High urgency task that should be handled ahead of routine resort work.
        High,
        /// Safety, customer-trust, or operations issue that should jump the queue.
        Critical,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Staff assignment state used to distinguish scheduled coverage from backup or inactive labor.
    pub enum Assignment {
        /// No staff member or role owns this work yet.
        Unassigned,
        /// Named staff member owns the task.
        Staff(StaffId),
        /// Labor role owns the task until a person claims it.
        Role(super::Role),
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Staff source system retained so labor records can be reconciled with provider authority.
    pub enum Source {
        /// Reservation record participating in the workflow.
        Reservation(entities::reservation::Id),
        /// Pet record participating in the workflow.
        Pet(PetId),
        /// Customer record participating in the workflow.
        Customer(CustomerId),
        /// Daily brief snapshot raised this task for staff review.
        DailyBrief(daily_brief::snapshot::Id),
        /// Workflow event raised this task for staff review.
        WorkflowEvent(crate::workflow::EventId),
        /// Staff created the task directly outside automated source ingestion.
        StaffCreated,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Resort labor role that can own or be assigned a staff task.
pub enum Role {
    /// Front desk team handling check-in, checkout, customer, or document work.
    FrontDesk,
    /// Kennel technician team handling pet care, feeding, medication, or cleanup work.
    KennelTechnician,
    /// Groomer handling grooming preparation, service, or follow-up work.
    Groomer,
    /// Trainer handling training assignment, progress, package, or follow-up work.
    Trainer,
    /// Lead staff member triaging work before manager escalation.
    LeadStaff,
    /// Manager accountable for approvals, exceptions, and queue escalation.
    Manager,
}
