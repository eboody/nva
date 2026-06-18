//! App-owned external tool-port contracts.
//!
//! These ports describe narrow capabilities the deterministic app may call.
//! Constructing a request is not authority for an agent to perform the side
//! effect directly: message tools can draft under review, reservation tools can
//! draft updates, and payment/provider traits remain behind app policy and human
//! review gates.
//!
//! ```
//! use app::tools::{draft_update, messaging};
//! use domain::{entities, message, workflow};
//! use uuid::Uuid;
//!
//! let draft_request = messaging::draft::Request {
//!     channel: messaging::DeliveryChannel::Portal,
//!     recipient: messaging::Recipient::Manager(
//!         entities::ManagerId::try_new("gm-fixture")?,
//!     ),
//!     body: messaging::message_body::Body::try_new(
//!         "Internal review task: verify vaccine evidence before any customer send.",
//!     )?,
//!     review: messaging::ReviewPolicy::ManagerApprovalRequired,
//! };
//!
//! assert_eq!(draft_request.review, messaging::ReviewPolicy::ManagerApprovalRequired);
//!
//! let reservation_update = draft_update::Request {
//!     reservation_id: entities::reservation::Id(Uuid::from_u128(0x123)),
//!     proposed_status: entities::reservation::Status::SpecialReview,
//!     rationale: draft_update::Rationale::ManagerReviewRequired,
//! };
//!
//! // The port models a draft/update proposal; it is not a provider/PMS write.
//! assert_eq!(reservation_update.proposed_status, entities::reservation::Status::SpecialReview);
//!
//! let internal_task = workflow::task::Title::try_new("Review ambiguous boarding vaccine proof")?;
//! assert_eq!(
//!     internal_task.into_inner(),
//!     "Review ambiguous boarding vaccine proof",
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
use async_trait::async_trait;
use nutype::nutype;
use serde::{Deserialize, Serialize};

use domain::entities::{Customer, CustomerId, LocationId, Pet, PetId, Reservation, reservation};
use domain::money::Money;
use domain::workflow;

/// Shared tool errors returned when an agent helper cannot safely complete.
pub mod error;

pub use error::{Error, ExternalFailure, Resource, ResourceId, Result};

#[async_trait]
/// Defines the behavior required from a customer store participant in the tools workflow.
pub trait CustomerStore: Send + Sync {
    /// Runs the id step while preserving the agent tool workflow safety boundary.
    async fn get_customer(&self, id: CustomerId) -> Result<Customer>;
    /// Runs the id step while preserving the agent tool workflow safety boundary.
    async fn get_pet(&self, id: PetId) -> Result<Pet>;
    /// Runs the id step while preserving the agent tool workflow safety boundary.
    async fn get_reservation(&self, id: reservation::Id) -> Result<Reservation>;
}

#[async_trait]
/// Defines the behavior required from a reservation system participant in the tools workflow.
pub trait ReservationSystem: Send + Sync {
    /// Runs the check availability step while preserving the agent tool workflow safety boundary.
    async fn check_availability(
        &self,
        request: availability::Request,
    ) -> Result<availability::Outcome>;
    /// Runs the draft reservation update step while preserving the agent tool workflow safety boundary.
    async fn draft_reservation_update(
        &self,
        request: draft_update::Request,
    ) -> Result<draft_update::draft::Id>;
}

#[async_trait]
/// Defines the behavior required from a agent runtime participant in the tools workflow.
pub trait AgentRuntime: Send + Sync {
    /// Runs the t in step while preserving the agent tool workflow safety boundary.
    async fn run_structured<TIn, TOut>(
        &self,
        event: workflow::Event,
        input: TIn,
    ) -> Result<workflow::Result<TOut>>
    where
        TIn: Send + Sync + Serialize,
        TOut: Send + Sync + for<'de> Deserialize<'de>;
}

/// Read-only availability checks that inform drafts without confirming bookings.
pub mod availability {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Input contract for building the workflow packet from source-grounded records.
    pub struct Request {
        /// Location id preserved as evidence for audit, review, or agent context.
        pub location_id: LocationId,
        /// Reservation id preserved as evidence for audit, review, or agent context.
        pub reservation_id: Option<reservation::Id>,
        /// Service notes preserved as evidence for audit, review, or agent context.
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
    /// Outcome carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
    pub struct Outcome {
        /// Decision preserved as evidence for audit, review, or agent context.
        pub decision: Decision,
    }

    impl Outcome {
        /// Reports whether the agent tool workflow satisfies the is available safety condition.
        pub fn is_available(&self) -> bool {
            matches!(self.decision, Decision::Available { .. })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies decision values that drive the agent tool workflow.
    pub enum Decision {
        /// Routes agent tool work flagged as available to the right queue, review gate, or agent packet.
        Available {
            /// Reason preserved as evidence for audit, review, or agent context.
            reason: SuccessReason,
            /// Capacity snapshot id preserved as evidence for audit, review, or agent context.
            capacity_snapshot_id: CapacitySnapshotId,
        },
        /// Routes agent tool work flagged as unavailable to the right queue, review gate, or agent packet.
        Unavailable {
            /// Reason preserved as evidence for audit, review, or agent context.
            reason: DenialReason,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies success reason values that drive the agent tool workflow.
    pub enum SuccessReason {
        /// Uses capacity held as source-grounded evidence for the deterministic decision.
        CapacityHeld,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies denial reason values that drive the agent tool workflow.
    pub enum DenialReason {
        /// Uses capacity unavailable as source-grounded evidence for the deterministic decision.
        CapacityUnavailable,
        /// Uses policy hard stop as source-grounded evidence for the deterministic decision.
        PolicyHardStop,
        /// Uses missing required information as source-grounded evidence for the deterministic decision.
        MissingRequiredInformation,
        /// Uses requires human review as source-grounded evidence for the deterministic decision.
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

/// Draft reservation-update requests held for staff review before provider writes.
pub mod draft_update {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Input contract for building the workflow packet from source-grounded records.
    pub struct Request {
        /// Reservation id preserved as evidence for audit, review, or agent context.
        pub reservation_id: reservation::Id,
        /// Proposed status preserved as evidence for audit, review, or agent context.
        pub proposed_status: reservation::Status,
        /// Rationale preserved as evidence for audit, review, or agent context.
        pub rationale: Rationale,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies rationale values that drive the agent tool workflow.
    pub enum Rationale {
        /// Routes agent tool work flagged as capacity unavailable to the right queue, review gate, or agent packet.
        CapacityUnavailable,
        /// Routes agent tool work flagged as policy hard stop to the right queue, review gate, or agent packet.
        PolicyHardStop,
        /// Routes agent tool work flagged as missing required information to the right queue, review gate, or agent packet.
        MissingRequiredInformation,
        /// Routes agent tool work flagged as manager review required to the right queue, review gate, or agent packet.
        ManagerReviewRequired,
        /// Routes agent tool work flagged as customer accepted offer to the right queue, review gate, or agent packet.
        CustomerAcceptedOffer,
    }

    pub use draft::Id as Draft;

    /// Draft payloads produced for review instead of direct customer/provider actions.
    pub mod draft {
        use super::*;

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
}
/// Provider portal lookups exposed as read-only agent context.
pub mod portal {
    use super::*;

    #[async_trait]
    /// Defines the behavior required from a lookup participant in the tools workflow.
    pub trait Lookup: Send + Sync {
        /// Runs the request step while preserving the agent tool workflow safety boundary.
        async fn lookup(&self, request: lookup::Request) -> Result<lookup::Outcome>;
    }

    /// Lookup requests and outcomes for pulling provider context into review packets.
    pub mod lookup {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Input contract for building the workflow packet from source-grounded records.
        pub struct Request {
            /// Provider preserved as evidence for audit, review, or agent context.
            pub provider: Provider,
            /// Account preserved as evidence for audit, review, or agent context.
            pub account: AccountId,
            /// Criteria preserved as evidence for audit, review, or agent context.
            pub criteria: Criteria,
            /// Include preserved as evidence for audit, review, or agent context.
            pub include: Vec<Include>,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Outcome carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct Outcome {
            /// Provider preserved as evidence for audit, review, or agent context.
            pub provider: Provider,
            /// Matched preserved as evidence for audit, review, or agent context.
            pub matched: Match,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Classifies match values that drive the agent tool workflow.
        pub enum Match {
            /// Routes agent tool work flagged as customer to the right queue, review gate, or agent packet.
            Customer(CustomerId),
            /// Routes agent tool work flagged as pet to the right queue, review gate, or agent packet.
            Pet(PetId),
            /// Routes agent tool work flagged as reservation to the right queue, review gate, or agent packet.
            Reservation(reservation::Id),
            /// Routes agent tool work flagged as not found to the right queue, review gate, or agent packet.
            NotFound,
            /// Candidates preserved as evidence for audit, review, or agent context.
            Ambiguous {
                /// Candidates carried by this variant.
                candidates: Vec<ExternalRecordId>,
            },
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Classifies criteria values that drive the agent tool workflow.
        pub enum Criteria {
            /// Routes agent tool work flagged as customer to the right queue, review gate, or agent packet.
            Customer(CustomerId),
            /// Routes agent tool work flagged as pet to the right queue, review gate, or agent packet.
            Pet(PetId),
            /// Routes agent tool work flagged as reservation to the right queue, review gate, or agent packet.
            Reservation(reservation::Id),
            /// Routes agent tool work flagged as external to the right queue, review gate, or agent packet.
            External(ExternalRecordId),
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies provider values that drive the agent tool workflow.
    pub enum Provider {
        /// Routes agent tool work flagged as gingr to the right queue, review gate, or agent packet.
        Gingr,
        /// Routes agent tool work flagged as pms to the right queue, review gate, or agent packet.
        Pms,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies include values that drive the agent tool workflow.
    pub enum Include {
        /// Routes agent tool work flagged as customer contact to the right queue, review gate, or agent packet.
        CustomerContact,
        /// Routes agent tool work flagged as pet profile to the right queue, review gate, or agent packet.
        PetProfile,
        /// Routes agent tool work flagged as reservation ledger to the right queue, review gate, or agent packet.
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

/// Payment helper contracts that keep authorizations, refunds, and deposits auditable.
pub mod payment {
    use super::*;

    #[async_trait]
    /// Defines the behavior required from a gateway participant in the tools workflow.
    pub trait Gateway: Send + Sync {
        /// Runs the authorize step while preserving the agent tool workflow safety boundary.
        async fn authorize(
            &self,
            request: authorization::Request,
        ) -> Result<authorization::provider::Result>;
        /// Runs the request step while preserving the agent tool workflow safety boundary.
        async fn refund(&self, request: refund::Request) -> Result<refund::provider::Result>;
        /// Runs the record deposit step while preserving the agent tool workflow safety boundary.
        async fn record_deposit(
            &self,
            request: deposit::RecordRequest,
        ) -> Result<deposit::RecordResult>;
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies subject values that drive the agent tool workflow.
    pub enum Subject {
        /// Routes agent tool work flagged as reservation deposit to the right queue, review gate, or agent packet.
        ReservationDeposit(reservation::Id),
        /// Routes agent tool work flagged as reservation balance to the right queue, review gate, or agent packet.
        ReservationBalance(reservation::Id),
        /// Routes agent tool work flagged as customer account to the right queue, review gate, or agent packet.
        CustomerAccount(CustomerId),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies capture policy values that drive the agent tool workflow.
    pub enum CapturePolicy {
        /// Routes agent tool work flagged as authorize only to the right queue, review gate, or agent packet.
        AuthorizeOnly,
        /// Routes agent tool work flagged as capture immediately to the right queue, review gate, or agent packet.
        CaptureImmediately,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies review reason values that drive the agent tool workflow.
    pub enum ReviewReason {
        /// Uses amount mismatch as source-grounded evidence for the deterministic decision.
        AmountMismatch,
        /// Uses duplicate risk as source-grounded evidence for the deterministic decision.
        DuplicateRisk,
        /// Uses provider ambiguity as source-grounded evidence for the deterministic decision.
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

    /// Payment authorization drafts that require explicit review before money movement.
    pub mod authorization {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Input contract for building the workflow packet from source-grounded records.
        pub struct Request {
            /// Subject preserved as evidence for audit, review, or agent context.
            pub subject: Subject,
            /// Amount preserved as evidence for audit, review, or agent context.
            pub amount: Money,
            /// Capture policy preserved as evidence for audit, review, or agent context.
            pub capture_policy: CapturePolicy,
            /// Idempotency key preserved as evidence for audit, review, or agent context.
            pub idempotency_key: IdempotencyKey,
        }

        /// Provider-facing result types kept separate from staff-review requests.
        pub mod provider {
            use super::*;

            pub use authorization_id::Id as AuthorizationId;

            #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
            /// Classifies result values that drive the agent tool workflow.
            pub enum Result {
                /// Routes agent tool work flagged as authorized to the right queue, review gate, or agent packet.
                Authorized {
                    /// Authorization id preserved as evidence for audit, review, or agent context.
                    authorization_id: AuthorizationId,
                    /// Amount preserved as evidence for audit, review, or agent context.
                    amount: Money,
                },
                /// Routes agent tool work flagged as declined to the right queue, review gate, or agent packet.
                Declined {
                    /// Reason preserved as evidence for audit, review, or agent context.
                    reason: DeclineReason,
                },
                /// Routes agent tool work flagged as requires human review to the right queue, review gate, or agent packet.
                RequiresHumanReview {
                    /// Reason preserved as evidence for audit, review, or agent context.
                    reason: ReviewReason,
                },
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
            /// Classifies decline reason values that drive the agent tool workflow.
            pub enum DeclineReason {
                /// Uses card declined as source-grounded evidence for the deterministic decision.
                CardDeclined,
                /// Uses insufficient funds as source-grounded evidence for the deterministic decision.
                InsufficientFunds,
                /// Uses provider unavailable as source-grounded evidence for the deterministic decision.
                ProviderUnavailable,
                /// Uses requires customer action as source-grounded evidence for the deterministic decision.
                RequiresCustomerAction,
            }

            /// Validated authorization identifiers returned by payment providers.
            pub mod authorization_id {
                use super::*;

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
        }
    }

    /// Refund requests and provider receipts that keep payment exceptions reviewable.
    pub mod refund {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Input contract for building the workflow packet from source-grounded records.
        pub struct Request {
            /// Payment reference preserved as evidence for audit, review, or agent context.
            pub payment_reference: domain::payment::Reference,
            /// Amount preserved as evidence for audit, review, or agent context.
            pub amount: Money,
            /// Reason preserved as evidence for audit, review, or agent context.
            pub reason: Reason,
            /// Idempotency key preserved as evidence for audit, review, or agent context.
            pub idempotency_key: IdempotencyKey,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Classifies reason values that drive the agent tool workflow.
        pub enum Reason {
            /// Uses reservation canceled as source-grounded evidence for the deterministic decision.
            ReservationCanceled,
            /// Uses service not rendered as source-grounded evidence for the deterministic decision.
            ServiceNotRendered,
            /// Uses manager approved adjustment as source-grounded evidence for the deterministic decision.
            ManagerApprovedAdjustment,
        }

        /// Provider-facing result types kept separate from staff-review requests.
        pub mod provider {
            use super::*;

            pub use refund_id::Id as RefundId;

            #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
            /// Classifies result values that drive the agent tool workflow.
            pub enum Result {
                /// Refund id preserved as evidence for audit, review, or agent context.
                Accepted {
                    /// Refund id carried by this variant.
                    refund_id: RefundId,
                },
                /// Reason preserved as evidence for audit, review, or agent context.
                Rejected {
                    /// Reason carried by this variant.
                    reason: RejectionReason,
                },
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
            /// Classifies rejection reason values that drive the agent tool workflow.
            pub enum RejectionReason {
                /// Uses payment not found as source-grounded evidence for the deterministic decision.
                PaymentNotFound,
                /// Uses already refunded as source-grounded evidence for the deterministic decision.
                AlreadyRefunded,
                /// Uses outside refund window as source-grounded evidence for the deterministic decision.
                OutsideRefundWindow,
                /// Uses provider rejected as source-grounded evidence for the deterministic decision.
                ProviderRejected,
            }

            /// Validated refund identifiers returned by payment providers.
            pub mod refund_id {
                use super::*;

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
        }
    }

    /// Deposit-recording requests that make payment handoffs explicit and auditable.
    pub mod deposit {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Record request carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct RecordRequest {
            /// Reservation id preserved as evidence for audit, review, or agent context.
            pub reservation_id: reservation::Id,
            /// Payment reference preserved as evidence for audit, review, or agent context.
            pub payment_reference: domain::payment::Reference,
            /// Amount preserved as evidence for audit, review, or agent context.
            pub amount: Money,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Record result carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct RecordResult {
            /// Reservation id preserved as evidence for audit, review, or agent context.
            pub reservation_id: reservation::Id,
            /// Deposit status preserved as evidence for audit, review, or agent context.
            pub deposit_status: domain::payment::DepositStatus,
        }
    }
}

/// Customer-message drafting contracts that never send without staff approval.
pub mod messaging {
    use super::*;

    #[async_trait]
    /// Defines the behavior required from a drafting participant in the tools workflow.
    pub trait Drafting: Send + Sync {
        /// Runs the request step while preserving the agent tool workflow safety boundary.
        async fn draft_message(&self, request: draft::Request) -> Result<draft::Result>;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies delivery channel values that drive the agent tool workflow.
    pub enum DeliveryChannel {
        /// Routes agent tool work flagged as email to the right queue, review gate, or agent packet.
        Email,
        /// Routes agent tool work flagged as sms to the right queue, review gate, or agent packet.
        Sms,
        /// Routes agent tool work flagged as portal to the right queue, review gate, or agent packet.
        Portal,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies recipient values that drive the agent tool workflow.
    pub enum Recipient {
        /// Routes agent tool work flagged as customer to the right queue, review gate, or agent packet.
        Customer(CustomerId),
        /// Routes agent tool work flagged as staff to the right queue, review gate, or agent packet.
        Staff(domain::entities::StaffId),
        /// Routes agent tool work flagged as manager to the right queue, review gate, or agent packet.
        Manager(domain::entities::ManagerId),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies review policy values that drive the agent tool workflow.
    pub enum ReviewPolicy {
        /// Routes agent tool work flagged as draft only to the right queue, review gate, or agent packet.
        DraftOnly,
        /// Routes agent tool work flagged as manager approval required to the right queue, review gate, or agent packet.
        ManagerApprovalRequired,
    }

    /// Draft payloads produced for review instead of direct customer/provider actions.
    pub mod draft {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Input contract for building the workflow packet from source-grounded records.
        pub struct Request {
            /// Channel preserved as evidence for audit, review, or agent context.
            pub channel: DeliveryChannel,
            /// Recipient preserved as evidence for audit, review, or agent context.
            pub recipient: Recipient,
            /// Body preserved as evidence for audit, review, or agent context.
            pub body: message_body::Body,
            /// Review preserved as evidence for audit, review, or agent context.
            pub review: ReviewPolicy,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Result carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct Result {
            /// Draft id preserved as evidence for audit, review, or agent context.
            pub draft_id: draft_update::draft::Id,
            /// Status preserved as evidence for audit, review, or agent context.
            pub status: Status,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Classifies status values that drive the agent tool workflow.
        pub enum Status {
            /// Labels work as drafted for queueing, review, and downstream agent context.
            Drafted,
            /// Labels work as drafted requires review for queueing, review, and downstream agent context.
            DraftedRequiresReview,
        }
    }

    /// Validated message bodies bounded for safe staff review.
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
/// Document intake and OCR helpers that turn uploads into reviewable evidence.
pub mod documents {
    use super::*;

    /// Document records and intake results used as source evidence.
    pub mod document {
        use super::*;

        #[async_trait]
        /// Defines the behavior required from a intake participant in the tools workflow.
        pub trait Intake: Send + Sync {
            /// Runs the request step while preserving the agent tool workflow safety boundary.
            async fn intake_document(&self, request: IntakeRequest) -> Result<IntakeResult>;
            /// Runs the request step while preserving the agent tool workflow safety boundary.
            async fn extract_ocr(&self, request: super::ocr::Request)
            -> Result<super::ocr::Result>;
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Intake request carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct IntakeRequest {
            /// Document preserved as evidence for audit, review, or agent context.
            pub document: reference::Ref,
            /// Source preserved as evidence for audit, review, or agent context.
            pub source: Source,
            /// Expected content preserved as evidence for audit, review, or agent context.
            pub expected_content: ExpectedContent,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Intake result carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct IntakeResult {
            /// Document preserved as evidence for audit, review, or agent context.
            pub document: reference::Ref,
            /// Classification preserved as evidence for audit, review, or agent context.
            pub classification: Classification,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Classifies source values that drive the agent tool workflow.
        pub enum Source {
            /// Routes agent tool work flagged as customer upload to the right queue, review gate, or agent packet.
            CustomerUpload,
            /// Routes agent tool work flagged as staff scan to the right queue, review gate, or agent packet.
            StaffScan,
            /// Routes agent tool work flagged as portal import to the right queue, review gate, or agent packet.
            PortalImport,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Classifies expected content values that drive the agent tool workflow.
        pub enum ExpectedContent {
            /// Routes agent tool work flagged as vaccine proof to the right queue, review gate, or agent packet.
            VaccineProof,
            /// Routes agent tool work flagged as medication instructions to the right queue, review gate, or agent packet.
            MedicationInstructions,
            /// Routes agent tool work flagged as boarding agreement to the right queue, review gate, or agent packet.
            BoardingAgreement,
            /// Routes agent tool work flagged as incident report to the right queue, review gate, or agent packet.
            IncidentReport,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Classifies classification values that drive the agent tool workflow.
        pub enum Classification {
            /// Routes agent tool work flagged as matches expected content to the right queue, review gate, or agent packet.
            MatchesExpectedContent,
            /// Routes agent tool work flagged as mismatch to the right queue, review gate, or agent packet.
            Mismatch,
            /// Routes agent tool work flagged as unreadable to the right queue, review gate, or agent packet.
            Unreadable,
        }

        /// Stable document references carried through extraction and review packets.
        pub mod reference {
            use super::*;

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
    }

    /// OCR extraction requests that supply evidence without making policy decisions.
    pub mod ocr {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Input contract for building the workflow packet from source-grounded records.
        pub struct Request {
            /// Document preserved as evidence for audit, review, or agent context.
            pub document: super::document::reference::Ref,
            /// Expected content preserved as evidence for audit, review, or agent context.
            pub expected_content: super::document::ExpectedContent,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Classifies result values that drive the agent tool workflow.
        pub enum Result {
            /// Text preserved as evidence for audit, review, or agent context.
            Extracted {
                /// Text carried by this variant.
                text: extracted_text::Text,
            },
            /// Reason preserved as evidence for audit, review, or agent context.
            NeedsHumanReview {
                /// Reason carried by this variant.
                reason: ReviewReason,
            },
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Classifies review reason values that drive the agent tool workflow.
        pub enum ReviewReason {
            /// Uses low confidence as source-grounded evidence for the deterministic decision.
            LowConfidence,
            /// Uses ambiguous dates as source-grounded evidence for the deterministic decision.
            AmbiguousDates,
            /// Uses missing required fields as source-grounded evidence for the deterministic decision.
            MissingRequiredFields,
        }

        /// Validated OCR text snippets attached to review packets.
        pub mod extracted_text {
            use super::*;

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
            pub struct Text(String);
        }
    }
}

/// Media snapshot helpers that provide visual context without live device control.
pub mod media {
    use super::*;

    #[async_trait]
    /// Defines the behavior required from a capture participant in the tools workflow.
    pub trait Capture: Send + Sync {
        /// Runs the request step while preserving the agent tool workflow safety boundary.
        async fn request_snapshot(&self, request: SnapshotRequest) -> Result<SnapshotResult>;
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Snapshot request carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
    pub struct SnapshotRequest {
        /// Location id preserved as evidence for audit, review, or agent context.
        pub location_id: LocationId,
        /// Camera id preserved as evidence for audit, review, or agent context.
        pub camera_id: CameraId,
        /// Purpose preserved as evidence for audit, review, or agent context.
        pub purpose: CapturePurpose,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies snapshot result values that drive the agent tool workflow.
    pub enum SnapshotResult {
        /// Media ref preserved as evidence for audit, review, or agent context.
        Captured {
            /// Media ref carried by this variant.
            media_ref: Ref,
        },
        /// Reason preserved as evidence for audit, review, or agent context.
        Unavailable {
            /// Reason carried by this variant.
            reason: UnavailableReason,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies capture purpose values that drive the agent tool workflow.
    pub enum CapturePurpose {
        /// Routes agent tool work flagged as pet status check to the right queue, review gate, or agent packet.
        PetStatusCheck(PetId),
        /// Routes agent tool work flagged as facility safety check to the right queue, review gate, or agent packet.
        FacilitySafetyCheck,
        /// Routes agent tool work flagged as incident review to the right queue, review gate, or agent packet.
        IncidentReview(reservation::Id),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies unavailable reason values that drive the agent tool workflow.
    pub enum UnavailableReason {
        /// Uses camera offline as source-grounded evidence for the deterministic decision.
        CameraOffline,
        /// Uses permission denied as source-grounded evidence for the deterministic decision.
        PermissionDenied,
        /// Uses retention expired as source-grounded evidence for the deterministic decision.
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

/// Hermes task and schedule drafts used to queue staff work safely.
pub mod hermes {
    use super::*;

    #[async_trait]
    /// Defines the behavior required from a automation hooks participant in the tools workflow.
    pub trait AutomationHooks: Send + Sync {
        /// Runs the draft task step while preserving the agent tool workflow safety boundary.
        async fn draft_task(
            &self,
            request: task::DraftRequest,
        ) -> Result<task::kanban::DraftResult>;
        /// Runs the draft schedule step while preserving the agent tool workflow safety boundary.
        async fn draft_schedule(
            &self,
            request: schedule::DraftRequest,
        ) -> Result<schedule::DraftResult>;
    }

    /// Internal task drafts that route work to staff instead of mutating provider records.
    pub mod task {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Draft request carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct DraftRequest {
            /// Title preserved as evidence for audit, review, or agent context.
            pub title: workflow::task::Title,
            /// Body preserved as evidence for audit, review, or agent context.
            pub body: workflow::task::Body,
            /// Queue preserved as evidence for audit, review, or agent context.
            pub queue: QueueName,
            /// Trigger preserved as evidence for audit, review, or agent context.
            pub trigger: Trigger,
        }

        /// Kanban task records returned after staff-work items are drafted.
        pub mod kanban {
            use super::*;

            pub use task_id::Id as TaskId;

            #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
            /// Draft result carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
            pub struct DraftResult {
                /// Task id preserved as evidence for audit, review, or agent context.
                pub task_id: TaskId,
                /// Status preserved as evidence for audit, review, or agent context.
                pub status: DraftStatus,
            }

            /// Validated task identifiers for Hermes/kanban work items.
            pub mod task_id {
                use super::*;

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
        }
    }

    /// Schedule drafts that coordinate staff work without starting unattended automation.
    pub mod schedule {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Draft request carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct DraftRequest {
            /// Name preserved as evidence for audit, review, or agent context.
            pub name: Name,
            /// Cadence preserved as evidence for audit, review, or agent context.
            pub cadence: Cadence,
            /// Queue preserved as evidence for audit, review, or agent context.
            pub queue: QueueName,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Draft result carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct DraftResult {
            /// Schedule id preserved as evidence for audit, review, or agent context.
            pub schedule_id: Id,
            /// Status preserved as evidence for audit, review, or agent context.
            pub status: DraftStatus,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Classifies cadence values that drive the agent tool workflow.
        pub enum Cadence {
            /// Routes agent tool work flagged as daily to the right queue, review gate, or agent packet.
            Daily,
            /// Routes agent tool work flagged as hourly to the right queue, review gate, or agent packet.
            Hourly,
            /// Routes agent tool work flagged as manual only to the right queue, review gate, or agent packet.
            ManualOnly,
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
        pub struct Name(String);

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

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies trigger values that drive the agent tool workflow.
    pub enum Trigger {
        /// Routes agent tool work flagged as workflow review to the right queue, review gate, or agent packet.
        WorkflowReview,
        /// Routes agent tool work flagged as operations brief to the right queue, review gate, or agent packet.
        OperationsBrief,
        /// Routes agent tool work flagged as integration failure to the right queue, review gate, or agent packet.
        IntegrationFailure,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Classifies draft status values that drive the agent tool workflow.
    pub enum DraftStatus {
        /// Labels work as drafted for queueing, review, and downstream agent context.
        Drafted,
        /// Labels work as drafted requires review for queueing, review, and downstream agent context.
        DraftedRequiresReview,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Classifies external tool candidate values that drive the agent tool workflow.
pub enum ExternalToolCandidate {
    /// Routes agent tool work flagged as gingr portal to the right queue, review gate, or agent packet.
    GingrPortal,
    /// Routes agent tool work flagged as payment provider to the right queue, review gate, or agent packet.
    PaymentProvider,
    /// Routes agent tool work flagged as sms provider to the right queue, review gate, or agent packet.
    SmsProvider,
    /// Routes agent tool work flagged as email provider to the right queue, review gate, or agent packet.
    EmailProvider,
    /// Routes agent tool work flagged as file storage to the right queue, review gate, or agent packet.
    FileStorage,
    /// Routes agent tool work flagged as ocr or document ai to the right queue, review gate, or agent packet.
    OcrOrDocumentAi,
    /// Routes agent tool work flagged as camera or webcam provider to the right queue, review gate, or agent packet.
    CameraOrWebcamProvider,
    /// Routes agent tool work flagged as hermes kanban to the right queue, review gate, or agent packet.
    HermesKanban,
    /// Routes agent tool work flagged as hermes cron or webhook to the right queue, review gate, or agent packet.
    HermesCronOrWebhook,
    /// Routes agent tool work flagged as postgres to the right queue, review gate, or agent packet.
    Postgres,
}
