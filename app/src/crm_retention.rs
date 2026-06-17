use chrono::{DateTime, Utc};
use domain::{entities, message, policy, source};
use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::checkout_completion;

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
pub struct EvidenceSummary(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SourceGroundedReasonCode {
    CompletedBoardingStay,
    CompletedDaycareVisit,
    CompletedGroomingVisit,
    CustomerAskedAboutFutureStay,
    PetEligibleForRecurringCare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OpportunityKind {
    NextBoardingStay,
    RecurringDaycare,
    GroomingRebook,
    TrainingConsult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ConsentStatus {
    Granted,
    Missing,
    OptedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EligibilityReason {
    SourceGroundedRetentionOpportunity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IneligibilityReason {
    CheckoutNotStaffVerified,
    NoSourceGroundedOpportunity,
    ContactPermissionNotSourceGrounded,
    ContactConsentMissing,
    ContactOptedOut,
    PreferredChannelNotAllowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FollowUpEligibility {
    Eligible { reason: EligibilityReason },
    Ineligible { reason: IneligibilityReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SafeAgentAction {
    SummarizeRetentionEvidence,
    CreateInternalStaffReviewTask,
    DraftCustomerFollowUpForReview,
    RecordFollowUpOutcomeEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BlockedAction {
    SendCustomerMessage,
    MutateProviderOrPmsRecord,
    MoveRefundDiscountOrPayment,
    AutoApplyDiscount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FollowUpOutcome {
    BookedNextStay,
    InterestedNeedsStaffCall,
    NotInterested,
    NoResponse,
    SuppressedByStaff,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct OpportunityEvidence {
    reason_code: SourceGroundedReasonCode,
    summary: EvidenceSummary,
    provenance: source::Provenance,
}

impl OpportunityEvidence {
    pub const fn reason_code(&self) -> SourceGroundedReasonCode {
        self.reason_code
    }

    pub const fn summary(&self) -> &EvidenceSummary {
        &self.summary
    }

    pub const fn provenance(&self) -> &source::Provenance {
        &self.provenance
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct RetentionOpportunity {
    kind: OpportunityKind,
    evidence: OpportunityEvidence,
}

impl RetentionOpportunity {
    pub const fn kind(&self) -> OpportunityKind {
        self.kind
    }

    pub const fn evidence(&self) -> &OpportunityEvidence {
        &self.evidence
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct ContactPermission {
    preferred_channel: message::Channel,
    #[builder(default)]
    allowed_channels: Vec<message::Channel>,
    marketing_consent: ConsentStatus,
    transactional_consent: ConsentStatus,
    #[builder(default)]
    source_record_refs: Vec<source::RecordRef>,
}

impl ContactPermission {
    pub const fn preferred_channel(&self) -> message::Channel {
        self.preferred_channel
    }

    pub fn allowed_channels(&self) -> &[message::Channel] {
        &self.allowed_channels
    }

    pub const fn marketing_consent(&self) -> ConsentStatus {
        self.marketing_consent
    }

    pub const fn transactional_consent(&self) -> ConsentStatus {
        self.transactional_consent
    }

    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }

    pub fn has_source_evidence(&self) -> bool {
        !self.source_record_refs.is_empty()
    }

    fn retention_draft_channel(&self) -> Option<message::Channel> {
        if !matches!(self.marketing_consent, ConsentStatus::Granted) {
            return None;
        }
        if matches!(self.preferred_channel, message::Channel::Internal) {
            return None;
        }
        self.allowed_channels
            .contains(&self.preferred_channel)
            .then_some(self.preferred_channel)
    }

    fn denial_reason(&self) -> IneligibilityReason {
        match self.marketing_consent {
            ConsentStatus::Granted => IneligibilityReason::PreferredChannelNotAllowed,
            ConsentStatus::Missing => IneligibilityReason::ContactConsentMissing,
            ConsentStatus::OptedOut => IneligibilityReason::ContactOptedOut,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct Request {
    reservation_id: entities::reservation::Id,
    customer_id: entities::CustomerId,
    checkout_packet: checkout_completion::Packet,
    contact_permission: ContactPermission,
    #[builder(default)]
    opportunities: Vec<RetentionOpportunity>,
}

impl Request {
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    pub const fn customer_id(&self) -> entities::CustomerId {
        self.customer_id
    }

    pub const fn checkout_packet(&self) -> &checkout_completion::Packet {
        &self.checkout_packet
    }

    pub const fn contact_permission(&self) -> &ContactPermission {
        &self.contact_permission
    }

    pub fn opportunities(&self) -> &[RetentionOpportunity] {
        &self.opportunities
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffReviewPacket {
    reservation_id: entities::reservation::Id,
    customer_id: entities::CustomerId,
    eligibility: FollowUpEligibility,
    draft_channel: Option<message::Channel>,
    staff_evidence: Vec<OpportunityEvidence>,
    required_review_gates: Vec<policy::ReviewGate>,
}

impl StaffReviewPacket {
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    pub const fn customer_id(&self) -> entities::CustomerId {
        self.customer_id
    }

    pub const fn eligibility(&self) -> FollowUpEligibility {
        self.eligibility
    }

    pub const fn draft_channel(&self) -> Option<message::Channel> {
        self.draft_channel
    }

    pub fn staff_evidence(&self) -> &[OpportunityEvidence] {
        &self.staff_evidence
    }

    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }

    pub fn requires_human_review(&self) -> bool {
        !self.required_review_gates.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Packet {
    reservation_id: entities::reservation::Id,
    customer_id: entities::CustomerId,
    eligibility: FollowUpEligibility,
    draft_channel: Option<message::Channel>,
    review_packet: StaffReviewPacket,
    required_review_gates: Vec<policy::ReviewGate>,
    safe_agent_actions: Vec<SafeAgentAction>,
    blocked_actions: Vec<BlockedAction>,
    source_record_refs: Vec<source::RecordRef>,
}

impl Packet {
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    pub const fn customer_id(&self) -> entities::CustomerId {
        self.customer_id
    }

    pub const fn eligibility(&self) -> FollowUpEligibility {
        self.eligibility
    }

    pub const fn draft_channel(&self) -> Option<message::Channel> {
        self.draft_channel
    }

    pub const fn review_packet(&self) -> &StaffReviewPacket {
        &self.review_packet
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

    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Workflow;

impl Workflow {
    pub fn evaluate(request: Request) -> Packet {
        let draft_channel = request.contact_permission.retention_draft_channel();
        let eligibility = eligibility_for(&request, draft_channel);
        let required_review_gates = required_review_gates_for(eligibility);
        let staff_evidence = request
            .opportunities
            .iter()
            .map(|opportunity| opportunity.evidence.clone())
            .collect::<Vec<_>>();
        let review_packet = StaffReviewPacket {
            reservation_id: request.reservation_id,
            customer_id: request.customer_id,
            eligibility,
            draft_channel,
            staff_evidence,
            required_review_gates: required_review_gates.clone(),
        };
        let safe_agent_actions = safe_agent_actions_for(eligibility);
        let blocked_actions = blocked_actions_for();
        let mut source_record_refs = vec![source::RecordRef::from_provenance(
            request.checkout_packet.provenance(),
        )];
        source_record_refs.extend(
            request
                .contact_permission
                .source_record_refs()
                .iter()
                .cloned(),
        );

        Packet {
            reservation_id: request.reservation_id,
            customer_id: request.customer_id,
            eligibility,
            draft_channel,
            review_packet,
            required_review_gates,
            safe_agent_actions,
            blocked_actions,
            source_record_refs,
        }
    }
}

fn eligibility_for(
    request: &Request,
    draft_channel: Option<message::Channel>,
) -> FollowUpEligibility {
    if !matches!(
        request.checkout_packet.completion_status(),
        checkout_completion::CompletionStatus::StaffVerifiedCheckout
    ) {
        return FollowUpEligibility::Ineligible {
            reason: IneligibilityReason::CheckoutNotStaffVerified,
        };
    }

    if request.opportunities.is_empty() {
        return FollowUpEligibility::Ineligible {
            reason: IneligibilityReason::NoSourceGroundedOpportunity,
        };
    }

    if draft_channel.is_none() {
        return FollowUpEligibility::Ineligible {
            reason: request.contact_permission.denial_reason(),
        };
    }

    if !request.contact_permission.has_source_evidence() {
        return FollowUpEligibility::Ineligible {
            reason: IneligibilityReason::ContactPermissionNotSourceGrounded,
        };
    }

    FollowUpEligibility::Eligible {
        reason: EligibilityReason::SourceGroundedRetentionOpportunity,
    }
}

fn required_review_gates_for(eligibility: FollowUpEligibility) -> Vec<policy::ReviewGate> {
    match eligibility {
        FollowUpEligibility::Eligible { .. } => vec![policy::ReviewGate::CustomerMessageApproval],
        FollowUpEligibility::Ineligible { .. } => vec![policy::ReviewGate::ManagerApproval],
    }
}

fn safe_agent_actions_for(eligibility: FollowUpEligibility) -> Vec<SafeAgentAction> {
    let mut actions = vec![
        SafeAgentAction::SummarizeRetentionEvidence,
        SafeAgentAction::CreateInternalStaffReviewTask,
        SafeAgentAction::RecordFollowUpOutcomeEvidence,
    ];
    if matches!(eligibility, FollowUpEligibility::Eligible { .. }) {
        actions.push(SafeAgentAction::DraftCustomerFollowUpForReview);
    }
    actions
}

fn blocked_actions_for() -> Vec<BlockedAction> {
    vec![
        BlockedAction::AutoApplyDiscount,
        BlockedAction::MoveRefundDiscountOrPayment,
        BlockedAction::MutateProviderOrPmsRecord,
        BlockedAction::SendCustomerMessage,
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct OutcomeRecord {
    reservation_id: entities::reservation::Id,
    customer_id: entities::CustomerId,
    recorded_by: entities::ActorRef,
    recorded_at: DateTime<Utc>,
    outcome: FollowUpOutcome,
    source_provenance: source::Provenance,
    #[builder(default)]
    evidence: Vec<OpportunityEvidence>,
}

impl OutcomeRecord {
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    pub const fn customer_id(&self) -> entities::CustomerId {
        self.customer_id
    }

    pub const fn recorded_by(&self) -> &entities::ActorRef {
        &self.recorded_by
    }

    pub const fn recorded_at(&self) -> DateTime<Utc> {
        self.recorded_at
    }

    pub const fn outcome(&self) -> FollowUpOutcome {
        self.outcome
    }

    pub const fn source_provenance(&self) -> &source::Provenance {
        &self.source_provenance
    }

    pub fn evidence(&self) -> &[OpportunityEvidence] {
        &self.evidence
    }

    pub const fn records_staff_evidence_only(&self) -> bool {
        true
    }

    pub fn blocked_actions(&self) -> Vec<BlockedAction> {
        blocked_actions_for()
    }
}
