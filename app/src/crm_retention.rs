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
/// Decision taxonomy for source grounded reason code in the retention follow-up workflow; each value carries operational meaning for source-grounded routing and review.
pub enum SourceGroundedReasonCode {
    /// Uses completed boarding stay as source-grounded evidence for the deterministic decision.
    CompletedBoardingStay,
    /// Uses completed daycare visit as source-grounded evidence for the deterministic decision.
    CompletedDaycareVisit,
    /// Uses completed grooming visit as source-grounded evidence for the deterministic decision.
    CompletedGroomingVisit,
    /// Uses customer asked about future stay as source-grounded evidence for the deterministic decision.
    CustomerAskedAboutFutureStay,
    /// Uses pet eligible for recurring care as source-grounded evidence for the deterministic decision.
    PetEligibleForRecurringCare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision taxonomy for opportunity kind in the retention follow-up workflow; each value carries operational meaning for source-grounded routing and review.
pub enum OpportunityKind {
    /// Represents next boarding stay in the retention follow-up decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    NextBoardingStay,
    /// Represents recurring daycare in the retention follow-up decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    RecurringDaycare,
    /// Represents grooming rebook in the retention follow-up decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    GroomingRebook,
    /// Represents training consult in the retention follow-up decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    TrainingConsult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision taxonomy for consent status in the retention follow-up workflow; each value carries operational meaning for source-grounded routing and review.
pub enum ConsentStatus {
    /// Labels work as granted for queueing, review, and downstream agent context.
    Granted,
    /// Labels work as missing for queueing, review, and downstream agent context.
    Missing,
    /// Labels work as opted out for queueing, review, and downstream agent context.
    OptedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Decision taxonomy for eligibility reason in the retention follow-up workflow; each value carries operational meaning for source-grounded routing and review.
pub enum EligibilityReason {
    /// Explains that the workflow is source grounded retention opportunity when deciding whether an agent draft is allowed.
    SourceGroundedRetentionOpportunity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Decision taxonomy for ineligibility reason in the retention follow-up workflow; each value carries operational meaning for source-grounded routing and review.
pub enum IneligibilityReason {
    /// Explains that the workflow is checkout not staff verified when deciding whether an agent draft is allowed.
    CheckoutNotStaffVerified,
    /// Explains that the workflow is no source grounded opportunity when deciding whether an agent draft is allowed.
    NoSourceGroundedOpportunity,
    /// Explains that the workflow is contact permission not source grounded when deciding whether an agent draft is allowed.
    ContactPermissionNotSourceGrounded,
    /// Explains that the workflow is contact consent missing when deciding whether an agent draft is allowed.
    ContactConsentMissing,
    /// Explains that the workflow is contact opted out when deciding whether an agent draft is allowed.
    ContactOptedOut,
    /// Explains that the workflow is preferred channel not allowed when deciding whether an agent draft is allowed.
    PreferredChannelNotAllowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Outcome of the deterministic contact-safety check for retention follow-up.
pub enum FollowUpEligibility {
    /// Source-derived Reason retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    Eligible {
        /// Reason carried by this variant.
        reason: EligibilityReason,
    },
    /// Source-derived Reason retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    Ineligible {
        /// Reason carried by this variant.
        reason: IneligibilityReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Review-safe agent tasks allowed to save staff time without crossing mutation or send boundaries.
pub enum SafeAgentAction {
    /// Allows agents to summarize retention evidence for staff review without mutating records or contacting customers.
    SummarizeRetentionEvidence,
    /// Allows agents to create internal staff review task for staff review without mutating records or contacting customers.
    CreateInternalStaffReviewTask,
    /// Allows agents to draft customer follow up for review for staff review without mutating records or contacting customers.
    DraftCustomerFollowUpForReview,
    /// Allows agents to record follow up outcome evidence for staff review without mutating records or contacting customers.
    RecordFollowUpOutcomeEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Actions the agent must never perform without a human/operator system of record.
pub enum BlockedAction {
    /// Blocks agents from send customer message until staff or the system of record performs the action.
    SendCustomerMessage,
    /// Blocks agents from mutate provider or pms record until staff or the system of record performs the action.
    MutateProviderOrPmsRecord,
    /// Blocks agents from move refund discount or payment until staff or the system of record performs the action.
    MoveRefundDiscountOrPayment,
    /// Blocks agents from auto apply discount until staff or the system of record performs the action.
    AutoApplyDiscount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision taxonomy for follow up outcome in the retention follow-up workflow; each value carries operational meaning for source-grounded routing and review.
pub enum FollowUpOutcome {
    /// Records a booked next stay result so follow-up impact is auditable.
    BookedNextStay,
    /// Records a interested needs staff call result so follow-up impact is auditable.
    InterestedNeedsStaffCall,
    /// Records a not interested result so follow-up impact is auditable.
    NotInterested,
    /// Records a no response result so follow-up impact is auditable.
    NoResponse,
    /// Records a suppressed by staff result so follow-up impact is auditable.
    SuppressedByStaff,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Opportunity evidence carried by the retention follow-up workflow; it turns source-grounded visit evidence into safe follow-up drafts without sending customer messages automatically.
pub struct OpportunityEvidence {
    reason_code: SourceGroundedReasonCode,
    summary: EvidenceSummary,
    provenance: source::Provenance,
}

impl OpportunityEvidence {
    /// Returns the reason code source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn reason_code(&self) -> SourceGroundedReasonCode {
        self.reason_code
    }

    /// Returns the summary source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn summary(&self) -> &EvidenceSummary {
        &self.summary
    }

    /// Returns the provenance source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn provenance(&self) -> &source::Provenance {
        &self.provenance
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Retention opportunity carried by the retention follow-up workflow; it turns source-grounded visit evidence into safe follow-up drafts without sending customer messages automatically.
pub struct RetentionOpportunity {
    kind: OpportunityKind,
    evidence: OpportunityEvidence,
}

impl RetentionOpportunity {
    /// Returns the kind source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn kind(&self) -> OpportunityKind {
        self.kind
    }

    /// Returns the evidence source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn evidence(&self) -> &OpportunityEvidence {
        &self.evidence
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Contact permission carried by the retention follow-up workflow; it turns source-grounded visit evidence into safe follow-up drafts without sending customer messages automatically.
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
    /// Returns the preferred channel source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn preferred_channel(&self) -> message::Channel {
        self.preferred_channel
    }

    /// Returns the allowed channels source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub fn allowed_channels(&self) -> &[message::Channel] {
        &self.allowed_channels
    }

    /// Returns the marketing consent source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn marketing_consent(&self) -> ConsentStatus {
        self.marketing_consent
    }

    /// Returns the transactional consent source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn transactional_consent(&self) -> ConsentStatus {
        self.transactional_consent
    }

    /// Returns the source record refs source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }

    /// Reports whether the retention follow-up workflow satisfies the has source evidence safety condition.
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
/// Input contract for building the workflow packet from source-grounded records.
pub struct Request {
    reservation_id: entities::reservation::Id,
    customer_id: entities::CustomerId,
    checkout_packet: checkout_completion::Packet,
    contact_permission: ContactPermission,
    #[builder(default)]
    opportunities: Vec<RetentionOpportunity>,
}

impl Request {
    /// Returns the reservation id source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    /// Returns the customer id source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn customer_id(&self) -> entities::CustomerId {
        self.customer_id
    }

    /// Returns the checkout packet source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn checkout_packet(&self) -> &checkout_completion::Packet {
        &self.checkout_packet
    }

    /// Returns the contact permission source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn contact_permission(&self) -> &ContactPermission {
        &self.contact_permission
    }

    /// Returns the opportunities source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub fn opportunities(&self) -> &[RetentionOpportunity] {
        &self.opportunities
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Staff-facing packet that explains the evidence, draft limits, and required review gates.
pub struct StaffReviewPacket {
    reservation_id: entities::reservation::Id,
    customer_id: entities::CustomerId,
    eligibility: FollowUpEligibility,
    draft_channel: Option<message::Channel>,
    staff_evidence: Vec<OpportunityEvidence>,
    required_review_gates: Vec<policy::ReviewGate>,
}

impl StaffReviewPacket {
    /// Returns the reservation id source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    /// Returns the customer id source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn customer_id(&self) -> entities::CustomerId {
        self.customer_id
    }

    /// Returns the eligibility source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn eligibility(&self) -> FollowUpEligibility {
        self.eligibility
    }

    /// Returns the draft channel source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn draft_channel(&self) -> Option<message::Channel> {
        self.draft_channel
    }

    /// Returns the staff evidence source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub fn staff_evidence(&self) -> &[OpportunityEvidence] {
        &self.staff_evidence
    }

    /// Returns the required review gates source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }

    /// Reports whether the retention follow-up workflow satisfies the requires human review safety condition.
    pub fn requires_human_review(&self) -> bool {
        !self.required_review_gates.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Reviewable packet handed to staff or agents with deterministic gates already applied.
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
    /// Returns the reservation id source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    /// Returns the customer id source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn customer_id(&self) -> entities::CustomerId {
        self.customer_id
    }

    /// Returns the eligibility source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn eligibility(&self) -> FollowUpEligibility {
        self.eligibility
    }

    /// Returns the draft channel source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn draft_channel(&self) -> Option<message::Channel> {
        self.draft_channel
    }

    /// Returns the review packet source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn review_packet(&self) -> &StaffReviewPacket {
        &self.review_packet
    }

    /// Returns the required review gates source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }

    /// Returns the safe agent actions source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub fn safe_agent_actions(&self) -> &[SafeAgentAction] {
        &self.safe_agent_actions
    }

    /// Returns the blocked actions source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub fn blocked_actions(&self) -> &[BlockedAction] {
        &self.blocked_actions
    }

    /// Returns the source record refs source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Workflow carried by the retention follow-up workflow; it turns source-grounded visit evidence into safe follow-up drafts without sending customer messages automatically.
pub struct Workflow;

impl Workflow {
    /// Builds evaluate for the retention follow-up workflow contract from validated source facts while preserving review gates and draft-only side-effect boundaries.
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
/// Outcome record carried by the retention follow-up workflow; it turns source-grounded visit evidence into safe follow-up drafts without sending customer messages automatically.
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
    /// Returns the reservation id source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    /// Returns the customer id source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn customer_id(&self) -> entities::CustomerId {
        self.customer_id
    }

    /// Returns the recorded by source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn recorded_by(&self) -> &entities::ActorRef {
        &self.recorded_by
    }

    /// Returns the recorded at source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn recorded_at(&self) -> DateTime<Utc> {
        self.recorded_at
    }

    /// Returns the outcome source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn outcome(&self) -> FollowUpOutcome {
        self.outcome
    }

    /// Returns the source provenance source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn source_provenance(&self) -> &source::Provenance {
        &self.source_provenance
    }

    /// Returns the evidence source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub fn evidence(&self) -> &[OpportunityEvidence] {
        &self.evidence
    }

    /// Returns the records staff evidence only source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub const fn records_staff_evidence_only(&self) -> bool {
        true
    }

    /// Returns the blocked actions source evidence carried by this retention follow-up workflow artifact without changing provider, customer, payment, or schedule state.
    pub fn blocked_actions(&self) -> Vec<BlockedAction> {
        blocked_actions_for()
    }
}
