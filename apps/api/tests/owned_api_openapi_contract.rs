use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::{http, public_contract};
use serde_json::{Value, json};
use tower::ServiceExt;

const OPENAPI: &str = include_str!("../openapi/owned-operations-v0.openapi.json");

async fn get_json(uri: &str) -> (axum_http::StatusCode, Value) {
    let response = http::router_with_state(http::VaccineDocumentState::default())
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::GET)
                .uri(uri)
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("owned v0 get request succeeds");

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload = serde_json::from_slice(&body).expect("json response payload");

    (status, payload)
}

async fn post_json(uri: &str, body: Value) -> (axum_http::StatusCode, Value) {
    let response = http::router_with_state(http::VaccineDocumentState::default())
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::POST)
                .uri(uri)
                .header(axum_http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .expect("request builds"),
        )
        .await
        .expect("owned v0 post request succeeds");

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload = serde_json::from_slice(&body).expect("json response payload");

    (status, payload)
}

#[test]
fn checked_openapi_artifact_names_owned_v0_operations_and_safe_schemas() {
    let spec: Value = serde_json::from_str(OPENAPI).expect("checked OpenAPI json parses");

    assert_eq!(spec["openapi"], "3.1.0");
    assert_eq!(
        spec["info"]["title"],
        "NVA Pet Resorts Owned Operations API"
    );
    assert_eq!(spec["info"]["version"], "0.1.0");

    let paths = spec["paths"].as_object().expect("paths object");
    for route in [
        "/v0/healthz",
        "/v0/readyz",
        "/v0/ops/metrics/summary",
        "/v0/agent/context/data-quality-hygiene",
        "/v0/agent/drafts/data-quality-hygiene",
        "/v0/data-quality-hygiene/actions/{action_id}/outcome",
        "/v0/data-quality-hygiene/outcomes/summary",
        "/v0/read-models/source-quality-backlog",
    ] {
        assert!(paths.contains_key(route), "missing v0 route {route}");
    }

    let schemas = spec["components"]["schemas"]
        .as_object()
        .expect("component schemas object");
    for schema_name in [
        "ApiContractMetadata",
        "RequestMetadata",
        "SourceRef",
        "ReviewGateRef",
        "BlockedAction",
        "AuditRef",
        "ErrorEnvelope",
        "DataQualityHygieneContextResponse",
        "DataQualityHygieneDraftSubmissionRequest",
        "DataQualityHygieneDraftSubmissionResponse",
        "DataQualityHygieneOutcomeCaptureRequest",
        "DataQualityHygieneOutcomeCaptureResponse",
        "DataQualityHygieneOutcomeSummaryResponse",
        "ReadinessResponse",
        "OpsMetricsSummaryResponse",
    ] {
        assert!(
            schemas.contains_key(schema_name),
            "missing schema {schema_name}"
        );
    }

    assert_eq!(
        schemas["ApiContractMetadata"]["properties"]["provider_payload_passthrough"]["const"],
        false
    );
    assert_eq!(
        schemas["ApiContractMetadata"]["properties"]["live_side_effects_allowed"]["const"],
        false
    );
    assert_eq!(
        schemas["ErrorEnvelope"]["properties"]["live_side_effects_allowed"]["const"],
        false
    );
}

#[test]
fn public_contract_dtos_serialize_owned_boundary_and_error_posture() {
    let metadata = public_contract::ApiContractMetadata::operations_v0("data-quality-hygiene");
    let error = public_contract::ErrorEnvelope::validation_failed(
        "req_contract_test".to_owned(),
        Some("data-quality-hygiene:test".to_owned()),
        vec![public_contract::ErrorDetail::field(
            "actions[0].requested_side_effects".to_owned(),
            "customer_send_requires_review_and_live_sends_are_disabled".to_owned(),
        )],
    );

    let metadata_json = serde_json::to_value(metadata).expect("metadata serializes");
    let error_json = serde_json::to_value(error).expect("error envelope serializes");

    assert_eq!(metadata_json["owner"], "nva_pet_resorts_operations");
    assert_eq!(metadata_json["boundary"], "owned_operations_api_v0");
    assert_eq!(metadata_json["provider_payload_passthrough"], false);
    assert_eq!(metadata_json["live_side_effects_allowed"], false);
    assert_eq!(error_json["error"]["safe_error_class"], "validation_failed");
    assert_eq!(error_json["live_side_effects_allowed"], false);
}

#[tokio::test]
async fn v0_routes_expose_safe_runtime_readiness_and_data_quality_context() {
    let (health_status, health) = get_json("/v0/healthz").await;
    assert_eq!(health_status, axum_http::StatusCode::OK);
    assert_eq!(health["live_side_effects"], "disabled");
    assert_eq!(
        health["api_contract"]["provider_payload_passthrough"],
        false
    );

    let (ready_status, ready) = get_json("/v0/readyz").await;
    assert_eq!(ready_status, axum_http::StatusCode::OK);
    assert_eq!(ready["workflow_repository"]["active_adapter"], "in_memory");
    assert_eq!(ready["live_customer_messaging"], "disabled");
    assert_eq!(ready["live_provider_writes"], "disabled");

    let (context_status, context) = get_json(
        "/v0/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
    )
    .await;
    assert_eq!(context_status, axum_http::StatusCode::OK);
    assert_eq!(context["workflow"]["name"], "data-quality-hygiene");
    assert_eq!(context["live_side_effects_allowed"], false);
    assert!(
        context["blocked_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("mutate_provider_or_pms_record"))
    );
    assert!(
        context["blocked_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("hide_or_auto_resolve_source_ambiguity"))
    );
}

#[tokio::test]
async fn v0_data_quality_draft_rejection_preserves_safe_error_shape() {
    let context = get_json(
        "/v0/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
    )
    .await
    .1;
    let action = context["hygiene_actions"][0].clone();
    let (status, payload) = post_json(
        "/v0/agent/drafts/data-quality-hygiene",
        json!({
            "context_packet_id": context["audit"]["context_packet_id"],
            "correlation_id": "data-quality-hygiene:test-v0-draft",
            "actions": [{
                "action_id": action["id"],
                "kind": action["kind"],
                "source_refs": action["source_refs"],
                "issue_refs": action["issue_refs"],
                "review_gates": action["review_gates"],
                "requested_side_effects": ["send_customer_message"],
                "attempted_ambiguity_resolution": true
            }]
        }),
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["live_side_effects_allowed"], false);
    assert_eq!(
        payload["validation"]["safe_error_class"],
        "validation_failed"
    );
    assert!(
        payload["rejected_actions"][0]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("blocked_side_effect_requested"))
    );
    assert!(
        payload["rejected_actions"][0]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("attempted_ambiguity_hiding"))
    );
}

#[tokio::test]
async fn v0_planned_read_model_route_returns_owned_planned_error_envelope() {
    let (status, payload) =
        get_json("/v0/read-models/source-quality-backlog?location_id=local").await;

    assert_eq!(status, axum_http::StatusCode::NOT_IMPLEMENTED);
    assert_eq!(payload["error"]["code"], "planned_not_wired");
    assert_eq!(payload["error"]["safe_error_class"], "planned_not_wired");
    assert_eq!(payload["live_side_effects_allowed"], false);

    let spec: Value = serde_json::from_str(OPENAPI).expect("checked OpenAPI json parses");
    let error_code_enum =
        spec["components"]["schemas"]["ErrorEnvelope"]["properties"]["error"]["properties"]["code"]
            ["enum"]
            .as_array()
            .expect("error code enum exists");
    assert!(error_code_enum.contains(&json!("planned_not_wired")));
}

#[tokio::test]
async fn v0_success_payloads_include_openapi_required_contract_fields() {
    let spec: Value = serde_json::from_str(OPENAPI).expect("checked OpenAPI json parses");
    let schemas = &spec["components"]["schemas"];

    let context = get_json(
        "/v0/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
    )
    .await
    .1;
    let action = context["hygiene_actions"][0].clone();

    let (draft_status, draft) = post_json(
        "/v0/agent/drafts/data-quality-hygiene",
        json!({
            "context_packet_id": context["audit"]["context_packet_id"],
            "correlation_id": "data-quality-hygiene:test-v0-contract-success",
            "actions": [{
                "action_id": action["id"],
                "kind": action["kind"],
                "source_refs": action["source_refs"],
                "issue_refs": action["issue_refs"],
                "review_gates": action["review_gates"],
                "requested_side_effects": [],
                "attempted_ambiguity_resolution": false
            }]
        }),
    )
    .await;
    assert_eq!(draft_status, axum_http::StatusCode::CREATED);
    assert_required_fields_present(
        &schemas["DataQualityHygieneDraftSubmissionResponse"],
        &draft,
    );

    let (summary_status, summary) = get_json(
        "/v0/data-quality-hygiene/outcomes/summary?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
    )
    .await;
    assert_eq!(summary_status, axum_http::StatusCode::OK);
    assert_required_fields_present(
        &schemas["DataQualityHygieneOutcomeSummaryResponse"],
        &summary,
    );

    let outcome_uri = format!(
        "/v0/data-quality-hygiene/actions/{}/outcome",
        action["id"].as_str().expect("action id is a string")
    );
    let (outcome_status, outcome) = post_json(
        &outcome_uri,
        json!({
            "outcome": "completed",
            "actual_minutes": 10,
            "actor": { "id": "front-desk:test", "persona": "front_desk_agent" },
            "feedback": "Resolved duplicate aliases after manager review.",
            "source_refs": action["source_refs"],
            "issue_refs": action["issue_refs"],
            "resolution_status_after_review": "repaired",
            "timestamp": "2026-06-17T16:00:00Z",
            "audit": { "correlation_id": "data-quality-hygiene:test-v0-outcome" },
            "requested_side_effects": []
        }),
    )
    .await;
    assert_eq!(outcome_status, axum_http::StatusCode::CREATED);
    assert_required_fields_present(
        &schemas["DataQualityHygieneOutcomeCaptureResponse"],
        &outcome,
    );
}

fn assert_required_fields_present(schema: &Value, payload: &Value) {
    for required in schema["required"]
        .as_array()
        .expect("schema has required array")
    {
        let field = required.as_str().expect("required field is string");
        assert!(
            payload.get(field).is_some(),
            "payload is missing required OpenAPI field {field}: {payload}"
        );
    }
}
