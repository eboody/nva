use chrono::{DateTime, NaiveDate, Utc};
use domain::{document, entities, pet, policy, source, vaccine, workflow};
use serde::Deserialize;
use uuid::Uuid;

use crate::{booking_triage, checkout_completion, daily_update};

#[derive(Debug, thiserror::Error)]
/// Decision choices for error in the local smoke-test workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum Error {
    #[error("local smoke fixture is not valid JSON: {0}")]
    /// Identifies invalid fixture as the reason the workflow must stop, retry, or request review.
    InvalidFixture(#[from] serde_json::Error),
    #[error("local smoke fixture is missing a required semantic value: {0}")]
    /// Identifies invalid domain value as the reason the workflow must stop, retry, or request review.
    InvalidDomainValue(String),
    #[error("local smoke daily-update preview failed: {0}")]
    /// Identifies daily update as the reason the workflow must stop, retry, or request review.
    DailyUpdate(#[from] daily_update::Error),
}

/// Result type returned by fallible local smoke operations.
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Source event key used by the local smoke-test workflow; it exercises the local shell with deterministic fixtures and no external side effects.
pub struct SourceEventKey(String);

impl SourceEventKey {}

impl AsRef<str> for SourceEventKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Decision choices for stage in the local smoke-test workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum Stage {
    /// Selects inquiry for the local smoke decision model so the app can choose a review, evidence, or draft path without taking live action.
    Inquiry,
    /// Selects profile for the local smoke decision model so the app can choose a review, evidence, or draft path without taking live action.
    Profile,
    /// Selects vaccine docs for the local smoke decision model so the app can choose a review, evidence, or draft path without taking live action.
    VaccineDocs,
    /// Selects booking triage for the local smoke decision model so the app can choose a review, evidence, or draft path without taking live action.
    BookingTriage,
    /// Selects confirmation draft for the local smoke decision model so the app can choose a review, evidence, or draft path without taking live action.
    ConfirmationDraft,
    /// Selects check in today view for the local smoke decision model so the app can choose a review, evidence, or draft path without taking live action.
    CheckInTodayView,
    /// Selects staff note daily update draft for the local smoke decision model so the app can choose a review, evidence, or draft path without taking live action.
    StaffNoteDailyUpdateDraft,
    /// Selects checkout completion for the local smoke decision model so the app can choose a review, evidence, or draft path without taking live action.
    CheckoutCompletion,
    /// Selects follow up retention for the local smoke decision model so the app can choose a review, evidence, or draft path without taking live action.
    FollowUpRetention,
}

impl Stage {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Smoke gates used by the local smoke-test workflow; it exercises the local shell with deterministic fixtures and no external side effects.
pub struct SmokeBoundaries {
    draft_only_ai: bool,
    blocks_live_customer_sends: bool,
    blocks_provider_or_pms_mutations: bool,
    blocks_payment_refund_or_discount_actions: bool,
}

impl SmokeBoundaries {
    /// Returns the draft only ai evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn draft_only_ai(&self) -> bool {
        self.draft_only_ai
    }

    /// Returns the blocks live customer sends evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn blocks_live_customer_sends(&self) -> bool {
        self.blocks_live_customer_sends
    }

    /// Returns the blocks provider or pms mutations evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn blocks_provider_or_pms_mutations(&self) -> bool {
        self.blocks_provider_or_pms_mutations
    }

    /// Returns the blocks payment refund or discount actions evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn blocks_payment_refund_or_discount_actions(&self) -> bool {
        self.blocks_payment_refund_or_discount_actions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Review evidence ref used by the local smoke-test workflow; it exercises the local shell with deterministic fixtures and no external side effects.
pub struct ReviewEvidenceRef(String);

impl ReviewEvidenceRef {}

impl AsRef<str> for ReviewEvidenceRef {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Smoke confirmation draft used by the local smoke-test workflow; it exercises the local shell with deterministic fixtures and no external side effects.
pub struct SmokeConfirmationDraft {
    review_gate: booking_triage::ApprovalGate,
}

impl SmokeConfirmationDraft {
    /// Reports whether the local smoke-test workflow satisfies the requires customer message approval safety condition.
    pub const fn requires_customer_message_approval(&self) -> bool {
        matches!(
            self.review_gate,
            booking_triage::ApprovalGate::CustomerMessageApproval
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Reservation label used by the local smoke-test workflow; it exercises the local shell with deterministic fixtures and no external side effects.
pub struct ReservationLabel(String);

impl AsRef<str> for ReservationLabel {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Today view used by the local smoke-test workflow; it exercises the local shell with deterministic fixtures and no external side effects.
pub struct TodayView {
    reservation_labels: Vec<ReservationLabel>,
    status: entities::reservation::Status,
}

impl TodayView {
    /// Returns the reservation labels evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn reservation_labels(&self) -> &[ReservationLabel] {
        &self.reservation_labels
    }

    /// Returns the status evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn status(&self) -> &entities::reservation::Status {
        &self.status
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Checkout completion used by the local smoke-test workflow; it exercises the local shell with deterministic fixtures and no external side effects.
pub struct CheckoutCompletion {
    packet: checkout_completion::ReviewPacket,
}

impl CheckoutCompletion {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Evidence-only retention dispositions for the local smoke workflow.
pub enum RetentionNextAction {
    /// Preserves serialized retention history as suppressed evidence without creating a candidate, queue, task, or draft.
    PreserveSuppressedEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Retention follow up used by the local smoke-test workflow; it exercises the local shell with deterministic fixtures and no external side effects.
pub struct RetentionFollowUp {
    next_action: RetentionNextAction,
    review_gate: policy::ReviewGate,
}

impl RetentionFollowUp {
    /// Returns the next action evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn next_action(&self) -> RetentionNextAction {
        self.next_action
    }

    /// Returns the review gate evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn review_gate(&self) -> policy::ReviewGate {
        self.review_gate.clone()
    }
}

impl FullChainEvidence {
    /// Returns the source event key evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn source_event_key(&self) -> &SourceEventKey {
        &self.source_event_key
    }

    /// Returns the gates evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn boundaries(&self) -> &SmokeBoundaries {
        &self.boundaries
    }

    /// Returns the booking packet evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn booking_packet(&self) -> &booking_triage::StaffEvaluationPacket {
        &self.booking_packet
    }

    /// Returns the confirmation draft evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn confirmation_draft(&self) -> &SmokeConfirmationDraft {
        &self.confirmation_draft
    }

    /// Returns the today view evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn today_view(&self) -> &TodayView {
        &self.today_view
    }

    /// Returns the daily update preview evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn daily_update_preview(&self) -> &daily_update::MvpPreview {
        &self.daily_update_preview
    }

    /// Returns the checkout completion evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn checkout_completion(&self) -> &CheckoutCompletion {
        &self.checkout_completion
    }

    /// Returns the retention follow up evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn retention_follow_up(&self) -> &RetentionFollowUp {
        &self.retention_follow_up
    }

    /// Returns the review gated evidence refs evidence available to local smoke-test review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn review_gated_evidence_refs(&self) -> &[ReviewEvidenceRef] {
        &self.review_gated_evidence_refs
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InquiryRecord {
    source_event_key: SourceEventKey,
    requested_service: String,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProfileEvidence {
    customer: entities::Customer,
    pet: entities::Pet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VaccineDocumentEvidence {
    document: entities::Document,
    record: entities::VaccineRecord,
}

impl SmokeIds {}
