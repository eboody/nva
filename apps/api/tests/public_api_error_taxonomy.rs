use axum::response::IntoResponse;
use pet_resort_api::error::{
    AuthenticationFailure, AuthorizationFailure, ErrorContext, ErrorKind, PublicApiError,
};
use pet_resort_api::public_contract::{ErrorDetail, LiveSideEffectsMode};

#[test]
fn every_public_failure_class_has_one_stable_safe_envelope_conversion() {
    let cases = [
        (
            ErrorKind::Authentication(AuthenticationFailure::MissingTrustedActorContext),
            401,
            "missing_trusted_actor_context",
            "authentication_failed",
        ),
        (
            ErrorKind::Authorization(AuthorizationFailure::BodyActorClaimMismatch),
            403,
            "body_actor_claim_mismatch",
            "authorization_failed",
        ),
        (
            ErrorKind::Validation {
                details: vec![ErrorDetail::field(
                    "source_refs".to_owned(),
                    "required".to_owned(),
                )],
            },
            422,
            "workflow_validation_failed",
            "validation_failed",
        ),
        (
            ErrorKind::ReviewConflict,
            409,
            "review_conflict",
            "review_conflict",
        ),
        (
            ErrorKind::IdempotencyConflict,
            409,
            "idempotency_conflict",
            "idempotency_conflict",
        ),
        (
            ErrorKind::SourceAmbiguity,
            422,
            "source_ambiguity",
            "source_ambiguity",
        ),
        (
            ErrorKind::Unavailable,
            503,
            "dependency_unavailable",
            "unavailable",
        ),
        (ErrorKind::Internal, 500, "internal_error", "internal_error"),
    ];

    for (kind, expected_status, expected_code, expected_class) in cases {
        let failure = PublicApiError::new(
            kind,
            ErrorContext::new("request-17").with_correlation_id("correlation-17"),
        );
        let envelope = failure.envelope();

        assert_eq!(failure.status_code().as_u16(), expected_status);
        assert_eq!(envelope.error.code, expected_code);
        assert_eq!(envelope.error.safe_error_class, expected_class);
        assert_eq!(envelope.request_id, "request-17");
        assert_eq!(envelope.correlation_id.as_deref(), Some("correlation-17"));
        assert_eq!(envelope.live_side_effects, LiveSideEffectsMode::Disabled);

        let response = failure.into_response();
        assert_eq!(response.status().as_u16(), expected_status);
    }
}

#[test]
fn unavailable_and_internal_envelopes_cannot_serialize_private_error_sources() {
    let private_source = "postgres://operator:secret@example.invalid/pet_resort";

    for kind in [ErrorKind::Unavailable, ErrorKind::Internal] {
        let serialized = serde_json::to_string(
            &PublicApiError::new(kind, ErrorContext::new("request-redacted")).envelope(),
        )
        .expect("safe error envelope serializes");

        assert!(!serialized.contains(private_source));
        assert!(!serialized.contains("operator"));
        assert!(!serialized.contains("secret"));
    }
}
