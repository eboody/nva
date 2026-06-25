//! Product-owned API schema contracts for NVA Pet Resorts operations v0.
//!
//! These DTOs are the stable public boundary for the replacement API. They are
//! intentionally named around NVA operations, review gates, labor outcomes, audit
//! refs, and BI/read-model needs. Provider/PMS payloads remain source evidence and
//! must not pass through this module as canonical API resources.

#![allow(missing_docs)]

use serde::Serialize;
use serde_json::Value;

pub const OWNED_OPERATIONS_API_VERSION: &str = "0.1.0";
pub const OWNED_OPERATIONS_API_BOUNDARY: &str = "owned_operations_api_v0";
pub const OWNED_OPERATIONS_API_OWNER: &str = "nva_pet_resorts_operations";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ApiContractMetadata {
    pub owner: &'static str,
    pub boundary: &'static str,
    pub version: &'static str,
    pub workflow: &'static str,
    pub provider_payload_passthrough: bool,
    pub provider_dto_boundary: &'static str,
    pub live_side_effects_allowed: bool,
}

impl ApiContractMetadata {
    pub fn operations_v0(workflow: &'static str) -> Self {
        Self {
            owner: OWNED_OPERATIONS_API_OWNER,
            boundary: OWNED_OPERATIONS_API_BOUNDARY,
            version: OWNED_OPERATIONS_API_VERSION,
            workflow,
            provider_payload_passthrough: false,
            provider_dto_boundary: "provider_evidence_refs_only",
            live_side_effects_allowed: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RequestMetadata {
    pub request_id: String,
    pub correlation_id: Option<String>,
    pub payload_logging: PayloadLogging,
    pub actor: Option<ActorRef>,
    pub location_id: Option<String>,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PayloadLogging {
    Disabled,
    RedactedSummaryOnly,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ActorRef {
    pub actor_kind: String,
    pub actor_id: Option<String>,
    pub actor_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SourceRef {
    pub source_system: String,
    pub external_record_ref: String,
    pub observed_at: Option<String>,
    pub adapter_version: Option<String>,
    pub source_visibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReviewGateRef {
    pub gate: String,
    pub required: bool,
    pub reviewer_role: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BlockedAction {
    pub action: String,
    pub blocked_reason: String,
    pub review_gate: Option<ReviewGateRef>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AuditRef {
    pub audit_event_id: String,
    pub event_name: String,
    pub workflow_event_id: Option<String>,
    pub review_packet_id: Option<String>,
    pub approval_record_id: Option<String>,
    pub outbox_record_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ErrorEnvelope {
    pub error: ApiError,
    pub request_id: String,
    pub correlation_id: Option<String>,
    pub live_side_effects_allowed: bool,
}

impl ErrorEnvelope {
    pub fn validation_failed(
        request_id: String,
        correlation_id: Option<String>,
        details: Vec<ErrorDetail>,
    ) -> Self {
        Self {
            error: ApiError {
                code: "workflow_validation_failed",
                message: "The request violates the owned workflow safety contract.",
                safe_error_class: "validation_failed",
                details,
            },
            request_id,
            correlation_id,
            live_side_effects_allowed: false,
        }
    }

    pub fn not_found(request_id: String, path: String) -> Self {
        Self {
            error: ApiError {
                code: "not_found",
                message: "The requested owned operations API route is not available.",
                safe_error_class: "not_found",
                details: vec![ErrorDetail::field("path".to_owned(), path)],
            },
            request_id,
            correlation_id: None,
            live_side_effects_allowed: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ApiError {
    pub code: &'static str,
    pub message: &'static str,
    pub safe_error_class: &'static str,
    pub details: Vec<ErrorDetail>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ErrorDetail {
    pub field: String,
    pub reason: String,
}

impl ErrorDetail {
    pub fn field(field: String, reason: String) -> Self {
        Self { field, reason }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DataQualityHygieneContextResponse {
    pub metadata: RequestMetadata,
    pub api_contract: ApiContractMetadata,
    pub workflow: Value,
    pub location_id: String,
    pub operating_day: String,
    pub prepared_for: String,
    pub candidates: Vec<Value>,
    pub hygiene_actions: Vec<Value>,
    pub labor_savings_estimate: Value,
    pub allowed_agent_actions: Vec<String>,
    pub blocked_actions: Vec<String>,
    pub audit: Value,
    pub live_side_effects_allowed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DataQualityHygieneDraftSubmissionRequest {
    pub context_packet_id: String,
    pub correlation_id: String,
    pub actions: Vec<Value>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DataQualityHygieneDraftSubmissionResponse {
    pub metadata: RequestMetadata,
    pub validation: Value,
    pub accepted_actions: Vec<Value>,
    pub rejected_actions: Vec<Value>,
    pub workflow_event_id: Option<String>,
    pub review_packet_id: Option<String>,
    pub outbox_candidate: Option<Value>,
    pub audit_refs: Vec<AuditRef>,
    pub live_side_effects_allowed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DataQualityHygieneOutcomeCaptureRequest {
    pub outcome: String,
    pub actual_minutes: u16,
    pub actor: ActorRef,
    pub feedback: String,
    pub source_refs: Vec<SourceRef>,
    pub issue_refs: Vec<String>,
    pub resolution_status_after_review: String,
    pub timestamp: String,
    pub audit: Value,
    pub requested_side_effects: Vec<String>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DataQualityHygieneOutcomeCaptureResponse {
    pub metadata: RequestMetadata,
    pub outcome_record: Value,
    pub audit_refs: Vec<AuditRef>,
    pub summary_ref: Option<String>,
    pub live_side_effects_allowed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DataQualityHygieneOutcomeSummaryResponse {
    pub metadata: RequestMetadata,
    pub filters: Value,
    pub summary: Value,
    pub source_refs: Vec<SourceRef>,
    pub issue_refs: Vec<String>,
    pub caveats: Vec<String>,
    pub live_side_effects_allowed: bool,
}
