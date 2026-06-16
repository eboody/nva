use app::daily_update;
use domain::{entities, policy, workflow};

fn workflow_event(event_type: workflow::EventType) -> workflow::Event {
    workflow::Event {
        event_id: workflow::EventId(uuid::Uuid::nil()),
        event_type,
        occurred_at: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
        actor: entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("lead-care-1").unwrap(),
        },
        location_id: entities::LocationId(uuid::Uuid::nil()),
        subject: workflow::Subject::Reservation(entities::ReservationId(uuid::Uuid::nil())),
        policy_context: workflow::PolicyContext {
            allowed_actions: vec![
                workflow::AllowedAction::SummarizeCareNotes,
                workflow::AllowedAction::DraftCustomerMessage,
            ],
            automation_level: policy::automation::Level::DraftOnly,
            required_reviews: vec![policy::ReviewGate::CustomerMessageApproval],
        },
    }
}

fn note(
    id: uuid::Uuid,
    kind: entities::CareNoteKind,
    visibility: entities::CareNoteVisibility,
    body: &str,
) -> entities::CareNote {
    entities::CareNote::builder()
        .id(entities::CareNoteId(id))
        .subject(entities::CareNoteSubject::Reservation(
            entities::ReservationId(uuid::Uuid::nil()),
        ))
        .kind(kind)
        .visibility(visibility)
        .body(entities::CareNoteBody::try_new(body).unwrap())
        .author(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("kennel-1").unwrap(),
        })
        .recorded_at(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
        .audit_refs(vec![domain::audit::EventId(id)])
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
            entities::CareNoteKind::General,
            entities::CareNoteVisibility::CustomerVisibleAfterReview,
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
            .any(|fact| fact.source_note_id == entities::CareNoteId(uuid::Uuid::from_u128(1)))
    );
    assert_eq!(
        preview.approval.lifecycle,
        entities::ApprovalLifecycle::ApprovalRequested
    );
    assert!(preview.send_stub.is_blocked_until_human_approval());
    assert!(
        preview
            .audit_log
            .iter()
            .any(|event| event.action == entities::AuditAction::WorkflowEventRecorded)
    );
    assert!(
        preview
            .audit_log
            .iter()
            .any(|event| event.action == entities::AuditAction::MessageApprovalRequested)
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
                entities::CareNoteKind::Behavior,
                entities::CareNoteVisibility::CustomerVisibleAfterReview,
                "noise sensitive and needs behavior review before owner wording",
            ),
            note(
                uuid::Uuid::from_u128(3),
                entities::CareNoteKind::General,
                entities::CareNoteVisibility::InternalOnly,
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
    assert_eq!(preview.approval.gate, policy::ReviewGate::ManagerApproval);
    assert!(preview.send_stub.is_blocked_until_human_approval());
}
