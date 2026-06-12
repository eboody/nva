use chrono::{DateTime, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{entities, policy};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

pub mod status_update {
    use crate::entities;
    use nutype::nutype;
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};

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

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum TransitionIntent {
        RequestMedicalReview,
        ApplyCapacityDecision,
        ConfirmAcceptedOffer,
        CancelReservation,
        RejectByPolicy,
        CompleteCheckout,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Reservation {
        pub status: entities::ReservationStatus,
        pub intent: TransitionIntent,
        pub reason: Reason,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Target {
        Reservation(Reservation),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub event_id: EventId,
    pub event_type: EventType,
    pub occurred_at: DateTime<Utc>,
    pub actor: entities::ActorRef,
    pub location_id: entities::LocationId,
    pub subject: Subject,
    pub policy_context: PolicyContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    InquiryReceived,
    CustomerRegistered,
    PetProfileCreated,
    VaccineDocumentUploaded,
    BookingRequested,
    BookingTriageNeeded,
    BookingConfirmationNeeded,
    DailyNoteCreated,
    DailyUpdateNeeded,
    IncidentCreated,
    CheckoutCompleted,
    ReviewRequestEligible,
    MembershipChanged,
    LoyaltyCreditAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Subject {
    Customer(entities::CustomerId),
    Pet(entities::PetId),
    Reservation(entities::ReservationId),
    External {
        provider: external::Provider,
        id: external::Id,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyContext {
    pub allowed_actions: Vec<AllowedAction>,
    pub automation_level: policy::AutomationLevel,
    pub required_reviews: Vec<policy::ReviewGate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllowedAction {
    ReadEntities,
    ExtractStructuredData,
    DraftCustomerMessage,
    CreateInternalTask,
    SuggestReservationStatus,
    SuggestPlayEligibility,
    SummarizeCareNotes,
    FlagRisk,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Result<T> {
    pub status: Status,
    pub summary: Summary,
    pub structured_output: Option<T>,
    pub recommended_actions: Vec<RecommendedAction>,
    pub risk_flags: Vec<RiskFlag>,
    pub verification: Vec<VerificationNote>,
    pub human_review_reason: Option<ReviewReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Completed,
    NeedsHumanReview,
    RejectedByPolicy,
    NeedsMoreInformation,
    FailedSafely,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendedAction {
    InternalTask {
        title: task::Title,
        body: task::Body,
    },
    DraftMessage {
        channel: message::Channel,
        body: message::Body,
    },
    UpdateStatus {
        target: status_update::Target,
    },
    RequestHumanReview(policy::ReviewGate),
}
