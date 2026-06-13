use gingr::{response, webhook};

const FIXTURE_KEY: &str = "test-webhook-signature-key";

#[test]
fn reservation_check_out_fixture_verifies_before_payload_can_be_inspected() {
    let raw = include_str!(
        "../../../docs/integrations/gingr/fixtures/webhooks/reservation-check-out.json"
    );
    let key = webhook::SignatureKey::from_secret(FIXTURE_KEY);

    let envelope =
        webhook::Envelope::from_json(raw).expect("fixture parses as unverified boundary payload");
    let verified = envelope
        .verify(&key)
        .expect("fixture signature matches fake key");

    assert_eq!(verified.event_type(), webhook::EventType::CheckOut);
    assert_eq!(verified.entity_id().as_str(), "76390");
    assert_eq!(verified.entity_type(), webhook::EntityType::Reservation);
    assert_eq!(verified.payload().entity_data()["animal_name"], "Bella");
}

#[test]
fn email_sent_fixture_normalizes_numeric_entity_id_for_signature_verification() {
    let raw = include_str!("../../../docs/integrations/gingr/fixtures/webhooks/email-sent.json");
    let key = webhook::SignatureKey::from_secret(FIXTURE_KEY);

    let verified = webhook::Envelope::from_json(raw)
        .expect("fixture parses")
        .verify(&key)
        .expect("numeric entity id verifies with decimal text normalization");

    assert_eq!(verified.event_type(), webhook::EventType::EmailSent);
    assert_eq!(verified.entity_id().as_str(), "5917");
    assert_eq!(verified.entity_type(), webhook::EntityType::Owner);
    assert_eq!(
        verified.payload().email_data().unwrap()["subject"],
        "Sanitized email subject"
    );
    assert_eq!(
        verified.payload().recipients().unwrap()[0]["email"],
        "recipient@example.test"
    );
}

#[test]
fn unverified_debug_output_exposes_only_verification_metadata_not_payload() {
    let raw = include_str!(
        "../../../docs/integrations/gingr/fixtures/webhooks/reservation-check-out.json"
    );

    let envelope = webhook::Envelope::from_json(raw).expect("fixture parses");
    let debug = format!("{envelope:?}");

    assert!(debug.contains("webhook_type"));
    assert!(debug.contains("entity_type"));
    assert!(debug.contains("signature_present"));
    assert!(!debug.contains("Bella"));
    assert!(!debug.contains("Sample Owner"));
    assert!(!debug.contains("entity_data"));
}

#[test]
fn verification_rejects_tampered_signatures_without_exposing_secret_material() {
    let raw = include_str!(
        "../../../docs/integrations/gingr/fixtures/webhooks/reservation-check-out.json"
    )
    .replace(
        "e6d62e27528513a9c4aa399e3a79192aacc490cfeae202fa753f319967ab30eb",
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    );
    let key = webhook::SignatureKey::from_secret(FIXTURE_KEY);

    let error = webhook::Envelope::from_json(&raw)
        .expect("tampered fixture still parses")
        .verify(&key)
        .expect_err("tampered signature must not verify");

    assert_eq!(error, webhook::VerificationError::SignatureMismatch);
    assert!(!format!("{error:?}").contains(FIXTURE_KEY));
    assert!(!error.to_string().contains(FIXTURE_KEY));
}

#[test]
fn verification_reports_unsupported_entity_id_and_malformed_signature_boundaries() {
    let key = webhook::SignatureKey::from_secret(FIXTURE_KEY);
    let unsupported_entity_id = r#"{
        "webhook_url":"https://example.test/gingr/webhooks",
        "webhook_type":"check_out",
        "entity_id":{"nested":"76390"},
        "entity_type":"reservation",
        "signature":"e6d62e27528513a9c4aa399e3a79192aacc490cfeae202fa753f319967ab30eb",
        "entity_data":{}
    }"#;
    let malformed_signature = r#"{
        "webhook_url":"https://example.test/gingr/webhooks",
        "webhook_type":"check_out",
        "entity_id":"76390",
        "entity_type":"reservation",
        "signature":"not-lowercase-hex",
        "entity_data":{}
    }"#;

    let entity_id_error = webhook::Envelope::from_json(unsupported_entity_id)
        .expect("boundary payload parses")
        .verify(&key)
        .expect_err("object entity_id is unsupported");
    assert!(matches!(
        entity_id_error,
        webhook::VerificationError::UnsupportedEntityId { .. }
    ));

    let signature_error = webhook::Envelope::from_json(malformed_signature)
        .expect("boundary payload parses")
        .verify(&key)
        .expect_err("malformed signature is rejected before comparison");
    assert!(matches!(
        signature_error,
        webhook::VerificationError::MalformedSignature { .. }
    ));
}

#[test]
fn receiver_ack_semantics_match_gingr_retry_contract() {
    assert_eq!(
        webhook::Ack::Processed.http_status(),
        response::HttpStatus::OK
    );
    assert_eq!(
        webhook::Ack::RejectedPermanently.http_status(),
        response::HttpStatus::FORBIDDEN
    );
    assert_eq!(
        webhook::Ack::RetryableFailure.http_status(),
        response::HttpStatus::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        webhook::Ack::retryable_status(response::HttpStatus::new(503)).http_status(),
        response::HttpStatus::new(503)
    );
    assert_eq!(
        webhook::Ack::retryable_status(response::HttpStatus::OK).http_status(),
        response::HttpStatus::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        webhook::Ack::retryable_status(response::HttpStatus::FORBIDDEN).http_status(),
        response::HttpStatus::INTERNAL_SERVER_ERROR
    );
}
