use chrono::{TimeZone, Utc};
use domain::{document, entities, incident, message, money, payment, policy, vaccine, workflow};
use uuid::Uuid;

fn main() {
    let _bypass = entities::Reservation {
        id: entities::reservation::Id::new(uuid::Uuid::from_u128(1)),
        location_id: entities::LocationId::new(Uuid::from_u128(2)),
        customer_id: entities::CustomerId::new(Uuid::from_u128(3)),
        pet_ids: Vec::new(),
        service: entities::ServiceKind::Boarding,
        status: entities::reservation::Status::Confirmed,
        starts_at: Utc.with_ymd_and_hms(2026, 8, 12, 17, 0, 0).unwrap(),
        ends_at: Utc.with_ymd_and_hms(2026, 8, 12, 9, 0, 0).unwrap(),
        deposit: Some(payment::Deposit::required(
            money::Money::usd(5_000).unwrap(),
        )),
        source: entities::reservation::Source::Portal(entities::PortalProvider::ProviderHosted),
        requested_add_ons: Vec::new(),
        hard_stops: Vec::new(),
    };

    let _message_bypass = entities::Message {
        id: entities::MessageId::new(Uuid::from_u128(23)),
        subject: entities::MessageSubject::Reservation(entities::reservation::Id::new(
            Uuid::from_u128(24),
        )),
        direction: message::Direction::InboundReceived,
        channel: message::Channel::Email,
        status: message::Status::Queued,
        body_ref: message::BodyRef::try_new("message-body/bypass").unwrap(),
        approval_gate: Some(domain::policy::ReviewGate::CustomerMessageApproval),
        audit_refs: Vec::new(),
    };

    let _workflow_event_bypass = workflow::Event {
        event_id: workflow::EventId::new(Uuid::from_u128(35)),
        event_type: workflow::EventType::CheckoutCompleted,
        occurred_at: Utc.with_ymd_and_hms(2026, 8, 12, 9, 0, 0).unwrap(),
        actor: entities::ActorRef::System,
        location_id: entities::LocationId::new(Uuid::from_u128(36)),
        subject: workflow::Subject::Customer(entities::CustomerId::new(Uuid::from_u128(37))),
        policy_context: workflow::PolicyContext {
            allowed_actions: vec![workflow::AllowedAction::ReadEntities],
            automation_level: policy::automation::Level::DraftOnly,
            required_reviews: vec![policy::ReviewGate::ManagerApproval],
        },
    };

    let _document_bypass = entities::Document {
        id: entities::DocumentId::new(Uuid::from_u128(50)),
        location_id: entities::LocationId::new(Uuid::from_u128(51)),
        subject: entities::DocumentSubject::Pet(entities::PetId::new(Uuid::from_u128(52))),
        classification: document::Classification::VaccineProof,
        source: document::Source::CustomerUpload,
        uploaded_by_actor: entities::ActorRef::System,
        uploaded_at: Utc.with_ymd_and_hms(2026, 8, 12, 9, 0, 0).unwrap(),
        original_file: serde_json::from_value(serde_json::json!({
            "filename": "rabies.pdf",
            "mime_type": "application/pdf",
            "content_length": 42,
            "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        })).unwrap(),
        storage_ref: serde_json::from_value(serde_json::json!({
            "bucket": "vaccine-documents",
            "key": "pets/moose/rabies.pdf",
            "version": "v1"
        })).unwrap(),
        virus_scan_status: document::VirusScanStatus::Pending,
        pii_redaction_status: document::PiiRedactionStatus::Pending,
        verification_status: document::Status::Verified,
        audit_refs: Vec::new(),
    };

    let _vaccine_bypass = entities::VaccineRecord {
        id: entities::VaccineRecordId::new(Uuid::from_u128(53)),
        pet_id: entities::PetId::new(Uuid::from_u128(54)),
        vaccine_name: policy::VaccineName::try_new("Rabies").unwrap(),
        source_document_id: entities::DocumentId::new(Uuid::from_u128(55)),
        status: vaccine::Status::ExceptionApproved,
        effective_on: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        expires_on: None,
        review_gate: policy::ReviewGate::MedicalDocumentReview,
        audit_refs: Vec::new(),
    };

    let _incident_bypass = entities::Incident {
        id: entities::IncidentId::new(Uuid::from_u128(56)),
        location_id: entities::LocationId::new(Uuid::from_u128(57)),
        primary_subject: entities::IncidentSubject::Pet(entities::PetId::new(Uuid::from_u128(58))),
        category: incident::Category::Medication,
        severity: incident::Severity::Critical,
        status: incident::Status::CustomerMessageReview,
        reported_by: entities::ActorRef::System,
        reported_at: Utc.with_ymd_and_hms(2026, 8, 12, 9, 0, 0).unwrap(),
        summary: incident::Summary::try_new("missed medication dose").unwrap(),
        required_review_gates: Vec::new(),
        audit_refs: Vec::new(),
    };
}
