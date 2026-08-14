//! Staff tasking decisions for resort labor assignment and closeout.
//!
//! Staff work is shared across service lines. These types turn validated daily-brief,
//! reservation, pet, and workflow signals into assignable labor, making the cost levers
//! explicit: what work exists, who/what role owns it, priority, due time, and completion
//! evidence.

use chrono::{DateTime, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Deserializer, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
/// Staff-task aggregate construction and rehydration failures.
pub enum TaskError {
    #[error("staff task location id is required")]
    /// Represents the `LocationIdRequired` semantic case.
    LocationIdRequired,
    #[error("staff task kind is required")]
    /// Represents the `KindRequired` semantic case.
    KindRequired,
    #[error("staff task title is required")]
    /// Represents the `TitleRequired` semantic case.
    TitleRequired,
    #[error("staff task status is required")]
    /// Represents the `StatusRequired` semantic case.
    StatusRequired,
    #[error("staff task priority is required")]
    /// Represents the `PriorityRequired` semantic case.
    PriorityRequired,
    #[error("staff task due time is required")]
    /// Represents the `DueAtRequired` semantic case.
    DueAtRequired,
    #[error("staff task assignment is required")]
    /// Represents the `AssignmentRequired` semantic case.
    AssignmentRequired,
    #[error("staff task source is required")]
    /// Represents the `SourceRequired` semantic case.
    SourceRequired,
    #[error("completed staff task requires completion evidence")]
    /// Represents the `CompletedRequiresCompletionEvidence` semantic case.
    CompletedRequiresCompletionEvidence,
    #[error("only completed staff tasks may carry completion evidence")]
    /// Represents the `OnlyCompletedTasksMayCarryCompletionEvidence` semantic case.
    OnlyCompletedTasksMayCarryCompletionEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Staff task assembled from source-backed resort work so managers can route labor without guessing.
pub struct Task {
    /// Resort location whose team owns this task.
    location_id: LocationId,
    /// Type of labor staff must perform or review.
    kind: task::Kind,
    /// Staff-visible task title used in work queues and manager briefs.
    title: workflow_task::Title,
    /// Current workflow state controlling whether staff can act, wait, or review.
    status: task::Status,
    /// Urgency used to rank the labor queue for leads and managers.
    priority: task::Priority,
    /// Time by which the resort work should be completed or escalated.
    due_at: DateTime<Utc>,
    /// Staff member or labor role currently responsible for the work.
    assignment: task::Assignment,
    /// Source record or workflow event that explains why this task exists.
    source: task::Source,
    /// Optional closeout note proving the task was finished before reports treat it as done.
    completion_evidence: Option<completion_evidence::Evidence>,
}

#[derive(Deserialize)]
struct RawTask {
    location_id: LocationId,
    kind: task::Kind,
    title: workflow_task::Title,
    status: task::Status,
    priority: task::Priority,
    due_at: DateTime<Utc>,
    assignment: task::Assignment,
    source: task::Source,
    completion_evidence: Option<completion_evidence::Evidence>,
}

impl RawTask {
    fn try_into_task(self) -> std::result::Result<Task, TaskError> {
        if self.status == task::Status::Completed && self.completion_evidence.is_none() {
            return Err(TaskError::CompletedRequiresCompletionEvidence);
        }
        if self.status != task::Status::Completed && self.completion_evidence.is_some() {
            return Err(TaskError::OnlyCompletedTasksMayCarryCompletionEvidence);
        }
        Ok(Task {
            location_id: self.location_id,
            kind: self.kind,
            title: self.title,
            status: self.status,
            priority: self.priority,
            due_at: self.due_at,
            assignment: self.assignment,
            source: self.source,
            completion_evidence: self.completion_evidence,
        })
    }
}

impl<'de> Deserialize<'de> for Task {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RawTask::deserialize(deserializer)?
            .try_into_task()
            .map_err(serde::de::Error::custom)
    }
}

impl Task {
    /// Starts checked construction of the aggregate.
    pub fn builder() -> TaskBuilder {
        TaskBuilder::default()
    }

    /// Returns the aggregate location id.
    pub const fn location_id(&self) -> LocationId {
        self.location_id
    }

    /// Returns the aggregate kind.
    pub const fn kind(&self) -> &task::Kind {
        &self.kind
    }

    /// Returns the aggregate title.
    pub const fn title(&self) -> &workflow_task::Title {
        &self.title
    }

    /// Returns the aggregate status.
    pub const fn status(&self) -> task::Status {
        self.status
    }

    /// Returns the aggregate priority.
    pub const fn priority(&self) -> task::Priority {
        self.priority
    }

    /// Returns the aggregate due at.
    pub const fn due_at(&self) -> DateTime<Utc> {
        self.due_at
    }

    /// Returns the aggregate assignment.
    pub const fn assignment(&self) -> &task::Assignment {
        &self.assignment
    }

    /// Returns the aggregate source.
    pub const fn source(&self) -> &task::Source {
        &self.source
    }

    /// Returns the aggregate completion evidence.
    pub const fn completion_evidence(&self) -> Option<&completion_evidence::Evidence> {
        self.completion_evidence.as_ref()
    }

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

#[derive(Debug, Clone, Default)]
/// Relationship-checked task builder used at this boundary.
pub struct TaskBuilder {
    location_id: Option<LocationId>,
    kind: Option<task::Kind>,
    title: Option<workflow_task::Title>,
    status: Option<task::Status>,
    priority: Option<task::Priority>,
    due_at: Option<DateTime<Utc>>,
    assignment: Option<task::Assignment>,
    source: Option<task::Source>,
    completion_evidence: Option<completion_evidence::Evidence>,
}

impl TaskBuilder {
    /// Returns the aggregate location id.
    pub fn location_id(mut self, value: LocationId) -> Self {
        self.location_id = Some(value);
        self
    }

    /// Returns the aggregate kind.
    pub fn kind(mut self, value: task::Kind) -> Self {
        self.kind = Some(value);
        self
    }

    /// Returns the aggregate title.
    pub fn title(mut self, value: workflow_task::Title) -> Self {
        self.title = Some(value);
        self
    }

    /// Returns the aggregate status.
    pub fn status(mut self, value: task::Status) -> Self {
        self.status = Some(value);
        self
    }

    /// Returns the aggregate priority.
    pub fn priority(mut self, value: task::Priority) -> Self {
        self.priority = Some(value);
        self
    }

    /// Returns the aggregate due at.
    pub fn due_at(mut self, value: DateTime<Utc>) -> Self {
        self.due_at = Some(value);
        self
    }

    /// Returns the aggregate assignment.
    pub fn assignment(mut self, value: task::Assignment) -> Self {
        self.assignment = Some(value);
        self
    }

    /// Returns the aggregate source.
    pub fn source(mut self, value: task::Source) -> Self {
        self.source = Some(value);
        self
    }

    /// Returns the aggregate completion evidence.
    pub fn completion_evidence(mut self, value: completion_evidence::Evidence) -> Self {
        self.completion_evidence = Some(value);
        self
    }

    /// Validates the accumulated fields and builds the aggregate.
    pub fn build(self) -> std::result::Result<Task, TaskError> {
        RawTask {
            location_id: self.location_id.ok_or(TaskError::LocationIdRequired)?,
            kind: self.kind.ok_or(TaskError::KindRequired)?,
            title: self.title.ok_or(TaskError::TitleRequired)?,
            status: self.status.ok_or(TaskError::StatusRequired)?,
            priority: self.priority.ok_or(TaskError::PriorityRequired)?,
            due_at: self.due_at.ok_or(TaskError::DueAtRequired)?,
            assignment: self.assignment.ok_or(TaskError::AssignmentRequired)?,
            source: self.source.ok_or(TaskError::SourceRequired)?,
            completion_evidence: self.completion_evidence,
        }
        .try_into_task()
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
