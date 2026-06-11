use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

static VACCINE_DOCUMENT_STATE: std::sync::OnceLock<VaccineDocumentState> =
    std::sync::OnceLock::new();

#[derive(Clone)]
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
    inquiry_intake_records: Vec<InquiryIntakeRecord>,
    audit_events: Vec<AuditEvent>,
}

#[derive(Debug, Serialize)]
struct HealthPayload {
    service: &'static str,
    status: &'static str,
    live_side_effects: &'static str,
}

#[derive(Debug, Serialize)]
struct ReadinessPayload {
    service: &'static str,
    database: &'static str,
    object_storage: &'static str,
    agent_runtime: &'static str,
    live_customer_messaging: &'static str,
    live_provider_writes: &'static str,
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

#[derive(Debug, Deserialize)]
struct InquirySubmissionRequest {
    source_event_key: String,
    location_id: String,
    customer: InquiryCustomerRequest,
    pet: InquiryPetRequest,
    service: String,
    requested_dates: Option<InquiryDateWindowRequest>,
    message: String,
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

#[derive(Debug, Clone, Serialize)]
struct InquiryIntakeRecord {
    event: InquiryEvent,
    lead: ParsedInquiryLead,
    draft_reply: InquiryDraftReply,
    task: InquiryTask,
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
    records: Vec<InquiryIntakeRecord>,
}

#[derive(Debug, Clone, Serialize)]
struct VaccineDocumentWorkflowPayload {
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

pub fn router() -> Router {
    router_with_state(
        VACCINE_DOCUMENT_STATE
            .get_or_init(VaccineDocumentState::default)
            .clone(),
    )
}

pub fn router_with_state(state: VaccineDocumentState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/inquiries", post(submit_inquiry))
        .route("/staff/inquiries", get(staff_inquiries))
        .route("/vaccine-documents/uploads", post(upload_vaccine_document))
        .route(
            "/vaccine-documents/review-packets/{review_packet_id}/approve",
            post(approve_vaccine_document),
        )
        .route(
            "/vaccine-documents/review-packets/{review_packet_id}/reject",
            post(reject_vaccine_document),
        )
        .with_state(state)
}

async fn healthz() -> Json<HealthPayload> {
    Json(HealthPayload {
        service: "pet-resort-api",
        status: "ok",
        live_side_effects: "disabled",
    })
}

async fn readyz() -> Json<ReadinessPayload> {
    Json(ReadinessPayload {
        service: "pet-resort-api",
        database: "not_configured",
        object_storage: "not_configured",
        agent_runtime: "fake_deterministic",
        live_customer_messaging: "disabled",
        live_provider_writes: "disabled",
    })
}

async fn submit_inquiry(
    State(state): State<VaccineDocumentState>,
    Json(request): Json<InquirySubmissionRequest>,
) -> (StatusCode, Json<InquiryIntakeRecord>) {
    let mut store = state.store.lock().await;
    let record = build_inquiry_intake_record(request);
    store.inquiry_intake_records.push(record.clone());
    (StatusCode::CREATED, Json(record))
}

async fn staff_inquiries(
    State(state): State<VaccineDocumentState>,
) -> Json<InquiryStaffQueuePayload> {
    let store = state.store.lock().await;
    Json(InquiryStaffQueuePayload {
        records: store.inquiry_intake_records.clone(),
    })
}

async fn upload_vaccine_document(
    State(state): State<VaccineDocumentState>,
    Json(request): Json<VaccineDocumentUploadRequest>,
) -> (StatusCode, Json<VaccineDocumentWorkflowPayload>) {
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
    (StatusCode::CREATED, Json(payload))
}

async fn approve_vaccine_document(
    State(state): State<VaccineDocumentState>,
    Path(review_packet_id): Path<Uuid>,
    Json(request): Json<VaccineReviewDecisionRequest>,
) -> (StatusCode, Json<VaccineDocumentWorkflowPayload>) {
    decide_vaccine_document(state, review_packet_id, request, true).await
}

async fn reject_vaccine_document(
    State(state): State<VaccineDocumentState>,
    Path(review_packet_id): Path<Uuid>,
    Json(request): Json<VaccineReviewDecisionRequest>,
) -> (StatusCode, Json<VaccineDocumentWorkflowPayload>) {
    decide_vaccine_document(state, review_packet_id, request, false).await
}

async fn decide_vaccine_document(
    state: VaccineDocumentState,
    review_packet_id: Uuid,
    request: VaccineReviewDecisionRequest,
    approved: bool,
) -> (StatusCode, Json<VaccineDocumentWorkflowPayload>) {
    let mut store = state.store.lock().await;
    let packet = store
        .review_packets
        .get_mut(&review_packet_id)
        .expect("review packet exists");
    packet.status = if approved { "approved" } else { "rejected" };
    let document_id = packet.document_id;
    let vaccine_record_id = packet.vaccine_record_id;

    let document = store
        .documents
        .get_mut(&document_id)
        .expect("document exists for packet");
    document.verification_status = if approved { "verified" } else { "rejected" };

    let vaccine_record = store
        .vaccine_records
        .get_mut(&vaccine_record_id)
        .expect("vaccine exists for packet");
    vaccine_record.status = if approved {
        "verified_current"
    } else {
        "rejected"
    };
    let pet_id = vaccine_record.pet_id;

    let eligibility = PetEligibility {
        pet_id,
        rabies_current: approved,
        source_vaccine_record_id: Some(vaccine_record_id),
        status: if approved {
            "eligible_from_approved_vaccine_document"
        } else {
            "ineligible_after_rejected_vaccine_document"
        },
    };
    store.eligibility.insert(pet_id, eligibility);

    let approval = ApprovalRecord {
        id: Uuid::new_v4(),
        review_packet_id,
        target_document_id: document_id,
        target_vaccine_record_id: vaccine_record_id,
        gate: "medical_document_review",
        status: if approved { "approved" } else { "rejected" },
        decided_by_staff_id: request.reviewed_by_staff_id.clone(),
        decided_at: Utc::now().to_rfc3339(),
        reason: request.reason,
    };
    store.approvals.insert(approval.id, approval.clone());
    store.audit_events.push(audit(
        "approval.decision.recorded",
        &request.reviewed_by_staff_id,
        "approval",
        approval.id,
        [("status", approval.status.to_owned())],
    ));
    store.audit_events.push(audit(
        "pet.eligibility.updated",
        &request.reviewed_by_staff_id,
        "pet",
        pet_id,
        [("rabies_current", approved.to_string())],
    ));

    let payload = store.payload(
        document_id,
        vaccine_record_id,
        review_packet_id,
        Some(approval),
    );
    (StatusCode::OK, Json(payload))
}

impl VaccineDocumentStore {
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

    InquiryIntakeRecord {
        event: InquiryEvent {
            event_type: "inquiry.received",
            source_event_key: request.source_event_key,
            location_id: request.location_id,
        },
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
        task: InquiryTask {
            kind: "missing_info_review",
            status: "open",
            title: task_title,
            review_gate: "front_desk_staff_review",
        },
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
