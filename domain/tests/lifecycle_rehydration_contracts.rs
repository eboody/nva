use chrono::{TimeZone, Utc};
use domain::{document, entities, incident, money, payment, policy, vaccine, workflow};
use uuid::Uuid;

fn reservation_id(value: u128) -> entities::reservation::Id {
    entities::reservation::Id::new(Uuid::from_u128(value))
}

fn location_id() -> entities::LocationId {
    entities::LocationId::new(Uuid::from_u128(10))
}

fn customer_id() -> entities::CustomerId {
    entities::CustomerId::new(Uuid::from_u128(11))
}

fn pet_id() -> entities::PetId {
    entities::PetId::new(Uuid::from_u128(12))
}

fn actor() -> entities::ActorRef {
    entities::ActorRef::System
}

fn starts_at() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 12, 9, 0, 0).unwrap()
}

fn ends_at() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 12, 17, 0, 0).unwrap()
}

fn paid_deposit() -> payment::Deposit {
    payment::Deposit::paid(
        money::Money::usd(5_000).unwrap(),
        payment::Reference::try_new("receipt-42").unwrap(),
    )
}

#[test]
fn reservation_builder_uses_same_invariants_as_persisted_rehydration() {
    assert!(
        entities::Reservation::builder()
            .id(reservation_id(99))
            .location_id(location_id())
            .customer_id(customer_id())
            .pet_id(pet_id())
            .service(entities::ServiceKind::Boarding)
            .status(entities::reservation::Status::Confirmed)
            .starts_at(starts_at())
            .ends_at(ends_at())
            .deposit(payment::Deposit::required(
                money::Money::usd(5_000).unwrap(),
            ))
            .source(entities::reservation::Source::Portal(
                entities::PortalProvider::Gingr,
            ))
            .build()
            .is_ok()
    );

    let invalid_period = entities::Reservation::builder()
        .id(reservation_id(100))
        .location_id(location_id())
        .customer_id(customer_id())
        .pet_id(pet_id())
        .service(entities::ServiceKind::Boarding)
        .status(entities::reservation::Status::Confirmed)
        .starts_at(ends_at())
        .ends_at(starts_at())
        .deposit(payment::Deposit::required(
            money::Money::usd(5_000).unwrap(),
        ))
        .source(entities::reservation::Source::Portal(
            entities::PortalProvider::Gingr,
        ))
        .build()
        .expect_err("ordinary construction must reject impossible reservation intervals");
    assert_eq!(
        invalid_period,
        entities::reservation::Error::StayIntervalMustEndAfterStart
    );

    let empty_party = entities::Reservation::builder()
        .id(reservation_id(104))
        .location_id(location_id())
        .customer_id(customer_id())
        .service(entities::ServiceKind::Boarding)
        .status(entities::reservation::Status::Confirmed)
        .starts_at(starts_at())
        .ends_at(ends_at())
        .deposit(payment::Deposit::required(
            money::Money::usd(5_000).unwrap(),
        ))
        .source(entities::reservation::Source::Portal(
            entities::PortalProvider::Gingr,
        ))
        .build()
        .expect_err("ordinary construction must reject reservations without a pet party");
    assert_eq!(empty_party, entities::reservation::Error::PetPartyRequired);

    let duplicate_party = entities::Reservation::builder()
        .id(reservation_id(105))
        .location_id(location_id())
        .customer_id(customer_id())
        .pet_id(pet_id())
        .pet_id(pet_id())
        .service(entities::ServiceKind::Boarding)
        .status(entities::reservation::Status::Confirmed)
        .starts_at(starts_at())
        .ends_at(ends_at())
        .deposit(payment::Deposit::required(
            money::Money::usd(5_000).unwrap(),
        ))
        .source(entities::reservation::Source::Portal(
            entities::PortalProvider::Gingr,
        ))
        .build()
        .expect_err("ordinary construction must reject duplicate pet participation");
    assert_eq!(
        duplicate_party,
        entities::reservation::Error::DuplicatePet { pet_id: pet_id() }
    );

    let paid_deposit_stop = entities::Reservation::builder()
        .id(reservation_id(106))
        .location_id(location_id())
        .customer_id(customer_id())
        .pet_id(pet_id())
        .service(entities::ServiceKind::Boarding)
        .status(entities::reservation::Status::Confirmed)
        .starts_at(starts_at())
        .ends_at(ends_at())
        .deposit(paid_deposit())
        .source(entities::reservation::Source::Portal(
            entities::PortalProvider::Gingr,
        ))
        .hard_stop(entities::HardStop::DepositRequired)
        .build()
        .expect_err(
            "ordinary construction must reject deposit-required stops after deposit payment",
        );
    assert_eq!(
        paid_deposit_stop,
        entities::reservation::Error::DepositRequiredHardStopNeedsCollectibleDeposit
    );

    let terminal_stop = entities::Reservation::builder()
        .id(reservation_id(107))
        .location_id(location_id())
        .customer_id(customer_id())
        .pet_id(pet_id())
        .service(entities::ServiceKind::Boarding)
        .status(entities::reservation::Status::CheckedOut)
        .starts_at(starts_at())
        .ends_at(ends_at())
        .deposit(paid_deposit())
        .source(entities::reservation::Source::Portal(
            entities::PortalProvider::Gingr,
        ))
        .hard_stop(entities::HardStop::MedicalOrMedicationReviewRequired)
        .build()
        .expect_err(
            "ordinary construction must reject terminal reservations with active hard stops",
        );
    assert_eq!(
        terminal_stop,
        entities::reservation::Error::TerminalReservationCannotCarryActiveHardStops
    );
}

#[test]
fn persisted_deposit_rejects_paid_status_without_payment_reference() {
    let impossible = serde_json::json!({
        "amount": { "minor_units": 5000, "currency": "Usd" },
        "refundable_until": null,
        "status": "Paid",
        "payment_reference": null
    });

    let error = serde_json::from_value::<payment::Deposit>(impossible)
        .expect_err("paid persisted deposits must promote through payment-reference validation");

    assert!(
        error
            .to_string()
            .contains("paid deposit requires payment reference")
    );
}

#[test]
fn persisted_reservation_rejects_invalid_periods_and_contradictory_deposit_hard_stops() {
    let invalid_period = serde_json::json!({
        "id": reservation_id(100),
        "location_id": location_id(),
        "customer_id": customer_id(),
        "pet_ids": [pet_id()],
        "service": "Boarding",
        "status": "Confirmed",
        "starts_at": ends_at(),
        "ends_at": starts_at(),
        "deposit": payment::Deposit::required(money::Money::usd(5_000).unwrap()),
        "source": { "Portal": "Gingr" }
    });
    assert!(serde_json::from_value::<entities::Reservation>(invalid_period).is_err());

    let impossible_hard_stop = serde_json::json!({
        "id": reservation_id(101),
        "location_id": location_id(),
        "customer_id": customer_id(),
        "pet_ids": [pet_id()],
        "service": "Boarding",
        "status": "Confirmed",
        "starts_at": starts_at(),
        "ends_at": ends_at(),
        "deposit": paid_deposit(),
        "source": { "Portal": "Gingr" },
        "hard_stops": ["DepositRequired"]
    });
    let error = serde_json::from_value::<entities::Reservation>(impossible_hard_stop)
        .expect_err("paid deposits cannot rehydrate with a deposit-required hard stop");

    assert!(
        error
            .to_string()
            .contains("deposit-required hard stop requires a collectible deposit")
    );

    let empty_pet_relationship = serde_json::json!({
        "id": reservation_id(102),
        "location_id": location_id(),
        "customer_id": customer_id(),
        "pet_ids": [],
        "service": "Boarding",
        "status": "Confirmed",
        "starts_at": starts_at(),
        "ends_at": ends_at(),
        "deposit": payment::Deposit::required(money::Money::usd(5_000).unwrap()),
        "source": { "Portal": "Gingr" }
    });
    assert!(serde_json::from_value::<entities::Reservation>(empty_pet_relationship).is_err());

    let terminal_with_active_hard_stop = serde_json::json!({
        "id": reservation_id(103),
        "location_id": location_id(),
        "customer_id": customer_id(),
        "pet_ids": [pet_id()],
        "service": "Boarding",
        "status": "CheckedOut",
        "starts_at": starts_at(),
        "ends_at": ends_at(),
        "deposit": paid_deposit(),
        "source": { "Portal": "Gingr" },
        "hard_stops": ["MedicalOrMedicationReviewRequired"]
    });
    assert!(
        serde_json::from_value::<entities::Reservation>(terminal_with_active_hard_stop).is_err()
    );
}

#[test]
fn persisted_message_rejects_delivered_drafts_and_queue_states_without_review_gate() {
    let delivered_draft = serde_json::json!({
        "id": entities::MessageId::new(Uuid::from_u128(200)),
        "subject": { "Reservation": reservation_id(100) },
        "direction": "OutboundDraft",
        "channel": "Email",
        "status": "Delivered",
        "body_ref": "message-body/evidence-1",
        "approval_gate": "CustomerMessageApproval",
        "audit_refs": []
    });
    assert!(serde_json::from_value::<entities::Message>(delivered_draft).is_err());

    let approved_without_gate = serde_json::json!({
        "id": entities::MessageId::new(Uuid::from_u128(201)),
        "subject": { "Reservation": reservation_id(100) },
        "direction": "OutboundQueued",
        "channel": "Email",
        "status": "ApprovedToQueue",
        "body_ref": "message-body/evidence-2",
        "approval_gate": null,
        "audit_refs": []
    });
    let error = serde_json::from_value::<entities::Message>(approved_without_gate)
        .expect_err("approved-to-queue persisted messages need review-gate evidence");

    assert!(
        error
            .to_string()
            .contains("queued or sent message rehydration requires opaque queue authorization")
    );

    let queued_with_gate_but_without_decision_evidence = serde_json::json!({
        "id": entities::MessageId::new(Uuid::from_u128(202)),
        "subject": { "Reservation": reservation_id(100) },
        "direction": "OutboundQueued",
        "channel": "Email",
        "status": "Queued",
        "body_ref": "message-body/evidence-without-decision",
        "approval_gate": "CustomerMessageApproval",
        "audit_refs": []
    });
    assert!(
        serde_json::from_value::<entities::Message>(queued_with_gate_but_without_decision_evidence)
            .is_err(),
        "a review-gate label alone must not manufacture historical approval evidence"
    );
}

#[test]
fn persisted_message_rejects_sent_direction_with_draft_or_queue_status() {
    let message_id = entities::MessageId::new(Uuid::from_u128(202));
    let approval = entities::approval::Record::builder()
        .id(entities::approval::Id::new(Uuid::from_u128(205)))
        .target(entities::approval::Target::Message(message_id))
        .gate(policy::ReviewGate::CustomerMessageApproval)
        .lifecycle(entities::approval::Lifecycle::Approved {
            decided_by: actor(),
            decided_at: starts_at(),
        })
        .requested_by(actor())
        .requested_at(starts_at())
        .build()
        .unwrap();
    let evidence = entities::message_record::ApprovalEvidence::try_from_approval(
        &approval,
        message_id,
        policy::ReviewGate::CustomerMessageApproval,
    )
    .unwrap();
    let sent_but_still_draft = serde_json::json!({
        "id": message_id,
        "subject": { "Reservation": reservation_id(100) },
        "direction": "OutboundSent",
        "channel": "Email",
        "status": "DraftCreated",
        "body_ref": "message-body/evidence-3",
        "approval_gate": "CustomerMessageApproval",
        "approval_evidence": evidence,
        "audit_refs": []
    });
    let error = serde_json::from_value::<entities::Message>(sent_but_still_draft)
        .expect_err("sent messages cannot rehydrate as draft lifecycle states");

    assert!(
        error
            .to_string()
            .contains("queued or sent message rehydration requires opaque queue authorization")
    );
}

#[test]
fn persisted_queue_state_cannot_rehydrate_from_historical_evidence_alone() {
    let message_id = entities::MessageId::new(Uuid::from_u128(203));
    let approval_id = entities::approval::Id::new(Uuid::from_u128(204));
    let decided_by = entities::ActorRef::Manager {
        manager_id: entities::ManagerId::try_new("manager-queue-approval").unwrap(),
    };
    let approval = entities::approval::Record::builder()
        .id(approval_id)
        .target(entities::approval::Target::Message(message_id))
        .gate(policy::ReviewGate::CustomerMessageApproval)
        .lifecycle(entities::approval::Lifecycle::Approved {
            decided_by,
            decided_at: starts_at(),
        })
        .requested_by(actor())
        .requested_at(starts_at())
        .build()
        .unwrap();

    let evidence = entities::message_record::ApprovalEvidence::try_from_approval(
        &approval,
        message_id,
        policy::ReviewGate::CustomerMessageApproval,
    )
    .expect("approved message record should produce serializable historical evidence");
    let evidence_json = serde_json::to_string(&evidence).unwrap();
    assert!(evidence_json.contains("CustomerMessageApproval"));

    let persisted = serde_json::json!({
        "id": message_id,
        "subject": { "Reservation": reservation_id(100) },
        "direction": "OutboundQueued",
        "channel": "Email",
        "status": "Queued",
        "body_ref": "message-body/evidence-4",
        "approval_gate": "CustomerMessageApproval",
        "approval_evidence": evidence,
        "audit_refs": []
    });
    let error = serde_json::from_value::<entities::Message>(persisted)
        .expect_err("historical approval fields cannot rehydrate executable queue authority");
    let error_text = error.to_string();
    assert!(
        error_text
            .contains("queued or sent message rehydration requires opaque queue authorization"),
        "unexpected rehydration error: {error_text}"
    );
}

#[test]
fn persisted_workflow_event_rejects_event_type_subject_mismatches() {
    let mismatched = serde_json::json!({
        "event_id": workflow::EventId::new(Uuid::from_u128(300)),
        "event_type": "CheckoutCompleted",
        "occurred_at": starts_at(),
        "actor": actor(),
        "location_id": location_id(),
        "subject": { "Customer": customer_id() },
        "policy_context": {
            "allowed_actions": ["ReadEntities"],
            "automation_level": "DraftOnly",
            "required_reviews": ["ManagerApproval"]
        }
    });
    let error = serde_json::from_value::<workflow::Event>(mismatched)
        .expect_err("checkout workflow events must rehydrate only against reservation subjects");

    assert!(
        error
            .to_string()
            .contains("workflow event subject does not match event type")
    );

    let error = workflow::Event::try_new(
        workflow::EventId::new(Uuid::from_u128(301)),
        workflow::EventType::CheckoutCompleted,
        starts_at(),
        actor(),
        location_id(),
        workflow::Subject::Customer(customer_id()),
        workflow::PolicyContext {
            allowed_actions: vec![workflow::AllowedAction::ReadEntities],
            automation_level: policy::automation::Level::DraftOnly,
            required_reviews: vec![policy::ReviewGate::ManagerApproval],
        },
    )
    .expect_err("ordinary event construction must share persisted subject/type validation");

    assert_eq!(error, workflow::EventError::EventTypeSubjectMismatch);
}

#[test]
fn persisted_workflow_results_reject_status_reason_output_mismatches() {
    let needs_review_without_reason = serde_json::json!({
        "status": "NeedsHumanReview",
        "summary": "Vaccine proof needs manager review.",
        "structured_output": null,
        "recommended_actions": [],
        "risk_flags": [],
        "verification": ["Matched vaccine policy gate."],
        "human_review_reason": null
    });

    let error =
        serde_json::from_value::<workflow::Result<serde_json::Value>>(needs_review_without_reason)
            .expect_err("review-required workflow results must carry review evidence");

    assert!(
        error
            .to_string()
            .contains("workflow outcome needing human review requires review reason evidence")
    );

    let caller_completed = serde_json::json!({
        "status": "Completed",
        "summary": "Daily care update draft prepared.",
        "structured_output": { "draft_id": "daily-update-1" },
        "recommended_actions": [{
            "InternalTask": {
                "title": "Send customer draft",
                "body": "Caller-selected recommended action"
            }
        }],
        "risk_flags": [],
        "verification": ["Caller-provided verification label."],
        "human_review_reason": null
    });

    let error = serde_json::from_value::<workflow::Result<serde_json::Value>>(caller_completed)
        .expect_err("serialized completion labels must remain non-authoritative evidence");

    assert!(
        error
            .to_string()
            .contains("serialized completed workflow outcome is not accepted")
    );

    let review_required_with_completed_output = serde_json::json!({
        "status": "NeedsHumanReview",
        "summary": "Vaccine proof needs manager review.",
        "structured_output": { "draft_id": "must-not-survive-rehydration" },
        "recommended_actions": [],
        "risk_flags": [],
        "verification": ["Matched vaccine policy gate."],
        "human_review_reason": "manager approval is still required"
    });

    let error = serde_json::from_value::<workflow::Result<serde_json::Value>>(
        review_required_with_completed_output,
    )
    .expect_err("non-completed workflow results must reject completed output evidence");

    assert!(
        error
            .to_string()
            .contains("non-completed workflow outcome must not carry structured output")
    );
}

#[test]
fn document_vaccine_incident_approval_and_task_rehydration_reject_relationship_bypasses() {
    let unsafe_verified_document = serde_json::json!({
        "id": entities::DocumentId::new(Uuid::from_u128(400)),
        "location_id": location_id(),
        "subject": { "Pet": pet_id() },
        "classification": "VaccineProof",
        "source": "CustomerUpload",
        "uploaded_by_actor": actor(),
        "uploaded_at": starts_at(),
        "original_file": original_file_json(),
        "storage_ref": storage_ref_json(),
        "virus_scan_status": "Pending",
        "pii_redaction_status": "Redacted",
        "verification_status": "Verified",
        "audit_refs": []
    });
    let error = serde_json::from_value::<entities::Document>(unsafe_verified_document)
        .expect_err("verified documents must require passed scan and safe redaction evidence");
    assert!(
        error
            .to_string()
            .contains("verified document requires passed virus scan")
    );

    let vaccine_expired_without_expiration = serde_json::json!({
        "id": entities::VaccineRecordId::new(Uuid::from_u128(401)),
        "pet_id": pet_id(),
        "vaccine_name": "Rabies",
        "source_document_id": entities::DocumentId::new(Uuid::from_u128(400)),
        "status": "VerifiedExpired",
        "effective_on": "2026-01-01",
        "expires_on": null,
        "review_gate": "MedicalDocumentReview",
        "audit_refs": []
    });
    let error =
        serde_json::from_value::<entities::VaccineRecord>(vaccine_expired_without_expiration)
            .expect_err("expired vaccine status must carry an expiration date");
    assert!(
        error
            .to_string()
            .contains("expired vaccine status requires an expiration date")
    );

    let critical_incident_without_gate = serde_json::json!({
        "id": entities::IncidentId::new(Uuid::from_u128(402)),
        "location_id": location_id(),
        "primary_subject": { "Pet": pet_id() },
        "category": "Medication",
        "severity": "Critical",
        "status": "Reported",
        "reported_by": actor(),
        "reported_at": starts_at(),
        "summary": "missed medication dose",
        "required_review_gates": [],
        "audit_refs": []
    });
    let error = serde_json::from_value::<entities::Incident>(critical_incident_without_gate)
        .expect_err("critical incidents must rehydrate with manager-review requirements");
    assert!(
        error
            .to_string()
            .contains("incident requires manager approval review gate")
    );

    let terminal_approval_before_request = serde_json::json!({
        "id": entities::approval::Id::new(Uuid::from_u128(403)),
        "target": { "Message": entities::MessageId::new(Uuid::from_u128(404)) },
        "gate": "CustomerMessageApproval",
        "lifecycle": { "Approved": { "decided_by": actor(), "decided_at": starts_at() } },
        "requested_by": actor(),
        "requested_at": ends_at(),
        "audit_refs": []
    });
    let error =
        serde_json::from_value::<entities::approval::Record>(terminal_approval_before_request)
            .expect_err("terminal approvals must not decide before the request exists");
    assert!(
        error
            .to_string()
            .contains("approval decision time cannot precede request time")
    );
}

#[test]
fn document_vaccine_incident_approval_and_task_builders_reject_relationship_bypasses() {
    let document_error = entities::Document::builder()
        .id(entities::DocumentId::new(Uuid::from_u128(410)))
        .location_id(location_id())
        .subject(entities::DocumentSubject::Pet(pet_id()))
        .classification(document::Classification::VaccineProof)
        .source(document::Source::CustomerUpload)
        .uploaded_by_actor(actor())
        .uploaded_at(starts_at())
        .original_file(
            serde_json::from_value(original_file_json()).expect("valid original file fixture"),
        )
        .storage_ref(serde_json::from_value(storage_ref_json()).expect("valid storage ref fixture"))
        .pii_redaction_status(document::PiiRedactionStatus::Redacted)
        .virus_scan_status(document::VirusScanStatus::Pending)
        .verification_status(document::Status::Verified)
        .build()
        .expect_err("ordinary document construction must enforce scan/review coherence");
    assert!(
        document_error
            .to_string()
            .contains("verified document requires passed virus scan")
    );

    let vaccine_error = entities::VaccineRecord::builder()
        .id(entities::VaccineRecordId::new(Uuid::from_u128(411)))
        .pet_id(pet_id())
        .vaccine_name(policy::VaccineName::try_new("Rabies").unwrap())
        .source_document_id(entities::DocumentId::new(Uuid::from_u128(410)))
        .status(vaccine::Status::ExceptionApproved)
        .effective_on(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
        .expires_on(chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap())
        .review_gate(policy::ReviewGate::MedicalDocumentReview)
        .build()
        .expect_err("ordinary vaccine construction must enforce status/gate coherence");
    assert!(
        vaccine_error
            .to_string()
            .contains("vaccine exception status requires manager approval review gate")
    );

    let incident_error = entities::Incident::builder()
        .id(entities::IncidentId::new(Uuid::from_u128(412)))
        .location_id(location_id())
        .primary_subject(entities::IncidentSubject::Pet(pet_id()))
        .category(incident::Category::Medication)
        .severity(incident::Severity::Medium)
        .reported_by(actor())
        .reported_at(starts_at())
        .summary(incident::Summary::try_new("missed medication dose").unwrap())
        .status(incident::Status::CustomerMessageReview)
        .required_review_gates(vec![policy::ReviewGate::ManagerApproval])
        .build()
        .expect_err("ordinary incident construction must enforce review-gate requirements");
    assert!(
        incident_error
            .to_string()
            .contains("customer-message incident requires customer message approval review gate")
    );

    let approval_error = entities::approval::Record::builder()
        .id(entities::approval::Id::new(Uuid::from_u128(413)))
        .target(entities::approval::Target::Incident(
            entities::IncidentId::new(Uuid::from_u128(414)),
        ))
        .gate(policy::ReviewGate::CustomerMessageApproval)
        .lifecycle(entities::approval::Lifecycle::ApprovalRequested)
        .requested_by(actor())
        .requested_at(starts_at())
        .build()
        .expect_err("ordinary approval construction must enforce target/gate agreement");
    assert!(
        approval_error
            .to_string()
            .contains("approval review gate does not match approval target")
    );
}

fn original_file_json() -> serde_json::Value {
    serde_json::json!({
        "filename": "rabies.pdf",
        "mime_type": "application/pdf",
        "content_length": 42,
        "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    })
}

fn storage_ref_json() -> serde_json::Value {
    serde_json::json!({
        "bucket": "vaccine-documents",
        "key": "pets/moose/rabies.pdf",
        "version": "v1"
    })
}
