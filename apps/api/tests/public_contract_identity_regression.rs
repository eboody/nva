use axum::body::Body;
use http_body_util::BodyExt;
use pet_resort_api::{
    http,
    public_contract::{
        ApiContractMetadata, DataQualityHygieneContextResponse,
        DataQualityHygieneOutcomeCaptureRequest, LiveSideEffectsMode,
        ManagerDailyBriefOutcomeCaptureRequest, ProviderBoundaryMode, SourceRecordRef,
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
    let request = serde_json::from_value::<DataQualityHygieneOutcomeCaptureRequest>(json!({
        "outcome": "completed",
        "actual_minutes": 7,
        "actor": {
            "persona": "front_desk_lead",
            "id": "sensitive-actor-17",
            "actor_role": "front_desk_lead"
        },
        "feedback": "sensitive customer context",
        "source_refs": [{
            "system": "gingr",
            "record_type": "customer",
            "record_id": "sensitive-record-42",
            "observed_at": "2026-06-17T00:00:00Z",
            "adapter_version": "gingr-v0-readonly"
        }],
        "issue_refs": ["sensitive-issue-9"],
        "resolution_status_after_review": "acknowledged",
        "timestamp": "2026-06-17T12:00:00Z",
        "audit": {"correlation_id": "sensitive-correlation"},
        "requested_side_effects": [],
        "idempotency_key": "sensitive-idempotency-key"
    }))
    .unwrap();

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

    let manager_request = serde_json::from_value::<ManagerDailyBriefOutcomeCaptureRequest>(json!({
        "outcome": "completed",
        "actual_minutes": 12,
        "actor": {"persona": "front_desk_lead", "id": "sensitive-manager-17"},
        "feedback": "sensitive manager feedback",
        "source_refs": [{
            "system": "gingr",
            "record_type": "reservation",
            "record_id": "sensitive-reservation-42",
            "observed_at": "2026-06-17T12:00:00Z",
            "adapter_version": "sensitive-adapter-version"
        }],
        "timestamp": "2026-06-17T13:15:00Z",
        "audit": {"correlation_id": "sensitive-manager-correlation"},
        "reporting": {
            "location_id": "11111111-1111-4111-8111-111111111111",
            "operating_day": "2026-06-17"
        },
        "requested_side_effects": [],
        "idempotency_key": "sensitive-manager-idempotency"
    }))
    .unwrap();
    let manager_debug = format!("{manager_request:?}");
    for sensitive in [
        "sensitive-manager-17",
        "sensitive manager feedback",
        "sensitive-reservation-42",
        "sensitive-adapter-version",
        "sensitive-manager-correlation",
        "11111111-1111-4111-8111-111111111111",
        "sensitive-manager-idempotency",
    ] {
        assert!(
            !manager_debug.contains(sensitive),
            "manager Debug leaked {sensitive}"
        );
    }
    assert!(manager_debug.contains("[REDACTED]"));
}

#[test]
fn outcome_dtos_reject_zero_minutes_nil_locations_and_malformed_operating_days_during_deserialization()
 {
    let mut manager = json!({
        "outcome": "completed",
        "actual_minutes": 9,
        "actor": {"persona": "general_manager", "id": "manager-1"},
        "feedback": "reported evidence",
        "source_refs": [],
        "timestamp": "2026-08-15T00:00:00Z",
        "audit": {"correlation_id": "corr-1"},
        "reporting": {
            "location_id": "11111111-1111-4111-8111-111111111111",
            "operating_day": "2026-08-15"
        },
        "requested_side_effects": [],
        "idempotency_key": "idem-1"
    });
    manager["actual_minutes"] = json!(0);
    assert!(
        serde_json::from_value::<ManagerDailyBriefOutcomeCaptureRequest>(manager.clone()).is_err()
    );
    manager["actual_minutes"] = json!(9);
    manager["reporting"]["location_id"] = json!("00000000-0000-0000-0000-000000000000");
    assert!(
        serde_json::from_value::<ManagerDailyBriefOutcomeCaptureRequest>(manager.clone()).is_err()
    );
    manager["reporting"]["location_id"] = json!("11111111-1111-4111-8111-111111111111");
    manager["reporting"]["operating_day"] = json!("2026-02-30");
    assert!(serde_json::from_value::<ManagerDailyBriefOutcomeCaptureRequest>(manager).is_err());

    let hygiene = json!({
        "outcome": "completed",
        "actual_minutes": 0,
        "actor": {"persona": "operations_analyst", "id": "analyst-1"},
        "feedback": "reported evidence",
        "source_refs": [],
        "issue_refs": [],
        "resolution_status_after_review": "repaired",
        "timestamp": "2026-08-15T00:00:00Z",
        "audit": {"correlation_id": "corr-2"},
        "requested_side_effects": [],
        "idempotency_key": "idem-2"
    });
    assert!(serde_json::from_value::<DataQualityHygieneOutcomeCaptureRequest>(hygiene).is_err());
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
