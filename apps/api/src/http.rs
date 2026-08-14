//! Staff-facing API/runtime DTO contracts.
//!
//! Types in this module are product-owned HTTP payloads for the local runtime shell.
//! They intentionally sit on our side of the boundary: provider DTOs may contribute
//! source evidence and record references, but provider response shapes are not passed
//! through as API contracts. Every exposed workflow DTO should preserve review gates,
//! audit/correlation evidence, labor/outcome fields, and disabled live-side-effect
//! status until a future approved adapter crosses the customer/provider boundary.

use crate::{
    authentication,
    error::{AuthenticationFailure, AuthorizationFailure, ErrorContext, ErrorKind, PublicApiError},
    public_contract,
};
use app::workflow_repository::OutcomeRepository as _;
use app::workflow_repository::Repository as _;
use app::workflow_repository::source_quality_backlog::Repository as _;
use app::{
    checkout_completion, crm_retention, data_quality_hygiene, information_lifespan,
    manager_daily_brief, site_finance, workflow_repository,
};
use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{Extension, FromRequestParts, MatchedPath, Path, Query, State},
    http::{HeaderName, HeaderValue, Request, Response, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use domain::{
    access, agent, analytics, data_quality, entities, message, operations, policy, source,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, sync::Arc, time::Duration};
use tokio::sync::Mutex;

use tower_http::trace::TraceLayer;
use tracing::{Level, Span};
use uuid::Uuid;

static VACCINE_DOCUMENT_STATE: std::sync::OnceLock<VaccineDocumentState> =
    std::sync::OnceLock::new();

#[derive(Clone)]
/// In-memory state kept on the API shell for deterministic workflow demos and tests.
///
/// The state stores documents, review packets, inquiry intake records, and labor-evidence
/// projections so HTTP handlers can demonstrate runtime rules without connecting to
/// live databases, customer messaging, or provider write APIs.
pub struct VaccineDocumentState {
    store: Arc<Mutex<VaccineDocumentStore>>,
}

impl Default for VaccineDocumentState {
    fn default() -> Self {
        Self {
            store: Arc::new(Mutex::new(VaccineDocumentStore::default())),
        }
    }
}

#[derive(Debug, Default)]
struct VaccineDocumentStore {
    documents: BTreeMap<Uuid, DocumentRecord>,
    extractions: BTreeMap<Uuid, VaccineExtractionRecord>,
    vaccine_records: BTreeMap<Uuid, VaccineRecord>,
    review_packets: BTreeMap<Uuid, ReviewPacket>,
    approvals: BTreeMap<Uuid, ApprovalRecord>,
    eligibility: BTreeMap<Uuid, PetEligibility>,
    manager_daily_brief_outcomes: storage::workflow_repository::InMemoryOutcomes<
        storage::operations::ManagerDailyBriefOutcomeRecord,
    >,
    data_quality_hygiene_outcomes: storage::workflow_repository::InMemoryOutcomes<
        storage::operations::DataQualityHygieneOutcomeRecord,
    >,
    data_quality_hygiene_persistence_records:
        Vec<storage::operations::DataQualityHygieneLocalPersistenceRecords>,
    inquiry_intake_records: Vec<InquiryIntakeRecord>,
    audit_events: Vec<AuditEvent>,
}

impl workflow_repository::Repository for VaccineDocumentStore {
    fn runtime_counters(&self) -> workflow_repository::RuntimeCounters {
        workflow_repository::RuntimeCounters {
            inquiry_count: self.inquiry_intake_records.len(),
            review_packet_count: self.review_packets.len(),
            audit_event_count: self.audit_events.len(),
            outcome_count: self.manager_daily_brief_outcomes.outcomes().len()
                + self.data_quality_hygiene_outcomes.outcomes().len(),
            internal_outbox_candidate_count: self
                .data_quality_hygiene_persistence_records
                .iter()
                .filter(|records| records.outbox_candidate.is_some())
                .count(),
            review_gated_internal_outbox_count: self
                .data_quality_hygiene_persistence_records
                .iter()
                .filter(|records| {
                    records.outbox_candidate.as_ref().is_some_and(|candidate| {
                        candidate.status() == storage::operations::OutboxStatusCode::Pending
                            && candidate
                                .payload()
                                .get("live_delivery_allowed")
                                .and_then(Value::as_bool)
                                == Some(false)
                    })
                })
                .count(),
        }
    }
}

#[derive(Debug, Serialize)]
struct HealthPayload {
    api_contract: public_contract::ApiContractMetadata,
    service: &'static str,
    status: &'static str,
    live_side_effects: &'static str,
}

#[derive(Debug, Serialize)]
struct ReadinessPayload {
    api_contract: public_contract::ApiContractMetadata,
    service: &'static str,
    database: &'static str,
    object_storage: &'static str,
    agent_runtime: &'static str,
    workflow_repository: WorkflowRepositoryReadinessPayload,
    observability: ObservabilityReadinessPayload,
    live_customer_messaging: &'static str,
    live_provider_writes: &'static str,
}

#[derive(Debug, Serialize)]
struct ObservabilityReadinessPayload {
    request_correlation: &'static str,
    workflow_correlation: &'static str,
    local_request_metrics: &'static str,
    metrics_scope: &'static str,
    production_gap: &'static str,
}

#[derive(Debug, Serialize)]
struct WorkflowRepositoryReadinessPayload {
    active_adapter: &'static str,
    postgres_adapter: &'static str,
    contract: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct ReadModelDescriptorPayload {
    name: &'static str,
    source: &'static str,
    projection_version: &'static str,
}

#[derive(Debug, Serialize)]
struct ReadModelDataPosturePayload {
    safe_synthetic_data: bool,
    live_side_effects_allowed: bool,
    provider_payload_passthrough: bool,
    provider_writes_allowed: bool,
    customer_messages_allowed: bool,
}

#[derive(Debug, Serialize)]
struct ReadModelDatabasePayload {
    status: &'static str,
    adapter: &'static str,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct SourceQualityBacklogPayload {
    api_contract: public_contract::ApiContractMetadata,
    read_model: ReadModelDescriptorPayload,
    data_posture: ReadModelDataPosturePayload,
    database: ReadModelDatabasePayload,
    records: Vec<workflow_repository::source_quality_backlog::Item>,
}

fn workflow_repository_readiness_payload() -> WorkflowRepositoryReadinessPayload {
    WorkflowRepositoryReadinessPayload {
        active_adapter: "in_memory",
        postgres_adapter: if database_url_configured() {
            "env_configured_not_verified"
        } else {
            "planned_same_contract"
        },
        contract: vec![
            "workflow_events",
            "review_packets",
            "audit_events",
            "outcomes",
            "documents",
        ],
    }
}

fn database_readiness_status() -> &'static str {
    if database_url_configured() {
        "env_configured_not_verified"
    } else {
        "not_configured"
    }
}

fn object_storage_readiness_status() -> &'static str {
    if minio_env_configured() {
        "env_configured_not_verified"
    } else {
        "not_configured"
    }
}

#[derive(Debug, Serialize)]
struct OpsMetricsSummaryPayload {
    api_contract: public_contract::ApiContractMetadata,
    api_request_metrics: ApiRequestMetricsPayload,
    product_labor_metrics: ProductLaborMetricsPayload,
    local_runtime_counters: LocalRuntimeCountersPayload,
    safety: MetricsSafetyPayload,
    observability_gap: ObservabilityGapPayload,
    production_metrics_plan: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct ApiRequestMetricsPayload {
    scope: &'static str,
    request_id_source: &'static str,
    correlation_id_source: &'static str,
    payload_logging: &'static str,
    safe_error_classes: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct ProductLaborMetricsPayload {
    manager_daily_brief: LaborOutcomeRollupPayload,
    data_quality_hygiene: LaborOutcomeRollupPayload,
}

#[derive(Debug, Serialize)]
struct LaborOutcomeRollupPayload {
    metric_source: &'static str,
    outcome_count: usize,
    completed_count: usize,
    total_estimated_minutes_saved: u16,
    completed_actual_minutes_saved: u16,
}

#[derive(Debug, Serialize)]
struct LocalRuntimeCountersPayload {
    inquiry_count: usize,
    review_packet_count: usize,
    audit_event_count: usize,
    outcome_count: usize,
    data_quality_hygiene_outbox_candidate_count: usize,
    data_quality_hygiene_review_gated_outbox_count: usize,
    production_queue_adapter: &'static str,
}

#[derive(Debug, Serialize)]
struct MetricsSafetyPayload {
    granularity: &'static str,
    contains_customer_pii: bool,
    contains_provider_payloads: bool,
    live_side_effects: &'static str,
}

#[derive(Debug, Serialize)]
struct ObservabilityGapPayload {
    production_traces: &'static str,
    durable_request_metrics: &'static str,
    dashboard_or_alerting: &'static str,
}

fn api_dto_contract(workflow: &'static str) -> public_contract::ApiContractMetadata {
    public_contract::ApiContractMetadata::operations_v0(workflow)
}

fn api_dto_contract_payload(workflow: &'static str) -> Value {
    json!(api_dto_contract(workflow))
}

#[derive(Debug, Deserialize)]
struct VaccineDocumentUploadRequest {
    pet_id: Uuid,
    customer_id: Uuid,
    filename: String,
    mime_type: String,
    content: String,
    uploaded_by_staff_id: String,
}

#[derive(Debug, Deserialize)]
struct VaccineReviewDecisionRequest {
    reviewed_by_staff_id: String,
    reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VaccineReviewDecision {
    Approve,
    Reject,
}

impl VaccineReviewDecision {
    fn status_code(self) -> &'static str {
        match self {
            Self::Approve => "approved",
            Self::Reject => "rejected",
        }
    }

    fn document_verification_status(self) -> &'static str {
        match self {
            Self::Approve => "verified",
            Self::Reject => "rejected",
        }
    }

    fn vaccine_record_status(self) -> &'static str {
        match self {
            Self::Approve => "verified_current",
            Self::Reject => "rejected",
        }
    }

    fn eligibility(self, pet_id: Uuid, vaccine_record_id: Uuid) -> PetEligibility {
        PetEligibility {
            pet_id,
            rabies_current: matches!(self, Self::Approve),
            source_vaccine_record_id: Some(vaccine_record_id),
            status: match self {
                Self::Approve => "eligible_from_approved_vaccine_document",
                Self::Reject => "ineligible_after_rejected_vaccine_document",
            },
        }
    }

    fn from_decided_status(status: &str) -> Option<Self> {
        match status {
            "approved" => Some(Self::Approve),
            "rejected" => Some(Self::Reject),
            _ => None,
        }
    }
}

#[derive(Clone)]
struct VaccineReviewDecisionEvidence {
    reviewed_by_staff_id: String,
    decided_at: String,
    reason: Option<String>,
}

#[derive(Debug)]
enum VaccineReviewDecisionRejection {
    PacketNotFound {
        review_packet_id: Uuid,
    },
    PacketAlreadyDecided {
        review_packet_id: Uuid,
        existing: VaccineReviewDecision,
        attempted: VaccineReviewDecision,
    },
    BrokenWorkflowState {
        review_packet_id: Uuid,
        code: &'static str,
    },
}

fn authorization_error_payload(
    rejection: authentication::Rejection,
    workflow: &'static str,
    persisted_field: &'static str,
) -> Value {
    let mut payload = json!({
        "api_contract": api_dto_contract_payload(workflow),
        "accepted": false,
    });
    payload[persisted_field] = Value::Bool(false);
    merge_error_envelope(
        payload,
        PublicApiError::new(rejection.into(), ErrorContext::new("missing_request_id")),
    )
}

impl authentication::Rejection {
    fn status_code(self) -> StatusCode {
        PublicApiError::new(self.into(), ErrorContext::new("missing_request_id")).status_code()
    }
}

struct Authenticated(authentication::Context);

impl<S> FromRequestParts<S> for Authenticated
where
    S: Send + Sync,
{
    type Rejection = axum::response::Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let context = parts
            .extensions
            .get::<authentication::Context>()
            .cloned()
            .unwrap_or_else(authentication::Context::missing);
        authentication::authenticate(&context).map_err(|rejection| {
            (
                rejection.status_code(),
                Json(authorization_error_payload(
                    rejection,
                    "authenticated_request",
                    "outcome_persisted",
                )),
            )
                .into_response()
        })?;
        Ok(Self(context))
    }
}

#[derive(Debug, Deserialize)]
struct InquirySubmissionRequest {
    source_event_key: String,
    source_system: Option<String>,
    provider_model_path: Option<String>,
    raw_payload_ref: Option<String>,
    received_at: Option<DateTime<Utc>>,
    location_id: String,
    customer: InquiryCustomerRequest,
    pet: InquiryPetRequest,
    service: String,
    requested_dates: Option<InquiryDateWindowRequest>,
    message: String,
    #[serde(default)]
    contact_attempts: Vec<InquiryContactAttemptRequest>,
    simulated_conversion: Option<InquirySimulatedConversionRequest>,
}

#[derive(Debug, Deserialize)]
struct InquiryCustomerRequest {
    full_name: String,
    email: Option<String>,
    phone: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InquiryPetRequest {
    name: String,
    species: String,
}

#[derive(Debug, Deserialize)]
struct InquiryDateWindowRequest {
    start: String,
    end: String,
}

#[derive(Debug, Deserialize)]
struct InquiryContactAttemptRequest {
    attempted_at: DateTime<Utc>,
    channel: String,
    purpose: String,
    outcome: String,
    message_ref: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InquirySimulatedConversionRequest {
    reservation_id: String,
    converted_at: DateTime<Utc>,
    attribution_source: String,
}

#[derive(Debug, Clone, Serialize)]
struct InquiryIntakeRecord {
    api_contract: public_contract::ApiContractMetadata,
    event: InquiryEvent,
    #[serde(skip_serializing_if = "Option::is_none")]
    provenance: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data_quality: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    canonical_lead_event: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workflow: Option<Value>,
    lead: ParsedInquiryLead,
    draft_reply: InquiryDraftReply,
    #[serde(skip_serializing_if = "Option::is_none")]
    review_packet: Option<Value>,
    task: InquiryTask,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage_projection: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_response: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    simulated_conversion: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome_attribution: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replay: Option<Value>,
    agent_runtime: &'static str,
    policy_boundary: &'static str,
    audit_events: Vec<InquiryAuditEvent>,
}

#[derive(Debug, Clone, Serialize)]
struct InquiryEvent {
    event_type: &'static str,
    source_event_key: String,
    location_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ParsedInquiryLead {
    customer_name: String,
    customer_email: Option<String>,
    customer_phone: Option<String>,
    pet_name: String,
    species: String,
    service: String,
    requested_dates: Option<ParsedInquiryDateWindow>,
    original_message: String,
    missing_info: Vec<&'static str>,
    review_status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct ParsedInquiryDateWindow {
    start: String,
    end: String,
}

#[derive(Debug, Clone, Serialize)]
struct InquiryDraftReply {
    status: &'static str,
    live_send_allowed: bool,
    approval_gate: &'static str,
    body: String,
}

#[derive(Debug, Clone, Serialize)]
struct InquiryTask {
    kind: &'static str,
    status: &'static str,
    title: String,
    review_gate: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct InquiryAuditEvent {
    action: &'static str,
    actor_kind: &'static str,
    subject_key: String,
}

#[derive(Debug, Serialize)]
struct InquiryStaffQueuePayload {
    api_contract: public_contract::ApiContractMetadata,
    records: Vec<InquiryIntakeRecord>,
}

#[derive(Debug, Clone, Serialize)]
struct VaccineDocumentWorkflowPayload {
    api_contract: public_contract::ApiContractMetadata,
    document: DocumentRecord,
    extraction: VaccineExtractionRecord,
    vaccine_record: VaccineRecord,
    review_packet: ReviewPacket,
    approval: Option<ApprovalRecord>,
    eligibility: PetEligibility,
    audit_events: Vec<AuditEvent>,
}

#[derive(Debug, Clone, Serialize)]
struct DocumentRecord {
    id: Uuid,
    pet_id: Uuid,
    customer_id: Uuid,
    classification: &'static str,
    source: &'static str,
    filename: String,
    mime_type: String,
    content_length_bytes: usize,
    sha256: String,
    storage_bucket: &'static str,
    storage_key: String,
    storage_version: String,
    virus_scan_status: &'static str,
    pii_redaction_status: &'static str,
    verification_status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct VaccineExtractionRecord {
    id: Uuid,
    document_id: Uuid,
    schema_version: &'static str,
    vaccine_name: String,
    effective_on: NaiveDate,
    expires_on: Option<NaiveDate>,
    confidence: f32,
    uncertainty_policy: &'static str,
    auto_accept_threshold: f32,
    raw_text_ref: String,
}

#[derive(Debug, Clone, Serialize)]
struct VaccineRecord {
    id: Uuid,
    pet_id: Uuid,
    source_document_id: Uuid,
    vaccine_name: String,
    status: &'static str,
    effective_on: NaiveDate,
    expires_on: Option<NaiveDate>,
    review_gate: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct ReviewPacket {
    id: Uuid,
    document_id: Uuid,
    vaccine_record_id: Uuid,
    gate: &'static str,
    status: &'static str,
    uncertainty: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct ApprovalRecord {
    id: Uuid,
    review_packet_id: Uuid,
    target_document_id: Uuid,
    target_vaccine_record_id: Uuid,
    gate: &'static str,
    status: &'static str,
    decided_by_staff_id: String,
    decided_at: String,
    reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct PetEligibility {
    pet_id: Uuid,
    rabies_current: bool,
    source_vaccine_record_id: Option<Uuid>,
    status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct AuditEvent {
    action: &'static str,
    actor_kind: &'static str,
    actor_id: String,
    subject_kind: &'static str,
    subject_id: Uuid,
    metadata: BTreeMap<&'static str, String>,
}

#[derive(Clone, Debug)]
struct RequestTraceEvidence {
    request_id: String,
    request_correlation_id: String,
}

impl RequestTraceEvidence {
    fn from_request_headers(headers: &axum::http::HeaderMap) -> Self {
        let request_id = headers
            .get(request_id_header())
            .and_then(|value| value.to_str().ok())
            .filter(|value| safe_request_id(value))
            .map(str::to_owned)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let request_correlation_id = headers
            .get(correlation_id_header())
            .and_then(|value| value.to_str().ok())
            .filter(|value| safe_request_id(value))
            .map(str::to_owned)
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        Self {
            request_id,
            request_correlation_id,
        }
    }

    fn request_id(&self) -> &str {
        &self.request_id
    }

    fn request_correlation_id(&self) -> &str {
        &self.request_correlation_id
    }
}

fn request_id_header() -> HeaderName {
    HeaderName::from_static("x-request-id")
}

fn correlation_id_header() -> HeaderName {
    HeaderName::from_static("x-correlation-id")
}

fn safe_request_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn workflow_observability_payload(
    correlation_id: &str,
    request_trace: &RequestTraceEvidence,
) -> public_contract::WorkflowObservability {
    public_contract::WorkflowObservability {
        correlation_id: correlation_id.to_owned(),
        request_id: request_trace.request_id().to_owned(),
        request_correlation_id: request_trace.request_correlation_id().to_owned(),
        route_status_trace: "enabled".to_owned(),
        safe_error_class: "not_applicable".to_owned(),
        payload_logging: "disabled".to_owned(),
        sensitive_payload_logging: None,
    }
}

async fn attach_request_trace(mut request: Request<Body>, next: Next) -> Response<Body> {
    let request_trace = RequestTraceEvidence::from_request_headers(request.headers());
    request.extensions_mut().insert(request_trace.clone());

    let mut response = next.run(request).await;
    let request_id = HeaderValue::from_str(request_trace.request_id())
        .expect("generated or validated request id is a header value");
    response
        .headers_mut()
        .insert(request_id_header(), request_id);
    let correlation_id = HeaderValue::from_str(request_trace.request_correlation_id())
        .expect("generated or validated correlation id is a header value");
    response
        .headers_mut()
        .insert(correlation_id_header(), correlation_id);
    response
}

/// Builds the default staff-facing workflow router with deterministic in-memory state.
///
/// Exposed routes are safe runtime surfaces: health/readiness probes, staff inquiry
/// intake, vaccine document review, manager daily-brief packet/draft/outcome paths,
/// and data-quality hygiene packet/draft/outcome paths. They return DTO evidence and
/// review gates rather than performing live customer sends or provider writes.
pub fn router() -> Router {
    router_with_state(
        VACCINE_DOCUMENT_STATE
            .get_or_init(VaccineDocumentState::default)
            .clone(),
    )
}

/// Builds the staff-facing workflow router over caller-provided deterministic state.
///
/// Tests and local runtimes use this entrypoint to share state across requests while
/// preserving the same safety rules as [`router`]: handlers may draft, audit, and
/// persist projections, but live side effects remain blocked unless a future runtime
/// adapter adds explicit approval and provider gates.
pub fn router_with_state(state: VaccineDocumentState) -> Router {
    router_with_authentication(state, AuthenticationSource::Production)
}

/// Builds a deterministic test harness router whose synthetic actor headers are
/// promoted only when callers explicitly select this entrypoint.
///
/// Production entrypoints never call this helper; keeping it available in normal
/// library builds lets black-box integration tests exercise authentication without
/// Cargo feature unification changing whether the test suite compiles.
#[doc(hidden)]
#[cfg(debug_assertions)]
pub fn router_with_test_auth_state(state: VaccineDocumentState) -> Router {
    router_with_authentication(state, AuthenticationSource::SyntheticTestHeaders)
}

#[derive(Clone, Copy)]
enum AuthenticationSource {
    Production,
    #[cfg(debug_assertions)]
    SyntheticTestHeaders,
}

fn router_with_authentication(
    state: VaccineDocumentState,
    authentication_source: AuthenticationSource,
) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v0/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v0/readyz", get(readyz))
        .route("/ops/metrics/summary", get(ops_metrics_summary))
        .route("/v0/ops/metrics/summary", get(ops_metrics_summary))
        .route("/inquiries", post(submit_inquiry))
        .route("/staff/inquiries", get(staff_inquiries))
        .route(
            "/agent/context/manager-daily-brief",
            get(manager_daily_brief_agent_context),
        )
        .route(
            "/v0/agent/context/manager-daily-brief",
            get(manager_daily_brief_agent_context),
        )
        .route(
            "/agent/drafts/manager-daily-brief",
            post(submit_manager_daily_brief_agent_draft),
        )
        .route(
            "/manager-daily-brief/actions/{action_id}/outcome",
            post(capture_manager_daily_brief_action_outcome),
        )
        .route(
            "/agent/context/data-quality-hygiene",
            get(data_quality_hygiene_agent_context),
        )
        .route(
            "/agent/context/permissioned-knowledge",
            get(permissioned_knowledge_agent_context),
        )
        .route(
            "/agent/context/site-finance",
            get(site_finance_agent_context),
        )
        .route(
            "/v0/agent/context/permissioned-knowledge",
            get(permissioned_knowledge_agent_context),
        )
        .route(
            "/v0/agent/context/site-finance",
            get(site_finance_agent_context),
        )
        .route(
            "/v0/agent/context/data-quality-hygiene",
            get(data_quality_hygiene_agent_context),
        )
        .route(
            "/agent/drafts/data-quality-hygiene",
            post(submit_data_quality_hygiene_agent_draft),
        )
        .route(
            "/v0/agent/drafts/data-quality-hygiene",
            post(submit_data_quality_hygiene_agent_draft),
        )
        .route(
            "/data-quality-hygiene/actions/{action_id}/outcome",
            post(capture_data_quality_hygiene_action_outcome),
        )
        .route(
            "/v0/data-quality-hygiene/actions/{action_id}/outcome",
            post(capture_data_quality_hygiene_action_outcome),
        )
        .route(
            "/data-quality-hygiene/outcomes/summary",
            get(data_quality_hygiene_outcome_summary),
        )
        .route(
            "/v0/data-quality-hygiene/outcomes/summary",
            get(data_quality_hygiene_outcome_summary),
        )
        .route(
            "/v0/read-models/source-quality-backlog",
            get(source_quality_backlog),
        )
        .route(
            "/demo/information-lifespan/run",
            post(run_information_lifespan_demo),
        )
        .route(
            "/v0/demo/information-lifespan/run",
            post(run_information_lifespan_demo),
        )
        .route(
            "/demo/information-lifespan/{correlation_id}/report",
            get(replay_information_lifespan_report),
        )
        .route(
            "/v0/demo/information-lifespan/{correlation_id}/report",
            get(replay_information_lifespan_report),
        )
        .route("/vaccine-documents/uploads", post(upload_vaccine_document))
        .route(
            "/vaccine-documents/review-packets/{review_packet_id}/approve",
            post(approve_vaccine_document),
        )
        .route(
            "/vaccine-documents/review-packets/{review_packet_id}/reject",
            post(reject_vaccine_document),
        )
        .fallback(owned_operations_not_found)
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let route = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str)
                        .unwrap_or("unknown");
                    let request_trace = request.extensions().get::<RequestTraceEvidence>();
                    let request_id = request_trace
                        .map(RequestTraceEvidence::request_id)
                        .unwrap_or("missing");
                    let correlation_id = request_trace
                        .map(RequestTraceEvidence::request_correlation_id)
                        .unwrap_or("missing");

                    tracing::info_span!(
                        "api_request",
                        http.method = %request.method(),
                        http.route = %route,
                        http.request_id = %request_id,
                        http.correlation_id = %correlation_id,
                        status_code = tracing::field::Empty,
                        duration_ms = tracing::field::Empty,
                        safe_error_class = tracing::field::Empty,
                        actor_source = "payload_or_unset",
                        location_id = tracing::field::Empty,
                        tenant_id = tracing::field::Empty,
                        payload_logging = "disabled"
                    )
                })
                .on_response(record_api_response_metrics),
        )
        .layer(middleware::from_fn(normalize_public_api_errors))
        .layer(middleware::from_fn_with_state(
            authentication_source,
            attach_authenticated_actor,
        ))
        .layer(middleware::from_fn(attach_request_trace))
}

async fn attach_authenticated_actor(
    State(source): State<AuthenticationSource>,
    mut request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let context = match source {
        AuthenticationSource::Production => authentication::Context::missing(),
        #[cfg(debug_assertions)]
        AuthenticationSource::SyntheticTestHeaders => {
            authentication::Context::from_test_headers(request.headers())
        }
    };
    request.extensions_mut().insert(context);
    next.run(request).await
}

async fn normalize_public_api_errors(request: Request<Body>, next: Next) -> Response<Body> {
    let context = request
        .extensions()
        .get::<RequestTraceEvidence>()
        .map(|trace| {
            ErrorContext::new(trace.request_id())
                .with_correlation_id(trace.request_correlation_id())
        })
        .unwrap_or_else(|| ErrorContext::new("missing_request_id"));
    let response = next.run(request).await;
    let status = response.status();
    if !status.is_client_error() && !status.is_server_error() {
        return response;
    }

    let (parts, body) = response.into_parts();
    let bytes = match to_bytes(body, 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return PublicApiError::new(ErrorKind::Internal, context).into_response();
        }
    };
    let payload = serde_json::from_slice::<Value>(&bytes).unwrap_or(Value::Null);
    if payload["error"]["safe_error_class"].is_string()
        && payload["request_id"].is_string()
        && payload["live_side_effects"].is_string()
    {
        return Response::from_parts(parts, Body::from(bytes));
    }

    let mut error = PublicApiError::new(error_kind_for_response(status, &payload), context);
    if let Some(public_code) = payload["error"]["code"].as_str() {
        error = error.with_public_code(public_code);
    }
    let merged = merge_error_envelope(payload, error);
    (status, Json(merged)).into_response()
}

fn merge_error_envelope(mut payload: Value, error: PublicApiError) -> Value {
    let envelope =
        serde_json::to_value(error.envelope()).expect("public error envelope always serializes");
    let Value::Object(envelope_fields) = envelope else {
        unreachable!("public error envelope serializes as an object")
    };
    let Value::Object(payload_fields) = &mut payload else {
        return Value::Object(envelope_fields);
    };
    payload_fields.extend(envelope_fields);
    payload
}

fn error_kind_for_response(status: StatusCode, payload: &Value) -> ErrorKind {
    match status {
        StatusCode::UNAUTHORIZED => {
            ErrorKind::Authentication(AuthenticationFailure::MissingTrustedActorContext)
        }
        StatusCode::FORBIDDEN => {
            ErrorKind::Authorization(AuthorizationFailure::ActorRoleNotAuthorized)
        }
        StatusCode::CONFLICT
            if payload["classification"] == "idempotency_payload_drift"
                || payload["error"]["code"] == "idempotency_payload_drift" =>
        {
            ErrorKind::IdempotencyConflict
        }
        StatusCode::CONFLICT => ErrorKind::ReviewConflict,
        StatusCode::UNPROCESSABLE_ENTITY if response_reports_source_ambiguity(payload) => {
            ErrorKind::SourceAmbiguity
        }
        StatusCode::UNPROCESSABLE_ENTITY | StatusCode::BAD_REQUEST => ErrorKind::Validation {
            details: validation_details(payload),
        },
        StatusCode::NOT_FOUND => ErrorKind::NotFound {
            details: Vec::new(),
        },
        StatusCode::SERVICE_UNAVAILABLE | StatusCode::BAD_GATEWAY | StatusCode::GATEWAY_TIMEOUT => {
            ErrorKind::Unavailable
        }
        _ => ErrorKind::Internal,
    }
}

fn response_reports_source_ambiguity(payload: &Value) -> bool {
    payload
        .get("reasons")
        .and_then(Value::as_array)
        .is_some_and(|reasons| {
            reasons.iter().any(|reason| {
                reason
                    .as_str()
                    .is_some_and(|reason| reason.contains("source") || reason.contains("issue_ref"))
            })
        })
}

fn validation_details(payload: &Value) -> Vec<public_contract::ErrorDetail> {
    payload
        .get("reasons")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|reason| public_contract::ErrorDetail::field("request".to_owned(), reason.to_owned()))
        .collect()
}

fn record_api_response_metrics(response: &Response<Body>, latency: Duration, span: &Span) {
    let status_code = response.status().as_u16();
    let duration_ms = latency.as_millis() as u64;
    let safe_error_class = safe_error_class_for_status(response.status());

    span.record("status_code", status_code);
    span.record("duration_ms", duration_ms);
    span.record("safe_error_class", safe_error_class);

    tracing::event!(
        parent: span,
        Level::INFO,
        status_code,
        duration_ms,
        safe_error_class,
        payload_logging = "disabled",
        "api_request completed"
    );
}

fn safe_error_class_for_status(status: StatusCode) -> &'static str {
    match status {
        StatusCode::NOT_FOUND => "not_found",
        status if status.is_client_error() => "validation_failed",
        status if status.is_server_error() => "internal_error",
        _ => "not_applicable",
    }
}

async fn owned_operations_not_found(request: Request<Body>) -> axum::response::Response {
    let request_id = request
        .extensions()
        .get::<RequestTraceEvidence>()
        .map(RequestTraceEvidence::request_id)
        .unwrap_or("missing_request_id")
        .to_owned();
    let path = request.uri().path().to_owned();

    PublicApiError::new(
        ErrorKind::NotFound {
            details: vec![public_contract::ErrorDetail::field("path".to_owned(), path)],
        },
        ErrorContext::new(request_id),
    )
    .into_response()
}

async fn source_quality_backlog(
    Authenticated(authentication): Authenticated,
    Extension(request_trace): Extension<RequestTraceEvidence>,
) -> axum::response::Response {
    let location_id = match authentication::actor_location_id(&authentication) {
        Ok(location_id) => location_id,
        Err(rejection) => {
            return PublicApiError::new(
                rejection.into(),
                ErrorContext::new(request_trace.request_id()),
            )
            .into_response();
        }
    };
    if let Err(rejection) = authentication::authorize_read(
        &authentication,
        authentication::Read::SourceQualityBacklog,
        Some(location_id),
        None,
    ) {
        return PublicApiError::new(
            rejection.into(),
            ErrorContext::new(request_trace.request_id()),
        )
        .into_response();
    }
    let Some(database_url) = configured_database_url() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(source_quality_backlog_payload(
                ReadModelDatabasePayload {
                    status: "not_configured",
                    adapter: "tokio_postgres",
                    error: Some("DATABASE_URL is not configured; in-memory workflow routes remain available".to_owned()),
                },
                Vec::new(),
            )),
        )
            .into_response();
    };

    let repository = storage::workflow_repository::PostgresSourceQualityBacklog::new(database_url);
    match repository
        .prioritized_items_for_location(entities::LocationId(location_id))
        .await
    {
        Ok(records) => (
            StatusCode::OK,
            Json(source_quality_backlog_payload(
                ReadModelDatabasePayload {
                    status: "connected",
                    adapter: "tokio_postgres",
                    error: None,
                },
                records,
            )),
        )
            .into_response(),
        Err(_error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(source_quality_backlog_payload(
                ReadModelDatabasePayload {
                    status: "query_failed",
                    adapter: "tokio_postgres",
                    error: Some("postgres read-model query failed; details redacted".to_owned()),
                },
                Vec::new(),
            )),
        )
            .into_response(),
    }
}

fn source_quality_backlog_payload(
    database: ReadModelDatabasePayload,
    records: Vec<workflow_repository::source_quality_backlog::Item>,
) -> SourceQualityBacklogPayload {
    SourceQualityBacklogPayload {
        api_contract: api_dto_contract("source_quality_backlog_read_model"),
        read_model: ReadModelDescriptorPayload {
            name: "source_quality_backlog",
            source: "postgres_view",
            projection_version: "source_quality_backlog.v1",
        },
        data_posture: ReadModelDataPosturePayload {
            safe_synthetic_data: true,
            live_side_effects_allowed: false,
            provider_payload_passthrough: false,
            provider_writes_allowed: false,
            customer_messages_allowed: false,
        },
        database,
        records,
    }
}

fn configured_database_url() -> Option<String> {
    env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty() && !value.contains("***"))
}

fn database_url_configured() -> bool {
    configured_database_url().is_some()
}

fn minio_env_configured() -> bool {
    env::var("MINIO_ENDPOINT")
        .ok()
        .is_some_and(|value| !value.trim().is_empty())
}

async fn run_information_lifespan_demo(
    Authenticated(authentication): Authenticated,
    Extension(request_trace): Extension<RequestTraceEvidence>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_source_ingest(
        &authentication,
        authentication::Mutation::InformationLifespanDemo,
        local_data_quality_hygiene_location_id().0,
    ) {
        return (
            rejection.status_code(),
            Json(authorization_error_payload(
                rejection,
                "information_lifespan_demo_run",
                "run_persisted",
            )),
        )
            .into_response();
    }

    let trace = information_lifespan::mock_gingr_manager_daily_report_trace();
    let correlation_id = trace.correlation_id().as_str().to_owned();
    let processor_proof = information_lifespan_processor_proof().await;

    tracing::info!(
        workflow = "information_lifespan_demo_run",
        correlation_id,
        synthetic_data_only = true,
        live_side_effects_allowed = false,
        "information lifespan demo replay returned manager report artifact"
    );

    (
        StatusCode::OK,
        Json(information_lifespan_run_payload(
            "information_lifespan_demo_run",
            &trace,
            &request_trace,
            processor_proof,
        )),
    )
        .into_response()
}

async fn replay_information_lifespan_report(
    Authenticated(authentication): Authenticated,
    Path(correlation_id): Path<String>,
    Extension(request_trace): Extension<RequestTraceEvidence>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_read(
        &authentication,
        authentication::Read::InformationLifespanReport,
        Some(local_data_quality_hygiene_location_id().0),
        None,
    ) {
        return PublicApiError::new(
            rejection.into(),
            ErrorContext::new(request_trace.request_id()),
        )
        .into_response();
    }
    let trace = information_lifespan::mock_gingr_manager_daily_report_trace();
    if correlation_id != trace.correlation_id().as_str() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": {
                    "code": "information_lifespan_run_not_found",
                    "message": "Only the deterministic synthetic information-lifespan demo run is available in the local replay API"
                },
                "requested_correlation_id": correlation_id,
                "available_correlation_id": trace.correlation_id().as_str(),
                "synthetic_data_only": true,
                "live_side_effects_allowed": false
            })),
        )
            .into_response();
    }

    let processor_proof = information_lifespan_processor_proof().await;
    (
        StatusCode::OK,
        Json(information_lifespan_run_payload(
            "information_lifespan_demo_report",
            &trace,
            &request_trace,
            processor_proof,
        )),
    )
        .into_response()
}

async fn information_lifespan_processor_proof() -> Value {
    let output_dir = env::var("INFORMATION_LIFESPAN_OUTPUT_DIR")
        .unwrap_or_else(|_| ".var/information-lifespan".to_owned());
    let output_path = format!("{output_dir}/processor-output.json");
    match std::fs::read_to_string(&output_path) {
        Ok(contents) => serde_json::from_str::<Value>(&contents).unwrap_or_else(|error| {
            information_lifespan_processor_fallback(
                "processor_output_parse_failed",
                Some(output_path),
                Some(error.to_string()),
            )
        }),
        Err(error) => information_lifespan_processor_fallback(
            "processor_output_not_present",
            Some(output_path),
            Some(error.to_string()),
        ),
    }
}

fn information_lifespan_processor_fallback(
    status: &'static str,
    output_path: Option<String>,
    error: Option<String>,
) -> Value {
    json!({
        "contract_version": "information_lifespan_hermes_processor_output.v1.local_api_fallback",
        "status": status,
        "output_path": output_path,
        "error": error,
        "simulated": true,
        "why_simulated": "The API replay can return the deterministic app trace without starting Docker; run scripts/demo_information_lifespan.sh to refresh the real processor artifact.",
        "processor": {
            "service": "hermes-processor",
            "runtime_status": "processor_artifact_unavailable_to_api"
        },
        "calculations": {
            "estimated_labor_minutes_saved": 42
        },
        "final_report": {
            "artifact_ref": "artifact://manager-daily-report/synthetic-2026-06-29",
            "manager_actions": [
                "review demand against staffing plan",
                "review vaccine/care exception gates"
            ]
        },
        "live_side_effects_allowed": false,
        "review_gates": [
            {"gate": "provider_write_locked", "locked": true},
            {"gate": "customer_send_locked", "locked": true},
            {"gate": "medical_review_required", "locked": true},
            {"gate": "schedule_change_locked", "locked": true},
            {"gate": "payment_movement_locked", "locked": true}
        ]
    })
}

fn information_lifespan_run_payload(
    workflow: &'static str,
    trace: &information_lifespan::TraceEnvelope,
    request_trace: &RequestTraceEvidence,
    processor_proof: Value,
) -> Value {
    json!({
        "api_contract": api_dto_contract(workflow),
        "correlation_id": trace.correlation_id().as_str(),
        "observability": workflow_observability_payload(trace.correlation_id().as_str(), request_trace),
        "trace": trace,
        "source_model_proof": {
            "source_system": trace.source_system(),
            "source_payloads": trace.source_payloads(),
            "provider_payloads_are_source_evidence_only": trace.provider_payloads_are_source_evidence_only()
        },
        "db_proof": trace.db_proof_entries(),
        "processor_proof": processor_proof,
        "calculations": trace.calculations(),
        "safety": {
            "synthetic_data_only": trace.uses_synthetic_data_only(),
            "live_side_effects_allowed": trace.live_side_effects_allowed(),
            "review_gates": trace.safety_gates(),
            "locked_side_effects": [
                "provider_pms_writes",
                "customer_sends",
                "medical_or_vaccine_acceptance",
                "schedule_or_staffing_changes",
                "payments_refunds_discounts"
            ]
        },
        "network_proof": trace.network_proof_entries(),
        "log_proof": trace.log_proof_entries(),
        "final_report": trace.final_artifact(),
        "deferred_production_work": [
            "live NVA/Gingr access remains intentionally absent; all source evidence is synthetic/mock read-only fixture data",
            "real customer sends, provider/PMS writes, payments/refunds/discounts, schedule changes, and medical/safety decisions remain locked behind human/system-of-record review gates",
            "the API replay reads the processor artifact when present and otherwise labels its local fallback instead of pretending Docker or an LLM ran"
        ]
    })
}

async fn healthz() -> Json<HealthPayload> {
    Json(HealthPayload {
        api_contract: api_dto_contract("runtime_health"),
        service: "pet-resort-api",
        status: "ok",
        live_side_effects: "disabled",
    })
}

async fn readyz() -> Json<ReadinessPayload> {
    Json(ReadinessPayload {
        api_contract: api_dto_contract("runtime_readiness"),
        service: "pet-resort-api",
        database: database_readiness_status(),
        object_storage: object_storage_readiness_status(),
        agent_runtime: "fake_deterministic",
        workflow_repository: workflow_repository_readiness_payload(),
        observability: ObservabilityReadinessPayload {
            request_correlation: "x_request_id_and_x_correlation_id_response_headers_with_workflow_payload_fields",
            workflow_correlation: "local_workflow_correlation_ids_only",
            local_request_metrics: "api_request_span_fields_and_aggregate_summary_only",
            metrics_scope: "aggregate_local_counters_and_labor_rollups",
            production_gap: "no_durable_traces_queue_dashboard_or_alerting",
        },
        live_customer_messaging: "disabled",
        live_provider_writes: "disabled",
    })
}

async fn ops_metrics_summary(
    State(state): State<VaccineDocumentState>,
    Authenticated(authentication): Authenticated,
    Extension(request_trace): Extension<RequestTraceEvidence>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_read(
        &authentication,
        authentication::Read::OperationalMetrics,
        Some(local_data_quality_hygiene_location_id().0),
        None,
    ) {
        return PublicApiError::new(
            rejection.into(),
            ErrorContext::new(request_trace.request_id()),
        )
        .into_response();
    }
    let store = state.store.lock().await;
    let manager_daily_brief =
        manager_daily_brief_labor_rollup(store.manager_daily_brief_outcomes.outcomes());
    let data_quality_hygiene =
        data_quality_hygiene_labor_rollup(store.data_quality_hygiene_outcomes.outcomes());
    let counters = store.runtime_counters();

    Json(OpsMetricsSummaryPayload {
        api_contract: api_dto_contract("ops_metrics_summary"),
        api_request_metrics: ApiRequestMetricsPayload {
            scope: "local_runtime_only",
            request_id_source: "x_request_id_or_generated_uuid",
            correlation_id_source: "x_correlation_id_or_generated_uuid",
            payload_logging: "disabled",
            safe_error_classes: vec!["validation_failed", "not_found", "not_applicable"],
        },
        product_labor_metrics: ProductLaborMetricsPayload {
            manager_daily_brief,
            data_quality_hygiene,
        },
        local_runtime_counters: LocalRuntimeCountersPayload {
            inquiry_count: counters.inquiry_count,
            review_packet_count: counters.review_packet_count,
            audit_event_count: counters.audit_event_count,
            outcome_count: counters.outcome_count,
            data_quality_hygiene_outbox_candidate_count: counters.internal_outbox_candidate_count,
            data_quality_hygiene_review_gated_outbox_count: counters
                .review_gated_internal_outbox_count,
            production_queue_adapter: "not_configured",
        },
        safety: MetricsSafetyPayload {
            granularity: "aggregate_only",
            contains_customer_pii: false,
            contains_provider_payloads: false,
            live_side_effects: "disabled",
        },
        observability_gap: ObservabilityGapPayload {
            production_traces: "not_configured",
            durable_request_metrics: "not_configured",
            dashboard_or_alerting: "not_configured",
        },
        production_metrics_plan: vec![
            "request_latency",
            "error_rate",
            "queue_depth",
            "dead_letter_count",
            "review_sla",
            "outbox_failures",
            "worker_lease_age",
        ],
    })
    .into_response()
}

fn manager_daily_brief_labor_rollup(
    records: &[storage::operations::ManagerDailyBriefOutcomeRecord],
) -> LaborOutcomeRollupPayload {
    let mut completed_count = 0;
    let mut total_estimated_minutes_saved = 0u16;
    let mut completed_actual_minutes_saved = 0u16;

    for record in records {
        total_estimated_minutes_saved =
            total_estimated_minutes_saved.saturating_add(record.estimated_minutes_saved);
        if record.outcome == storage::operations::ManagerDailyBriefOutcomeCode::Completed {
            completed_count += 1;
            completed_actual_minutes_saved =
                completed_actual_minutes_saved.saturating_add(record.actual_minutes_saved());
        }
    }

    LaborOutcomeRollupPayload {
        metric_source: "manager_daily_brief_outcome_records",
        outcome_count: records.len(),
        completed_count,
        total_estimated_minutes_saved,
        completed_actual_minutes_saved,
    }
}

fn data_quality_hygiene_labor_rollup(
    records: &[storage::operations::DataQualityHygieneOutcomeRecord],
) -> LaborOutcomeRollupPayload {
    let mut completed_count = 0;
    let mut total_estimated_minutes_saved = 0u16;
    let mut completed_actual_minutes_saved = 0u16;

    for record in records {
        total_estimated_minutes_saved =
            total_estimated_minutes_saved.saturating_add(record.estimated_minutes_saved);
        if record.outcome == storage::operations::DataQualityHygieneOutcomeCode::Completed {
            completed_count += 1;
            completed_actual_minutes_saved =
                completed_actual_minutes_saved.saturating_add(record.actual_minutes_saved());
        }
    }

    LaborOutcomeRollupPayload {
        metric_source: "data_quality_hygiene_outcome_records",
        outcome_count: records.len(),
        completed_count,
        total_estimated_minutes_saved,
        completed_actual_minutes_saved,
    }
}

fn classify_invalid_inquiry(request: &InquirySubmissionRequest) -> Option<Value> {
    let received_at = request.received_at?;
    let mut previous_attempt_at = None;
    for attempt in &request.contact_attempts {
        if attempt.attempted_at < received_at {
            return Some(invalid_inquiry_payload(
                "invalid_attempt_before_receipt",
                &request.source_event_key,
            ));
        }
        if previous_attempt_at.is_some_and(|previous| attempt.attempted_at < previous) {
            return Some(invalid_inquiry_payload(
                "invalid_out_of_order_event",
                &request.source_event_key,
            ));
        }
        previous_attempt_at = Some(attempt.attempted_at);
    }
    None
}

fn invalid_inquiry_payload(classification: &'static str, source_event_key: &str) -> Value {
    json!({
        "api_contract": api_dto_contract("inquiry_intake_rejected"),
        "classification": classification,
        "source_event_key": source_event_key,
        "accepted": false,
        "live_send_allowed": false,
        "provider_write_allowed": false,
        "queue_send_capability_reachable": false,
        "policy_boundary": "invalid_source_event_remains_draft_only_no_live_send_no_provider_write"
    })
}

async fn submit_inquiry(
    State(state): State<VaccineDocumentState>,
    Authenticated(authentication): Authenticated,
    Json(request): Json<InquirySubmissionRequest>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_source_ingest(
        &authentication,
        authentication::Mutation::InquiryIntake,
        Uuid::parse_str(&request.location_id).unwrap_or(Uuid::nil()),
    ) {
        return PublicApiError::new(rejection.into(), ErrorContext::new("missing_request_id"))
            .into_response();
    }
    let mut store = state.store.lock().await;
    if let Some(mut record) = store
        .inquiry_intake_records
        .iter()
        .find(|record| record.event.source_event_key == request.source_event_key)
        .cloned()
    {
        if !inquiry_payload_matches_record(&request, &record) {
            return (
                StatusCode::CONFLICT,
                Json(invalid_inquiry_payload(
                    "idempotency_payload_drift",
                    &request.source_event_key,
                )),
            )
                .into_response();
        }
        record.replay = Some(json!({
            "classification": "duplicate_idempotent_replay",
            "source_event_key": request.source_event_key,
            "stored_record_reused": true
        }));
        return (StatusCode::OK, Json(record)).into_response();
    }
    if let Some(error_payload) = classify_invalid_inquiry(&request) {
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(error_payload)).into_response();
    }
    let record = build_inquiry_intake_record(request);
    store.inquiry_intake_records.push(record.clone());
    (StatusCode::CREATED, Json(record)).into_response()
}

fn inquiry_payload_matches_record(
    request: &InquirySubmissionRequest,
    record: &InquiryIntakeRecord,
) -> bool {
    record.event.location_id == request.location_id
        && record.lead.customer_name == request.customer.full_name
        && record.lead.customer_email == request.customer.email
        && record.lead.customer_phone == request.customer.phone
        && record.lead.pet_name == request.pet.name
        && record.lead.species == request.pet.species
        && record.lead.service == request.service
        && record.lead.original_message == request.message
        && record
            .lead
            .requested_dates
            .as_ref()
            .map(|dates| (&dates.start, &dates.end))
            == request
                .requested_dates
                .as_ref()
                .map(|dates| (&dates.start, &dates.end))
}

async fn staff_inquiries(
    State(state): State<VaccineDocumentState>,
    Authenticated(authentication): Authenticated,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_read(
        &authentication,
        authentication::Read::StaffInquiries,
        None,
        None,
    ) {
        return PublicApiError::new(rejection.into(), ErrorContext::new("missing_request_id"))
            .into_response();
    }
    let location_id = authentication::actor_location_id(&authentication)
        .expect("authenticated read authorization resolved an actor");
    let store = state.store.lock().await;
    Json(InquiryStaffQueuePayload {
        api_contract: api_dto_contract("inquiry_staff_queue"),
        records: store
            .inquiry_intake_records
            .iter()
            .filter(|record| record.event.location_id == location_id.to_string())
            .cloned()
            .collect(),
    })
    .into_response()
}

#[derive(Debug, Deserialize)]
struct ManagerDailyBriefAgentContextQuery {
    location_id: Uuid,
    operating_day: NaiveDate,
}

#[derive(Debug, Deserialize)]
struct ManagerDailyBriefAgentDraftSubmissionRequest {
    context_packet_id: String,
    correlation_id: String,
    submitted_by: String,
    actions: Vec<ManagerDailyBriefSubmittedAction>,
}

#[derive(Debug, Deserialize)]
struct ManagerDailyBriefSubmittedAction {
    id: String,
    kind: String,
    recommendation: String,
    #[serde(default)]
    source_refs: Vec<public_contract::SourceRecordRef>,
    #[serde(default)]
    review_gates: Vec<String>,
    #[serde(default)]
    requested_side_effects: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ManagerDailyBriefOutcomeCaptureRequest {
    outcome: storage::operations::ManagerDailyBriefOutcomeCode,
    actual_minutes: u16,
    actor: ManagerDailyBriefOutcomeActorRequest,
    feedback: String,
    #[serde(default)]
    source_refs: Vec<public_contract::SourceRecordRef>,
    timestamp: String,
    audit: ManagerDailyBriefOutcomeAuditRequest,
    reporting: ManagerDailyBriefOutcomeReportingRequest,
    #[serde(default)]
    requested_side_effects: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ManagerDailyBriefOutcomeActorRequest {
    id: String,
    persona: storage::operations::ManagerDailyBriefPersonaCode,
}

#[derive(Debug, Deserialize)]
struct ManagerDailyBriefOutcomeAuditRequest {
    correlation_id: String,
}

#[derive(Debug, Deserialize)]
struct ManagerDailyBriefOutcomeReportingRequest {
    location_id: String,
    operating_day: String,
}

#[derive(Debug, Deserialize)]
struct DataQualityHygieneAgentContextQuery {
    location_id: Uuid,
    operating_day: NaiveDate,
}

#[derive(Debug, Deserialize)]
struct PermissionedKnowledgeAgentContextQuery {
    location_id: Uuid,
    service: String,
    role: String,
    section: String,
}

#[derive(Debug, Deserialize)]
struct DataQualityHygieneOutcomeCaptureRequest {
    outcome: storage::operations::DataQualityHygieneOutcomeCode,
    actual_minutes: u16,
    actor: DataQualityHygieneOutcomeActorRequest,
    feedback: String,
    #[serde(default)]
    source_refs: Vec<public_contract::SourceRecordRef>,
    #[serde(default)]
    issue_refs: Vec<String>,
    resolution_status_after_review: storage::operations::DataQualityResolutionStatusCode,
    timestamp: String,
    audit: DataQualityHygieneOutcomeAuditRequest,
    #[serde(default)]
    requested_side_effects: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DataQualityHygieneOutcomeActorRequest {
    id: String,
    persona: storage::operations::DataQualityHygienePersonaCode,
}

#[derive(Debug, Deserialize)]
struct DataQualityHygieneOutcomeAuditRequest {
    correlation_id: String,
}

#[derive(Debug, Deserialize)]
struct DataQualityHygieneOutcomeSummaryQuery {
    location_id: Uuid,
    operating_day: NaiveDate,
    correlation_id: Option<String>,
}

async fn capture_manager_daily_brief_action_outcome(
    State(state): State<VaccineDocumentState>,
    Authenticated(authentication): Authenticated,
    Path(action_id): Path<String>,
    Json(request): Json<ManagerDailyBriefOutcomeCaptureRequest>,
) -> (StatusCode, Json<Value>) {
    if let Err(rejection) = authentication::authorize_actor(
        &authentication,
        authentication::Mutation::ManagerDailyBriefOutcome,
        &request.actor.id,
    )
    .and_then(|()| {
        authentication::authorize_persona_claim(&authentication, &request.actor.persona.to_string())
    }) {
        return (
            rejection.status_code(),
            Json(authorization_error_payload(
                rejection,
                "manager_daily_brief_outcome",
                "outcome_persisted",
            )),
        );
    }

    let reasons = request
        .requested_side_effects
        .iter()
        .map(|side_effect| manager_daily_brief_requested_side_effect_rejection_reason(side_effect))
        .collect::<Vec<_>>();

    if !reasons.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "accepted": false,
                "outcome_persisted": false,
                "reasons": reasons,
                "live_side_effects_allowed": false,
                "blocked_actions": manager_daily_brief_blocked_action_codes()
            })),
        );
    }

    if request.source_refs.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "accepted": false,
                "outcome_persisted": false,
                "reasons": ["missing_source_refs"],
                "live_side_effects_allowed": false,
                "blocked_actions": manager_daily_brief_blocked_action_codes()
            })),
        );
    }

    let Ok(actual_minutes) =
        storage::operations::StoredManagerDailyBriefLaborMinutes::try_new(request.actual_minutes)
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "accepted": false,
                "outcome_persisted": false,
                "reasons": ["actual_minutes_must_be_greater_than_zero"],
                "live_side_effects_allowed": false,
                "blocked_actions": manager_daily_brief_blocked_action_codes()
            })),
        );
    };

    let Some((location_id, operating_day)) =
        manager_daily_brief_reporting_scope(&request.reporting)
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "accepted": false,
                "outcome_persisted": false,
                "reasons": ["invalid_reporting_scope"],
                "live_side_effects_allowed": false,
                "blocked_actions": manager_daily_brief_blocked_action_codes()
            })),
        );
    };

    if let Err(rejection) = authentication::authorize_location(&authentication, location_id.0) {
        return (
            rejection.status_code(),
            Json(authorization_error_payload(
                rejection,
                "manager_daily_brief_outcome",
                "outcome_persisted",
            )),
        );
    }

    let packet = local_manager_daily_brief_packet(location_id, operating_day);
    let Some(action) = packet
        .actions()
        .iter()
        .find(|action| action.id().clone().into_inner() == action_id)
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "accepted": false,
                "outcome_persisted": false,
                "reasons": ["unknown_manager_daily_brief_action_id"],
                "live_side_effects_allowed": false,
                "blocked_actions": manager_daily_brief_blocked_action_codes()
            })),
        );
    };

    let expected_source_refs = manager_daily_brief_action_source_refs(action, operating_day);
    if request.source_refs != expected_source_refs {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "accepted": false,
                "outcome_persisted": false,
                "reasons": ["source_refs_do_not_match_action"],
                "live_side_effects_allowed": false,
                "blocked_actions": manager_daily_brief_blocked_action_codes()
            })),
        );
    }

    let before_minutes = storage::operations::StoredManagerDailyBriefLaborMinutes::try_new(
        action.labor_impact().before_minutes().get(),
    )
    .expect("manager daily brief action labor impact uses non-zero domain labor minutes");

    let record = storage::operations::ManagerDailyBriefOutcomeRecord::builder()
        .action_id(action_id)
        .outcome(request.outcome)
        .before_minutes(before_minutes)
        .actual_minutes(actual_minutes)
        .actor_id(request.actor.id)
        .actor_persona(request.actor.persona)
        .feedback(request.feedback)
        .source_refs(
            expected_source_refs
                .iter()
                .map(stored_source_record_ref_from_payload)
                .collect(),
        )
        .recorded_at(request.timestamp)
        .correlation_id(request.audit.correlation_id)
        .location_id(location_id.0.to_string())
        .operating_day(operating_day.get().to_string())
        .action_kind(stored_manager_daily_brief_action_kind(action.kind()))
        .owner_persona(stored_manager_daily_brief_persona(action.owner_persona()))
        .estimated_minutes_saved(action.labor_impact().minutes_saved())
        .build();
    let reporting_group = record.reporting_group();
    let persisted_outcome_count = {
        let mut store = state.store.lock().await;
        store.manager_daily_brief_outcomes.record(record.clone())
    };

    (
        StatusCode::CREATED,
        Json(json!({
            "api_contract": api_dto_contract_payload("manager_daily_brief_outcome"),
            "accepted": true,
            "outcome_persisted": true,
            "outcome_record": {
                "action_id": record.action_id,
                "outcome": record.outcome,
                "before_minutes": record.before_minutes.get(),
                "actual_minutes": record.actual_minutes.get(),
                "actor": {
                    "id": record.actor_id,
                    "persona": record.actor_persona
                },
                "feedback": record.feedback,
                "source_refs": record.source_refs,
                "timestamp": record.recorded_at,
                "audit": {
                    "correlation_id": record.correlation_id
                }
            },
            "labor_savings_evidence": {
                "estimated_minutes_saved": record.estimated_minutes_saved,
                "actual_minutes_saved": record.actual_minutes_saved(),
                "grouping": {
                    "location_id": reporting_group.location_id,
                    "operating_day": reporting_group.operating_day,
                    "action_kind": reporting_group.action_kind,
                    "owner_persona": reporting_group.owner_persona
                },
                "persisted_outcome_count": persisted_outcome_count
            },
            "live_side_effects_allowed": false,
            "blocked_actions": manager_daily_brief_blocked_action_codes(),
            "audit": {
                "event": "manager_daily_brief_outcome_recorded",
                "policy_owner": "deterministic_app"
            }
        })),
    )
}

async fn submit_manager_daily_brief_agent_draft(
    Authenticated(authentication): Authenticated,
    Json(request): Json<ManagerDailyBriefAgentDraftSubmissionRequest>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize(
        &authentication,
        authentication::Mutation::ManagerDailyBriefDraft,
        &request.submitted_by,
        local_data_quality_hygiene_location_id().0,
    ) {
        return (
            rejection.status_code(),
            Json(authorization_error_payload(
                rejection,
                "manager_daily_brief_agent_draft",
                "draft_accepted",
            )),
        )
            .into_response();
    }

    let mut accepted_actions = Vec::new();
    let mut rejected_actions = Vec::new();
    let packet = manager_daily_brief_packet_from_context_id(&request.context_packet_id);

    for action in &request.actions {
        let reasons = validate_manager_daily_brief_submitted_action(action, packet.as_ref());
        if reasons.is_empty() {
            accepted_actions.push(json!({
                "id": action.id,
                "kind": action.kind,
                "recommendation": action.recommendation,
                "review_gates": action.review_gates,
                "source_refs": action.source_refs,
                "showable_to_manager": true,
                "live_side_effects_allowed": false
            }));
        } else {
            rejected_actions.push(json!({
                "id": action.id,
                "kind": action.kind,
                "reasons": reasons,
                "showable_to_manager": false,
                "live_side_effects_allowed": false
            }));
        }
    }

    let validation_status = match (accepted_actions.is_empty(), rejected_actions.is_empty()) {
        (false, true) => "accepted",
        (false, false) => "partially_accepted",
        (true, false) => "rejected",
        (true, true) => "rejected",
    };
    let status_code = if rejected_actions.is_empty() {
        StatusCode::CREATED
    } else {
        StatusCode::UNPROCESSABLE_ENTITY
    };

    (
        status_code,
        Json(json!({
            "api_contract": api_dto_contract_payload("manager_daily_brief_agent_draft"),
            "validation": {
                "status": validation_status,
                "validator": "pet_resort_api.manager_daily_brief.agent_draft_validator.v1"
            },
            "context_packet_id": request.context_packet_id,
            "correlation_id": request.correlation_id,
            "submitted_by": request.submitted_by,
            "accepted_actions": accepted_actions,
            "rejected_actions": rejected_actions,
            "live_side_effects_allowed": false,
            "audit": {
                "event": "manager_daily_brief_agent_draft_validated",
                "policy_owner": "deterministic_app"
            }
        })),
    )
        .into_response()
}

async fn data_quality_hygiene_agent_context(
    Authenticated(authentication): Authenticated,
    Extension(request_trace): Extension<RequestTraceEvidence>,
    Query(query): Query<DataQualityHygieneAgentContextQuery>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_read(
        &authentication,
        authentication::Read::OperationalContext,
        Some(query.location_id),
        None,
    ) {
        return PublicApiError::new(
            rejection.into(),
            ErrorContext::new(request_trace.request_id()),
        )
        .into_response();
    }
    let location_id = entities::LocationId(query.location_id);
    let operating_day = operations::operating_day::Date::try_new(query.operating_day)
        .expect("operating day date is always valid after query parsing");
    let packet = local_data_quality_hygiene_packet(location_id, operating_day);

    Json(data_quality_hygiene_packet_payload(&packet, &request_trace)).into_response()
}

async fn permissioned_knowledge_agent_context(
    Authenticated(authentication): Authenticated,
    Extension(request_trace): Extension<RequestTraceEvidence>,
    Query(query): Query<PermissionedKnowledgeAgentContextQuery>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_read(
        &authentication,
        authentication::Read::PermissionedKnowledge,
        Some(query.location_id),
        Some(&query.role),
    ) {
        return PublicApiError::new(rejection.into(), ErrorContext::new("missing_request_id"))
            .into_response();
    }
    let service = permissioned_knowledge_service(&query.service);
    let trusted_role = authentication::actor_role_claim(&authentication)
        .expect("authenticated read authorization resolved an actor role");
    let role = permissioned_knowledge_role(trusted_role);
    let context = agent::assistant::ActorContext::builder()
        .actor_id(
            access::ActorId::try_new(
                authentication::actor_id(&authentication)
                    .expect("authenticated read authorization resolved an actor id"),
            )
            .expect("trusted actor ids satisfy actor-context validation"),
        )
        .role(role)
        .title(access::Title::try_new("Fixture Knowledge Actor").unwrap())
        .location_id(entities::LocationId(query.location_id))
        .purpose(agent::assistant::Purpose::SopLookup)
        .allowed_uses(vec![access::AllowedUse::InternalDecisionSupport])
        .build();
    let request = app::permissioned_knowledge::Request::builder()
        .context(context)
        .service(service)
        .requested_section(agent::knowledge::SectionRef::try_new(&query.section).unwrap())
        .requested_at(Utc.with_ymd_and_hms(2026, 8, 12, 15, 0, 0).unwrap())
        .build();
    let packet = app::permissioned_knowledge::Workflow::answer(
        &app::permissioned_knowledge::DeterministicFixtureRepository::default(),
        request,
    );
    let correlation_id = format!(
        "permissioned-knowledge:{}:{}:{}",
        query.location_id, query.service, query.section
    );

    Json(json!({
        "api_contract": api_dto_contract_payload("permissioned_knowledge_assistant_packet"),
        "workflow": {
            "name": "permissioned_knowledge_retrieval",
            "version": "local-permissioned-knowledge-context-v1"
        },
        "actor_request": {
            "location_id": query.location_id.to_string(),
            "service": query.service,
            "role": query.role,
            "purpose": "sop_lookup"
        },
        "retrieval": {
            "repository_adapter": "deterministic_fixture_repository",
            "authorization_before_context": true,
            "safe_to_enter_assistant_context": packet.safe_to_enter_assistant_context(),
            "authorized_passage_count": packet.authorized_passages().len(),
            "citation_count": packet.citations().len(),
            "claims_are_cited": packet.claims_are_cited(),
            "content_redacted_from_api_debug": true
        },
        "answer_packet": {
            "state": packet.answer_state(),
            "escalation_reason": packet.escalation_reason(),
            "claim_text_redacted": true
        },
        "safety": {
            "live_side_effects_allowed": packet.live_side_effects_allowed(),
            "forbidden_actions": packet.forbidden_actions(),
            "provider_payload_passthrough": false,
            "customer_messages_allowed": false,
            "provider_writes_allowed": false
        },
        "audit": {
            "context_packet_id": format!("permissioned-knowledge-context:{}:{}", query.location_id, query.section),
            "correlation_id": correlation_id,
            "policy_owner": "deterministic_app"
        },
        "observability": workflow_observability_payload(&correlation_id, &request_trace)
    }))
    .into_response()
}

async fn site_finance_agent_context(
    Authenticated(authentication): Authenticated,
    Extension(request_trace): Extension<RequestTraceEvidence>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_read(
        &authentication,
        authentication::Read::SiteFinance,
        Some(local_data_quality_hygiene_location_id().0),
        None,
    ) {
        return PublicApiError::new(
            rejection.into(),
            ErrorContext::new(request_trace.request_id()),
        )
        .into_response();
    }
    let slice = match site_finance::fixture_site_period_projection() {
        Ok(slice) => slice,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "api_contract": api_dto_contract_payload("site_finance_recommendation_packet"),
                    "safe_error_class": "fixture_validation_failed",
                    "error": error.to_string(),
                    "live_side_effects_allowed": false
                })),
            )
                .into_response();
        }
    };
    let review_packet_id = serde_json::to_value(slice.action().review_packet_id())
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "site-finance-review:00c0ffee:2026-06".to_owned());
    let audit_event_id = serde_json::to_value(slice.action().audit_event_id())
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "audit:site-finance-review:00c0ffee:2026-06".to_owned());
    let correlation_id = "site-finance:00c0ffee:2026-06";

    (
        StatusCode::OK,
        Json(json!({
            "api_contract": api_dto_contract_payload("site_finance_recommendation_packet"),
            "workflow": {
                "name": "site_finance_recommendation_outcome",
                "version": "local-site-finance-context-v1"
            },
            "projection": {
                "data_quality_status": slice.projection().data_quality_status(),
                "net_revenue": slice.projection().net_revenue().ok(),
                "variance": slice.projection().variance(),
                "source_ref_count": slice.action().source_record_refs().len()
            },
            "recommendation": {
                "review_gate": slice.recommendation().required_review_gate(),
                "blocks_financial_mutation": slice.recommendation().blocks_financial_mutation(),
                "kind": slice.recommendation().recommendation()
            },
            "action": {
                "legal_action": slice.action().legal_action(),
                "allows_financial_mutation": slice.action().allows_financial_mutation(),
                "review_packet_id": review_packet_id,
                "audit_event_id": audit_event_id
            },
            "outcome": {
                "strong_attribution_claimable": slice.strong_outcome().can_support_value_claim(),
                "weak_attribution_claimable": slice.weak_outcome().can_support_value_claim()
            },
            "safety": {
                "live_side_effects_allowed": false,
                "customer_messages_allowed": false,
                "provider_writes_allowed": false,
                "financial_mutations_allowed": false
            },
            "observability": workflow_observability_payload(correlation_id, &request_trace)
        })),
    )
        .into_response()
}

fn permissioned_knowledge_service(raw: &str) -> entities::ServiceKind {
    match raw {
        "grooming" => entities::ServiceKind::Grooming,
        "training" => entities::ServiceKind::Training,
        "day_play" => entities::ServiceKind::DayPlay,
        _ => entities::ServiceKind::Boarding,
    }
}

fn permissioned_knowledge_role(raw: &str) -> access::ActorRole {
    match raw {
        "site_manager" => access::ActorRole::SiteManager,
        "marketing" => access::ActorRole::Marketing,
        "regional_operations" => access::ActorRole::RegionalOperations,
        _ => access::ActorRole::FrontDesk,
    }
}

async fn submit_data_quality_hygiene_agent_draft(
    Authenticated(authentication): Authenticated,
    Json(request): Json<public_contract::DataQualityHygieneDraftSubmissionRequest>,
) -> (StatusCode, Json<Value>) {
    if let Err(rejection) = authentication::authorize_source_ingest(
        &authentication,
        authentication::Mutation::DataQualityHygieneDraft,
        local_data_quality_hygiene_location_id().0,
    ) {
        return (
            rejection.status_code(),
            Json(authorization_error_payload(
                rejection,
                "data_quality_hygiene_agent_draft",
                "draft_accepted",
            )),
        );
    }

    let packet = local_data_quality_hygiene_packet(
        local_data_quality_hygiene_location_id(),
        local_data_quality_hygiene_operating_day(),
    );
    let mut accepted_actions = Vec::new();
    let mut rejected_actions = Vec::new();

    for action in &request.actions {
        let reasons = validate_data_quality_hygiene_submitted_action(&packet, action);
        if reasons.is_empty() {
            accepted_actions.push(json!({
                "id": action.action_id,
                "kind": action.kind,
                "review_gates": action.review_gates,
                "source_refs": action.source_refs,
                "issue_refs": action.issue_refs,
                "showable_to_manager": true,
                "live_side_effects_allowed": false
            }));
        } else {
            rejected_actions.push(json!({
                "id": action.action_id,
                "kind": action.kind,
                "reasons": reasons,
                "showable_to_manager": false,
                "live_side_effects_allowed": false
            }));
        }
    }

    let validation_status = match (accepted_actions.is_empty(), rejected_actions.is_empty()) {
        (false, true) => "accepted",
        (false, false) => "partially_accepted",
        (true, false) | (true, true) => "rejected",
    };
    let status_code = if rejected_actions.is_empty() {
        StatusCode::CREATED
    } else {
        StatusCode::UNPROCESSABLE_ENTITY
    };

    (
        status_code,
        Json(json!({
            "api_contract": api_dto_contract_payload("data_quality_hygiene_agent_draft"),
            "validation": {
                "status": validation_status,
                "validator": "pet_resort_api.data_quality_hygiene.agent_draft_validator.v1",
                "safe_error_class": if rejected_actions.is_empty() { "accepted" } else { "validation_failed" }
            },
            "context_packet_id": request.context_packet_id,
            "correlation_id": request.correlation_id,
            "accepted_actions": accepted_actions,
            "rejected_actions": rejected_actions,
            "live_side_effects_allowed": false,
            "audit": {
                "event": "data_quality_hygiene_agent_draft_validated",
                "policy_owner": "deterministic_app"
            }
        })),
    )
}

async fn capture_data_quality_hygiene_action_outcome(
    State(state): State<VaccineDocumentState>,
    Authenticated(authentication): Authenticated,
    Path(action_id): Path<String>,
    Json(request): Json<DataQualityHygieneOutcomeCaptureRequest>,
) -> (StatusCode, Json<Value>) {
    if let Err(rejection) = authentication::authorize(
        &authentication,
        authentication::Mutation::DataQualityHygieneOutcome,
        &request.actor.id,
        local_data_quality_hygiene_location_id().0,
    )
    .and_then(|()| {
        authentication::authorize_persona_claim(&authentication, &request.actor.persona.to_string())
    }) {
        return (
            rejection.status_code(),
            Json(authorization_error_payload(
                rejection,
                "data_quality_hygiene_outcome",
                "outcome_persisted",
            )),
        );
    }

    let reasons = request
        .requested_side_effects
        .iter()
        .map(|side_effect| data_quality_hygiene_requested_side_effect_rejection_reason(side_effect))
        .collect::<Vec<_>>();

    if !reasons.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "accepted": false,
                "outcome_persisted": false,
                "reasons": reasons,
                "live_side_effects_allowed": false,
                "blocked_actions": data_quality_hygiene_blocked_action_codes()
            })),
        );
    }

    if request.source_refs.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "accepted": false,
                "outcome_persisted": false,
                "reasons": ["missing_source_refs"],
                "live_side_effects_allowed": false,
                "blocked_actions": data_quality_hygiene_blocked_action_codes()
            })),
        );
    }

    if request.issue_refs.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "accepted": false,
                "outcome_persisted": false,
                "reasons": ["missing_data_quality_issue_refs"],
                "live_side_effects_allowed": false,
                "blocked_actions": data_quality_hygiene_blocked_action_codes()
            })),
        );
    }

    let Ok(actual_minutes) =
        storage::operations::StoredDataQualityHygieneLaborMinutes::try_new(request.actual_minutes)
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "accepted": false,
                "outcome_persisted": false,
                "reasons": ["actual_minutes_must_be_greater_than_zero"],
                "live_side_effects_allowed": false,
                "blocked_actions": data_quality_hygiene_blocked_action_codes()
            })),
        );
    };

    let packet = local_data_quality_hygiene_packet(
        local_data_quality_hygiene_location_id(),
        local_data_quality_hygiene_operating_day(),
    );
    let Some(action) = packet
        .actions()
        .iter()
        .find(|action| action.id().as_str() == action_id)
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "accepted": false,
                "outcome_persisted": false,
                "reasons": ["unknown_data_quality_hygiene_action_id"],
                "live_side_effects_allowed": false,
                "blocked_actions": data_quality_hygiene_blocked_action_codes()
            })),
        );
    };

    let expected_source_refs = data_quality_hygiene_action_source_refs(&packet, action);
    let expected_issue_refs = action
        .issue_refs()
        .iter()
        .map(|issue_ref| issue_ref.as_str().to_owned())
        .collect::<Vec<_>>();
    let mut provenance_reasons = Vec::new();
    if request.source_refs != expected_source_refs {
        provenance_reasons.push("source_refs_do_not_match_action");
    }
    if request.issue_refs != expected_issue_refs {
        provenance_reasons.push("issue_refs_do_not_match_action");
    }
    provenance_reasons.sort_unstable();
    if !provenance_reasons.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "accepted": false,
                "outcome_persisted": false,
                "reasons": provenance_reasons,
                "live_side_effects_allowed": false,
                "blocked_actions": data_quality_hygiene_blocked_action_codes()
            })),
        );
    }

    let before_minutes = storage::operations::StoredDataQualityHygieneLaborMinutes::try_new(
        action.labor_impact().before_minutes().get(),
    )
    .expect("data-quality hygiene action labor impact uses non-zero domain labor minutes");

    let record = storage::operations::DataQualityHygieneOutcomeRecord::builder()
        .action_id(action_id)
        .outcome(request.outcome)
        .before_minutes(before_minutes)
        .actual_minutes(actual_minutes)
        .actor_id(request.actor.id)
        .actor_persona(request.actor.persona)
        .feedback(request.feedback)
        .source_refs(
            request
                .source_refs
                .iter()
                .map(stored_source_record_ref_from_payload)
                .collect(),
        )
        .issue_refs(request.issue_refs)
        .resolution_status_after_review(request.resolution_status_after_review)
        .recorded_at(request.timestamp)
        .correlation_id(request.audit.correlation_id)
        .location_id(packet.location_id().0.to_string())
        .operating_day(packet.operating_day().get().to_string())
        .action_kind(stored_data_quality_hygiene_action_kind(action.kind()))
        .owner_persona(stored_data_quality_hygiene_persona(action.owner_persona()))
        .estimated_minutes_saved(action.labor_impact().minutes_saved())
        .build();
    let reporting_group = record.reporting_group();
    let local_persistence_records =
        storage::operations::DataQualityHygieneLocalPersistenceRecords::from_reviewed_outcome(
            data_quality_hygiene_lineage_ids(&record),
            record.clone(),
        );
    let storage_projection_proof =
        data_quality_hygiene_storage_projection_proof(&local_persistence_records);
    let observability = data_quality_hygiene_outcome_observability_payload(
        &record.correlation_id,
        &local_persistence_records,
    );
    let (persisted_outcome_count, persisted_projection_count) = {
        let mut store = state.store.lock().await;
        let persisted_outcome_count = store.data_quality_hygiene_outcomes.record(record.clone());
        store
            .data_quality_hygiene_persistence_records
            .push(local_persistence_records);
        let persisted_projection_count = store.data_quality_hygiene_persistence_records.len();
        (persisted_outcome_count, persisted_projection_count)
    };

    (
        StatusCode::CREATED,
        Json(json!({
            "api_contract": api_dto_contract_payload("data_quality_hygiene_outcome"),
            "accepted": true,
            "outcome_persisted": true,
            "outcome_record": {
                "action_id": record.action_id,
                "outcome": record.outcome,
                "before_minutes": record.before_minutes.get(),
                "actual_minutes": record.actual_minutes.get(),
                "actor": {
                    "id": record.actor_id,
                    "persona": record.actor_persona
                },
                "feedback": record.feedback,
                "source_refs": record.source_refs,
                "issue_refs": record.issue_refs,
                "resolution_status_after_review": record.resolution_status_after_review,
                "timestamp": record.recorded_at,
                "audit": {
                    "correlation_id": record.correlation_id
                }
            },
            "labor_savings_evidence": {
                "estimated_minutes_saved": record.estimated_minutes_saved,
                "actual_minutes_saved": record.actual_minutes_saved(),
                "grouping": {
                    "location_id": reporting_group.location_id,
                    "operating_day": reporting_group.operating_day,
                    "action_kind": reporting_group.action_kind,
                    "owner_persona": reporting_group.owner_persona
                },
                "persisted_outcome_count": persisted_outcome_count,
                "persisted_projection_count": persisted_projection_count
            },
            "local_demo_readiness": data_quality_hygiene_local_demo_readiness_payload(),
            "storage_projection_proof": storage_projection_proof,
            "observability": observability,
            "live_side_effects_allowed": false,
            "blocked_actions": data_quality_hygiene_blocked_action_codes(),
            "audit": {
                "event": "data_quality_hygiene_outcome_recorded",
                "policy_owner": "deterministic_app"
            }
        })),
    )
}

async fn data_quality_hygiene_outcome_summary(
    State(state): State<VaccineDocumentState>,
    Authenticated(authentication): Authenticated,
    Extension(request_trace): Extension<RequestTraceEvidence>,
    Query(query): Query<DataQualityHygieneOutcomeSummaryQuery>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_read(
        &authentication,
        authentication::Read::OperationalContext,
        Some(query.location_id),
        None,
    ) {
        return PublicApiError::new(
            rejection.into(),
            ErrorContext::new(request_trace.request_id()),
        )
        .into_response();
    }
    let location_id = query.location_id.to_string();
    let operating_day = query.operating_day.to_string();
    let records = {
        let store = state.store.lock().await;
        store.data_quality_hygiene_outcomes.outcomes().to_vec()
    };
    let summary = storage::operations::DataQualityHygieneOutcomeSummary::from_records(
        &records,
        &location_id,
        &operating_day,
        query.correlation_id.as_deref(),
    );

    Json(json!({
        "api_contract": api_dto_contract_payload("data_quality_hygiene_outcome_summary"),
        "summary": summary,
        "live_side_effects_allowed": false,
        "blocked_actions": data_quality_hygiene_blocked_action_codes(),
        "audit": {
            "event": "data_quality_hygiene_outcome_summary_reported",
            "policy_owner": "deterministic_app"
        }
    }))
    .into_response()
}

fn data_quality_hygiene_lineage_ids(
    record: &storage::operations::DataQualityHygieneOutcomeRecord,
) -> storage::operations::DataQualityHygieneLineageIds {
    let lineage_key = data_quality_hygiene_lineage_key(record);
    storage::operations::DataQualityHygieneLineageIds::builder()
        .workflow_event_id(format!("dqh-workflow-event:{lineage_key}"))
        .review_packet_id(format!("dqh-review-packet:{lineage_key}"))
        .approval_record_id(format!("dqh-approval:{lineage_key}"))
        .outbox_record_id(format!("dqh-outbox:{lineage_key}"))
        .subject_id(record.location_id.clone())
        .idempotency_key(format!("dqh:{lineage_key}"))
        .recorded_at(record.recorded_at.clone())
        .build()
}

fn data_quality_hygiene_lineage_key(
    record: &storage::operations::DataQualityHygieneOutcomeRecord,
) -> String {
    format!(
        "{}:{}:{}",
        record.location_id, record.operating_day, record.action_id
    )
    .chars()
    .map(|character| match character {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => character,
        _ => '-',
    })
    .collect()
}

fn data_quality_hygiene_local_demo_readiness_payload() -> Value {
    json!({
        "mode": "local_demo_only",
        "workflow_repository": "in_memory_typed_storage_projection",
        "database": "not_configured",
        "provider_access": "fixture_source_refs_only",
        "live_provider_writes": "disabled",
        "live_customer_sends": "disabled",
        "payments": "disabled"
    })
}

fn data_quality_hygiene_outcome_observability_payload(
    correlation_id: &str,
    records: &storage::operations::DataQualityHygieneLocalPersistenceRecords,
) -> Value {
    let outbox_state = if records.outbox_candidate.is_some() {
        "approved_internal_outbox_candidate_created"
    } else {
        "review_pending_no_outbox_authority"
    };
    json!({
        "correlation_id": correlation_id,
        "workflow_event_id": records.workflow_event.id,
        "review_packet_id": records.review_packet.id,
        "outbox_candidate_id": records.outbox_candidate.as_ref().map(|candidate| candidate.id()),
        "what_happened": outbox_state,
        "what_was_blocked": ["provider_writes", "customer_sends", "payments", "schedule_changes"],
        "production_next_step": "durable_worker_leasing_retry_dead_letter_metrics_and_approved_adapter_execution",
        "observability_scope": "single_local_workflow_response_only"
    })
}

fn data_quality_hygiene_storage_projection_proof(
    records: &storage::operations::DataQualityHygieneLocalPersistenceRecords,
) -> Value {
    let outbox_candidate = records.outbox_candidate.as_ref().map(|candidate| {
        json!({
            "id": candidate.id(),
            "topic": candidate.topic(),
            "status": candidate.status(),
            "review_gate": candidate.review_gate(),
            "internal_handoff_only": candidate.payload()["internal_handoff_only"].as_bool().unwrap_or(false),
            "live_delivery_allowed": candidate.payload()["live_delivery_allowed"].as_bool().unwrap_or(false)
        })
    });

    json!({
        "workflow_event_id": records.workflow_event.id,
        "workflow_result_status": records.workflow_result.status,
        "review_packet_id": records.review_packet.id,
        "review_gate": records.review_packet.gate,
        "approval_record_id": records.approval_record.id,
        "audit_event_count": records.audit_events.len(),
        "outbox_candidate": outbox_candidate,
        "live_side_effects_allowed": false
    })
}

fn validate_manager_daily_brief_submitted_action(
    action: &ManagerDailyBriefSubmittedAction,
    packet: Option<&manager_daily_brief::Packet>,
) -> Vec<String> {
    let mut reasons = Vec::new();

    let Some(required_review_gate) = required_manager_daily_brief_review_gate(&action.kind) else {
        reasons.push("unsupported_action_kind".to_owned());
        return reasons;
    };

    let source_refs_are_complete = !action.source_refs.is_empty()
        && action.source_refs.iter().all(|source_ref| {
            [
                &source_ref.system,
                &source_ref.record_type,
                &source_ref.record_id,
                &source_ref.observed_at,
                &source_ref.adapter_version,
            ]
            .iter()
            .all(|field| !field.trim().is_empty())
        });
    if action.source_refs.is_empty() {
        reasons.push("missing_source_refs".to_owned());
    } else if !source_refs_are_complete {
        reasons.push("incomplete_source_ref".to_owned());
    }

    if source_refs_are_complete {
        match packet {
            Some(packet) => {
                let matching_action = packet.actions().iter().find(|packet_action| {
                    brief_action_kind_code(packet_action.kind()) == action.kind
                        && packet_action.id().clone().into_inner() == action.id
                });
                match matching_action {
                    Some(packet_action)
                        if action.source_refs
                            != manager_daily_brief_action_source_refs(
                                packet_action,
                                packet.operating_day(),
                            ) =>
                    {
                        reasons.push("source_refs_do_not_match_action".to_owned());
                    }
                    None => reasons.push("action_not_present_in_context_packet".to_owned()),
                    Some(_) => {}
                }
            }
            None => reasons.push("invalid_context_packet_id".to_owned()),
        }
    }

    if !action
        .review_gates
        .iter()
        .any(|gate| gate == required_review_gate)
    {
        reasons.push(format!(
            "missing_required_review_gate:{required_review_gate}"
        ));
    }

    for side_effect in &action.requested_side_effects {
        reasons.push(manager_daily_brief_requested_side_effect_rejection_reason(
            side_effect,
        ));
    }

    reasons
}

fn required_manager_daily_brief_review_gate(kind: &str) -> Option<&'static str> {
    match kind {
        "review_demand_against_staffing_plan" => Some("manager_approval"),
        "resolve_checkout_exception" => Some("manager_approval"),
        "approve_retention_follow_up_draft" => Some("customer_message_approval"),
        "investigate_source_data_quality_issue" => Some("manager_approval"),
        _ => None,
    }
}

fn manager_daily_brief_side_effect_is_blocked(side_effect: &str) -> bool {
    matches!(
        side_effect,
        "send_customer_message"
            | "mutate_provider_or_pms_record"
            | "change_staff_schedule"
            | "move_refund_discount_or_payment"
            | "hide_source_data_quality_issue"
    )
}

fn manager_daily_brief_requested_side_effect_rejection_reason(side_effect: &str) -> String {
    if manager_daily_brief_side_effect_is_blocked(side_effect) {
        format!("blocked_side_effect:{side_effect}")
    } else {
        format!("unsupported_side_effect:{side_effect}")
    }
}

fn manager_daily_brief_reporting_scope(
    reporting: &ManagerDailyBriefOutcomeReportingRequest,
) -> Option<(entities::LocationId, operations::operating_day::Date)> {
    let location_id = Uuid::parse_str(&reporting.location_id).ok()?;
    let operating_day = NaiveDate::parse_from_str(&reporting.operating_day, "%Y-%m-%d").ok()?;

    Some((
        entities::LocationId(location_id),
        operations::operating_day::Date::try_new(operating_day).ok()?,
    ))
}

fn manager_daily_brief_packet_from_context_id(
    context_packet_id: &str,
) -> Option<manager_daily_brief::Packet> {
    let scope = context_packet_id.strip_prefix("manager-daily-brief-context:")?;
    let (location_id, operating_day) = scope.rsplit_once(':')?;
    let location_id = entities::LocationId(Uuid::parse_str(location_id).ok()?);
    let operating_day = operations::operating_day::Date::try_new(
        NaiveDate::parse_from_str(operating_day, "%Y-%m-%d").ok()?,
    )
    .ok()?;
    Some(local_manager_daily_brief_packet(location_id, operating_day))
}

fn local_manager_daily_brief_packet(
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
) -> manager_daily_brief::Packet {
    manager_daily_brief::Workflow::evaluate(
        manager_daily_brief::Request::builder()
            .location_id(location_id)
            .operating_day(operating_day)
            .prepared_for(manager_daily_brief::ManagerBriefPersona::GeneralManager)
            .demand_attention_threshold(
                manager_daily_brief::DemandThresholdUnits::try_new(10)
                    .expect("static demand threshold is valid"),
            )
            .service_demand_facts(local_manager_daily_brief_service_demand_facts(
                location_id,
                operating_day,
            ))
            .checkout_packets(local_manager_daily_brief_checkout_packets(
                location_id,
                operating_day,
            ))
            .retention_packets(local_manager_daily_brief_retention_packets(
                location_id,
                operating_day,
            ))
            .build(),
    )
}

fn stored_manager_daily_brief_action_kind(
    kind: manager_daily_brief::BriefActionKind,
) -> storage::operations::ManagerDailyBriefActionKindCode {
    match kind {
        manager_daily_brief::BriefActionKind::ReviewDemandAgainstStaffingPlan => {
            storage::operations::ManagerDailyBriefActionKindCode::ReviewDemandAgainstStaffingPlan
        }
        manager_daily_brief::BriefActionKind::ResolveCheckoutException => {
            storage::operations::ManagerDailyBriefActionKindCode::ResolveCheckoutException
        }
        manager_daily_brief::BriefActionKind::ApproveRetentionFollowUpDraft => {
            storage::operations::ManagerDailyBriefActionKindCode::ApproveRetentionFollowUpDraft
        }
        manager_daily_brief::BriefActionKind::InvestigateSourceDataQualityIssue => {
            storage::operations::ManagerDailyBriefActionKindCode::InvestigateSourceDataQualityIssue
        }
        manager_daily_brief::BriefActionKind::ReviewCapacityLaborRecommendation => {
            storage::operations::ManagerDailyBriefActionKindCode::ReviewCapacityLaborRecommendation
        }
    }
}

fn stored_manager_daily_brief_persona(
    persona: manager_daily_brief::ManagerBriefPersona,
) -> storage::operations::ManagerDailyBriefPersonaCode {
    match persona {
        manager_daily_brief::ManagerBriefPersona::GeneralManager => {
            storage::operations::ManagerDailyBriefPersonaCode::GeneralManager
        }
        manager_daily_brief::ManagerBriefPersona::AssistantGeneralManager => {
            storage::operations::ManagerDailyBriefPersonaCode::AssistantGeneralManager
        }
        manager_daily_brief::ManagerBriefPersona::FrontDeskLead => {
            storage::operations::ManagerDailyBriefPersonaCode::FrontDeskLead
        }
        manager_daily_brief::ManagerBriefPersona::FrontDeskAgent => {
            storage::operations::ManagerDailyBriefPersonaCode::FrontDeskAgent
        }
    }
}

async fn manager_daily_brief_agent_context(
    Authenticated(authentication): Authenticated,
    Extension(request_trace): Extension<RequestTraceEvidence>,
    Query(query): Query<ManagerDailyBriefAgentContextQuery>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_read(
        &authentication,
        authentication::Read::OperationalContext,
        Some(query.location_id),
        None,
    ) {
        return PublicApiError::new(
            rejection.into(),
            ErrorContext::new(request_trace.request_id()),
        )
        .into_response();
    }
    let location_id = entities::LocationId(query.location_id);
    let operating_day = operations::operating_day::Date::try_new(query.operating_day)
        .expect("operating day date is always valid after query parsing");
    let service_demand_facts =
        local_manager_daily_brief_service_demand_facts(location_id, operating_day);
    let checkout_packets = local_manager_daily_brief_checkout_packets(location_id, operating_day);
    let retention_packets = local_manager_daily_brief_retention_packets(location_id, operating_day);

    let request = manager_daily_brief::Request::builder()
        .location_id(location_id)
        .operating_day(operating_day)
        .prepared_for(manager_daily_brief::ManagerBriefPersona::GeneralManager)
        .demand_attention_threshold(
            manager_daily_brief::DemandThresholdUnits::try_new(10)
                .expect("static demand threshold is valid"),
        )
        .service_demand_facts(service_demand_facts.clone())
        .checkout_packets(checkout_packets.clone())
        .retention_packets(retention_packets.clone())
        .build();
    let packet = manager_daily_brief::Workflow::evaluate(request);
    let mut data_quality_issues = service_demand_facts
        .iter()
        .flat_map(|fact| fact.data_quality_issues().iter())
        .map(data_quality_issue_payload)
        .collect::<Vec<_>>();

    if service_demand_facts.is_empty() {
        data_quality_issues.push(missing_context_issue_payload(
            "missing_service_demand_fact",
            "No source-grounded service demand fact exists for the requested location and operating day.",
        ));
    }
    if checkout_packets.is_empty() {
        data_quality_issues.push(missing_context_issue_payload(
            "missing_checkout_completion_packet",
            "No checkout/completion packet exists for the requested location and operating day.",
        ));
    }
    if retention_packets.is_empty() {
        data_quality_issues.push(missing_context_issue_payload(
            "missing_crm_retention_packet",
            "No CRM/retention packet exists for the requested location and operating day.",
        ));
    }

    let mut source_refs = Vec::new();
    source_refs.extend(
        service_demand_facts
            .iter()
            .flat_map(|fact| fact.source_record_refs())
            .map(source_record_ref_payload),
    );
    source_refs.extend(checkout_packets.iter().map(|scoped| {
        source_record_ref_payload(&source::RecordRef::from_provenance(
            scoped.packet().provenance(),
        ))
    }));
    source_refs.extend(
        retention_packets
            .iter()
            .flat_map(|scoped| scoped.packet().source_record_refs())
            .map(source_record_ref_payload),
    );

    let correlation_id = format!(
        "manager-daily-brief:{}:{}",
        query.location_id, query.operating_day
    );

    Json(json!({
        "api_contract": api_dto_contract_payload("manager_daily_brief"),
        "workflow": {
            "name": "manager_daily_brief",
            "version": "local-manager-daily-brief-context-v1"
        },
        "location_id": query.location_id.to_string(),
        "operating_day": query.operating_day.to_string(),
        "service_demand_facts": service_demand_facts.iter().map(service_demand_fact_payload).collect::<Vec<_>>(),
        "checkout_completion_exceptions": checkout_packets.iter().filter_map(checkout_exception_payload).collect::<Vec<_>>(),
        "crm_retention_opportunities": retention_packets.iter().filter_map(retention_opportunity_payload).collect::<Vec<_>>(),
        "manager_brief_actions": packet
            .actions()
            .iter()
            .map(|action| manager_brief_action_payload(action, packet.operating_day()))
            .collect::<Vec<_>>(),
        "data_quality_issues": data_quality_issues,
        "source_refs": source_refs,
        "allowed_agent_actions": packet.safe_agent_actions().iter().map(safe_agent_action_code).collect::<Vec<_>>(),
        "blocked_actions": packet.blocked_actions().iter().map(blocked_action_code).collect::<Vec<_>>(),
        "labor_impact": {
            "before_minutes": packet.before_minutes().get(),
            "after_minutes": packet.after_minutes().get(),
            "minutes_saved": packet.minutes_saved()
        },
        "audit": {
            "context_packet_id": format!("manager-daily-brief-context:{}:{}", query.location_id, query.operating_day),
            "correlation_id": correlation_id
        },
        "observability": workflow_observability_payload(&correlation_id, &request_trace)
    }))
    .into_response()
}

async fn upload_vaccine_document(
    State(state): State<VaccineDocumentState>,
    Authenticated(authentication): Authenticated,
    Json(request): Json<VaccineDocumentUploadRequest>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize(
        &authentication,
        authentication::Mutation::VaccineDocumentUpload,
        &request.uploaded_by_staff_id,
        local_manager_daily_brief_location_id().0,
    ) {
        return (
            rejection.status_code(),
            Json(authorization_error_payload(
                rejection,
                "vaccine_document_review",
                "accepted",
            )),
        )
            .into_response();
    }

    let mut store = state.store.lock().await;
    let document_id = Uuid::new_v4();
    let extraction_id = Uuid::new_v4();
    let vaccine_record_id = Uuid::new_v4();
    let review_packet_id = Uuid::new_v4();
    let filename = request.filename.trim().to_owned();
    let sha256 = format!("{:x}", Sha256::digest(request.content.as_bytes()));
    let storage_key = format!(
        "vaccine-documents/pets/{}/{document_id}/{filename}",
        request.pet_id
    );
    let extraction = extract_vaccine_evidence(extraction_id, document_id, &request.content);

    let document = DocumentRecord {
        id: document_id,
        pet_id: request.pet_id,
        customer_id: request.customer_id,
        classification: "vaccine_proof",
        source: "staff_upload",
        filename,
        mime_type: request.mime_type,
        content_length_bytes: request.content.len(),
        sha256,
        storage_bucket: "local-dev-vaccine-documents",
        storage_key,
        storage_version: "mvp-local-v1".to_owned(),
        virus_scan_status: "passed",
        pii_redaction_status: "pending",
        verification_status: "awaiting_review",
    };
    let vaccine_record = VaccineRecord {
        id: vaccine_record_id,
        pet_id: request.pet_id,
        source_document_id: document_id,
        vaccine_name: extraction.vaccine_name.clone(),
        status: "pending_review",
        effective_on: extraction.effective_on,
        expires_on: extraction.expires_on,
        review_gate: "medical_document_review",
    };
    let review_packet = ReviewPacket {
        id: review_packet_id,
        document_id,
        vaccine_record_id,
        gate: "medical_document_review",
        status: "ready_for_review",
        uncertainty: "medical_document_uncertainty_policy_requires_staff_approval",
    };
    let eligibility = PetEligibility {
        pet_id: request.pet_id,
        rabies_current: false,
        source_vaccine_record_id: Some(vaccine_record_id),
        status: "awaiting_medical_document_review",
    };

    store.documents.insert(document_id, document.clone());
    store.extractions.insert(document_id, extraction.clone());
    store
        .vaccine_records
        .insert(vaccine_record_id, vaccine_record.clone());
    store
        .review_packets
        .insert(review_packet_id, review_packet.clone());
    store
        .eligibility
        .insert(request.pet_id, eligibility.clone());
    store.audit_events.push(audit(
        "document.received",
        &request.uploaded_by_staff_id,
        "document",
        document_id,
        [("storage_bucket", document.storage_bucket.to_owned())],
    ));
    store.audit_events.push(audit(
        "vaccine_extraction.persisted",
        "vaccine-document-agent",
        "document",
        document_id,
        [("schema_version", extraction.schema_version.to_owned())],
    ));
    store.audit_events.push(audit(
        "vaccine_record.review_requested",
        "vaccine-document-agent",
        "vaccine_record",
        vaccine_record_id,
        [("review_packet_id", review_packet_id.to_string())],
    ));

    let payload = store.payload(document_id, vaccine_record_id, review_packet_id, None);
    (StatusCode::CREATED, Json(payload)).into_response()
}

async fn approve_vaccine_document(
    State(state): State<VaccineDocumentState>,
    Authenticated(authentication): Authenticated,
    Path(review_packet_id): Path<Uuid>,
    Json(request): Json<VaccineReviewDecisionRequest>,
) -> axum::response::Response {
    decide_vaccine_document(
        state,
        authentication,
        review_packet_id,
        request,
        VaccineReviewDecision::Approve,
    )
    .await
}

async fn reject_vaccine_document(
    State(state): State<VaccineDocumentState>,
    Authenticated(authentication): Authenticated,
    Path(review_packet_id): Path<Uuid>,
    Json(request): Json<VaccineReviewDecisionRequest>,
) -> axum::response::Response {
    decide_vaccine_document(
        state,
        authentication,
        review_packet_id,
        request,
        VaccineReviewDecision::Reject,
    )
    .await
}

async fn decide_vaccine_document(
    state: VaccineDocumentState,
    authentication: authentication::Context,
    review_packet_id: Uuid,
    request: VaccineReviewDecisionRequest,
    decision: VaccineReviewDecision,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize(
        &authentication,
        authentication::Mutation::VaccineReviewDecision,
        &request.reviewed_by_staff_id,
        local_manager_daily_brief_location_id().0,
    ) {
        return (
            rejection.status_code(),
            Json(authorization_error_payload(
                rejection,
                "vaccine_document_review",
                "accepted",
            )),
        )
            .into_response();
    }

    let evidence = VaccineReviewDecisionEvidence {
        reviewed_by_staff_id: request.reviewed_by_staff_id,
        decided_at: Utc::now().to_rfc3339(),
        reason: request.reason,
    };
    let mut store = state.store.lock().await;
    match store.apply_vaccine_review_decision(review_packet_id, decision, evidence) {
        Ok(payload) => (StatusCode::OK, Json(payload)).into_response(),
        Err(rejection) => vaccine_review_decision_error_response(&store, rejection),
    }
}

fn vaccine_review_decision_error_response(
    store: &VaccineDocumentStore,
    rejection: VaccineReviewDecisionRejection,
) -> axum::response::Response {
    match rejection {
        VaccineReviewDecisionRejection::PacketNotFound { review_packet_id } => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "api_contract": api_dto_contract_payload("vaccine_document_review"),
                "accepted": false,
                "error": {
                    "code": "vaccine_review_packet_not_found",
                    "message": "The vaccine review packet is unknown to this application-owned workflow store."
                },
                "review_packet_id": review_packet_id
            })),
        )
            .into_response(),
        VaccineReviewDecisionRejection::PacketAlreadyDecided {
            review_packet_id,
            existing,
            attempted,
        } => {
            let payload = store.payload_for_review_conflict(review_packet_id, existing, attempted);
            (StatusCode::CONFLICT, Json(payload)).into_response()
        }
        VaccineReviewDecisionRejection::BrokenWorkflowState {
            review_packet_id,
            code,
        } => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "api_contract": api_dto_contract_payload("vaccine_document_review"),
                "accepted": false,
                "error": {
                    "code": code,
                    "message": "The vaccine review packet cannot transition because its correlated workflow records are incomplete."
                },
                "review_packet_id": review_packet_id
            })),
        )
            .into_response(),
    }
}

impl VaccineDocumentStore {
    fn apply_vaccine_review_decision(
        &mut self,
        review_packet_id: Uuid,
        decision: VaccineReviewDecision,
        evidence: VaccineReviewDecisionEvidence,
    ) -> Result<VaccineDocumentWorkflowPayload, VaccineReviewDecisionRejection> {
        let packet = self
            .review_packets
            .get(&review_packet_id)
            .cloned()
            .ok_or(VaccineReviewDecisionRejection::PacketNotFound { review_packet_id })?;

        if let Some(existing) = VaccineReviewDecision::from_decided_status(packet.status) {
            return Err(VaccineReviewDecisionRejection::PacketAlreadyDecided {
                review_packet_id,
                existing,
                attempted: decision,
            });
        }

        let document_id = packet.document_id;
        let vaccine_record_id = packet.vaccine_record_id;
        let pet_id = self
            .vaccine_records
            .get(&vaccine_record_id)
            .map(|record| record.pet_id)
            .ok_or(VaccineReviewDecisionRejection::BrokenWorkflowState {
                review_packet_id,
                code: "vaccine_review_record_not_found",
            })?;
        self.documents.get(&document_id).ok_or(
            VaccineReviewDecisionRejection::BrokenWorkflowState {
                review_packet_id,
                code: "vaccine_review_document_not_found",
            },
        )?;
        self.extractions.get(&document_id).ok_or(
            VaccineReviewDecisionRejection::BrokenWorkflowState {
                review_packet_id,
                code: "vaccine_review_extraction_not_found",
            },
        )?;
        self.eligibility.get(&pet_id).ok_or(
            VaccineReviewDecisionRejection::BrokenWorkflowState {
                review_packet_id,
                code: "vaccine_review_eligibility_not_found",
            },
        )?;

        self.review_packets
            .get_mut(&review_packet_id)
            .ok_or(VaccineReviewDecisionRejection::PacketNotFound { review_packet_id })?
            .status = decision.status_code();
        self.documents
            .get_mut(&document_id)
            .ok_or(VaccineReviewDecisionRejection::BrokenWorkflowState {
                review_packet_id,
                code: "vaccine_review_document_not_found",
            })?
            .verification_status = decision.document_verification_status();
        self.vaccine_records
            .get_mut(&vaccine_record_id)
            .ok_or(VaccineReviewDecisionRejection::BrokenWorkflowState {
                review_packet_id,
                code: "vaccine_review_record_not_found",
            })?
            .status = decision.vaccine_record_status();
        let eligibility = decision.eligibility(pet_id, vaccine_record_id);
        self.eligibility.insert(pet_id, eligibility);

        let approval = ApprovalRecord {
            id: Uuid::new_v4(),
            review_packet_id,
            target_document_id: document_id,
            target_vaccine_record_id: vaccine_record_id,
            gate: "medical_document_review",
            status: decision.status_code(),
            decided_by_staff_id: evidence.reviewed_by_staff_id.clone(),
            decided_at: evidence.decided_at,
            reason: evidence.reason,
        };
        self.approvals.insert(approval.id, approval.clone());
        self.audit_events.push(audit(
            "approval.decision.recorded",
            &evidence.reviewed_by_staff_id,
            "approval",
            approval.id,
            [("status", approval.status.to_owned())],
        ));
        self.audit_events.push(audit(
            "pet.eligibility.updated",
            &evidence.reviewed_by_staff_id,
            "pet",
            pet_id,
            [(
                "rabies_current",
                matches!(decision, VaccineReviewDecision::Approve).to_string(),
            )],
        ));

        Ok(self.payload(
            document_id,
            vaccine_record_id,
            review_packet_id,
            Some(approval),
        ))
    }

    fn payload_for_review_conflict(
        &self,
        review_packet_id: Uuid,
        existing: VaccineReviewDecision,
        attempted: VaccineReviewDecision,
    ) -> Value {
        let Some(packet) = self.review_packets.get(&review_packet_id) else {
            return json!({
                "api_contract": api_dto_contract_payload("vaccine_document_review"),
                "accepted": false,
                "error": {"code": "vaccine_review_packet_not_found"},
                "review_packet_id": review_packet_id
            });
        };
        let workflow = self.payload(
            packet.document_id,
            packet.vaccine_record_id,
            review_packet_id,
            None,
        );
        json!({
            "api_contract": api_dto_contract_payload("vaccine_document_review"),
            "accepted": false,
            "error": {
                "code": "vaccine_review_packet_already_decided",
                "message": "A vaccine review packet accepts exactly one approve or reject decision."
            },
            "review_packet_id": review_packet_id,
            "existing_decision": existing.status_code(),
            "attempted_decision": attempted.status_code(),
            "document": workflow.document,
            "vaccine_record": workflow.vaccine_record,
            "review_packet": workflow.review_packet,
            "eligibility": workflow.eligibility,
            "approval": workflow.approval
        })
    }

    fn payload(
        &self,
        document_id: Uuid,
        vaccine_record_id: Uuid,
        review_packet_id: Uuid,
        approval: Option<ApprovalRecord>,
    ) -> VaccineDocumentWorkflowPayload {
        let document = self.documents.get(&document_id).expect("document").clone();
        let extraction = self
            .extractions
            .get(&document_id)
            .expect("extraction")
            .clone();
        let vaccine_record = self
            .vaccine_records
            .get(&vaccine_record_id)
            .expect("vaccine record")
            .clone();
        let review_packet = self
            .review_packets
            .get(&review_packet_id)
            .expect("review packet")
            .clone();
        let eligibility = self
            .eligibility
            .get(&vaccine_record.pet_id)
            .expect("eligibility")
            .clone();
        VaccineDocumentWorkflowPayload {
            api_contract: api_dto_contract("vaccine_document_review"),
            document,
            extraction,
            vaccine_record,
            review_packet,
            approval,
            eligibility,
            audit_events: self.audit_events.clone(),
        }
    }
}

fn build_inquiry_intake_record(request: InquirySubmissionRequest) -> InquiryIntakeRecord {
    let first_name = request
        .customer
        .full_name
        .split_whitespace()
        .next()
        .unwrap_or("there")
        .to_owned();
    let missing_info = if request.message.to_ascii_lowercase().contains("vaccine") {
        vec!["vaccine_records"]
    } else {
        vec!["requested_dates", "vaccine_records"]
    };
    let task_title = format!(
        "Collect missing info for {} / {} inquiry",
        request.customer.full_name, request.pet.name
    );
    let source_event_key = request.source_event_key.clone();
    let has_lead_response_fixture = request.source_system.is_some()
        || request.provider_model_path.is_some()
        || request.raw_payload_ref.is_some()
        || !request.contact_attempts.is_empty()
        || request.simulated_conversion.is_some();
    let source_system = request
        .source_system
        .clone()
        .unwrap_or_else(|| "local_inquiry_fixture".to_owned());
    let provider_model_path = request
        .provider_model_path
        .clone()
        .unwrap_or_else(|| "pet_resort_api::InquirySubmissionRequest".to_owned());
    let raw_payload_ref = request
        .raw_payload_ref
        .clone()
        .unwrap_or_else(|| format!("api://inquiries/{}", request.source_event_key));
    let received_at = request.received_at;
    let first_attempt = request.contact_attempts.first();
    let simulated_conversion = request.simulated_conversion.as_ref().map(|conversion| {
        json!({
            "reservation_id": conversion.reservation_id,
            "converted_at": conversion.converted_at,
            "attribution_source": conversion.attribution_source,
            "source_event_key": request.source_event_key
        })
    });
    let outcome_attribution = simulated_conversion.as_ref().map(|conversion| {
        json!({
            "review_status": "reviewed_simulated_outcome",
            "source_event_key": request.source_event_key,
            "converted_reservation_id": conversion["reservation_id"],
            "supports_value_claim": false,
            "value_claim_boundary": "synthetic conversion proves attribution semantics but not measured NVA value"
        })
    });

    InquiryIntakeRecord {
        api_contract: api_dto_contract("inquiry_intake"),
        event: InquiryEvent {
            event_type: "inquiry.received",
            source_event_key: request.source_event_key,
            location_id: request.location_id,
        },
        provenance: has_lead_response_fixture.then(|| {
            json!({
                "source_system": source_system,
                "provider_model_path": provider_model_path,
                "raw_payload_ref": raw_payload_ref,
                "received_at": received_at,
                "provider_payload_passthrough": false
            })
        }),
        data_quality: has_lead_response_fixture.then(|| {
            json!({
                "classification": "accepted_fixture_evidence",
                "quality_gate": "source_ref_present_provider_model_declared_no_raw_payload_passthrough",
                "duplicate_policy": "source_event_key_idempotency"
            })
        }),
        canonical_lead_event: has_lead_response_fixture.then(|| {
            json!({
                "event_type": "lead.website_form_submitted",
                "source_event_key": source_event_key,
                "stage": "waiting_on_customer",
                "service_intent": request.service,
                "relationship_check": "candidate_customer_contact_matches_inquiry_envelope"
            })
        }),
        workflow: has_lead_response_fixture.then(|| {
            json!({
                "identity_status": "candidate_match_from_contact_envelope",
                "consent_status": "reply_draft_requires_staff_review",
                "sla_status": if first_attempt.is_some() { "met_by_reviewed_draft_attempt" } else { "awaiting_reviewed_attempt" },
                "attempts": request.contact_attempts.iter().map(|attempt| json!({
                    "attempted_at": attempt.attempted_at,
                    "channel": attempt.channel,
                    "purpose": attempt.purpose,
                    "outcome": attempt.outcome,
                    "review_gate": "front_desk_staff_review",
                    "message_ref": attempt.message_ref
                })).collect::<Vec<_>>()
            })
        }),
        lead: ParsedInquiryLead {
            customer_name: request.customer.full_name,
            customer_email: request.customer.email,
            customer_phone: request.customer.phone,
            pet_name: request.pet.name,
            species: request.pet.species,
            service: request.service,
            requested_dates: request
                .requested_dates
                .map(|dates| ParsedInquiryDateWindow {
                    start: dates.start,
                    end: dates.end,
                }),
            original_message: request.message,
            missing_info,
            review_status: "needs_staff_review",
        },
        draft_reply: InquiryDraftReply {
            status: "draft_created",
            live_send_allowed: false,
            approval_gate: "staff approval required before customer reply",
            body: format!(
                "Thanks {first_name} — we received your inquiry. Could you send current vaccine records so our staff can review availability and next steps?"
            ),
        },
        review_packet: has_lead_response_fixture.then(|| {
            json!({
                "status": "ready_for_front_desk_review",
                "gate": "customer_message_approval",
                "live_send_allowed": false,
                "provider_write_allowed": false,
                "queue_send_capability_reachable": false,
                "message_ref": first_attempt.and_then(|attempt| attempt.message_ref.as_deref())
            })
        }),
        task: InquiryTask {
            kind: "missing_info_review",
            status: "open",
            title: task_title,
            review_gate: "front_desk_staff_review",
        },
        storage_projection: has_lead_response_fixture.then(|| {
            json!({
                "adapter": "in_memory_workflow_repository",
                "read_model": "staff_inquiry_review_queue",
                "preserves_source_provenance": true,
                "preserves_review_gate": true
            })
        }),
        api_response: has_lead_response_fixture.then(|| {
            json!({
                "safe_to_return_to_staff": true,
                "provider_payload_passthrough": false,
                "live_side_effects_enabled": false
            })
        }),
        simulated_conversion,
        outcome_attribution,
        replay: None,
        agent_runtime: "agent.inquiry-intake.fake_deterministic",
        policy_boundary: "draft_only_no_live_send_no_provider_write_no_booking_decision_without_staff_approval",
        audit_events: vec![
            InquiryAuditEvent {
                action: "inquiry.received.normalized",
                actor_kind: "workflow_event_normalizer",
                subject_key: source_event_key.clone(),
            },
            InquiryAuditEvent {
                action: "agent.inquiry-intake.fake_deterministic",
                actor_kind: "agent_runtime",
                subject_key: source_event_key.clone(),
            },
            InquiryAuditEvent {
                action: "message.draft.created",
                actor_kind: "agent_runtime",
                subject_key: source_event_key,
            },
        ],
    }
}

fn extract_vaccine_evidence(id: Uuid, document_id: Uuid, content: &str) -> VaccineExtractionRecord {
    let lowered = content.to_ascii_lowercase();
    let vaccine_name = if lowered.contains("rabies") {
        "Rabies"
    } else {
        "Unknown vaccine"
    };
    VaccineExtractionRecord {
        id,
        document_id,
        schema_version: "vaccine_extraction.v1",
        vaccine_name: vaccine_name.to_owned(),
        effective_on: NaiveDate::from_ymd_opt(2026, 1, 15).expect("fixture date valid"),
        expires_on: lowered
            .contains("2027")
            .then(|| NaiveDate::from_ymd_opt(2027, 1, 15).expect("fixture date valid")),
        confidence: if lowered.contains("expires") {
            0.78
        } else {
            0.42
        },
        uncertainty_policy: "medical_document_uncertainty_policy_requires_staff_review",
        auto_accept_threshold: 0.95,
        raw_text_ref: format!("local-dev-ocr://documents/{document_id}/redacted-text"),
    }
}

fn audit(
    action: &'static str,
    actor_id: &str,
    subject_kind: &'static str,
    subject_id: Uuid,
    metadata: impl IntoIterator<Item = (&'static str, String)>,
) -> AuditEvent {
    AuditEvent {
        action,
        actor_kind: "staff_or_agent",
        actor_id: actor_id.to_owned(),
        subject_kind,
        subject_id,
        metadata: metadata.into_iter().collect(),
    }
}

fn local_manager_daily_brief_service_demand_facts(
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
) -> Vec<analytics::service_demand::Fact> {
    if location_id == local_manager_daily_brief_location_id()
        && operating_day == local_manager_daily_brief_operating_day()
    {
        vec![
            analytics::service_demand::Fact::try_new(
                analytics::service_demand::Id::try_new("service-demand-42")
                    .expect("static service demand id is valid"),
                operations::operating_day::Key::new(
                    location_id,
                    operations::service_core::ServiceLine::Boarding,
                    operating_day,
                ),
                analytics::service_demand::DemandUnits::try_new(18)
                    .expect("static demand units are valid"),
                vec![source::RecordRef::from_provenance(
                    &manager_brief_source_provenance(),
                )],
                analytics::ProjectionVersion::try_new("local-manager-brief-v1")
                    .expect("static projection version is valid"),
                vec![data_quality::Issue::new(
                    data_quality::Kind::UnmappedServiceType,
                    data_quality::Severity::Warning,
                    manager_brief_source_provenance(),
                    source::Timestamp::try_new("2026-06-17T00:00:00Z")
                        .expect("static timestamp is valid"),
                    false,
                )],
            )
            .expect("fixture source refs make service demand fact valid"),
        ]
    } else {
        Vec::new()
    }
}

fn local_manager_daily_brief_checkout_packets(
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
) -> Vec<manager_daily_brief::ScopedCheckoutPacket> {
    if location_id == local_manager_daily_brief_location_id()
        && operating_day == local_manager_daily_brief_operating_day()
    {
        let packet = checkout_completion::Workflow::evaluate(
            checkout_completion::Request::builder()
                .reservation_id(local_manager_daily_brief_reservation_id())
                .source_provenance(manager_brief_source_provenance())
                .observed_source_status(source::reservation::Status::CheckedOut)
                .staff_handoff(open_manager_brief_staff_handoff())
                .build(),
        );
        vec![
            manager_daily_brief::ScopedCheckoutPacket::builder()
                .location_id(location_id)
                .operating_day(operating_day)
                .packet(packet)
                .build(),
        ]
    } else {
        Vec::new()
    }
}

fn local_manager_daily_brief_retention_packets(
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
) -> Vec<manager_daily_brief::ScopedRetentionPacket> {
    if location_id == local_manager_daily_brief_location_id()
        && operating_day == local_manager_daily_brief_operating_day()
    {
        let checkout_packet = checkout_completion::Workflow::evaluate(
            checkout_completion::Request::builder()
                .reservation_id(local_manager_daily_brief_reservation_id())
                .source_provenance(manager_brief_source_provenance())
                .observed_source_status(source::reservation::Status::CheckedOut)
                .staff_handoff(resolved_manager_brief_staff_handoff())
                .build(),
        );
        let packet = crm_retention::Workflow::evaluate(
            crm_retention::Request::builder()
                .reservation_id(local_manager_daily_brief_reservation_id())
                .customer_id(local_manager_daily_brief_customer_id())
                .checkout_packet(checkout_packet)
                .contact_permission(manager_brief_contact_permission())
                .opportunities(vec![manager_brief_retention_opportunity()])
                .build(),
        );
        vec![
            manager_daily_brief::ScopedRetentionPacket::builder()
                .location_id(location_id)
                .operating_day(operating_day)
                .packet(packet)
                .build(),
        ]
    } else {
        Vec::new()
    }
}

fn service_demand_fact_payload(fact: &analytics::service_demand::Fact) -> Value {
    json!({
        "kind": "service_demand_forecast",
        "service_line": "boarding",
        "demand_units": fact.demand_units().get(),
        "projection_version": fact.projection_version().as_str(),
        "data_quality_status": service_demand_data_quality_status_code(fact.data_quality_status()),
        "source_refs": fact.source_record_refs().iter().map(source_record_ref_payload).collect::<Vec<_>>()
    })
}

fn checkout_exception_payload(scoped: &manager_daily_brief::ScopedCheckoutPacket) -> Option<Value> {
    let packet = scoped.packet();
    if matches!(
        packet.completion_status(),
        checkout_completion::CompletionStatus::StaffVerifiedCheckout
    ) {
        return None;
    }
    Some(json!({
        "reservation_id": format!("{:?}", packet.reservation_id()),
        "completion_status": checkout_completion_status_code(packet.completion_status()),
        "required_review_gates": packet.required_review_gates().iter().map(review_gate_code).collect::<Vec<_>>(),
        "source_refs": [source_record_ref_payload(&source::RecordRef::from_provenance(packet.provenance()))]
    }))
}

fn retention_opportunity_payload(
    scoped: &manager_daily_brief::ScopedRetentionPacket,
) -> Option<Value> {
    let packet = scoped.packet();
    if !matches!(
        packet.eligibility(),
        crm_retention::FollowUpEligibility::Eligible { .. }
    ) {
        return None;
    }
    Some(json!({
        "reservation_id": format!("{:?}", packet.reservation_id()),
        "eligibility": "eligible",
        "required_review_gates": packet.required_review_gates().iter().map(review_gate_code).collect::<Vec<_>>(),
        "source_refs": packet.source_record_refs().iter().map(source_record_ref_payload).collect::<Vec<_>>()
    }))
}

fn manager_brief_action_payload(
    action: &manager_daily_brief::BriefAction,
    operating_day: operations::operating_day::Date,
) -> Value {
    json!({
        "id": action.id().clone().into_inner(),
        "kind": brief_action_kind_code(action.kind()),
        "priority": brief_action_priority_code(action.priority()),
        "owner_persona": manager_brief_persona_code(action.owner_persona()),
        "removed_manual_work": removed_manual_work_code(action.removed_manual_work()),
        "source_facts": action.source_facts().iter().map(source_fact_payload).collect::<Vec<_>>(),
        "source_refs": manager_daily_brief_action_source_refs(action, operating_day),
        "required_review_gates": action.required_review_gates().iter().map(review_gate_code).collect::<Vec<_>>(),
        "labor_impact": {
            "before_minutes": action.labor_impact().before_minutes().get(),
            "after_minutes": action.labor_impact().after_minutes().get(),
            "minutes_saved": action.labor_impact().minutes_saved()
        }
    })
}

fn source_fact_payload(fact: &manager_daily_brief::SourceFact) -> Value {
    json!({
        "kind": source_fact_kind_code(fact.kind()),
        "summary": fact.summary().clone().into_inner(),
        "source_refs": fact.source_record_refs().iter().map(source_record_ref_payload).collect::<Vec<_>>()
    })
}

fn data_quality_issue_payload(issue: &data_quality::Issue) -> Value {
    json!({
        "kind": data_quality_kind_code(&issue.kind()),
        "severity": data_quality_severity_code(issue.severity()),
        "workflow_blocking": issue.workflow_blocking(),
        "source_refs": [source_record_ref_payload(issue.source_record_ref())]
    })
}

fn data_quality_issue_contract(issue: &data_quality::Issue) -> public_contract::DataQualityIssue {
    public_contract::DataQualityIssue {
        kind: data_quality_kind_code(&issue.kind()).to_owned(),
        severity: data_quality_severity_code(issue.severity()).to_owned(),
        workflow_blocking: issue.workflow_blocking(),
        detail: None,
        source_refs: vec![source_record_ref_contract(
            issue.source_record_ref(),
            issue.provenance(),
        )],
    }
}

fn missing_context_issue_payload(kind: &'static str, detail: &'static str) -> Value {
    json!({
        "kind": kind,
        "severity": "warning",
        "workflow_blocking": false,
        "detail": detail,
        "source_refs": []
    })
}

fn source_record_ref_payload(record_ref: &source::RecordRef) -> Value {
    json!({
        "system": source_system_code(record_ref.system()),
        "record_id": record_ref.record_id().as_str()
    })
}

fn source_record_ref_contract(
    record_ref: &source::RecordRef,
    provenance: &source::Provenance,
) -> public_contract::SourceRecordRef {
    public_contract::SourceRecordRef {
        system: source_system_code(record_ref.system()).to_owned(),
        record_type: provenance.endpoint().as_str().to_owned(),
        record_id: record_ref.record_id().as_str().to_owned(),
        observed_at: provenance.pulled_at().get().to_rfc3339(),
        adapter_version: provenance.schema_version().as_str().to_owned(),
    }
}

fn manager_daily_brief_action_source_refs(
    action: &manager_daily_brief::BriefAction,
    operating_day: operations::operating_day::Date,
) -> Vec<public_contract::SourceRecordRef> {
    let mut source_refs = action
        .source_facts()
        .iter()
        .flat_map(|fact| {
            fact.source_record_refs().iter().map(move |record_ref| {
                public_contract::SourceRecordRef {
                    system: source_system_code(record_ref.system()).to_owned(),
                    record_type: source_fact_kind_code(fact.kind()).to_owned(),
                    record_id: record_ref.record_id().as_str().to_owned(),
                    observed_at: format!("{}T00:00:00Z", operating_day.get()),
                    adapter_version: "nva-local-manager-daily-brief-fixture-v1".to_owned(),
                }
            })
        })
        .collect::<Vec<_>>();
    source_refs.sort();
    source_refs.dedup();
    source_refs
}

fn local_data_quality_hygiene_packet(
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
) -> data_quality_hygiene::Packet {
    data_quality_hygiene::Workflow::evaluate(
        data_quality_hygiene::Request::builder()
            .location_id(location_id)
            .operating_day(operating_day)
            .prepared_for(data_quality_hygiene::HygienePersona::GeneralManager)
            .candidates(local_data_quality_hygiene_candidates())
            .build(),
    )
}

fn local_data_quality_hygiene_candidates() -> Vec<data_quality_hygiene::Candidate> {
    vec![
        data_quality_hygiene::Candidate::builder()
            .id(
                data_quality_hygiene::IssueRef::try_new("dq-vaccine-stale-42")
                    .expect("static issue ref is valid"),
            )
            .kind(data_quality_hygiene::CandidateKind::SourceFreshness)
            .issue(data_quality::Issue::new(
                data_quality::Kind::MissingVaccinationRecord,
                data_quality::Severity::Blocking,
                data_quality_hygiene_source_provenance(
                    "GET /vaccinations/{id}",
                    "vaccine-record-42",
                ),
                source::Timestamp::try_new("2026-06-17T09:00:00Z")
                    .expect("static timestamp is valid"),
                true,
            ))
            .source_record_refs(vec![source::RecordRef::from_provenance(
                &data_quality_hygiene_source_provenance(
                    "GET /vaccinations/{id}",
                    "vaccine-record-42",
                ),
            )])
            .source_freshness(data_quality_hygiene::SourceFreshness::Stale)
            .sensitivity(data_quality_hygiene::Sensitivity::VaccineEvidence)
            .build(),
        data_quality_hygiene::Candidate::builder()
            .id(
                data_quality_hygiene::IssueRef::try_new("dq-duplicate-customer-17")
                    .expect("static issue ref is valid"),
            )
            .kind(data_quality_hygiene::CandidateKind::DuplicateCandidate)
            .issue(data_quality::Issue::new(
                data_quality::Kind::DuplicateSourceRecord,
                data_quality::Severity::Warning,
                data_quality_hygiene_source_provenance(
                    "GET /customers/{id}",
                    "customer-duplicate-17",
                ),
                source::Timestamp::try_new("2026-06-17T09:05:00Z")
                    .expect("static timestamp is valid"),
                false,
            ))
            .source_record_refs(vec![source::RecordRef::from_provenance(
                &data_quality_hygiene_source_provenance(
                    "GET /customers/{id}",
                    "customer-duplicate-17",
                ),
            )])
            .source_freshness(data_quality_hygiene::SourceFreshness::Conflicting)
            .sensitivity(data_quality_hygiene::Sensitivity::StandardOperationalEvidence)
            .build(),
    ]
}

fn data_quality_hygiene_packet_payload(
    packet: &data_quality_hygiene::Packet,
    request_trace: &RequestTraceEvidence,
) -> public_contract::DataQualityHygieneContextResponse {
    let correlation_id = packet.correlation_id().as_str();
    public_contract::DataQualityHygieneContextResponse {
        api_contract: api_dto_contract("data_quality_hygiene"),
        workflow: public_contract::WorkflowDescriptor {
            name: packet.workflow().to_owned(),
            version: packet.schema_version().to_owned(),
        },
        location_id: packet.location_id().0.to_string(),
        operating_day: packet.operating_day().get().to_string(),
        prepared_for: data_quality_hygiene_persona_code(packet.prepared_for()).to_owned(),
        candidates: packet
            .candidates()
            .iter()
            .map(data_quality_hygiene_candidate_payload)
            .collect(),
        hygiene_actions: packet
            .actions()
            .iter()
            .map(|action| data_quality_hygiene_action_payload(packet, action))
            .collect(),
        allowed_agent_actions: packet
            .safe_agent_actions()
            .iter()
            .map(|action| data_quality_hygiene_safe_action_code(*action).to_owned())
            .collect(),
        blocked_actions: packet
            .blocked_actions()
            .iter()
            .map(|action| data_quality_hygiene_blocked_action_code(*action).to_owned())
            .collect(),
        labor_savings_estimate: public_contract::LaborSavingsEstimate {
            before_minutes: packet.before_minutes().get(),
            after_minutes: packet.after_minutes().get(),
            estimated_minutes_saved: packet.minutes_saved(),
        },
        live_side_effects_allowed: false,
        audit: public_contract::WorkflowAudit {
            context_packet_id: packet.context_packet_id().as_str().to_owned(),
            correlation_id: correlation_id.to_owned(),
            runtime: "agent.data-quality-hygiene.fake_deterministic".to_owned(),
        },
        observability: workflow_observability_payload(correlation_id, request_trace),
    }
}

fn data_quality_hygiene_candidate_payload(
    candidate: &data_quality_hygiene::Candidate,
) -> public_contract::DataQualityCandidate {
    public_contract::DataQualityCandidate {
        id: candidate.id().as_str().to_owned(),
        kind: data_quality_hygiene_candidate_kind_code(candidate.kind()).to_owned(),
        issue: data_quality_issue_contract(candidate.issue()),
        source_refs: candidate
            .source_record_refs()
            .iter()
            .map(|record_ref| {
                source_record_ref_contract(record_ref, candidate.issue().provenance())
            })
            .collect(),
        source_freshness: data_quality_hygiene_source_freshness_code(candidate.source_freshness())
            .to_owned(),
        sensitivity: data_quality_hygiene_sensitivity_code(candidate.sensitivity()).to_owned(),
    }
}

fn data_quality_hygiene_action_source_refs(
    packet: &data_quality_hygiene::Packet,
    action: &data_quality_hygiene::Action,
) -> Vec<public_contract::SourceRecordRef> {
    let mut source_refs = action
        .issue_refs()
        .iter()
        .filter_map(|issue_ref| {
            packet
                .candidates()
                .iter()
                .find(|candidate| candidate.id() == issue_ref)
        })
        .flat_map(|candidate| {
            candidate.source_record_refs().iter().map(|record_ref| {
                source_record_ref_contract(record_ref, candidate.issue().provenance())
            })
        })
        .collect::<Vec<_>>();
    source_refs.sort();
    source_refs.dedup();
    source_refs
}

fn data_quality_hygiene_action_payload(
    packet: &data_quality_hygiene::Packet,
    action: &data_quality_hygiene::Action,
) -> public_contract::DataQualityAction {
    public_contract::DataQualityAction {
        id: action.id().as_str().to_owned(),
        kind: data_quality_hygiene_action_kind_code(action.kind()).to_owned(),
        priority: data_quality_hygiene_action_priority_code(action.priority()).to_owned(),
        owner_persona: data_quality_hygiene_persona_code(action.owner_persona()).to_owned(),
        removed_manual_work: data_quality_hygiene_removed_manual_work_code(
            action.removed_manual_work(),
        )
        .to_owned(),
        rationale: action.rationale().clone().into_inner(),
        source_refs: data_quality_hygiene_action_source_refs(packet, action),
        issue_refs: action
            .issue_refs()
            .iter()
            .map(|issue_ref| issue_ref.as_str().to_owned())
            .collect(),
        review_gates: action
            .required_review_gates()
            .iter()
            .map(|gate| review_gate_code(gate).to_owned())
            .collect(),
        labor_impact: public_contract::LaborSavingsEstimate {
            before_minutes: action.labor_impact().before_minutes().get(),
            after_minutes: action.labor_impact().after_minutes().get(),
            estimated_minutes_saved: action.labor_impact().minutes_saved(),
        },
        live_side_effects_allowed: false,
    }
}

fn validate_data_quality_hygiene_submitted_action(
    packet: &data_quality_hygiene::Packet,
    action: &public_contract::DataQualityHygieneSubmittedAction,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if action.source_refs.is_empty() {
        reasons.push("missing_source_refs".to_owned());
    }
    if action.issue_refs.is_empty() {
        reasons.push("missing_data_quality_issue_refs".to_owned());
    }
    if action.attempted_ambiguity_resolution {
        reasons.push("attempted_ambiguity_hiding".to_owned());
    }
    for side_effect in &action.requested_side_effects {
        reasons.push(data_quality_hygiene_requested_side_effect_rejection_reason(
            side_effect,
        ));
    }
    let matching_action = packet.actions().iter().find(|packet_action| {
        packet_action.id().as_str() == action.action_id
            && data_quality_hygiene_action_kind_code(packet_action.kind()) == action.kind
    });
    match matching_action {
        Some(packet_action) => {
            let expected_source_refs =
                data_quality_hygiene_action_source_refs(packet, packet_action);
            let expected_issue_refs = packet_action
                .issue_refs()
                .iter()
                .map(|issue_ref| issue_ref.as_str().to_owned())
                .collect::<Vec<_>>();
            if action.source_refs != expected_source_refs {
                reasons.push("source_refs_do_not_match_action".to_owned());
            }
            if action.issue_refs != expected_issue_refs {
                reasons.push("issue_refs_do_not_match_action".to_owned());
            }
            let required_gates = packet_action
                .required_review_gates()
                .iter()
                .map(|gate| review_gate_code(gate).to_owned())
                .collect::<Vec<_>>();
            if required_gates != action.review_gates {
                reasons.push("wrong_review_gate".to_owned());
            }
        }
        None => reasons.push("unsupported_action_kind".to_owned()),
    }
    reasons.sort_unstable();
    reasons.dedup();
    reasons
}

fn data_quality_hygiene_requested_side_effect_rejection_reason(side_effect: &str) -> String {
    match side_effect.trim() {
        "send_customer_message"
        | "mutate_provider_or_pms_record"
        | "change_staff_schedule"
        | "move_refund_discount_or_payment"
        | "hide_or_auto_resolve_source_ambiguity"
        | "expose_quarantined_sensitive_payload" => "blocked_side_effect_requested".to_owned(),
        _ => "unsupported_side_effect_requested".to_owned(),
    }
}

fn data_quality_hygiene_blocked_action_codes() -> Vec<&'static str> {
    vec![
        "send_customer_message",
        "mutate_provider_or_pms_record",
        "change_staff_schedule",
        "move_refund_discount_or_payment",
        "hide_or_auto_resolve_source_ambiguity",
        "expose_quarantined_sensitive_payload",
    ]
}

fn stored_source_record_ref_from_payload(
    value: &public_contract::SourceRecordRef,
) -> storage::operations::StoredSourceRecordRef {
    storage::operations::StoredSourceRecordRef::builder()
        .system(value.system.clone())
        .record_type(value.record_type.clone())
        .record_id(value.record_id.clone())
        .observed_at(value.observed_at.clone())
        .adapter_version(value.adapter_version.clone())
        .build()
}

fn stored_data_quality_hygiene_action_kind(
    kind: data_quality_hygiene::ActionKind,
) -> storage::operations::DataQualityHygieneActionKindCode {
    use data_quality_hygiene::ActionKind as App;
    use storage::operations::DataQualityHygieneActionKindCode as Stored;
    match kind {
        App::InvestigateMissingSourceEvidence => Stored::InvestigateMissingSourceEvidence,
        App::ReconcileDuplicateCustomerOrPetCandidate => {
            Stored::ReconcileDuplicateCustomerOrPetCandidate
        }
        App::CompleteMissingPetOrCustomerProfileFields => {
            Stored::CompleteMissingPetOrCustomerProfileFields
        }
        App::ReviewStaleVaccinationSourceFreshness => Stored::ReviewStaleVaccinationSourceFreshness,
        App::NormalizeAmbiguousServiceLineNaming => Stored::NormalizeAmbiguousServiceLineNaming,
        App::ReviewCheckoutOrUnclosedReservationEvidence => {
            Stored::ReviewCheckoutOrUnclosedReservationEvidence
        }
        App::EscalateSensitiveOrQuarantinedPayload => Stored::EscalateSensitiveOrQuarantinedPayload,
        App::ReviewPaymentStateConflict => Stored::ReviewPaymentStateConflict,
    }
}

fn stored_data_quality_hygiene_persona(
    persona: data_quality_hygiene::HygienePersona,
) -> storage::operations::DataQualityHygienePersonaCode {
    use data_quality_hygiene::HygienePersona as App;
    use storage::operations::DataQualityHygienePersonaCode as Stored;
    match persona {
        App::GeneralManager => Stored::GeneralManager,
        App::AssistantGeneralManager => Stored::AssistantGeneralManager,
        App::FrontDeskLead => Stored::FrontDeskLead,
        App::FrontDeskAgent => Stored::FrontDeskAgent,
        App::RegionalOperator => Stored::RegionalOperator,
        App::OperationsAnalyst => Stored::OperationsAnalyst,
    }
}

fn local_data_quality_hygiene_location_id() -> entities::LocationId {
    local_manager_daily_brief_location_id()
}

fn local_data_quality_hygiene_operating_day() -> operations::operating_day::Date {
    local_manager_daily_brief_operating_day()
}

fn data_quality_hygiene_source_provenance(
    endpoint: &'static str,
    record_id: &'static str,
) -> source::Provenance {
    source::Provenance::builder()
        .system(source::System::Gingr)
        .endpoint(source::Endpoint::try_new(endpoint).expect("static endpoint is valid"))
        .record_id(source::record::Id::try_new(record_id).expect("static record id is valid"))
        .extraction_batch(
            source::ExtractionBatchId::try_new("dq-hygiene-batch-local")
                .expect("static batch id is valid"),
        )
        .pulled_at(
            source::Timestamp::try_new("2026-06-17T00:00:00Z").expect("static timestamp is valid"),
        )
        .request_scope(
            source::RequestScope::try_new("local-data-quality-hygiene-context")
                .expect("static request scope is valid"),
        )
        .schema_version(
            source::SchemaVersion::try_new("gingr-v0-readonly")
                .expect("static schema version is valid"),
        )
        .payload_hash(
            source::PayloadHash::try_new("sha256:dataqualityhygienefixture")
                .expect("static payload hash is valid"),
        )
        .raw_payload_ref(
            source::RawPayloadRef::try_new("fixtures/gingr/data-quality-hygiene.json")
                .expect("static raw payload ref is valid"),
        )
        .build()
}

fn data_quality_hygiene_action_kind_code(kind: data_quality_hygiene::ActionKind) -> &'static str {
    match kind {
        data_quality_hygiene::ActionKind::InvestigateMissingSourceEvidence => {
            "investigate_missing_source_evidence"
        }
        data_quality_hygiene::ActionKind::ReconcileDuplicateCustomerOrPetCandidate => {
            "reconcile_duplicate_customer_or_pet_candidate"
        }
        data_quality_hygiene::ActionKind::CompleteMissingPetOrCustomerProfileFields => {
            "complete_missing_pet_or_customer_profile_fields"
        }
        data_quality_hygiene::ActionKind::ReviewStaleVaccinationSourceFreshness => {
            "review_stale_vaccination_source_freshness"
        }
        data_quality_hygiene::ActionKind::NormalizeAmbiguousServiceLineNaming => {
            "normalize_ambiguous_service_line_naming"
        }
        data_quality_hygiene::ActionKind::ReviewCheckoutOrUnclosedReservationEvidence => {
            "review_checkout_or_unclosed_reservation_evidence"
        }
        data_quality_hygiene::ActionKind::EscalateSensitiveOrQuarantinedPayload => {
            "escalate_sensitive_or_quarantined_payload"
        }
        data_quality_hygiene::ActionKind::ReviewPaymentStateConflict => {
            "review_payment_state_conflict"
        }
    }
}

fn data_quality_hygiene_persona_code(
    persona: data_quality_hygiene::HygienePersona,
) -> &'static str {
    match persona {
        data_quality_hygiene::HygienePersona::GeneralManager => "general_manager",
        data_quality_hygiene::HygienePersona::AssistantGeneralManager => {
            "assistant_general_manager"
        }
        data_quality_hygiene::HygienePersona::FrontDeskLead => "front_desk_lead",
        data_quality_hygiene::HygienePersona::FrontDeskAgent => "front_desk_agent",
        data_quality_hygiene::HygienePersona::RegionalOperator => "regional_operator",
        data_quality_hygiene::HygienePersona::OperationsAnalyst => "operations_analyst",
    }
}

fn data_quality_hygiene_candidate_kind_code(
    kind: data_quality_hygiene::CandidateKind,
) -> &'static str {
    match kind {
        data_quality_hygiene::CandidateKind::SourceIssue => "source_issue",
        data_quality_hygiene::CandidateKind::DuplicateCandidate => "duplicate_candidate",
        data_quality_hygiene::CandidateKind::ProfileGap => "profile_gap",
        data_quality_hygiene::CandidateKind::ServiceLineMapping => "service_line_mapping",
        data_quality_hygiene::CandidateKind::SourceFreshness => "source_freshness",
    }
}

fn data_quality_hygiene_source_freshness_code(
    freshness: data_quality_hygiene::SourceFreshness,
) -> &'static str {
    match freshness {
        data_quality_hygiene::SourceFreshness::Current => "current",
        data_quality_hygiene::SourceFreshness::Stale => "stale",
        data_quality_hygiene::SourceFreshness::Conflicting => "conflicting",
        data_quality_hygiene::SourceFreshness::Missing => "missing",
    }
}

fn data_quality_hygiene_sensitivity_code(
    sensitivity: data_quality_hygiene::Sensitivity,
) -> &'static str {
    match sensitivity {
        data_quality_hygiene::Sensitivity::StandardOperationalEvidence => {
            "standard_operational_evidence"
        }
        data_quality_hygiene::Sensitivity::VaccineEvidence => "vaccine_evidence",
        data_quality_hygiene::Sensitivity::IncidentOrBehaviorEvidence => {
            "incident_or_behavior_evidence"
        }
        data_quality_hygiene::Sensitivity::PaymentEvidence => "payment_evidence",
        data_quality_hygiene::Sensitivity::QuarantinedSensitivePayload => {
            "quarantined_sensitive_payload"
        }
    }
}

fn data_quality_hygiene_action_priority_code(
    priority: data_quality_hygiene::ActionPriority,
) -> &'static str {
    match priority {
        data_quality_hygiene::ActionPriority::High => "high",
        data_quality_hygiene::ActionPriority::Medium => "medium",
        data_quality_hygiene::ActionPriority::Low => "low",
    }
}

fn data_quality_hygiene_removed_manual_work_code(
    work: data_quality_hygiene::RemovedManualWork,
) -> &'static str {
    match work {
        data_quality_hygiene::RemovedManualWork::MissingEvidenceInvestigation => {
            "missing_evidence_investigation"
        }
        data_quality_hygiene::RemovedManualWork::DuplicateCandidateReconciliation => {
            "duplicate_candidate_reconciliation"
        }
        data_quality_hygiene::RemovedManualWork::IncompleteProfileCleanupPreparation => {
            "incomplete_profile_cleanup_preparation"
        }
        data_quality_hygiene::RemovedManualWork::SourceFreshnessReview => "source_freshness_review",
        data_quality_hygiene::RemovedManualWork::ServiceLineNormalizationReview => {
            "service_line_normalization_review"
        }
        data_quality_hygiene::RemovedManualWork::CheckoutEvidenceReview => {
            "checkout_evidence_review"
        }
        data_quality_hygiene::RemovedManualWork::SensitivePayloadEscalation => {
            "sensitive_payload_escalation"
        }
    }
}

fn data_quality_hygiene_safe_action_code(
    action: data_quality_hygiene::SafeAgentAction,
) -> &'static str {
    match action {
        data_quality_hygiene::SafeAgentAction::SummarizeSourceEvidence => {
            "summarize_source_evidence"
        }
        data_quality_hygiene::SafeAgentAction::RankHygieneActions => "rank_hygiene_actions",
        data_quality_hygiene::SafeAgentAction::DraftInternalCleanupTask => {
            "draft_internal_cleanup_task"
        }
        data_quality_hygiene::SafeAgentAction::PreserveAmbiguityForReview => {
            "preserve_ambiguity_for_review"
        }
        data_quality_hygiene::SafeAgentAction::EstimateReconciliationMinutesSaved => {
            "estimate_reconciliation_minutes_saved"
        }
    }
}

fn data_quality_hygiene_blocked_action_code(
    action: data_quality_hygiene::BlockedAction,
) -> &'static str {
    match action {
        data_quality_hygiene::BlockedAction::SendCustomerMessage => "send_customer_message",
        data_quality_hygiene::BlockedAction::MutateProviderOrPmsRecord => {
            "mutate_provider_or_pms_record"
        }
        data_quality_hygiene::BlockedAction::ChangeStaffSchedule => "change_staff_schedule",
        data_quality_hygiene::BlockedAction::MoveRefundDiscountOrPayment => {
            "move_refund_discount_or_payment"
        }
        data_quality_hygiene::BlockedAction::HideOrAutoResolveSourceAmbiguity => {
            "hide_or_auto_resolve_source_ambiguity"
        }
        data_quality_hygiene::BlockedAction::ExposeQuarantinedSensitivePayload => {
            "expose_quarantined_sensitive_payload"
        }
    }
}

fn local_manager_daily_brief_location_id() -> entities::LocationId {
    entities::LocationId(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0001))
}

fn local_manager_daily_brief_customer_id() -> entities::CustomerId {
    entities::CustomerId(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0099))
}

fn local_manager_daily_brief_reservation_id() -> entities::reservation::Id {
    entities::reservation::Id(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0042))
}

fn local_manager_daily_brief_operating_day() -> operations::operating_day::Date {
    operations::operating_day::Date::try_new(
        NaiveDate::from_ymd_opt(2026, 6, 17).expect("fixture operating day is valid"),
    )
    .expect("fixture operating day is valid")
}

fn open_manager_brief_staff_handoff() -> checkout_completion::StaffHandoff {
    checkout_completion::StaffHandoff::builder()
        .completed_by(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("front-desk-erin")
                .expect("static staff id is valid"),
        })
        .completed_at(DateTime::<Utc>::UNIX_EPOCH)
        .belongings_status(checkout_completion::BelongingsStatus::NeedsStaffFollowUp)
        .care_summary(
            checkout_completion::CareSummary::try_new("Medication bag needs review.")
                .expect("static care summary is valid"),
        )
        .departure_notes_review(checkout_completion::DepartureNotesReview::ManagerReviewRequired)
        .build()
}

fn resolved_manager_brief_staff_handoff() -> checkout_completion::StaffHandoff {
    checkout_completion::StaffHandoff::builder()
        .completed_by(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("front-desk-erin")
                .expect("static staff id is valid"),
        })
        .completed_at(DateTime::<Utc>::UNIX_EPOCH)
        .belongings_status(checkout_completion::BelongingsStatus::ReturnedToCustomer)
        .care_summary(
            checkout_completion::CareSummary::try_new("Clean checkout.")
                .expect("static care summary is valid"),
        )
        .departure_notes_review(checkout_completion::DepartureNotesReview::StaffReviewed)
        .build()
}

fn manager_brief_retention_opportunity() -> crm_retention::RetentionOpportunity {
    crm_retention::RetentionOpportunity::builder()
        .kind(crm_retention::OpportunityKind::NextBoardingStay)
        .evidence(
            crm_retention::OpportunityEvidence::builder()
                .reason_code(crm_retention::SourceGroundedReasonCode::CompletedBoardingStay)
                .summary(
                    crm_retention::EvidenceSummary::try_new(
                        "Completed boarding stay and owner mentioned a return trip.",
                    )
                    .expect("static evidence summary is valid"),
                )
                .provenance(manager_brief_source_provenance())
                .build(),
        )
        .build()
}

fn manager_brief_contact_permission() -> crm_retention::ContactPermission {
    crm_retention::ContactPermission::builder()
        .preferred_channel(message::Channel::Email)
        .allowed_channels(vec![message::Channel::Email])
        .marketing_consent(crm_retention::ConsentStatus::Granted)
        .transactional_consent(crm_retention::ConsentStatus::Granted)
        .source_record_refs(vec![source::RecordRef::from_provenance(
            &manager_brief_contact_provenance(),
        )])
        .build()
}

fn manager_brief_source_provenance() -> source::Provenance {
    source::Provenance::builder()
        .system(source::System::Gingr)
        .endpoint(
            source::Endpoint::try_new("GET /reservations/{id}").expect("static endpoint is valid"),
        )
        .record_id(
            source::record::Id::try_new("reservation-42").expect("static record id is valid"),
        )
        .extraction_batch(
            source::ExtractionBatchId::try_new("manager-brief-batch-local")
                .expect("static batch id is valid"),
        )
        .pulled_at(
            source::Timestamp::try_new("2026-06-17T00:00:00Z").expect("static timestamp is valid"),
        )
        .request_scope(
            source::RequestScope::try_new("local-manager-daily-brief-context")
                .expect("static request scope is valid"),
        )
        .schema_version(
            source::SchemaVersion::try_new("gingr-v0-readonly")
                .expect("static schema version is valid"),
        )
        .payload_hash(
            source::PayloadHash::try_new("sha256:managerbrieffixture")
                .expect("static payload hash is valid"),
        )
        .raw_payload_ref(
            source::RawPayloadRef::try_new("fixtures/gingr/manager-brief.json")
                .expect("static raw payload ref is valid"),
        )
        .build()
}

fn manager_brief_contact_provenance() -> source::Provenance {
    source::Provenance::builder()
        .system(source::System::Gingr)
        .endpoint(
            source::Endpoint::try_new("GET /customers/{id}/contact-permissions")
                .expect("static endpoint is valid"),
        )
        .record_id(
            source::record::Id::try_new("customer-contact-99").expect("static record id is valid"),
        )
        .extraction_batch(
            source::ExtractionBatchId::try_new("manager-brief-batch-local")
                .expect("static batch id is valid"),
        )
        .pulled_at(
            source::Timestamp::try_new("2026-06-17T00:00:00Z").expect("static timestamp is valid"),
        )
        .request_scope(
            source::RequestScope::try_new("local-manager-daily-brief-context")
                .expect("static request scope is valid"),
        )
        .schema_version(
            source::SchemaVersion::try_new("gingr-v0-readonly")
                .expect("static schema version is valid"),
        )
        .payload_hash(
            source::PayloadHash::try_new("sha256:managerbriefcontactfixture")
                .expect("static payload hash is valid"),
        )
        .raw_payload_ref(
            source::RawPayloadRef::try_new("fixtures/gingr/manager-brief-contact.json")
                .expect("static raw payload ref is valid"),
        )
        .build()
}

fn source_system_code(system: source::System) -> &'static str {
    match system {
        source::System::Gingr => "gingr",
        source::System::Telephony => "telephony",
        source::System::SmsProvider => "sms_provider",
        source::System::Email => "email",
        source::System::WebChat => "web_chat",
        source::System::WebsiteForms => "website_forms",
        source::System::MarketingAutomation => "marketing_automation",
        source::System::Crm => "crm",
        source::System::FinanceAccounting => "finance_accounting",
        source::System::WorkforceManagement => "workforce_management",
        source::System::KnowledgeBase => "knowledge_base",
        source::System::ProviderOrPms => "provider_or_pms",
        source::System::BusinessIntelligence => "business_intelligence",
        source::System::LaborScheduling => "labor_scheduling",
        source::System::Timeclock => "timeclock",
        source::System::Payroll => "payroll",
        source::System::CapacityInventory => "capacity_inventory",
        source::System::PointOfSale => "point_of_sale",
        source::System::ManualImport => "manual_import",
    }
}

fn service_demand_data_quality_status_code(
    status: analytics::service_demand::DataQualityStatus,
) -> &'static str {
    match status {
        analytics::service_demand::DataQualityStatus::Complete => "complete",
        analytics::service_demand::DataQualityStatus::ManagerReviewRequired => {
            "manager_review_required"
        }
    }
}

fn checkout_completion_status_code(status: checkout_completion::CompletionStatus) -> &'static str {
    match status {
        checkout_completion::CompletionStatus::StaffVerifiedCheckout => "staff_verified_checkout",
        checkout_completion::CompletionStatus::NeedsStaffHandoffReview => {
            "needs_staff_handoff_review"
        }
        checkout_completion::CompletionStatus::SourceNotCheckedOut => "source_not_checked_out",
    }
}

fn review_gate_code(gate: &policy::ReviewGate) -> &'static str {
    match gate {
        policy::ReviewGate::ManagerApproval => "manager_approval",
        policy::ReviewGate::CustomerMessageApproval => "customer_message_approval",
        policy::ReviewGate::MedicalDocumentReview => "medical_document_review",
        policy::ReviewGate::BehaviorReview => "behavior_review",
        policy::ReviewGate::RefundOrDepositException => "refund_or_deposit_exception",
    }
}

fn safe_agent_action_code(action: &manager_daily_brief::SafeAgentAction) -> &'static str {
    match action {
        manager_daily_brief::SafeAgentAction::SummarizeSourceEvidence => {
            "summarize_source_evidence"
        }
        manager_daily_brief::SafeAgentAction::RankManagerActions => "rank_manager_actions",
        manager_daily_brief::SafeAgentAction::DraftInternalTaskForReview => "draft_internal_tasks",
        manager_daily_brief::SafeAgentAction::RecordManagerFeedback => "record_manager_feedback",
        manager_daily_brief::SafeAgentAction::EstimateLaborMinutesSaved => {
            "estimate_labor_minutes_saved"
        }
    }
}

fn manager_daily_brief_blocked_action_codes() -> Vec<&'static str> {
    manager_daily_brief::Workflow::evaluate(
        manager_daily_brief::Request::builder()
            .location_id(entities::LocationId(Uuid::nil()))
            .operating_day(
                operations::operating_day::Date::try_new(
                    NaiveDate::from_ymd_opt(2026, 1, 1).expect("static date is valid"),
                )
                .expect("static operating day is valid"),
            )
            .prepared_for(manager_daily_brief::ManagerBriefPersona::GeneralManager)
            .demand_attention_threshold(
                manager_daily_brief::DemandThresholdUnits::try_new(1)
                    .expect("static demand threshold is valid"),
            )
            .build(),
    )
    .blocked_actions()
    .iter()
    .map(blocked_action_code)
    .collect()
}

fn blocked_action_code(action: &manager_daily_brief::BlockedAction) -> &'static str {
    match action {
        manager_daily_brief::BlockedAction::ChangeStaffSchedule => "change_staff_schedule",
        manager_daily_brief::BlockedAction::MutateProviderOrPmsRecord => {
            "mutate_provider_or_pms_record"
        }
        manager_daily_brief::BlockedAction::SendCustomerMessage => "send_customer_message",
        manager_daily_brief::BlockedAction::MoveRefundDiscountOrPayment => {
            "move_refund_discount_or_payment"
        }
        manager_daily_brief::BlockedAction::HideSourceDataQualityIssue => {
            "hide_source_data_quality_issue"
        }
    }
}

fn brief_action_kind_code(kind: manager_daily_brief::BriefActionKind) -> &'static str {
    match kind {
        manager_daily_brief::BriefActionKind::ReviewDemandAgainstStaffingPlan => {
            "review_demand_against_staffing_plan"
        }
        manager_daily_brief::BriefActionKind::ResolveCheckoutException => {
            "resolve_checkout_exception"
        }
        manager_daily_brief::BriefActionKind::ApproveRetentionFollowUpDraft => {
            "approve_retention_follow_up_draft"
        }
        manager_daily_brief::BriefActionKind::InvestigateSourceDataQualityIssue => {
            "investigate_source_data_quality_issue"
        }
        manager_daily_brief::BriefActionKind::ReviewCapacityLaborRecommendation => {
            "review_capacity_labor_recommendation"
        }
    }
}

fn brief_action_priority_code(priority: manager_daily_brief::BriefActionPriority) -> &'static str {
    match priority {
        manager_daily_brief::BriefActionPriority::High => "high",
        manager_daily_brief::BriefActionPriority::Medium => "medium",
        manager_daily_brief::BriefActionPriority::Low => "low",
    }
}

fn manager_brief_persona_code(persona: manager_daily_brief::ManagerBriefPersona) -> &'static str {
    match persona {
        manager_daily_brief::ManagerBriefPersona::GeneralManager => "general_manager",
        manager_daily_brief::ManagerBriefPersona::AssistantGeneralManager => {
            "assistant_general_manager"
        }
        manager_daily_brief::ManagerBriefPersona::FrontDeskLead => "front_desk_lead",
        manager_daily_brief::ManagerBriefPersona::FrontDeskAgent => "front_desk_agent",
    }
}

fn removed_manual_work_code(work: manager_daily_brief::RemovedManualWork) -> &'static str {
    match work {
        manager_daily_brief::RemovedManualWork::MorningDashboardReconciliation => {
            "morning_dashboard_reconciliation"
        }
        manager_daily_brief::RemovedManualWork::DemandVersusStaffingScan => {
            "demand_versus_staffing_scan"
        }
        manager_daily_brief::RemovedManualWork::CheckoutExceptionAudit => {
            "checkout_exception_audit"
        }
        manager_daily_brief::RemovedManualWork::RetentionFollowUpQueuePrioritization => {
            "retention_follow_up_queue_prioritization"
        }
        manager_daily_brief::RemovedManualWork::DataQualityExceptionTriage => {
            "data_quality_exception_triage"
        }
        manager_daily_brief::RemovedManualWork::ServiceCapacityLaborPlanning => {
            "service_capacity_labor_planning"
        }
    }
}

fn source_fact_kind_code(kind: manager_daily_brief::SourceFactKind) -> &'static str {
    match kind {
        manager_daily_brief::SourceFactKind::ServiceDemandForecast => "service_demand_forecast",
        manager_daily_brief::SourceFactKind::CheckoutCompletionStatus => {
            "checkout_completion_status"
        }
        manager_daily_brief::SourceFactKind::RetentionFollowUpEligibility => {
            "retention_follow_up_eligibility"
        }
        manager_daily_brief::SourceFactKind::SourceDataQualityIssue => "source_data_quality_issue",
        manager_daily_brief::SourceFactKind::CapacityLaborRecommendation => {
            "capacity_labor_recommendation"
        }
    }
}

fn data_quality_kind_code(kind: &data_quality::Kind) -> &'static str {
    match kind {
        data_quality::Kind::MissingRequiredField { .. } => "missing_required_field",
        data_quality::Kind::AssumptionInForce { .. } => "assumption_in_force",
        data_quality::Kind::UnknownSourceStatus { .. } => "unknown_source_status",
        data_quality::Kind::ConflictingTimestamps => "conflicting_timestamps",
        data_quality::Kind::DuplicateSourceRecord => "duplicate_source_record",
        data_quality::Kind::AmbiguousOwnerPetRelationship => "ambiguous_owner_pet_relationship",
        data_quality::Kind::UnmappedServiceType => "unmapped_service_type",
        data_quality::Kind::LocationScopeAmbiguity => "location_scope_ambiguity",
        data_quality::Kind::PaymentStateConflict => "payment_state_conflict",
        data_quality::Kind::CheckoutEvidenceMissing => "checkout_evidence_missing",
        data_quality::Kind::UnclosedReservation => "unclosed_reservation",
        data_quality::Kind::IncompletePetProfile => "incomplete_pet_profile",
        data_quality::Kind::MissingVaccinationRecord => "missing_vaccination_record",
        data_quality::Kind::SensitivePayloadQuarantined => "sensitive_payload_quarantined",
    }
}

fn data_quality_severity_code(severity: data_quality::Severity) -> &'static str {
    match severity {
        data_quality::Severity::Informational => "informational",
        data_quality::Severity::Warning => "warning",
        data_quality::Severity::Blocking => "blocking",
        data_quality::Severity::Critical => "critical",
    }
}
