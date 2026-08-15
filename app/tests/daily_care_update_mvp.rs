use app::daily_update;
use domain::{audit, entities, message, policy, workflow};

fn workflow_event(event_type: workflow::EventType) -> workflow::Event {
    workflow_event_for_subject(
        event_type,
        workflow::Subject::Reservation(entities::reservation::Id::new(uuid::Uuid::from_u128(3))),
    )
}

fn workflow_event_for_subject(
    event_type: workflow::EventType,
    subject: workflow::Subject,
) -> workflow::Event {
    workflow::Event::try_new(
        workflow::EventId::new(uuid::Uuid::from_u128(1)),
        event_type,
        chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
        entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("lead-care-1").unwrap(),
        },
        entities::LocationId::new(uuid::Uuid::from_u128(2)),
        subject,
        workflow::PolicyContext {
            allowed_actions: vec![
                workflow::AllowedAction::SummarizeCareNotes,
                workflow::AllowedAction::DraftCustomerMessage,
            ],
            automation_level: policy::automation::Level::DraftOnly,
            required_reviews: vec![policy::ReviewGate::CustomerMessageApproval],
        },
    )
    .expect("daily update workflow event fixture should match reservation subject")
}

fn note(
    id: uuid::Uuid,
    kind: entities::care_note::Kind,
    visibility: entities::care_note::Visibility,
    body: &str,
) -> entities::CareNote {
    entities::CareNote::builder()
        .id(entities::care_note::Id::new(id))
        .subject(entities::care_note::Subject::Reservation(
            entities::reservation::Id::new(uuid::Uuid::from_u128(1)),
        ))
        .kind(kind)
        .visibility(visibility)
        .body(entities::care_note::Body::try_new(body).unwrap())
        .author(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("kennel-1").unwrap(),
        })
        .recorded_at(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
        .audit_refs(vec![domain::audit::EventId::new(id)])
        .build()
}

#[test]
fn routine_staff_notes_become_review_gated_owner_preview_with_audit_lineage() {
    let request = daily_update::MvpPreviewRequest::builder()
        .event(workflow_event(workflow::EventType::DailyNoteCreated))
        .pet_name(domain::pet::Name::try_new("  Juniper  ").unwrap())
        .owner_display_name(domain::customer::Name::try_new("  R. Patel  ").unwrap())
        .policy_snapshot_id(policy::Id::try_new("daily-care-update-mvp-v1").unwrap())
        .notes(vec![note(
            uuid::Uuid::from_u128(1),
            entities::care_note::Kind::General,
            entities::care_note::Visibility::CustomerVisibleAfterReview,
            "  enjoyed supervised play and is resting after lunch.  ",
        )])
        .build();

    let preview = daily_update::build_mvp_preview(request).expect("routine preview builds");

    assert_eq!(
        preview.agent_packet.workflow_name.into_inner(),
        "daily-care-update"
    );
    assert_eq!(
        preview.owner_message_draft.body_ref.clone().into_inner(),
        "Hi R. Patel — Juniper enjoyed supervised play and is resting after lunch."
    );
    assert!(!preview.output.disposition.allows_live_send());
    assert!(preview.output.disposition.requires_human_review());
    assert_eq!(
        preview
            .output
            .disposition
            .review_reason()
            .clone()
            .into_inner(),
        "customer_message_approval_not_configured"
    );
    assert!(
        preview
            .output
            .included_facts
            .iter()
            .any(|fact| fact.source_note_id
                == entities::care_note::Id::new(uuid::Uuid::from_u128(1)))
    );
    assert_eq!(
        preview.approval.lifecycle().clone(),
        entities::approval::Lifecycle::ApprovalRequested
    );
    assert!(preview.send_stub.is_blocked_until_human_approval());
    assert!(
        preview
            .audit_log
            .iter()
            .any(|event| event.action == audit::Action::WorkflowEventRecorded)
    );
    assert!(
        preview
            .audit_log
            .iter()
            .any(|event| event.action == audit::Action::MessageApprovalRequested)
    );

    let output_json = serde_json::to_value(&preview.output).expect("output serializes");
    assert_eq!(output_json["should_send"], serde_json::json!(false));
    assert_eq!(output_json["requires_review"], serde_json::json!(true));
    assert_eq!(
        output_json["review_reason"],
        serde_json::json!("customer_message_approval_not_configured")
    );
    assert!(
        output_json.get("disposition").is_none(),
        "DailyCareUpdateOutput.v1 keeps the flat review-gate wire contract"
    );

    let roundtrip: daily_update::daily_care_update::Output =
        serde_json::from_value(output_json).expect("v1 output shape deserializes");
    assert_eq!(roundtrip.disposition, preview.output.disposition);
}

#[test]
fn concern_notes_are_suppressed_from_customer_copy_and_route_to_manager_review() {
    let request = daily_update::MvpPreviewRequest::builder()
        .event(workflow_event(workflow::EventType::DailyUpdateNeeded))
        .pet_name(domain::pet::Name::try_new("Miso").unwrap())
        .owner_display_name(domain::customer::Name::try_new("Avery Chen").unwrap())
        .policy_snapshot_id(policy::Id::try_new("daily-care-update-mvp-v1").unwrap())
        .notes(vec![
            note(
                uuid::Uuid::from_u128(2),
                entities::care_note::Kind::Behavior,
                entities::care_note::Visibility::CustomerVisibleAfterReview,
                "noise sensitive and needs behavior review before owner wording",
            ),
            note(
                uuid::Uuid::from_u128(3),
                entities::care_note::Kind::General,
                entities::care_note::Visibility::InternalOnly,
                "staff debate: do not publish raw note",
            ),
        ])
        .build();

    let preview = daily_update::build_mvp_preview(request).expect("concern preview builds safely");

    assert!(
        preview
            .owner_message_draft
            .body_ref
            .clone()
            .into_inner()
            .contains("update is being reviewed")
    );
    assert!(
        !preview
            .owner_message_draft
            .body_ref
            .clone()
            .into_inner()
            .contains("noise sensitive")
    );
    assert!(
        !preview
            .owner_message_draft
            .body_ref
            .clone()
            .into_inner()
            .contains("staff debate")
    );
    assert!(!preview.output.disposition.allows_live_send());
    assert!(preview.output.disposition.requires_human_review());
    assert_eq!(
        preview
            .output
            .disposition
            .review_reason()
            .clone()
            .into_inner(),
        "behavior_review_required"
    );
    assert!(
        preview
            .output
            .internal_flags
            .iter()
            .any(|flag| flag.code == daily_update::InternalFlagCode::BehaviorReviewRequired)
    );
    assert_eq!(
        *preview.approval.gate(),
        policy::ReviewGate::ManagerApproval
    );
    assert!(preview.send_stub.is_blocked_until_human_approval());
}

#[test]
fn sensitive_payment_incident_and_ambiguous_facts_create_suppression_records_not_customer_copy() {
    let request = daily_update::MvpPreviewRequest::builder()
        .event(workflow_event(workflow::EventType::DailyUpdateNeeded))
        .pet_name(domain::pet::Name::try_new("Miso").unwrap())
        .owner_display_name(domain::customer::Name::try_new("Avery Chen").unwrap())
        .policy_snapshot_id(policy::Id::try_new("daily-care-update-mvp-v1").unwrap())
        .notes(vec![
            note(
                uuid::Uuid::from_u128(4),
                entities::care_note::Kind::Medical,
                entities::care_note::Visibility::CustomerVisibleAfterReview,
                "medication dose changed and needs care team review",
            ),
            note(
                uuid::Uuid::from_u128(5),
                entities::care_note::Kind::General,
                entities::care_note::Visibility::CustomerVisibleAfterReview,
                "payment refund question should not be included in Pawgress copy",
            ),
            note(
                uuid::Uuid::from_u128(6),
                entities::care_note::Kind::General,
                entities::care_note::Visibility::CustomerVisibleAfterReview,
                "incident follow-up is pending manager review",
            ),
            note(
                uuid::Uuid::from_u128(7),
                entities::care_note::Kind::General,
                entities::care_note::Visibility::CustomerVisibleAfterReview,
                "source ambiguous: wrong-pet note may be mixed in",
            ),
        ])
        .media_document_refs(vec![daily_update::MediaDocumentRef {
            document_id: entities::DocumentId::new(uuid::Uuid::from_u128(44)),
            source_note_id: entities::care_note::Id::new(uuid::Uuid::from_u128(7)),
            review_state: message::ReviewState::ApprovalRequested,
        }])
        .build();

    let preview = daily_update::build_mvp_preview(request).expect("suppressed preview builds");
    let body = preview.owner_message_draft.body_ref.clone().into_inner();

    assert!(body.contains("update is being reviewed"));
    for unsafe_phrase in [
        "medication dose",
        "payment refund",
        "incident follow-up",
        "wrong-pet",
    ] {
        assert!(
            !body.contains(unsafe_phrase),
            "customer-facing draft leaked sensitive phrase: {unsafe_phrase}"
        );
    }
    assert!(preview.owner_message_draft.media_document_refs.is_empty());
    assert_eq!(
        preview.output.suppressed_media_document_refs[0].reason,
        message::SuppressionReason::MediaReviewRequired
    );

    let omitted_reasons: Vec<_> = preview
        .output
        .omitted_facts
        .iter()
        .map(|fact| fact.reason)
        .collect();
    assert!(omitted_reasons.contains(&daily_update::OmissionReason::MedicalOrMedicationReview));
    assert!(omitted_reasons.contains(&daily_update::OmissionReason::PaymentOrBillingReview));
    assert!(omitted_reasons.contains(&daily_update::OmissionReason::IncidentOrSafetyReview));
    assert!(omitted_reasons.contains(&daily_update::OmissionReason::SourceAmbiguousReview));
    assert!(
        preview
            .output
            .suppression_records
            .iter()
            .any(|record| record.reason == message::SuppressionReason::SourceAmbiguity)
    );
    assert_eq!(
        *preview.approval.gate(),
        policy::ReviewGate::ManagerApproval
    );
    assert!(preview.send_stub.is_blocked_until_human_approval());
}

#[test]
fn pet_subject_cannot_be_rebound_to_a_nil_reservation() {
    let request = daily_update::MvpPreviewRequest::builder()
        .event(workflow_event_for_subject(
            workflow::EventType::DailyUpdateNeeded,
            workflow::Subject::Pet(entities::PetId::new(uuid::Uuid::from_u128(99))),
        ))
        .pet_name(domain::pet::Name::try_new("Miso").unwrap())
        .owner_display_name(domain::customer::Name::try_new("Avery Chen").unwrap())
        .policy_snapshot_id(policy::Id::try_new("daily-care-update-mvp-v1").unwrap())
        .notes(vec![note(
            uuid::Uuid::from_u128(8),
            entities::care_note::Kind::General,
            entities::care_note::Visibility::CustomerVisibleAfterReview,
            "resting after supervised play",
        )])
        .build();

    assert_eq!(
        daily_update::build_mvp_preview(request),
        Err(daily_update::Error::ReservationSubjectRequired)
    );
}
