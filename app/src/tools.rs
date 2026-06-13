use async_trait::async_trait;
use nutype::nutype;
use serde::{Deserialize, Serialize};

use domain::entities::{
    Customer, CustomerId, LocationId, Pet, PetId, Reservation, ReservationId, ReservationStatus,
};
use domain::money::Money;
use domain::workflow;

pub mod error;

pub use error::{Error, ExternalFailure, Resource, ResourceId, Result};

#[async_trait]
pub trait CustomerStore: Send + Sync {
    async fn get_customer(&self, id: CustomerId) -> Result<Customer>;
    async fn get_pet(&self, id: PetId) -> Result<Pet>;
    async fn get_reservation(&self, id: ReservationId) -> Result<Reservation>;
}

#[async_trait]
pub trait ReservationSystem: Send + Sync {
    async fn check_availability(
        &self,
        request: availability::Request,
    ) -> Result<availability::Outcome>;
    async fn draft_reservation_update(
        &self,
        request: draft_update::Request,
    ) -> Result<draft_update::DraftId>;
}

#[async_trait]
pub trait AgentRuntime: Send + Sync {
    async fn run_structured<TIn, TOut>(
        &self,
        event: workflow::Event,
        input: TIn,
    ) -> Result<workflow::Result<TOut>>
    where
        TIn: Send + Sync + Serialize,
        TOut: Send + Sync + for<'de> Deserialize<'de>;
}

pub mod availability {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Request {
        pub location_id: LocationId,
        pub reservation_id: Option<ReservationId>,
        pub service_notes: ServiceNotes,
    }

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 1000),
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
    pub struct ServiceNotes(String);

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Outcome {
        pub decision: Decision,
    }

    impl Outcome {
        pub fn is_available(&self) -> bool {
            matches!(self.decision, Decision::Available { .. })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Decision {
        Available {
            reason: SuccessReason,
            capacity_snapshot_id: CapacitySnapshotId,
        },
        Unavailable {
            reason: DenialReason,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum SuccessReason {
        CapacityHeld,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum DenialReason {
        CapacityUnavailable,
        PolicyHardStop,
        MissingRequiredInformation,
        RequiresHumanReview,
    }

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
    pub struct CapacitySnapshotId(String);
}

pub mod draft_update {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Request {
        pub reservation_id: ReservationId,
        pub proposed_status: ReservationStatus,
        pub rationale: Rationale,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Rationale {
        CapacityUnavailable,
        PolicyHardStop,
        MissingRequiredInformation,
        ManagerReviewRequired,
        CustomerAcceptedOffer,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DraftId(pub Id);

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
pub mod portal {
    use super::*;

    #[async_trait]
    pub trait Lookup: Send + Sync {
        async fn lookup(&self, request: lookup::Request) -> Result<lookup::Outcome>;
    }

    pub mod lookup {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Request {
            pub provider: Provider,
            pub account: AccountId,
            pub criteria: Criteria,
            pub include: Vec<Include>,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Outcome {
            pub provider: Provider,
            pub matched: Match,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum Match {
            Customer(CustomerId),
            Pet(PetId),
            Reservation(ReservationId),
            NotFound,
            Ambiguous { candidates: Vec<ExternalRecordId> },
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum Criteria {
            Customer(CustomerId),
            Pet(PetId),
            Reservation(ReservationId),
            External(ExternalRecordId),
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Provider {
        Gingr,
        Pms,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Include {
        CustomerContact,
        PetProfile,
        ReservationLedger,
    }

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
    pub struct AccountId(String);

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
    pub struct ExternalRecordId(String);
}

pub mod payment {
    use super::*;

    #[async_trait]
    pub trait Gateway: Send + Sync {
        async fn authorize(&self, request: authorization::Request)
        -> Result<authorization::Result>;
        async fn refund(&self, request: refund::Request) -> Result<refund::Result>;
        async fn record_deposit(
            &self,
            request: deposit::RecordRequest,
        ) -> Result<deposit::RecordResult>;
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Subject {
        ReservationDeposit(ReservationId),
        ReservationBalance(ReservationId),
        CustomerAccount(CustomerId),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CapturePolicy {
        AuthorizeOnly,
        CaptureImmediately,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ReviewReason {
        AmountMismatch,
        DuplicateRisk,
        ProviderAmbiguity,
    }

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
    pub struct IdempotencyKey(String);

    pub mod authorization {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Request {
            pub subject: Subject,
            pub amount: Money,
            pub capture_policy: CapturePolicy,
            pub idempotency_key: IdempotencyKey,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum Result {
            Authorized { authorization_id: Id, amount: Money },
            Declined { reason: DeclineReason },
            RequiresHumanReview { reason: ReviewReason },
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum DeclineReason {
            CardDeclined,
            InsufficientFunds,
            ProviderUnavailable,
            RequiresCustomerAction,
        }

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
        pub struct Id(String);
    }

    pub mod refund {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Request {
            pub payment_reference: domain::payment::Reference,
            pub amount: Money,
            pub reason: Reason,
            pub idempotency_key: IdempotencyKey,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum Result {
            Accepted { refund_id: Id },
            Rejected { reason: RejectionReason },
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum Reason {
            ReservationCanceled,
            ServiceNotRendered,
            ManagerApprovedAdjustment,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum RejectionReason {
            PaymentNotFound,
            AlreadyRefunded,
            OutsideRefundWindow,
            ProviderRejected,
        }

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
        pub struct Id(String);
    }

    pub mod deposit {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct RecordRequest {
            pub reservation_id: ReservationId,
            pub payment_reference: domain::payment::Reference,
            pub amount: Money,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct RecordResult {
            pub reservation_id: ReservationId,
            pub deposit_status: domain::payment::DepositStatus,
        }
    }
}

pub mod messaging {
    use super::*;

    #[async_trait]
    pub trait Drafting: Send + Sync {
        async fn draft_message(&self, request: draft::Request) -> Result<draft::Result>;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum DeliveryChannel {
        Email,
        Sms,
        Portal,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Recipient {
        Customer(CustomerId),
        Staff(domain::entities::StaffId),
        Manager(domain::entities::ManagerId),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ReviewPolicy {
        DraftOnly,
        ManagerApprovalRequired,
    }

    pub mod draft {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Request {
            pub channel: DeliveryChannel,
            pub recipient: Recipient,
            pub body: message_body::Body,
            pub review: ReviewPolicy,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Result {
            pub draft_id: draft_update::DraftId,
            pub status: Status,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum Status {
            Drafted,
            DraftedRequiresReview,
        }
    }

    pub mod message_body {
        use super::*;

        #[nutype(
            sanitize(trim),
            validate(not_empty, len_char_max = 4000),
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
}
pub mod documents {
    use super::*;

    #[async_trait]
    pub trait DocumentIntake: Send + Sync {
        async fn intake_document(
            &self,
            request: DocumentIntakeRequest,
        ) -> Result<DocumentIntakeResult>;
        async fn extract_ocr(&self, request: OcrRequest) -> Result<OcrResult>;
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DocumentIntakeRequest {
        pub document: DocumentRef,
        pub source: DocumentSource,
        pub expected_content: ExpectedContent,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DocumentIntakeResult {
        pub document: DocumentRef,
        pub classification: DocumentClassification,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct OcrRequest {
        pub document: DocumentRef,
        pub expected_content: ExpectedContent,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum OcrResult {
        Extracted { text: ExtractedText },
        NeedsHumanReview { reason: OcrReviewReason },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum DocumentSource {
        CustomerUpload,
        StaffScan,
        PortalImport,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ExpectedContent {
        VaccineProof,
        MedicationInstructions,
        BoardingAgreement,
        IncidentReport,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum DocumentClassification {
        MatchesExpectedContent,
        Mismatch,
        Unreadable,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum OcrReviewReason {
        LowConfidence,
        AmbiguousDates,
        MissingRequiredFields,
    }

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 240),
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
    pub struct DocumentRef(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 8000),
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
    pub struct ExtractedText(String);
}

pub mod media {
    use super::*;

    #[async_trait]
    pub trait Capture: Send + Sync {
        async fn request_snapshot(&self, request: SnapshotRequest) -> Result<SnapshotResult>;
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct SnapshotRequest {
        pub location_id: LocationId,
        pub camera_id: CameraId,
        pub purpose: CapturePurpose,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum SnapshotResult {
        Captured { media_ref: Ref },
        Unavailable { reason: UnavailableReason },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CapturePurpose {
        PetStatusCheck(PetId),
        FacilitySafetyCheck,
        IncidentReview(ReservationId),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum UnavailableReason {
        CameraOffline,
        PermissionDenied,
        RetentionExpired,
    }

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
    pub struct CameraId(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 240),
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
    pub struct Ref(String);
}

pub mod hermes {
    use super::*;

    #[async_trait]
    pub trait AutomationHooks: Send + Sync {
        async fn draft_task(&self, request: TaskDraftRequest) -> Result<TaskDraftResult>;
        async fn draft_schedule(
            &self,
            request: ScheduleDraftRequest,
        ) -> Result<ScheduleDraftResult>;
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TaskDraftRequest {
        pub title: workflow::task::Title,
        pub body: workflow::task::Body,
        pub queue: QueueName,
        pub trigger: Trigger,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TaskDraftResult {
        pub task_id: TaskId,
        pub status: DraftStatus,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ScheduleDraftRequest {
        pub name: ScheduleName,
        pub cadence: ScheduleCadence,
        pub queue: QueueName,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ScheduleDraftResult {
        pub schedule_id: ScheduleId,
        pub status: DraftStatus,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Trigger {
        WorkflowReview,
        OperationsBrief,
        IntegrationFailure,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum DraftStatus {
        Drafted,
        DraftedRequiresReview,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ScheduleCadence {
        Daily,
        Hourly,
        ManualOnly,
    }

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
    pub struct QueueName(String);

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
    pub struct ScheduleName(String);

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
    pub struct TaskId(String);

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
    pub struct ScheduleId(String);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalToolCandidate {
    GingrPortal,
    PaymentProvider,
    SmsProvider,
    EmailProvider,
    FileStorage,
    OcrOrDocumentAi,
    CameraOrWebcamProvider,
    HermesKanban,
    HermesCronOrWebhook,
    Postgres,
}
