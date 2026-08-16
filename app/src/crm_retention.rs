use chrono::{DateTime, Utc};
use domain::{entities, grooming, message, policy, source};
use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::checkout_completion;

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 1200),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct EvidenceSummary(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Reported source-grounded reason code retained for staff inspection; it cannot establish eligibility, queue work, drafting, conversion, completion, or value.
pub enum SourceGroundedReasonCode {
    /// Retains a caller-reported boarding-completed label without proving a stay, checkout, source provenance, or completion.
    CompletedBoardingStay,
    /// Retains a caller-reported daycare-completed label without proving a visit, source provenance, or completion.
    CompletedDaycareVisit,
    /// Retains a caller-reported grooming-completed label without proving a visit, source provenance, or completion.
    CompletedGroomingVisit,
    /// Retains a caller-reported future-stay-interest label without proving customer intent, consent, contact authority, or source provenance.
    CustomerAskedAboutFutureStay,
    /// Retains a caller-reported recurring-care-eligibility label without proving eligibility, review, action authority, or source provenance.
    PetEligibleForRecurringCare,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Caller-reported business-reason evidence retained for inspection; it cannot establish an opportunity, draft, outreach, booking, conversion, completion, or value.
pub enum OpportunityReason {
    /// Reports a boarding-stay-completed label without proving checkout, eligibility, or review-packet authority.
    BoardingStayCompleted,
    /// Reports a daycare-visit-completed label without proving completion, eligibility, or follow-up authority.
    DaycareVisitCompleted,
    /// Reports grooming cadence evidence without establishing rebooking eligibility or action authority.
    GroomingCadenceDue {
        /// Cadence status from the grooming domain recommendation.
        status: grooming::rebooking::Status,
        /// Source-backed rationale explaining the cadence decision.
        rationale: grooming::rebooking::Rationale,
    },
    /// Reports a future-service-request label without proving current customer intent or contact authority.
    CustomerRequestedFutureService,
    /// Reports a recurring-care-eligibility label without proving staff review or eligibility authority.
    RecurringCareEligible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Reported opportunity kind retained for staff inspection; it cannot establish eligibility, queue work, drafting, conversion, completion, or value.
pub enum OpportunityKind {
    /// Reports next-boarding-stay context as evidence only.
    NextBoardingStay,
    /// Reports recurring-daycare context as evidence only.
    RecurringDaycare,
    /// Reports grooming-rebook context as evidence only.
    GroomingRebook,
    /// Reports training-consult context as evidence only.
    TrainingConsult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Reported consent history retained for staff inspection; it cannot establish current contact authority, eligibility, queue work, or drafting.
pub enum ConsentStatus {
    /// Reports a granted-consent label without establishing current consent or contact authority.
    Granted,
    /// Reports that consent evidence is missing.
    Missing,
    /// Reports an opt-out label as evidence; it cannot be overridden by this workflow.
    OptedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Modeled reason for future opaque eligibility; current serialized inputs cannot produce eligible authority.
pub enum EligibilityReason {
    /// Names the reason a future opaque issuer could use; current serialized inputs cannot issue it.
    SourceGroundedRetentionOpportunity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Evidence-only reason that current retention input remains ineligible and cannot create a queue, task, or draft.
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
    /// Explains that suppression, complaint, source-quality, or review flags prevent customer draft authority.
    SuppressionFlagRequiresReview,
    /// Explains that CRM notes, segment membership, or campaign evidence was rejected, expired, or operations-only and therefore cannot justify marketing copy.
    NoAcceptedMarketingEvidence,
    /// Historical consent claims exist, but no opaque accepted-consent authority was issued by an authenticated source adapter.
    AcceptedConsentAuthorityUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Outcome of the deterministic contact-safety check for retention follow-up.
pub enum FollowUpEligibility {
    /// Modeled future eligibility state; current serialized inputs never produce this variant and cannot authorize queue work or drafting.
    Eligible {
        /// Reason value stored on this variant.
        reason: EligibilityReason,
    },
    /// Reason copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    Ineligible {
        /// Reason value stored on this variant.
        reason: IneligibilityReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Review-safe agent tasks allowed to save staff time without crossing mutation or send gates.
pub enum SafeAgentAction {
    /// Allows agents to summarize retention evidence for staff review without mutating records or contacting customers.
    SummarizeRetentionEvidence,
    /// Allows agents to create internal staff review task for staff review without mutating records or contacting customers.
    /// This action is modeled for a future opaque eligibility issuer and is unavailable from current serialized inputs.
    CreateInternalStaffReviewTask,
    /// Allows agents to draft customer follow up for review for staff review without mutating records or contacting customers.
    /// This action is modeled for a future opaque eligibility issuer and is unavailable from current serialized inputs.
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
    /// Blocks agents from creating, changing, holding, or assigning a booking/groomer/calendar slot.
    CreateOrChangeBooking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Source or policy conditions retained as suppression evidence; staff review alone cannot promote current serialized inputs into draft authority.
pub enum SuppressionFlag {
    /// Customer is on a DNC, opt-out, or suppression list that the workflow must respect.
    DoNotContactOrSuppressionList,
    /// Complaint, service recovery, incident, or reputation context needs manager review before outreach.
    ComplaintOrServiceRecoveryReview,
    /// Care, handling, medical, or groomer-sensitive context needs specialist review before outreach.
    CareOrHandlingReview,
    /// Source facts appear stale, conflicting, wrong-subject, or otherwise unsafe for customer copy.
    SourceQualityReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Historical review-status claim observed on CRM notes or segment evidence.
///
/// This serializable value never constitutes accepted marketing authority.
pub enum EvidenceReviewStatus {
    /// Source history reports acceptance; authenticated promotion is still required.
    Accepted,
    /// Evidence was reviewed and rejected as unsuitable for personalization or recommendation.
    Rejected,
    /// Evidence is too old to justify a current marketing follow-up.
    Expired,
    /// Evidence may inform internal staff tasks but cannot drive customer marketing copy.
    OperationsOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Caller-serializable compatibility classification retained as non-authoritative history.
///
/// No variant proves review, a booking, recommendation impact, conversion, completion, or value;
/// current reporting normalizes these labels to no-action evidence.
pub enum ReportedOutcomeClassification {
    /// Reports a correlated-booking-observation label without proving a booking or attribution.
    CorrelatedBookingObservation,
    /// Reports a no-action label for deferred, suppressed, wrong-source, or otherwise ineligible evidence.
    NoActionOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Caller-reported conversion label retained as historical evidence; it does not prove conversion, attribution, completion, or value.
pub enum ConversionKind {
    /// Reports a grooming-rebook label without proving that a rebook occurred or was caused by follow-up.
    GroomingRebooked,
    /// Reports a resort-service-booking label without proving that a booking occurred or was caused by follow-up.
    ResortServiceBooked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Reason staff deferred the opportunity without marking it converted or wrong-source.
pub enum DeferralReason {
    /// Staff/customer follow-up remains pending or waiting for a human review gate.
    WaitingOnCustomerOrStaffReview,
    /// Follow-up should be retried in a later cadence window.
    FutureCadenceWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Caller-reported retention disposition history; no value grants queue, task, draft, contact, conversion, completion, or value authority.
pub enum FollowUpOutcome {
    /// Reports a booked-next-stay label without proving a booking or follow-up impact.
    BookedNextStay,
    /// Reports interest or a requested staff call without creating contact authority or a task.
    InterestedNeedsStaffCall,
    /// Reports a not-interested disposition as evidence only.
    NotInterested,
    /// Reports a no-response disposition as evidence only.
    NoResponse,
    /// Reports a staff-suppression disposition as evidence only.
    SuppressedByStaff,
    /// Retains a caller-reported conversion label without proving conversion or granting booking authority.
    Converted {
        /// Caller-reported conversion kind retained as non-authoritative evidence.
        conversion: ConversionKind,
    },
    /// Retains a caller-reported deferral label without proving review or authorizing action.
    Deferred {
        /// Caller-reported deferral reason retained as evidence only.
        reason: DeferralReason,
    },
    /// Retains a caller-reported suppression label without proving staff action.
    Suppressed {
        /// Caller-reported suppression reason retained as evidence only.
        reason: SuppressionFlag,
    },
    /// Retains a caller-reported wrong-source label that cannot drive outreach.
    WrongSource,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Reported retention evidence preserved for staff inspection; serialized values cannot establish eligibility, queue work, or a customer draft.
pub struct OpportunityEvidence {
    reason_code: SourceGroundedReasonCode,
    summary: EvidenceSummary,
    provenance: source::Provenance,
    #[builder(default = EvidenceReviewStatus::OperationsOnly)]
    review_status: EvidenceReviewStatus,
}

impl OpportunityEvidence {
    /// Returns the reason code evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn reason_code(&self) -> SourceGroundedReasonCode {
        self.reason_code
    }

    /// Returns the summary evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn summary(&self) -> &EvidenceSummary {
        &self.summary
    }

    /// Returns the provenance evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn provenance(&self) -> &source::Provenance {
        &self.provenance
    }

    /// Returns caller-reported review-status evidence; it cannot authorize marketing personalization or internal work.
    pub const fn review_status(&self) -> EvidenceReviewStatus {
        self.review_status
    }

    /// Reports whether this serializable history independently authorizes marketing use.
    ///
    /// Historical review labels cannot mint accepted evidence, so this is deliberately
    /// false until an opaque, authenticated promotion authority is integrated.
    pub const fn can_drive_marketing_draft(&self) -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Caller-reported opportunity evidence that remains ineligible until a future opaque authenticated authority promotes it.
pub struct RetentionOpportunity {
    kind: OpportunityKind,
    #[builder(default = OpportunityReason::BoardingStayCompleted)]
    reason: OpportunityReason,
    evidence: OpportunityEvidence,
}

impl RetentionOpportunity {
    /// Returns the kind evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn kind(&self) -> OpportunityKind {
        self.kind
    }

    /// Returns the source-backed business reason staff review before any follow-up, booking, or outreach authority exists.
    pub const fn reason(&self) -> OpportunityReason {
        self.reason
    }

    /// Returns the evidence evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn evidence(&self) -> &OpportunityEvidence {
        &self.evidence
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Historical contact-permission evidence that cannot establish current contact eligibility or authorize queue work or drafting.
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
    /// Returns the preferred channel evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn preferred_channel(&self) -> message::Channel {
        self.preferred_channel
    }

    /// Returns the allowed channels evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn allowed_channels(&self) -> &[message::Channel] {
        &self.allowed_channels
    }

    /// Returns the marketing consent evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn marketing_consent(&self) -> ConsentStatus {
        self.marketing_consent
    }

    /// Returns the transactional consent evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn transactional_consent(&self) -> ConsentStatus {
        self.transactional_consent
    }

    /// Returns the source record refs evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }

    /// Reports whether the retention follow-up workflow satisfies the has source evidence safety condition.
    pub fn has_source_evidence(&self) -> bool {
        !self.source_record_refs.is_empty()
    }

    fn retention_draft_channel(&self) -> Option<message::Channel> {
        // `ConsentStatus::Granted` and source refs are historical observations. They
        // cannot substitute for domain::consent::AcceptedConsent, whose production
        // issuer is intentionally unavailable until an authenticated source adapter exists.
        None
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Input rules for building the workflow packet from source-grounded records.
pub struct Request {
    reservation_id: entities::reservation::Id,
    customer_id: entities::CustomerId,
    checkout_packet: checkout_completion::Packet,
    contact_permission: ContactPermission,
    #[builder(default)]
    opportunities: Vec<RetentionOpportunity>,
    #[builder(default)]
    suppression_flags: Vec<SuppressionFlag>,
}

impl Request {
    /// Returns the reservation id evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    /// Returns the customer id evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn customer_id(&self) -> entities::CustomerId {
        self.customer_id
    }

    /// Returns the checkout packet evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn checkout_packet(&self) -> &checkout_completion::Packet {
        &self.checkout_packet
    }

    /// Returns the contact permission evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn contact_permission(&self) -> &ContactPermission {
        &self.contact_permission
    }

    /// Returns the opportunities evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn opportunities(&self) -> &[RetentionOpportunity] {
        &self.opportunities
    }

    /// Returns source/policy suppression evidence without creating staff work or a customer draft.
    pub fn suppression_flags(&self) -> &[SuppressionFlag] {
        &self.suppression_flags
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Historical follow-up metadata retained in suppressed form; it does not represent a current draft or authorize work or delivery.
pub struct DraftFollowUp {
    channel: message::Channel,
    review_state: message::ReviewState,
    suppression_flags: Vec<SuppressionFlag>,
}

impl DraftFollowUp {
    /// Returns the customer channel selected from source-grounded contact permission for staff review.
    pub const fn channel(&self) -> message::Channel {
        self.channel
    }

    /// Returns the reported historical review state; deserialization normalizes it to suppressed evidence.
    pub const fn review_state(&self) -> message::ReviewState {
        self.review_state
    }

    /// Returns suppression flags carried in the evidence packet.
    pub fn suppression_flags(&self) -> &[SuppressionFlag] {
        &self.suppression_flags
    }
}

#[derive(Clone, PartialEq, Eq, Serialize)]
/// Staff-facing evidence packet; current serialized values cannot create eligibility, queue work, a task, or a draft.
pub struct StaffReviewPacket {
    reservation_id: entities::reservation::Id,
    customer_id: entities::CustomerId,
    eligibility: FollowUpEligibility,
    draft_channel: Option<message::Channel>,
    opportunities: Vec<RetentionOpportunity>,
    staff_evidence: Vec<OpportunityEvidence>,
    draft_follow_up: DraftFollowUp,
    required_review_gates: Vec<policy::ReviewGate>,
}

#[derive(Deserialize)]
struct SerializedStaffReviewPacket {
    reservation_id: entities::reservation::Id,
    customer_id: entities::CustomerId,
    eligibility: FollowUpEligibility,
    draft_channel: Option<message::Channel>,
    opportunities: Vec<RetentionOpportunity>,
    staff_evidence: Vec<OpportunityEvidence>,
    draft_follow_up: DraftFollowUp,
    required_review_gates: Vec<policy::ReviewGate>,
}

impl<'de> Deserialize<'de> for StaffReviewPacket {
    fn deserialize<Deserializer>(deserializer: Deserializer) -> Result<Self, Deserializer::Error>
    where
        Deserializer: serde::Deserializer<'de>,
    {
        let serialized = SerializedStaffReviewPacket::deserialize(deserializer)?;
        let _reported_authority = (
            serialized.eligibility,
            serialized.draft_channel,
            serialized.required_review_gates,
        );
        let eligibility = FollowUpEligibility::Ineligible {
            reason: IneligibilityReason::AcceptedConsentAuthorityUnavailable,
        };

        Ok(Self {
            reservation_id: serialized.reservation_id,
            customer_id: serialized.customer_id,
            eligibility,
            draft_channel: None,
            opportunities: serialized.opportunities,
            staff_evidence: serialized.staff_evidence,
            draft_follow_up: DraftFollowUp {
                channel: serialized.draft_follow_up.channel,
                review_state: message::ReviewState::Suppressed,
                suppression_flags: serialized.draft_follow_up.suppression_flags,
            },
            required_review_gates: required_review_gates_for(eligibility),
        })
    }
}

macro_rules! impl_sensitive_retention_debug {
    ($($type:ident),+ $(,)?) => {
        $(
            impl std::fmt::Debug for $type {
                fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str(concat!(stringify!($type), "([REDACTED])"))
                }
            }
        )+
    };
}

impl_sensitive_retention_debug!(
    EvidenceSummary,
    OpportunityReason,
    OpportunityEvidence,
    RetentionOpportunity,
    ContactPermission,
    Request,
    StaffReviewPacket,
);

impl StaffReviewPacket {
    /// Returns the reservation id evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    /// Returns the customer id evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn customer_id(&self) -> entities::CustomerId {
        self.customer_id
    }

    /// Returns the eligibility evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn eligibility(&self) -> FollowUpEligibility {
        self.eligibility
    }

    /// Returns the draft channel evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn draft_channel(&self) -> Option<message::Channel> {
        self.draft_channel
    }

    /// Returns the staff evidence evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn staff_evidence(&self) -> &[OpportunityEvidence] {
        &self.staff_evidence
    }

    /// Returns caller-reported opportunity records for inspection without creating draft or disposition authority.
    pub fn opportunities(&self) -> &[RetentionOpportunity] {
        &self.opportunities
    }

    /// Returns an empty set because serialized opportunity evidence cannot personalize a customer follow-up draft.
    pub fn marketable_opportunities(&self) -> Vec<&RetentionOpportunity> {
        self.opportunities
            .iter()
            .filter(|opportunity| opportunity.evidence().can_drive_marketing_draft())
            .collect()
    }

    /// Returns suppressed historical draft metadata; it does not authorize queue work, drafting, or contact.
    pub const fn draft_follow_up(&self) -> &DraftFollowUp {
        &self.draft_follow_up
    }

    /// Returns the required review gates evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }

    /// Reports whether the retention follow-up workflow satisfies the requires human review safety condition.
    pub fn requires_human_review(&self) -> bool {
        !self.required_review_gates.is_empty()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize)]
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

#[derive(Deserialize)]
struct SerializedPacket {
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

impl<'de> Deserialize<'de> for Packet {
    fn deserialize<Deserializer>(deserializer: Deserializer) -> Result<Self, Deserializer::Error>
    where
        Deserializer: serde::Deserializer<'de>,
    {
        let serialized = SerializedPacket::deserialize(deserializer)?;
        let _reported_authority = (
            serialized.eligibility,
            serialized.draft_channel,
            serialized.required_review_gates,
            serialized.safe_agent_actions,
            serialized.blocked_actions,
        );
        let eligibility = FollowUpEligibility::Ineligible {
            reason: IneligibilityReason::AcceptedConsentAuthorityUnavailable,
        };

        Ok(Self {
            reservation_id: serialized.reservation_id,
            customer_id: serialized.customer_id,
            eligibility,
            draft_channel: None,
            review_packet: serialized.review_packet,
            required_review_gates: required_review_gates_for(eligibility),
            safe_agent_actions: safe_agent_actions_for(eligibility),
            blocked_actions: blocked_actions_for(),
            source_record_refs: serialized.source_record_refs,
        })
    }
}

impl std::fmt::Debug for Packet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Packet([REDACTED])")
    }
}

impl Packet {
    /// Returns the reservation id evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    /// Returns the customer id evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn customer_id(&self) -> entities::CustomerId {
        self.customer_id
    }

    /// Returns the eligibility evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn eligibility(&self) -> FollowUpEligibility {
        self.eligibility
    }

    /// Returns the draft channel evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn draft_channel(&self) -> Option<message::Channel> {
        self.draft_channel
    }

    /// Returns the review packet evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn review_packet(&self) -> &StaffReviewPacket {
        &self.review_packet
    }

    /// Returns the required review gates evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }

    /// Returns the safe agent actions evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn safe_agent_actions(&self) -> &[SafeAgentAction] {
        &self.safe_agent_actions
    }

    /// Returns the blocked actions evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn blocked_actions(&self) -> &[BlockedAction] {
        &self.blocked_actions
    }

    /// Returns the source record refs evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Evidence-only retention workflow. Current serialized checkout, contact, consent, and
/// opportunity records cannot establish eligibility, create queue work, or produce a
/// customer follow-up draft.
pub struct Workflow;

impl Workflow {
    /// Builds a suppressed evidence packet. Draft and internal-task actions remain absent
    /// unless a future non-serializable authority boundary establishes eligibility.
    pub fn evaluate(request: Request) -> Packet {
        let draft_channel = request.contact_permission.retention_draft_channel();
        let eligibility = eligibility_for(&request, draft_channel);
        let required_review_gates = required_review_gates_for(eligibility);
        let staff_evidence = request
            .opportunities
            .iter()
            .map(|opportunity| opportunity.evidence.clone())
            .collect::<Vec<_>>();
        let draft_follow_up = DraftFollowUp {
            channel: draft_channel.unwrap_or(request.contact_permission.preferred_channel()),
            review_state: if matches!(eligibility, FollowUpEligibility::Eligible { .. }) {
                message::ReviewState::ApprovalRequested
            } else {
                message::ReviewState::Suppressed
            },
            suppression_flags: request.suppression_flags.clone(),
        };
        let review_packet = StaffReviewPacket {
            reservation_id: request.reservation_id,
            customer_id: request.customer_id,
            eligibility,
            draft_channel,
            opportunities: request.opportunities.clone(),
            staff_evidence,
            draft_follow_up,
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
        source_record_refs.extend(request.opportunities.iter().map(|opportunity| {
            source::RecordRef::from_provenance(opportunity.evidence().provenance())
        }));

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
    _request: &Request,
    _draft_channel: Option<message::Channel>,
) -> FollowUpEligibility {
    FollowUpEligibility::Ineligible {
        reason: IneligibilityReason::CheckoutNotStaffVerified,
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
        SafeAgentAction::RecordFollowUpOutcomeEvidence,
    ];
    if matches!(eligibility, FollowUpEligibility::Eligible { .. }) {
        actions.push(SafeAgentAction::CreateInternalStaffReviewTask);
        actions.push(SafeAgentAction::DraftCustomerFollowUpForReview);
    }
    actions
}

fn blocked_actions_for() -> Vec<BlockedAction> {
    vec![
        BlockedAction::AutoApplyDiscount,
        BlockedAction::CreateOrChangeBooking,
        BlockedAction::MoveRefundDiscountOrPayment,
        BlockedAction::MutateProviderOrPmsRecord,
        BlockedAction::SendCustomerMessage,
    ]
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Reported retention disposition history for staff inspection; it cannot establish eligibility, queue work, drafting, conversion, completion, or value.
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

impl std::fmt::Debug for OutcomeRecord {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("OutcomeRecord([REDACTED])")
    }
}

impl OutcomeRecord {
    /// Returns the reservation id evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    /// Returns the customer id evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn customer_id(&self) -> entities::CustomerId {
        self.customer_id
    }

    /// Returns the recorded by evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn recorded_by(&self) -> &entities::ActorRef {
        &self.recorded_by
    }

    /// Returns the recorded at evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn recorded_at(&self) -> DateTime<Utc> {
        self.recorded_at
    }

    /// Returns the outcome evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn outcome(&self) -> FollowUpOutcome {
        self.outcome
    }

    /// Returns the source provenance evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn source_provenance(&self) -> &source::Provenance {
        &self.source_provenance
    }

    /// Returns the evidence evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn evidence(&self) -> &[OpportunityEvidence] {
        &self.evidence
    }

    /// Returns the records staff evidence only evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn records_staff_evidence_only(&self) -> bool {
        true
    }

    /// Returns the blocked actions evidence available to retention follow-up review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn blocked_actions(&self) -> Vec<BlockedAction> {
        blocked_actions_for()
    }

    /// Reports whether caller-reported outcome labels correlate with the same reservation, customer, and source-reference labels as the evidence packet. This shape check authenticates no accepted source evidence, review, booking, action, contact, completion, or value.
    pub fn matches_reported_packet_evidence(&self, packet: &Packet) -> bool {
        self.reservation_id == packet.reservation_id
            && self.customer_id == packet.customer_id
            && self.evidence.iter().any(|evidence| {
                evidence.can_drive_marketing_draft()
                    && packet
                        .source_record_refs()
                        .contains(&source::RecordRef::from_provenance(evidence.provenance()))
            })
    }

    /// Normalizes caller-reported retention history to no-action compatibility evidence.
    ///
    /// The current workflow has no accepted booking-correlation authority, so the positive
    /// compatibility variant is unreachable and cannot contribute to recovered-booking counts.
    pub fn reported_outcome_classification_for(
        &self,
        packet: &Packet,
    ) -> ReportedOutcomeClassification {
        if self.matches_reported_packet_evidence(packet)
            && matches!(
                self.outcome,
                FollowUpOutcome::BookedNextStay
                    | FollowUpOutcome::Converted {
                        conversion: ConversionKind::ResortServiceBooked
                            | ConversionKind::GroomingRebooked
                    }
            )
        {
            ReportedOutcomeClassification::CorrelatedBookingObservation
        } else {
            ReportedOutcomeClassification::NoActionOutcome
        }
    }
}
