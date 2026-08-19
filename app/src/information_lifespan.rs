//! Information-lifespan trace contract for the synthetic Hermes demo.
//!
//! This module owns the canonical envelope that later API, database, Hermes,
//! and staff-web cards can replay. Source payloads are observed evidence only;
//! they never become NVA product authority and never authorize live source-system,
//! customer, payment, or scheduling side effects.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Stable request/run identifier that ties every demo proof panel together.
pub struct CorrelationId(String);

impl CorrelationId {
    /// Builds the deterministic demo correlation id.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the trace correlation id as a displayable string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Versioned public/agent schema tag for the trace envelope.
pub enum TraceSchemaVersion {
    /// Canonical v1 information-lifespan trace contract used by the local Manager Daily Report demo.
    #[serde(rename = "information_lifespan_trace.v1")]
    V1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Source system posture for the trace; the demo currently uses only mocked provider evidence.
pub enum SourceSystem {
    /// Synthetic read-only provider-shaped fixture used for local demo proof.
    MockProviderReadOnlyFixture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Ordered stage kinds in the visible information-lifespan machine.
pub enum StageKind {
    /// Mocked provider evidence was received from a fixture, not a live provider.
    SourceEvidenceReceived,
    /// Provider DTO/source-model shape is preserved without treating it as domain truth.
    ProviderDtoPreserved,
    /// NVA-owned semantic app/domain models are assembled from source evidence.
    NormalizedNvaModels,
    /// Local DB rows or projections prove durable lineage.
    DatabaseProjectionProof,
    /// Dockerized Hermes processor step consumes/enriches the trace.
    HermesProcessorRun,
    /// Manager-report calculations/ranking/labor proof are produced.
    CalculationApplied,
    /// Unsafe side effects are locked behind review gates.
    ReviewGateLocked,
    /// Final manager daily report artifact is emitted for UI/API display.
    ManagerDailyReportArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Source-evidence family preserved in the trace drawer.
pub enum SourceEvidenceKind {
    /// Reservation or stay source evidence.
    Reservation,
    /// Care-note source evidence.
    CareNote,
    /// Vaccination source evidence.
    Vaccine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// HTTP methods displayed in the network proof panel.
pub enum HttpMethod {
    /// HTTP POST request.
    Post,
    /// HTTP GET request.
    Get,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Review/safety gate categories displayed by the trace.
pub enum SafetyGateKind {
    /// Provider/PMS writes are not allowed in the demo.
    ProviderWriteLocked,
    /// Customer sends are not allowed in the demo.
    CustomerSendLocked,
    /// Medical/vaccine acceptance remains staff-reviewed.
    MedicalReviewRequired,
    /// Schedule/staffing changes remain human/system-of-record owned.
    ScheduleChangeLocked,
    /// Payment/refund/discount movement remains locked.
    PaymentMovementLocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Final artifact families produced by an information-lifespan run.
pub enum FinalArtifactKind {
    /// Manager Daily Report artifact with lineage and labor proof.
    ManagerDailyReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Type path surfaced in the model/source proof drawer.
pub struct ModelPath {
    path: String,
    role: String,
}

impl ModelPath {
    /// Records a source, domain, storage, API, or artifact model path with its trace role.
    pub fn new(path: impl Into<String>, role: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            role: role.into(),
        }
    }

    /// Rust/module path shown in the proof drawer.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Human-facing explanation of the path's authority in the trace.
    pub fn role(&self) -> &str {
        &self.role
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
/// Observed source evidence retained without provider-specific shape or executable authority.
pub struct ObservedSourceEvidence {
    evidence_kind: SourceEvidenceKind,
    source_ref: String,
    value: serde_json::Value,
}

impl fmt::Debug for ObservedSourceEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ObservedSourceEvidence([REDACTED])")
    }
}

impl ObservedSourceEvidence {
    /// Captures an observed value as non-authoritative source evidence.
    pub fn new(
        evidence_kind: SourceEvidenceKind,
        source_ref: impl Into<String>,
        value: serde_json::Value,
    ) -> Self {
        Self {
            evidence_kind,
            source_ref: source_ref.into(),
            value,
        }
    }

    /// Evidence family used by the trace.
    pub const fn evidence_kind(&self) -> SourceEvidenceKind {
        self.evidence_kind
    }

    /// Adapter-owned reference to the observed source value.
    pub fn source_ref(&self) -> &str {
        &self.source_ref
    }

    /// Observed source value retained as evidence only.
    pub const fn value(&self) -> &serde_json::Value {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// One visible stage in the information-lifespan trace.
pub struct TraceStage {
    kind: StageKind,
    title: String,
    simulated: bool,
    why_simulated: Option<String>,
    model_paths: Vec<ModelPath>,
}

impl TraceStage {
    /// Builds a visible trace stage for API/UI replay.
    pub fn new(
        kind: StageKind,
        title: impl Into<String>,
        simulated: bool,
        why_simulated: Option<String>,
        model_paths: Vec<ModelPath>,
    ) -> Self {
        Self {
            kind,
            title: title.into(),
            simulated,
            why_simulated,
            model_paths,
        }
    }

    /// Stage kind used by UI timeline ordering.
    pub const fn kind(&self) -> StageKind {
        self.kind
    }

    /// Human-facing stage title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Whether this stage is simulated in the current card's fixture.
    pub const fn simulated(&self) -> bool {
        self.simulated
    }

    /// Reason a simulated stage is not yet backed by a later-card runtime surface.
    pub fn why_simulated(&self) -> Option<&str> {
        self.why_simulated.as_deref()
    }

    /// Model/type paths visible for this stage.
    pub fn model_paths(&self) -> &[ModelPath] {
        &self.model_paths
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Structured log line proof that can be rendered without secrets or real customer data.
pub struct LogProofEntry {
    level: String,
    target: String,
    message: String,
}

impl LogProofEntry {
    /// Builds a safe structured-log proof line.
    pub fn new(
        level: impl Into<String>,
        target: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            level: level.into(),
            target: target.into(),
            message: message.into(),
        }
    }

    /// Log level label.
    pub fn level(&self) -> &str {
        &self.level
    }

    /// Structured log target/module.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Redacted/synthetic log message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Local database row/projection proof attached to the trace.
pub struct DbProofEntry {
    table_or_view: String,
    proof_ref: String,
    model_path: String,
}

impl DbProofEntry {
    /// Builds a DB proof pointer without embedding sensitive row payloads.
    pub fn new(
        table_or_view: impl Into<String>,
        proof_ref: impl Into<String>,
        model_path: impl Into<String>,
    ) -> Self {
        Self {
            table_or_view: table_or_view.into(),
            proof_ref: proof_ref.into(),
            model_path: model_path.into(),
        }
    }

    /// Table or view name.
    pub fn table_or_view(&self) -> &str {
        &self.table_or_view
    }

    /// Stable synthetic row/projection reference.
    pub fn proof_ref(&self) -> &str {
        &self.proof_ref
    }

    /// Storage/API/domain model path associated with the row/projection.
    pub fn model_path(&self) -> &str {
        &self.model_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// HTTP request/response proof for browser-visible network panels.
pub struct NetworkProofEntry {
    method: HttpMethod,
    path: String,
    status: u16,
    response_ref: String,
}

impl NetworkProofEntry {
    /// Builds an HTTP proof entry without requiring a live network call in this scaffold card.
    pub fn new(
        method: HttpMethod,
        path: impl Into<String>,
        status: u16,
        response_ref: impl Into<String>,
    ) -> Self {
        Self {
            method,
            path: path.into(),
            status,
            response_ref: response_ref.into(),
        }
    }

    /// HTTP method.
    pub const fn method(&self) -> HttpMethod {
        self.method
    }

    /// API route path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// HTTP response status code.
    pub const fn status(&self) -> u16 {
        self.status
    }

    /// Synthetic response-body reference.
    pub fn response_ref(&self) -> &str {
        &self.response_ref
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Calculation proof line shown before the final report artifact.
pub struct CalculationProof {
    name: String,
    expression: String,
    result: String,
}

impl CalculationProof {
    /// Builds a deterministic calculation proof.
    pub fn new(
        name: impl Into<String>,
        expression: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            expression: expression.into(),
            result: result.into(),
        }
    }

    /// Calculation name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Human-readable expression.
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// Deterministic result.
    pub fn result(&self) -> &str {
        &self.result
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Safety/review gate proof that marks unsafe side effects as locked.
pub struct SafetyGateProof {
    gate: SafetyGateKind,
    locked: bool,
    reason: String,
}

impl SafetyGateProof {
    /// Builds a locked or visible review gate.
    pub fn new(gate: SafetyGateKind, locked: bool, reason: impl Into<String>) -> Self {
        Self {
            gate,
            locked,
            reason: reason.into(),
        }
    }

    /// Safety gate category.
    pub const fn gate(&self) -> SafetyGateKind {
        self.gate
    }

    /// Whether the side effect remains locked.
    pub const fn locked(&self) -> bool {
        self.locked
    }

    /// Review/safety explanation.
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Final Manager Daily Report artifact descriptor.
pub struct FinalArtifact {
    artifact_kind: FinalArtifactKind,
    title: String,
    artifact_ref: String,
    summary: String,
}

impl FinalArtifact {
    /// Builds the final report artifact descriptor.
    pub fn new(
        artifact_kind: FinalArtifactKind,
        title: impl Into<String>,
        artifact_ref: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            artifact_kind,
            title: title.into(),
            artifact_ref: artifact_ref.into(),
            summary: summary.into(),
        }
    }

    /// Artifact kind.
    pub const fn artifact_kind(&self) -> FinalArtifactKind {
        self.artifact_kind
    }

    /// Artifact title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Stable artifact ref used by API/UI panels.
    pub fn artifact_ref(&self) -> &str {
        &self.artifact_ref
    }

    /// Human-facing report summary.
    pub fn summary(&self) -> &str {
        &self.summary
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, bon::Builder)]
/// Canonical reusable information-lifespan trace envelope.
pub struct TraceEnvelope {
    schema_version: TraceSchemaVersion,
    correlation_id: CorrelationId,
    source_system: SourceSystem,
    synthetic_data_only: bool,
    provider_payloads_are_source_evidence_only: bool,
    live_side_effects_allowed: bool,
    source_evidence: Vec<ObservedSourceEvidence>,
    stages: Vec<TraceStage>,
    log_proof_entries: Vec<LogProofEntry>,
    db_proof_entries: Vec<DbProofEntry>,
    network_proof_entries: Vec<NetworkProofEntry>,
    calculations: Vec<CalculationProof>,
    safety_gates: Vec<SafetyGateProof>,
    final_artifact: FinalArtifact,
}

impl TraceEnvelope {
    /// Public/agent schema version that makes the durable trace payload contract explicit.
    pub const fn schema_version(&self) -> TraceSchemaVersion {
        self.schema_version
    }

    /// Correlation id linking the source, DB, processor, network, and artifact panels.
    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }

    /// Source-system posture for this trace.
    pub const fn source_system(&self) -> SourceSystem {
        self.source_system
    }

    /// Returns true when the trace contains no live customer/provider data.
    pub const fn uses_synthetic_data_only(&self) -> bool {
        self.synthetic_data_only
    }

    /// Returns true when provider payloads are evidence only, never product truth.
    pub const fn provider_payloads_are_source_evidence_only(&self) -> bool {
        self.provider_payloads_are_source_evidence_only
    }

    /// Returns false for this safe demo scaffold.
    pub const fn live_side_effects_allowed(&self) -> bool {
        self.live_side_effects_allowed
    }

    /// Observed source evidence that begins the trace.
    pub fn source_evidence(&self) -> &[ObservedSourceEvidence] {
        &self.source_evidence
    }

    /// Ordered visible stages.
    pub fn stages(&self) -> &[TraceStage] {
        &self.stages
    }

    /// Safe structured log proof entries.
    pub fn log_proof_entries(&self) -> &[LogProofEntry] {
        &self.log_proof_entries
    }

    /// Database/projection proof entries.
    pub fn db_proof_entries(&self) -> &[DbProofEntry] {
        &self.db_proof_entries
    }

    /// Browser/API network proof entries.
    pub fn network_proof_entries(&self) -> &[NetworkProofEntry] {
        &self.network_proof_entries
    }

    /// Calculation proof entries.
    pub fn calculations(&self) -> &[CalculationProof] {
        &self.calculations
    }

    /// Safety/review gate proof entries.
    pub fn safety_gates(&self) -> &[SafetyGateProof] {
        &self.safety_gates
    }

    /// Final report artifact descriptor.
    pub const fn final_artifact(&self) -> &FinalArtifact {
        &self.final_artifact
    }
}

impl fmt::Debug for TraceEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TraceEnvelope")
            .field("schema_version", &self.schema_version)
            .field("correlation_id", &self.correlation_id)
            .field("source_system", &self.source_system)
            .field("synthetic_data_only", &self.synthetic_data_only)
            .field(
                "provider_payloads_are_source_evidence_only",
                &self.provider_payloads_are_source_evidence_only,
            )
            .field("live_side_effects_allowed", &self.live_side_effects_allowed)
            .field("stages_count", &self.stages.len())
            .field("log_proof_entries_count", &self.log_proof_entries.len())
            .field("db_proof_entries_count", &self.db_proof_entries.len())
            .field(
                "network_proof_entries_count",
                &self.network_proof_entries.len(),
            )
            .field("calculations_count", &self.calculations.len())
            .field("safety_gates_count", &self.safety_gates.len())
            .field("final_artifact", &self.final_artifact)
            .finish()
    }
}
