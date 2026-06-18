use chrono::{DateTime, Utc};
use domain::{entities, policy, source};
use nutype::nutype;
use serde::{Deserialize, Serialize};

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
/// Classifies belongings status values that drive the checkout completion workflow.
pub enum BelongingsStatus {
    /// Labels work as returned to customer for queueing, review, and downstream agent context.
    ReturnedToCustomer,
    /// Labels work as needs staff follow up for queueing, review, and downstream agent context.
    NeedsStaffFollowUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Classifies departure notes review values that drive the checkout completion workflow.
pub enum DepartureNotesReview {
    /// Routes checkout completion work flagged as staff reviewed to the right queue, review gate, or agent packet.
    StaffReviewed,
    /// Routes checkout completion work flagged as manager review required to the right queue, review gate, or agent packet.
    ManagerReviewRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Classifies completion status values that drive the checkout completion workflow.
pub enum CompletionStatus {
    /// Labels work as staff verified checkout for queueing, review, and downstream agent context.
    StaffVerifiedCheckout,
    /// Labels work as needs staff handoff review for queueing, review, and downstream agent context.
    NeedsStaffHandoffReview,
    /// Labels work as source not checked out for queueing, review, and downstream agent context.
    SourceNotCheckedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Review-safe agent tasks allowed to save staff time without crossing mutation or send boundaries.
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
/// Classifies audit event draft values that drive the checkout completion workflow.
pub enum AuditEventDraft {
    /// Routes checkout completion work flagged as source checkout observed to the right queue, review gate, or agent packet.
    SourceCheckoutObserved,
    /// Records the staff-submitted handoff payload as received, even when the source status prevents
    /// treating it as checkout-completion evidence.
    StaffHandoffRecorded,
    /// Routes checkout completion work flagged as staff handoff review requested to the right queue, review gate, or agent packet.
    StaffHandoffReviewRequested,
    /// Routes checkout completion work flagged as checkout completion suggested to the right queue, review gate, or agent packet.
    CheckoutCompletionSuggested,
    /// Routes checkout completion work flagged as customer message approval requested to the right queue, review gate, or agent packet.
    CustomerMessageApprovalRequested,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Staff handoff carried by the checkout completion workflow; it keeps checkout tasks, payment exceptions, and handoff notes explicit for staff review.
pub struct StaffHandoff {
    completed_by: entities::ActorRef,
    completed_at: DateTime<Utc>,
    belongings_status: BelongingsStatus,
    care_summary: CareSummary,
    departure_notes_review: DepartureNotesReview,
}

impl StaffHandoff {
    /// Returns the completed by carried by this checkout completion workflow value.
    pub const fn completed_by(&self) -> &entities::ActorRef {
        &self.completed_by
    }

    /// Returns the completed at carried by this checkout completion workflow value.
    pub const fn completed_at(&self) -> DateTime<Utc> {
        self.completed_at
    }

    /// Returns the belongings status carried by this checkout completion workflow value.
    pub const fn belongings_status(&self) -> BelongingsStatus {
        self.belongings_status
    }

    /// Returns the care summary carried by this checkout completion workflow value.
    pub const fn care_summary(&self) -> &CareSummary {
        &self.care_summary
    }

    /// Returns the departure notes review carried by this checkout completion workflow value.
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
/// Input contract for building the workflow packet from source-grounded records.
pub struct Request {
    reservation_id: entities::reservation::Id,
    source_provenance: source::Provenance,
    observed_source_status: source::reservation::Status,
    staff_handoff: StaffHandoff,
}

impl Request {
    /// Returns the reservation id carried by this checkout completion workflow value.
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    /// Returns the source provenance carried by this checkout completion workflow value.
    pub const fn source_provenance(&self) -> &source::Provenance {
        &self.source_provenance
    }

    /// Returns the observed source status carried by this checkout completion workflow value.
    pub fn observed_source_status(&self) -> source::reservation::Status {
        self.observed_source_status.clone()
    }

    /// Returns the staff handoff carried by this checkout completion workflow value.
    pub const fn staff_handoff(&self) -> &StaffHandoff {
        &self.staff_handoff
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
}

impl Packet {
    /// Returns the reservation id carried by this checkout completion workflow value.
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    /// Returns the provenance carried by this checkout completion workflow value.
    pub const fn provenance(&self) -> &source::Provenance {
        &self.provenance
    }

    /// Returns the staff handoff carried by this checkout completion workflow value.
    pub const fn staff_handoff(&self) -> &StaffHandoff {
        &self.staff_handoff
    }

    /// Returns the completion status carried by this checkout completion workflow value.
    pub const fn completion_status(&self) -> CompletionStatus {
        self.completion_status
    }

    /// Returns the suggested reservation status carried by this checkout completion workflow value.
    pub fn suggested_reservation_status(&self) -> Option<entities::reservation::Status> {
        self.suggested_reservation_status.clone()
    }

    /// Returns the required review gates carried by this checkout completion workflow value.
    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }

    /// Returns the safe agent actions carried by this checkout completion workflow value.
    pub fn safe_agent_actions(&self) -> &[SafeAgentAction] {
        &self.safe_agent_actions
    }

    /// Returns the blocked actions carried by this checkout completion workflow value.
    pub fn blocked_actions(&self) -> &[BlockedAction] {
        &self.blocked_actions
    }

    /// Returns the audit event drafts carried by this checkout completion workflow value.
    pub fn audit_event_drafts(&self) -> &[AuditEventDraft] {
        &self.audit_event_drafts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Workflow carried by the checkout completion workflow; it keeps checkout tasks, payment exceptions, and handoff notes explicit for staff review.
pub struct Workflow;

impl Workflow {
    /// Builds or derives evaluate data for the checkout completion workflow contract.
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
