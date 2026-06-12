use chrono::{DateTime, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{ActorRef, CustomerId, LocationId, PetId, ReservationId};
use crate::policy::{ReviewGate, automation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WorkflowEventId(pub Uuid);

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
    use nutype::nutype;
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};

    use crate::entities::ReservationStatus;

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
    pub struct ReservationStatusUpdate {
        pub status: ReservationStatus,
        pub intent: TransitionIntent,
        pub reason: Reason,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Target {
        Reservation(ReservationStatusUpdate),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub event_id: WorkflowEventId,
    pub event_type: WorkflowEventType,
    pub occurred_at: DateTime<Utc>,
    pub actor: ActorRef,
    pub location_id: LocationId,
    pub subject: WorkflowSubject,
    pub policy_context: PolicyContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowEventType {
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
pub enum WorkflowSubject {
    Customer(CustomerId),
    Pet(PetId),
    Reservation(ReservationId),
    External {
        provider: external::Provider,
        id: external::Id,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyContext {
    pub allowed_actions: Vec<AllowedAction>,
    pub automation_level: automation::Level,
    pub required_reviews: Vec<ReviewGate>,
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
pub struct WorkflowResult<T> {
    pub status: WorkflowStatus,
    pub summary: Summary,
    pub structured_output: Option<T>,
    pub recommended_actions: Vec<RecommendedAction>,
    pub risk_flags: Vec<RiskFlag>,
    pub verification: Vec<VerificationNote>,
    pub human_review_reason: Option<ReviewReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
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
    RequestHumanReview(ReviewGate),
}
