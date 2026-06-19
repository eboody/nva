//! App-owned external tool-port rules.
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
/// for deterministic workflow evaluation. The trait is a read gate: returned
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
    /// Input rules for building the workflow packet from source-grounded records.
    pub struct Request {
        /// Reservation id copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub reservation_id: reservation::Id,
        /// Proposed status copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub proposed_status: reservation::Status,
        /// Rationale copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub rationale: Rationale,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision choices for rationale in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
    pub enum Rationale {
        /// Selects capacity unavailable for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        CapacityUnavailable,
        /// Selects policy hard stop for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        PolicyHardStop,
        /// Selects missing required information for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        MissingRequiredInformation,
        /// Selects manager review required for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        ManagerReviewRequired,
        /// Selects customer accepted offer for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
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
        /// Input rules for building the workflow packet from source-grounded records.
        pub struct Request {
            /// Provider copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub provider: Provider,
            /// Account copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub account: AccountId,
            /// Criteria copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub criteria: Criteria,
            /// Include copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub include: Vec<Include>,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Outcome used by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct Outcome {
            /// Provider copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub provider: Provider,
            /// Matched copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub matched: Match,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision choices for match in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
        pub enum Match {
            /// Selects customer for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            Customer(CustomerId),
            /// Selects pet for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            Pet(PetId),
            /// Selects reservation for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            Reservation(reservation::Id),
            /// Selects not found for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            NotFound,
            /// Candidates copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            Ambiguous {
                /// Candidates value stored on this variant.
                candidates: Vec<ExternalRecordId>,
            },
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision choices for criteria in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
        pub enum Criteria {
            /// Selects customer for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            Customer(CustomerId),
            /// Selects pet for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            Pet(PetId),
            /// Selects reservation for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            Reservation(reservation::Id),
            /// Selects external for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            External(ExternalRecordId),
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision choices for provider in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
    pub enum Provider {
        /// Selects gingr for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        Gingr,
        /// Selects pms for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        Pms,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision choices for include in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
    pub enum Include {
        /// Selects customer contact for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        CustomerContact,
        /// Selects pet profile for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        PetProfile,
        /// Selects reservation ledger for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
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

/// Payment helper rules that keep authorizations, refunds, and deposits auditable.
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
    /// Decision choices for subject in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
    pub enum Subject {
        /// Selects reservation deposit for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        ReservationDeposit(reservation::Id),
        /// Selects reservation balance for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        ReservationBalance(reservation::Id),
        /// Selects customer account for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        CustomerAccount(CustomerId),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision choices for capture policy in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
    pub enum CapturePolicy {
        /// Selects authorize only for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        AuthorizeOnly,
        /// Selects capture immediately for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        CaptureImmediately,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision choices for review reason in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
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
        /// Input rules for building the workflow packet from source-grounded records.
        pub struct Request {
            /// Subject copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub subject: Subject,
            /// Amount copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub amount: Money,
            /// Capture policy copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub capture_policy: CapturePolicy,
            /// Idempotency key copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub idempotency_key: IdempotencyKey,
        }

        /// Provider-facing result types kept separate from staff-review requests.
        pub mod provider {
            use super::*;

            pub use authorization_id::Id as AuthorizationId;

            #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
            /// Decision choices for result in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
            pub enum Result {
                /// Selects authorized for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
                Authorized {
                    /// Authorization id copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
                    authorization_id: AuthorizationId,
                    /// Amount copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
                    amount: Money,
                },
                /// Selects declined for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
                Declined {
                    /// Reason copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
                    reason: DeclineReason,
                },
                /// Selects requires human review for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
                RequiresHumanReview {
                    /// Reason copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
                    reason: ReviewReason,
                },
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
            /// Decision choices for decline reason in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
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
        /// Input rules for building the workflow packet from source-grounded records.
        pub struct Request {
            /// Payment reference copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub payment_reference: domain::payment::Reference,
            /// Amount copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub amount: Money,
            /// Reason copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub reason: Reason,
            /// Idempotency key copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub idempotency_key: IdempotencyKey,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision choices for reason in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
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
            /// Decision choices for result in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
            pub enum Result {
                /// Refund id copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
                Accepted {
                    /// Refund id value stored on this variant.
                    refund_id: RefundId,
                },
                /// Reason copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
                Rejected {
                    /// Reason value stored on this variant.
                    reason: RejectionReason,
                },
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
            /// Decision choices for rejection reason in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
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
        /// Record request used by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct RecordRequest {
            /// Reservation id copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub reservation_id: reservation::Id,
            /// Payment reference copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub payment_reference: domain::payment::Reference,
            /// Amount copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub amount: Money,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Record result used by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct RecordResult {
            /// Reservation id copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub reservation_id: reservation::Id,
            /// Deposit status copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub deposit_status: domain::payment::DepositStatus,
        }
    }
}

/// Customer-message drafting rules that never send without staff approval.
pub mod messaging {
    use super::*;

    #[async_trait]
    /// Defines the behavior required from a drafting participant in the tools workflow.
    pub trait Drafting: Send + Sync {
        /// Resolves the request into source-grounded lookup evidence without editing provider records or contacting customers.
        async fn draft_message(&self, request: draft::Request) -> Result<draft::Result>;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision choices for delivery channel in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
    pub enum DeliveryChannel {
        /// Selects email for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        Email,
        /// Selects sms for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        Sms,
        /// Selects portal for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        Portal,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision choices for recipient in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
    pub enum Recipient {
        /// Selects customer for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        Customer(CustomerId),
        /// Selects staff for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        Staff(domain::entities::StaffId),
        /// Selects manager for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        Manager(domain::entities::ManagerId),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision choices for review policy in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
    pub enum ReviewPolicy {
        /// Selects draft only for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        DraftOnly,
        /// Selects manager approval required for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        ManagerApprovalRequired,
    }

    /// Draft payloads produced for review instead of direct customer/provider actions.
    pub mod draft {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Input rules for building the workflow packet from source-grounded records.
        pub struct Request {
            /// Channel copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub channel: DeliveryChannel,
            /// Recipient copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub recipient: Recipient,
            /// Body copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub body: message_body::Body,
            /// Review copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub review: ReviewPolicy,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Result used by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct Result {
            /// Draft id copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub draft_id: draft_update::draft::Id,
            /// Status copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub status: Status,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision choices for status in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
        pub enum Status {
            /// Routes the item to drafted for staff queueing, review, and downstream agent context.
            Drafted,
            /// Routes the item to drafted requires review for staff queueing, review, and downstream agent context.
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
        /// Intake request used by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct IntakeRequest {
            /// Document copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub document: reference::Ref,
            /// Source copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub source: Source,
            /// Expected content copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub expected_content: ExpectedContent,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Intake result used by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct IntakeResult {
            /// Document copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub document: reference::Ref,
            /// Classification copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub classification: Classification,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision choices for source in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
        pub enum Source {
            /// Selects customer upload for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            CustomerUpload,
            /// Selects staff scan for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            StaffScan,
            /// Selects portal import for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            PortalImport,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision choices for expected content in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
        pub enum ExpectedContent {
            /// Selects vaccine proof for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            VaccineProof,
            /// Selects medication instructions for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            MedicationInstructions,
            /// Selects boarding agreement for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            BoardingAgreement,
            /// Selects incident report for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            IncidentReport,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision choices for classification in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
        pub enum Classification {
            /// Selects matches expected content for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            MatchesExpectedContent,
            /// Selects mismatch for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            Mismatch,
            /// Selects unreadable for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
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
        /// Input rules for building the workflow packet from source-grounded records.
        pub struct Request {
            /// Document copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub document: super::document::reference::Ref,
            /// Expected content copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub expected_content: super::document::ExpectedContent,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision choices for result in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
        pub enum Result {
            /// Text copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            Extracted {
                /// Text value stored on this variant.
                text: extracted_text::Text,
            },
            /// Reason copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            NeedsHumanReview {
                /// Reason value stored on this variant.
                reason: ReviewReason,
            },
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision choices for review reason in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
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
    /// Snapshot request used by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
    pub struct SnapshotRequest {
        /// Location id copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub location_id: LocationId,
        /// Camera id copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub camera_id: CameraId,
        /// Purpose copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub purpose: CapturePurpose,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision choices for snapshot result in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
    pub enum SnapshotResult {
        /// Media ref copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        Captured {
            /// Media ref value stored on this variant.
            media_ref: Ref,
        },
        /// Reason copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        Unavailable {
            /// Reason value stored on this variant.
            reason: UnavailableReason,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision choices for capture purpose in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
    pub enum CapturePurpose {
        /// Selects pet status check for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        PetStatusCheck(PetId),
        /// Selects facility safety check for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        FacilitySafetyCheck,
        /// Selects incident review for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        IncidentReview(reservation::Id),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision choices for unavailable reason in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
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
        /// Draft request used by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct DraftRequest {
            /// Title copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub title: workflow::task::Title,
            /// Body copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub body: workflow::task::Body,
            /// Queue copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub queue: QueueName,
            /// Trigger copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub trigger: Trigger,
        }

        /// Kanban task records returned after staff-work items are drafted.
        pub mod kanban {
            use super::*;

            pub use task_id::Id as TaskId;

            #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
            /// Draft result used by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
            pub struct DraftResult {
                /// Task id copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
                pub task_id: TaskId,
                /// Status copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
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
        /// Draft request used by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct DraftRequest {
            /// Name copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub name: Name,
            /// Cadence copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub cadence: Cadence,
            /// Queue copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub queue: QueueName,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Draft result used by the agent tool workflow; it exposes tightly-scoped read/draft helpers agents can call behind review gates.
        pub struct DraftResult {
            /// Schedule id copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub schedule_id: Id,
            /// Status copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
            pub status: DraftStatus,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Decision choices for cadence in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
        pub enum Cadence {
            /// Selects daily for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            Daily,
            /// Selects hourly for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
            Hourly,
            /// Selects manual only for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
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
    /// Decision choices for trigger in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
    pub enum Trigger {
        /// Selects workflow review for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        WorkflowReview,
        /// Selects operations brief for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        OperationsBrief,
        /// Selects integration failure for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
        IntegrationFailure,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision choices for draft status in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
    pub enum DraftStatus {
        /// Routes the item to drafted for staff queueing, review, and downstream agent context.
        Drafted,
        /// Routes the item to drafted requires review for staff queueing, review, and downstream agent context.
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
/// Decision choices for external tool candidate in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum ExternalToolCandidate {
    /// Selects gingr portal for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    GingrPortal,
    /// Selects payment provider for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    PaymentProvider,
    /// Selects sms provider for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    SmsProvider,
    /// Selects email provider for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    EmailProvider,
    /// Selects file storage for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    FileStorage,
    /// Selects ocr or document ai for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    OcrOrDocumentAi,
    /// Selects camera or webcam provider for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    CameraOrWebcamProvider,
    /// Selects hermes kanban for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    HermesKanban,
    /// Selects hermes cron or webhook for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    HermesCronOrWebhook,
    /// Selects postgres for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    Postgres,
}
