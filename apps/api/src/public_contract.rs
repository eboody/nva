//! Product-owned API schema contracts for NVA Pet Resorts operations v0.
//!
//! These DTOs are the stable public boundary for the replacement API. Provider
//! payloads remain quarantined source evidence and never become public resources.

#![allow(missing_docs)]

use core::fmt;

use serde::{Deserialize, Serialize};

pub const OWNED_OPERATIONS_API_VERSION: &str = "pet_resort_api.runtime.v0";
pub const OWNED_OPERATIONS_API_BOUNDARY: &str = "api_runtime_dto";
pub const OWNED_OPERATIONS_API_OWNER: &str = "pet_resort_api";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderBoundaryMode {
    EvidenceRefsOnly,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveSideEffectsMode {
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiContractMetadata {
    pub owner: String,
    pub boundary: String,
    pub schema_version: String,
    pub workflow: String,
    pub provider_boundary: ProviderBoundaryMode,
    pub live_side_effects: LiveSideEffectsMode,
}

impl ApiContractMetadata {
    pub fn operations_v0(workflow: impl Into<String>) -> Self {
        Self {
            owner: OWNED_OPERATIONS_API_OWNER.to_owned(),
            boundary: OWNED_OPERATIONS_API_BOUNDARY.to_owned(),
            schema_version: OWNED_OPERATIONS_API_VERSION.to_owned(),
            workflow: workflow.into(),
            provider_boundary: ProviderBoundaryMode::EvidenceRefsOnly,
            live_side_effects: LiveSideEffectsMode::Disabled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestMetadata {
    pub request_id: String,
    pub correlation_id: Option<String>,
    pub payload_logging: PayloadLogging,
    pub actor: Option<ActorRef>,
    pub location_id: Option<String>,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PayloadLogging {
    Disabled,
    RedactedSummaryOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActorRef {
    #[serde(alias = "actor_kind")]
    pub persona: String,
    #[serde(alias = "actor_id")]
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceRef {
    pub source_system: String,
    pub external_record_ref: String,
    pub observed_at: Option<String>,
    pub adapter_version: Option<String>,
    pub source_visibility: Option<String>,
}

/// Stable source-record evidence shape used by the v0 workflow routes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceRecordRef {
    pub system: String,
    pub record_type: String,
    pub record_id: String,
    pub observed_at: String,
    pub adapter_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewGateRef {
    pub gate: String,
    pub required: bool,
    pub reviewer_role: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockedAction {
    pub action: String,
    pub blocked_reason: String,
    pub review_gate: Option<ReviewGateRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditRef {
    pub audit_event_id: String,
    pub event_name: String,
    pub workflow_event_id: Option<String>,
    pub review_packet_id: Option<String>,
    pub approval_record_id: Option<String>,
    pub outbox_record_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorEnvelope {
    pub error: ApiError,
    pub request_id: String,
    pub correlation_id: Option<String>,
    pub live_side_effects: LiveSideEffectsMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub safe_error_class: String,
    pub details: Vec<ErrorDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorDetail {
    pub field: String,
    pub reason: String,
}

impl ErrorDetail {
    pub fn field(field: String, reason: String) -> Self {
        Self { field, reason }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowDescriptor {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaborSavingsEstimate {
    pub before_minutes: u16,
    pub after_minutes: u16,
    pub estimated_minutes_saved: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowAudit {
    pub context_packet_id: String,
    pub correlation_id: String,
    pub runtime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowObservability {
    pub correlation_id: String,
    pub request_id: String,
    pub request_correlation_id: String,
    pub route_status_trace: String,
    pub safe_error_class: String,
    pub payload_logging: String,
    pub sensitive_payload_logging: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataQualityIssue {
    pub kind: String,
    pub severity: String,
    pub workflow_blocking: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default)]
    pub source_refs: Vec<SourceRecordRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataQualityCandidate {
    pub id: String,
    pub kind: String,
    pub issue: DataQualityIssue,
    pub source_refs: Vec<SourceRecordRef>,
    pub source_freshness: String,
    pub sensitivity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataQualityAction {
    pub id: String,
    pub kind: String,
    pub priority: String,
    pub owner_persona: String,
    pub removed_manual_work: String,
    pub rationale: String,
    pub source_refs: Vec<SourceRecordRef>,
    pub issue_refs: Vec<String>,
    pub review_gates: Vec<String>,
    pub labor_impact: LaborSavingsEstimate,
    pub live_side_effects_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataQualityHygieneContextResponse {
    pub api_contract: ApiContractMetadata,
    pub workflow: WorkflowDescriptor,
    pub location_id: String,
    pub operating_day: String,
    pub prepared_for: String,
    pub candidates: Vec<DataQualityCandidate>,
    pub hygiene_actions: Vec<DataQualityAction>,
    pub labor_savings_estimate: LaborSavingsEstimate,
    pub allowed_agent_actions: Vec<String>,
    pub blocked_actions: Vec<String>,
    pub live_side_effects_allowed: bool,
    pub audit: WorkflowAudit,
    pub observability: WorkflowObservability,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataQualityHygieneSubmittedAction {
    pub action_id: String,
    pub kind: String,
    #[serde(default)]
    pub source_refs: Vec<SourceRecordRef>,
    #[serde(default)]
    pub issue_refs: Vec<String>,
    #[serde(default)]
    pub review_gates: Vec<String>,
    #[serde(default)]
    pub requested_side_effects: Vec<String>,
    #[serde(default)]
    pub attempted_ambiguity_resolution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataQualityHygieneDraftSubmissionRequest {
    pub context_packet_id: String,
    pub correlation_id: String,
    pub actions: Vec<DataQualityHygieneSubmittedAction>,
    pub idempotency_key: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataQualityHygieneOutcomeCaptureRequest {
    pub outcome: String,
    pub actual_minutes: u16,
    pub actor: ActorRef,
    pub feedback: String,
    #[serde(default)]
    pub source_refs: Vec<SourceRecordRef>,
    #[serde(default)]
    pub issue_refs: Vec<String>,
    pub resolution_status_after_review: String,
    pub timestamp: String,
    pub audit: OutcomeAudit,
    #[serde(default)]
    pub requested_side_effects: Vec<String>,
    pub idempotency_key: Option<String>,
}

impl fmt::Debug for DataQualityHygieneOutcomeCaptureRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DataQualityHygieneOutcomeCaptureRequest")
            .field("outcome", &self.outcome)
            .field("actual_minutes", &self.actual_minutes)
            .field(
                "resolution_status_after_review",
                &self.resolution_status_after_review,
            )
            .field("source_ref_count", &self.source_refs.len())
            .field("issue_ref_count", &self.issue_refs.len())
            .field(
                "requested_side_effect_count",
                &self.requested_side_effects.len(),
            )
            .field("sensitive_fields", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutcomeAudit {
    pub correlation_id: String,
}
