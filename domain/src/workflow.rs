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
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

use crate::{entities, policy};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Stable non-nil identifier for a workflow event emitted by an agent, adapter, or staff-facing process.
pub struct EventId(Uuid);

impl EventId {
    /// Constructs an event identity, panicking when handed the forbidden nil sentinel.
    #[track_caller]
    pub fn new(value: Uuid) -> Self {
        Self::try_new(value).expect("workflow event identity UUID must be non-nil")
    }

    /// Validates an event identity at an untrusted boundary.
    pub const fn try_new(value: Uuid) -> std::result::Result<Self, entities::NilIdentityError> {
        if value.is_nil() {
            Err(entities::NilIdentityError)
        } else {
            Ok(Self(value))
        }
    }

    /// Returns the validated UUID.
    pub const fn get(self) -> Uuid {
        self.0
    }
}

impl<'de> Deserialize<'de> for EventId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(Uuid::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Workflow event that records what changed, who/what it concerns, and what evidence/risk came with it.
pub struct Event {
    /// Workflow event ID value preserved for staff review and audit evidence.
    event_id: EventId,
    /// Workflow event type value preserved for staff review and audit evidence.
    event_type: EventType,
    /// Workflow occurred at value preserved for staff review and audit evidence.
    occurred_at: DateTime<Utc>,
    /// Workflow actor value preserved for staff review and audit evidence.
    actor: entities::ActorRef,
    /// Workflow location ID value preserved for staff review and audit evidence.
    location_id: entities::LocationId,
    /// Workflow subject value preserved for staff review and audit evidence.
    subject: Subject,
    /// Workflow policy context value preserved for staff review and audit evidence.
    policy_context: PolicyContext,
}

impl Event {
    /// Creates a workflow event after validating event-type/subject agreement.
    pub fn try_new(
        event_id: EventId,
        event_type: EventType,
        occurred_at: DateTime<Utc>,
        actor: entities::ActorRef,
        location_id: entities::LocationId,
        subject: Subject,
        policy_context: PolicyContext,
    ) -> std::result::Result<Self, EventError> {
        Self::try_from_persisted(RawEvent {
            event_id,
            event_type,
            occurred_at,
            actor,
            location_id,
            subject,
            policy_context,
        })
    }

    /// Workflow event id used for audit correlation.
    pub const fn event_id(&self) -> EventId {
        self.event_id
    }

    /// Event category emitted by triage, policy, review, external sync, or source ingestion.
    pub const fn event_type(&self) -> &EventType {
        &self.event_type
    }

    /// Instant when the workflow event occurred.
    pub const fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    /// Actor that emitted or caused the workflow event.
    pub const fn actor(&self) -> &entities::ActorRef {
        &self.actor
    }

    /// Owning location for labor routing and policy lookup.
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    /// Subject that the event type is allowed to concern.
    pub const fn subject(&self) -> &Subject {
        &self.subject
    }

    /// Policy context attached for review and automation boundaries.
    pub const fn policy_context(&self) -> &PolicyContext {
        &self.policy_context
    }
}

#[derive(Deserialize)]
struct RawEvent {
    event_id: EventId,
    event_type: EventType,
    occurred_at: DateTime<Utc>,
    actor: entities::ActorRef,
    location_id: entities::LocationId,
    subject: Subject,
    policy_context: PolicyContext,
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawEvent::deserialize(deserializer)?;
        Self::try_from_persisted(raw).map_err(serde::de::Error::custom)
    }
}

impl Event {
    fn try_from_persisted(raw: RawEvent) -> std::result::Result<Self, EventError> {
        if !event_type_matches_subject(&raw.event_type, &raw.subject) {
            return Err(EventError::EventTypeSubjectMismatch);
        }
        Ok(Self {
            event_id: raw.event_id,
            event_type: raw.event_type,
            occurred_at: raw.occurred_at,
            actor: raw.actor,
            location_id: raw.location_id,
            subject: raw.subject,
            policy_context: raw.policy_context,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures returned by workflow event construction and rehydration.
pub enum EventError {
    /// Event type and subject represented different workflow ownership domains.
    #[error("workflow event subject does not match event type")]
    EventTypeSubjectMismatch,
}

fn event_type_matches_subject(event_type: &EventType, subject: &Subject) -> bool {
    match event_type {
        EventType::InquiryReceived
        | EventType::CustomerRegistered
        | EventType::ReviewRequestEligible
        | EventType::MembershipChanged
        | EventType::LoyaltyCreditAvailable => {
            matches!(subject, Subject::Customer(_) | Subject::External { .. })
        }
        EventType::PetProfileCreated
        | EventType::VaccineDocumentUploaded
        | EventType::DailyNoteCreated
        | EventType::DailyUpdateNeeded
        | EventType::IncidentCreated => {
            matches!(
                subject,
                Subject::Pet(_) | Subject::Reservation(_) | Subject::External { .. }
            )
        }
        EventType::BookingRequested
        | EventType::BookingTriageNeeded
        | EventType::BookingConfirmationNeeded
        | EventType::CheckoutCompleted => {
            matches!(subject, Subject::Reservation(_) | Subject::External { .. })
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
/// Workflow result carrying a summary plus one coherent, evidence-bearing outcome variant.
pub struct Result<T> {
    /// Workflow summary value preserved for staff review and audit evidence.
    summary: Summary,
    /// Evidence-bearing workflow outcome. The variant owns the fields legal for that state.
    outcome: Outcome<T>,
}

impl<T> Result<T> {
    /// Records a completed workflow outcome with structured output and verification evidence.
    pub fn completed(
        summary: Summary,
        structured_output: T,
        recommended_actions: Vec<RecommendedAction>,
        risk_flags: Vec<RiskFlag>,
        verification: Vec<VerificationNote>,
    ) -> std::result::Result<Self, Error> {
        ensure_verification_evidence(&verification)?;
        Ok(Self {
            summary,
            outcome: Outcome::Completed {
                structured_output,
                recommended_actions,
                risk_flags,
                verification,
            },
        })
    }

    /// Records a workflow outcome that stopped at a human review gate.
    pub fn needs_human_review(
        summary: Summary,
        human_review_reason: ReviewReason,
        recommended_actions: Vec<RecommendedAction>,
        risk_flags: Vec<RiskFlag>,
        verification: Vec<VerificationNote>,
    ) -> std::result::Result<Self, Error> {
        ensure_verification_evidence(&verification)?;
        Ok(Self {
            summary,
            outcome: Outcome::NeedsHumanReview {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            },
        })
    }

    /// Records a workflow outcome rejected by deterministic policy evidence.
    pub fn rejected_by_policy(
        summary: Summary,
        human_review_reason: ReviewReason,
        recommended_actions: Vec<RecommendedAction>,
        risk_flags: Vec<RiskFlag>,
        verification: Vec<VerificationNote>,
    ) -> std::result::Result<Self, Error> {
        ensure_verification_evidence(&verification)?;
        Ok(Self {
            summary,
            outcome: Outcome::RejectedByPolicy {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            },
        })
    }

    /// Records a workflow outcome that needs more source/staff information.
    pub fn needs_more_information(
        summary: Summary,
        human_review_reason: ReviewReason,
        recommended_actions: Vec<RecommendedAction>,
        risk_flags: Vec<RiskFlag>,
        verification: Vec<VerificationNote>,
    ) -> std::result::Result<Self, Error> {
        ensure_verification_evidence(&verification)?;
        Ok(Self {
            summary,
            outcome: Outcome::NeedsMoreInformation {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            },
        })
    }

    /// Records a workflow outcome that failed safely without producing live side effects.
    pub fn failed_safely(
        summary: Summary,
        human_review_reason: ReviewReason,
        recommended_actions: Vec<RecommendedAction>,
        risk_flags: Vec<RiskFlag>,
        verification: Vec<VerificationNote>,
    ) -> std::result::Result<Self, Error> {
        ensure_verification_evidence(&verification)?;
        Ok(Self {
            summary,
            outcome: Outcome::FailedSafely {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            },
        })
    }

    /// Summary preserved for staff review and audit evidence.
    pub const fn summary(&self) -> &Summary {
        &self.summary
    }

    /// Coherent evidence-bearing outcome variant.
    pub const fn outcome(&self) -> &Outcome<T> {
        &self.outcome
    }

    /// Stable status code used only by explicit versioned DTO/storage projections.
    pub const fn status(&self) -> Status {
        self.outcome.status()
    }

    /// Recommended staff/automation actions for this outcome.
    pub fn recommended_actions(&self) -> &[RecommendedAction] {
        self.outcome.recommended_actions()
    }

    /// Risk flags carried as review and reporting evidence for this outcome.
    pub fn risk_flags(&self) -> &[RiskFlag] {
        self.outcome.risk_flags()
    }

    /// Verification evidence carried by the outcome.
    pub fn verification(&self) -> &[VerificationNote] {
        self.outcome.verification()
    }

    /// Human review evidence for variants that legally stop at a review gate.
    pub fn human_review_reason(&self) -> Option<&ReviewReason> {
        self.outcome.human_review_reason()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Evidence-bearing result variant. Each outcome owns exactly the fields legal for its status.
pub enum Outcome<T> {
    /// Workflow completed with structured output and verification evidence.
    Completed {
        /// Typed output accepted by the owning workflow validator.
        structured_output: T,
        /// Staff-visible next actions created from the accepted output.
        recommended_actions: Vec<RecommendedAction>,
        /// Risk markers retained for reports and review.
        risk_flags: Vec<RiskFlag>,
        /// Evidence proving why the output can be treated as completed workflow output.
        verification: Vec<VerificationNote>,
    },
    /// Workflow stopped at a human review gate with an explicit reason.
    NeedsHumanReview {
        /// Review-gate evidence explaining why automation must stop.
        human_review_reason: ReviewReason,
        /// Staff-visible next actions created before the review gate.
        recommended_actions: Vec<RecommendedAction>,
        /// Risk markers retained for reports and review.
        risk_flags: Vec<RiskFlag>,
        /// Evidence proving why review is required.
        verification: Vec<VerificationNote>,
    },
    /// Workflow was rejected by policy with reviewable evidence.
    RejectedByPolicy {
        /// Policy/review evidence explaining the rejection.
        human_review_reason: ReviewReason,
        /// Staff-visible next actions created before rejection.
        recommended_actions: Vec<RecommendedAction>,
        /// Risk markers retained for reports and review.
        risk_flags: Vec<RiskFlag>,
        /// Evidence proving why the policy rejection is valid.
        verification: Vec<VerificationNote>,
    },
    /// Workflow needs more source/staff information before continuing.
    NeedsMoreInformation {
        /// Evidence explaining the missing information boundary.
        human_review_reason: ReviewReason,
        /// Staff-visible next actions for gathering missing proof.
        recommended_actions: Vec<RecommendedAction>,
        /// Risk markers retained for reports and review.
        risk_flags: Vec<RiskFlag>,
        /// Evidence proving which facts were insufficient.
        verification: Vec<VerificationNote>,
    },
    /// Workflow failed safely without live side effects.
    FailedSafely {
        /// Evidence explaining the safe-failure boundary.
        human_review_reason: ReviewReason,
        /// Staff-visible next actions after safe failure.
        recommended_actions: Vec<RecommendedAction>,
        /// Risk markers retained for reports and review.
        risk_flags: Vec<RiskFlag>,
        /// Evidence proving no unsafe completion was manufactured.
        verification: Vec<VerificationNote>,
    },
}

impl<T> Outcome<T> {
    /// Stable status code used only by explicit versioned DTO/storage projections.
    pub const fn status(&self) -> Status {
        match self {
            Self::Completed { .. } => Status::Completed,
            Self::NeedsHumanReview { .. } => Status::NeedsHumanReview,
            Self::RejectedByPolicy { .. } => Status::RejectedByPolicy,
            Self::NeedsMoreInformation { .. } => Status::NeedsMoreInformation,
            Self::FailedSafely { .. } => Status::FailedSafely,
        }
    }

    /// Structured output for completed outcomes.
    pub const fn structured_output(&self) -> Option<&T> {
        match self {
            Self::Completed {
                structured_output, ..
            } => Some(structured_output),
            Self::NeedsHumanReview { .. }
            | Self::RejectedByPolicy { .. }
            | Self::NeedsMoreInformation { .. }
            | Self::FailedSafely { .. } => None,
        }
    }

    /// Recommended staff/automation actions for this outcome.
    pub fn recommended_actions(&self) -> &[RecommendedAction] {
        match self {
            Self::Completed {
                recommended_actions,
                ..
            }
            | Self::NeedsHumanReview {
                recommended_actions,
                ..
            }
            | Self::RejectedByPolicy {
                recommended_actions,
                ..
            }
            | Self::NeedsMoreInformation {
                recommended_actions,
                ..
            }
            | Self::FailedSafely {
                recommended_actions,
                ..
            } => recommended_actions,
        }
    }

    /// Risk flags carried as review and reporting evidence for this outcome.
    pub fn risk_flags(&self) -> &[RiskFlag] {
        match self {
            Self::Completed { risk_flags, .. }
            | Self::NeedsHumanReview { risk_flags, .. }
            | Self::RejectedByPolicy { risk_flags, .. }
            | Self::NeedsMoreInformation { risk_flags, .. }
            | Self::FailedSafely { risk_flags, .. } => risk_flags,
        }
    }

    /// Verification evidence carried by the outcome.
    pub fn verification(&self) -> &[VerificationNote] {
        match self {
            Self::Completed { verification, .. }
            | Self::NeedsHumanReview { verification, .. }
            | Self::RejectedByPolicy { verification, .. }
            | Self::NeedsMoreInformation { verification, .. }
            | Self::FailedSafely { verification, .. } => verification,
        }
    }

    /// Human review evidence for variants that legally stop at a review gate.
    pub fn human_review_reason(&self) -> Option<&ReviewReason> {
        match self {
            Self::Completed { .. } => None,
            Self::NeedsHumanReview {
                human_review_reason,
                ..
            }
            | Self::RejectedByPolicy {
                human_review_reason,
                ..
            }
            | Self::NeedsMoreInformation {
                human_review_reason,
                ..
            }
            | Self::FailedSafely {
                human_review_reason,
                ..
            } => Some(human_review_reason),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures returned by workflow result/outcome construction and rehydration.
pub enum Error {
    /// Completed outcome was missing structured output evidence.
    #[error("completed workflow outcome requires structured output evidence")]
    CompletedOutcomeRequiresStructuredOutput,
    /// Completed outcome carried a human review reason, which belongs only to stopped/rejected variants.
    #[error("completed workflow outcome must not carry human review reason")]
    CompletedOutcomeMustNotCarryHumanReviewReason,
    /// A stopped or rejected outcome carried completed structured output evidence.
    #[error("non-completed workflow outcome must not carry structured output")]
    NonCompletedOutcomeMustNotCarryStructuredOutput,
    /// Human-review outcome omitted the review reason evidence.
    #[error("workflow outcome needing human review requires review reason evidence")]
    HumanReviewOutcomeRequiresReviewReason,
    /// Policy-rejected outcome omitted the review/policy reason evidence.
    #[error("policy-rejected workflow outcome requires review reason evidence")]
    PolicyRejectedOutcomeRequiresReviewReason,
    /// More-information outcome omitted the missing-information reason evidence.
    #[error("workflow outcome needing more information requires review reason evidence")]
    NeedsMoreInformationOutcomeRequiresReviewReason,
    /// Safe-failure outcome omitted the reason/evidence that explains the stop.
    #[error("failed-safe workflow outcome requires review reason evidence")]
    FailedSafelyOutcomeRequiresReviewReason,
    /// Outcome variants must carry at least one verification note before they can enter reports.
    #[error("workflow outcome requires verification evidence")]
    VerificationEvidenceRequired,
}

fn ensure_verification_evidence(
    verification: &[VerificationNote],
) -> std::result::Result<(), Error> {
    if verification.is_empty() {
        return Err(Error::VerificationEvidenceRequired);
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
/// Versioned compatibility projection for the historic workflow result wire shape.
struct ResultV1<T> {
    status: Status,
    summary: Summary,
    structured_output: Option<T>,
    recommended_actions: Vec<RecommendedAction>,
    risk_flags: Vec<RiskFlag>,
    verification: Vec<VerificationNote>,
    human_review_reason: Option<ReviewReason>,
}

impl<T> TryFrom<ResultV1<T>> for Result<T> {
    type Error = Error;

    fn try_from(value: ResultV1<T>) -> std::result::Result<Self, Self::Error> {
        if value.status != Status::Completed && value.structured_output.is_some() {
            return Err(Error::NonCompletedOutcomeMustNotCarryStructuredOutput);
        }
        match value.status {
            Status::Completed => {
                if value.human_review_reason.is_some() {
                    return Err(Error::CompletedOutcomeMustNotCarryHumanReviewReason);
                }
                let Some(structured_output) = value.structured_output else {
                    return Err(Error::CompletedOutcomeRequiresStructuredOutput);
                };
                Self::completed(
                    value.summary,
                    structured_output,
                    value.recommended_actions,
                    value.risk_flags,
                    value.verification,
                )
            }
            Status::NeedsHumanReview => {
                let Some(reason) = value.human_review_reason else {
                    return Err(Error::HumanReviewOutcomeRequiresReviewReason);
                };
                Self::needs_human_review(
                    value.summary,
                    reason,
                    value.recommended_actions,
                    value.risk_flags,
                    value.verification,
                )
            }
            Status::RejectedByPolicy => {
                let Some(reason) = value.human_review_reason else {
                    return Err(Error::PolicyRejectedOutcomeRequiresReviewReason);
                };
                Self::rejected_by_policy(
                    value.summary,
                    reason,
                    value.recommended_actions,
                    value.risk_flags,
                    value.verification,
                )
            }
            Status::NeedsMoreInformation => {
                let Some(reason) = value.human_review_reason else {
                    return Err(Error::NeedsMoreInformationOutcomeRequiresReviewReason);
                };
                Self::needs_more_information(
                    value.summary,
                    reason,
                    value.recommended_actions,
                    value.risk_flags,
                    value.verification,
                )
            }
            Status::FailedSafely => {
                let Some(reason) = value.human_review_reason else {
                    return Err(Error::FailedSafelyOutcomeRequiresReviewReason);
                };
                Self::failed_safely(
                    value.summary,
                    reason,
                    value.recommended_actions,
                    value.risk_flags,
                    value.verification,
                )
            }
        }
    }
}

impl<T> From<Result<T>> for ResultV1<T> {
    fn from(value: Result<T>) -> Self {
        let status = value.status();
        match value.outcome {
            Outcome::Completed {
                structured_output,
                recommended_actions,
                risk_flags,
                verification,
            } => Self {
                status,
                summary: value.summary,
                structured_output: Some(structured_output),
                recommended_actions,
                risk_flags,
                verification,
                human_review_reason: None,
            },
            Outcome::NeedsHumanReview {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            }
            | Outcome::RejectedByPolicy {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            }
            | Outcome::NeedsMoreInformation {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            }
            | Outcome::FailedSafely {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            } => Self {
                status,
                summary: value.summary,
                structured_output: None,
                recommended_actions,
                risk_flags,
                verification,
                human_review_reason: Some(human_review_reason),
            },
        }
    }
}

impl<T> Serialize for Result<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let status = self.status();
        match &self.outcome {
            Outcome::Completed {
                structured_output,
                recommended_actions,
                risk_flags,
                verification,
            } => ResultV1 {
                status,
                summary: self.summary.clone(),
                structured_output: Some(structured_output),
                recommended_actions: recommended_actions.clone(),
                risk_flags: risk_flags.clone(),
                verification: verification.clone(),
                human_review_reason: None,
            }
            .serialize(serializer),
            Outcome::NeedsHumanReview {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            }
            | Outcome::RejectedByPolicy {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            }
            | Outcome::NeedsMoreInformation {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            }
            | Outcome::FailedSafely {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            } => ResultV1::<&T> {
                status,
                summary: self.summary.clone(),
                structured_output: None,
                recommended_actions: recommended_actions.clone(),
                risk_flags: risk_flags.clone(),
                verification: verification.clone(),
                human_review_reason: Some(human_review_reason.clone()),
            }
            .serialize(serializer),
        }
    }
}

impl<'de, T> Deserialize<'de> for Result<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let projection = ResultV1::deserialize(deserializer)?;
        Self::try_from(projection).map_err(serde::de::Error::custom)
    }
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
