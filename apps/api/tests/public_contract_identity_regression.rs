use axum::body::Body;
use http_body_util::BodyExt;
use pet_resort_api::{
    http,
    public_contract::{
        ActorRef, ApiContractMetadata, DataQualityHygieneContextResponse,
        DataQualityHygieneOutcomeCaptureRequest, LiveSideEffectsMode, OutcomeAudit,
        ProviderBoundaryMode, SourceRecordRef,
    },
};
use serde_json::json;
use tower::ServiceExt;

#[test]
fn v0_contract_metadata_round_trips_with_semantic_protocol_modes() {
    let metadata = ApiContractMetadata::operations_v0("data_quality_hygiene");
    let snapshot = json!({
        "owner": "pet_resort_api",
        "boundary": "api_runtime_dto",
        "schema_version": "pet_resort_api.runtime.v0",
        "workflow": "data_quality_hygiene",
        "provider_boundary": "evidence_refs_only",
        "live_side_effects": "disabled"
    });

    assert_eq!(serde_json::to_value(&metadata).unwrap(), snapshot);
    assert_eq!(
        serde_json::from_value::<ApiContractMetadata>(snapshot).unwrap(),
        metadata
    );
    assert_eq!(
        metadata.provider_boundary,
        ProviderBoundaryMode::EvidenceRefsOnly
    );
    assert_eq!(metadata.live_side_effects, LiveSideEffectsMode::Disabled);
}

#[test]
fn v0_source_record_ref_round_trips_without_an_untyped_value_boundary() {
    let snapshot = json!({
        "system": "gingr",
        "record_type": "customer",
        "record_id": "customer-17",
        "observed_at": "2026-06-17T09:05:00Z",
        "adapter_version": "gingr-v0-readonly"
    });

    let source_ref = serde_json::from_value::<SourceRecordRef>(snapshot.clone()).unwrap();
    assert_eq!(serde_json::to_value(source_ref).unwrap(), snapshot);
}

#[test]
fn canonical_public_contract_contains_no_known_schema_json_value_fields() {
    let source = include_str!("../src/public_contract.rs");

    assert!(!source.contains("serde_json::Value"));
    assert!(!source.contains(": Value"));
    assert!(!source.contains("Vec<Value>"));
}

#[test]
fn sensitive_outcome_capture_debug_redacts_identity_feedback_and_provenance() {
    let request = DataQualityHygieneOutcomeCaptureRequest {
        outcome: "completed".to_owned(),
        actual_minutes: 7,
        actor: ActorRef {
            persona: "front_desk_lead".to_owned(),
            id: "sensitive-actor-17".to_owned(),
            actor_role: None,
        },
        feedback: "sensitive customer context".to_owned(),
        source_refs: vec![SourceRecordRef {
            system: "gingr".to_owned(),
            record_type: "customer".to_owned(),
            record_id: "sensitive-record-42".to_owned(),
            observed_at: "2026-06-17T00:00:00Z".to_owned(),
            adapter_version: "gingr-v0-readonly".to_owned(),
        }],
        issue_refs: vec!["sensitive-issue-9".to_owned()],
        resolution_status_after_review: "acknowledged".to_owned(),
        timestamp: "2026-06-17T12:00:00Z".to_owned(),
        audit: OutcomeAudit {
            correlation_id: "sensitive-correlation".to_owned(),
        },
        requested_side_effects: Vec::new(),
        idempotency_key: Some("sensitive-idempotency-key".to_owned()),
    };

    let debug = format!("{request:?}");
    for sensitive in [
        "sensitive-actor-17",
        "sensitive customer context",
        "sensitive-record-42",
        "sensitive-issue-9",
        "sensitive-correlation",
        "sensitive-idempotency-key",
    ] {
        assert!(!debug.contains(sensitive), "Debug leaked {sensitive}");
    }
    assert!(debug.contains("[REDACTED]"));
}

#[tokio::test]
async fn v0_data_quality_context_round_trips_through_the_canonical_public_dto() {
    let response = http::router_with_test_auth_state(http::VaccineDocumentState::default())
        .oneshot(
            axum::http::Request::builder()
                .uri("/v0/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17")
                .header("x-test-auth-actor-id", "general-manager-17")
                .header("x-test-auth-role", "general_manager")
                .header(
                    "x-test-auth-location-id",
                    "00c0ffee-0000-0000-0000-000000000001",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let contract: DataQualityHygieneContextResponse = serde_json::from_value(wire.clone()).unwrap();

    assert_eq!(serde_json::to_value(contract).unwrap(), wire);
}
