use super::*;

#[derive(Debug, Serialize)]
pub(super) struct HealthPayload {
    pub(super) api_contract: public_contract::ApiContractMetadata,
    pub(super) service: &'static str,
    pub(super) status: &'static str,
    pub(super) live_side_effects: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct ReadinessPayload {
    pub(super) api_contract: public_contract::ApiContractMetadata,
    pub(super) service: &'static str,
    pub(super) database: &'static str,
    pub(super) object_storage: &'static str,
    pub(super) agent_runtime: &'static str,
    pub(super) workflow_repository: WorkflowRepositoryReadinessPayload,
    pub(super) observability: ObservabilityReadinessPayload,
    pub(super) live_customer_messaging: &'static str,
    pub(super) live_provider_writes: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct ObservabilityReadinessPayload {
    pub(super) request_correlation: &'static str,
    pub(super) workflow_correlation: &'static str,
    pub(super) local_request_metrics: &'static str,
    pub(super) metrics_scope: &'static str,
    pub(super) production_gap: &'static str,
    pub(super) durable_traces: &'static str,
    pub(super) production_metrics: &'static str,
    pub(super) dashboard: &'static str,
    pub(super) alerting: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct WorkflowRepositoryReadinessPayload {
    pub(super) active_adapter: &'static str,
    pub(super) postgres_adapter: &'static str,
    pub(super) contract: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub(super) struct ReadModelDescriptorPayload {
    pub(super) name: &'static str,
    pub(super) source: &'static str,
    pub(super) projection_version: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct ReadModelDataPosturePayload {
    pub(super) safe_synthetic_data: bool,
    pub(super) live_side_effects_allowed: bool,
    pub(super) provider_payload_passthrough: bool,
    pub(super) provider_writes_allowed: bool,
    pub(super) customer_messages_allowed: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct ReadModelDatabasePayload {
    pub(super) status: &'static str,
    pub(super) adapter: &'static str,
    pub(super) error: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct SourceQualityBacklogPayload {
    pub(super) api_contract: public_contract::ApiContractMetadata,
    pub(super) read_model: ReadModelDescriptorPayload,
    pub(super) data_posture: ReadModelDataPosturePayload,
    pub(super) database: ReadModelDatabasePayload,
    pub(super) records: Vec<workflow_repository::source_quality_backlog::Item>,
}

pub(super) fn workflow_repository_readiness_payload() -> WorkflowRepositoryReadinessPayload {
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

pub(super) fn database_readiness_status() -> &'static str {
    if database_url_configured() {
        "env_configured_not_verified"
    } else {
        "not_configured"
    }
}

pub(super) fn object_storage_readiness_status() -> &'static str {
    if minio_env_configured() {
        "env_configured_not_verified"
    } else {
        "not_configured"
    }
}

#[derive(Debug, Serialize)]
pub(super) struct OpsMetricsSummaryPayload {
    pub(super) api_contract: public_contract::ApiContractMetadata,
    pub(super) api_request_metrics: ApiRequestMetricsPayload,
    pub(super) product_labor_metrics: ProductLaborMetricsPayload,
    pub(super) local_runtime_counters: LocalRuntimeCountersPayload,
    pub(super) safety: MetricsSafetyPayload,
    pub(super) observability_gap: ObservabilityGapPayload,
    pub(super) production_metrics_plan: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub(super) struct ApiRequestMetricsPayload {
    pub(super) scope: &'static str,
    pub(super) request_id_source: &'static str,
    pub(super) correlation_id_source: &'static str,
    pub(super) payload_logging: &'static str,
    pub(super) safe_error_classes: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub(super) struct ProductLaborMetricsPayload {
    pub(super) manager_daily_brief: LaborOutcomeRollupPayload,
    pub(super) data_quality_hygiene: LaborOutcomeRollupPayload,
}

#[derive(Debug, Serialize)]
pub(super) struct LaborOutcomeRollupPayload {
    pub(super) metric_source: &'static str,
    pub(super) reported_outcome_count: usize,
    pub(super) reported_actual_minutes_spent: u16,
}

#[derive(Debug, Serialize)]
pub(super) struct LocalRuntimeCountersPayload {
    pub(super) inquiry_count: usize,
    pub(super) review_packet_count: usize,
    pub(super) audit_event_count: usize,
    pub(super) outcome_count: usize,
    pub(super) data_quality_hygiene_outbox_candidate_count: usize,
    pub(super) data_quality_hygiene_review_gated_outbox_count: usize,
    pub(super) production_queue_adapter: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct MetricsSafetyPayload {
    pub(super) granularity: &'static str,
    pub(super) contains_customer_pii: bool,
    pub(super) contains_provider_payloads: bool,
    pub(super) live_side_effects: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct ObservabilityGapPayload {
    pub(super) production_traces: &'static str,
    pub(super) durable_request_metrics: &'static str,
    pub(super) dashboard_or_alerting: &'static str,
}

pub(super) fn api_dto_contract(workflow: &'static str) -> public_contract::ApiContractMetadata {
    public_contract::ApiContractMetadata::operations_v1(workflow)
}

pub(super) fn api_dto_contract_payload(workflow: &'static str) -> Value {
    json!(api_dto_contract(workflow))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VaccineDocumentUploadRequest {
    pub(super) pet_id: Uuid,
    pub(super) customer_id: Uuid,
    pub(super) filename: String,
    pub(super) mime_type: String,
    pub(super) content: String,
    pub(super) uploaded_by_staff_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VaccineReviewDecisionRequest {
    pub(super) reviewed_by_staff_id: String,
    pub(super) reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VaccineReviewDecision {
    Approve,
    Reject,
}

impl VaccineReviewDecision {
    pub(super) fn status_code(self) -> &'static str {
        match self {
            Self::Approve => "approved",
            Self::Reject => "rejected",
        }
    }

    pub(super) fn document_verification_status(self) -> &'static str {
        match self {
            Self::Approve => "verified",
            Self::Reject => "rejected",
        }
    }

    pub(super) fn vaccine_record_status(self) -> &'static str {
        match self {
            Self::Approve => "verified_current",
            Self::Reject => "rejected",
        }
    }

    pub(super) fn eligibility(self, pet_id: Uuid, vaccine_record_id: Uuid) -> PetEligibility {
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

    pub(super) fn from_decided_status(status: &str) -> Option<Self> {
        match status {
            "approved" => Some(Self::Approve),
            "rejected" => Some(Self::Reject),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub(super) struct VaccineReviewDecisionEvidence {
    pub(super) reviewed_by_staff_id: String,
    pub(super) decided_at: String,
    pub(super) reason: Option<String>,
}

#[derive(Debug)]
pub(super) enum VaccineReviewDecisionRejection {
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

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InquirySubmissionRequest {
    pub(super) source_event_key: String,
    pub(super) source_system: Option<String>,
    pub(super) provider_model_path: Option<String>,
    pub(super) raw_payload_ref: Option<String>,
    pub(super) received_at: Option<DateTime<Utc>>,
    pub(super) location_id: String,
    pub(super) customer: InquiryCustomerRequest,
    pub(super) pet: InquiryPetRequest,
    pub(super) service: String,
    pub(super) requested_dates: Option<InquiryDateWindowRequest>,
    pub(super) message: String,
    pub(super) contact_attempts: Vec<InquiryContactAttemptRequest>,
    pub(super) simulated_conversion: Option<InquirySimulatedConversionRequest>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InquiryCustomerRequest {
    pub(super) full_name: String,
    pub(super) email: Option<String>,
    pub(super) phone: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InquiryPetRequest {
    pub(super) name: String,
    pub(super) species: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InquiryDateWindowRequest {
    pub(super) start: String,
    pub(super) end: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InquiryContactAttemptRequest {
    pub(super) attempted_at: DateTime<Utc>,
    pub(super) channel: String,
    pub(super) purpose: String,
    pub(super) outcome: String,
    pub(super) message_ref: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InquirySimulatedConversionRequest {
    pub(super) reservation_id: String,
    pub(super) converted_at: DateTime<Utc>,
    pub(super) attribution_source: String,
}

#[derive(Clone, Serialize)]
pub(super) struct InquiryIntakeRecord {
    pub(super) api_contract: public_contract::ApiContractMetadata,
    pub(super) event: InquiryEvent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) provenance: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) data_quality: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) canonical_lead_event: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) workflow: Option<Value>,
    pub(super) lead: ParsedInquiryLead,
    pub(super) draft_reply: InquiryDraftReply,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) review_packet: Option<Value>,
    pub(super) task: InquiryTask,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) storage_projection: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) api_response: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) simulated_conversion: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) outcome_attribution: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) replay: Option<Value>,
    #[serde(skip)]
    pub(super) request_fingerprint: String,
    pub(super) agent_runtime: &'static str,
    pub(super) policy_boundary: &'static str,
    pub(super) audit_events: Vec<InquiryAuditEvent>,
}

#[derive(Clone, Serialize)]
pub(super) struct InquiryEvent {
    pub(super) event_type: &'static str,
    pub(super) source_event_key: String,
    pub(super) location_id: String,
}

#[derive(Clone, Serialize)]
pub(super) struct ParsedInquiryLead {
    pub(super) customer_name: String,
    pub(super) customer_email: Option<String>,
    pub(super) customer_phone: Option<String>,
    pub(super) pet_name: String,
    pub(super) species: String,
    pub(super) service: String,
    pub(super) requested_dates: Option<ParsedInquiryDateWindow>,
    pub(super) original_message: String,
    pub(super) missing_info: Vec<&'static str>,
    pub(super) review_status: &'static str,
}

#[derive(Clone, Serialize)]
pub(super) struct ParsedInquiryDateWindow {
    pub(super) start: String,
    pub(super) end: String,
}

#[derive(Clone, Serialize)]
pub(super) struct InquiryDraftReply {
    pub(super) status: &'static str,
    pub(super) live_send_allowed: bool,
    pub(super) approval_gate: &'static str,
    pub(super) body: String,
}

#[derive(Clone, Serialize)]
pub(super) struct InquiryTask {
    pub(super) kind: &'static str,
    pub(super) status: &'static str,
    pub(super) title: String,
    pub(super) review_gate: &'static str,
}

#[derive(Clone, Serialize)]
pub(super) struct InquiryAuditEvent {
    pub(super) action: &'static str,
    pub(super) actor_kind: &'static str,
    pub(super) subject_key: String,
}

#[derive(Serialize)]
pub(super) struct InquiryStaffQueuePayload {
    pub(super) api_contract: public_contract::ApiContractMetadata,
    pub(super) records: Vec<InquiryIntakeRecord>,
}

#[derive(Clone, Serialize)]
pub(super) struct VaccineDocumentWorkflowPayload {
    pub(super) api_contract: public_contract::ApiContractMetadata,
    pub(super) document: DocumentRecord,
    pub(super) extraction: VaccineExtractionRecord,
    pub(super) vaccine_record: VaccineRecord,
    pub(super) review_packet: ReviewPacket,
    pub(super) approval: Option<ApprovalRecord>,
    pub(super) eligibility: PetEligibility,
    pub(super) audit_events: Vec<AuditEvent>,
}

#[derive(Clone, Serialize)]
pub(super) struct DocumentRecord {
    pub(super) id: Uuid,
    pub(super) pet_id: Uuid,
    pub(super) customer_id: Uuid,
    pub(super) classification: &'static str,
    pub(super) source: &'static str,
    pub(super) filename: String,
    pub(super) mime_type: String,
    pub(super) content_length_bytes: usize,
    pub(super) sha256: String,
    pub(super) storage_bucket: &'static str,
    pub(super) storage_key: String,
    pub(super) storage_version: String,
    pub(super) virus_scan_status: &'static str,
    pub(super) pii_redaction_status: &'static str,
    pub(super) verification_status: &'static str,
}

#[derive(Clone, Serialize)]
pub(super) struct VaccineExtractionRecord {
    pub(super) id: Uuid,
    pub(super) document_id: Uuid,
    pub(super) schema_version: &'static str,
    pub(super) vaccine_name: String,
    pub(super) effective_on: NaiveDate,
    pub(super) expires_on: Option<NaiveDate>,
    pub(super) confidence: f32,
    pub(super) uncertainty_policy: &'static str,
    pub(super) auto_accept_threshold: f32,
    pub(super) raw_text_ref: String,
}

#[derive(Clone, Serialize)]
pub(super) struct VaccineRecord {
    pub(super) id: Uuid,
    pub(super) pet_id: Uuid,
    pub(super) source_document_id: Uuid,
    pub(super) vaccine_name: String,
    pub(super) status: &'static str,
    pub(super) effective_on: NaiveDate,
    pub(super) expires_on: Option<NaiveDate>,
    pub(super) review_gate: &'static str,
}

#[derive(Clone, Serialize)]
pub(super) struct ReviewPacket {
    pub(super) id: Uuid,
    pub(super) document_id: Uuid,
    pub(super) vaccine_record_id: Uuid,
    pub(super) gate: &'static str,
    pub(super) status: &'static str,
    pub(super) uncertainty: &'static str,
}

#[derive(Clone, Serialize)]
pub(super) struct ApprovalRecord {
    pub(super) id: Uuid,
    pub(super) review_packet_id: Uuid,
    pub(super) target_document_id: Uuid,
    pub(super) target_vaccine_record_id: Uuid,
    pub(super) gate: &'static str,
    pub(super) status: &'static str,
    pub(super) decided_by_staff_id: String,
    pub(super) decided_at: String,
    pub(super) reason: Option<String>,
}

#[derive(Clone, Serialize)]
pub(super) struct PetEligibility {
    pub(super) pet_id: Uuid,
    pub(super) rabies_current: bool,
    pub(super) source_vaccine_record_id: Option<Uuid>,
    pub(super) status: &'static str,
}

#[derive(Clone, Serialize)]
pub(super) struct AuditEvent {
    pub(super) action: &'static str,
    pub(super) actor_kind: &'static str,
    pub(super) actor_id: String,
    pub(super) subject_kind: &'static str,
    pub(super) subject_id: Uuid,
    pub(super) metadata: BTreeMap<&'static str, String>,
}

#[derive(Clone, Debug)]
pub(super) struct RequestTraceEvidence {
    pub(super) request_id: String,
    pub(super) request_correlation_id: String,
}

impl RequestTraceEvidence {
    pub(super) fn from_request_headers(headers: &axum::http::HeaderMap) -> Self {
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

    pub(super) fn request_id(&self) -> &str {
        &self.request_id
    }

    pub(super) fn request_correlation_id(&self) -> &str {
        &self.request_correlation_id
    }
}
