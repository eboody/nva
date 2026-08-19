use super::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PermissionedKnowledgeAgentContextQuery {
    pub(super) location_id: Uuid,
    pub(super) service: String,
    pub(super) role: String,
    pub(super) section: String,
}

pub(super) async fn permissioned_knowledge_agent_context(
    Protected(authentication): Protected,
    Extension(request_trace): Extension<RequestTraceEvidence>,
    Query(query): Query<PermissionedKnowledgeAgentContextQuery>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_read(
        &authentication,
        authentication::Read::PermissionedKnowledge,
        Some(query.location_id),
        Some(&query.role),
    ) {
        return PublicApiError::new(rejection.into(), ErrorContext::new("missing_request_id"))
            .into_response();
    }
    let service = permissioned_knowledge_service(&query.service);
    let trusted_role = authentication::actor_role_claim(&authentication)
        .expect("authenticated read authorization resolved an actor role");
    let role = permissioned_knowledge_role(trusted_role);
    let context = agent::assistant::ActorContext::builder()
        .actor_id(
            access::ActorId::try_new(
                authentication::actor_id(&authentication)
                    .expect("authenticated read authorization resolved an actor id"),
            )
            .expect("trusted actor ids satisfy actor-context validation"),
        )
        .role(role)
        .title(access::Title::try_new("Fixture Knowledge Actor").unwrap())
        .location_id(entities::LocationId::new(query.location_id))
        .purpose(agent::assistant::Purpose::SopLookup)
        .allowed_uses(vec![access::AllowedUse::InternalDecisionSupport])
        .build();
    let request = app::permissioned_knowledge::Request::builder()
        .context(context)
        .service(service)
        .requested_section(agent::knowledge::SectionRef::try_new(&query.section).unwrap())
        .requested_at(Utc.with_ymd_and_hms(2026, 8, 12, 15, 0, 0).unwrap())
        .build();
    let packet = app::permissioned_knowledge::Workflow::answer(
        &app::permissioned_knowledge::DeterministicFixtureRepository,
        request,
    );
    let correlation_id = format!(
        "permissioned-knowledge:{}:{}:{}",
        query.location_id, query.service, query.section
    );

    Json(json!({
        "api_contract": api_dto_contract_payload("permissioned_knowledge_assistant_packet"),
        "workflow": {
            "name": "permissioned_knowledge_retrieval",
            "version": "local-permissioned-knowledge-context-v1"
        },
        "actor_request": {
            "location_id": query.location_id.to_string(),
            "service": query.service,
            "role": query.role,
            "purpose": "sop_lookup"
        },
        "retrieval": {
            "repository_adapter": "deterministic_fixture_repository",
            "authorization_before_context": true,
            "safe_to_enter_assistant_context": packet.safe_to_enter_assistant_context(),
            "authorized_passage_count": packet.authorized_passages().len(),
            "citation_count": packet.citations().len(),
            "claims_are_cited": packet.claims_are_cited(),
            "content_redacted_from_api_debug": true
        },
        "answer_packet": {
            "state": packet.answer_state(),
            "escalation_reason": packet.escalation_reason(),
            "claim_text_redacted": true
        },
        "safety": {
            "live_side_effects_allowed": packet.live_side_effects_allowed(),
            "forbidden_actions": packet.forbidden_actions(),
            "provider_payload_passthrough": false,
            "customer_messages_allowed": false,
            "provider_writes_allowed": false
        },
        "audit": {
            "context_packet_id": format!("permissioned-knowledge-context:{}:{}", query.location_id, query.section),
            "correlation_id": correlation_id,
            "policy_owner": "deterministic_app"
        },
        "observability": workflow_observability_payload(&correlation_id, &request_trace)
    }))
    .into_response()
}

pub(super) fn permissioned_knowledge_service(raw: &str) -> entities::ServiceKind {
    match raw {
        "grooming" => entities::ServiceKind::Grooming,
        "training" => entities::ServiceKind::Training,
        "day_play" => entities::ServiceKind::DayPlay,
        _ => entities::ServiceKind::Boarding,
    }
}

pub(super) fn permissioned_knowledge_role(raw: &str) -> access::ActorRole {
    match raw {
        "site_manager" => access::ActorRole::SiteManager,
        "marketing" => access::ActorRole::Marketing,
        "regional_operations" => access::ActorRole::RegionalOperations,
        _ => access::ActorRole::FrontDesk,
    }
}
