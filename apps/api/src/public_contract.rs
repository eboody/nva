//! Product-owned API schema contracts for NVA Pet Resorts operations v1.
//!
//! These DTOs are the stable public boundary for the replacement API. Provider
//! payloads remain quarantined source evidence and never become public resources.

#![allow(missing_docs)]

use core::{fmt, num::NonZeroU16};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

fn deserialize_non_nil_uuid<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Uuid::deserialize(deserializer)?;
    if value.is_nil() {
        return Err(serde::de::Error::custom("UUID must not be nil"));
    }
    Ok(value)
}

const OWNED_OPERATIONS_API_VERSION: &str = "pet_resort_api.runtime.v1";
pub(crate) const OWNED_OPERATIONS_API_BOUNDARY: &str = "api_runtime_dto";
pub(crate) const OWNED_OPERATIONS_API_OWNER: &str = "pet_resort_api";

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
    pub fn operations_v1(workflow: impl Into<String>) -> Self {
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

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestMetadata {
    pub request_id: String,
    pub correlation_id: Option<String>,
    pub payload_logging: PayloadLogging,
    pub actor: Option<WireActorRef>,
    pub location_id: Option<String>,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PayloadLogging {
    Disabled,
    RedactedSummaryOnly,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// Wire-only actor payload. Handlers must authenticate and promote it into canonical actor types before use.
pub struct WireActorRef {
    /// Product persona asserted for this operation.
    pub persona: String,
    /// Stable actor identity authenticated by the runtime boundary.
    pub id: String,
    /// Optional narrower role claim when a workflow requires one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_role: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceRef {
    pub source_system: String,
    pub external_record_ref: String,
    pub observed_at: Option<String>,
    pub adapter_version: Option<String>,
    pub source_visibility: Option<String>,
}

/// Wire-only source-record evidence shape used by the v1 workflow routes.
///
/// Handlers must validate and promote these primitives before persistence; the
/// payload itself carries no source acceptance or executable authority.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct WireSourceRecordRef {
    pub system: String,
    pub record_type: String,
    pub record_id: String,
    pub observed_at: DateTime<Utc>,
    pub adapter_version: String,
}

/// Manager Daily Brief outcome request owned by the runtime API contract.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagerDailyBriefOutcomeCaptureRequest {
    /// Caller-reported disposition label retained as nonclaimable evidence.
    pub outcome: app::manager_daily_brief::FeedbackOutcome,
    /// Caller-reported minutes spent handling the action; not proof of completion or measured labor effect.
    pub actual_minutes: NonZeroU16,
    /// Caller-provided actor/persona labels; transport authentication does not prove review.
    pub actor: WireActorRef,
    /// Caller-reported feedback describing what was claimed to have happened.
    pub feedback: String,
    /// Source records correlated to the report; they do not prove review or completion.
    pub source_refs: Vec<WireSourceRecordRef>,
    /// Caller-reported observation timestamp used in replay identity; durable recording time is server-issued.
    pub timestamp: DateTime<Utc>,
    /// Correlation evidence for this workflow operation.
    pub audit: ManagerDailyBriefOutcomeAudit,
    /// Grouping dimensions for nonclaimable reported labor evidence.
    pub reporting: ManagerDailyBriefOutcomeReporting,
    /// Requested effects; runtime policy currently requires this list to be empty.
    pub requested_side_effects: Vec<String>,
    /// Required replay identity for atomic outcome recording.
    pub idempotency_key: IdempotencyKey,
}

/// Correlation evidence nested in a Manager Daily Brief outcome request.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagerDailyBriefOutcomeAudit {
    /// Workflow correlation id connecting context, action, and outcome.
    pub correlation_id: IdempotencyKey,
}

/// Grouping dimensions nested in a Manager Daily Brief reported-outcome request.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagerDailyBriefOutcomeReporting {
    /// Caller-reported location used to group retained evidence.
    #[serde(deserialize_with = "deserialize_non_nil_uuid")]
    pub location_id: Uuid,
    /// Caller-reported operating day used to group retained evidence.
    pub operating_day: NaiveDate,
}

impl fmt::Debug for WireActorRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("WireActorRef([REDACTED])")
    }
}

impl fmt::Debug for WireSourceRecordRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("WireSourceRecordRef([REDACTED])")
    }
}

impl fmt::Debug for ManagerDailyBriefOutcomeAudit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ManagerDailyBriefOutcomeAudit([REDACTED])")
    }
}

impl fmt::Debug for ManagerDailyBriefOutcomeReporting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ManagerDailyBriefOutcomeReporting([REDACTED])")
    }
}

impl fmt::Debug for ManagerDailyBriefOutcomeCaptureRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ManagerDailyBriefOutcomeCaptureRequest([REDACTED])")
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewGateRef {
    pub gate: String,
    pub required: bool,
    pub reviewer_role: Option<String>,
    pub reason: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockedAction {
    pub action: String,
    pub blocked_reason: String,
    pub review_gate: Option<ReviewGateRef>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditRef {
    pub audit_event_id: String,
    pub event_name: String,
    pub workflow_event_id: Option<String>,
    pub review_packet_id: Option<String>,
    pub approval_record_id: Option<String>,
    pub outbox_record_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorEnvelope {
    pub error: ApiError,
    pub request_id: String,
    pub correlation_id: Option<String>,
    pub live_side_effects: LiveSideEffectsMode,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub safe_error_class: String,
    pub details: Vec<ErrorDetail>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
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
pub struct ReportedLaborEstimateEvidence {
    pub before_minutes: u16,
    pub after_minutes: u16,
    pub reported_estimated_minutes_difference: u16,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowAudit {
    pub context_packet_id: String,
    pub correlation_id: String,
    pub runtime: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowObservability {
    pub correlation_id: String,
    pub request_id: String,
    pub request_correlation_id: String,
    pub route_status_trace: String,
    pub safe_error_class: String,
    pub payload_logging: String,
    pub sensitive_payload_logging: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataQualityIssue {
    pub kind: String,
    pub severity: String,
    pub workflow_blocking: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    pub source_refs: Vec<WireSourceRecordRef>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataQualityCandidate {
    pub id: String,
    pub kind: String,
    pub issue: DataQualityIssue,
    pub source_refs: Vec<WireSourceRecordRef>,
    pub source_freshness: String,
    pub sensitivity: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataQualityAction {
    pub id: String,
    pub kind: String,
    pub priority: String,
    pub owner_persona: String,
    pub removed_manual_work: String,
    pub rationale: String,
    pub source_refs: Vec<WireSourceRecordRef>,
    pub issue_refs: Vec<String>,
    pub review_gates: Vec<String>,
    pub labor_impact: ReportedLaborEstimateEvidence,
    pub live_side_effects_allowed: bool,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataQualityHygieneContextResponse {
    pub api_contract: ApiContractMetadata,
    pub workflow: WorkflowDescriptor,
    pub location_id: String,
    pub operating_day: String,
    pub prepared_for: String,
    pub candidates: Vec<DataQualityCandidate>,
    pub hygiene_actions: Vec<DataQualityAction>,
    pub reported_labor_estimate_evidence: ReportedLaborEstimateEvidence,
    pub allowed_agent_actions: Vec<String>,
    pub blocked_actions: Vec<String>,
    pub live_side_effects_allowed: bool,
    pub audit: WorkflowAudit,
    pub observability: WorkflowObservability,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DataQualityHygieneSubmittedAction {
    pub action_id: String,
    pub kind: String,
    pub source_refs: Vec<WireSourceRecordRef>,
    pub issue_refs: Vec<String>,
    pub review_gates: Vec<String>,
    pub requested_side_effects: Vec<String>,
    pub attempted_ambiguity_resolution: bool,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DataQualityHygieneDraftSubmissionRequest {
    pub context_packet_id: String,
    pub correlation_id: String,
    pub actions: Vec<DataQualityHygieneSubmittedAction>,
    pub idempotency_key: Option<String>,
}

macro_rules! impl_sensitive_debug {
    ($($type:ident),+ $(,)?) => {
        $(
            impl fmt::Debug for $type {
                fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str(concat!(stringify!($type), "([REDACTED])"))
                }
            }
        )+
    };
}

impl_sensitive_debug!(
    RequestMetadata,
    SourceRef,
    ReviewGateRef,
    BlockedAction,
    AuditRef,
    ErrorEnvelope,
    ApiError,
    ErrorDetail,
    WorkflowAudit,
    WorkflowObservability,
    DataQualityIssue,
    DataQualityCandidate,
    DataQualityAction,
    DataQualityHygieneContextResponse,
    DataQualityHygieneSubmittedAction,
    DataQualityHygieneDraftSubmissionRequest,
    DataQualityHygieneOutcomeActor,
    OutcomeAudit,
);

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
    actual_minutes: NonZeroU16,
    actor: DataQualityHygieneOutcomeActor,
    feedback: String,
    source_refs: Vec<WireSourceRecordRef>,
    issue_refs: Vec<String>,
    reported_resolution_status: DataQualityResolutionStatus,
    timestamp: DateTime<Utc>,
    audit: OutcomeAudit,
    requested_side_effects: Vec<String>,
    idempotency_key: IdempotencyKey,
}

impl DataQualityHygieneOutcomeCaptureRequest {
    pub const fn outcome(&self) -> DataQualityHygieneOutcome {
        self.outcome
    }

    pub const fn actual_minutes(&self) -> u16 {
        self.actual_minutes.get()
    }

    pub fn actor(&self) -> &DataQualityHygieneOutcomeActor {
        &self.actor
    }

    pub fn feedback(&self) -> &str {
        &self.feedback
    }

    pub fn source_refs(&self) -> &[WireSourceRecordRef] {
        &self.source_refs
    }

    pub fn issue_refs(&self) -> &[String] {
        &self.issue_refs
    }

    pub const fn reported_resolution_status(&self) -> DataQualityResolutionStatus {
        self.reported_resolution_status
    }

    pub const fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
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
        formatter.write_str("DataQualityHygieneOutcomeCaptureRequest([REDACTED])")
    }
}

/// One field in the canonical runtime-to-OpenAPI structural mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeFieldContract {
    /// Runtime serde field name.
    pub name: &'static str,
    /// Whether explicit JSON `null` is accepted.
    pub nullable: bool,
    /// Nested component reference, including array item references.
    pub nested_ref: Option<&'static str>,
    /// Exhaustive stable wire values when the field is an enum.
    pub enum_values: &'static [&'static str],
    /// OpenAPI scalar format that runtime deserialization enforces.
    pub format: Option<&'static str>,
    /// Inclusive numeric minimum enforced by runtime deserialization.
    pub minimum: Option<u64>,
    /// Inclusive minimum string length enforced by runtime deserialization.
    pub min_length: Option<u64>,
}

/// Canonical structural description of a runtime serde DTO.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeSchemaContract {
    /// OpenAPI component name mapped to the runtime type.
    pub name: &'static str,
    /// Fields required by runtime deserialization, in canonical declaration order.
    pub required: &'static [&'static str],
    /// Exhaustive runtime field contracts.
    pub fields: &'static [RuntimeFieldContract],
}

const fn field(
    name: &'static str,
    nullable: bool,
    nested_ref: Option<&'static str>,
    enum_values: &'static [&'static str],
) -> RuntimeFieldContract {
    RuntimeFieldContract {
        name,
        nullable,
        nested_ref,
        enum_values,
        format: None,
        minimum: None,
        min_length: None,
    }
}

const fn date_time_field(name: &'static str) -> RuntimeFieldContract {
    RuntimeFieldContract {
        name,
        nullable: false,
        nested_ref: None,
        enum_values: &[],
        format: Some("date-time"),
        minimum: None,
        min_length: None,
    }
}

const fn formatted_field(name: &'static str, format: &'static str) -> RuntimeFieldContract {
    RuntimeFieldContract {
        name,
        nullable: false,
        nested_ref: None,
        enum_values: &[],
        format: Some(format),
        minimum: None,
        min_length: None,
    }
}

const fn positive_integer_field(name: &'static str) -> RuntimeFieldContract {
    RuntimeFieldContract {
        name,
        nullable: false,
        nested_ref: None,
        enum_values: &[],
        format: None,
        minimum: Some(1),
        min_length: None,
    }
}

const fn nonnegative_integer_field(name: &'static str) -> RuntimeFieldContract {
    RuntimeFieldContract {
        name,
        nullable: false,
        nested_ref: None,
        enum_values: &[],
        format: None,
        minimum: Some(0),
        min_length: None,
    }
}

const fn nonempty_string_field(name: &'static str) -> RuntimeFieldContract {
    RuntimeFieldContract {
        name,
        nullable: false,
        nested_ref: None,
        enum_values: &[],
        format: None,
        minimum: None,
        min_length: Some(1),
    }
}

/// Returns the exhaustive DTO mapping checked against the owned OpenAPI artifact.
///
/// Adding or changing any listed serde field requires changing this owner and the
/// checked artifact together; parity tests compare names, requiredness,
/// nullability, enum values, nested references, and unknown-field posture.
pub fn runtime_schema_contracts() -> &'static [RuntimeSchemaContract] {
    const CONTRACTS: &[RuntimeSchemaContract] = &[
        RuntimeSchemaContract {
            name: "ActorRef",
            required: &["persona", "id"],
            fields: &[
                field("persona", false, None, &[]),
                field("id", false, None, &[]),
                field("actor_role", true, None, &[]),
            ],
        },
        RuntimeSchemaContract {
            name: "SourceRecordRef",
            required: &[
                "system",
                "record_type",
                "record_id",
                "observed_at",
                "adapter_version",
            ],
            fields: &[
                field("system", false, None, &[]),
                field("record_type", false, None, &[]),
                field("record_id", false, None, &[]),
                date_time_field("observed_at"),
                field("adapter_version", false, None, &[]),
            ],
        },
        RuntimeSchemaContract {
            name: "DataQualityHygieneOutcomeCaptureRequest",
            required: &[
                "outcome",
                "actual_minutes",
                "actor",
                "feedback",
                "source_refs",
                "issue_refs",
                "reported_resolution_status",
                "timestamp",
                "audit",
                "requested_side_effects",
                "idempotency_key",
            ],
            fields: &[
                field(
                    "outcome",
                    false,
                    None,
                    &[
                        "completed",
                        "deferred",
                        "suppressed_by_manager",
                        "source_fact_was_wrong",
                        "not_actionable",
                    ],
                ),
                positive_integer_field("actual_minutes"),
                field("actor", false, Some("#/components/schemas/ActorRef"), &[]),
                field("feedback", false, None, &[]),
                field(
                    "source_refs",
                    false,
                    Some("#/components/schemas/SourceRecordRef"),
                    &[],
                ),
                field("issue_refs", false, None, &[]),
                field(
                    "reported_resolution_status",
                    false,
                    None,
                    &["open", "acknowledged", "ignored", "repaired"],
                ),
                date_time_field("timestamp"),
                field(
                    "audit",
                    false,
                    Some("#/components/schemas/OutcomeAudit"),
                    &[],
                ),
                field("requested_side_effects", false, None, &[]),
                nonempty_string_field("idempotency_key"),
            ],
        },
        RuntimeSchemaContract {
            name: "ManagerDailyBriefOutcomeAudit",
            required: &["correlation_id"],
            fields: &[nonempty_string_field("correlation_id")],
        },
        RuntimeSchemaContract {
            name: "ReportedLaborEstimateEvidence",
            required: &[
                "before_minutes",
                "after_minutes",
                "reported_estimated_minutes_difference",
            ],
            fields: &[
                nonnegative_integer_field("before_minutes"),
                nonnegative_integer_field("after_minutes"),
                nonnegative_integer_field("reported_estimated_minutes_difference"),
            ],
        },
        RuntimeSchemaContract {
            name: "ManagerDailyBriefOutcomeReporting",
            required: &["location_id", "operating_day"],
            fields: &[
                formatted_field("location_id", "uuid"),
                formatted_field("operating_day", "date"),
            ],
        },
        RuntimeSchemaContract {
            name: "ManagerDailyBriefOutcomeCaptureRequest",
            required: &[
                "outcome",
                "actual_minutes",
                "actor",
                "feedback",
                "source_refs",
                "timestamp",
                "audit",
                "reporting",
                "requested_side_effects",
                "idempotency_key",
            ],
            fields: &[
                field(
                    "outcome",
                    false,
                    None,
                    &[
                        "completed",
                        "deferred",
                        "suppressed_by_manager",
                        "source_fact_was_wrong",
                    ],
                ),
                positive_integer_field("actual_minutes"),
                field("actor", false, Some("#/components/schemas/ActorRef"), &[]),
                field("feedback", false, None, &[]),
                field(
                    "source_refs",
                    false,
                    Some("#/components/schemas/SourceRecordRef"),
                    &[],
                ),
                date_time_field("timestamp"),
                field(
                    "audit",
                    false,
                    Some("#/components/schemas/ManagerDailyBriefOutcomeAudit"),
                    &[],
                ),
                field(
                    "reporting",
                    false,
                    Some("#/components/schemas/ManagerDailyBriefOutcomeReporting"),
                    &[],
                ),
                field("requested_side_effects", false, None, &[]),
                nonempty_string_field("idempotency_key"),
            ],
        },
    ];
    CONTRACTS
}

#[cfg(test)]
mod coverage_convergence_tests {
    use chrono::{TimeZone as _, Utc};
    use serde_json::json;

    use super::*;

    #[test]
    fn sensitive_runtime_contract_values_remain_redacted_and_validated() {
        let actor = WireActorRef {
            persona: "general_manager".to_owned(),
            id: "manager-7".to_owned(),
            actor_role: Some("site_manager".to_owned()),
        };
        let source = WireSourceRecordRef {
            system: "manual_import".to_owned(),
            record_type: "data_quality_issue".to_owned(),
            record_id: "source-7".to_owned(),
            observed_at: Utc.with_ymd_and_hms(2026, 8, 18, 5, 0, 0).unwrap(),
            adapter_version: "manual-v1".to_owned(),
        };
        let audit = ManagerDailyBriefOutcomeAudit {
            correlation_id: IdempotencyKey::try_new("correlation-7").unwrap(),
        };
        let reporting = ManagerDailyBriefOutcomeReporting {
            location_id: Uuid::from_u128(7),
            operating_day: NaiveDate::from_ymd_opt(2026, 8, 18).unwrap(),
        };
        let request = ManagerDailyBriefOutcomeCaptureRequest {
            outcome: app::manager_daily_brief::FeedbackOutcome::Completed,
            actual_minutes: NonZeroU16::new(7).unwrap(),
            actor: actor.clone(),
            feedback: "reviewed".to_owned(),
            source_refs: vec![source.clone()],
            timestamp: source.observed_at,
            audit,
            reporting,
            requested_side_effects: Vec::new(),
            idempotency_key: IdempotencyKey::try_new("manager-outcome-7").unwrap(),
        };

        assert_eq!(format!("{actor:?}"), "WireActorRef([REDACTED])");
        assert_eq!(format!("{source:?}"), "WireSourceRecordRef([REDACTED])");
        assert_eq!(
            format!("{:?}", request.audit),
            "ManagerDailyBriefOutcomeAudit([REDACTED])"
        );
        assert_eq!(
            format!("{:?}", request.reporting),
            "ManagerDailyBriefOutcomeReporting([REDACTED])"
        );
        assert_eq!(
            format!("{request:?}"),
            "ManagerDailyBriefOutcomeCaptureRequest([REDACTED])"
        );
        assert_eq!(
            format!("{:?}", request.idempotency_key),
            "IdempotencyKey([REDACTED])"
        );
        assert!(
            serde_json::from_value::<ManagerDailyBriefOutcomeReporting>(json!({
                "location_id": Uuid::nil(),
                "operating_day": "2026-08-18"
            }))
            .unwrap_err()
            .to_string()
            .contains("UUID must not be nil")
        );
    }

    #[test]
    fn runtime_field_contract_helpers_describe_every_enforced_scalar_shape() {
        let ordinary = field(
            "actor",
            true,
            Some("#/components/schemas/ActorRef"),
            &["staff"],
        );
        assert!(ordinary.nullable);
        assert_eq!(ordinary.nested_ref, Some("#/components/schemas/ActorRef"));
        assert_eq!(ordinary.enum_values, &["staff"]);

        let date_time = date_time_field("observed_at");
        assert_eq!(date_time.format, Some("date-time"));
        let formatted = formatted_field("location_id", "uuid");
        assert_eq!(formatted.format, Some("uuid"));
        let positive = positive_integer_field("actual_minutes");
        assert_eq!(positive.minimum, Some(1));
        let nonnegative = nonnegative_integer_field("before_minutes");
        assert_eq!(nonnegative.minimum, Some(0));
        let nonempty = nonempty_string_field("idempotency_key");
        assert_eq!(nonempty.min_length, Some(1));

        assert!(runtime_schema_contracts().iter().any(|contract| {
            contract.name == "ManagerDailyBriefOutcomeReporting"
                && contract
                    .fields
                    .iter()
                    .any(|field| field.format == Some("uuid"))
        }));
    }

    #[test]
    fn persona_and_outcome_accessors_preserve_exact_boundary_vocabulary() {
        assert_eq!(
            DataQualityHygienePersona::GeneralManager.as_str(),
            "general_manager"
        );
        assert_eq!(
            DataQualityHygienePersona::AssistantGeneralManager.as_str(),
            "assistant_general_manager"
        );
        assert_eq!(
            DataQualityHygienePersona::FrontDeskLead.as_str(),
            "front_desk_lead"
        );
        assert_eq!(
            DataQualityHygienePersona::FrontDeskAgent.as_str(),
            "front_desk_agent"
        );
        assert_eq!(
            DataQualityHygienePersona::RegionalOperator.as_str(),
            "regional_operator"
        );
        assert_eq!(
            DataQualityHygienePersona::OperationsAnalyst.as_str(),
            "operations_analyst"
        );

        let captured: DataQualityHygieneOutcomeCaptureRequest = serde_json::from_value(json!({
            "outcome": "deferred",
            "actual_minutes": 3,
            "actor": {
                "id": "actor-3",
                "persona": "front_desk_agent",
                "actor_role": "front_desk_agent"
            },
            "feedback": "waiting for source evidence",
            "source_refs": [{
                "system": "manual_import",
                "record_type": "issue",
                "record_id": "issue-3",
                "observed_at": "2026-08-18T05:00:00Z",
                "adapter_version": "manual-v1"
            }],
            "issue_refs": ["issue-3"],
            "reported_resolution_status": "acknowledged",
            "timestamp": "2026-08-18T05:00:00Z",
            "audit": {"correlation_id": "correlation-3"},
            "requested_side_effects": [],
            "idempotency_key": "outcome-3"
        }))
        .unwrap();

        assert_eq!(captured.outcome(), DataQualityHygieneOutcome::Deferred);
        assert_eq!(captured.actual_minutes(), 3);
        assert_eq!(captured.actor().id(), "actor-3");
        assert_eq!(
            captured.actor().persona(),
            DataQualityHygienePersona::FrontDeskAgent
        );
        assert_eq!(
            captured.actor().actor_role(),
            DataQualityHygienePersona::FrontDeskAgent
        );
        assert_eq!(captured.feedback(), "waiting for source evidence");
        assert_eq!(captured.source_refs().len(), 1);
        assert_eq!(captured.issue_refs(), &["issue-3"]);
        assert_eq!(
            captured.reported_resolution_status(),
            DataQualityResolutionStatus::Acknowledged
        );
        assert_eq!(
            captured.timestamp(),
            Utc.with_ymd_and_hms(2026, 8, 18, 5, 0, 0).unwrap()
        );
        assert_eq!(captured.audit().correlation_id(), "correlation-3");
        assert!(captured.requested_side_effects().is_empty());
        assert_eq!(
            captured.idempotency_key().expose_for_fingerprint(),
            "outcome-3"
        );
    }
}
