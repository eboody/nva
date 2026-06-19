//! Workflow events and outcomes for reviewable resort operations.
//!
//! # Operator framing
//!
//! Use this page to understand how a source fact turns into a staff-visible task,
//! review reason, draft message, or recommended next action. It matters to
//! operators because workflow values preserve why something is being suggested,
//! what evidence supports it, and which human review gate still controls the live
//! care, labor, payment, or customer-communication step.
//!
//! The next step is to follow the type that matches the queue you are explaining:
//! events identify why work started, task/message modules describe staff-facing
//! drafts, review values explain why automation stopped, and outcomes record the
//! evidence trail. The Rust API details below are the generated implementation surface for
//! implementers; this framing is the business reading guide.
//!
//! Workflows connect provider/read-model facts to staff-visible tasks, customer-message drafts, policy
//! context, and recommended next actions. They preserve evidence and review reasons so AI agents can
//! reduce manual triage while keeping live care, labor, payment, and customer communications inside
//! explicit approval boundaries.

use chrono::{DateTime, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{entities, policy};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Stable identifier for a workflow event emitted by an agent, adapter, or staff-facing process.
pub struct EventId(pub Uuid);

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
pub struct Summary(String);

/// Risk marker surfaced when a workflow may affect pet safety, labor cost, payment, or customer trust.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
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
pub struct RiskFlag(String);

/// Evidence note proving what source fact, review, or staff action verified a workflow outcome.
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
pub struct VerificationNote(String);

/// Review explanation recorded when automation must stop at a manager, medical, or customer-message gate.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 300),
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
pub struct ReviewReason(String);

/// External workflow-provider vocabulary retained before promotion into domain tasks or messages.
pub mod external {
    use nutype::nutype;
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};

    /// External workflow provider or system name that supplied a task, message, or status update.
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
    pub struct Provider(String);

    /// External workflow identifier used to correlate provider tasks and status updates.
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

/// Provider task fields used to create staff work without losing source evidence.
pub mod task {
    use nutype::nutype;
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};

    /// Staff-visible task title summarizing the operational work item.
    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 160),
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
    pub struct Title(String);

    /// Task or message body text that carries source evidence and review instructions.
    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 2000),
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
    pub struct Body(String);
}

/// Provider message fields used before normalization into customer-message workflows.
pub mod message {
    use nutype::nutype;
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 80),
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
    pub struct Channel(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 2000),
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
    pub struct Body(String);
}

/// Provider status-update fields used to reconcile external task or message progress.
pub mod status_update {
    use crate::entities;
    use nutype::nutype;
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};

    /// Provider-supplied status reason text preserved as review evidence.
    pub mod reason {
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
        pub struct Reason(String);
    }

    pub use reason::Reason;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Intended reservation transition requested by a workflow before policy and review checks are applied.
    pub enum TransitionIntent {
        /// Request medical review workflow state, command, or review outcome.
        RequestMedicalReview,
        /// Apply capacity decision workflow state, command, or review outcome.
        ApplyCapacityDecision,
        /// Confirm accepted offer workflow state, command, or review outcome.
        ConfirmAcceptedOffer,
        /// Cancel reservation workflow state, command, or review outcome.
        CancelReservation,
        /// Reject by policy workflow state, command, or review outcome.
        RejectByPolicy,
        /// Complete checkout workflow state, command, or review outcome.
        CompleteCheckout,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Workflow-scoped reservation transition request with target state, reason, and review intent.
    pub struct Reservation {
        /// Workflow status value preserved for staff review and audit evidence.
        pub status: entities::reservation::Status,
        /// Workflow intent value preserved for staff review and audit evidence.
        pub intent: TransitionIntent,
        /// Business reason staff should review before proceeding.
        pub reason: Reason,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Workflow target that a task, event, or recommended action is about.
    pub enum Target {
        /// Reservation record participating in the workflow.
        Reservation(Reservation),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Workflow event that records what changed, who/what it concerns, and what evidence/risk came with it.
pub struct Event {
    /// Workflow event ID value preserved for staff review and audit evidence.
    pub event_id: EventId,
    /// Workflow event type value preserved for staff review and audit evidence.
    pub event_type: EventType,
    /// Workflow occurred at value preserved for staff review and audit evidence.
    pub occurred_at: DateTime<Utc>,
    /// Workflow actor value preserved for staff review and audit evidence.
    pub actor: entities::ActorRef,
    /// Workflow location ID value preserved for staff review and audit evidence.
    pub location_id: entities::LocationId,
    /// Workflow subject value preserved for staff review and audit evidence.
    pub subject: Subject,
    /// Workflow policy context value preserved for staff review and audit evidence.
    pub policy_context: PolicyContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Event category emitted by triage, policy, review, external sync, or source ingestion.
pub enum EventType {
    /// Inquiry received workflow state, command, or review outcome.
    InquiryReceived,
    /// Customer registered workflow state, command, or review outcome.
    CustomerRegistered,
    /// Pet profile created workflow state, command, or review outcome.
    PetProfileCreated,
    /// Vaccine document uploaded workflow state, command, or review outcome.
    VaccineDocumentUploaded,
    /// Booking requested workflow state, command, or review outcome.
    BookingRequested,
    /// Booking triage needed workflow state, command, or review outcome.
    BookingTriageNeeded,
    /// Booking confirmation needed workflow state, command, or review outcome.
    BookingConfirmationNeeded,
    /// Daily note created workflow state, command, or review outcome.
    DailyNoteCreated,
    /// Daily update needed workflow state, command, or review outcome.
    DailyUpdateNeeded,
    /// Incident created workflow state, command, or review outcome.
    IncidentCreated,
    /// Checkout completed workflow state, command, or review outcome.
    CheckoutCompleted,
    /// Review request eligible workflow state, command, or review outcome.
    ReviewRequestEligible,
    /// Membership changed workflow state, command, or review outcome.
    MembershipChanged,
    /// Loyalty credit available workflow state, command, or review outcome.
    LoyaltyCreditAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Subject of a workflow event or recommendation.
pub enum Subject {
    /// Customer record participating in the workflow.
    Customer(entities::CustomerId),
    /// Pet record participating in the workflow.
    Pet(entities::PetId),
    /// Reservation record participating in the workflow.
    Reservation(entities::reservation::Id),
    /// External system object referenced from domain history.
    External {
        /// Workflow provider value preserved for staff review and audit evidence.
        provider: external::Provider,
        /// Workflow id value preserved for staff review and audit evidence.
        id: external::Id,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Policy context attached to a workflow so reviewers can see allowed actions and required gates.
pub struct PolicyContext {
    /// Workflow allowed actions value preserved for staff review and audit evidence.
    pub allowed_actions: Vec<AllowedAction>,
    /// Workflow automation level value preserved for staff review and audit evidence.
    pub automation_level: policy::automation::Level,
    /// Workflow required reviews value preserved for staff review and audit evidence.
    pub required_reviews: Vec<policy::ReviewGate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Action an automation policy permits for a workflow outcome.
pub enum AllowedAction {
    /// Read entities workflow state, command, or review outcome.
    ReadEntities,
    /// Extract structured data workflow state, command, or review outcome.
    ExtractStructuredData,
    /// Draft customer message workflow state, command, or review outcome.
    DraftCustomerMessage,
    /// Create internal task workflow state, command, or review outcome.
    CreateInternalTask,
    /// Suggest reservation status workflow state, command, or review outcome.
    SuggestReservationStatus,
    /// Suggest play eligibility workflow state, command, or review outcome.
    SuggestPlayEligibility,
    /// Summarize care notes workflow state, command, or review outcome.
    SummarizeCareNotes,
    /// Flag risk workflow state, command, or review outcome.
    FlagRisk,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Workflow result carrying status, summary, recommended action, and verification notes for staff review.
pub struct Result<T> {
    /// Workflow status value preserved for staff review and audit evidence.
    pub status: Status,
    /// Workflow summary value preserved for staff review and audit evidence.
    pub summary: Summary,
    /// Workflow structured output value preserved for staff review and audit evidence.
    pub structured_output: Option<T>,
    /// Workflow recommended actions value preserved for staff review and audit evidence.
    pub recommended_actions: Vec<RecommendedAction>,
    /// Workflow risk flags value preserved for staff review and audit evidence.
    pub risk_flags: Vec<RiskFlag>,
    /// Workflow verification value preserved for staff review and audit evidence.
    pub verification: Vec<VerificationNote>,
    /// Workflow human review reason value preserved for staff review and audit evidence.
    pub human_review_reason: Option<ReviewReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Normalized lifecycle states used to reconcile source-system data with domain workflows.
pub enum Status {
    /// Completed workflow state, command, or review outcome.
    Completed,
    /// Needs human review workflow state, command, or review outcome.
    NeedsHumanReview,
    /// Rejected by policy workflow state, command, or review outcome.
    RejectedByPolicy,
    /// Needs more information workflow state, command, or review outcome.
    NeedsMoreInformation,
    /// Failed safely workflow state, command, or review outcome.
    FailedSafely,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Recommended next action for staff, managers, or automation after evaluating a workflow.
pub enum RecommendedAction {
    /// Internal task workflow state, command, or review outcome.
    InternalTask {
        /// Workflow title value preserved for staff review and audit evidence.
        title: task::Title,
        /// Workflow body value preserved for staff review and audit evidence.
        body: task::Body,
    },
    /// Draft message workflow state, command, or review outcome.
    DraftMessage {
        /// Workflow channel value preserved for staff review and audit evidence.
        channel: message::Channel,
        /// Workflow body value preserved for staff review and audit evidence.
        body: message::Body,
    },
    /// Update status workflow state, command, or review outcome.
    UpdateStatus {
        /// Workflow target value preserved for staff review and audit evidence.
        target: status_update::Target,
    },
    /// Request human review workflow state, command, or review outcome.
    RequestHumanReview(policy::ReviewGate),
}
