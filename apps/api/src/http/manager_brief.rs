use super::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ManagerDailyBriefAgentContextQuery {
    pub(super) location_id: Uuid,
    pub(super) operating_day: NaiveDate,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ManagerDailyBriefAgentDraftSubmissionRequest {
    pub(super) context_packet_id: String,
    pub(super) correlation_id: String,
    pub(super) submitted_by: String,
    pub(super) actions: Vec<ManagerDailyBriefSubmittedAction>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ManagerDailyBriefSubmittedAction {
    pub(super) id: String,
    pub(super) kind: String,
    pub(super) recommendation: String,
    pub(super) source_refs: Vec<public_contract::WireSourceRecordRef>,
    pub(super) review_gates: Vec<String>,
    pub(super) requested_side_effects: Vec<String>,
}

pub(super) type ManagerDailyBriefOutcomeCaptureRequest =
    public_contract::ManagerDailyBriefOutcomeCaptureRequest;

pub(super) fn manager_daily_brief_payload_fingerprint(
    action_id: &str,
    request: &ManagerDailyBriefOutcomeCaptureRequest,
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

pub(super) fn manager_daily_brief_idempotency_conflict_payload() -> Value {
    merge_error_envelope(
        json!({
            "api_contract": api_dto_contract_payload("manager_daily_brief_outcome"),
            "accepted": false,
            "outcome_persisted": false,
            "live_side_effects_allowed": false,
            "blocked_actions": manager_daily_brief_blocked_action_codes()
        }),
        PublicApiError::new(
            ErrorKind::IdempotencyConflict,
            ErrorContext::new("missing_request_id"),
        ),
    )
}

pub(super) fn manager_daily_brief_validation_payload(reasons: Vec<String>) -> Value {
    let details = reasons
        .iter()
        .map(|reason| public_contract::ErrorDetail::field("request".to_owned(), reason.clone()))
        .collect();
    merge_error_envelope(
        json!({
            "api_contract": api_dto_contract_payload("manager_daily_brief_outcome"),
            "accepted": false,
            "outcome_persisted": false,
            "reasons": reasons,
            "live_side_effects_allowed": false,
            "blocked_actions": manager_daily_brief_blocked_action_codes()
        }),
        PublicApiError::new(
            ErrorKind::Validation { details },
            ErrorContext::new("missing_request_id"),
        ),
    )
}

pub(super) async fn capture_manager_daily_brief_action_outcome(
    State(state): State<VaccineDocumentState>,
    Protected(authentication): Protected,
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

    let idempotency_key_digest = format!(
        "{:x}",
        Sha256::digest(request.idempotency_key.expose_for_fingerprint().as_bytes())
    );
    let idempotency_key =
        storage::workflow_repository::IdempotencyKey::try_new(idempotency_key_digest)
            .expect("SHA-256 idempotency digest is non-empty");
    let payload_fingerprint = storage::workflow_repository::OperationFingerprint::try_new(
        manager_daily_brief_payload_fingerprint(&action_id, &request),
    )
    .expect("SHA-256 semantic payload fingerprint is non-empty");

    let reasons = request
        .requested_side_effects
        .iter()
        .map(|side_effect| {
            manager_daily_brief_requested_side_effect_rejection_reason(
                &state.contract_observation,
                side_effect,
            )
        })
        .collect::<Vec<_>>();

    if !reasons.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(manager_daily_brief_validation_payload(reasons)),
        );
    }

    if request.source_refs.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(manager_daily_brief_validation_payload(vec![
                "missing_source_refs".to_owned(),
            ])),
        );
    }

    let Ok(actual_minutes) = storage::operations::StoredManagerDailyBriefLaborMinutes::try_new(
        request.actual_minutes.get(),
    ) else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(manager_daily_brief_validation_payload(vec![
                "actual_minutes_must_be_greater_than_zero".to_owned(),
            ])),
        );
    };

    let Ok(actor_persona) = request
        .actor
        .persona
        .parse::<storage::operations::ManagerDailyBriefPersonaCode>()
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(manager_daily_brief_validation_payload(vec![
                "unsupported_actor_persona".to_owned(),
            ])),
        );
    };

    let Some((location_id, operating_day)) =
        manager_daily_brief_reporting_scope(&request.reporting)
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(manager_daily_brief_validation_payload(vec![
                "invalid_reporting_scope".to_owned(),
            ])),
        );
    };

    if let Err(rejection) = authentication::authorize_location(&authentication, location_id.get()) {
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
            Json(manager_daily_brief_validation_payload(vec![
                "unknown_manager_daily_brief_action_id".to_owned(),
            ])),
        );
    };

    let expected_source_refs = manager_daily_brief_action_source_refs(action, operating_day);
    if request.source_refs != expected_source_refs {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(manager_daily_brief_validation_payload(vec![
                "source_refs_do_not_match_action".to_owned(),
            ])),
        );
    }

    let before_minutes = storage::operations::StoredManagerDailyBriefLaborMinutes::try_new(
        action.labor_impact().before_minutes().get(),
    )
    .expect("manager daily brief action labor impact uses non-zero domain labor minutes");

    let record = storage::operations::ManagerDailyBriefOutcomeRecord::builder()
        .action_id(action_id)
        .outcome(stored_manager_daily_brief_outcome(request.outcome))
        .before_minutes(before_minutes)
        .actual_minutes(actual_minutes)
        .actor_id(request.actor.id)
        .actor_persona(actor_persona)
        .feedback(request.feedback)
        .source_refs(
            expected_source_refs
                .iter()
                .map(stored_source_record_ref_from_payload)
                .collect(),
        )
        .recorded_at(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true))
        .correlation_id(
            request
                .audit
                .correlation_id
                .expose_for_fingerprint()
                .to_owned(),
        )
        .location_id(location_id.get().to_string())
        .operating_day(operating_day.get().to_string())
        .action_kind(stored_manager_daily_brief_action_kind(action.kind()))
        .owner_persona(stored_manager_daily_brief_persona(action.owner_persona()))
        .reported_estimated_minutes_difference(
            action
                .labor_impact()
                .reported_estimated_minutes_difference(),
        )
        .build();
    let idempotent_result = {
        let mut store = state.store.lock().await;
        store.manager_daily_brief_outcomes.record_idempotently(
            idempotency_key,
            payload_fingerprint,
            record,
        )
    };
    let (status, persisted_outcome_count, idempotent_replay, record) = match idempotent_result {
        storage::workflow_repository::IdempotentRecord::Recorded {
            retained_count,
            retained_outcome,
        } => (StatusCode::CREATED, retained_count, false, retained_outcome),
        storage::workflow_repository::IdempotentRecord::Replay {
            retained_count,
            retained_outcome,
        } => (StatusCode::OK, retained_count, true, retained_outcome),
        storage::workflow_repository::IdempotentRecord::Conflict => {
            return (
                StatusCode::CONFLICT,
                Json(manager_daily_brief_idempotency_conflict_payload()),
            );
        }
    };
    let reporting_group = record.reporting_group();

    (
        status,
        Json(json!({
            "api_contract": api_dto_contract_payload("manager_daily_brief_outcome"),
            "accepted": true,
            "outcome_persisted": true,
            "idempotent_replay": idempotent_replay,
            "outcome_record": {
                "action_id": record.action_id,
                "outcome": reported_manager_daily_brief_outcome(record.outcome),
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

pub(super) async fn submit_manager_daily_brief_agent_draft(
    State(state): State<VaccineDocumentState>,
    Protected(authentication): Protected,
    Json(request): Json<ManagerDailyBriefAgentDraftSubmissionRequest>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize(
        &authentication,
        authentication::Mutation::ManagerDailyBriefDraft,
        &request.submitted_by,
        local_data_quality_hygiene_location_id().get(),
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
        let reasons = validate_manager_daily_brief_submitted_action(
            &state.contract_observation,
            action,
            packet.as_ref(),
        );
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

pub(super) fn validate_manager_daily_brief_submitted_action(
    denial_boundary: &contract_observation::State,
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
            denial_boundary,
            side_effect,
        ));
    }

    reasons
}

pub(super) fn required_manager_daily_brief_review_gate(kind: &str) -> Option<&'static str> {
    match kind {
        "review_demand_against_staffing_plan" => Some("manager_approval"),
        "resolve_checkout_exception" => Some("manager_approval"),
        "approve_retention_follow_up_draft" => Some("customer_message_approval"),
        "investigate_source_data_quality_issue" => Some("manager_approval"),
        _ => None,
    }
}

pub(super) fn manager_daily_brief_requested_side_effect_rejection_reason(
    denial_boundary: &contract_observation::State,
    side_effect: &str,
) -> String {
    match denial_boundary.reject_requested_intent(
        contract_observation::DeniedLiveEffectWorkflow::ManagerDailyBrief,
        side_effect,
    ) {
        contract_observation::RequestedIntentRejection::Denied(intent) => {
            format!("blocked_side_effect:{}", intent.code())
        }
        contract_observation::RequestedIntentRejection::Unsupported => {
            format!("unsupported_side_effect:{side_effect}")
        }
    }
}

pub(super) fn manager_daily_brief_reporting_scope(
    reporting: &public_contract::ManagerDailyBriefOutcomeReporting,
) -> Option<(entities::LocationId, operations::operating_day::Date)> {
    Some((
        entities::LocationId::try_new(reporting.location_id).ok()?,
        operations::operating_day::Date::try_new(reporting.operating_day).ok()?,
    ))
}

pub(super) fn manager_daily_brief_packet_from_context_id(
    context_packet_id: &str,
) -> Option<manager_daily_brief::Packet> {
    let scope = context_packet_id.strip_prefix("manager-daily-brief-context:")?;
    let (location_id, operating_day) = scope.rsplit_once(':')?;
    let location_id = entities::LocationId::try_new(Uuid::parse_str(location_id).ok()?).ok()?;
    let operating_day = operations::operating_day::Date::try_new(
        NaiveDate::parse_from_str(operating_day, "%Y-%m-%d").ok()?,
    )
    .ok()?;
    Some(local_manager_daily_brief_packet(location_id, operating_day))
}

pub(super) fn local_manager_daily_brief_packet(
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

pub(super) fn stored_manager_daily_brief_action_kind(
    kind: manager_daily_brief::BriefActionKind,
) -> storage::operations::ManagerDailyBriefActionKindCode {
    match kind {
        manager_daily_brief::BriefActionKind::ReviewDemandAgainstStaffingPlan => {
            storage::operations::ManagerDailyBriefActionKindCode::ReviewDemandAgainstStaffingPlan
        }
        manager_daily_brief::BriefActionKind::ResolveCheckoutException => {
            storage::operations::ManagerDailyBriefActionKindCode::ResolveCheckoutException
        }

        manager_daily_brief::BriefActionKind::InvestigateSourceDataQualityIssue => {
            storage::operations::ManagerDailyBriefActionKindCode::InvestigateSourceDataQualityIssue
        }
        manager_daily_brief::BriefActionKind::ReviewCapacityLaborRecommendation => {
            storage::operations::ManagerDailyBriefActionKindCode::ReviewCapacityLaborRecommendation
        }
    }
}

pub(super) fn stored_manager_daily_brief_persona(
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

fn stored_manager_daily_brief_outcome(
    outcome: app::manager_daily_brief::FeedbackOutcome,
) -> storage::operations::ManagerDailyBriefOutcomeCode {
    match outcome {
        app::manager_daily_brief::FeedbackOutcome::Completed => {
            storage::operations::ManagerDailyBriefOutcomeCode::Completed
        }
        app::manager_daily_brief::FeedbackOutcome::Deferred => {
            storage::operations::ManagerDailyBriefOutcomeCode::Deferred
        }
        app::manager_daily_brief::FeedbackOutcome::SuppressedByManager => {
            storage::operations::ManagerDailyBriefOutcomeCode::SuppressedByManager
        }
        app::manager_daily_brief::FeedbackOutcome::SourceFactWasWrong => {
            storage::operations::ManagerDailyBriefOutcomeCode::SourceFactWasWrong
        }
    }
}

pub(super) fn reported_manager_daily_brief_outcome(
    outcome: storage::operations::ManagerDailyBriefOutcomeCode,
) -> &'static str {
    match outcome {
        storage::operations::ManagerDailyBriefOutcomeCode::Completed => "reported_completed",
        storage::operations::ManagerDailyBriefOutcomeCode::Deferred => "reported_deferred",
        storage::operations::ManagerDailyBriefOutcomeCode::SuppressedByManager => {
            "reported_suppressed_by_manager"
        }
        storage::operations::ManagerDailyBriefOutcomeCode::SourceFactWasWrong => {
            "reported_source_fact_was_wrong"
        }
    }
}

pub(super) async fn manager_daily_brief_agent_context(
    Protected(authentication): Protected,
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
    let location_id = entities::LocationId::new(query.location_id);
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
        "crm_retention_opportunities": [],
        "manager_brief_actions": packet
            .actions()
            .iter()
            .map(|action| manager_brief_action_payload(action, packet.operating_day()))
            .collect::<Vec<_>>(),
        "data_quality_issues": data_quality_issues,
        "source_refs": source_refs,
        "allowed_agent_actions": packet.safe_agent_actions().iter().map(safe_agent_action_code).collect::<Vec<_>>(),
        "blocked_actions": packet
            .blocked_actions()
            .iter()
            .map(|action| manager_daily_brief_denied_intent(action).code())
            .collect::<Vec<_>>(),
        "labor_impact": {
            "before_minutes": packet.before_minutes().get(),
            "after_minutes": packet.after_minutes().get(),
            "reported_estimated_minutes_difference": packet.reported_estimated_minutes_difference()
        },
        "audit": {
            "context_packet_id": format!("manager-daily-brief-context:{}:{}", query.location_id, query.operating_day),
            "correlation_id": correlation_id
        },
        "observability": workflow_observability_payload(&correlation_id, &request_trace)
    }))
    .into_response()
}

pub(super) fn local_manager_daily_brief_service_demand_facts(
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
                vec![domain_data_quality::Issue::new(
                    domain_data_quality::Kind::UnmappedServiceType,
                    domain_data_quality::Severity::Warning,
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

pub(super) fn local_manager_daily_brief_checkout_packets(
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
                .departure_observation(open_manager_brief_departure_observation())
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

pub(super) fn local_manager_daily_brief_retention_packets(
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
                .departure_observation(resolved_manager_brief_departure_observation())
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

pub(super) fn service_demand_fact_payload(fact: &analytics::service_demand::Fact) -> Value {
    json!({
        "kind": "service_demand_forecast",
        "service_line": "boarding",
        "demand_units": fact.demand_units().get(),
        "projection_version": fact.projection_version().as_str(),
        "data_quality_status": service_demand_data_quality_status_code(fact.data_quality_status()),
        "source_refs": fact.source_record_refs().iter().map(source_record_ref_payload).collect::<Vec<_>>()
    })
}

pub(super) fn checkout_exception_payload(
    scoped: &manager_daily_brief::ScopedCheckoutPacket,
) -> Option<Value> {
    let packet = scoped.packet();
    Some(json!({
        "reservation_id": format!("{:?}", packet.reservation_id()),
        "review_reasons": packet.review_reasons().iter().map(checkout_review_reason_code).collect::<Vec<_>>(),
        "required_review_gates": packet.required_review_gates().iter().map(review_gate_code).collect::<Vec<_>>(),
        "source_refs": [source_record_ref_payload(&source::RecordRef::from_provenance(packet.provenance()))]
    }))
}

pub(super) fn manager_brief_action_payload(
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
            "reported_estimated_minutes_difference": action.labor_impact().reported_estimated_minutes_difference()
        }
    })
}

pub(super) fn source_fact_payload(fact: &manager_daily_brief::SourceFact) -> Value {
    json!({
        "kind": source_fact_kind_code(fact.kind()),
        "summary": fact.summary().clone().into_inner(),
        "source_refs": fact.source_record_refs().iter().map(source_record_ref_payload).collect::<Vec<_>>()
    })
}

pub(super) fn data_quality_issue_payload(issue: &domain_data_quality::Issue) -> Value {
    json!({
        "kind": data_quality_kind_code(&issue.kind()),
        "severity": data_quality_severity_code(issue.severity()),
        "workflow_blocking": issue.workflow_blocking(),
        "source_refs": [source_record_ref_payload(issue.source_record_ref())]
    })
}

pub(super) fn data_quality_issue_contract(
    issue: &domain_data_quality::Issue,
) -> public_contract::DataQualityIssue {
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

pub(super) fn missing_context_issue_payload(kind: &'static str, detail: &'static str) -> Value {
    json!({
        "kind": kind,
        "severity": "warning",
        "workflow_blocking": false,
        "detail": detail,
        "source_refs": []
    })
}

pub(super) fn source_record_ref_payload(record_ref: &source::RecordRef) -> Value {
    json!({
        "system": source_system_code(record_ref.system()),
        "record_id": record_ref.record_id().as_str()
    })
}

pub(super) fn source_record_ref_contract(
    record_ref: &source::RecordRef,
    provenance: &source::Provenance,
) -> public_contract::WireSourceRecordRef {
    public_contract::WireSourceRecordRef {
        system: source_system_code(record_ref.system()).to_owned(),
        record_type: provenance.endpoint().as_str().to_owned(),
        record_id: record_ref.record_id().as_str().to_owned(),
        observed_at: *provenance.pulled_at().get(),
        adapter_version: provenance.schema_version().as_str().to_owned(),
    }
}

pub(super) fn manager_daily_brief_action_source_refs(
    action: &manager_daily_brief::BriefAction,
    operating_day: operations::operating_day::Date,
) -> Vec<public_contract::WireSourceRecordRef> {
    let mut source_refs = action
        .source_facts()
        .iter()
        .flat_map(|fact| {
            fact.source_record_refs().iter().map(move |record_ref| {
                public_contract::WireSourceRecordRef {
                    system: source_system_code(record_ref.system()).to_owned(),
                    record_type: source_fact_kind_code(fact.kind()).to_owned(),
                    record_id: record_ref.record_id().as_str().to_owned(),
                    observed_at: operating_day
                        .get()
                        .and_hms_opt(0, 0, 0)
                        .expect("valid operating day has midnight")
                        .and_utc(),
                    adapter_version: "nva-local-manager-daily-brief-fixture-v1".to_owned(),
                }
            })
        })
        .collect::<Vec<_>>();
    source_refs.sort();
    source_refs.dedup();
    source_refs
}

pub(super) fn local_manager_daily_brief_location_id() -> entities::LocationId {
    entities::LocationId::new(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0001))
}

pub(super) fn local_manager_daily_brief_customer_id() -> entities::CustomerId {
    entities::CustomerId::new(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0099))
}

pub(super) fn local_manager_daily_brief_reservation_id() -> entities::reservation::Id {
    entities::reservation::Id::new(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0042))
}

pub(super) fn local_manager_daily_brief_operating_day() -> operations::operating_day::Date {
    operations::operating_day::Date::try_new(
        NaiveDate::from_ymd_opt(2026, 6, 17).expect("fixture operating day is valid"),
    )
    .expect("fixture operating day is valid")
}

pub(super) fn open_manager_brief_departure_observation() -> checkout_completion::DepartureObservation
{
    checkout_completion::DepartureObservation::builder()
        .reported_by(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("front-desk-erin")
                .expect("static staff id is valid"),
        })
        .reported_at(DateTime::<Utc>::UNIX_EPOCH)
        .reported_belongings_returned(false)
        .care_summary(
            checkout_completion::CareSummary::try_new("Medication bag needs review.")
                .expect("static care summary is valid"),
        )
        .reported_care_summary_reviewed(false)
        .build()
}

pub(super) fn resolved_manager_brief_departure_observation()
-> checkout_completion::DepartureObservation {
    checkout_completion::DepartureObservation::builder()
        .reported_by(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("front-desk-erin")
                .expect("static staff id is valid"),
        })
        .reported_at(DateTime::<Utc>::UNIX_EPOCH)
        .reported_belongings_returned(true)
        .care_summary(
            checkout_completion::CareSummary::try_new("Clean checkout.")
                .expect("static care summary is valid"),
        )
        .reported_care_summary_reviewed(true)
        .build()
}

pub(super) fn manager_brief_retention_opportunity() -> crm_retention::RetentionOpportunity {
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

pub(super) fn manager_brief_contact_permission() -> crm_retention::ContactPermission {
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

pub(super) fn manager_brief_source_provenance() -> source::Provenance {
    source::Provenance::builder()
        .system(source::System::ProviderOrPms)
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
            source::SchemaVersion::try_new("gingr-v1-readonly")
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

pub(super) fn manager_brief_contact_provenance() -> source::Provenance {
    source::Provenance::builder()
        .system(source::System::ProviderOrPms)
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
            source::SchemaVersion::try_new("gingr-v1-readonly")
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

pub(super) fn source_system_code(system: source::System) -> &'static str {
    match system {
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

pub(super) fn service_demand_data_quality_status_code(
    status: analytics::service_demand::DataQualityStatus,
) -> &'static str {
    match status {
        analytics::service_demand::DataQualityStatus::Complete => "complete",
        analytics::service_demand::DataQualityStatus::ManagerReviewRequired => {
            "manager_review_required"
        }
    }
}

pub(super) fn checkout_review_reason_code(
    reason: &checkout_completion::ReviewReason,
) -> &'static str {
    match reason {
        checkout_completion::ReviewReason::DepartureEvidenceRequiresReview => {
            "departure_evidence_requires_review"
        }
        checkout_completion::ReviewReason::BelongingsFollowUpReported => {
            "belongings_follow_up_reported"
        }
        checkout_completion::ReviewReason::CareReviewReported => "care_review_reported",
        checkout_completion::ReviewReason::SourceNotCheckedOut => "source_not_checked_out",
        checkout_completion::ReviewReason::Payment(_) => "payment_exception",
        checkout_completion::ReviewReason::Source(_) => "source_exception",
    }
}

pub(super) fn review_gate_code(gate: &policy::ReviewGate) -> &'static str {
    match gate {
        policy::ReviewGate::ManagerApproval => "manager_approval",
        policy::ReviewGate::CustomerMessageApproval => "customer_message_approval",
        policy::ReviewGate::MedicalDocumentReview => "medical_document_review",
        policy::ReviewGate::BehaviorReview => "behavior_review",
        policy::ReviewGate::RefundOrDepositException => "refund_or_deposit_exception",
    }
}

pub(super) fn safe_agent_action_code(
    action: &manager_daily_brief::SafeAgentAction,
) -> &'static str {
    match action {
        manager_daily_brief::SafeAgentAction::SummarizeSourceEvidence => {
            "summarize_source_evidence"
        }
        manager_daily_brief::SafeAgentAction::RankManagerActions => "rank_manager_actions",
        manager_daily_brief::SafeAgentAction::DraftInternalTaskForReview => "draft_internal_tasks",
        manager_daily_brief::SafeAgentAction::RecordManagerFeedback => "record_manager_feedback",
        manager_daily_brief::SafeAgentAction::ReportLaborEstimateDifference => {
            "report_labor_estimate_difference"
        }
    }
}

pub(super) fn manager_daily_brief_blocked_action_codes() -> Vec<&'static str> {
    contract_observation::DeniedLiveEffectIntent::for_workflow(
        contract_observation::DeniedLiveEffectWorkflow::ManagerDailyBrief,
    )
    .map(contract_observation::DeniedLiveEffectIntent::code)
    .collect()
}

pub(super) fn manager_daily_brief_denied_intent(
    action: &manager_daily_brief::BlockedAction,
) -> contract_observation::DeniedLiveEffectIntent {
    match action {
        manager_daily_brief::BlockedAction::ChangeStaffSchedule => {
            contract_observation::DeniedLiveEffectIntent::ChangeStaffSchedule
        }
        manager_daily_brief::BlockedAction::MutateProviderOrPmsRecord => {
            contract_observation::DeniedLiveEffectIntent::MutateProviderOrPmsRecord
        }
        manager_daily_brief::BlockedAction::SendCustomerMessage => {
            contract_observation::DeniedLiveEffectIntent::SendCustomerMessage
        }
        manager_daily_brief::BlockedAction::MoveRefundDiscountOrPayment => {
            contract_observation::DeniedLiveEffectIntent::MoveRefundDiscountOrPayment
        }
        manager_daily_brief::BlockedAction::HideSourceDataQualityIssue => {
            contract_observation::DeniedLiveEffectIntent::HideSourceDataQualityIssue
        }
    }
}

pub(super) fn brief_action_kind_code(kind: manager_daily_brief::BriefActionKind) -> &'static str {
    match kind {
        manager_daily_brief::BriefActionKind::ReviewDemandAgainstStaffingPlan => {
            "review_demand_against_staffing_plan"
        }
        manager_daily_brief::BriefActionKind::ResolveCheckoutException => {
            "resolve_checkout_exception"
        }

        manager_daily_brief::BriefActionKind::InvestigateSourceDataQualityIssue => {
            "investigate_source_data_quality_issue"
        }
        manager_daily_brief::BriefActionKind::ReviewCapacityLaborRecommendation => {
            "review_capacity_labor_recommendation"
        }
    }
}

pub(super) fn brief_action_priority_code(
    priority: manager_daily_brief::BriefActionPriority,
) -> &'static str {
    match priority {
        manager_daily_brief::BriefActionPriority::High => "high",
        manager_daily_brief::BriefActionPriority::Medium => "medium",
        manager_daily_brief::BriefActionPriority::Low => "low",
    }
}

pub(super) fn manager_brief_persona_code(
    persona: manager_daily_brief::ManagerBriefPersona,
) -> &'static str {
    match persona {
        manager_daily_brief::ManagerBriefPersona::GeneralManager => "general_manager",
        manager_daily_brief::ManagerBriefPersona::AssistantGeneralManager => {
            "assistant_general_manager"
        }
        manager_daily_brief::ManagerBriefPersona::FrontDeskLead => "front_desk_lead",
        manager_daily_brief::ManagerBriefPersona::FrontDeskAgent => "front_desk_agent",
    }
}

pub(super) fn removed_manual_work_code(
    work: manager_daily_brief::RemovedManualWork,
) -> &'static str {
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

        manager_daily_brief::RemovedManualWork::DataQualityExceptionTriage => {
            "data_quality_exception_triage"
        }
        manager_daily_brief::RemovedManualWork::ServiceCapacityLaborPlanning => {
            "service_capacity_labor_planning"
        }
    }
}

pub(super) fn source_fact_kind_code(kind: manager_daily_brief::SourceFactKind) -> &'static str {
    match kind {
        manager_daily_brief::SourceFactKind::ServiceDemandForecast => "service_demand_forecast",
        manager_daily_brief::SourceFactKind::CheckoutCompletionStatus => {
            "checkout_completion_status"
        }

        manager_daily_brief::SourceFactKind::SourceDataQualityIssue => "source_data_quality_issue",
        manager_daily_brief::SourceFactKind::CapacityLaborRecommendation => {
            "capacity_labor_recommendation"
        }
    }
}

pub(super) fn data_quality_kind_code(kind: &domain_data_quality::Kind) -> &'static str {
    match kind {
        domain_data_quality::Kind::MissingRequiredField { .. } => "missing_required_field",
        domain_data_quality::Kind::AssumptionInForce { .. } => "assumption_in_force",
        domain_data_quality::Kind::UnknownSourceStatus { .. } => "unknown_source_status",
        domain_data_quality::Kind::ConflictingTimestamps => "conflicting_timestamps",
        domain_data_quality::Kind::DuplicateSourceRecord => "duplicate_source_record",
        domain_data_quality::Kind::AmbiguousOwnerPetRelationship => {
            "ambiguous_owner_pet_relationship"
        }
        domain_data_quality::Kind::UnmappedServiceType => "unmapped_service_type",
        domain_data_quality::Kind::LocationScopeAmbiguity => "location_scope_ambiguity",
        domain_data_quality::Kind::PaymentStateConflict => "payment_state_conflict",
        domain_data_quality::Kind::CheckoutEvidenceMissing => "checkout_evidence_missing",
        domain_data_quality::Kind::UnclosedReservation => "unclosed_reservation",
        domain_data_quality::Kind::IncompletePetProfile => "incomplete_pet_profile",
        domain_data_quality::Kind::MissingVaccinationRecord => "missing_vaccination_record",
        domain_data_quality::Kind::SensitivePayloadQuarantined => "sensitive_payload_quarantined",
    }
}

pub(super) fn data_quality_severity_code(severity: domain_data_quality::Severity) -> &'static str {
    match severity {
        domain_data_quality::Severity::Informational => "informational",
        domain_data_quality::Severity::Warning => "warning",
        domain_data_quality::Severity::Blocking => "blocking",
        domain_data_quality::Severity::Critical => "critical",
    }
}

#[cfg(test)]
mod coverage_convergence_tests {
    use super::*;

    #[test]
    fn every_feedback_outcome_has_an_explicit_storage_code() {
        let mappings = [
            (
                app::manager_daily_brief::FeedbackOutcome::Completed,
                storage::operations::ManagerDailyBriefOutcomeCode::Completed,
            ),
            (
                app::manager_daily_brief::FeedbackOutcome::Deferred,
                storage::operations::ManagerDailyBriefOutcomeCode::Deferred,
            ),
            (
                app::manager_daily_brief::FeedbackOutcome::SuppressedByManager,
                storage::operations::ManagerDailyBriefOutcomeCode::SuppressedByManager,
            ),
            (
                app::manager_daily_brief::FeedbackOutcome::SourceFactWasWrong,
                storage::operations::ManagerDailyBriefOutcomeCode::SourceFactWasWrong,
            ),
        ];
        for (reported, stored) in mappings {
            assert_eq!(stored_manager_daily_brief_outcome(reported), stored);
        }
    }

    #[test]
    fn data_quality_codes_cover_every_domain_variant() {
        let kinds = [
            domain_data_quality::Kind::MissingRequiredField {
                field: domain_data_quality::FieldPath::reservation(
                    domain_data_quality::ReservationField::Status,
                ),
            },
            domain_data_quality::Kind::AssumptionInForce {
                assumption: domain::source::reservation::Assumption::RawPayloadRetentionUnknown,
            },
            domain_data_quality::Kind::UnknownSourceStatus {
                observed: domain::source::ObservedStatus::try_new("unknown").unwrap(),
            },
            domain_data_quality::Kind::ConflictingTimestamps,
            domain_data_quality::Kind::DuplicateSourceRecord,
            domain_data_quality::Kind::AmbiguousOwnerPetRelationship,
            domain_data_quality::Kind::UnmappedServiceType,
            domain_data_quality::Kind::LocationScopeAmbiguity,
            domain_data_quality::Kind::PaymentStateConflict,
            domain_data_quality::Kind::CheckoutEvidenceMissing,
            domain_data_quality::Kind::UnclosedReservation,
            domain_data_quality::Kind::IncompletePetProfile,
            domain_data_quality::Kind::MissingVaccinationRecord,
            domain_data_quality::Kind::SensitivePayloadQuarantined,
        ];
        assert!(
            kinds
                .iter()
                .all(|kind| !data_quality_kind_code(kind).is_empty())
        );
        for severity in [
            domain_data_quality::Severity::Informational,
            domain_data_quality::Severity::Warning,
            domain_data_quality::Severity::Blocking,
            domain_data_quality::Severity::Critical,
        ] {
            assert!(!data_quality_severity_code(severity).is_empty());
        }
    }
}
