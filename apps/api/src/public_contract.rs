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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct DataQualityHygieneDraftSubmissionRequest {
    pub context_packet_id: String,
    pub correlation_id: String,
    pub actions: Vec<DataQualityHygieneSubmittedAction>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataQualityHygieneOutcome {
    Completed,
    Deferred,
    SuppressedByManager,
    SourceFactWasWrong,
    NotActionable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataQualityHygienePersona {
    GeneralManager,
    AssistantGeneralManager,
    FrontDeskLead,
    FrontDeskAgent,
    RegionalOperator,
    OperationsAnalyst,
}

impl DataQualityHygienePersona {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GeneralManager => "general_manager",
            Self::AssistantGeneralManager => "assistant_general_manager",
            Self::FrontDeskLead => "front_desk_lead",
            Self::FrontDeskAgent => "front_desk_agent",
            Self::RegionalOperator => "regional_operator",
            Self::OperationsAnalyst => "operations_analyst",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataQualityResolutionStatus {
    Open,
    Acknowledged,
    Ignored,
    Repaired,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DataQualityHygieneOutcomeActor {
    id: String,
    persona: DataQualityHygienePersona,
    actor_role: DataQualityHygienePersona,
}

impl DataQualityHygieneOutcomeActor {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn persona(&self) -> DataQualityHygienePersona {
        self.persona
    }

    pub const fn actor_role(&self) -> DataQualityHygienePersona {
        self.actor_role
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OutcomeAudit {
    correlation_id: String,
}

impl OutcomeAudit {
    pub fn correlation_id(&self) -> &str {
        &self.correlation_id
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub fn try_new(raw: impl Into<String>) -> Result<Self, IdempotencyKeyError> {
        let raw = raw.into();
        if raw.is_empty() {
            return Err(IdempotencyKeyError::Empty);
        }
        if raw.len() > 128 {
            return Err(IdempotencyKeyError::TooLong);
        }
        if !raw
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
        {
            return Err(IdempotencyKeyError::InvalidCharacter);
        }
        Ok(Self(raw))
    }

    pub(crate) fn expose_for_fingerprint(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for IdempotencyKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("IdempotencyKey([REDACTED])")
    }
}

impl Serialize for IdempotencyKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for IdempotencyKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::try_new(raw).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdempotencyKeyError {
    Empty,
    TooLong,
    InvalidCharacter,
}

impl fmt::Display for IdempotencyKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "idempotency key must not be empty",
            Self::TooLong => "idempotency key must not exceed 128 bytes",
            Self::InvalidCharacter => "idempotency key contains an unsupported character",
        })
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DataQualityHygieneOutcomeCaptureRequest {
    outcome: DataQualityHygieneOutcome,
    actual_minutes: u16,
    actor: DataQualityHygieneOutcomeActor,
    feedback: String,
    source_refs: Vec<SourceRecordRef>,
    issue_refs: Vec<String>,
    resolution_status_after_review: DataQualityResolutionStatus,
    timestamp: String,
    audit: OutcomeAudit,
    requested_side_effects: Vec<String>,
    idempotency_key: IdempotencyKey,
}

impl DataQualityHygieneOutcomeCaptureRequest {
    pub const fn outcome(&self) -> DataQualityHygieneOutcome {
        self.outcome
    }

    pub const fn actual_minutes(&self) -> u16 {
        self.actual_minutes
    }

    pub fn actor(&self) -> &DataQualityHygieneOutcomeActor {
        &self.actor
    }

    pub fn feedback(&self) -> &str {
        &self.feedback
    }

    pub fn source_refs(&self) -> &[SourceRecordRef] {
        &self.source_refs
    }

    pub fn issue_refs(&self) -> &[String] {
        &self.issue_refs
    }

    pub const fn resolution_status_after_review(&self) -> DataQualityResolutionStatus {
        self.resolution_status_after_review
    }

    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    pub fn audit(&self) -> &OutcomeAudit {
        &self.audit
    }

    pub fn requested_side_effects(&self) -> &[String] {
        &self.requested_side_effects
    }

    pub fn idempotency_key(&self) -> &IdempotencyKey {
        &self.idempotency_key
    }
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
