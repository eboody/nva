use chrono::{TimeZone, Utc};
use domain::{entities, incident, message, policy};
use uuid::Uuid;

fn at(hour: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 15, hour, 0, 0).unwrap()
}

fn customer_id(value: u128) -> entities::CustomerId {
    entities::CustomerId::new(Uuid::from_u128(value))
}

#[test]
fn historical_system_approval_remains_serializable_evidence_not_authority() {
    let message_id = entities::MessageId::new(Uuid::from_u128(20));
    let approval = entities::approval::Record::builder()
        .id(entities::approval::Id::new(Uuid::from_u128(21)))
        .target(entities::approval::Target::Message(message_id))
        .gate(policy::ReviewGate::CustomerMessageApproval)
        .lifecycle(entities::approval::Lifecycle::Approved {
            decided_by: entities::ActorRef::System,
            decided_at: at(10),
        })
        .requested_by(entities::ActorRef::System)
        .requested_at(at(9))
        .build()
        .unwrap();

    let evidence = entities::message_record::ApprovalEvidence::try_from_approval(
        &approval,
        message_id,
        policy::ReviewGate::CustomerMessageApproval,
    )
    .expect("historical evidence preserves the source decision without granting authority");

    assert!(serde_json::to_value(evidence).is_ok());
}

#[test]
fn high_severity_incident_cannot_close_with_only_a_gate_label() {
    let result = entities::Incident::builder()
        .id(entities::IncidentId::new(Uuid::from_u128(30)))
        .location_id(entities::LocationId::new(Uuid::from_u128(31)))
        .primary_subject(entities::IncidentSubject::Customer(customer_id(1)))
        .category(incident::Category::CustomerService)
        .severity(incident::Severity::High)
        .status(incident::Status::Closed)
        .reported_by(entities::ActorRef::System)
        .reported_at(at(8))
        .summary(incident::Summary::try_new("Customer escalation reviewed.").unwrap())
        .required_review_gates(vec![policy::ReviewGate::ManagerApproval])
        .build();

    assert!(
        result.is_err(),
        "a required-gate label is not closure proof"
    );
}

#[test]
fn exact_incident_approval_remains_historical_closure_evidence_not_authority() {
    let incident_id = entities::IncidentId::new(Uuid::from_u128(40));
    let incident = entities::Incident::builder()
        .id(incident_id)
        .location_id(entities::LocationId::new(Uuid::from_u128(41)))
        .primary_subject(entities::IncidentSubject::Customer(customer_id(1)))
        .category(incident::Category::CustomerService)
        .severity(incident::Severity::High)
        .status(incident::Status::NeedsManagerReview)
        .reported_by(entities::ActorRef::System)
        .reported_at(at(8))
        .summary(incident::Summary::try_new("Customer escalation reviewed.").unwrap())
        .required_review_gates(vec![policy::ReviewGate::ManagerApproval])
        .build()
        .unwrap();
    let approval = entities::approval::Record::builder()
        .id(entities::approval::Id::new(Uuid::from_u128(42)))
        .target(entities::approval::Target::Incident(incident_id))
        .gate(policy::ReviewGate::ManagerApproval)
        .lifecycle(entities::approval::Lifecycle::Approved {
            decided_by: entities::ActorRef::Manager {
                manager_id: entities::ManagerId::try_new("incident-manager").unwrap(),
            },
            decided_at: at(10),
        })
        .requested_by(entities::ActorRef::System)
        .requested_at(at(9))
        .build()
        .unwrap();

    let evidence =
        entities::incident_record::ClosureEvidence::try_from_approval(&approval, incident_id)
            .unwrap();
    assert_eq!(incident.status(), incident::Status::NeedsManagerReview);
    assert_eq!(evidence.incident_id(), incident_id);
    assert!(serde_json::to_value(evidence).is_ok());
}

#[test]
fn forged_closure_fields_cannot_rehydrate_a_closed_incident() {
    let incident_id = entities::IncidentId::new(Uuid::from_u128(44));
    let forged = serde_json::json!({
        "id": incident_id,
        "location_id": entities::LocationId::new(Uuid::from_u128(45)),
        "primary_subject": { "Customer": customer_id(1) },
        "category": "CustomerService",
        "severity": "High",
        "status": "Closed",
        "reported_by": entities::ActorRef::System,
        "reported_at": at(8),
        "summary": "fabricated closure must not rehydrate",
        "required_review_gates": ["ManagerApproval"],
        "closure_evidence": {
            "approval_id": entities::approval::Id::new(Uuid::from_u128(46)),
            "incident_id": incident_id,
            "decided_by": entities::ActorRef::System,
            "decided_at": at(9)
        },
        "audit_refs": []
    });

    let error = serde_json::from_value::<entities::Incident>(forged)
        .expect_err("historical fields cannot manufacture trusted closure approval");
    assert!(
        error
            .to_string()
            .contains("closed incident rehydration requires a trusted approval aggregate")
    );
}

#[allow(dead_code)]
fn approved_draft(message_id: entities::MessageId) -> entities::Message {
    entities::Message::approval_requested_outbound_draft(
        message_id,
        entities::MessageSubject::Customer(customer_id(1)),
        message::Channel::Email,
        message::BodyRef::try_new("message/body/1").unwrap(),
        policy::ReviewGate::CustomerMessageApproval,
    )
}
