//! Checkout-completion workflow rules for staff departure handoff and closeout review.
//!
//! ## Operator summary
//!
//! Staff use this workflow to route a departure to manager-gated handoff review or
//! source-status reconciliation. It compares observed source reservation status with caller-reported
//! front-desk handoff evidence such as belongings return, care summary, and departure-note review.
//! Serialized handoff evidence never creates verified-checkout or retention authority.
//!
//! The workflow may summarize checkout evidence, create an internal handoff task, and produce
//! audit-event drafts. It never grants the retention-follow-up draft action from serialized packet
//! state. It is not
//! allowed to close a live PMS/provider record, send a customer message, apply a checkout status
//! without staff/source agreement, release capacity, waive/discount/refund, collect payment, or
//! move money. Payment and closeout surfaces remain review queues, not autonomous execution.
//!
//! Source facts remain authoritative in their own systems: `domain::source::Provenance` and
//! `domain::source::reservation::Status` for observed provider state, staff-submitted handoff
//! evidence for belongings and departure notes, `domain::entities::reservation::Status` for the
//! normalized lifecycle suggestion, and approved payment/ledger records for balances, refunds,
//! discounts, or waivers. Review gates protect pets, customers, and staff by requiring manager
//! approval when source or handoff evidence is incomplete and customer-message approval before any
//! departure or retention copy leaves draft form.

use chrono::{DateTime, Utc};
use domain::{entities, policy, source};
use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::fmt;

pub use domain::boarding::handoff::DepartureTaskDraft as StaffTaskDraft;
pub use domain::payment::CheckoutException as PaymentException;
pub use domain::reservation::CheckoutCompletionDisposition as ReportedDisposition;
pub use domain::reservation::CheckoutSourceException as SourceException;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Staff minutes used to compare manual checkout audit effort with packet-review effort.
pub struct LaborMinutes(u16);

impl LaborMinutes {
    /// Validates non-zero minutes before a checkout labor estimate can appear in a packet.
    pub const fn try_new(value: u16) -> Result<Self, &'static str> {
        if value == 0 {
            return Err("checkout labor minutes must be greater than zero");
        }
        Ok(Self(value))
    }

    /// Returns the numeric labor-minute value for review and tests.
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Named unresolved checkout work that staff can clear before final closeout.
pub enum UnresolvedException {
    /// Belongings have not been verified as returned to the customer.
    Belongings,
    /// Care summary or departure notes still need staff/manager review.
    Care,
    /// Payment, refund, discount, waiver, or balance issue retained for ledger/PMS review.
    Payment(PaymentException),
    /// Source/PMS checkout state or provider record conflict retained for reconciliation.
    Source(SourceException),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reviewable labor estimate for the open-stay audit workflow.
pub struct LaborImpact {
    manual_audit_minutes: LaborMinutes,
    packet_review_minutes: LaborMinutes,
}

impl LaborImpact {
    /// Captures the expected manual audit effort and packet-review effort without claiming realized savings.
    pub const fn new(
        manual_audit_minutes: LaborMinutes,
        packet_review_minutes: LaborMinutes,
    ) -> Self {
        Self {
            manual_audit_minutes,
            packet_review_minutes,
        }
    }

    /// Returns the manual checkout audit effort estimated for front-desk staff.
    pub const fn manual_audit_minutes(&self) -> LaborMinutes {
        self.manual_audit_minutes
    }

    /// Returns the packet review effort estimated for front-desk staff.
    pub const fn packet_review_minutes(&self) -> LaborMinutes {
        self.packet_review_minutes
    }

    /// Returns a reported estimate difference when packet review is lower than manual audit effort; this is not realized savings.
    pub const fn reported_estimated_minutes_difference(&self) -> Option<u16> {
        let manual = self.manual_audit_minutes.get();
        let review = self.packet_review_minutes.get();
        if manual > review {
            Some(manual - review)
        } else {
            None
        }
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 1200),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct CareSummary(String);

impl fmt::Debug for CareSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CareSummary(<redacted>)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Caller-reported belongings-status labels retained as compatibility evidence; they route no queue, draft, gate, or checkout authority.
pub enum BelongingsStatus {
    /// Caller reports a returned-to-customer label; it creates no staff queue, review, draft, gate, action, or checkout authority.
    ReturnedToCustomer,
    /// Caller reports a needs-follow-up label; it creates no staff queue, review, draft, gate, action, or checkout authority.
    NeedsStaffFollowUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Caller-reported departure-note review labels retained as compatibility evidence; they route no queue, draft, gate, or checkout authority.
pub enum DepartureNotesReview {
    /// Retains a caller-reported staff-reviewed label without authenticating staff, review, provider state, or checkout completion.
    StaffReviewed,
    /// Retains a caller-reported manager-review-required label without authenticating a manager, review request, gate, or queue authority.
    ManagerReviewRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Caller-reported checkout status labels retained as compatibility evidence; they route no queue, draft, gate, or completion authority.
pub enum CompletionStatus {
    /// Reports that serialized evidence labels checkout as staff-complete; this is never completion authority.
    ReportedStaffCheckout,
    /// Routes the item to needs staff handoff review for staff queueing, review, and downstream agent context.
    NeedsStaffHandoffReview,
    /// Routes the item to source not checked out for staff queueing, review, and downstream agent context.
    SourceNotCheckedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Current review-safe checkout tasks plus legacy compatibility labels.
///
/// Only variants returned by workflow evaluation are executable capabilities; the retention-draft
/// variant is retained for compatibility and is never emitted by the current workflow.
pub enum SafeAgentAction {
    /// Allows agents to summarize checkout evidence for staff review without mutating records or contacting customers.
    SummarizeCheckoutEvidence,
    /// Allows agents to create internal handoff task for staff review without mutating records or contacting customers.
    CreateInternalHandoffTask,
    /// Legacy compatibility label for retention drafting that the current checkout workflow cannot emit.
    DraftRetentionFollowUpForReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Actions the agent must never perform without a human/operator system of record.
pub enum BlockedAction {
    /// Blocks agents from suggest checked out status until staff or the system of record performs the action.
    SuggestCheckedOutStatus,
    /// Blocks agents from send customer message until staff or the system of record performs the action.
    SendCustomerMessage,
    /// Blocks agents from mutate provider or pms record until staff or the system of record performs the action.
    MutateProviderOrPmsRecord,
    /// Blocks agents from move refund discount or payment until staff or the system of record performs the action.
    MoveRefundDiscountOrPayment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Internal audit-draft labels describe pending evidence only; they prove no review, event, action, queue admission, or checkout completion.
pub enum AuditEventDraft {
    /// Selects source checkout observed for the checkout completion decision model so the app can choose a review, evidence, or draft path without taking live action.
    SourceCheckoutObserved,
    /// Records the staff-submitted handoff payload as received, even when the source status prevents
    /// treating it as checkout-completion evidence.
    StaffHandoffRecorded,
    /// Selects staff handoff review requested for the checkout completion decision model so the app can choose a review, evidence, or draft path without taking live action.
    StaffHandoffReviewRequested,
    /// Selects checkout completion suggested for the checkout completion decision model so the app can choose a review, evidence, or draft path without taking live action.
    CheckoutCompletionSuggested,
    /// Selects customer message approval requested for the checkout completion decision model so the app can choose a review, evidence, or draft path without taking live action.
    CustomerMessageApprovalRequested,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Caller-reported handoff compatibility data retained for review; its actor, time, status, summary, and disposition labels prove no staff identity, handoff, review, action, queue admission, or checkout completion.
pub struct StaffHandoff {
    reported_completed_by: entities::ActorRef,
    reported_completed_at: DateTime<Utc>,
    belongings_status: BelongingsStatus,
    care_summary: CareSummary,
    departure_notes_review: DepartureNotesReview,
}

impl fmt::Debug for StaffHandoff {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("StaffHandoff([REDACTED])")
    }
}

impl StaffHandoff {
    /// Returns the caller-reported completion-actor label; it authenticates no actor, completion, or review.
    pub const fn reported_completed_by(&self) -> &entities::ActorRef {
        &self.reported_completed_by
    }

    /// Returns the caller-reported completion-time label; it proves no completion, review, or action.
    pub const fn reported_completed_at(&self) -> DateTime<Utc> {
        self.reported_completed_at
    }

    /// Returns the belongings status evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn belongings_status(&self) -> BelongingsStatus {
        self.belongings_status
    }

    /// Returns the care summary evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn care_summary(&self) -> &CareSummary {
        &self.care_summary
    }

    /// Returns the departure notes review evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn departure_notes_review(&self) -> DepartureNotesReview {
        self.departure_notes_review
    }

    const fn is_resolved_for_checkout_completion(&self) -> bool {
        matches!(self.belongings_status, BelongingsStatus::ReturnedToCustomer)
            && matches!(
                self.departure_notes_review,
                DepartureNotesReview::StaffReviewed
            )
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Input rules for building the workflow packet from source-grounded records.
pub struct Request {
    reservation_id: entities::reservation::Id,
    source_provenance: source::Provenance,
    observed_source_status: source::reservation::Status,
    staff_handoff: StaffHandoff,
    payment_exception: Option<PaymentException>,
    source_exception: Option<SourceException>,
    #[builder(default = LaborMinutes::try_new(15).expect("default checkout audit minutes are non-zero"))]
    estimated_manual_audit_minutes: LaborMinutes,
    #[builder(default = LaborMinutes::try_new(5).expect("default checkout packet minutes are non-zero"))]
    estimated_packet_review_minutes: LaborMinutes,
}

impl fmt::Debug for Request {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Request([REDACTED])")
    }
}

impl Request {
    /// Returns the reservation id evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    /// Returns the source provenance evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn source_provenance(&self) -> &source::Provenance {
        &self.source_provenance
    }

    /// Returns the observed source status evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn observed_source_status(&self) -> source::reservation::Status {
        self.observed_source_status.clone()
    }

    /// Returns the staff handoff evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn staff_handoff(&self) -> &StaffHandoff {
        &self.staff_handoff
    }

    /// Returns retained payment exception evidence; agents may route it, not move money.
    pub const fn payment_exception(&self) -> Option<PaymentException> {
        self.payment_exception
    }

    /// Returns retained source exception evidence; agents may route it, not mutate providers.
    pub const fn source_exception(&self) -> Option<SourceException> {
        self.source_exception
    }
}

#[derive(Clone, PartialEq, Eq, Serialize)]
/// Reviewable packet handed to staff or agents with deterministic gates already applied.
///
/// Deserialization preserves identifiers and reported evidence but discards caller-supplied
/// completion, status-suggestion, action, audit, disposition, gate, and blocker authority.
pub struct Packet {
    reservation_id: entities::reservation::Id,
    provenance: source::Provenance,
    staff_handoff: StaffHandoff,
    completion_status: CompletionStatus,
    suggested_reservation_status: Option<entities::reservation::Status>,
    required_review_gates: Vec<policy::ReviewGate>,
    safe_agent_actions: Vec<SafeAgentAction>,
    blocked_actions: Vec<BlockedAction>,
    audit_event_drafts: Vec<AuditEventDraft>,
    unresolved_exceptions: Vec<UnresolvedException>,
    staff_task_drafts: Vec<StaffTaskDraft>,
    reported_disposition: ReportedDisposition,
    labor_impact: LaborImpact,
}

impl<'de> Deserialize<'de> for Packet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SerializedPacketEvidence {
            reservation_id: entities::reservation::Id,
            provenance: source::Provenance,
            staff_handoff: StaffHandoff,
            #[serde(rename = "completion_status")]
            _completion_status: CompletionStatus,
            #[serde(rename = "suggested_reservation_status")]
            _suggested_reservation_status: Option<entities::reservation::Status>,
            #[serde(rename = "required_review_gates")]
            _required_review_gates: Vec<policy::ReviewGate>,
            #[serde(rename = "safe_agent_actions")]
            _safe_agent_actions: Vec<SafeAgentAction>,
            #[serde(rename = "blocked_actions")]
            _blocked_actions: Vec<BlockedAction>,
            #[serde(rename = "audit_event_drafts")]
            _audit_event_drafts: Vec<AuditEventDraft>,
            unresolved_exceptions: Vec<UnresolvedException>,
            #[serde(rename = "staff_task_drafts")]
            _staff_task_drafts: Vec<StaffTaskDraft>,
            #[serde(rename = "reported_disposition")]
            _reported_disposition: ReportedDisposition,
            labor_impact: LaborImpact,
        }

        let serialized = SerializedPacketEvidence::deserialize(deserializer)?;
        let completion_status = CompletionStatus::NeedsStaffHandoffReview;
        let unresolved_exceptions = serialized.unresolved_exceptions;
        let staff_task_drafts = staff_task_drafts_for(&unresolved_exceptions);

        Ok(Self {
            reservation_id: serialized.reservation_id,
            provenance: serialized.provenance,
            staff_handoff: serialized.staff_handoff,
            completion_status,
            suggested_reservation_status: None,
            required_review_gates: required_review_gates_for(completion_status),
            safe_agent_actions: safe_agent_actions_for(completion_status),
            blocked_actions: blocked_actions_for(completion_status),
            audit_event_drafts: vec![
                AuditEventDraft::StaffHandoffRecorded,
                AuditEventDraft::StaffHandoffReviewRequested,
            ],
            unresolved_exceptions,
            staff_task_drafts,
            reported_disposition: ReportedDisposition::ManagerReviewRequired,
            labor_impact: serialized.labor_impact,
        })
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Packet([REDACTED])")
    }
}

impl Packet {
    /// Returns the reservation id evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    /// Returns the provenance evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn provenance(&self) -> &source::Provenance {
        &self.provenance
    }

    /// Returns the staff handoff evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn staff_handoff(&self) -> &StaffHandoff {
        &self.staff_handoff
    }

    /// Returns the completion status evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn completion_status(&self) -> CompletionStatus {
        self.completion_status
    }

    /// Returns the suggested reservation status evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn suggested_reservation_status(&self) -> Option<entities::reservation::Status> {
        self.suggested_reservation_status.clone()
    }

    /// Returns the required review gates evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }

    /// Returns the safe agent actions evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn safe_agent_actions(&self) -> &[SafeAgentAction] {
        &self.safe_agent_actions
    }

    /// Returns the blocked actions evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn blocked_actions(&self) -> &[BlockedAction] {
        &self.blocked_actions
    }

    /// Returns the audit event drafts evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn audit_event_drafts(&self) -> &[AuditEventDraft] {
        &self.audit_event_drafts
    }

    /// Returns unresolved checkout exceptions that need staff, manager, billing, or source-system review before final closeout.
    pub fn unresolved_exceptions(&self) -> &[UnresolvedException] {
        &self.unresolved_exceptions
    }

    /// Returns draft-only staff task recommendations; agents may prepare these but not complete live checkout work.
    pub fn staff_task_drafts(&self) -> &[StaffTaskDraft] {
        &self.staff_task_drafts
    }

    /// Returns the review disposition used to keep outcome/labor reporting tied to human or system-of-record review.
    pub const fn reported_disposition(&self) -> ReportedDisposition {
        self.reported_disposition
    }

    /// Returns estimated labor impact for open-stay audit packet review; this is not a realized savings claim.
    pub const fn labor_impact(&self) -> &LaborImpact {
        &self.labor_impact
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Workflow used by the checkout completion workflow; it keeps checkout tasks, payment exceptions, and handoff notes explicit for staff review.
pub struct Workflow;

impl Workflow {
    /// Builds the evaluate result for the checkout completion workflow from reviewed source facts while preserving human review gates and draft-only side effects.
    pub fn evaluate(request: Request) -> Packet {
        let completion_status = completion_status_for(&request);
        let suggested_reservation_status = None;
        let required_review_gates = required_review_gates_for(completion_status);
        let safe_agent_actions = safe_agent_actions_for(completion_status);
        let blocked_actions = blocked_actions_for(completion_status);
        let audit_event_drafts = audit_event_drafts_for(completion_status);
        let unresolved_exceptions = unresolved_exceptions_for(&request, completion_status);
        let staff_task_drafts = staff_task_drafts_for(&unresolved_exceptions);
        let reported_disposition = reported_disposition_for(completion_status);
        let labor_impact = LaborImpact::new(
            request.estimated_manual_audit_minutes,
            request.estimated_packet_review_minutes,
        );

        Packet {
            reservation_id: request.reservation_id,
            provenance: request.source_provenance,
            staff_handoff: request.staff_handoff,
            completion_status,
            suggested_reservation_status,
            required_review_gates,
            safe_agent_actions,
            blocked_actions,
            audit_event_drafts,
            unresolved_exceptions,
            staff_task_drafts,
            reported_disposition,
            labor_impact,
        }
    }
}

fn completion_status_for(request: &Request) -> CompletionStatus {
    if !matches!(
        request.observed_source_status,
        source::reservation::Status::CheckedOut
    ) {
        return CompletionStatus::SourceNotCheckedOut;
    }

    let _serialized_handoff_reports_resolution =
        request.staff_handoff.is_resolved_for_checkout_completion();
    CompletionStatus::NeedsStaffHandoffReview
}

fn required_review_gates_for(completion_status: CompletionStatus) -> Vec<policy::ReviewGate> {
    let _ = completion_status;
    vec![policy::ReviewGate::ManagerApproval]
}

fn safe_agent_actions_for(completion_status: CompletionStatus) -> Vec<SafeAgentAction> {
    let actions = vec![
        SafeAgentAction::SummarizeCheckoutEvidence,
        SafeAgentAction::CreateInternalHandoffTask,
    ];
    let _ = completion_status;
    actions
}

fn blocked_actions_for(_completion_status: CompletionStatus) -> Vec<BlockedAction> {
    let mut blocked_actions = vec![
        BlockedAction::SendCustomerMessage,
        BlockedAction::MutateProviderOrPmsRecord,
        BlockedAction::MoveRefundDiscountOrPayment,
    ];
    blocked_actions.push(BlockedAction::SuggestCheckedOutStatus);
    blocked_actions.sort_unstable();
    blocked_actions.dedup();
    blocked_actions
}

fn audit_event_drafts_for(completion_status: CompletionStatus) -> Vec<AuditEventDraft> {
    let mut drafts = vec![AuditEventDraft::StaffHandoffRecorded];
    match completion_status {
        CompletionStatus::ReportedStaffCheckout => {
            drafts.push(AuditEventDraft::SourceCheckoutObserved);
            drafts.push(AuditEventDraft::CheckoutCompletionSuggested);
            drafts.push(AuditEventDraft::CustomerMessageApprovalRequested);
        }
        CompletionStatus::NeedsStaffHandoffReview => {
            drafts.push(AuditEventDraft::SourceCheckoutObserved);
            drafts.push(AuditEventDraft::StaffHandoffReviewRequested);
        }
        CompletionStatus::SourceNotCheckedOut => {
            drafts.push(AuditEventDraft::StaffHandoffReviewRequested);
        }
    }
    drafts.sort_unstable();
    drafts.dedup();
    drafts
}

fn unresolved_exceptions_for(
    request: &Request,
    completion_status: CompletionStatus,
) -> Vec<UnresolvedException> {
    let mut exceptions = Vec::new();

    if matches!(
        request.staff_handoff.belongings_status(),
        BelongingsStatus::NeedsStaffFollowUp
    ) {
        exceptions.push(UnresolvedException::Belongings);
    }

    if matches!(
        request.staff_handoff.departure_notes_review(),
        DepartureNotesReview::ManagerReviewRequired
    ) {
        exceptions.push(UnresolvedException::Care);
    }

    if let Some(payment_exception) = request.payment_exception() {
        exceptions.push(UnresolvedException::Payment(payment_exception));
    }

    if let Some(source_exception) = request.source_exception() {
        exceptions.push(UnresolvedException::Source(source_exception));
    }

    if matches!(completion_status, CompletionStatus::SourceNotCheckedOut)
        && !exceptions
            .iter()
            .any(|exception| matches!(exception, UnresolvedException::Source(_)))
    {
        exceptions.push(UnresolvedException::Source(
            SourceException::ProviderRecordConflict,
        ));
    }

    exceptions
}

fn staff_task_drafts_for(exceptions: &[UnresolvedException]) -> Vec<StaffTaskDraft> {
    let mut drafts = Vec::new();
    for exception in exceptions {
        match exception {
            UnresolvedException::Belongings => {
                drafts.push(StaffTaskDraft::VerifyBelongingsReturn);
            }
            UnresolvedException::Care => {
                drafts.push(StaffTaskDraft::ReviewCareAndDepartureNotes);
            }
            UnresolvedException::Payment(_) => {
                drafts.push(StaffTaskDraft::ResolvePaymentException);
            }
            UnresolvedException::Source(_) => {
                drafts.push(StaffTaskDraft::ReconcileSourceStatus);
            }
        }
    }
    drafts.sort_unstable();
    drafts.dedup();
    drafts
}

const fn reported_disposition_for(completion_status: CompletionStatus) -> ReportedDisposition {
    match completion_status {
        CompletionStatus::ReportedStaffCheckout => ReportedDisposition::ManagerReviewRequired,
        CompletionStatus::NeedsStaffHandoffReview => ReportedDisposition::ManagerReviewRequired,
        CompletionStatus::SourceNotCheckedOut => ReportedDisposition::SourceReconciliationRequired,
    }
}
