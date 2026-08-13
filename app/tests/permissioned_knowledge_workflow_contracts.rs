use chrono::{TimeZone, Utc};
use domain::{access, agent, entities};
use uuid::Uuid;

fn location_id() -> entities::LocationId {
    entities::LocationId(Uuid::from_u128(0x170))
}

fn alternate_location_id() -> entities::LocationId {
    entities::LocationId(Uuid::from_u128(0x171))
}

fn front_desk_context(location_id: entities::LocationId) -> agent::assistant::ActorContext {
    agent::assistant::ActorContext::builder()
        .actor_id(access::ActorId::try_new("front-desk-alice").unwrap())
        .role(access::ActorRole::FrontDesk)
        .title(access::Title::try_new("Front Desk Lead").unwrap())
        .location_id(location_id)
        .purpose(agent::assistant::Purpose::SopLookup)
        .allowed_uses(vec![access::AllowedUse::InternalDecisionSupport])
        .build()
}

#[test]
fn permissioned_knowledge_retrieval_authorizes_before_content_enters_assistant_context() {
    let request = app::permissioned_knowledge::Request::builder()
        .context(front_desk_context(location_id()))
        .service(entities::ServiceKind::Boarding)
        .requested_section(
            agent::knowledge::SectionRef::try_new("check-in.required-documents").unwrap(),
        )
        .requested_at(Utc.with_ymd_and_hms(2026, 8, 12, 15, 0, 0).unwrap())
        .build();

    let packet = app::permissioned_knowledge::Workflow::answer(
        &app::permissioned_knowledge::DeterministicFixtureRepository::default(),
        request,
    );

    assert_eq!(packet.answer_state(), agent::assistant::AnswerState::Cited);
    assert!(packet.safe_to_enter_assistant_context());
    assert_eq!(packet.authorized_passages().len(), 1);
    assert_eq!(packet.citations().len(), 1);
    assert!(packet.claims_are_cited());
    assert!(
        packet
            .forbidden_actions()
            .contains(&"send_customer_message")
    );
    assert!(packet.forbidden_actions().contains(&"provider_pms_write"));
    assert!(!packet.live_side_effects_allowed());
    assert!(!format!("{packet:?}").contains("boarding check-in checklist"));
}

#[test]
fn permissioned_knowledge_retrieval_escalates_before_context_for_location_or_role_mismatch() {
    let request = app::permissioned_knowledge::Request::builder()
        .context(front_desk_context(alternate_location_id()))
        .service(entities::ServiceKind::Boarding)
        .requested_section(
            agent::knowledge::SectionRef::try_new("check-in.required-documents").unwrap(),
        )
        .requested_at(Utc.with_ymd_and_hms(2026, 8, 12, 15, 0, 0).unwrap())
        .build();

    let packet = app::permissioned_knowledge::Workflow::answer(
        &app::permissioned_knowledge::DeterministicFixtureRepository::default(),
        request,
    );

    assert_eq!(
        packet.answer_state(),
        agent::assistant::AnswerState::Escalated
    );
    assert!(!packet.safe_to_enter_assistant_context());
    assert!(packet.authorized_passages().is_empty());
    assert_eq!(
        packet.escalation_reason(),
        Some(agent::assistant::EscalationReason::StaleOrMissingSource)
    );
}

#[test]
fn permissioned_knowledge_retrieval_escalates_conflicting_or_missing_sources_instead_of_citing() {
    let repository = app::permissioned_knowledge::DeterministicFixtureRepository::with_conflicting_boarding_fixture();
    let request = app::permissioned_knowledge::Request::builder()
        .context(front_desk_context(location_id()))
        .service(entities::ServiceKind::Boarding)
        .requested_section(
            agent::knowledge::SectionRef::try_new("check-in.required-documents").unwrap(),
        )
        .requested_at(Utc.with_ymd_and_hms(2026, 8, 12, 15, 0, 0).unwrap())
        .build();

    let conflict_packet = app::permissioned_knowledge::Workflow::answer(&repository, request);
    assert_eq!(
        conflict_packet.answer_state(),
        agent::assistant::AnswerState::Escalated
    );
    assert_eq!(
        conflict_packet.escalation_reason(),
        Some(agent::assistant::EscalationReason::ConflictingSources)
    );
    assert!(conflict_packet.citations().is_empty());

    let missing_request = app::permissioned_knowledge::Request::builder()
        .context(front_desk_context(location_id()))
        .service(entities::ServiceKind::Grooming)
        .requested_section(
            agent::knowledge::SectionRef::try_new("check-in.required-documents").unwrap(),
        )
        .requested_at(Utc.with_ymd_and_hms(2026, 8, 12, 15, 0, 0).unwrap())
        .build();
    let missing_packet = app::permissioned_knowledge::Workflow::answer(
        &app::permissioned_knowledge::DeterministicFixtureRepository::default(),
        missing_request,
    );
    assert_eq!(
        missing_packet.answer_state(),
        agent::assistant::AnswerState::Escalated
    );
    assert_eq!(
        missing_packet.escalation_reason(),
        Some(agent::assistant::EscalationReason::StaleOrMissingSource)
    );
}
