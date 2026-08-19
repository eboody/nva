use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Stable non-nil identifier for a workflow event emitted by an agent, adapter, or staff-facing process.
pub struct EventId(Uuid);

impl EventId {
    /// Validates an event identity at an untrusted boundary.
    pub const fn try_new(value: Uuid) -> std::result::Result<Self, entities::NilIdentityError> {
        if value.is_nil() {
            Err(entities::NilIdentityError)
        } else {
            Ok(Self(value))
        }
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
