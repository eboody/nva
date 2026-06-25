#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Classifies agent runtime modes used by the worker shell.
///
/// The mode chooses whether workflow packets are answered by deterministic fixtures or
/// skipped entirely; neither variant grants authority to perform live customer messaging
/// or provider writes.
pub enum AgentRuntimeMode {
    /// Uses deterministic fixtures so local workers can exercise packet flow without calling live agents.
    FakeDeterministic,
    /// Skips agent execution while keeping the worker process and side-effect stubs available.
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Classifies side-effect posture for the worker runtime.
///
/// Current workers expose only stubbed side effects so tests and demos cannot write to
/// Gingr, payment providers, SMS/email systems, or customer-facing channels.
pub enum SideEffectMode {
    /// Keeps provider writes, customer sends, and payment movement behind no-op test doubles.
    Stubbed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Names review gates that remain mandatory when a worker claims durable work.
///
/// These gates mirror the MVP migration review vocabulary. Worker code may carry
/// the gate through a processing plan, but it cannot convert a gate into execution
/// authority.
pub enum ReviewGate {
    /// Manager/operator must review before the proposed action can execute.
    ManagerApproval,
    /// Staff/manager review is required for medical-document/vaccine facts.
    MedicalDocumentReview,
    /// Behavior/play-safety review is required before acting on the recommendation.
    BehaviorReview,
    /// Customer-facing outbound copy must be approved before delivery.
    CustomerMessageApproval,
    /// Deposit, payment, refund, waiver, or credit exceptions require approval.
    RefundOrDepositException,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Worker-visible status for a claimed outbox/workflow record.
///
/// The MVP worker only produces review-gated stubs. A claimed record can be
/// inspected, traced, and mapped to a review packet; it is not publishable by this
/// runtime.
pub enum OutboxProcessingStatus {
    /// The claimed record remains blocked on human review and uses stubbed adapters.
    ReviewGatedStub,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Durable workflow/outbox work claimed by the worker runtime.
///
/// This is a local contract type rather than a database adapter. It captures the
/// fields the eventual leasing query must preserve: the durable workflow event,
/// semantic workflow name, and required review gate.
pub struct ClaimedWorkflowRecord {
    workflow_event_ref: String,
    workflow_name: String,
    required_review_gate: ReviewGate,
}

impl ClaimedWorkflowRecord {
    /// Builds a local representation of a durable workflow/outbox claim.
    pub fn new(
        workflow_event_ref: impl Into<String>,
        workflow_name: impl Into<String>,
        required_review_gate: ReviewGate,
    ) -> Self {
        Self {
            workflow_event_ref: workflow_event_ref.into(),
            workflow_name: workflow_name.into(),
            required_review_gate,
        }
    }

    /// Returns the durable workflow event reference carried by the claim.
    pub fn workflow_event_ref(&self) -> &str {
        &self.workflow_event_ref
    }

    /// Returns the semantic workflow name carried by the claim.
    pub fn workflow_name(&self) -> &str {
        &self.workflow_name
    }

    /// Returns the review gate that still blocks external execution.
    pub fn required_review_gate(&self) -> ReviewGate {
        self.required_review_gate
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Local processing contract for a claimed durable workflow/outbox record.
///
/// The plan intentionally separates durable storage evidence from side-effect
/// authority: the worker may process a packet into a reviewable result, but every
/// live customer/provider/payment boundary stays blocked by the stubbed posture.
pub struct ProcessingContract {
    workflow_event_ref: String,
    workflow_name: String,
    required_review_gate: ReviewGate,
    agent_runtime_mode: AgentRuntimeMode,
    side_effect_mode: SideEffectMode,
    outbox_status: OutboxProcessingStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Auditable worker proof for one reviewed Data-Quality Hygiene storage projection.
///
/// This proof is deliberately local/fake. It lets the demo narrate a worker seeing
/// a reviewed outcome and internal outbox candidate, while the worker posture still
/// blocks customer messages, provider/PMS writes, and payment movement.
pub struct DataQualityHygieneWorkerProof {
    workflow_event_ref: String,
    workflow_name: String,
    correlation_id: String,
    required_review_gate: ReviewGate,
    agent_runtime_mode: AgentRuntimeMode,
    side_effect_mode: SideEffectMode,
    outbox_status: OutboxProcessingStatus,
    outbox_candidate_id: Option<String>,
    outbox_topic: Option<String>,
    has_reviewed_outcome: bool,
    audit_event_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Safe low-cardinality telemetry fields emitted by the local worker proof.
///
/// These fields are intentionally IDs/status classes only. They never include raw
/// customer text, provider payloads, message bodies, payment data, or secrets.
pub struct WorkerTelemetryFields {
    event: &'static str,
    workflow_event_id: String,
    correlation_id: String,
    worker_id: &'static str,
    attempt_count: u16,
    safe_error_class: &'static str,
    payload_logging: &'static str,
    live_delivery_allowed: bool,
}

impl ProcessingContract {
    fn review_gated_stub(config: Config, claim: &ClaimedWorkflowRecord) -> Self {
        Self {
            workflow_event_ref: claim.workflow_event_ref().to_owned(),
            workflow_name: claim.workflow_name().to_owned(),
            required_review_gate: claim.required_review_gate(),
            agent_runtime_mode: config.agent_runtime_mode,
            side_effect_mode: config.side_effect_mode,
            outbox_status: OutboxProcessingStatus::ReviewGatedStub,
        }
    }

    /// Returns the durable workflow event reference the worker is processing.
    pub fn workflow_event_ref(&self) -> &str {
        &self.workflow_event_ref
    }

    /// Returns the semantic workflow name the worker is processing.
    pub fn workflow_name(&self) -> &str {
        &self.workflow_name
    }

    /// Returns the review gate that still blocks external delivery.
    pub fn required_review_gate(&self) -> ReviewGate {
        self.required_review_gate
    }

    /// Returns the configured agent runtime posture.
    pub fn agent_runtime_mode(&self) -> AgentRuntimeMode {
        self.agent_runtime_mode
    }

    /// Returns the configured side-effect posture.
    pub fn side_effect_mode(&self) -> SideEffectMode {
        self.side_effect_mode
    }

    /// Returns the worker-visible outbox status for the claimed record.
    pub fn outbox_status(&self) -> OutboxProcessingStatus {
        self.outbox_status
    }

    /// True when the plan cannot cross an external boundary without human review.
    pub fn requires_human_review_before_external_delivery(&self) -> bool {
        self.outbox_status == OutboxProcessingStatus::ReviewGatedStub
    }

    /// True when customer-facing sends remain unavailable to this worker runtime.
    pub fn blocks_live_customer_messages(&self) -> bool {
        self.side_effect_mode == SideEffectMode::Stubbed
    }

    /// True when provider/PMS writes remain unavailable to this worker runtime.
    pub fn blocks_live_provider_writes(&self) -> bool {
        self.side_effect_mode == SideEffectMode::Stubbed
    }

    /// True when payment/deposit/refund movement remains unavailable to this worker runtime.
    pub fn blocks_live_payment_actions(&self) -> bool {
        self.side_effect_mode == SideEffectMode::Stubbed
    }
}

impl DataQualityHygieneWorkerProof {
    fn from_projection(
        config: Config,
        records: &storage::operations::DataQualityHygieneLocalPersistenceRecords,
    ) -> Self {
        Self {
            workflow_event_ref: records.workflow_event.id.clone(),
            workflow_name: records.workflow_event.workflow_name.clone(),
            correlation_id: records.outcome.record.correlation_id.clone(),
            required_review_gate: ReviewGate::ManagerApproval,
            agent_runtime_mode: config.agent_runtime_mode,
            side_effect_mode: config.side_effect_mode,
            outbox_status: OutboxProcessingStatus::ReviewGatedStub,
            outbox_candidate_id: records
                .outbox_candidate
                .as_ref()
                .map(|candidate| candidate.id.clone()),
            outbox_topic: records
                .outbox_candidate
                .as_ref()
                .map(|candidate| candidate.topic.clone()),
            has_reviewed_outcome: Self::has_reviewed_internal_handoff(records),
            audit_event_count: records.audit_events.len(),
        }
    }

    fn has_reviewed_internal_handoff(
        records: &storage::operations::DataQualityHygieneLocalPersistenceRecords,
    ) -> bool {
        use storage::operations::{
            OutboxStatusCode, ReviewGateCode, ReviewPacketStatusCode, WorkflowResultStatusCode,
        };

        let Some(candidate) = records.outbox_candidate.as_ref() else {
            return false;
        };

        records.workflow_result.status == WorkflowResultStatusCode::Succeeded
            && records.review_packet.status == ReviewPacketStatusCode::Approved
            && records.review_packet.gate == ReviewGateCode::ManagerApproval
            && records.approval_record.status == "approved"
            && records.approval_record.gate == ReviewGateCode::ManagerApproval
            && candidate.review_gate == ReviewGateCode::ManagerApproval
            && candidate.status == OutboxStatusCode::Pending
            && candidate.topic == "internal.data_quality_hygiene.reviewed_handoff"
            && candidate
                .payload
                .get("internal_handoff_only")
                .and_then(serde_json::Value::as_bool)
                == Some(true)
            && candidate
                .payload
                .get("live_delivery_allowed")
                .and_then(serde_json::Value::as_bool)
                == Some(false)
    }

    /// Returns the durable workflow event reference processed by the fake worker proof.
    pub fn workflow_event_ref(&self) -> &str {
        &self.workflow_event_ref
    }

    /// Returns the semantic workflow name processed by the fake worker proof.
    pub fn workflow_name(&self) -> &str {
        &self.workflow_name
    }

    /// Returns the business correlation id carried from reviewed outcome storage.
    pub fn correlation_id(&self) -> &str {
        &self.correlation_id
    }

    /// Returns the review gate that still blocks external delivery.
    pub fn required_review_gate(&self) -> ReviewGate {
        self.required_review_gate
    }

    /// Returns the configured agent runtime posture for the fake/disabled worker.
    pub fn agent_runtime_mode(&self) -> AgentRuntimeMode {
        self.agent_runtime_mode
    }

    /// Returns the configured side-effect posture for the fake/disabled worker.
    pub fn side_effect_mode(&self) -> SideEffectMode {
        self.side_effect_mode
    }

    /// Returns the worker-visible outbox status for the reviewed handoff candidate.
    pub fn outbox_status(&self) -> OutboxProcessingStatus {
        self.outbox_status
    }

    /// Returns the local outbox candidate id, if the reviewed outcome produced one.
    pub fn outbox_candidate_id(&self) -> Option<&str> {
        self.outbox_candidate_id.as_deref()
    }

    /// Returns the local internal handoff topic, if the reviewed outcome produced one.
    pub fn outbox_topic(&self) -> Option<&str> {
        self.outbox_topic.as_deref()
    }

    /// True when storage evidence says the local outcome reached reviewed/approved posture.
    pub fn has_reviewed_outcome(&self) -> bool {
        self.has_reviewed_outcome
    }

    /// Returns the count of audit events available to narrate the fake worker handoff.
    pub fn audit_event_count(&self) -> usize {
        self.audit_event_count
    }

    /// Returns safe telemetry fields for logs/metrics/readiness narration.
    pub fn telemetry_fields(&self) -> WorkerTelemetryFields {
        WorkerTelemetryFields {
            event: "outbox_blocked",
            workflow_event_id: self.workflow_event_ref.clone(),
            correlation_id: self.correlation_id.clone(),
            worker_id: "local-disabled-worker",
            attempt_count: 1,
            safe_error_class: "review_gated_stub",
            payload_logging: "disabled",
            live_delivery_allowed: false,
        }
    }

    /// True when the proof is intentionally local/fake and cannot publish externally.
    pub fn is_fake_local_only(&self) -> bool {
        self.agent_runtime_mode == AgentRuntimeMode::Disabled
            && self.side_effect_mode == SideEffectMode::Stubbed
            && self.outbox_status == OutboxProcessingStatus::ReviewGatedStub
    }

    /// True when the proof cannot cross an external boundary without human review.
    pub fn requires_human_review_before_external_delivery(&self) -> bool {
        self.outbox_status == OutboxProcessingStatus::ReviewGatedStub
    }

    /// True when customer-facing sends remain unavailable to this worker runtime.
    pub fn blocks_live_customer_messages(&self) -> bool {
        self.side_effect_mode == SideEffectMode::Stubbed
    }

    /// True when provider/PMS writes remain unavailable to this worker runtime.
    pub fn blocks_live_provider_writes(&self) -> bool {
        self.side_effect_mode == SideEffectMode::Stubbed
    }

    /// True when payment/deposit/refund movement remains unavailable to this worker runtime.
    pub fn blocks_live_payment_actions(&self) -> bool {
        self.side_effect_mode == SideEffectMode::Stubbed
    }
}

impl WorkerTelemetryFields {
    /// Stable telemetry event name emitted by the worker proof.
    pub fn event(&self) -> &'static str {
        self.event
    }

    /// Workflow event identifier that links telemetry to durable review/audit evidence.
    pub fn workflow_event_id(&self) -> &str {
        &self.workflow_event_id
    }

    /// Correlation identifier shared across safe local worker telemetry and API traces.
    pub fn correlation_id(&self) -> &str {
        &self.correlation_id
    }

    /// Stable worker identifier for BI/ops grouping.
    pub fn worker_id(&self) -> &'static str {
        self.worker_id
    }

    /// Number of local deterministic attempts represented by this telemetry proof.
    pub fn attempt_count(&self) -> u16 {
        self.attempt_count
    }

    /// Redacted error class safe for logs, metrics, and read models.
    pub fn safe_error_class(&self) -> &'static str {
        self.safe_error_class
    }

    /// Payload logging posture; remains disabled for safe local proofs.
    pub fn payload_logging(&self) -> &'static str {
        self.payload_logging
    }

    /// Whether this worker proof may deliver externally; false for local/stubbed runtimes.
    pub fn live_delivery_allowed(&self) -> bool {
        self.live_delivery_allowed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Configuration kept on the worker runtime.
///
/// The config is intentionally small: it selects deterministic agent execution and an
/// explicit side-effect posture so durable workflow workers can be inspected without
/// confusing storage evidence with permission to act in live systems.
pub struct Config {
    agent_runtime_mode: AgentRuntimeMode,
    side_effect_mode: SideEffectMode,
}

impl Config {
    /// Reads safe local defaults from the environment without enabling live side effects.
    ///
    /// `PET_RESORT_AGENT_RUNTIME_MODE=disabled` turns agent execution off; every other
    /// value falls back to deterministic fixtures. Side effects remain [`SideEffectMode::Stubbed`].
    pub fn from_env_defaults() -> Self {
        let agent_runtime_mode = match std::env::var("PET_RESORT_AGENT_RUNTIME_MODE")
            .unwrap_or_else(|_| "fake".to_owned())
            .as_str()
        {
            "disabled" => AgentRuntimeMode::Disabled,
            _ => AgentRuntimeMode::FakeDeterministic,
        };

        Self {
            agent_runtime_mode,
            side_effect_mode: SideEffectMode::Stubbed,
        }
    }

    /// Builds a disabled-agent config for tests while preserving stubbed side effects.
    pub fn disabled_for_tests() -> Self {
        Self {
            agent_runtime_mode: AgentRuntimeMode::Disabled,
            side_effect_mode: SideEffectMode::Stubbed,
        }
    }

    /// Returns the agent runtime mode kept on this worker runtime value.
    pub fn agent_runtime_mode(&self) -> AgentRuntimeMode {
        self.agent_runtime_mode
    }

    /// Returns the side effect mode kept on this worker runtime value.
    pub fn side_effect_mode(&self) -> SideEffectMode {
        self.side_effect_mode
    }

    /// Produces a local processing plan for a claimed durable workflow/outbox record.
    ///
    /// This is the worker contract proof for the current MVP: claiming durable work
    /// makes the record observable and review-routable, not externally executable.
    pub fn processing_contract_for(&self, claim: &ClaimedWorkflowRecord) -> ProcessingContract {
        ProcessingContract::review_gated_stub(*self, claim)
    }

    /// Processes a reviewed Data-Quality Hygiene storage projection into fake local worker proof.
    ///
    /// The worker only reflects the reviewed outcome and approved internal handoff candidate;
    /// it does not publish the outbox row or cross customer, provider, payment, or schedule
    /// boundaries. Retry/dead-letter leasing is not implemented in this local proof yet.
    pub fn process_data_quality_hygiene_projection(
        &self,
        records: &storage::operations::DataQualityHygieneLocalPersistenceRecords,
    ) -> DataQualityHygieneWorkerProof {
        DataQualityHygieneWorkerProof::from_projection(*self, records)
    }
}
