//! Checkout-completion workflow rules for staff departure handoff and closeout review.
//!
//! ## Operator summary
//!
//! Staff use this workflow to decide whether a departure belongs in the verified-checkout queue,
//! the staff-handoff-review queue, or the source-status-reconciliation queue. It compares the
//! source reservation status with front-desk handoff evidence such as belongings return, care
//! summary, and departure-note review so operators do not manually audit every checkout record
//! across the PMS, care notes, and follow-up queues.
//!
//! The workflow can reduce labor by summarizing checkout evidence, creating an internal handoff
//! task, drafting retention follow-up for review, and producing audit-event drafts. It is not
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

pub use domain::boarding::handoff::DepartureTaskDraft as StaffTaskDraft;
pub use domain::payment::CheckoutException as PaymentException;
pub use domain::reservation::CheckoutCompletionDisposition as ReviewedDisposition;
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

    /// Returns estimated minutes saved only when packet review is lower than manual audit effort.
    pub const fn estimated_minutes_saved(&self) -> Option<u16> {
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
pub struct CareSummary(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for belongings status in the checkout completion workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum BelongingsStatus {
    /// Routes the item to returned to customer for staff queueing, review, and downstream agent context.
    ReturnedToCustomer,
    /// Routes the item to needs staff follow up for staff queueing, review, and downstream agent context.
    NeedsStaffFollowUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for departure notes review in the checkout completion workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum DepartureNotesReview {
    /// Selects staff reviewed for the checkout completion decision model so the app can choose a review, evidence, or draft path without taking live action.
    StaffReviewed,
    /// Selects manager review required for the checkout completion decision model so the app can choose a review, evidence, or draft path without taking live action.
    ManagerReviewRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for completion status in the checkout completion workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum CompletionStatus {
    /// Routes the item to staff verified checkout for staff queueing, review, and downstream agent context.
    StaffVerifiedCheckout,
    /// Routes the item to needs staff handoff review for staff queueing, review, and downstream agent context.
    NeedsStaffHandoffReview,
    /// Routes the item to source not checked out for staff queueing, review, and downstream agent context.
    SourceNotCheckedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Review-safe agent tasks allowed to save staff time without crossing mutation or send gates.
pub enum SafeAgentAction {
    /// Allows agents to summarize checkout evidence for staff review without mutating records or contacting customers.
    SummarizeCheckoutEvidence,
    /// Allows agents to create internal handoff task for staff review without mutating records or contacting customers.
    CreateInternalHandoffTask,
    /// Allows agents to draft retention follow up for review for staff review without mutating records or contacting customers.
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
/// Decision choices for audit event draft in the checkout completion workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Staff handoff used by the checkout completion workflow; it keeps checkout tasks, payment exceptions, and handoff notes explicit for staff review.
pub struct StaffHandoff {
    completed_by: entities::ActorRef,
    completed_at: DateTime<Utc>,
    belongings_status: BelongingsStatus,
    care_summary: CareSummary,
    departure_notes_review: DepartureNotesReview,
}

impl StaffHandoff {
    /// Returns the completed by evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn completed_by(&self) -> &entities::ActorRef {
        &self.completed_by
    }

    /// Returns the completed at evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn completed_at(&self) -> DateTime<Utc> {
        self.completed_at
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Reviewable packet handed to staff or agents with deterministic gates already applied.
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
    reviewed_disposition: ReviewedDisposition,
    labor_impact: LaborImpact,
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
    pub const fn reviewed_disposition(&self) -> ReviewedDisposition {
        self.reviewed_disposition
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
        let suggested_reservation_status = match completion_status {
            CompletionStatus::StaffVerifiedCheckout => {
                Some(entities::reservation::Status::CheckedOut)
            }
            CompletionStatus::NeedsStaffHandoffReview | CompletionStatus::SourceNotCheckedOut => {
                None
            }
        };
        let required_review_gates = required_review_gates_for(completion_status);
        let safe_agent_actions = safe_agent_actions_for(completion_status);
        let blocked_actions = blocked_actions_for(completion_status);
        let audit_event_drafts = audit_event_drafts_for(completion_status);
        let unresolved_exceptions = unresolved_exceptions_for(&request, completion_status);
        let staff_task_drafts = staff_task_drafts_for(&unresolved_exceptions);
        let reviewed_disposition = reviewed_disposition_for(completion_status);
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
            reviewed_disposition,
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

    if request.staff_handoff.is_resolved_for_checkout_completion() {
        CompletionStatus::StaffVerifiedCheckout
    } else {
        CompletionStatus::NeedsStaffHandoffReview
    }
}

fn required_review_gates_for(completion_status: CompletionStatus) -> Vec<policy::ReviewGate> {
    match completion_status {
        CompletionStatus::StaffVerifiedCheckout => {
            vec![policy::ReviewGate::CustomerMessageApproval]
        }
        CompletionStatus::NeedsStaffHandoffReview | CompletionStatus::SourceNotCheckedOut => {
            vec![policy::ReviewGate::ManagerApproval]
        }
    }
}

fn safe_agent_actions_for(completion_status: CompletionStatus) -> Vec<SafeAgentAction> {
    let mut actions = vec![
        SafeAgentAction::SummarizeCheckoutEvidence,
        SafeAgentAction::CreateInternalHandoffTask,
    ];
    if matches!(completion_status, CompletionStatus::StaffVerifiedCheckout) {
        actions.push(SafeAgentAction::DraftRetentionFollowUpForReview);
    }
    actions
}

fn blocked_actions_for(completion_status: CompletionStatus) -> Vec<BlockedAction> {
    let mut blocked_actions = vec![
        BlockedAction::SendCustomerMessage,
        BlockedAction::MutateProviderOrPmsRecord,
        BlockedAction::MoveRefundDiscountOrPayment,
    ];
    if !matches!(completion_status, CompletionStatus::StaffVerifiedCheckout) {
        blocked_actions.push(BlockedAction::SuggestCheckedOutStatus);
    }
    blocked_actions.sort_unstable();
    blocked_actions.dedup();
    blocked_actions
}

fn audit_event_drafts_for(completion_status: CompletionStatus) -> Vec<AuditEventDraft> {
    let mut drafts = vec![AuditEventDraft::StaffHandoffRecorded];
    match completion_status {
        CompletionStatus::StaffVerifiedCheckout => {
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

const fn reviewed_disposition_for(completion_status: CompletionStatus) -> ReviewedDisposition {
    match completion_status {
        CompletionStatus::StaffVerifiedCheckout => ReviewedDisposition::StaffVerified,
        CompletionStatus::NeedsStaffHandoffReview => ReviewedDisposition::ManagerReviewRequired,
        CompletionStatus::SourceNotCheckedOut => ReviewedDisposition::SourceReconciliationRequired,
    }
}
