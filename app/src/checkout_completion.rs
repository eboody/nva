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
pub enum BelongingsStatus {
    ReturnedToCustomer,
    NeedsStaffFollowUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DepartureNotesReview {
    StaffReviewed,
    ManagerReviewRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CompletionStatus {
    StaffVerifiedCheckout,
    NeedsStaffHandoffReview,
    SourceNotCheckedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SafeAgentAction {
    SummarizeCheckoutEvidence,
    CreateInternalHandoffTask,
    DraftRetentionFollowUpForReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BlockedAction {
    SuggestCheckedOutStatus,
    SendCustomerMessage,
    MutateProviderOrPmsRecord,
    MoveRefundDiscountOrPayment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AuditEventDraft {
    SourceCheckoutObserved,
    /// Records the staff-submitted handoff payload as received, even when the source status prevents
    /// treating it as checkout-completion evidence.
    StaffHandoffRecorded,
    StaffHandoffReviewRequested,
    CheckoutCompletionSuggested,
    CustomerMessageApprovalRequested,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct StaffHandoff {
    completed_by: entities::ActorRef,
    completed_at: DateTime<Utc>,
    belongings_status: BelongingsStatus,
    care_summary: CareSummary,
    departure_notes_review: DepartureNotesReview,
}

impl StaffHandoff {
    pub const fn completed_by(&self) -> &entities::ActorRef {
        &self.completed_by
    }

    pub const fn completed_at(&self) -> DateTime<Utc> {
        self.completed_at
    }

    pub const fn belongings_status(&self) -> BelongingsStatus {
        self.belongings_status
    }

    pub const fn care_summary(&self) -> &CareSummary {
        &self.care_summary
    }

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
pub struct Request {
    reservation_id: entities::reservation::Id,
    source_provenance: source::Provenance,
    observed_source_status: source::reservation::Status,
    staff_handoff: StaffHandoff,
}

impl Request {
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    pub const fn source_provenance(&self) -> &source::Provenance {
        &self.source_provenance
    }

    pub fn observed_source_status(&self) -> source::reservation::Status {
        self.observed_source_status.clone()
    }

    pub const fn staff_handoff(&self) -> &StaffHandoff {
        &self.staff_handoff
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    pub const fn provenance(&self) -> &source::Provenance {
        &self.provenance
    }

    pub const fn staff_handoff(&self) -> &StaffHandoff {
        &self.staff_handoff
    }

    pub const fn completion_status(&self) -> CompletionStatus {
        self.completion_status
    }

    pub fn suggested_reservation_status(&self) -> Option<entities::reservation::Status> {
        self.suggested_reservation_status.clone()
    }

    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }

    pub fn safe_agent_actions(&self) -> &[SafeAgentAction] {
        &self.safe_agent_actions
    }

    pub fn blocked_actions(&self) -> &[BlockedAction] {
        &self.blocked_actions
    }

    pub fn audit_event_drafts(&self) -> &[AuditEventDraft] {
        &self.audit_event_drafts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Workflow;

impl Workflow {
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
