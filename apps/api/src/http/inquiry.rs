use super::*;

pub(super) fn classify_invalid_inquiry(request: &InquirySubmissionRequest) -> Option<Value> {
    let provenance_field_count = [
        request.source_system.is_some(),
        request.provider_model_path.is_some(),
        request.raw_payload_ref.is_some(),
        request.received_at.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if provenance_field_count != 0 && provenance_field_count != 4 {
        return Some(invalid_inquiry_payload(
            "incomplete_source_provenance",
            &request.source_event_key,
        ));
    }

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

pub(super) fn invalid_inquiry_payload(
    classification: &'static str,
    source_event_key: &str,
) -> Value {
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

pub(super) async fn submit_inquiry(
    State(state): State<VaccineDocumentState>,
    Protected(authentication): Protected,
    Json(request): Json<InquirySubmissionRequest>,
) -> axum::response::Response {
    let Some(location_id) = Uuid::parse_str(&request.location_id)
        .ok()
        .and_then(|location_id| entities::LocationId::try_new(location_id).ok())
    else {
        return PublicApiError::new(
            ErrorKind::Validation {
                details: Vec::new(),
            },
            ErrorContext::new("missing_request_id"),
        )
        .with_public_code("invalid_location_id")
        .into_response();
    };
    if let Err(rejection) = authentication::authorize_source_ingest(
        &authentication,
        authentication::Mutation::InquiryIntake,
        location_id.get(),
    ) {
        return PublicApiError::new(rejection.into(), ErrorContext::new("missing_request_id"))
            .into_response();
    }
    if let Some(error_payload) = classify_invalid_inquiry(&request) {
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(error_payload)).into_response();
    }
    let request_fingerprint = inquiry_request_fingerprint(&request);
    let mut store = state.store.lock().await;
    if let Some(mut record) = store
        .inquiry_intake_records
        .iter()
        .find(|record| record.event.source_event_key == request.source_event_key)
        .cloned()
    {
        if record.request_fingerprint != request_fingerprint {
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
    let record = build_inquiry_intake_record(request);
    store.inquiry_intake_records.push(record.clone());
    (StatusCode::CREATED, Json(record)).into_response()
}

pub(super) fn inquiry_request_fingerprint(request: &InquirySubmissionRequest) -> String {
    let canonical_payload = serde_json::to_vec(request)
        .expect("inquiry request DTO contains only infallibly serializable fields");
    format!("{:x}", Sha256::digest(canonical_payload))
}

pub(super) async fn staff_inquiries(
    State(state): State<VaccineDocumentState>,
    Protected(authentication): Protected,
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

pub(super) fn build_inquiry_intake_record(
    request: InquirySubmissionRequest,
) -> InquiryIntakeRecord {
    let request_fingerprint = inquiry_request_fingerprint(&request);
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
    let has_source_provenance = request.source_system.is_some();
    let has_lead_response_fixture = has_source_provenance
        || !request.contact_attempts.is_empty()
        || request.simulated_conversion.is_some();
    let source_system = request.source_system.clone();
    let provider_model_path = request.provider_model_path.clone();
    let raw_payload_ref = request.raw_payload_ref.clone();
    let received_at = request.received_at;
    let first_attempt = request.contact_attempts.first();
    let simulated_conversion = request.simulated_conversion.as_ref().map(|conversion| {
        json!({
            "reported_reservation_id": conversion.reservation_id,
            "reported_conversion_at": conversion.converted_at,
            "reported_attribution_source": conversion.attribution_source,
            "source_event_key": request.source_event_key
        })
    });
    let outcome_attribution = simulated_conversion.as_ref().map(|conversion| {
        json!({
            "report_status": "caller_reported_simulated_conversion",
            "source_event_key": request.source_event_key,
            "reported_reservation_id": conversion["reported_reservation_id"],
            "supports_value_claim": false,
            "value_claim_boundary": "caller-reported simulation proves no review, conversion, reservation attribution, completion, or measured value"
        })
    });

    InquiryIntakeRecord {
        api_contract: api_dto_contract("inquiry_intake"),
        event: InquiryEvent {
            event_type: "inquiry.received",
            source_event_key: request.source_event_key,
            location_id: request.location_id,
        },
        provenance: has_source_provenance.then(|| {
            json!({
                "source_system": source_system,
                "provider_model_path": provider_model_path,
                "raw_payload_ref": raw_payload_ref,
                "received_at": received_at,
                "provider_payload_passthrough": false
            })
        }),
        data_quality: has_source_provenance.then(|| {
            json!({
                "classification": "accepted_fixture_evidence",
                "quality_gate": "source_ref_present_provider_model_declared_no_raw_payload_passthrough",
                "duplicate_policy": "source_event_key_idempotency"
            })
        }),
        canonical_lead_event: has_source_provenance.then(|| {
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
                "sla_status": if first_attempt.is_some() { "caller_reported_attempt_present" } else { "no_caller_reported_attempt" },
                "attempts": request.contact_attempts.iter().map(|attempt| json!({
                    "reported_attempted_at": attempt.attempted_at,
                    "reported_channel": attempt.channel,
                    "reported_purpose": attempt.purpose,
                    "reported_outcome": attempt.outcome,
                    "required_review_gate": "front_desk_staff_review",
                    "reported_message_ref": attempt.message_ref
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
        request_fingerprint,
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
