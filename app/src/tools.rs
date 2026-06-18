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
/// Read-only customer/reservation evidence store exposed to app workflows.
///
/// Implementations fetch source-grounded customer, pet, and reservation records
/// for deterministic workflow evaluation. The trait is a read boundary: returned
/// facts may be summarized or used to prepare review packets, but callers must
/// use separate draft/review ports for any customer message, booking update, or
/// provider-system write.
pub trait CustomerStore: Send + Sync {
    /// Fetches the customer record identified by `id` as app-owned evidence.
    ///
    /// The returned customer can ground triage, retention, or communication
    /// drafts. A missing record should surface as [`Error::NotFound`], and this
    /// read must not create, edit, merge, or contact the customer.
    async fn get_customer(&self, id: CustomerId) -> Result<Customer>;
    /// Fetches the pet profile identified by `id` for policy and care-context checks.
    ///
    /// The result may inform vaccine, temperament, service-line, or daily-care
    /// review packets. It is evidence only; it must not change pet profile fields
    /// or treat incomplete data as permission to invent missing facts.
    async fn get_pet(&self, id: PetId) -> Result<Pet>;
    /// Fetches the reservation identified by `id` for source-grounded workflow state.
    ///
    /// The result may drive booking triage, daily updates, checkout completion,
    /// or retention follow-up decisions. It must not confirm, cancel, check in,
    /// check out, or otherwise mutate the reservation.
    async fn get_reservation(&self, id: reservation::Id) -> Result<Reservation>;
}

#[async_trait]
/// Reservation-system port for read checks and review-held draft updates.
///
/// This trait separates source availability checks and draft reservation updates
/// from live provider/PMS mutation. Implementations may consult external systems
/// or create internal draft records, but workflow code must still apply review
/// gates before any booking promise, status change, deposit, or customer message.
pub trait ReservationSystem: Send + Sync {
    /// Checks capacity/policy availability without confirming a booking.
    ///
    /// The request carries location, optional reservation context, and service
    /// notes. The outcome should identify available/unavailable/review-required
    /// evidence, including any capacity snapshot id, and must not reserve space
    /// unless the implementation explicitly models that as a review-safe hold.
    async fn check_availability(
        &self,
        request: availability::Request,
    ) -> Result<availability::Outcome>;
    /// Persists a proposed reservation status change as a draft for review.
    ///
    /// The returned draft id identifies the review artifact. Implementations must
    /// not write the proposed status to the provider/PMS or represent the draft as
    /// a customer-visible booking decision before the required app/human gates run.
    async fn draft_reservation_update(
        &self,
        request: draft_update::Request,
    ) -> Result<draft_update::draft::Id>;
}

#[async_trait]
/// Structured agent runtime used only after deterministic app context is built.
///
/// Implementations execute a model/tool runner against typed input and return a
/// typed workflow result for validation. The runtime is not a shortcut around
/// source evidence, review gates, or blocked live actions.
pub trait AgentRuntime: Send + Sync {
    /// Runs one typed agent call for a workflow event and input packet.
    ///
    /// The call returns a structured workflow result for the app validator to
    /// accept or reject. Even successful output remains draft/evidence until the
    /// owning workflow applies policy gates.
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
    /// Availability lookup input for a location/service request.
    ///
    /// This packet supplies enough source context for an adapter to check capacity
    /// or policy hard-stops while keeping the workflow in read/draft mode.
    pub struct Request {
        /// Resort/location whose capacity or policy should be checked.
        pub location_id: LocationId,
        /// Reservation context when the check is for an existing booking workflow.
        pub reservation_id: Option<reservation::Id>,
        /// Staff/customer service notes that clarify requested dates, service line,
        /// pet constraints, or missing information for the availability check.
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
    /// Availability decision returned to the app as source evidence, not a booking promise.
    pub struct Outcome {
        /// Deterministic availability result and the evidence required to explain it.
        pub decision: Decision,
    }

    impl Outcome {
        /// Reports whether capacity was found without implying customer-visible confirmation.
        pub fn is_available(&self) -> bool {
            matches!(self.decision, Decision::Available { .. })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Availability result used to decide whether staff may review a draft offer.
    pub enum Decision {
        /// Capacity evidence exists for a reviewable booking offer.
        Available {
            /// Why the adapter considers the requested service/date capacity available.
            reason: SuccessReason,
            /// Snapshot or hold identifier staff can inspect before relying on the result.
            capacity_snapshot_id: CapacitySnapshotId,
        },
        /// Capacity or policy evidence prevents a safe draft offer without review.
        Unavailable {
            /// Reason the workflow should deny, defer, or escalate the draft offer.
            reason: DenialReason,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Positive availability evidence reasons.
    pub enum SuccessReason {
        /// A capacity snapshot or review-safe hold exists for staff to evaluate.
        CapacityHeld,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Negative or escalation availability evidence reasons.
    pub enum DenialReason {
        /// Source capacity data shows no safe space/service availability.
        CapacityUnavailable,
        /// A deterministic policy rule blocks the requested booking path.
        PolicyHardStop,
        /// Staff/customer information is incomplete, so the workflow cannot infer availability.
        MissingRequiredInformation,
        /// The adapter found ambiguity that requires staff or manager review.
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
        /// Source-derived Reservation id retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub reservation_id: reservation::Id,
        /// Source-derived Proposed status retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub proposed_status: reservation::Status,
        /// Source-derived Rationale retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub rationale: Rationale,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision taxonomy for rationale in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
    pub enum Rationale {
        /// Represents capacity unavailable in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        CapacityUnavailable,
        /// Represents policy hard stop in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        PolicyHardStop,
        /// Represents missing required information in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        MissingRequiredInformation,
        /// Represents manager review required in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        ManagerReviewRequired,
        /// Represents customer accepted offer in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
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
    /// Read-only provider lookup port for turning external identifiers into reviewable app evidence.
    pub trait Lookup: Send + Sync {
        /// Resolves the request into source-grounded lookup evidence without editing provider records or contacting customers.
        async fn lookup(&self, request: lookup::Request) -> Result<lookup::Outcome>;
    }

    /// Lookup requests and outcomes for pulling provider context into review packets.
    pub mod lookup {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Input contract for building the workflow packet from source-grounded records.
        pub struct Request {
            /// Source-derived Provider retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub provider: Provider,
            /// Source-derived Account retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub account: AccountId,
            /// Source-derived Criteria retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub criteria: Criteria,
            /// Source-derived Include retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub include: Vec<Include>,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Outcome carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct Outcome {
            /// Source-derived Provider retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub provider: Provider,
            /// Source-derived Matched retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub matched: Match,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision taxonomy for match in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
        pub enum Match {
            /// Represents customer in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            Customer(CustomerId),
            /// Represents pet in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            Pet(PetId),
            /// Represents reservation in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            Reservation(reservation::Id),
            /// Represents not found in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            NotFound,
            /// Source-derived Candidates retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            Ambiguous {
                /// Candidates carried by this variant.
                candidates: Vec<ExternalRecordId>,
            },
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision taxonomy for criteria in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
        pub enum Criteria {
            /// Represents customer in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            Customer(CustomerId),
            /// Represents pet in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            Pet(PetId),
            /// Represents reservation in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            Reservation(reservation::Id),
            /// Represents external in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            External(ExternalRecordId),
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision taxonomy for provider in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
    pub enum Provider {
        /// Represents gingr in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        Gingr,
        /// Represents pms in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        Pms,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision taxonomy for include in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
    pub enum Include {
        /// Represents customer contact in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        CustomerContact,
        /// Represents pet profile in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        PetProfile,
        /// Represents reservation ledger in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
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
        /// Evaluates whether a payment operation is policy-authorized without moving money or changing invoices.
        async fn authorize(
            &self,
            request: authorization::Request,
        ) -> Result<authorization::provider::Result>;
        /// Resolves the request into source-grounded lookup evidence without editing provider records or contacting customers.
        async fn refund(&self, request: refund::Request) -> Result<refund::provider::Result>;
        /// Records a reviewed deposit artifact after authorization while keeping payment movement behind explicit policy gates.
        async fn record_deposit(
            &self,
            request: deposit::RecordRequest,
        ) -> Result<deposit::RecordResult>;
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision taxonomy for subject in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
    pub enum Subject {
        /// Represents reservation deposit in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        ReservationDeposit(reservation::Id),
        /// Represents reservation balance in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        ReservationBalance(reservation::Id),
        /// Represents customer account in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        CustomerAccount(CustomerId),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision taxonomy for capture policy in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
    pub enum CapturePolicy {
        /// Represents authorize only in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        AuthorizeOnly,
        /// Represents capture immediately in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        CaptureImmediately,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision taxonomy for review reason in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
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
            /// Source-derived Subject retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub subject: Subject,
            /// Source-derived Amount retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub amount: Money,
            /// Source-derived Capture policy retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub capture_policy: CapturePolicy,
            /// Source-derived Idempotency key retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub idempotency_key: IdempotencyKey,
        }

        /// Provider-facing result types kept separate from staff-review requests.
        pub mod provider {
            use super::*;

            pub use authorization_id::Id as AuthorizationId;

            #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
            /// Decision taxonomy for result in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
            pub enum Result {
                /// Represents authorized in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
                Authorized {
                    /// Source-derived Authorization id retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
                    authorization_id: AuthorizationId,
                    /// Source-derived Amount retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
                    amount: Money,
                },
                /// Represents declined in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
                Declined {
                    /// Source-derived Reason retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
                    reason: DeclineReason,
                },
                /// Represents requires human review in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
                RequiresHumanReview {
                    /// Source-derived Reason retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
                    reason: ReviewReason,
                },
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
            /// Decision taxonomy for decline reason in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
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
            /// Source-derived Payment reference retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub payment_reference: domain::payment::Reference,
            /// Source-derived Amount retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub amount: Money,
            /// Source-derived Reason retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub reason: Reason,
            /// Source-derived Idempotency key retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub idempotency_key: IdempotencyKey,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision taxonomy for reason in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
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
            /// Decision taxonomy for result in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
            pub enum Result {
                /// Source-derived Refund id retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
                Accepted {
                    /// Refund id carried by this variant.
                    refund_id: RefundId,
                },
                /// Source-derived Reason retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
                Rejected {
                    /// Reason carried by this variant.
                    reason: RejectionReason,
                },
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
            /// Decision taxonomy for rejection reason in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
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
            /// Source-derived Reservation id retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub reservation_id: reservation::Id,
            /// Source-derived Payment reference retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub payment_reference: domain::payment::Reference,
            /// Source-derived Amount retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub amount: Money,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Record result carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct RecordResult {
            /// Source-derived Reservation id retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub reservation_id: reservation::Id,
            /// Source-derived Deposit status retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
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
        /// Resolves the request into source-grounded lookup evidence without editing provider records or contacting customers.
        async fn draft_message(&self, request: draft::Request) -> Result<draft::Result>;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision taxonomy for delivery channel in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
    pub enum DeliveryChannel {
        /// Represents email in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        Email,
        /// Represents sms in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        Sms,
        /// Represents portal in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        Portal,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision taxonomy for recipient in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
    pub enum Recipient {
        /// Represents customer in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        Customer(CustomerId),
        /// Represents staff in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        Staff(domain::entities::StaffId),
        /// Represents manager in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        Manager(domain::entities::ManagerId),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision taxonomy for review policy in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
    pub enum ReviewPolicy {
        /// Represents draft only in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        DraftOnly,
        /// Represents manager approval required in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        ManagerApprovalRequired,
    }

    /// Draft payloads produced for review instead of direct customer/provider actions.
    pub mod draft {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Input contract for building the workflow packet from source-grounded records.
        pub struct Request {
            /// Source-derived Channel retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub channel: DeliveryChannel,
            /// Source-derived Recipient retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub recipient: Recipient,
            /// Source-derived Body retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub body: message_body::Body,
            /// Source-derived Review retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub review: ReviewPolicy,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Result carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct Result {
            /// Source-derived Draft id retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub draft_id: draft_update::draft::Id,
            /// Source-derived Status retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub status: Status,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision taxonomy for status in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
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
            /// Resolves the request into source-grounded lookup evidence without editing provider records or contacting customers.
            async fn intake_document(&self, request: IntakeRequest) -> Result<IntakeResult>;
            /// Resolves the request into source-grounded lookup evidence without editing provider records or contacting customers.
            async fn extract_ocr(&self, request: super::ocr::Request)
            -> Result<super::ocr::Result>;
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Intake request carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct IntakeRequest {
            /// Source-derived Document retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub document: reference::Ref,
            /// Source-derived Source retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub source: Source,
            /// Source-derived Expected content retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub expected_content: ExpectedContent,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Intake result carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct IntakeResult {
            /// Source-derived Document retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub document: reference::Ref,
            /// Source-derived Classification retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub classification: Classification,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision taxonomy for source in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
        pub enum Source {
            /// Represents customer upload in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            CustomerUpload,
            /// Represents staff scan in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            StaffScan,
            /// Represents portal import in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            PortalImport,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision taxonomy for expected content in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
        pub enum ExpectedContent {
            /// Represents vaccine proof in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            VaccineProof,
            /// Represents medication instructions in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            MedicationInstructions,
            /// Represents boarding agreement in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            BoardingAgreement,
            /// Represents incident report in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            IncidentReport,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision taxonomy for classification in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
        pub enum Classification {
            /// Represents matches expected content in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            MatchesExpectedContent,
            /// Represents mismatch in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            Mismatch,
            /// Represents unreadable in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
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
            /// Source-derived Document retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub document: super::document::reference::Ref,
            /// Source-derived Expected content retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub expected_content: super::document::ExpectedContent,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision taxonomy for result in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
        pub enum Result {
            /// Source-derived Text retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            Extracted {
                /// Text carried by this variant.
                text: extracted_text::Text,
            },
            /// Source-derived Reason retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            NeedsHumanReview {
                /// Reason carried by this variant.
                reason: ReviewReason,
            },
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision taxonomy for review reason in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
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
        /// Resolves the request into source-grounded lookup evidence without editing provider records or contacting customers.
        async fn request_snapshot(&self, request: SnapshotRequest) -> Result<SnapshotResult>;
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Snapshot request carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
    pub struct SnapshotRequest {
        /// Source-derived Location id retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub location_id: LocationId,
        /// Source-derived Camera id retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub camera_id: CameraId,
        /// Source-derived Purpose retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub purpose: CapturePurpose,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision taxonomy for snapshot result in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
    pub enum SnapshotResult {
        /// Source-derived Media ref retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        Captured {
            /// Media ref carried by this variant.
            media_ref: Ref,
        },
        /// Source-derived Reason retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        Unavailable {
            /// Reason carried by this variant.
            reason: UnavailableReason,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision taxonomy for capture purpose in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
    pub enum CapturePurpose {
        /// Represents pet status check in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        PetStatusCheck(PetId),
        /// Represents facility safety check in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        FacilitySafetyCheck,
        /// Represents incident review in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        IncidentReview(reservation::Id),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision taxonomy for unavailable reason in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
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
        /// Creates an internal task draft for staff review rather than directly assigning labor or changing schedules.
        async fn draft_task(
            &self,
            request: task::DraftRequest,
        ) -> Result<task::kanban::DraftResult>;
        /// Creates a schedule-change draft for manager review rather than mutating staff rosters.
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
            /// Source-derived Title retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub title: workflow::task::Title,
            /// Source-derived Body retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub body: workflow::task::Body,
            /// Source-derived Queue retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub queue: QueueName,
            /// Source-derived Trigger retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub trigger: Trigger,
        }

        /// Kanban task records returned after staff-work items are drafted.
        pub mod kanban {
            use super::*;

            pub use task_id::Id as TaskId;

            #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
            /// Draft result carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
            pub struct DraftResult {
                /// Source-derived Task id retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
                pub task_id: TaskId,
                /// Source-derived Status retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
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
            /// Source-derived Name retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub name: Name,
            /// Source-derived Cadence retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub cadence: Cadence,
            /// Source-derived Queue retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub queue: QueueName,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Draft result carried by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct DraftResult {
            /// Source-derived Schedule id retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub schedule_id: Id,
            /// Source-derived Status retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub status: DraftStatus,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision taxonomy for cadence in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
        pub enum Cadence {
            /// Represents daily in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            Daily,
            /// Represents hourly in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
            Hourly,
            /// Represents manual only in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
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
    /// Decision taxonomy for trigger in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
    pub enum Trigger {
        /// Represents workflow review in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        WorkflowReview,
        /// Represents operations brief in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        OperationsBrief,
        /// Represents integration failure in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
        IntegrationFailure,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision taxonomy for draft status in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
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
/// Decision taxonomy for external tool candidate in the agent tool workflow; each value carries operational meaning for source-grounded routing and review.
pub enum ExternalToolCandidate {
    /// Represents gingr portal in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    GingrPortal,
    /// Represents payment provider in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    PaymentProvider,
    /// Represents sms provider in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    SmsProvider,
    /// Represents email provider in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    EmailProvider,
    /// Represents file storage in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    FileStorage,
    /// Represents ocr or document ai in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    OcrOrDocumentAi,
    /// Represents camera or webcam provider in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    CameraOrWebcamProvider,
    /// Represents hermes kanban in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    HermesKanban,
    /// Represents hermes cron or webhook in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    HermesCronOrWebhook,
    /// Represents postgres in the agent tool decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    Postgres,
}
