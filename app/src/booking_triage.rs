//! Booking triage rules for deterministic review before agent drafting.
//!
//! ## Operator summary
//!
//! Staff use booking triage to decide which reservation queue owns the next action: ready for
//! staff approval, missing information, vaccine/document review, special care/behavior/payment
//! review, waitlist/availability review, or failed-safe data cleanup. The workflow reduces labor
//! by assembling source-backed reservation, pet-profile, policy, deposit, and hard-stop evidence
//! into one staff packet before any agent drafts customer-safe language.
//!
//! Booking triage is not allowed to confirm or reject a booking, promise availability, hold or
//! release capacity, assign rooms or play groups, mutate provider/PMS records, send customer
//! messages, clear vaccine/care/behavior exceptions, or move payment/deposit money. Provider/PMS
//! lifecycle state, approved location policy, verified vaccine/document facts, trusted payment
//! records, staff approvals, and source snapshots remain authoritative. Review gates protect pets,
//! customers, and staff whenever facts are missing, stale, conflicting, sensitive, payment-related,
//! or require manager/care/medical/behavior/customer-message approval.
//!
//! The typestate request machine models the safe sequence for triage evidence:
//! intake, pet profile attachment, reservation fact attachment, deterministic
//! review, and staff-ready handoff. The machine's macro helper pages are a
//! `statum` implementation detail; this module documents the operational rules
//! here and on the source state variants so external readers understand that the
//! `Request`/state APIs emitted by the macro enforce evidence order rather than granting live
//! booking authority.
//! ```
//! use app::booking_triage as triage;
//!
//! let vaccine_review = triage::rule::ReviewFinding::builder()
//!     .rule_id(triage::rule::Id::VaccineRequirements)
//!     .failure_code(triage::FailureCode::MissingOrUnverifiedVaccine)
//!     .readiness_bucket(triage::ReadinessBucket::VaccinePending)
//!     .human_approval_required(triage::ApprovalGate::MedicalDocumentReview)
//!     .evidence_refs(vec![triage::EvidenceRef::try_new(
//!         "provider:reservation:fixture-123:vaccine-expired",
//!     )?])
//!     .build();
//!
//! let deterministic = triage::DeterministicResult::evaluate(vec![
//!     triage::rule::Evaluation::needs_human_approval(vaccine_review),
//! ]);
//!
//! assert_eq!(deterministic.recommended_status(), triage::ReadinessBucket::VaccinePending);
//! assert!(deterministic.requires(triage::ApprovalGate::MedicalDocumentReview));
//! assert_eq!(
//!     deterministic.staff_decision_boundary(),
//!     triage::StaffDecisionBoundary::ReviewPacketOnly,
//! );
//! assert!(deterministic.blocked_actions().contains(&triage::BlockedAction::ConfirmBooking));
//! assert!(deterministic.blocked_actions().contains(&triage::BlockedAction::SendCustomerMessage));
//! assert!(deterministic.blocked_actions().contains(&triage::BlockedAction::MutateProviderRecord));
//!
//! let packet = triage::StaffEvaluationPacket::new(
//!     domain::entities::reservation::Id::new(uuid::Uuid::from_u128(123)),
//!     deterministic,
//! );
//! let draft = triage::ConfirmationDraft::new(
//!     triage::CustomerMessageDraft::try_new("We can draft this only after staff review.")?,
//! );
//!
//! assert_eq!(
//!     packet.try_with_confirmation_draft(draft).unwrap_err(),
//!     triage::ConfirmationDraftError::DeterministicGateNotReadyForDraft,
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
use nutype::nutype;
use serde::{Deserialize, Serialize};

use domain::entities::reservation as reservation_entity;
use domain::pet;

mod workflow;

pub use workflow::{
    AgentRecommendedAction, AiRecommendation, ApprovalGate, AuditEventDraft, BlockedAction,
    BlockerEvidence, BlockerKind, ConfirmationDraft, ConfirmationDraftError, CustomerMessageDraft,
    DeterministicResult, EvidenceRef, FailureCode, MissingInfoDraft, MissingInfoReason, PetProfile,
    PetProfileCompleteness, PolicyAttachedData, PolicySnapshot, ReadinessBucket,
    RecommendationText, SafeAgentAction, StaffDecisionBoundary, StaffEvaluationPacket, rule,
};
