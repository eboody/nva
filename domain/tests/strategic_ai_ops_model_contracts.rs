use chrono::{TimeZone, Utc};
use domain::{
    access, agent, analytics, consent, customer, entities, identity, lead, operations, policy,
    source,
};
use uuid::Uuid;

fn location_id() -> entities::LocationId {
    entities::LocationId::new(Uuid::from_u128(0x170))
}

fn customer_id() -> entities::CustomerId {
    entities::CustomerId::new(Uuid::from_u128(0xc0570))
}

fn pet_id() -> entities::PetId {
    entities::PetId::new(Uuid::from_u128(0x0d06))
}

#[test]
fn realtime_lead_model_tracks_sla_contact_attempts_consent_and_conversion_attribution() {
    let received_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 0, 0).unwrap();
    let due_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 5, 0).unwrap();

    let event = lead::response::Event::builder()
        .id(lead::response::EventId::try_new("missed-call-42").unwrap())
        .idempotency_key(
            lead::response::IdempotencyKey::try_new("masked-phone-loc170-20260812T1400").unwrap(),
        )
        .location_id(location_id())
        .kind(lead::response::EventKind::MissedCall)
        .received_at(received_at)
        .source_system(source::System::Telephony)
        .customer_match(identity::Match::Candidate {
            customer_id: customer_id(),
            confidence: identity::Confidence::High,
        })
        .service_intent(entities::ServiceKind::Boarding)
        .build();

    let sla = lead::response::ResponseSla::builder()
        .target(lead::response::SlaTarget::FirstResponseWithinMinutes(
            lead::response::Minutes::try_new(5).unwrap(),
        ))
        .received_at(received_at)
        .due_at(due_at)
        .status(lead::response::SlaStatus::Open)
        .build()
        .unwrap();

    let attempt = lead::response::ContactAttempt::builder()
        .attempted_at(Utc.with_ymd_and_hms(2026, 8, 12, 14, 2, 0).unwrap())
        .channel(consent::Channel::Sms)
        .purpose(consent::Purpose::TransactionalLeadResponse)
        .outcome(lead::response::AttemptOutcome::DraftedForReview)
        .review_gate(policy::ReviewGate::CustomerMessageApproval)
        .build();

    let packet = lead::response::ResponsePacket::builder()
        .event(event)
        .sla(sla)
        .consent(lead_consent(
            received_at,
            consent::Channel::Sms,
            consent::Purpose::TransactionalLeadResponse,
            consent::ConsentStatus::Granted,
        ))
        .attempts(vec![attempt])
        .attribution(
            lead::response::ConversionObservation::builder()
                .source(lead::response::AttributionSource::MissedCall)
                .estimated_value(money(45_000))
                .build(),
        )
        .build()
        .unwrap();

    assert!(
        packet.first_response_sla_is_met_at(Utc.with_ymd_and_hms(2026, 8, 12, 14, 3, 0).unwrap())
    );
    assert!(packet.requires_customer_message_approval());
    assert!(packet.consent_allows_response(
        consent::Channel::Sms,
        consent::Purpose::TransactionalLeadResponse
    ));
    assert_eq!(packet.event().source_system(), source::System::Telephony);
}

#[test]
fn lead_response_packet_rejects_sla_due_time_that_disagrees_with_event_receipt() {
    let received_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 0, 0).unwrap();
    let wrong_due_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 6, 0).unwrap();

    assert!(matches!(
        lead::response::ResponseSla::try_new(
            lead::response::SlaTarget::FirstResponseWithinMinutes(
                lead::response::Minutes::try_new(5).unwrap(),
            ),
            received_at,
            wrong_due_at,
            lead::response::SlaStatus::Open,
        ),
        Err(lead::response::Error::SlaDueAtDoesNotMatchTarget)
    ));
}

#[test]
fn lead_response_packet_rejects_attempts_before_receipt_or_out_of_order() {
    let received_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 0, 0).unwrap();
    let before_receipt = lead_attempt_at(Utc.with_ymd_and_hms(2026, 8, 12, 13, 59, 0).unwrap());
    let later = lead_attempt_at(Utc.with_ymd_and_hms(2026, 8, 12, 14, 3, 0).unwrap());
    let earlier = lead_attempt_at(Utc.with_ymd_and_hms(2026, 8, 12, 14, 2, 0).unwrap());

    assert!(matches!(
        lead_response_packet_with(received_at, vec![before_receipt]),
        Err(lead::response::Error::ContactAttemptBeforeReceipt)
    ));
    assert!(matches!(
        lead_response_packet_with(received_at, vec![later, earlier]),
        Err(lead::response::Error::ContactAttemptsOutOfOrder)
    ));
}

#[test]
fn lead_response_packet_rejects_channel_purpose_mismatch_or_opt_out() {
    let received_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 0, 0).unwrap();
    let attempt = lead_attempt_at(Utc.with_ymd_and_hms(2026, 8, 12, 14, 2, 0).unwrap());

    assert!(matches!(
        lead_response_packet_with_consent(
            received_at,
            lead_consent(
                received_at,
                consent::Channel::Email,
                consent::Purpose::TransactionalLeadResponse,
                consent::ConsentStatus::Granted,
            ),
            vec![attempt.clone()],
        ),
        Err(lead::response::Error::ConsentDoesNotCoverAttempt)
    ));
    assert!(matches!(
        lead_response_packet_with_consent(
            received_at,
            lead_consent(
                received_at,
                consent::Channel::Sms,
                consent::Purpose::TransactionalLeadResponse,
                consent::ConsentStatus::OptedOut,
            ),
            vec![attempt],
        ),
        Err(lead::response::Error::ConsentDoesNotCoverAttempt)
    ));
}

#[test]
fn lead_response_packet_rejects_empty_attempts_at_builder_and_serde_boundaries() {
    let received_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 0, 0).unwrap();

    assert!(matches!(
        lead_response_packet_with(received_at, vec![]),
        Err(lead::response::Error::MissingContactAttempt)
    ));

    let valid_packet = lead_response_packet_with(
        received_at,
        vec![lead_attempt_at(
            Utc.with_ymd_and_hms(2026, 8, 12, 14, 2, 0).unwrap(),
        )],
    )
    .unwrap();
    let mut raw = serde_json::to_value(&valid_packet).unwrap();
    raw["attempts"] = serde_json::json!([]);

    assert!(serde_json::from_value::<lead::response::ResponsePacket>(raw).is_err());
}

#[test]
fn lead_response_review_evidence_remains_serializable_without_queue_authority() {
    let received_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 0, 0).unwrap();
    let message_ref = domain::message::BodyRef::try_new("draft-body-42").unwrap();
    let packet = lead_response_packet_with(
        received_at,
        vec![
            lead_attempt_at(Utc.with_ymd_and_hms(2026, 8, 12, 14, 2, 0).unwrap())
                .with_message_ref(message_ref.clone()),
        ],
    )
    .unwrap();

    let review_evidence = lead::response::ReviewApprovalEvidence::builder()
        .gate(policy::ReviewGate::CustomerMessageApproval)
        .location_id(location_id())
        .customer_id(customer_id())
        .service_intent(entities::ServiceKind::Boarding)
        .channel(consent::Channel::Sms)
        .purpose(consent::Purpose::TransactionalLeadResponse)
        .message_ref(message_ref)
        .build();

    assert!(serde_json::to_value(review_evidence).is_ok());
    assert!(packet.requires_customer_message_approval());
}

#[test]
fn lead_response_booking_observation_requires_reservation_evidence_and_stays_nonclaimable() {
    assert!(matches!(
        lead::response::ConversionObservation::try_reported_booking_observation(
            lead::response::AttributionSource::MissedCall,
            None,
            None,
            Some(money(45_000)),
        ),
        Err(lead::response::Error::ConvertedLeadRequiresReservation)
    ));

    let observation = lead::response::ConversionObservation::try_reported_booking_observation(
        lead::response::AttributionSource::MissedCall,
        None,
        Some(entities::reservation::Id::new(Uuid::from_u128(99))),
        Some(money(45_000)),
    )
    .unwrap();
    assert!(!observation.can_support_value_claim());
}

#[test]
fn lead_response_packet_deserialize_rejects_invalid_actionable_relationships_and_redacts_idempotency_key()
 {
    let received_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 0, 0).unwrap();
    let packet = lead_response_packet_with(
        received_at,
        vec![lead_attempt_at(
            Utc.with_ymd_and_hms(2026, 8, 12, 14, 2, 0).unwrap(),
        )],
    )
    .unwrap();
    let mut raw = serde_json::to_value(&packet).unwrap();
    raw["sla"]["due_at"] =
        serde_json::to_value(Utc.with_ymd_and_hms(2026, 8, 12, 14, 6, 0).unwrap()).unwrap();

    assert!(serde_json::from_value::<lead::response::ResponsePacket>(raw).is_err());
    assert!(!format!("{:?}", packet.event().idempotency_key()).contains("masked-phone"));
}

#[test]
fn lead_sla_rejects_pre_receipt_and_breached_response_times() {
    let received_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 0, 0).unwrap();
    let due_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 5, 0).unwrap();

    let open_sla = lead::response::ResponseSla::builder()
        .target(lead::response::SlaTarget::FirstResponseWithinMinutes(
            lead::response::Minutes::try_new(5).unwrap(),
        ))
        .received_at(received_at)
        .due_at(due_at)
        .status(lead::response::SlaStatus::Open)
        .build()
        .unwrap();

    assert!(!open_sla.is_met_at(Utc.with_ymd_and_hms(2026, 8, 12, 13, 59, 0).unwrap()));

    let breached_sla = lead::response::ResponseSla::builder()
        .target(lead::response::SlaTarget::FirstResponseWithinMinutes(
            lead::response::Minutes::try_new(5).unwrap(),
        ))
        .received_at(received_at)
        .due_at(due_at)
        .status(lead::response::SlaStatus::Breached)
        .build()
        .unwrap();

    assert!(!breached_sla.is_met_at(Utc.with_ymd_and_hms(2026, 8, 12, 14, 3, 0).unwrap()));
}

#[test]
fn crm_intelligence_model_separates_operations_personalization_from_marketing_segments() {
    let now = Utc.with_ymd_and_hms(2026, 8, 12, 15, 0, 0).unwrap();
    let note = customer::intelligence::StructuredNote::builder()
        .id(customer::intelligence::NoteId::try_new("note-123").unwrap())
        .customer_id(customer_id())
        .pet_id(pet_id())
        .kind(customer::intelligence::NoteKind::PetHandlingPreference)
        .body(
            customer::intelligence::NoteBody::try_new(
                "Bella prefers a quiet handoff and peanut-butter Kong.",
            )
            .unwrap(),
        )
        .visibility(access::VisibilityScope::OperationsOnly)
        .allowed_uses(vec![access::AllowedUse::ServicePersonalization])
        .source(customer::intelligence::SignalSource::StaffObservation)
        .confidence(identity::Confidence::Medium)
        .review_state(customer::intelligence::ReviewState::Accepted)
        .reviewed_by(access::ActorId::try_new("manager-approval-1").unwrap())
        .effective_interval(active_interval(now))
        .recorded_at(now)
        .build();

    assert!(note.can_support(access::AllowedUse::ServicePersonalization));
    assert!(!note.can_support(access::AllowedUse::MarketingCampaign));
    assert_eq!(
        note.review_state(),
        customer::intelligence::ReviewState::Accepted
    );
    assert!(!format!("{note:?}").contains("Bella prefers"));
}

#[test]
fn crm_segment_membership_requires_opaque_accepted_evidence() {
    let now = Utc.with_ymd_and_hms(2026, 8, 12, 15, 0, 0).unwrap();

    assert!(matches!(
        customer::intelligence::SegmentMembership::try_new(
            customer_id(),
            marketing_segment_definition(),
            vec![customer::intelligence::SegmentBasis::ComplaintResolvedRecently],
            active_interval(now),
            vec![],
        ),
        Err(customer::intelligence::Error::MissingEvidence)
    ));
}

#[test]
fn consent_observation_binds_exact_customer_scope_and_current_interval() {
    let now = Utc.with_ymd_and_hms(2026, 8, 12, 15, 0, 0).unwrap();
    let evidence = marketing_consent();
    assert!(evidence.permits_customer(
        customer_id(),
        consent::Channel::Email,
        consent::Purpose::MarketingRetention,
        now,
    ));
    assert!(!evidence.permits_customer(
        entities::CustomerId::new(Uuid::from_u128(999)),
        consent::Channel::Email,
        consent::Purpose::MarketingRetention,
        now,
    ));
}

#[test]
fn capacity_labor_model_carries_time_bucket_constraints_shift_cost_and_reviewed_recommendation() {
    let bucket = operations::time_bucket::Window::new(
        Utc.with_ymd_and_hms(2026, 8, 12, 7, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 8, 12, 11, 0, 0).unwrap(),
    )
    .unwrap();

    let demand = operations::capacity::DemandUnit::builder()
        .location_id(location_id())
        .service(entities::ServiceKind::Boarding)
        .bucket(bucket)
        .quantity(operations::capacity::Quantity::try_new(25).unwrap())
        .labor_minutes_per_unit(operations::labor::Minutes::try_new(12).unwrap())
        .constraints(vec![
            operations::capacity::Constraint::CheckInCheckoutBottleneck,
        ])
        .build();

    let coverage = operations::labor::ScheduledCoverage::builder()
        .location_id(location_id())
        .bucket(bucket)
        .role(operations::labor::Role::FrontDesk)
        .scheduled_people(operations::labor::PeopleCount::try_new(1).unwrap())
        .scheduled_minutes(operations::labor::Minutes::try_new(240).unwrap())
        .loaded_cost(money(7_200))
        .build();

    let recommendation = operations::capacity::OptimizationRecommendation::builder()
        .objective(operations::capacity::OptimizationObjective::ReduceFrontDeskBottleneck)
        .demand(demand)
        .coverage(coverage)
        .source_evidence(vec![capacity_provenance(
            "demand-forecast-legacy",
            source::System::BusinessIntelligence,
        )])
        .action(operations::capacity::RecommendedAction::AddRoleCoverage {
            role: operations::labor::Role::FrontDesk,
            minutes: operations::labor::Minutes::try_new(60).unwrap(),
        })
        .expected_labor_delta_minutes(operations::labor::SignedMinutes::new(60))
        .review_gate(policy::ReviewGate::ManagerApproval)
        .build()
        .unwrap();

    assert_eq!(
        recommendation.review_gate(),
        policy::ReviewGate::ManagerApproval
    );
    assert!(recommendation.blocks_live_schedule_change());
}

#[test]
fn capacity_labor_recommendation_requires_same_grain_source_evidence_feasibility_and_manager_review()
 {
    let bucket = capacity_bucket();
    let demand = capacity_demand_for(location_id(), entities::ServiceKind::Boarding, bucket);
    let coverage = scheduled_coverage_for(
        location_id(),
        operations::labor::Role::FrontDesk,
        bucket,
        240,
    );

    let recommendation = operations::capacity::OptimizationRecommendation::try_new(
        operations::capacity::OptimizationObjective::ReduceFrontDeskBottleneck,
        demand.clone(),
        coverage.clone(),
        vec![capacity_provenance(
            "demand-forecast-1",
            source::System::BusinessIntelligence,
        )],
        operations::capacity::SolverStatus::Feasible,
        vec![operations::capacity::RecommendedAction::ManagerReviewOnly],
        operations::capacity::RecommendedAction::add_role_coverage(
            operations::labor::Role::FrontDesk,
            operations::labor::Minutes::try_new(72).unwrap(),
        ),
        policy::ReviewGate::ManagerApproval,
    )
    .unwrap();

    assert_eq!(
        recommendation.expected_labor_delta_minutes(),
        operations::labor::SignedMinutes::new(72)
    );
    assert_eq!(recommendation.source_evidence().len(), 1);
    assert_eq!(
        recommendation.solver_status(),
        operations::capacity::SolverStatus::Feasible
    );
    assert_eq!(recommendation.alternatives().len(), 1);
    assert!(recommendation.blocks_live_schedule_change());

    assert!(matches!(
        operations::capacity::OptimizationRecommendation::try_new(
            operations::capacity::OptimizationObjective::ReduceFrontDeskBottleneck,
            demand.clone(),
            scheduled_coverage_for(
                entities::LocationId::new(Uuid::from_u128(0x171)),
                operations::labor::Role::FrontDesk,
                bucket,
                240,
            ),
            vec![capacity_provenance(
                "demand-forecast-1",
                source::System::BusinessIntelligence
            )],
            operations::capacity::SolverStatus::Feasible,
            vec![],
            operations::capacity::RecommendedAction::add_role_coverage(
                operations::labor::Role::FrontDesk,
                operations::labor::Minutes::try_new(72).unwrap(),
            ),
            policy::ReviewGate::ManagerApproval,
        ),
        Err(operations::capacity::Error::LocationMismatch)
    ));

    assert!(matches!(
        operations::capacity::OptimizationRecommendation::try_new(
            operations::capacity::OptimizationObjective::ReduceFrontDeskBottleneck,
            demand.clone(),
            coverage.clone(),
            vec![],
            operations::capacity::SolverStatus::Feasible,
            vec![],
            operations::capacity::RecommendedAction::add_role_coverage(
                operations::labor::Role::FrontDesk,
                operations::labor::Minutes::try_new(72).unwrap(),
            ),
            policy::ReviewGate::ManagerApproval,
        ),
        Err(operations::capacity::Error::MissingSourceEvidence)
    ));

    assert!(matches!(
        operations::capacity::OptimizationRecommendation::try_new(
            operations::capacity::OptimizationObjective::ReduceFrontDeskBottleneck,
            demand.clone(),
            coverage.clone(),
            vec![capacity_provenance(
                "demand-forecast-1",
                source::System::BusinessIntelligence
            )],
            operations::capacity::SolverStatus::Infeasible {
                reason: operations::capacity::InfeasibilityReason::InsufficientQualifiedStaff,
            },
            vec![],
            operations::capacity::RecommendedAction::add_role_coverage(
                operations::labor::Role::FrontDesk,
                operations::labor::Minutes::try_new(72).unwrap(),
            ),
            policy::ReviewGate::ManagerApproval,
        ),
        Err(operations::capacity::Error::InfeasibleInputs)
    ));

    assert!(matches!(
        operations::capacity::OptimizationRecommendation::try_new(
            operations::capacity::OptimizationObjective::ReduceFrontDeskBottleneck,
            demand.clone(),
            coverage,
            vec![capacity_provenance(
                "demand-forecast-1",
                source::System::BusinessIntelligence
            )],
            operations::capacity::SolverStatus::Feasible,
            vec![],
            operations::capacity::RecommendedAction::add_role_coverage(
                operations::labor::Role::Trainer,
                operations::labor::Minutes::try_new(72).unwrap(),
            ),
            policy::ReviewGate::ManagerApproval,
        ),
        Err(operations::capacity::Error::RoleMismatch)
    ));

    assert!(matches!(
        operations::capacity::OptimizationRecommendation::try_new(
            operations::capacity::OptimizationObjective::ReduceFrontDeskBottleneck,
            demand,
            scheduled_coverage_for(
                location_id(),
                operations::labor::Role::FrontDesk,
                bucket,
                240,
            ),
            vec![capacity_provenance(
                "demand-forecast-1",
                source::System::BusinessIntelligence
            )],
            operations::capacity::SolverStatus::Feasible,
            vec![],
            operations::capacity::RecommendedAction::add_role_coverage(
                operations::labor::Role::FrontDesk,
                operations::labor::Minutes::try_new(60).unwrap(),
            ),
            policy::ReviewGate::CustomerMessageApproval,
        ),
        Err(operations::capacity::Error::ManagerReviewRequired)
    ));
}

#[test]
fn knowledge_metadata_remains_serializable_evidence_without_retrieval_authority() {
    let now = Utc.with_ymd_and_hms(2026, 8, 12, 15, 0, 0).unwrap();
    let section = agent::knowledge::SectionRef::try_new("check-in.required-documents").unwrap();
    let document = agent::knowledge::Document::builder()
        .id(agent::knowledge::DocumentId::try_new("sop-boarding-v1").unwrap())
        .title(agent::knowledge::Title::try_new("Boarding Check-in SOP").unwrap())
        .kind(agent::knowledge::DocumentKind::Sop)
        .status(agent::knowledge::ApprovalStatus::Approved)
        .applicability(
            agent::knowledge::Applicability::builder()
                .locations(vec![location_id()])
                .services(vec![entities::ServiceKind::Boarding])
                .roles(vec![access::ActorRole::FrontDesk])
                .build(),
        )
        .sections(vec![section.clone()])
        .effective_at(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap())
        .review_due_at(Utc.with_ymd_and_hms(2027, 1, 1, 0, 0, 0).unwrap())
        .build();
    let context = agent_context_for(
        access::ActorRole::FrontDesk,
        location_id(),
        agent::assistant::Purpose::SopLookup,
        vec![access::AllowedUse::InternalDecisionSupport],
    );
    assert!(document.applies_to(
        location_id(),
        entities::ServiceKind::Boarding,
        access::ActorRole::FrontDesk
    ));
    assert!(document.is_current_at(now));
    assert!(context.allows_use(access::AllowedUse::InternalDecisionSupport));
    assert!(serde_json::to_value(&document).is_ok());
    assert!(serde_json::to_value(&context).is_ok());

    let citation = agent::knowledge::Citation::builder()
        .document_id(document.id().clone())
        .section(section)
        .build();
    let claim = agent::assistant::Claim::builder()
        .id(agent::assistant::ClaimId::try_new("claim-checkin-docs").unwrap())
        .citation(citation.clone())
        .build();
    assert!(matches!(
        agent::assistant::AnswerPacket::try_cited(
            context,
            agent::assistant::AnswerText::try_new("Unaccepted metadata cannot answer.").unwrap(),
            vec![claim],
            vec![],
            vec![citation],
            identity::Confidence::High,
        ),
        Err(agent::assistant::Error::MissingAuthorizedEvidence)
    ));
}

#[test]
fn assistant_answer_deserialize_rejects_cited_state_without_authorized_evidence() {
    let now = Utc.with_ymd_and_hms(2026, 8, 12, 15, 0, 0).unwrap();
    let document_id = agent::knowledge::DocumentId::try_new("sop-boarding-v1").unwrap();
    let section = agent::knowledge::SectionRef::try_new("check-in.required-documents").unwrap();
    let unsafe_cited_without_evidence = serde_json::json!({
        "context": agent_context_for(
            access::ActorRole::FrontDesk,
            location_id(),
            agent::assistant::Purpose::SopLookup,
            vec![access::AllowedUse::InternalDecisionSupport],
        ),
        "answer": "Looks cited but no authorized retrieval happened.",
        "state": "Cited",
        "claims": [{
            "id": "claim-checkin-docs",
            "citation": {
                "document_id": document_id,
                "section": section,
            },
        }],
        "authorized_evidence": [],
        "citations": [{
            "document_id": agent::knowledge::DocumentId::try_new("sop-boarding-v1").unwrap(),
            "section": agent::knowledge::SectionRef::try_new("check-in.required-documents").unwrap(),
        }],
        "confidence": "High",
        "escalation": null,
    });

    assert!(
        serde_json::from_value::<agent::assistant::AnswerPacket>(unsafe_cited_without_evidence)
            .is_err()
    );

    let valid_escalated = agent::assistant::AnswerPacket::escalated(
        agent_context_for(
            access::ActorRole::FrontDesk,
            location_id(),
            agent::assistant::Purpose::SopLookup,
            vec![access::AllowedUse::InternalDecisionSupport],
        ),
        agent::assistant::AnswerText::try_new("Escalate missing source.").unwrap(),
        identity::Confidence::Medium,
        agent::assistant::EscalationReason::StaleOrMissingSource,
    );
    let encoded = serde_json::to_value(&valid_escalated).unwrap();
    let decoded = serde_json::from_value::<agent::assistant::AnswerPacket>(encoded).unwrap();

    assert_eq!(decoded.state(), agent::assistant::AnswerState::Escalated);
    assert!(!decoded.is_cited());
    assert!(valid_escalated.authorized_evidence().is_empty());
    assert!(now >= Utc.with_ymd_and_hms(2026, 8, 12, 0, 0, 0).unwrap());
}

fn agent_context_for(
    role: access::ActorRole,
    location_id: entities::LocationId,
    purpose: agent::assistant::Purpose,
    allowed_uses: Vec<access::AllowedUse>,
) -> agent::assistant::ActorContext {
    agent::assistant::ActorContext::builder()
        .actor_id(access::ActorId::try_new("knowledge-actor").unwrap())
        .role(role)
        .title(access::Title::try_new("Knowledge Actor").unwrap())
        .location_id(location_id)
        .purpose(purpose)
        .allowed_uses(allowed_uses)
        .build()
}

#[test]
fn strategic_source_system_is_the_canonical_provenance_system() {
    let source_system: domain::source::System = source::System::Telephony;
    assert_eq!(source_system, domain::source::System::Telephony);
}

#[test]
fn strategic_communication_channels_use_canonical_message_channels_and_exact_consent() {
    let channel: domain::message::Channel = consent::Channel::Sms;
    assert_eq!(channel, domain::message::Channel::Sms);

    let consent = consent::ConsentEvidence::builder()
        .channel(consent::Channel::Sms)
        .purpose(consent::Purpose::TransactionalLeadResponse)
        .status(consent::ConsentStatus::Granted)
        .source(source::System::Crm)
        .build();

    assert!(consent.permits(
        consent::Channel::Sms,
        consent::Purpose::TransactionalLeadResponse
    ));
    assert!(!consent.permits(
        consent::Channel::Email,
        consent::Purpose::TransactionalLeadResponse
    ));
    assert!(!consent.permits(consent::Channel::Sms, consent::Purpose::MarketingRetention));
}

#[test]
fn strategic_staff_roles_promote_only_when_they_are_site_labor_roles() {
    let front_desk: domain::staff::Role = access::ActorRole::FrontDesk.try_into().unwrap();
    assert_eq!(front_desk, domain::staff::Role::FrontDesk);

    let care_staff: domain::staff::Role = access::ActorRole::CareStaff.try_into().unwrap();
    assert_eq!(care_staff, domain::staff::Role::KennelTechnician);

    assert!(domain::staff::Role::try_from(access::ActorRole::RegionalOperations).is_err());
    assert!(domain::staff::Role::try_from(access::ActorRole::Finance).is_err());
    assert!(domain::staff::Role::try_from(access::ActorRole::Marketing).is_err());
}

#[test]
fn strategic_actor_context_promotes_to_canonical_actor_ref_only_for_matching_roles() {
    let front_desk_context = agent::assistant::ActorContext::builder()
        .actor_id(access::ActorId::try_new("staff-front-desk-1").unwrap())
        .role(access::ActorRole::FrontDesk)
        .title(access::Title::try_new("Front Desk").unwrap())
        .location_id(location_id())
        .purpose(agent::assistant::Purpose::SopLookup)
        .build();

    assert_eq!(front_desk_context.location_id(), location_id());
    assert_eq!(
        front_desk_context.promote_staff_actor_ref().unwrap(),
        entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("staff-front-desk-1").unwrap()
        }
    );
    assert!(front_desk_context.promote_manager_actor_ref().is_err());

    let manager_context = agent::assistant::ActorContext::builder()
        .actor_id(access::ActorId::try_new("manager-1").unwrap())
        .role(access::ActorRole::SiteManager)
        .title(access::Title::try_new("Site Manager").unwrap())
        .location_id(location_id())
        .purpose(agent::assistant::Purpose::SiteOpsSupport)
        .build();

    assert_eq!(
        manager_context.promote_manager_actor_ref().unwrap(),
        entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("manager-1").unwrap()
        }
    );
}

fn lead_response_event(received_at: chrono::DateTime<Utc>) -> lead::response::Event {
    lead::response::Event::builder()
        .id(lead::response::EventId::try_new("missed-call-42").unwrap())
        .idempotency_key(
            lead::response::IdempotencyKey::try_new("masked-phone-loc170-20260812T1400").unwrap(),
        )
        .location_id(location_id())
        .kind(lead::response::EventKind::MissedCall)
        .received_at(received_at)
        .source_system(source::System::Telephony)
        .customer_match(identity::Match::Candidate {
            customer_id: customer_id(),
            confidence: identity::Confidence::High,
        })
        .service_intent(entities::ServiceKind::Boarding)
        .build()
}

fn lead_attempt_at(attempted_at: chrono::DateTime<Utc>) -> lead::response::ContactAttempt {
    lead::response::ContactAttempt::try_new(
        attempted_at,
        consent::Channel::Sms,
        consent::Purpose::TransactionalLeadResponse,
        lead::response::AttemptOutcome::DraftedForReview,
        policy::ReviewGate::CustomerMessageApproval,
    )
    .unwrap()
}

fn lead_response_packet_with(
    received_at: chrono::DateTime<Utc>,
    attempts: Vec<lead::response::ContactAttempt>,
) -> Result<lead::response::ResponsePacket, lead::response::Error> {
    lead_response_packet_with_consent(
        received_at,
        lead_consent(
            received_at,
            consent::Channel::Sms,
            consent::Purpose::TransactionalLeadResponse,
            consent::ConsentStatus::Granted,
        ),
        attempts,
    )
}

fn lead_response_packet_with_consent(
    received_at: chrono::DateTime<Utc>,
    consent: consent::ConsentEvidence,
    attempts: Vec<lead::response::ContactAttempt>,
) -> Result<lead::response::ResponsePacket, lead::response::Error> {
    lead::response::ResponsePacket::try_new(
        lead_response_event(received_at),
        lead::response::ResponseSla::try_new(
            lead::response::SlaTarget::FirstResponseWithinMinutes(
                lead::response::Minutes::try_new(5).unwrap(),
            ),
            received_at,
            received_at + chrono::Duration::minutes(5),
            lead::response::SlaStatus::Open,
        )
        .unwrap(),
        consent,
        attempts,
        lead::response::ConversionObservation::builder()
            .source(lead::response::AttributionSource::MissedCall)
            .estimated_value(money(45_000))
            .build(),
    )
}

fn lead_consent(
    received_at: chrono::DateTime<Utc>,
    channel: consent::Channel,
    purpose: consent::Purpose,
    status: consent::ConsentStatus,
) -> consent::ConsentEvidence {
    consent::ConsentEvidence::builder()
        .channel(channel)
        .purpose(purpose)
        .status(status)
        .source(source::System::Crm)
        .subject(consent::Subject::SourceRecord(
            lead_response_event(received_at).source_record(),
        ))
        .source_record(source::RecordRef::new(
            source::System::Crm,
            source::record::Id::try_new("consent-lead-missed-call-42").unwrap(),
        ))
        .source_schema_version(source::SchemaVersion::try_new("crm-consent-v1").unwrap())
        .effective_from(received_at - chrono::Duration::days(1))
        .effective_until(received_at + chrono::Duration::days(1))
        .build()
}

fn marketing_segment_definition() -> customer::intelligence::SegmentDefinition {
    customer::intelligence::SegmentDefinition::builder()
        .segment(customer::intelligence::Segment::ServiceRecoveryWatchlist)
        .version(customer::intelligence::SegmentVersion::try_new("service-recovery-v1").unwrap())
        .allowed_use(access::AllowedUse::MarketingCampaign)
        .visibility(access::VisibilityScope::MarketingEligible)
        .review_gate(policy::ReviewGate::ManagerApproval)
        .build()
}

fn active_interval(now: chrono::DateTime<Utc>) -> customer::intelligence::EffectiveInterval {
    customer::intelligence::EffectiveInterval::try_new(
        now - chrono::Duration::days(1),
        Some(now + chrono::Duration::days(30)),
    )
    .unwrap()
}

fn marketing_consent() -> consent::ConsentEvidence {
    let source_record = source::RecordRef::new(
        source::System::Crm,
        source::record::Id::try_new("consent-customer-1").unwrap(),
    );
    consent::ConsentEvidence::builder()
        .channel(consent::Channel::Email)
        .purpose(consent::Purpose::MarketingRetention)
        .status(consent::ConsentStatus::Granted)
        .source(source::System::Crm)
        .subject(consent::Subject::Customer(customer_id()))
        .source_record(source_record)
        .source_schema_version(source::SchemaVersion::try_new("crm-consent-v1").unwrap())
        .effective_from(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap())
        .build()
}

fn capacity_bucket() -> operations::time_bucket::Window {
    operations::time_bucket::Window::new(
        Utc.with_ymd_and_hms(2026, 8, 12, 7, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 8, 12, 11, 0, 0).unwrap(),
    )
    .unwrap()
}

fn capacity_demand_for(
    location_id: entities::LocationId,
    service: entities::ServiceKind,
    bucket: operations::time_bucket::Window,
) -> operations::capacity::DemandUnit {
    operations::capacity::DemandUnit::builder()
        .location_id(location_id)
        .service(service)
        .bucket(bucket)
        .quantity(operations::capacity::Quantity::try_new(26).unwrap())
        .labor_minutes_per_unit(operations::labor::Minutes::try_new(12).unwrap())
        .constraints(vec![
            operations::capacity::Constraint::CheckInCheckoutBottleneck,
        ])
        .build()
}

fn scheduled_coverage_for(
    location_id: entities::LocationId,
    role: operations::labor::Role,
    bucket: operations::time_bucket::Window,
    minutes: u16,
) -> operations::labor::ScheduledCoverage {
    operations::labor::ScheduledCoverage::builder()
        .location_id(location_id)
        .bucket(bucket)
        .role(role)
        .scheduled_people(operations::labor::PeopleCount::try_new(1).unwrap())
        .scheduled_minutes(operations::labor::Minutes::try_new(minutes).unwrap())
        .loaded_cost(money(7_200))
        .build()
}

fn capacity_provenance(record_id: &str, system: source::System) -> source::Provenance {
    source::Provenance::builder()
        .system(system)
        .endpoint(source::Endpoint::try_new("capacity-labor-forecast").unwrap())
        .record_id(source::record::Id::try_new(record_id).unwrap())
        .extraction_batch(source::ExtractionBatchId::try_new("capacity-labor-batch-1").unwrap())
        .pulled_at(source::Timestamp::try_new("2026-08-12T06:00:00Z").unwrap())
        .request_scope(source::RequestScope::try_new("manager-daily-brief-capacity-labor").unwrap())
        .schema_version(source::SchemaVersion::try_new("capacity-labor-v1").unwrap())
        .payload_hash(source::PayloadHash::try_new("sha256:capacity-labor-proof").unwrap())
        .raw_payload_ref(
            source::RawPayloadRef::try_new("s3://fixture/capacity-labor.json").unwrap(),
        )
        .build()
}

fn money(cents: u64) -> domain::money::Money {
    domain::money::Money::usd(cents).unwrap()
}

#[test]
fn financial_insight_model_is_site_period_source_backed_and_blocks_price_discount_payment_changes()
{
    let period = analytics::finance::SitePeriod::builder()
        .location_id(location_id())
        .start(Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap())
        .end(Utc.with_ymd_and_hms(2026, 8, 31, 23, 59, 59).unwrap())
        .build()
        .unwrap();

    let fact = analytics::finance::ReportedRevenueObservationFact::builder()
        .period(period)
        .service(entities::ServiceKind::Grooming)
        .gross_revenue(money(900_000))
        .discount(money(75_000))
        .refund(money(20_000))
        .labor_cost(money(310_000))
        .source_system(source::System::FinanceAccounting)
        .build();

    let insight = analytics::finance::Insight::builder()
        .fact(fact)
        .kind(analytics::finance::InsightKind::DiscountLeakage)
        .expected_impact(money(50_000))
        .recommendation(analytics::finance::Recommendation::ReviewDiscountPolicy)
        .review_gate(policy::ReviewGate::ManagerApproval)
        .build()
        .unwrap();

    assert_eq!(insight.net_revenue().unwrap().minor_units().get(), 805_000);
    assert!(insight.blocks_financial_mutation());
}

#[test]
fn financial_insight_requires_manager_review_gate_before_recommendation_use() {
    let period = analytics::finance::SitePeriod::builder()
        .location_id(location_id())
        .start(Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap())
        .end(Utc.with_ymd_and_hms(2026, 8, 31, 23, 59, 59).unwrap())
        .build()
        .unwrap();
    let fact = analytics::finance::ReportedRevenueObservationFact::builder()
        .period(period)
        .service(entities::ServiceKind::Grooming)
        .gross_revenue(money(900_000))
        .discount(money(75_000))
        .refund(money(20_000))
        .labor_cost(money(310_000))
        .source_system(source::System::FinanceAccounting)
        .build();

    assert!(matches!(
        analytics::finance::Insight::try_new(
            fact,
            analytics::finance::InsightKind::DiscountLeakage,
            money(50_000),
            analytics::finance::Recommendation::ReviewDiscountPolicy,
            policy::ReviewGate::CustomerMessageApproval,
        ),
        Err(analytics::finance::Error::ManagerReviewRequired)
    ));
}

#[test]
fn canonical_money_supports_zero_nonnegative_checked_arithmetic_and_serde_boundaries() {
    assert_eq!(
        domain::money::Money::zero(domain::money::Currency::Usd)
            .minor_units()
            .get(),
        0
    );
    assert!(domain::money::Money::try_new(0, domain::money::Currency::Usd).is_ok());
    assert!(domain::money::Money::try_new(-1, domain::money::Currency::Usd).is_err());
    assert!(
        domain::money::Money::try_new(i128::from(u64::MAX) + 1, domain::money::Currency::Usd)
            .is_err()
    );

    let rejected =
        serde_json::from_str::<domain::money::Money>(r#"{"minor_units":-1,"currency":"Usd"}"#);
    assert!(rejected.is_err());

    assert_eq!(
        money(900_000)
            .checked_sub(money(95_000))
            .unwrap()
            .minor_units()
            .get(),
        805_000
    );
    assert!(money(50_000).checked_sub(money(95_000)).is_err());
    assert!(
        domain::money::Money::usd(u64::MAX)
            .unwrap()
            .checked_add(money(1))
            .is_err()
    );
}

#[test]
fn canonical_money_rejects_currency_mismatched_arithmetic() {
    let usd = domain::money::Money::usd(10_000).unwrap();
    let accounting_currency = domain::money::Currency::AccountingSystem("legacy-cad".to_owned());
    let legacy_cad = domain::money::Money::try_new(10_000, accounting_currency).unwrap();

    assert!(usd.checked_add(legacy_cad.clone()).is_err());
    assert!(usd.checked_sub(legacy_cad).is_err());
}

#[test]
fn financial_net_revenue_reports_deductions_exceeding_gross_without_panic() {
    let period = analytics::finance::SitePeriod::builder()
        .location_id(location_id())
        .start(Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap())
        .end(Utc.with_ymd_and_hms(2026, 8, 31, 23, 59, 59).unwrap())
        .build()
        .unwrap();

    let fact = analytics::finance::ReportedRevenueObservationFact::builder()
        .period(period)
        .service(entities::ServiceKind::Grooming)
        .gross_revenue(money(50_000))
        .discount(money(75_000))
        .refund(money(20_000))
        .labor_cost(money(310_000))
        .source_system(source::System::FinanceAccounting)
        .build();

    assert!(matches!(
        fact.net_revenue(),
        Err(domain::money::Error::SubtractionWouldBeNegative)
    ));
}

#[test]
fn generalized_outcome_attribution_links_recommendations_to_measured_business_results() {
    let outcome = analytics::outcome::Record::builder()
        .id(analytics::outcome::Id::try_new("outcome-lead-42").unwrap())
        .workstream(analytics::outcome::Workstream::LeadResponse)
        .location_id(location_id())
        .change(analytics::outcome::MeasuredChange::booking_converted(0, 1))
        .attribution(
            analytics::outcome::AttributionEvidence::reported_reviewed_action(
                analytics::outcome::RecommendationRef::try_new("lead-response-review-42").unwrap(),
                analytics::outcome::EvidenceRef::try_new("crm-reservation-42").unwrap(),
            ),
        )
        .source(source::System::Crm)
        .recorded_at(Utc.with_ymd_and_hms(2026, 8, 12, 16, 0, 0).unwrap())
        .build();

    assert!(!outcome.can_support_value_claim());
    assert_eq!(
        outcome.workstream(),
        analytics::outcome::Workstream::LeadResponse
    );
}

#[test]
fn serialized_outcome_evidence_cannot_issue_value_claim_authority() {
    let weakly_correlated = analytics::outcome::Record::builder()
        .id(analytics::outcome::Id::try_new("outcome-revenue-weak").unwrap())
        .workstream(analytics::outcome::Workstream::FinancialInsights)
        .location_id(location_id())
        .change(analytics::outcome::MeasuredChange::revenue(
            money(900_000),
            money(950_000),
        ))
        .attribution(analytics::outcome::AttributionEvidence::correlated_only(
            analytics::outcome::EvidenceRef::try_new("finance-dashboard-correlation").unwrap(),
        ))
        .source(source::System::FinanceAccounting)
        .recorded_at(Utc.with_ymd_and_hms(2026, 8, 31, 23, 59, 59).unwrap())
        .build();

    assert!(!weakly_correlated.can_support_value_claim());

    let reviewed_strong = analytics::outcome::Record::builder()
        .id(analytics::outcome::Id::try_new("outcome-revenue-reviewed").unwrap())
        .workstream(analytics::outcome::Workstream::FinancialInsights)
        .location_id(location_id())
        .change(analytics::outcome::MeasuredChange::revenue(
            money(900_000),
            money(950_000),
        ))
        .attribution(
            analytics::outcome::AttributionEvidence::reported_reviewed_action(
                analytics::outcome::RecommendationRef::try_new("discount-review-42").unwrap(),
                analytics::outcome::EvidenceRef::try_new("reviewed-finance-export-42").unwrap(),
            ),
        )
        .source(source::System::FinanceAccounting)
        .recorded_at(Utc.with_ymd_and_hms(2026, 8, 31, 23, 59, 59).unwrap())
        .build();

    assert!(!reviewed_strong.can_support_value_claim());
    assert_eq!(
        reviewed_strong.metric(),
        analytics::outcome::Metric::ReportedRevenueObservation
    );
}

#[test]
fn strategic_positive_scalars_reject_zero_at_constructor_and_serde_boundaries() {
    assert!(lead::response::Minutes::try_new(0).is_err());
    assert!(operations::labor::Minutes::try_new(0).is_err());
    assert!(operations::labor::PeopleCount::try_new(0).is_err());
    assert!(operations::capacity::Quantity::try_new(0).is_err());

    assert!(serde_json::from_str::<lead::response::Minutes>("0").is_err());
    assert!(serde_json::from_str::<operations::labor::Minutes>("0").is_err());
    assert!(serde_json::from_str::<operations::labor::PeopleCount>("0").is_err());
    assert!(serde_json::from_str::<operations::capacity::Quantity>("0").is_err());

    assert!(serde_json::from_str::<lead::response::Minutes>("65536").is_err());
    assert!(serde_json::from_str::<operations::labor::Minutes>("65536").is_err());
    assert!(serde_json::from_str::<operations::labor::PeopleCount>("65536").is_err());
    assert!(serde_json::from_str::<operations::capacity::Quantity>("4294967296").is_err());
}

#[test]
fn strategic_positive_scalars_round_trip_valid_serde_values() {
    let lead_minutes = lead::response::Minutes::try_new(5).unwrap();
    let labor_minutes = operations::labor::Minutes::try_new(15).unwrap();
    let people_count = operations::labor::PeopleCount::try_new(2).unwrap();
    let quantity = operations::capacity::Quantity::try_new(9).unwrap();

    assert_eq!(serde_json::to_string(&lead_minutes).unwrap(), "5");
    assert_eq!(serde_json::to_string(&labor_minutes).unwrap(), "15");
    assert_eq!(serde_json::to_string(&people_count).unwrap(), "2");
    assert_eq!(serde_json::to_string(&quantity).unwrap(), "9");

    assert_eq!(
        serde_json::from_str::<lead::response::Minutes>("5").unwrap(),
        lead_minutes
    );
    assert_eq!(
        serde_json::from_str::<operations::labor::Minutes>("15").unwrap(),
        labor_minutes
    );
    assert_eq!(
        serde_json::from_str::<operations::labor::PeopleCount>("2").unwrap(),
        people_count
    );
    assert_eq!(
        serde_json::from_str::<operations::capacity::Quantity>("9").unwrap(),
        quantity
    );
}

#[test]
fn strategic_windows_reject_non_forward_ranges_at_constructor_and_serde_boundaries() {
    let start = Utc.with_ymd_and_hms(2026, 8, 12, 10, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 8, 12, 9, 59, 59).unwrap();

    assert!(operations::time_bucket::Window::new(start, start).is_err());
    assert!(operations::time_bucket::Window::new(start, end).is_err());

    let equal_window = serde_json::json!({
        "start": start,
        "end": start,
    });
    let reversed_window = serde_json::json!({
        "start": start,
        "end": end,
    });

    assert!(serde_json::from_value::<operations::time_bucket::Window>(equal_window).is_err());
    assert!(serde_json::from_value::<operations::time_bucket::Window>(reversed_window).is_err());
}

#[test]
fn strategic_windows_round_trip_valid_serde_values() {
    let start = Utc.with_ymd_and_hms(2026, 8, 12, 10, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 8, 12, 11, 0, 0).unwrap();
    let window = operations::time_bucket::Window::new(start, end).unwrap();

    let encoded = serde_json::to_value(window).unwrap();
    assert_eq!(
        serde_json::from_value::<operations::time_bucket::Window>(encoded).unwrap(),
        window
    );
}

#[test]
fn financial_site_period_rejects_non_forward_ranges_at_builder_and_serde_boundaries() {
    let start = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 7, 31, 23, 59, 59).unwrap();

    assert!(
        analytics::finance::SitePeriod::builder()
            .location_id(location_id())
            .start(start)
            .end(start)
            .build()
            .is_err()
    );
    assert!(
        analytics::finance::SitePeriod::builder()
            .location_id(location_id())
            .start(start)
            .end(end)
            .build()
            .is_err()
    );

    let equal_period = serde_json::json!({
        "location_id": location_id(),
        "start": start,
        "end": start,
    });
    let reversed_period = serde_json::json!({
        "location_id": location_id(),
        "start": start,
        "end": end,
    });

    assert!(serde_json::from_value::<analytics::finance::SitePeriod>(equal_period).is_err());
    assert!(serde_json::from_value::<analytics::finance::SitePeriod>(reversed_period).is_err());
}

#[test]
fn financial_site_period_round_trips_valid_serde_values() {
    let period = analytics::finance::SitePeriod::builder()
        .location_id(location_id())
        .start(Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap())
        .end(Utc.with_ymd_and_hms(2026, 8, 31, 23, 59, 59).unwrap())
        .build()
        .unwrap();

    let encoded = serde_json::to_value(period).unwrap();
    assert_eq!(
        serde_json::from_value::<analytics::finance::SitePeriod>(encoded).unwrap(),
        period
    );
}
