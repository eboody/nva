use super::*;

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        AsRef,
        Serialize,
        Deserialize
    )
)]
/// Issue ref used by the data-quality hygiene workflow; it finds duplicate, stale, or inconsistent records while blocking automatic provider-system mutation.
pub struct IssueRef(String);

impl IssueRef {
    /// Returns the source issue ref available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 130),
    derive(
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        AsRef,
        Serialize,
        Deserialize
    )
)]
/// Action id used by the data-quality hygiene workflow; it allows the `dq-action-` prefix plus a maximum-length issue ref while blocking automatic provider-system mutation.
pub struct ActionId(String);

impl ActionId {
    /// Returns the action id available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        AsRef,
        Serialize,
        Deserialize
    )
)]
/// Context packet id used by the data-quality hygiene workflow; it finds duplicate, stale, or inconsistent records while blocking automatic provider-system mutation.
pub struct ContextPacketId(String);

impl ContextPacketId {
    /// Returns the context packet id available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        AsRef,
        Serialize,
        Deserialize
    )
)]
/// Correlation id used by the data-quality hygiene workflow; it finds duplicate, stale, or inconsistent records while blocking automatic provider-system mutation.
pub struct CorrelationId(String);

impl CorrelationId {
    /// Returns the correlation id available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 500),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
/// Action rationale used by the data-quality hygiene workflow; it finds duplicate, stale, or inconsistent records while blocking automatic provider-system mutation.
pub struct ActionRationale(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Labor minutes used by the data-quality hygiene workflow; it finds duplicate, stale, or inconsistent records while blocking automatic provider-system mutation.
pub struct LaborMinutes(u16);

impl LaborMinutes {
    /// Validates a non-zero value for the data-quality hygiene workflow before it can appear in a manager packet or outcome record.
    pub const fn try_new(value: u16) -> Result<Self> {
        if value == 0 {
            return Err(Error::ZeroLaborMinutes);
        }
        Ok(Self(value))
    }

    /// Returns the numeric value available to data-quality hygiene review without touching provider, customer, payment, or schedule systems.
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Aggregate labor minutes used by the data-quality hygiene workflow; it finds duplicate, stale, or inconsistent records while blocking automatic provider-system mutation.
pub struct AggregateLaborMinutes(u16);

impl AggregateLaborMinutes {
    /// Stores a caller-reported value for the data-quality hygiene workflow without proving review or triggering provider, customer, payment, or schedule side effects.
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// Returns the numeric value available to data-quality hygiene review without touching provider, customer, payment, or schedule systems.
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for hygiene persona in the data-quality hygiene workflow; each value classifies evidence without proving review or independently authorizing a queue, draft, or staff gate.
pub enum HygienePersona {
    /// Selects general manager for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    GeneralManager,
    /// Selects assistant general manager for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    AssistantGeneralManager,
    /// Selects front desk lead for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    FrontDeskLead,
    /// Selects front desk agent for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    FrontDeskAgent,
    /// Selects regional operator for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    RegionalOperator,
    /// Selects operations analyst for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    OperationsAnalyst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for candidate kind in the data-quality hygiene workflow; each value classifies evidence without proving review or independently authorizing a queue, draft, or staff gate.
pub enum CandidateKind {
    /// Selects source issue for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    SourceIssue,
    /// Selects duplicate candidate for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    DuplicateCandidate,
    /// Selects profile gap for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    ProfileGap,
    /// Selects service line mapping for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    ServiceLineMapping,
    /// Selects source freshness for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    SourceFreshness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for source freshness in the data-quality hygiene workflow; each value classifies evidence without proving review or independently authorizing a queue, draft, or staff gate.
pub enum SourceFreshness {
    /// Selects current for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    Current,
    /// Selects stale for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    Stale,
    /// Selects conflicting for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    Conflicting,
    /// Selects missing for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for sensitivity in the data-quality hygiene workflow; each value classifies evidence without proving review or independently authorizing a queue, draft, or staff gate.
pub enum Sensitivity {
    /// Selects standard operational evidence for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    StandardOperationalEvidence,
    /// Selects vaccine evidence for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    VaccineEvidence,
    /// Selects incident or behavior evidence for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    IncidentOrBehaviorEvidence,
    /// Selects payment evidence for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    PaymentEvidence,
    /// Selects quarantined sensitive payload for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    QuarantinedSensitivePayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Entity or source fact family affected by a data-quality hygiene candidate.
pub enum AffectedEntity {
    /// Customer profile or owner identity evidence needs cleanup review.
    CustomerProfile,
    /// Pet profile, care, behavior, or vaccine-adjacent evidence needs cleanup review.
    PetProfile,
    /// Reservation or booking evidence needs cleanup review.
    Reservation,
    /// Service-line or reservation-type mapping evidence needs cleanup review.
    ServiceLine,
    /// Vaccination source evidence needs staff or document review before downstream workflows trust it.
    VaccinationRecord,
    /// Payment, deposit, discount, refund, invoice, or POS evidence needs protected review.
    PaymentRecord,
    /// Checkout or unclosed-stay evidence needs staff review.
    CheckoutRecord,
    /// Provider/source metadata itself is missing, stale, conflicting, or unverifiable.
    SourceEvidence,
    /// Quarantined payload evidence is present but raw contents must remain hidden from agent workflows.
    SensitivePayload,
    /// Location, regional, or policy-scope evidence needs cleanup review.
    LocationPolicyScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Business category for a data-quality issue so cleanup queues can group similar ambiguity.
pub enum IssueCategory {
    /// Required source evidence is missing, unknown, or internally inconsistent.
    MissingEvidence,
    /// Duplicate source or profile evidence may cause repeated staff review or inflated demand.
    DuplicateRecord,
    /// Customer/pet profile completeness needs internal cleanup preparation.
    ProfileCompleteness,
    /// Source freshness, staleness, or trust needs review before downstream use.
    SourceFreshness,
    /// Service-line naming or source mapping needs normalization review.
    ServiceLineMapping,
    /// Checkout or unclosed-stay evidence needs review before labor/reporting use.
    CheckoutEvidence,
    /// Sensitive or quarantined payload evidence needs protected escalation.
    SensitivePayload,
    /// Payment, deposit, refund, or checkout payment state conflicts need manager-safe review.
    PaymentConflict,
    /// Owner-pet relationship ambiguity needs human review before communication or merge work.
    AmbiguousRelationship,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Redaction boundary for agent-visible cleanup work.
pub enum RedactionPolicy {
    /// Agent workflows may reference the normalized source fact and source ref.
    SourceFactVisible,
    /// Agent workflows may summarize metadata and source refs, but not raw payload contents.
    SummarizeMetadataOnly,
    /// Raw payload is quarantined; only protected escalation metadata may flow to drafts.
    QuarantineRawPayload,
}

impl RedactionPolicy {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Review role named on a cleanup action before any provider/system-of-record mutation can happen.
pub enum ReviewerRole {
    /// General manager owns the cleanup review gate.
    GeneralManager,
    /// Assistant general manager owns the cleanup review gate.
    AssistantGeneralManager,
    /// Front desk lead owns the cleanup review gate.
    FrontDeskLead,
    /// Front desk agent can prepare the cleanup work but not close protected review gates.
    FrontDeskAgent,
    /// Regional operator owns portfolio or regional-policy cleanup review.
    RegionalOperator,
    /// Operations analyst owns reporting-quality review.
    OperationsAnalyst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Internal cleanup action family prepared by the workflow; these are draft/review tasks, not provider writes.
pub enum CleanupAction {
    /// Prepare missing or conflicting evidence investigation.
    InvestigateEvidence,
    /// Prepare duplicate customer/pet/source review without merging records.
    PrepareDuplicateReview,
    /// Prepare profile completion work for staff review.
    PrepareProfileCompletion,
    /// Prepare stale or missing vaccine source review.
    PrepareVaccineSourceReview,
    /// Prepare service-line mapping review.
    PrepareServiceLineMapping,
    /// Prepare checkout or unclosed-reservation evidence review.
    PrepareCheckoutEvidenceReview,
    /// Escalate sensitive payload handling without exposing raw payload contents.
    EscalateSensitivePayload,
    /// Prepare payment conflict review without moving money or changing invoices.
    PreparePaymentConflictReview,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Candidate used by the data-quality hygiene workflow; it finds duplicate, stale, or inconsistent records while blocking automatic provider-system mutation.
pub struct Candidate {
    id: IssueRef,
    kind: CandidateKind,
    issue: data_quality::Issue,
    #[builder(default)]
    source_record_refs: Vec<source::RecordRef>,
    source_freshness: SourceFreshness,
    sensitivity: Sensitivity,
}

impl Candidate {
    /// Returns the id evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn id(&self) -> &IssueRef {
        &self.id
    }

    /// Returns the kind evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn kind(&self) -> CandidateKind {
        self.kind
    }

    /// Returns the issue evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn issue(&self) -> &data_quality::Issue {
        &self.issue
    }

    /// Returns the source record refs evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }

    /// Returns the source freshness evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn source_freshness(&self) -> SourceFreshness {
        self.source_freshness
    }

    /// Returns the sensitivity evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn sensitivity(&self) -> Sensitivity {
        self.sensitivity
    }

    fn effective_source_record_refs(&self) -> Vec<source::RecordRef> {
        let mut refs = self.source_record_refs.clone();
        let issue_ref = self.issue.source_record_ref().clone();
        if !refs.contains(&issue_ref) {
            refs.push(issue_ref);
        }
        refs
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for action kind in the data-quality hygiene workflow; each value classifies evidence without proving review or independently authorizing a queue, draft, or staff gate.
pub enum ActionKind {
    /// Selects investigate missing source evidence for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    InvestigateMissingSourceEvidence,
    /// Selects reconcile duplicate customer or pet candidate for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    ReconcileDuplicateCustomerOrPetCandidate,
    /// Selects complete missing pet or customer profile fields for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    CompleteMissingPetOrCustomerProfileFields,
    /// Selects review stale vaccination source freshness for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    ReviewStaleVaccinationSourceFreshness,
    /// Selects normalize ambiguous service line naming for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    NormalizeAmbiguousServiceLineNaming,
    /// Selects review checkout or unclosed reservation evidence for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    ReviewCheckoutOrUnclosedReservationEvidence,
    /// Selects escalate sensitive or quarantined payload for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    EscalateSensitiveOrQuarantinedPayload,
    /// Selects protected payment conflict review for the data-quality hygiene decision model without moving money, changing invoices, or mutating provider records.
    ReviewPaymentStateConflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for action priority in the data-quality hygiene workflow; each value classifies evidence without proving review or independently authorizing a queue, draft, or staff gate.
pub enum ActionPriority {
    /// Selects high for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    High,
    /// Selects medium for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    Medium,
    /// Selects low for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for removed manual work in the data-quality hygiene workflow; each value classifies evidence without proving review or independently authorizing a queue, draft, or staff gate.
pub enum RemovedManualWork {
    /// Selects missing evidence investigation for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    MissingEvidenceInvestigation,
    /// Selects duplicate candidate reconciliation for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    DuplicateCandidateReconciliation,
    /// Selects incomplete profile cleanup preparation for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    IncompleteProfileCleanupPreparation,
    /// Selects source freshness review for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    SourceFreshnessReview,
    /// Selects service line normalization review for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    ServiceLineNormalizationReview,
    /// Selects checkout evidence review for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    CheckoutEvidenceReview,
    /// Selects sensitive payload escalation for the data-quality hygiene decision model so the app can choose a review, evidence, or draft path without taking live action.
    SensitivePayloadEscalation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Review-safe agent tasks allowed to save staff time without crossing mutation or send gates.
pub enum SafeAgentAction {
    /// Allows agents to summarize source evidence for staff review without mutating records or contacting customers.
    SummarizeSourceEvidence,
    /// Allows agents to rank hygiene actions for staff review without mutating records or contacting customers.
    RankHygieneActions,
    /// Allows agents to draft internal cleanup task for staff review without mutating records or contacting customers.
    DraftInternalCleanupTask,
    /// Allows agents to preserve ambiguity for review for staff review without mutating records or contacting customers.
    PreserveAmbiguityForReview,
    /// Allows agents to report a reconciliation estimate difference for staff review without claiming realized savings or enabling side effects.
    ReportReconciliationEstimateDifference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Actions the agent must never perform without a human/operator system of record.
pub enum BlockedAction {
    /// Blocks agents from send customer message until staff or the system of record performs the action.
    SendCustomerMessage,
    /// Blocks agents from mutate provider or pms record until staff or the system of record performs the action.
    MutateProviderOrPmsRecord,
    /// Blocks agents from change staff schedule until staff or the system of record performs the action.
    ChangeStaffSchedule,
    /// Blocks agents from move refund discount or payment until staff or the system of record performs the action.
    MoveRefundDiscountOrPayment,
    /// Blocks agents from hide or auto resolve source ambiguity until staff or the system of record performs the action.
    HideOrAutoResolveSourceAmbiguity,
    /// Blocks agents from expose quarantined sensitive payload until staff or the system of record performs the action.
    ExposeQuarantinedSensitivePayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Labor impact estimate used by the data-quality hygiene workflow; it finds duplicate, stale, or inconsistent records while blocking automatic provider-system mutation.
pub struct LaborImpactEstimate {
    before_minutes: LaborMinutes,
    after_minutes: LaborMinutes,
}

impl LaborImpactEstimate {
    /// Stores a caller-reported value for the data-quality hygiene workflow without proving review or triggering provider, customer, payment, or schedule side effects.
    pub const fn new(before_minutes: LaborMinutes, after_minutes: LaborMinutes) -> Self {
        Self {
            before_minutes,
            after_minutes,
        }
    }

    /// Returns the before minutes evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn before_minutes(&self) -> LaborMinutes {
        self.before_minutes
    }

    /// Returns the after minutes evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn after_minutes(&self) -> LaborMinutes {
        self.after_minutes
    }

    /// Returns a caller-reported estimate difference for prioritization, never a realized-savings claim.
    pub const fn reported_estimated_minutes_difference(&self) -> u16 {
        self.before_minutes.0.saturating_sub(self.after_minutes.0)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Action used by the data-quality hygiene workflow; it finds duplicate, stale, or inconsistent records while blocking automatic provider-system mutation.
pub struct Action {
    id: ActionId,
    kind: ActionKind,
    priority: ActionPriority,
    owner_persona: HygienePersona,
    removed_manual_work: RemovedManualWork,
    rationale: ActionRationale,
    #[builder(default)]
    source_record_refs: Vec<source::RecordRef>,
    #[builder(default)]
    issue_refs: Vec<IssueRef>,
    #[builder(default)]
    required_review_gates: Vec<policy::ReviewGate>,
    labor_impact: LaborImpactEstimate,
}

impl Action {
    /// Returns the id evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn id(&self) -> &ActionId {
        &self.id
    }

    /// Returns the kind evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn kind(&self) -> ActionKind {
        self.kind
    }

    /// Returns the priority evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn priority(&self) -> ActionPriority {
        self.priority
    }

    /// Returns the owner persona evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn owner_persona(&self) -> HygienePersona {
        self.owner_persona
    }

    /// Returns the removed manual work evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn removed_manual_work(&self) -> RemovedManualWork {
        self.removed_manual_work
    }

    /// Returns the rationale evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn rationale(&self) -> &ActionRationale {
        &self.rationale
    }

    /// Returns the source record refs evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }

    /// Returns the issue refs evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn issue_refs(&self) -> &[IssueRef] {
        &self.issue_refs
    }

    /// Returns the required review gates evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }

    /// Returns the labor impact evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn labor_impact(&self) -> &LaborImpactEstimate {
        &self.labor_impact
    }

    /// Names the internal cleanup action family without authorizing the action against the provider system.
    pub const fn cleanup_action(&self) -> CleanupAction {
        cleanup_action_for_kind(self.kind)
    }

    /// Names the human/operator role that must review this internal cleanup action.
    pub const fn reviewer_role(&self) -> ReviewerRole {
        reviewer_role_for_persona(self.owner_persona)
    }

    /// Reports that this action remains draft/review work until a human or system of record closes the gate.
    pub fn requires_human_or_system_of_record_review(&self) -> bool {
        !self.required_review_gates.is_empty()
            && blocked_actions_for().contains(&BlockedAction::MutateProviderOrPmsRecord)
    }

    /// Reports whether the data-quality hygiene workflow satisfies the is source grounded safety condition.
    pub fn is_source_grounded(&self) -> bool {
        !self.source_record_refs.is_empty() && !self.issue_refs.is_empty()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Input rules for building the workflow packet from source-grounded records.
pub struct Request {
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    prepared_for: HygienePersona,
    #[builder(default)]
    candidates: Vec<Candidate>,
}

impl Request {
    /// Returns the location id evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    /// Returns the operating day evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn operating_day(&self) -> operations::operating_day::Date {
        self.operating_day
    }

    /// Returns the prepared for evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn prepared_for(&self) -> HygienePersona {
        self.prepared_for
    }

    /// Returns the candidates evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }
}

#[derive(Clone, PartialEq, Eq, Serialize)]
/// Server-issued review packet with deterministic gates already applied.
///
/// It is serializable for presentation but intentionally not caller-deserializable; callers must
/// submit evidence through [`Request`] so the workflow can revalidate candidates and actions.
pub struct Packet {
    workflow: &'static str,
    schema_version: &'static str,
    context_packet_id: ContextPacketId,
    correlation_id: CorrelationId,
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    prepared_for: HygienePersona,
    candidates: Vec<Candidate>,
    actions: Vec<Action>,
    safe_agent_actions: Vec<SafeAgentAction>,
    blocked_actions: Vec<BlockedAction>,
    before_minutes: AggregateLaborMinutes,
    after_minutes: AggregateLaborMinutes,
}

impl Packet {
    /// Returns the workflow evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn workflow(&self) -> &'static str {
        self.workflow
    }

    /// Returns the schema version evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn schema_version(&self) -> &'static str {
        self.schema_version
    }

    /// Returns the context packet id evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn context_packet_id(&self) -> &ContextPacketId {
        &self.context_packet_id
    }

    /// Returns the correlation id evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }

    /// Returns the location id evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    /// Returns the operating day evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn operating_day(&self) -> operations::operating_day::Date {
        self.operating_day
    }

    /// Returns the prepared for evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn prepared_for(&self) -> HygienePersona {
        self.prepared_for
    }

    /// Returns the candidates evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    /// Returns the actions evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// Returns the safe agent actions evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn safe_agent_actions(&self) -> &[SafeAgentAction] {
        &self.safe_agent_actions
    }

    /// Returns the blocked actions evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn blocked_actions(&self) -> &[BlockedAction] {
        &self.blocked_actions
    }

    /// Returns the before minutes evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn before_minutes(&self) -> AggregateLaborMinutes {
        self.before_minutes
    }

    /// Returns the after minutes evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn after_minutes(&self) -> AggregateLaborMinutes {
        self.after_minutes
    }

    /// Returns a caller-reported estimate difference for prioritization, never a realized-savings claim.
    pub const fn reported_estimated_minutes_difference(&self) -> u16 {
        self.before_minutes.0.saturating_sub(self.after_minutes.0)
    }

    /// Returns the all actions are source grounded evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn all_actions_are_source_grounded(&self) -> bool {
        self.actions.iter().all(Action::is_source_grounded)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Draft action used by the data-quality hygiene workflow; it finds duplicate, stale, or inconsistent records while blocking automatic provider-system mutation.
pub struct DraftAction {
    action_id: ActionId,
    kind: ActionKind,
    source_record_refs: Vec<source::RecordRef>,
    issue_refs: Vec<IssueRef>,
    required_review_gates: Vec<policy::ReviewGate>,
    requested_side_effects: Vec<String>,
    attempted_ambiguity_resolution: bool,
}

impl DraftAction {}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Draft submission used by the data-quality hygiene workflow; it finds duplicate, stale, or inconsistent records while blocking automatic provider-system mutation.
pub struct DraftSubmission {
    context_packet_id: ContextPacketId,
    correlation_id: CorrelationId,
    #[builder(default)]
    actions: Vec<DraftAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Draft validation used by the data-quality hygiene workflow; it finds duplicate, stale, or inconsistent records while blocking automatic provider-system mutation.
pub struct DraftValidation {
    rejection_reasons: Vec<DraftRejectionReason>,
}

impl DraftValidation {
    /// Reports whether the data-quality hygiene workflow satisfies the is accepted safety condition.
    pub fn is_accepted(&self) -> bool {
        self.rejection_reasons.is_empty()
    }

    /// Returns the rejection reasons evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn rejection_reasons(&self) -> &[DraftRejectionReason] {
        &self.rejection_reasons
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Decision choices for draft rejection reason in the data-quality hygiene workflow; each value classifies evidence without proving review or independently authorizing a queue, draft, or staff gate.
pub enum DraftRejectionReason {
    /// Uses stale or unknown context packet as source-grounded evidence for the deterministic decision.
    StaleOrUnknownContextPacket,
    /// Uses unsupported action kind as source-grounded evidence for the deterministic decision.
    UnsupportedActionKind,
    /// Uses missing source refs as source-grounded evidence for the deterministic decision.
    MissingSourceRefs,
    /// Uses source refs not present in context packet as source-grounded evidence for the deterministic decision.
    SourceRefsNotPresentInContextPacket,
    /// Uses missing data quality issue refs as source-grounded evidence for the deterministic decision.
    MissingDataQualityIssueRefs,
    /// Uses wrong review gate as source-grounded evidence for the deterministic decision.
    WrongReviewGate,
    /// Uses blocked side effect requested as source-grounded evidence for the deterministic decision.
    BlockedSideEffectRequested,
    /// Uses unsupported side effect requested as source-grounded evidence for the deterministic decision.
    UnsupportedSideEffectRequested,
    /// Uses attempted ambiguity hiding as source-grounded evidence for the deterministic decision.
    AttemptedAmbiguityHiding,
    /// Uses sensitive payload exposure attempted as source-grounded evidence for the deterministic decision.
    SensitivePayloadExposureAttempted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Decision choices for error in the data-quality hygiene workflow; each value classifies evidence without proving review or independently authorizing a queue, draft, or staff gate.
pub enum Error {
    #[error("labor minutes must be greater than zero")]
    /// Identifies zero labor minutes as the reason the workflow must stop, retry, or request review.
    ZeroLaborMinutes,
    #[error("outcome record requires at least one source record ref")]
    /// Identifies missing source refs as the reason reported-outcome capture must stop.
    OutcomeSourceRecordRefRequired,
    #[error("outcome record requires at least one data-quality issue ref")]
    /// Identifies missing issue refs as the reason reported-outcome capture must stop.
    OutcomeIssueRefRequired,
    #[error("outcome record requires field {0}")]
    /// Identifies a missing required field as the reason reported-outcome capture must stop.
    OutcomeFieldRequired(&'static str),
    #[error("actor id did not resolve to a known data-quality hygiene reviewer")]
    /// Identifies missing actor-directory lookup as the reason reported-outcome admission must stop.
    ActorNotFound,
    #[error("review queue item was not found for the outcome action")]
    /// Identifies absent review-queue metadata as the reason reported-outcome admission must stop.
    ReviewQueueItemNotFound,
    #[error("actor is not authorized to record this data-quality hygiene outcome evidence")]
    /// Identifies a role or location policy failure as the reason reported-outcome admission must stop.
    ActorNotAuthorized,
}

/// Result type returned by fallible data quality hygiene operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Workflow used by the data-quality hygiene workflow; it finds duplicate, stale, or inconsistent records while blocking automatic provider-system mutation.
pub struct Workflow;

impl Workflow {
    /// Builds the evaluate result for the data-quality hygiene workflow from source-correlated evidence while preserving human review gates and draft-only side effects; source correlation does not prove review.
    pub fn evaluate(request: Request) -> Packet {
        let actions = request
            .candidates
            .iter()
            .map(action_for_candidate)
            .collect::<Vec<_>>();
        let before_minutes = total_before_minutes(&actions);
        let after_minutes = total_after_minutes(&actions);

        Packet {
            workflow: WORKFLOW_NAME,
            schema_version: SCHEMA_VERSION,
            context_packet_id: ContextPacketId::try_new(format!(
                "data-quality-hygiene-context:{:?}:{:?}",
                request.location_id, request.operating_day
            ))
            .expect("formatted context packet id is non-empty"),
            correlation_id: CorrelationId::try_new(format!(
                "data-quality-hygiene:{:?}:{:?}",
                request.location_id, request.operating_day
            ))
            .expect("formatted correlation id is non-empty"),
            location_id: request.location_id,
            operating_day: request.operating_day,
            prepared_for: request.prepared_for,
            candidates: request.candidates,
            actions,
            safe_agent_actions: safe_agent_actions_for(),
            blocked_actions: blocked_actions_for(),
            before_minutes,
            after_minutes,
        }
    }
}

const fn cleanup_action_for_kind(kind: ActionKind) -> CleanupAction {
    match kind {
        ActionKind::InvestigateMissingSourceEvidence => CleanupAction::InvestigateEvidence,
        ActionKind::ReconcileDuplicateCustomerOrPetCandidate => {
            CleanupAction::PrepareDuplicateReview
        }
        ActionKind::CompleteMissingPetOrCustomerProfileFields => {
            CleanupAction::PrepareProfileCompletion
        }
        ActionKind::ReviewStaleVaccinationSourceFreshness => {
            CleanupAction::PrepareVaccineSourceReview
        }
        ActionKind::NormalizeAmbiguousServiceLineNaming => CleanupAction::PrepareServiceLineMapping,
        ActionKind::ReviewCheckoutOrUnclosedReservationEvidence => {
            CleanupAction::PrepareCheckoutEvidenceReview
        }
        ActionKind::EscalateSensitiveOrQuarantinedPayload => {
            CleanupAction::EscalateSensitivePayload
        }
        ActionKind::ReviewPaymentStateConflict => CleanupAction::PreparePaymentConflictReview,
    }
}

const fn reviewer_role_for_persona(persona: HygienePersona) -> ReviewerRole {
    match persona {
        HygienePersona::GeneralManager => ReviewerRole::GeneralManager,
        HygienePersona::AssistantGeneralManager => ReviewerRole::AssistantGeneralManager,
        HygienePersona::FrontDeskLead => ReviewerRole::FrontDeskLead,
        HygienePersona::FrontDeskAgent => ReviewerRole::FrontDeskAgent,
        HygienePersona::RegionalOperator => ReviewerRole::RegionalOperator,
        HygienePersona::OperationsAnalyst => ReviewerRole::OperationsAnalyst,
    }
}

fn action_for_candidate(candidate: &Candidate) -> Action {
    let (kind, owner_persona, removed_manual_work, before, after) = action_shape_for(candidate);
    Action::builder()
        .id(
            ActionId::try_new(format!("dq-action-{}", candidate.id().as_str()))
                .expect("candidate ids are non-empty"),
        )
        .kind(kind)
        .priority(priority_for(candidate))
        .owner_persona(owner_persona)
        .removed_manual_work(removed_manual_work)
        .rationale(rationale_for(candidate))
        .source_record_refs(candidate.effective_source_record_refs())
        .issue_refs(vec![candidate.id.clone()])
        .required_review_gates(review_gates_for(candidate))
        .labor_impact(LaborImpactEstimate::new(
            LaborMinutes::try_new(before).expect("static before minutes are valid"),
            LaborMinutes::try_new(after).expect("static after minutes are valid"),
        ))
        .build()
}

fn action_shape_for(
    candidate: &Candidate,
) -> (ActionKind, HygienePersona, RemovedManualWork, u16, u16) {
    if candidate.sensitivity == Sensitivity::QuarantinedSensitivePayload {
        return (
            ActionKind::EscalateSensitiveOrQuarantinedPayload,
            HygienePersona::GeneralManager,
            RemovedManualWork::SensitivePayloadEscalation,
            20,
            8,
        );
    }

    match candidate.issue.kind() {
        data_quality::Kind::MissingVaccinationRecord => (
            ActionKind::ReviewStaleVaccinationSourceFreshness,
            HygienePersona::FrontDeskLead,
            RemovedManualWork::SourceFreshnessReview,
            25,
            10,
        ),
        data_quality::Kind::DuplicateSourceRecord => (
            ActionKind::ReconcileDuplicateCustomerOrPetCandidate,
            HygienePersona::GeneralManager,
            RemovedManualWork::DuplicateCandidateReconciliation,
            30,
            12,
        ),
        data_quality::Kind::IncompletePetProfile
        | data_quality::Kind::AmbiguousOwnerPetRelationship => (
            ActionKind::CompleteMissingPetOrCustomerProfileFields,
            HygienePersona::FrontDeskLead,
            RemovedManualWork::IncompleteProfileCleanupPreparation,
            20,
            7,
        ),
        data_quality::Kind::UnmappedServiceType | data_quality::Kind::LocationScopeAmbiguity => (
            ActionKind::NormalizeAmbiguousServiceLineNaming,
            HygienePersona::GeneralManager,
            RemovedManualWork::ServiceLineNormalizationReview,
            20,
            6,
        ),
        data_quality::Kind::CheckoutEvidenceMissing | data_quality::Kind::UnclosedReservation => (
            ActionKind::ReviewCheckoutOrUnclosedReservationEvidence,
            HygienePersona::FrontDeskLead,
            RemovedManualWork::CheckoutEvidenceReview,
            20,
            8,
        ),
        data_quality::Kind::SensitivePayloadQuarantined => (
            ActionKind::EscalateSensitiveOrQuarantinedPayload,
            HygienePersona::GeneralManager,
            RemovedManualWork::SensitivePayloadEscalation,
            20,
            8,
        ),
        data_quality::Kind::PaymentStateConflict => (
            ActionKind::ReviewPaymentStateConflict,
            HygienePersona::FrontDeskLead,
            RemovedManualWork::MissingEvidenceInvestigation,
            25,
            8,
        ),
        data_quality::Kind::MissingRequiredField { .. }
        | data_quality::Kind::AssumptionInForce { .. }
        | data_quality::Kind::UnknownSourceStatus { .. }
        | data_quality::Kind::ConflictingTimestamps => (
            ActionKind::InvestigateMissingSourceEvidence,
            HygienePersona::FrontDeskLead,
            RemovedManualWork::MissingEvidenceInvestigation,
            25,
            8,
        ),
    }
}

fn priority_for(candidate: &Candidate) -> ActionPriority {
    match candidate.issue.severity() {
        data_quality::Severity::Critical | data_quality::Severity::Blocking => ActionPriority::High,
        data_quality::Severity::Warning => ActionPriority::Medium,
        data_quality::Severity::Informational => ActionPriority::Low,
    }
}

fn rationale_for(candidate: &Candidate) -> ActionRationale {
    let text = match candidate.issue.kind() {
        data_quality::Kind::MissingVaccinationRecord => {
            "Route stale or missing vaccination source evidence to staff review while preserving ambiguity; this workflow does not approve service eligibility or send the customer a message."
        }
        data_quality::Kind::DuplicateSourceRecord => {
            "Prepare a source-grounded duplicate candidate for manager review without merging or mutating provider records."
        }
        data_quality::Kind::UnmappedServiceType => {
            "Prepare ambiguous service-line naming for manager review before reporting or labor automation consumes the source value."
        }
        data_quality::Kind::SensitivePayloadQuarantined => {
            "Escalate quarantined sensitive evidence as metadata only; do not expose raw payload contents to the agent."
        }
        _ => {
            "Prepare a source-grounded internal data-quality hygiene task for human review without hiding ambiguity or mutating source systems."
        }
    };
    ActionRationale::try_new(text).expect("static rationale is valid")
}

fn review_gates_for(candidate: &Candidate) -> Vec<policy::ReviewGate> {
    match candidate.issue.kind() {
        data_quality::Kind::MissingVaccinationRecord => vec![policy::ReviewGate::ManagerApproval],
        data_quality::Kind::SensitivePayloadQuarantined => {
            vec![policy::ReviewGate::ManagerApproval]
        }
        data_quality::Kind::PaymentStateConflict => vec![
            policy::ReviewGate::ManagerApproval,
            policy::ReviewGate::RefundOrDepositException,
        ],
        _ if matches!(
            candidate.issue.severity(),
            data_quality::Severity::Blocking | data_quality::Severity::Critical
        ) =>
        {
            vec![policy::ReviewGate::ManagerApproval]
        }
        _ => vec![policy::ReviewGate::ManagerApproval],
    }
}

fn total_before_minutes(actions: &[Action]) -> AggregateLaborMinutes {
    AggregateLaborMinutes::new(
        actions
            .iter()
            .map(|action| action.labor_impact.before_minutes().get())
            .sum::<u16>(),
    )
}

fn total_after_minutes(actions: &[Action]) -> AggregateLaborMinutes {
    AggregateLaborMinutes::new(
        actions
            .iter()
            .map(|action| action.labor_impact.after_minutes().get())
            .sum::<u16>(),
    )
}

fn safe_agent_actions_for() -> Vec<SafeAgentAction> {
    vec![
        SafeAgentAction::SummarizeSourceEvidence,
        SafeAgentAction::RankHygieneActions,
        SafeAgentAction::DraftInternalCleanupTask,
        SafeAgentAction::PreserveAmbiguityForReview,
        SafeAgentAction::ReportReconciliationEstimateDifference,
    ]
}

pub(super) fn blocked_actions_for() -> Vec<BlockedAction> {
    vec![
        BlockedAction::SendCustomerMessage,
        BlockedAction::MutateProviderOrPmsRecord,
        BlockedAction::ChangeStaffSchedule,
        BlockedAction::MoveRefundDiscountOrPayment,
        BlockedAction::HideOrAutoResolveSourceAmbiguity,
        BlockedAction::ExposeQuarantinedSensitivePayload,
    ]
}
