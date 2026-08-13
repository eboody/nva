use chrono::{TimeZone, Utc};
use domain::{entities, message, money, payment, policy, workflow};
use uuid::Uuid;

fn reservation_id(value: u128) -> entities::reservation::Id {
    entities::reservation::Id(Uuid::from_u128(value))
}

fn location_id() -> entities::LocationId {
    entities::LocationId(Uuid::from_u128(10))
}

fn customer_id() -> entities::CustomerId {
    entities::CustomerId(Uuid::from_u128(11))
}

fn pet_id() -> entities::PetId {
    entities::PetId(Uuid::from_u128(12))
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
    let invalid_period = entities::Reservation {
        id: reservation_id(100),
        location_id: location_id(),
        customer_id: customer_id(),
        pet_ids: vec![pet_id()],
        service: entities::ServiceKind::Boarding,
        status: entities::reservation::Status::Confirmed,
        starts_at: ends_at(),
        ends_at: starts_at(),
        deposit: Some(payment::Deposit::required(
            money::Money::usd(5_000).unwrap(),
        )),
        source: entities::reservation::Source::Portal(entities::PortalProvider::Gingr),
        requested_add_ons: Vec::new(),
        hard_stops: Vec::new(),
    };
    let json = serde_json::to_string(&invalid_period).unwrap();
    assert!(serde_json::from_str::<entities::Reservation>(&json).is_err());

    let impossible_hard_stop = entities::Reservation {
        id: reservation_id(101),
        location_id: location_id(),
        customer_id: customer_id(),
        pet_ids: vec![pet_id()],
        service: entities::ServiceKind::Boarding,
        status: entities::reservation::Status::Confirmed,
        starts_at: starts_at(),
        ends_at: ends_at(),
        deposit: Some(paid_deposit()),
        source: entities::reservation::Source::Portal(entities::PortalProvider::Gingr),
        requested_add_ons: Vec::new(),
        hard_stops: vec![entities::HardStop::DepositRequired],
    };
    let json = serde_json::to_string(&impossible_hard_stop).unwrap();
    let error = serde_json::from_str::<entities::Reservation>(&json)
        .expect_err("paid deposits cannot rehydrate with a deposit-required hard stop");

    assert!(
        error
            .to_string()
            .contains("deposit-required hard stop requires a collectible deposit")
    );

    let empty_pet_relationship = entities::Reservation {
        id: reservation_id(102),
        location_id: location_id(),
        customer_id: customer_id(),
        pet_ids: Vec::new(),
        service: entities::ServiceKind::Boarding,
        status: entities::reservation::Status::Confirmed,
        starts_at: starts_at(),
        ends_at: ends_at(),
        deposit: Some(payment::Deposit::required(
            money::Money::usd(5_000).unwrap(),
        )),
        source: entities::reservation::Source::Portal(entities::PortalProvider::Gingr),
        requested_add_ons: Vec::new(),
        hard_stops: Vec::new(),
    };
    let json = serde_json::to_string(&empty_pet_relationship).unwrap();
    assert!(serde_json::from_str::<entities::Reservation>(&json).is_err());

    let terminal_with_active_hard_stop = entities::Reservation {
        id: reservation_id(103),
        location_id: location_id(),
        customer_id: customer_id(),
        pet_ids: vec![pet_id()],
        service: entities::ServiceKind::Boarding,
        status: entities::reservation::Status::CheckedOut,
        starts_at: starts_at(),
        ends_at: ends_at(),
        deposit: Some(paid_deposit()),
        source: entities::reservation::Source::Portal(entities::PortalProvider::Gingr),
        requested_add_ons: Vec::new(),
        hard_stops: vec![entities::HardStop::MedicalOrMedicationReviewRequired],
    };
    let json = serde_json::to_string(&terminal_with_active_hard_stop).unwrap();
    assert!(serde_json::from_str::<entities::Reservation>(&json).is_err());
}

#[test]
fn persisted_message_rejects_delivered_drafts_and_queue_states_without_review_gate() {
    let delivered_draft = entities::Message {
        id: entities::MessageId(Uuid::from_u128(200)),
        subject: entities::MessageSubject::Reservation(reservation_id(100)),
        direction: message::Direction::OutboundDraft,
        channel: message::Channel::Email,
        status: message::Status::Delivered,
        body_ref: message::BodyRef::try_new("message-body/evidence-1").unwrap(),
        approval_gate: Some(policy::ReviewGate::CustomerMessageApproval),
        audit_refs: Vec::new(),
    };
    let json = serde_json::to_string(&delivered_draft).unwrap();
    assert!(serde_json::from_str::<entities::Message>(&json).is_err());

    let approved_without_gate = entities::Message {
        id: entities::MessageId(Uuid::from_u128(201)),
        subject: entities::MessageSubject::Reservation(reservation_id(100)),
        direction: message::Direction::OutboundQueued,
        channel: message::Channel::Email,
        status: message::Status::ApprovedToQueue,
        body_ref: message::BodyRef::try_new("message-body/evidence-2").unwrap(),
        approval_gate: None,
        audit_refs: Vec::new(),
    };
    let json = serde_json::to_string(&approved_without_gate).unwrap();
    let error = serde_json::from_str::<entities::Message>(&json)
        .expect_err("approved-to-queue persisted messages need review-gate evidence");

    assert!(
        error
            .to_string()
            .contains("queued or approved outbound message requires approval gate evidence")
    );
}

#[test]
fn persisted_message_rejects_sent_direction_with_draft_or_queue_status() {
    let sent_but_still_draft = entities::Message {
        id: entities::MessageId(Uuid::from_u128(202)),
        subject: entities::MessageSubject::Reservation(reservation_id(100)),
        direction: message::Direction::OutboundSent,
        channel: message::Channel::Email,
        status: message::Status::DraftCreated,
        body_ref: message::BodyRef::try_new("message-body/evidence-3").unwrap(),
        approval_gate: Some(policy::ReviewGate::CustomerMessageApproval),
        audit_refs: Vec::new(),
    };
    let json = serde_json::to_string(&sent_but_still_draft).unwrap();
    let error = serde_json::from_str::<entities::Message>(&json)
        .expect_err("sent messages cannot rehydrate as draft lifecycle states");

    assert!(
        error
            .to_string()
            .contains("outbound sent message requires attempted, delivered, or failed status")
    );
}

#[test]
fn persisted_workflow_event_rejects_event_type_subject_mismatches() {
    let mismatched = workflow::Event {
        event_id: workflow::EventId(Uuid::from_u128(300)),
        event_type: workflow::EventType::CheckoutCompleted,
        occurred_at: starts_at(),
        actor: actor(),
        location_id: location_id(),
        subject: workflow::Subject::Customer(customer_id()),
        policy_context: workflow::PolicyContext {
            allowed_actions: vec![workflow::AllowedAction::ReadEntities],
            automation_level: policy::automation::Level::DraftOnly,
            required_reviews: vec![policy::ReviewGate::ManagerApproval],
        },
    };
    let json = serde_json::to_string(&mismatched).unwrap();
    let error = serde_json::from_str::<workflow::Event>(&json)
        .expect_err("checkout workflow events must rehydrate only against reservation subjects");

    assert!(
        error
            .to_string()
            .contains("workflow event subject does not match event type")
    );
}
