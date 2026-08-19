use super::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DataQualityHygieneAgentContextQuery {
    pub(super) location_id: Uuid,
    pub(super) operating_day: NaiveDate,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DataQualityHygieneOutcomeSummaryQuery {
    pub(super) location_id: Uuid,
    pub(super) operating_day: NaiveDate,
    pub(super) correlation_id: Option<String>,
}

pub(super) async fn data_quality_hygiene_agent_context(
    Protected(authentication): Protected,
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
    let location_id = entities::LocationId::new(query.location_id);
    let operating_day = operations::operating_day::Date::try_new(query.operating_day)
        .expect("operating day date is always valid after query parsing");
    let packet = local_data_quality_hygiene_packet(location_id, operating_day);

    Json(data_quality_hygiene_packet_payload(&packet, &request_trace)).into_response()
}

pub(super) async fn submit_data_quality_hygiene_agent_draft(
    State(state): State<VaccineDocumentState>,
    Protected(authentication): Protected,
    Json(request): Json<public_contract::DataQualityHygieneDraftSubmissionRequest>,
) -> (StatusCode, Json<Value>) {
    if let Err(rejection) = authentication::authorize_source_ingest(
        &authentication,
        authentication::Mutation::DataQualityHygieneDraft,
        local_data_quality_hygiene_location_id().get(),
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
        let reasons = validate_data_quality_hygiene_submitted_action(
            &state.contract_observation,
            &packet,
            action,
        );
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

pub(super) fn data_quality_hygiene_payload_fingerprint(
    action_id: &str,
    request: &public_contract::DataQualityHygieneOutcomeCaptureRequest,
) -> String {
    let mut canonical_payload =
        serde_json::to_value(request).expect("public outcome DTO serializes infallibly");
    canonical_payload
        .as_object_mut()
        .expect("public outcome DTO serializes as an object")
        .remove("idempotency_key");
    canonical_payload["action_id"] = Value::String(action_id.to_owned());
    let bytes = serde_json::to_vec(&canonical_payload)
        .expect("canonical public outcome payload serializes infallibly");
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn data_quality_hygiene_idempotency_conflict_payload() -> Value {
    merge_error_envelope(
        json!({
            "api_contract": api_dto_contract_payload("data_quality_hygiene_outcome"),
            "accepted": false,
            "outcome_persisted": false,
            "live_side_effects_allowed": false,
            "blocked_actions": data_quality_hygiene_blocked_action_codes()
        }),
        PublicApiError::new(
            ErrorKind::IdempotencyConflict,
            ErrorContext::new("missing_request_id"),
        ),
    )
}

pub(super) async fn capture_data_quality_hygiene_action_outcome(
    State(state): State<VaccineDocumentState>,
    Protected(authentication): Protected,
    Path(action_id): Path<String>,
    Json(request): Json<public_contract::DataQualityHygieneOutcomeCaptureRequest>,
) -> (StatusCode, Json<Value>) {
    if let Err(rejection) = authentication::authorize(
        &authentication,
        authentication::Mutation::DataQualityHygieneOutcome,
        request.actor().id(),
        local_data_quality_hygiene_location_id().get(),
    )
    .and_then(|()| {
        authentication::authorize_persona_claim(&authentication, request.actor().persona().as_str())
            .and_then(|()| {
                authentication::authorize_persona_claim(
                    &authentication,
                    request.actor().actor_role().as_str(),
                )
            })
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

    let idempotency_key_digest = format!(
        "{:x}",
        Sha256::digest(
            request
                .idempotency_key()
                .expose_for_fingerprint()
                .as_bytes()
        )
    );
    let payload_fingerprint = data_quality_hygiene_payload_fingerprint(&action_id, &request);
    {
        let store = state.store.lock().await;
        if let Some(replay) = store
            .data_quality_hygiene_idempotency
            .get(&idempotency_key_digest)
        {
            if replay.payload_fingerprint == payload_fingerprint {
                let mut response = replay.response.clone();
                response["idempotent_replay"] = Value::Bool(true);
                return (StatusCode::OK, Json(response));
            }
            return (
                StatusCode::CONFLICT,
                Json(data_quality_hygiene_idempotency_conflict_payload()),
            );
        }
    }

    let reasons = request
        .requested_side_effects()
        .iter()
        .map(|side_effect| {
            data_quality_hygiene_requested_side_effect_rejection_reason(
                &state.contract_observation,
                side_effect,
            )
        })
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

    if request.source_refs().is_empty() {
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

    if request.issue_refs().is_empty() {
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

    let Ok(actual_minutes) = storage::operations::StoredDataQualityHygieneLaborMinutes::try_new(
        request.actual_minutes(),
    ) else {
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
    if request.source_refs() != expected_source_refs {
        provenance_reasons.push("source_refs_do_not_match_action");
    }
    if request.issue_refs() != expected_issue_refs {
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
        .outcome(stored_data_quality_hygiene_outcome(request.outcome()))
        .before_minutes(before_minutes)
        .actual_minutes(actual_minutes)
        .actor_id(request.actor().id().to_owned())
        .actor_persona(stored_data_quality_hygiene_persona_claim(
            request.actor().persona(),
        ))
        .feedback(request.feedback().to_owned())
        .source_refs(
            request
                .source_refs()
                .iter()
                .map(stored_source_record_ref_from_payload)
                .collect(),
        )
        .issue_refs(request.issue_refs().to_vec())
        .reported_resolution_status(stored_data_quality_resolution_status(
            request.reported_resolution_status(),
        ))
        .recorded_at(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true))
        .correlation_id(request.audit().correlation_id().to_owned())
        .location_id(packet.location_id().get().to_string())
        .operating_day(packet.operating_day().get().to_string())
        .action_kind(stored_data_quality_hygiene_action_kind(action.kind()))
        .owner_persona(stored_data_quality_hygiene_persona(action.owner_persona()))
        .reported_estimated_minutes_difference(
            action
                .labor_impact()
                .reported_estimated_minutes_difference(),
        )
        .build();
    let reporting_group = record.reporting_group();
    let local_persistence_records =
        storage::operations::DataQualityHygieneLocalPersistenceRecords::from_reported_outcome(
            data_quality_hygiene_lineage_ids(&record),
            record.clone(),
        );
    let storage_projection_proof =
        data_quality_hygiene_storage_projection_proof(&local_persistence_records);
    let observability = data_quality_hygiene_outcome_observability_payload(
        &record.correlation_id,
        &local_persistence_records,
    );
    let response = {
        let mut store = state.store.lock().await;
        if let Some(replay) = store
            .data_quality_hygiene_idempotency
            .get(&idempotency_key_digest)
        {
            if replay.payload_fingerprint == payload_fingerprint {
                let mut response = replay.response.clone();
                response["idempotent_replay"] = Value::Bool(true);
                return (StatusCode::OK, Json(response));
            }
            return (
                StatusCode::CONFLICT,
                Json(data_quality_hygiene_idempotency_conflict_payload()),
            );
        }

        let persisted_outcome_count = store.data_quality_hygiene_outcomes.record(record.clone());
        store
            .data_quality_hygiene_persistence_records
            .push(local_persistence_records);
        let persisted_projection_count = store.data_quality_hygiene_persistence_records.len();
        let response = json!({
            "api_contract": api_dto_contract_payload("data_quality_hygiene_outcome"),
            "accepted": true,
            "outcome_persisted": true,
            "idempotent_replay": false,
            "outcome_record": {
                "action_id": record.action_id,
                "outcome": reported_data_quality_hygiene_outcome(record.outcome),
                "authority_disposition": "needs_review",
                "claimable": false,
                "before_minutes": record.before_minutes.get(),
                "actual_minutes": record.actual_minutes.get(),
                "actor": {
                    "id": record.actor_id,
                    "persona": record.actor_persona
                },
                "feedback": record.feedback,
                "source_refs": record.source_refs,
                "issue_refs": record.issue_refs,
                "reported_resolution_status": record.reported_resolution_status,
                "timestamp": record.recorded_at,
                "audit": {
                    "correlation_id": record.correlation_id
                }
            },
            "reported_labor_evidence": {
                "reported_estimated_minutes_difference": record.reported_estimated_minutes_difference,
                "reported_actual_minutes_spent": record.actual_minutes.get(),
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
        });
        store.data_quality_hygiene_idempotency.insert(
            idempotency_key_digest,
            DataQualityHygieneReplay {
                payload_fingerprint,
                response: response.clone(),
            },
        );
        response
    };

    (StatusCode::CREATED, Json(response))
}

pub(super) async fn data_quality_hygiene_outcome_summary(
    State(state): State<VaccineDocumentState>,
    Protected(authentication): Protected,
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

pub(super) fn data_quality_hygiene_lineage_ids(
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

pub(super) fn data_quality_hygiene_lineage_key(
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

pub(super) fn data_quality_hygiene_local_demo_readiness_payload() -> Value {
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

pub(super) fn data_quality_hygiene_outcome_observability_payload(
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

pub(super) fn data_quality_hygiene_storage_projection_proof(
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

pub(super) fn reported_data_quality_hygiene_outcome(
    outcome: storage::operations::DataQualityHygieneOutcomeCode,
) -> &'static str {
    match outcome {
        storage::operations::DataQualityHygieneOutcomeCode::Completed => "reported_completed",
        storage::operations::DataQualityHygieneOutcomeCode::Deferred => "reported_deferred",
        storage::operations::DataQualityHygieneOutcomeCode::SuppressedByManager => {
            "reported_suppressed_by_manager"
        }
        storage::operations::DataQualityHygieneOutcomeCode::SourceFactWasWrong => {
            "reported_source_fact_was_wrong"
        }
        storage::operations::DataQualityHygieneOutcomeCode::NotActionable => {
            "reported_not_actionable"
        }
    }
}

pub(super) fn local_data_quality_hygiene_packet(
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

pub(super) fn local_data_quality_hygiene_candidates() -> Vec<data_quality_hygiene::Candidate> {
    vec![
        data_quality_hygiene::Candidate::builder()
            .id(
                data_quality_hygiene::IssueRef::try_new("dq-vaccine-stale-42")
                    .expect("static issue ref is valid"),
            )
            .kind(data_quality_hygiene::CandidateKind::SourceFreshness)
            .issue(domain_data_quality::Issue::new(
                domain_data_quality::Kind::MissingVaccinationRecord,
                domain_data_quality::Severity::Blocking,
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
            .issue(domain_data_quality::Issue::new(
                domain_data_quality::Kind::DuplicateSourceRecord,
                domain_data_quality::Severity::Warning,
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

pub(super) fn data_quality_hygiene_packet_payload(
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
        location_id: packet.location_id().get().to_string(),
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
            .map(|action| {
                data_quality_hygiene_denied_intent(*action)
                    .code()
                    .to_owned()
            })
            .collect(),
        reported_labor_estimate_evidence: public_contract::ReportedLaborEstimateEvidence {
            before_minutes: packet.before_minutes().get(),
            after_minutes: packet.after_minutes().get(),
            reported_estimated_minutes_difference: packet.reported_estimated_minutes_difference(),
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

pub(super) fn data_quality_hygiene_candidate_payload(
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

pub(super) fn data_quality_hygiene_action_source_refs(
    packet: &data_quality_hygiene::Packet,
    action: &data_quality_hygiene::Action,
) -> Vec<public_contract::WireSourceRecordRef> {
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

pub(super) fn data_quality_hygiene_action_payload(
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
        labor_impact: public_contract::ReportedLaborEstimateEvidence {
            before_minutes: action.labor_impact().before_minutes().get(),
            after_minutes: action.labor_impact().after_minutes().get(),
            reported_estimated_minutes_difference: action
                .labor_impact()
                .reported_estimated_minutes_difference(),
        },
        live_side_effects_allowed: false,
    }
}

pub(super) fn validate_data_quality_hygiene_submitted_action(
    denial_boundary: &contract_observation::State,
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
            denial_boundary,
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

pub(super) fn data_quality_hygiene_requested_side_effect_rejection_reason(
    denial_boundary: &contract_observation::State,
    side_effect: &str,
) -> String {
    match denial_boundary.reject_requested_intent(
        contract_observation::DeniedLiveEffectWorkflow::DataQualityHygiene,
        side_effect,
    ) {
        contract_observation::RequestedIntentRejection::Denied(_) => {
            "blocked_side_effect_requested".to_owned()
        }
        contract_observation::RequestedIntentRejection::Unsupported => {
            "unsupported_side_effect_requested".to_owned()
        }
    }
}

pub(super) fn data_quality_hygiene_blocked_action_codes() -> Vec<&'static str> {
    contract_observation::DeniedLiveEffectIntent::for_workflow(
        contract_observation::DeniedLiveEffectWorkflow::DataQualityHygiene,
    )
    .map(contract_observation::DeniedLiveEffectIntent::code)
    .collect()
}

pub(super) fn stored_source_record_ref_from_payload(
    value: &public_contract::WireSourceRecordRef,
) -> storage::operations::StoredSourceRecordRef {
    storage::operations::StoredSourceRecordRef::builder()
        .system(value.system.clone())
        .record_type(value.record_type.clone())
        .record_id(value.record_id.clone())
        .observed_at(
            value
                .observed_at
                .to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true),
        )
        .adapter_version(value.adapter_version.clone())
        .build()
}

pub(super) fn stored_data_quality_hygiene_action_kind(
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

pub(super) fn stored_data_quality_hygiene_persona(
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

pub(super) fn stored_data_quality_hygiene_outcome(
    outcome: public_contract::DataQualityHygieneOutcome,
) -> storage::operations::DataQualityHygieneOutcomeCode {
    use public_contract::DataQualityHygieneOutcome as Api;
    use storage::operations::DataQualityHygieneOutcomeCode as Stored;
    match outcome {
        Api::Completed => Stored::Completed,
        Api::Deferred => Stored::Deferred,
        Api::SuppressedByManager => Stored::SuppressedByManager,
        Api::SourceFactWasWrong => Stored::SourceFactWasWrong,
        Api::NotActionable => Stored::NotActionable,
    }
}

pub(super) fn stored_data_quality_hygiene_persona_claim(
    persona: public_contract::DataQualityHygienePersona,
) -> storage::operations::DataQualityHygienePersonaCode {
    use public_contract::DataQualityHygienePersona as Api;
    use storage::operations::DataQualityHygienePersonaCode as Stored;
    match persona {
        Api::GeneralManager => Stored::GeneralManager,
        Api::AssistantGeneralManager => Stored::AssistantGeneralManager,
        Api::FrontDeskLead => Stored::FrontDeskLead,
        Api::FrontDeskAgent => Stored::FrontDeskAgent,
        Api::RegionalOperator => Stored::RegionalOperator,
        Api::OperationsAnalyst => Stored::OperationsAnalyst,
    }
}

pub(super) fn stored_data_quality_resolution_status(
    status: public_contract::DataQualityResolutionStatus,
) -> storage::operations::DataQualityResolutionStatusCode {
    use public_contract::DataQualityResolutionStatus as Api;
    use storage::operations::DataQualityResolutionStatusCode as Stored;
    match status {
        Api::Open => Stored::Open,
        Api::Acknowledged => Stored::Acknowledged,
        Api::Ignored => Stored::Ignored,
        Api::Repaired => Stored::Repaired,
    }
}

pub(super) fn local_data_quality_hygiene_location_id() -> entities::LocationId {
    local_manager_daily_brief_location_id()
}

pub(super) fn local_data_quality_hygiene_operating_day() -> operations::operating_day::Date {
    local_manager_daily_brief_operating_day()
}

pub(super) fn data_quality_hygiene_source_provenance(
    endpoint: &'static str,
    record_id: &'static str,
) -> source::Provenance {
    source::Provenance::builder()
        .system(source::System::ProviderOrPms)
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
            source::SchemaVersion::try_new("gingr-v1-readonly")
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

pub(super) fn data_quality_hygiene_action_kind_code(
    kind: data_quality_hygiene::ActionKind,
) -> &'static str {
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

pub(super) fn data_quality_hygiene_persona_code(
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

pub(super) fn data_quality_hygiene_candidate_kind_code(
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

pub(super) fn data_quality_hygiene_source_freshness_code(
    freshness: data_quality_hygiene::SourceFreshness,
) -> &'static str {
    match freshness {
        data_quality_hygiene::SourceFreshness::Current => "current",
        data_quality_hygiene::SourceFreshness::Stale => "stale",
        data_quality_hygiene::SourceFreshness::Conflicting => "conflicting",
        data_quality_hygiene::SourceFreshness::Missing => "missing",
    }
}

pub(super) fn data_quality_hygiene_sensitivity_code(
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

pub(super) fn data_quality_hygiene_action_priority_code(
    priority: data_quality_hygiene::ActionPriority,
) -> &'static str {
    match priority {
        data_quality_hygiene::ActionPriority::High => "high",
        data_quality_hygiene::ActionPriority::Medium => "medium",
        data_quality_hygiene::ActionPriority::Low => "low",
    }
}

pub(super) fn data_quality_hygiene_removed_manual_work_code(
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

pub(super) fn data_quality_hygiene_safe_action_code(
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
        data_quality_hygiene::SafeAgentAction::ReportReconciliationEstimateDifference => {
            "report_reconciliation_estimate_difference"
        }
    }
}

pub(super) fn data_quality_hygiene_denied_intent(
    action: data_quality_hygiene::BlockedAction,
) -> contract_observation::DeniedLiveEffectIntent {
    match action {
        data_quality_hygiene::BlockedAction::SendCustomerMessage => {
            contract_observation::DeniedLiveEffectIntent::SendCustomerMessage
        }
        data_quality_hygiene::BlockedAction::MutateProviderOrPmsRecord => {
            contract_observation::DeniedLiveEffectIntent::MutateProviderOrPmsRecord
        }
        data_quality_hygiene::BlockedAction::ChangeStaffSchedule => {
            contract_observation::DeniedLiveEffectIntent::ChangeStaffSchedule
        }
        data_quality_hygiene::BlockedAction::MoveRefundDiscountOrPayment => {
            contract_observation::DeniedLiveEffectIntent::MoveRefundDiscountOrPayment
        }
        data_quality_hygiene::BlockedAction::HideOrAutoResolveSourceAmbiguity => {
            contract_observation::DeniedLiveEffectIntent::HideOrAutoResolveSourceAmbiguity
        }
        data_quality_hygiene::BlockedAction::ExposeQuarantinedSensitivePayload => {
            contract_observation::DeniedLiveEffectIntent::ExposeQuarantinedSensitivePayload
        }
    }
}

#[cfg(test)]
mod coverage_convergence_tests {
    use super::*;

    #[test]
    fn unsupported_requested_side_effect_stays_explicitly_rejected() {
        assert_eq!(
            data_quality_hygiene_requested_side_effect_rejection_reason(
                &contract_observation::State::default(),
                "not_a_supported_live_effect",
            ),
            "unsupported_side_effect_requested"
        );
    }
}
