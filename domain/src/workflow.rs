use chrono::{DateTime, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{entities, policy};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Typed event id domain value that keeps raw primitives out of workflow workflows.
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

/// External boundary for workflow contracts.
pub mod external {
    use nutype::nutype;
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};

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

/// Task boundary for workflow contracts.
pub mod task {
    use nutype::nutype;
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};

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

/// Message boundary for workflow contracts.
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

/// Status update boundary for workflow contracts.
pub mod status_update {
    use crate::entities;
    use nutype::nutype;
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};

    /// Reason boundary for workflow contracts.
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
    /// Domain vocabulary for transition intent decisions in workflow workflows.
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
    /// Typed reservation domain value that keeps raw primitives out of workflow workflows.
    pub struct Reservation {
        /// Status fact promoted into this workflow contract.
        pub status: entities::reservation::Status,
        /// Intent fact promoted into this workflow contract.
        pub intent: TransitionIntent,
        /// Business reason staff should review before proceeding.
        pub reason: Reason,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for target decisions in workflow workflows.
    pub enum Target {
        /// Reservation record participating in the workflow.
        Reservation(Reservation),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed event domain value that keeps raw primitives out of workflow workflows.
pub struct Event {
    /// Event id fact promoted into this workflow contract.
    pub event_id: EventId,
    /// Event type fact promoted into this workflow contract.
    pub event_type: EventType,
    /// Occurred at fact promoted into this workflow contract.
    pub occurred_at: DateTime<Utc>,
    /// Actor fact promoted into this workflow contract.
    pub actor: entities::ActorRef,
    /// Location id fact promoted into this workflow contract.
    pub location_id: entities::LocationId,
    /// Subject fact promoted into this workflow contract.
    pub subject: Subject,
    /// Policy context fact promoted into this workflow contract.
    pub policy_context: PolicyContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for event type decisions in workflow workflows.
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
/// Domain vocabulary for subject decisions in workflow workflows.
pub enum Subject {
    /// Customer record participating in the workflow.
    Customer(entities::CustomerId),
    /// Pet record participating in the workflow.
    Pet(entities::PetId),
    /// Reservation record participating in the workflow.
    Reservation(entities::reservation::Id),
    /// External system object referenced from domain history.
    External {
        /// Provider fact promoted into this workflow contract.
        provider: external::Provider,
        /// Id fact promoted into this workflow contract.
        id: external::Id,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed policy context domain value that keeps raw primitives out of workflow workflows.
pub struct PolicyContext {
    /// Allowed actions fact promoted into this workflow contract.
    pub allowed_actions: Vec<AllowedAction>,
    /// Automation level fact promoted into this workflow contract.
    pub automation_level: policy::automation::Level,
    /// Required reviews fact promoted into this workflow contract.
    pub required_reviews: Vec<policy::ReviewGate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for allowed action decisions in workflow workflows.
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
/// Typed result domain value that keeps raw primitives out of workflow workflows.
pub struct Result<T> {
    /// Status fact promoted into this workflow contract.
    pub status: Status,
    /// Summary fact promoted into this workflow contract.
    pub summary: Summary,
    /// Structured output fact promoted into this workflow contract.
    pub structured_output: Option<T>,
    /// Recommended actions fact promoted into this workflow contract.
    pub recommended_actions: Vec<RecommendedAction>,
    /// Risk flags fact promoted into this workflow contract.
    pub risk_flags: Vec<RiskFlag>,
    /// Verification fact promoted into this workflow contract.
    pub verification: Vec<VerificationNote>,
    /// Human review reason fact promoted into this workflow contract.
    pub human_review_reason: Option<ReviewReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Normalized reservation states observed during source-data ingestion.
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
/// Domain vocabulary for recommended action decisions in workflow workflows.
pub enum RecommendedAction {
    /// Internal task workflow state, command, or review outcome.
    InternalTask {
        /// Title fact promoted into this workflow contract.
        title: task::Title,
        /// Body fact promoted into this workflow contract.
        body: task::Body,
    },
    /// Draft message workflow state, command, or review outcome.
    DraftMessage {
        /// Channel fact promoted into this workflow contract.
        channel: message::Channel,
        /// Body fact promoted into this workflow contract.
        body: message::Body,
    },
    /// Update status workflow state, command, or review outcome.
    UpdateStatus {
        /// Target fact promoted into this workflow contract.
        target: status_update::Target,
    },
    /// Request human review workflow state, command, or review outcome.
    RequestHumanReview(policy::ReviewGate),
}
