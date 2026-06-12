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
    async fn draft_reservation_update(&self, request: ReservationUpdateDraft) -> Result<DraftId>;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReservationUpdateDraft {
    pub reservation_id: ReservationId,
    pub proposed_status: ReservationStatus,
    pub rationale: StatusSuggestionReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusSuggestionReason {
    CapacityUnavailable,
    PolicyHardStop,
    MissingRequiredInformation,
    ManagerReviewRequired,
    CustomerAcceptedOffer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftId(pub DraftUpdateId);

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
pub struct DraftUpdateId(String);

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

pub mod payments {
    use super::*;

    #[async_trait]
    pub trait PaymentGateway: Send + Sync {
        async fn authorize(&self, request: AuthorizationRequest) -> Result<AuthorizationResult>;
        async fn refund(&self, request: RefundRequest) -> Result<RefundResult>;
        async fn record_deposit(
            &self,
            request: DepositRecordRequest,
        ) -> Result<DepositRecordResult>;
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct AuthorizationRequest {
        pub subject: PaymentSubject,
        pub amount: Money,
        pub capture_policy: CapturePolicy,
        pub idempotency_key: IdempotencyKey,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum AuthorizationResult {
        Authorized {
            authorization_id: AuthorizationId,
            amount: Money,
        },
        Declined {
            reason: DeclineReason,
        },
        RequiresHumanReview {
            reason: PaymentReviewReason,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct RefundRequest {
        pub payment_reference: domain::payment::Reference,
        pub amount: Money,
        pub reason: RefundReason,
        pub idempotency_key: IdempotencyKey,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RefundResult {
        Accepted { refund_id: RefundId },
        Rejected { reason: RefundRejectionReason },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DepositRecordRequest {
        pub reservation_id: ReservationId,
        pub payment_reference: domain::payment::Reference,
        pub amount: Money,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DepositRecordResult {
        pub reservation_id: ReservationId,
        pub deposit_status: domain::payment::DepositStatus,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum PaymentSubject {
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
    pub enum DeclineReason {
        CardDeclined,
        InsufficientFunds,
        ProviderUnavailable,
        RequiresCustomerAction,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RefundReason {
        ReservationCanceled,
        ServiceNotRendered,
        ManagerApprovedAdjustment,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RefundRejectionReason {
        PaymentNotFound,
        AlreadyRefunded,
        OutsideRefundWindow,
        ProviderRejected,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum PaymentReviewReason {
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
    pub struct AuthorizationId(String);

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
    pub struct RefundId(String);
}

pub mod messaging {
    use super::*;

    #[async_trait]
    pub trait MessageDrafting: Send + Sync {
        async fn draft_message(&self, request: DraftMessageRequest) -> Result<DraftMessageResult>;
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DraftMessageRequest {
        pub channel: DeliveryChannel,
        pub recipient: Recipient,
        pub body: MessageBody,
        pub review: MessageReviewPolicy,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DraftMessageResult {
        pub draft_id: DraftId,
        pub status: DraftMessageStatus,
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
    pub enum MessageReviewPolicy {
        DraftOnly,
        ManagerApprovalRequired,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum DraftMessageStatus {
        Drafted,
        DraftedRequiresReview,
    }

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
    pub struct MessageBody(String);
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
