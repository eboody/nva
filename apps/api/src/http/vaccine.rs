use super::*;

pub(super) async fn upload_vaccine_document(
    State(state): State<VaccineDocumentState>,
    Protected(authentication): Protected,
    Json(request): Json<VaccineDocumentUploadRequest>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize(
        &authentication,
        authentication::Mutation::VaccineDocumentUpload,
        &request.uploaded_by_staff_id,
        local_manager_daily_brief_location_id().get(),
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

pub(super) async fn approve_vaccine_document(
    State(state): State<VaccineDocumentState>,
    Protected(authentication): Protected,
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

pub(super) async fn reject_vaccine_document(
    State(state): State<VaccineDocumentState>,
    Protected(authentication): Protected,
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

pub(super) async fn decide_vaccine_document(
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
        local_manager_daily_brief_location_id().get(),
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

pub(super) fn vaccine_review_decision_error_response(
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

pub(super) fn extract_vaccine_evidence(
    id: Uuid,
    document_id: Uuid,
    content: &str,
) -> VaccineExtractionRecord {
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

pub(super) fn audit(
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
