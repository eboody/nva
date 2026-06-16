use app::{agents, booking_triage, tools};
use domain::{agent, entities, money, pet, policy, workflow};

#[test]
fn booking_triage_uses_typestate_for_legal_readiness_progression() {
    let intake = booking_triage::Request::<booking_triage::Intake>::builder()
        .reservation(booking_triage::Reservation::try_new("REQ-123").unwrap())
        .build();

    let with_pet_profile = intake.attach_pet_profile(
        pet::Name::try_new("Moose").unwrap(),
        booking_triage::PetProfileCompleteness::Complete,
    );
    let with_policy = with_pet_profile.attach_policy_snapshot(
        booking_triage::PolicySnapshot::try_new("default-conservative-play-policy").unwrap(),
    );
    let ready = with_policy.mark_ready_for_policy_decision();

    assert_eq!(ready.reservation().clone().into_inner(), "REQ-123");
}

#[test]
fn agent_prompt_packets_use_domain_event_contracts_from_application_boundary() {
    let packet = agents::AgentPromptPacket::builder()
        .workflow_name(agent::Name::try_new("booking-triage").unwrap())
        .goal(agent::Purpose::try_new("Prepare deterministic booking triage context.").unwrap())
        .event(workflow::Event {
            event_id: workflow::EventId(uuid::Uuid::nil()),
            event_type: workflow::EventType::BookingTriageNeeded,
            occurred_at: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
            actor: entities::ActorRef::System,
            location_id: entities::LocationId(uuid::Uuid::nil()),
            subject: workflow::Subject::Reservation(entities::reservation::Id(uuid::Uuid::nil())),
            policy_context: workflow::PolicyContext {
                allowed_actions: vec![workflow::AllowedAction::ExtractStructuredData],
                automation_level: policy::automation::Level::DraftOnly,
                required_reviews: vec![policy::ReviewGate::ManagerApproval],
            },
        })
        .input(())
        .policies(vec![
            agent::PolicyInstruction::try_new("manager approval before customer-facing output")
                .unwrap(),
        ])
        .output_schema_name(agent::OutputSchemaName::try_new("BookingTriageOutput").unwrap())
        .build();

    assert_eq!(
        packet.output_schema_name.into_inner(),
        "BookingTriageOutput"
    );
    assert_eq!(packet.policies.len(), 1);
}

#[test]
fn agent_prompt_packet_semantics_are_reexported_from_application_agent_boundary() {
    let policy_instruction =
        agents::PolicyInstruction::try_new("manager approval before customer-facing output")
            .expect("policy instruction is an agent-packet semantic value");
    let schema_name = agents::OutputSchemaName::try_new("BookingTriageOutput")
        .expect("schema name is an agent-packet semantic value");

    assert_eq!(
        policy_instruction.into_inner(),
        "manager approval before customer-facing output"
    );
    assert_eq!(schema_name.into_inner(), "BookingTriageOutput");
}

#[test]
fn tool_and_policy_results_use_semantic_decisions_not_bool_string_pairs() {
    let availability = tools::availability::Outcome {
        decision: tools::availability::Decision::Available {
            reason: tools::availability::SuccessReason::CapacityHeld,
            capacity_snapshot_id: tools::availability::CapacitySnapshotId::try_new("  cap-123  ")
                .unwrap(),
        },
    };
    assert!(availability.is_available());
    let tools::availability::Decision::Available {
        capacity_snapshot_id,
        ..
    } = availability.decision
    else {
        panic!("expected held capacity snapshot for available decision");
    };
    assert_eq!(capacity_snapshot_id.into_inner(), "cap-123");

    let availability_request = tools::availability::Request {
        location_id: entities::LocationId(uuid::Uuid::nil()),
        reservation_id: Some(entities::reservation::Id(uuid::Uuid::nil())),
        service_notes: tools::availability::ServiceNotes::try_new(
            "  boarding suite with medication watch  ",
        )
        .unwrap(),
    };
    assert_eq!(
        availability_request.service_notes.into_inner(),
        "boarding suite with medication watch"
    );
    assert!(tools::availability::ServiceNotes::try_new("   ").is_err());

    let draft = tools::draft_update::Request {
        reservation_id: entities::reservation::Id(uuid::Uuid::nil()),
        proposed_status: entities::reservation::Status::Waitlisted,
        rationale: tools::draft_update::Rationale::CapacityUnavailable,
    };
    assert_eq!(
        draft.proposed_status,
        entities::reservation::Status::Waitlisted
    );

    let tool_denial = tools::Error::policy_denied(policy::denial::Reason::ManagerApprovalRequired);
    assert_eq!(
        tool_denial.to_string(),
        "policy denied: manager approval required"
    );
}

#[test]
fn external_integration_contracts_are_application_tool_port_types() {
    let lookup = tools::portal::lookup::Request {
        provider: tools::portal::Provider::Gingr,
        account: tools::portal::AccountId::try_new("  gingr-east-1  ").unwrap(),
        criteria: tools::portal::lookup::Criteria::Reservation(entities::reservation::Id(
            uuid::Uuid::nil(),
        )),
        include: vec![tools::portal::Include::PetProfile],
    };
    assert_eq!(lookup.account.into_inner(), "gingr-east-1");

    let authorization = tools::payment::authorization::Request {
        subject: tools::payment::Subject::ReservationDeposit(entities::reservation::Id(
            uuid::Uuid::nil(),
        )),
        amount: money::Money::new(
            money::MinorUnits::try_new(5_000).unwrap(),
            money::Currency::Usd,
        ),
        capture_policy: tools::payment::CapturePolicy::AuthorizeOnly,
        idempotency_key: tools::payment::IdempotencyKey::try_new("  reservation-deposit-123  ")
            .unwrap(),
    };
    assert_eq!(
        authorization.idempotency_key.into_inner(),
        "reservation-deposit-123"
    );

    let message = tools::messaging::draft::Request {
        channel: tools::messaging::DeliveryChannel::Email,
        recipient: tools::messaging::Recipient::Customer(entities::CustomerId(uuid::Uuid::nil())),
        body: tools::messaging::message_body::Body::try_new(
            "  Please upload updated rabies records.  ",
        )
        .unwrap(),
        review: tools::messaging::ReviewPolicy::ManagerApprovalRequired,
    };
    assert_eq!(
        message.body.into_inner(),
        "Please upload updated rabies records."
    );

    let intake = tools::documents::document::IntakeRequest {
        document: tools::documents::document::reference::Ref::try_new("  file/vaccine.pdf  ")
            .unwrap(),
        source: tools::documents::document::Source::CustomerUpload,
        expected_content: tools::documents::document::ExpectedContent::VaccineProof,
    };
    assert_eq!(intake.document.into_inner(), "file/vaccine.pdf");

    let snapshot = tools::media::SnapshotRequest {
        location_id: entities::LocationId(uuid::Uuid::nil()),
        camera_id: tools::media::CameraId::try_new("  lobby-cam-1  ").unwrap(),
        purpose: tools::media::CapturePurpose::PetStatusCheck(entities::PetId(uuid::Uuid::nil())),
    };
    assert_eq!(snapshot.camera_id.into_inner(), "lobby-cam-1");

    let task = tools::hermes::task::DraftRequest {
        title: workflow::task::Title::try_new("  Review vaccine proof  ").unwrap(),
        body: workflow::task::Body::try_new("Confirm rabies expiration date.").unwrap(),
        queue: tools::hermes::QueueName::try_new("  manager-review  ").unwrap(),
        trigger: tools::hermes::Trigger::WorkflowReview,
    };
    assert_eq!(task.queue.into_inner(), "manager-review");

    assert!(tools::portal::AccountId::try_new("   ").is_err());
    assert!(tools::payment::IdempotencyKey::try_new("   ").is_err());
    assert!(tools::messaging::message_body::Body::try_new("   ").is_err());
    assert!(tools::documents::document::reference::Ref::try_new("   ").is_err());
    assert!(tools::media::CameraId::try_new("   ").is_err());
    assert!(tools::hermes::QueueName::try_new("   ").is_err());
}

#[test]
fn baseline_agent_specs_include_context_driven_operations_agents() {
    let names: Vec<_> = agents::baseline_agent_specs()
        .into_iter()
        .map(|spec| spec.name.into_inner())
        .collect();

    for expected in [
        "manager-daily-brief",
        "lead-conversion",
        "grooming-rebooking",
        "reputation-triage",
        "sop-policy-assistant",
    ] {
        assert!(
            names.iter().any(|name| name == expected),
            "missing baseline operations agent spec: {expected}"
        );
    }
}

#[test]
fn application_prelude_consolidates_agent_and_tool_boundaries() {
    use app::prelude as api;

    let spec: api::AgentPromptPacket<()> = api::AgentPromptPacket::builder()
        .workflow_name(agent::Name::try_new("booking-triage").unwrap())
        .goal(agent::Purpose::try_new("Evaluate deterministic booking policy.").unwrap())
        .event(workflow::Event {
            event_id: workflow::EventId(uuid::Uuid::nil()),
            event_type: workflow::EventType::BookingTriageNeeded,
            occurred_at: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
            actor: entities::ActorRef::System,
            location_id: entities::LocationId(uuid::Uuid::nil()),
            subject: workflow::Subject::Reservation(entities::reservation::Id(uuid::Uuid::nil())),
            policy_context: workflow::PolicyContext {
                allowed_actions: vec![workflow::AllowedAction::ExtractStructuredData],
                automation_level: policy::automation::Level::DraftOnly,
                required_reviews: vec![policy::ReviewGate::ManagerApproval],
            },
        })
        .input(())
        .policies(vec![
            agent::PolicyInstruction::try_new("manager approval").unwrap(),
        ])
        .output_schema_name(agent::OutputSchemaName::try_new("BookingTriageOutput").unwrap())
        .build();

    let availability = api::availability::Outcome {
        decision: api::availability::Decision::Unavailable {
            reason: api::availability::DenialReason::RequiresHumanReview,
        },
    };

    assert_eq!(spec.workflow_name.into_inner(), "booking-triage");
    assert!(!availability.is_available());
}
