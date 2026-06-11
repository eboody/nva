use domain::{audit, document, entities, incident, message, policy, vaccine};

#[test]
fn documents_and_vaccine_records_preserve_evidence_review_and_audit_lineage() {
    let audit_id = audit::EventId(uuid::Uuid::nil());
    let document = entities::Document::builder()
        .id(entities::DocumentId(uuid::Uuid::nil()))
        .location_id(entities::LocationId(uuid::Uuid::nil()))
        .subject(entities::DocumentSubject::Pet(entities::PetId(
            uuid::Uuid::nil(),
        )))
        .classification(document::Classification::VaccineProof)
        .source(document::Source::CustomerUpload)
        .uploaded_by_actor(entities::ActorRef::Customer(entities::CustomerId(
            uuid::Uuid::nil(),
        )))
        .uploaded_at(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
        .original_file(
            document::OriginalFile::builder()
                .filename(document::FileName::try_new("  rabies.pdf  ").unwrap())
                .mime_type(document::MimeType::try_new("application/pdf").unwrap())
                .content_length(document::ContentLengthBytes::try_new(42).unwrap())
                .sha256(
                    document::Sha256Digest::try_new(
                        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                    )
                    .unwrap(),
                )
                .build(),
        )
        .storage_ref(
            document::StorageRef::builder()
                .bucket(document::StorageBucket::try_new("vaccine-documents").unwrap())
                .key(document::StorageKey::try_new("pets/moose/rabies.pdf").unwrap())
                .version(document::StorageVersion::try_new("v1").unwrap())
                .build(),
        )
        .virus_scan_status(document::VirusScanStatus::Passed)
        .pii_redaction_status(document::PiiRedactionStatus::Pending)
        .verification_status(document::Status::AwaitingReview)
        .audit_refs(vec![audit_id])
        .build();

    assert_eq!(
        document.original_file.filename.clone().into_inner(),
        "rabies.pdf"
    );
    assert!(document.requires_human_review_before_use());
    assert_eq!(document.audit_refs, vec![audit_id]);

    let vaccine = entities::VaccineRecord::builder()
        .id(entities::VaccineRecordId(uuid::Uuid::nil()))
        .pet_id(entities::PetId(uuid::Uuid::nil()))
        .vaccine_name(policy::VaccineName::try_new("  Rabies  ").unwrap())
        .source_document_id(document.id)
        .status(vaccine::Status::PendingReview)
        .effective_on(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
        .expires_on(chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap())
        .review_gate(policy::ReviewGate::MedicalDocumentReview)
        .audit_refs(vec![audit_id])
        .build();

    assert!(vaccine.requires_human_review_before_compliance());
    assert_eq!(vaccine.vaccine_name.into_inner(), "Rabies");
}

#[test]
fn notes_incidents_messages_and_approvals_have_invariant_lifecycle_enums() {
    let note = entities::CareNote::builder()
        .id(entities::CareNoteId(uuid::Uuid::nil()))
        .subject(entities::CareNoteSubject::Reservation(
            entities::ReservationId(uuid::Uuid::nil()),
        ))
        .kind(entities::CareNoteKind::Medication)
        .visibility(entities::CareNoteVisibility::InternalOnly)
        .body(entities::CareNoteBody::try_new("  gave medication with dinner  ").unwrap())
        .author(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("kennel-1").unwrap(),
        })
        .recorded_at(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
        .audit_refs(vec![audit::EventId(uuid::Uuid::nil())])
        .build();
    assert!(!note.is_customer_visible_without_review());
    assert_eq!(note.body.into_inner(), "gave medication with dinner");

    let incident = entities::Incident::builder()
        .id(entities::IncidentId(uuid::Uuid::nil()))
        .location_id(entities::LocationId(uuid::Uuid::nil()))
        .primary_subject(entities::IncidentSubject::Pet(entities::PetId(
            uuid::Uuid::nil(),
        )))
        .category(incident::Category::Medication)
        .severity(incident::Severity::High)
        .status(incident::Status::NeedsManagerReview)
        .reported_by(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("lead-1").unwrap(),
        })
        .reported_at(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
        .summary(
            incident::Summary::try_new("  missed noon dose; manager review required  ").unwrap(),
        )
        .required_review_gates(vec![policy::ReviewGate::ManagerApproval])
        .audit_refs(vec![audit::EventId(uuid::Uuid::nil())])
        .build();
    assert!(incident.requires_manager_attention());

    let message = entities::Message::builder()
        .id(entities::MessageId(uuid::Uuid::nil()))
        .subject(entities::MessageSubject::Incident(incident.id))
        .direction(message::Direction::OutboundDraft)
        .channel(message::Channel::Email)
        .status(message::Status::ApprovalRequested)
        .body_ref(message::BodyRef::try_new("message-body/evidence-1").unwrap())
        .approval_gate(policy::ReviewGate::CustomerMessageApproval)
        .audit_refs(vec![audit::EventId(uuid::Uuid::nil())])
        .build();
    assert!(message.requires_approval_before_send());

    let approval = entities::ApprovalRecord::builder()
        .id(entities::ApprovalId(uuid::Uuid::nil()))
        .target(entities::ApprovalTarget::Message(message.id))
        .gate(policy::ReviewGate::CustomerMessageApproval)
        .lifecycle(entities::ApprovalLifecycle::ApprovalRequested)
        .requested_by(entities::ActorRef::Agent {
            workflow: domain::agent::Name::try_new("incident-escalation").unwrap(),
        })
        .requested_at(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
        .audit_refs(vec![audit::EventId(uuid::Uuid::nil())])
        .build();
    assert!(!approval.is_applicable());
}

#[test]
fn terminal_approval_decisions_carry_actor_and_decision_time() {
    let decided_by = entities::ActorRef::Manager {
        manager_id: entities::ManagerId::try_new("manager-1").unwrap(),
    };
    let decided_at = chrono::DateTime::<chrono::Utc>::UNIX_EPOCH;

    let approved = entities::ApprovalRecord::builder()
        .id(entities::ApprovalId(uuid::Uuid::nil()))
        .target(entities::ApprovalTarget::Reservation(
            entities::ReservationId(uuid::Uuid::nil()),
        ))
        .gate(policy::ReviewGate::RefundOrDepositException)
        .lifecycle(entities::ApprovalLifecycle::Approved {
            decided_by: decided_by.clone(),
            decided_at,
        })
        .requested_by(entities::ActorRef::Agent {
            workflow: domain::agent::Name::try_new("deposit-exception").unwrap(),
        })
        .requested_at(decided_at)
        .build();

    assert_eq!(approved.status(), entities::ApprovalStatus::Approved);
    assert!(approved.is_terminal_decision());
    assert!(approved.is_applicable());
    assert_eq!(
        approved.decision_actor_and_time(),
        Some((&decided_by, decided_at))
    );
}

#[test]
fn audit_subjects_and_actions_represent_required_write_paths() {
    let actions = [
        entities::AuditAction::DocumentReceived,
        entities::AuditAction::VaccineRecordReviewRequested,
        entities::AuditAction::IncidentStatusChanged,
        entities::AuditAction::MessageApprovalRequested,
        entities::AuditAction::ApprovalDecisionRecorded,
        entities::AuditAction::WorkflowEventRecorded,
    ];

    assert!(actions.contains(&entities::AuditAction::WorkflowEventRecorded));
    assert_eq!(
        entities::AuditSubject::Message(entities::MessageId(uuid::Uuid::nil())),
        entities::AuditSubject::Message(entities::MessageId(uuid::Uuid::nil()))
    );
    assert_eq!(
        entities::AuditSubject::Approval(entities::ApprovalId(uuid::Uuid::nil())),
        entities::AuditSubject::Approval(entities::ApprovalId(uuid::Uuid::nil()))
    );
}

#[test]
fn semantic_scalars_reject_empty_or_invalid_storage_evidence_values() {
    assert!(document::FileName::try_new("   ").is_err());
    assert!(document::ContentLengthBytes::try_new(0).is_err());
    assert!(document::Sha256Digest::try_new("not-a-sha").is_err());
    assert!(message::BodyRef::try_new("   ").is_err());
    assert!(incident::Summary::try_new("   ").is_err());
}
