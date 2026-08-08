//! Information-lifespan trace contract for the synthetic Hermes demo.
//!
//! This module owns the canonical envelope that later API, database, Hermes,
//! and staff-web cards can replay. Gingr-shaped payloads here are synthetic,
//! read-only source evidence only; they never become NVA product authority and
//! never authorize live PMS/provider/customer/payment side effects.

use serde::{Deserialize, Serialize};
use serde_json::json;

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
#[serde(rename_all = "snake_case")]
/// Source system posture for the trace; the demo currently uses only mocked Gingr evidence.
pub enum SourceSystem {
    /// Synthetic read-only Gingr-shaped fixture used for local demo proof.
    MockGingrReadOnlyFixture,
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
/// Provider-shaped payload family preserved in the source evidence drawer.
pub enum SourcePayloadKind {
    /// Synthetic Gingr reservation/stay payload.
    Reservation,
    /// Synthetic Gingr care-note payload.
    CareNote,
    /// Synthetic Gingr vaccination payload.
    Vaccine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Authority assigned to a source payload before NVA normalization.
pub enum PayloadAuthority {
    /// Payload is evidence only and must be promoted through NVA rules before use.
    ProviderEvidenceOnly,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Synthetic provider payload plus provenance labels.
pub struct SourcePayload {
    payload_kind: SourcePayloadKind,
    authority: PayloadAuthority,
    provider_model_path: String,
    nva_target_model_path: String,
    raw_payload_ref: String,
    payload: serde_json::Value,
}

impl SourcePayload {
    /// Captures a mocked provider payload as source evidence only.
    pub fn new(
        payload_kind: SourcePayloadKind,
        provider_model_path: impl Into<String>,
        nva_target_model_path: impl Into<String>,
        raw_payload_ref: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            payload_kind,
            authority: PayloadAuthority::ProviderEvidenceOnly,
            provider_model_path: provider_model_path.into(),
            nva_target_model_path: nva_target_model_path.into(),
            raw_payload_ref: raw_payload_ref.into(),
            payload,
        }
    }

    /// Payload family used by report-relevant source facts.
    pub const fn payload_kind(&self) -> SourcePayloadKind {
        self.payload_kind
    }

    /// Authority boundary attached to the payload.
    pub const fn authority(&self) -> PayloadAuthority {
        self.authority
    }

    /// Provider DTO/source model path that preserves the raw evidence shape.
    pub fn provider_model_path(&self) -> &str {
        &self.provider_model_path
    }

    /// NVA-owned model path that later stages may promote into after validation.
    pub fn nva_target_model_path(&self) -> &str {
        &self.nva_target_model_path
    }

    /// Fixture/ref for the raw provider-shaped payload.
    pub fn raw_payload_ref(&self) -> &str {
        &self.raw_payload_ref
    }

    /// Provider-shaped synthetic payload.
    pub const fn payload(&self) -> &serde_json::Value {
        &self.payload
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Canonical reusable information-lifespan trace envelope.
pub struct TraceEnvelope {
    correlation_id: CorrelationId,
    source_system: SourceSystem,
    synthetic_data_only: bool,
    provider_payloads_are_source_evidence_only: bool,
    live_side_effects_allowed: bool,
    source_payloads: Vec<SourcePayload>,
    stages: Vec<TraceStage>,
    log_proof_entries: Vec<LogProofEntry>,
    db_proof_entries: Vec<DbProofEntry>,
    network_proof_entries: Vec<NetworkProofEntry>,
    calculations: Vec<CalculationProof>,
    safety_gates: Vec<SafetyGateProof>,
    final_artifact: FinalArtifact,
}

impl TraceEnvelope {
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

    /// Source payloads that begin the trace.
    pub fn source_payloads(&self) -> &[SourcePayload] {
        &self.source_payloads
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

/// Builds the deterministic synthetic trace fixture used by Piece 1 and later replay cards.
pub fn mock_gingr_manager_daily_report_trace() -> TraceEnvelope {
    let source_payloads = vec![
        SourcePayload::new(
            SourcePayloadKind::Reservation,
            "gingr::response::ReservationRecord",
            "domain::reservation::StayFact",
            "fixture://mock-gingr/reservations/9001001.json",
            json!({
                "synthetic": true,
                "id": 9001001,
                "owner_id": 7001,
                "animal_id": 8101,
                "status": "checked_in",
                "service_type": "boarding",
                "start_at": "2026-06-29T13:00:00Z",
                "end_at": "2026-07-02T15:00:00Z",
                "why_received": "manager daily report needs today's in-house boarding demand and source lineage"
            }),
        ),
        SourcePayload::new(
            SourcePayloadKind::CareNote,
            "gingr::response::provider::Payload",
            "domain::care::CareNoteFact",
            "fixture://mock-gingr/care-notes/9001001-feeding.json",
            json!({
                "synthetic": true,
                "reservation_id": 9001001,
                "animal_id": 8101,
                "note_type": "feeding",
                "body": "Ate breakfast; monitor dinner appetite.",
                "recorded_at": "2026-06-29T14:30:00Z",
                "visibility": "internal_only",
                "why_received": "manager report highlights care exceptions without customer sends"
            }),
        ),
        SourcePayload::new(
            SourcePayloadKind::Vaccine,
            "gingr::response::provider::Payload",
            "domain::vaccine::ReviewFact",
            "fixture://mock-gingr/vaccines/8101-rabies.json",
            json!({
                "synthetic": true,
                "animal_id": 8101,
                "vaccine_name": "rabies",
                "expires_on": "2026-07-05",
                "verification_status": "needs_staff_review",
                "why_received": "manager report surfaces near-expiry vaccine work as a review gate, not an automated medical decision"
            }),
        ),
    ];

    TraceEnvelope {
        correlation_id: CorrelationId::new("info-lifespan-demo-2026-06-29"),
        source_system: SourceSystem::MockGingrReadOnlyFixture,
        synthetic_data_only: true,
        provider_payloads_are_source_evidence_only: true,
        live_side_effects_allowed: false,
        source_payloads,
        stages: vec![
            TraceStage::new(
                StageKind::SourceEvidenceReceived,
                "Mock Gingr event received",
                false,
                None,
                vec![ModelPath::new(
                    "app::information_lifespan::SourcePayload",
                    "synthetic read-only source evidence",
                )],
            ),
            TraceStage::new(
                StageKind::ProviderDtoPreserved,
                "Provider DTO/source model preserved",
                false,
                None,
                vec![
                    ModelPath::new(
                        "gingr::response::ReservationRecord",
                        "provider reservation evidence",
                    ),
                    ModelPath::new(
                        "gingr::response::provider::Payload",
                        "provider-shaped care/vaccine evidence",
                    ),
                ],
            ),
            TraceStage::new(
                StageKind::NormalizedNvaModels,
                "NVA-owned models normalized",
                false,
                None,
                vec![
                    ModelPath::new("domain::source::RecordRef", "source lineage pointer"),
                    ModelPath::new(
                        "app::manager_daily_brief::SourceFact",
                        "reviewable NVA source fact",
                    ),
                    ModelPath::new(
                        "app::manager_daily_brief::Packet",
                        "manager-owned workflow packet",
                    ),
                ],
            ),
            TraceStage::new(
                StageKind::DatabaseProjectionProof,
                "Database rows/projections prove lineage",
                false,
                None,
                vec![
                    ModelPath::new(
                        "migrations::source_import_runs/source_quality_issues/workflow_events/review_packets/approval_records/audit_events/manager_daily_brief_outcomes",
                        "local Postgres seed rows linked by correlation id",
                    ),
                    ModelPath::new(
                        "migrations::information_lifespan_db_lifecycle_proof",
                        "queryable lifecycle projection for the demo trace",
                    ),
                ],
            ),
            TraceStage::new(
                StageKind::HermesProcessorRun,
                "Hermes processor container enriches trace",
                false,
                None,
                vec![
                    ModelPath::new(
                        "apps::hermes_processor::processor",
                        "Docker Compose service that consumes the synthetic trace and emits report-ready JSON",
                    ),
                    ModelPath::new(
                        "schemas::information_lifespan_hermes_processor_output",
                        "validated processor output contract for API/UI handoff",
                    ),
                ],
            ),
            TraceStage::new(
                StageKind::CalculationApplied,
                "Calculations and ranking applied",
                false,
                None,
                vec![ModelPath::new(
                    "app::manager_daily_brief::LaborImpactEstimate",
                    "labor-value calculation",
                )],
            ),
            TraceStage::new(
                StageKind::ReviewGateLocked,
                "Unsafe side effects locked behind review",
                false,
                None,
                vec![ModelPath::new(
                    "app::manager_daily_brief::BlockedAction",
                    "explicit side-effect lock enum",
                )],
            ),
            TraceStage::new(
                StageKind::ManagerDailyReportArtifact,
                "Manager Daily Report artifact produced",
                false,
                None,
                vec![
                    ModelPath::new(
                        "app::information_lifespan::FinalArtifact",
                        "report artifact contract emitted by the Piece 4 API response",
                    ),
                    ModelPath::new(
                        "apps::api::http::information_lifespan_run_payload",
                        "local API renderer for the final Manager Daily Report artifact",
                    ),
                ],
            ),
        ],
        log_proof_entries: vec![
            LogProofEntry::new(
                "INFO",
                "information_lifespan",
                "mock Gingr source evidence accepted from fixture",
            ),
            LogProofEntry::new(
                "INFO",
                "information_lifespan",
                "provider payload retained as source evidence only",
            ),
            LogProofEntry::new(
                "WARN",
                "information_lifespan.safety",
                "provider writes/customer sends/payment movement locked",
            ),
        ],
        db_proof_entries: vec![
            DbProofEntry::new(
                "source_import_runs",
                "source_import_run:info-lifespan-demo-2026-06-29;correlation_id:info-lifespan-demo-2026-06-29",
                "migrations::source_import_runs",
            ),
            DbProofEntry::new(
                "source_quality_issues",
                "source_quality_issue:vaccine-near-expiry:8101;correlation_id:info-lifespan-demo-2026-06-29",
                "migrations::source_quality_issues",
            ),
            DbProofEntry::new(
                "workflow_events",
                "workflow_event:manager-daily-report:2026-06-29;correlation_id:info-lifespan-demo-2026-06-29",
                "app::manager_daily_brief::Request",
            ),
            DbProofEntry::new(
                "review_packets",
                "review_packet:vaccine-near-expiry:8101;correlation_id:info-lifespan-demo-2026-06-29",
                "app::manager_daily_brief::BriefAction",
            ),
            DbProofEntry::new(
                "manager_daily_brief_outcomes",
                "manager_daily_brief_outcome:synthetic-2026-06-29;correlation_id:info-lifespan-demo-2026-06-29",
                "storage::operations::ManagerDailyBriefOutcomeRecord",
            ),
            DbProofEntry::new(
                "information_lifespan_db_lifecycle_proof",
                "correlation_id:info-lifespan-demo-2026-06-29",
                "migrations::information_lifespan_db_lifecycle_proof",
            ),
        ],
        network_proof_entries: vec![
            NetworkProofEntry::new(
                HttpMethod::Post,
                "/demo/information-lifespan/run",
                200,
                "response://information-lifespan/info-lifespan-demo-2026-06-29",
            ),
            NetworkProofEntry::new(
                HttpMethod::Get,
                "/demo/information-lifespan/info-lifespan-demo-2026-06-29/report",
                200,
                "response://manager-daily-report/synthetic-2026-06-29",
            ),
        ],
        calculations: vec![
            CalculationProof::new("source_snapshots", "reservation + care_note + vaccine", "3"),
            CalculationProof::new(
                "normalized_facts",
                "reservation demand + care exception + vaccine review",
                "3",
            ),
            CalculationProof::new(
                "estimated_labor_minutes_saved",
                "60 minute manual morning scan - 18 minute reviewed packet",
                "42",
            ),
        ],
        safety_gates: vec![
            SafetyGateProof::new(
                SafetyGateKind::ProviderWriteLocked,
                true,
                "demo is read-only and never mutates Gingr/PMS records",
            ),
            SafetyGateProof::new(
                SafetyGateKind::CustomerSendLocked,
                true,
                "manager report can draft internal tasks only; no live sends",
            ),
            SafetyGateProof::new(
                SafetyGateKind::MedicalReviewRequired,
                true,
                "vaccine fact is surfaced for staff review, not auto-accepted",
            ),
            SafetyGateProof::new(
                SafetyGateKind::ScheduleChangeLocked,
                true,
                "staffing/demand recommendation cannot change schedules",
            ),
            SafetyGateProof::new(
                SafetyGateKind::PaymentMovementLocked,
                true,
                "payments, refunds, and discounts are outside this demo authority",
            ),
        ],
        final_artifact: FinalArtifact::new(
            FinalArtifactKind::ManagerDailyReport,
            "Manager Daily Report — synthetic 2026-06-29",
            "artifact://manager-daily-report/synthetic-2026-06-29",
            "3 source snapshots, 3 normalized facts, 6 DB proof refs, 5 review locks, 42 estimated labor minutes saved",
        ),
    }
}
