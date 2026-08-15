use axum::body::Body;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn response_json(response: axum::response::Response) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body is collectable")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("response body is json")
}

#[tokio::test]
async fn manager_daily_brief_outcome_capture_requires_source_refs_before_storage_persistence() {
    let app = pet_resort_api::http::router_with_test_auth_state(Default::default());
    let location_id = "00c0ffee-0000-0000-0000-000000000001";
    let operating_day = "2026-06-17";
    let context_response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri(format!(
                    "/agent/context/manager-daily-brief?location_id={location_id}&operating_day={operating_day}"
                ))
                .header("x-test-auth-actor-id", "general-manager-contract")
                .header("x-test-auth-role", "general_manager")
                .header("x-test-auth-location-id", location_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let context = response_json(context_response).await;
    let action_id = context["manager_brief_actions"][0]["id"]
        .as_str()
        .expect("fixture exposes a manager daily-brief action");

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/manager-daily-brief/actions/{action_id}/outcome"
                ))
                .header("content-type", "application/json")
                .header("x-test-auth-actor-id", "general-manager-contract")
                .header("x-test-auth-role", "general_manager")
                .header("x-test-auth-location-id", location_id)
                .body(Body::from(
                    json!({
                        "outcome": "completed",
                        "actual_minutes": 4,
                        "actor": {"id": "general-manager-contract", "persona": "general_manager"},
                        "feedback": "Reviewed checkout exception and saved front-desk open-stay audit time.",
                        "source_refs": [],
                        "timestamp": "2026-06-17T12:00:00Z",
                        "audit": {"correlation_id": "manager-daily-brief:test"},
                        "reporting": {"location_id": location_id, "operating_day": operating_day},
                        "requested_side_effects": [],
                        "idempotency_key": "manager-daily-brief:test:missing-source-refs"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::UNPROCESSABLE_ENTITY
    );
    let payload = response_json(response).await;
    assert_eq!(payload["outcome_persisted"], false);
    assert_eq!(payload["reasons"], json!(["missing_source_refs"]));
    assert_eq!(payload["live_side_effects_allowed"], false);
}

#[tokio::test]
async fn data_quality_hygiene_outcome_capture_requires_issue_refs_before_storage_persistence() {
    let app = pet_resort_api::http::router_with_test_auth_state(Default::default());
    let context_response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17")
                .header("x-test-auth-actor-id", "general-manager-contract")
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
    let context = response_json(context_response).await;
    let action_id = context["hygiene_actions"][0]["id"]
        .as_str()
        .expect("fixture exposes a data-quality hygiene action");
    let source_ref = context["hygiene_actions"][0]["source_refs"][0].clone();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/data-quality-hygiene/actions/{action_id}/outcome"))
                .header("content-type", "application/json")
                .header("x-test-auth-actor-id", "front-desk-lead-17")
                .header("x-test-auth-role", "front_desk_lead")
                .header(
                    "x-test-auth-location-id",
                    "00c0ffee-0000-0000-0000-000000000001",
                )
                .body(Body::from(
                    json!({
                        "outcome": "completed",
                        "actual_minutes": 6,
                        "actor": {"id": "front-desk-lead-17", "persona": "front_desk_lead", "actor_role": "front_desk_lead"},
                        "feedback": "Reviewed stale vaccination evidence without hiding ambiguity.",
                        "source_refs": [source_ref],
                        "issue_refs": [],
                        "resolution_status_after_review": "acknowledged",
                        "timestamp": "2026-06-17T12:15:00Z",
                        "audit": {"correlation_id": "data-quality-hygiene:test"},
                        "requested_side_effects": [],
                        "idempotency_key": "outcome-capture-validation-1"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::UNPROCESSABLE_ENTITY
    );
    let payload = response_json(response).await;
    assert_eq!(payload["outcome_persisted"], false);
    assert_eq!(
        payload["reasons"],
        json!(["missing_data_quality_issue_refs"])
    );
    assert_eq!(payload["live_side_effects_allowed"], false);
}

#[tokio::test]
async fn data_quality_hygiene_outcome_capture_requires_source_refs_before_storage_persistence() {
    let app = pet_resort_api::http::router_with_test_auth_state(Default::default());
    let context_response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17")
                .header("x-test-auth-actor-id", "general-manager-contract")
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
    let context = response_json(context_response).await;
    let action_id = context["hygiene_actions"][0]["id"]
        .as_str()
        .expect("fixture exposes a data-quality hygiene action");

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/data-quality-hygiene/actions/{action_id}/outcome"))
                .header("content-type", "application/json")
                .header("x-test-auth-actor-id", "front-desk-lead-17")
                .header("x-test-auth-role", "front_desk_lead")
                .header(
                    "x-test-auth-location-id",
                    "00c0ffee-0000-0000-0000-000000000001",
                )
                .body(Body::from(
                    json!({
                        "outcome": "completed",
                        "actual_minutes": 6,
                        "actor": {"id": "front-desk-lead-17", "persona": "front_desk_lead", "actor_role": "front_desk_lead"},
                        "feedback": "Reviewed stale vaccination evidence without hiding ambiguity.",
                        "source_refs": [],
                        "issue_refs": ["dq-vaccine-stale-42"],
                        "resolution_status_after_review": "acknowledged",
                        "timestamp": "2026-06-17T12:15:00Z",
                        "audit": {"correlation_id": "data-quality-hygiene:test"},
                        "requested_side_effects": [],
                        "idempotency_key": "outcome-capture-validation-1"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::UNPROCESSABLE_ENTITY
    );
    let payload = response_json(response).await;
    assert_eq!(payload["outcome_persisted"], false);
    assert_eq!(payload["reasons"], json!(["missing_source_refs"]));
    assert_eq!(payload["live_side_effects_allowed"], false);
}

#[tokio::test]
async fn data_quality_hygiene_outcome_capture_rejects_unbound_or_malformed_provenance() {
    let app = pet_resort_api::http::router_with_test_auth_state(Default::default());
    let location_id = "00c0ffee-0000-0000-0000-000000000001";
    let context_response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri(format!(
                    "/agent/context/data-quality-hygiene?location_id={location_id}&operating_day=2026-06-17"
                ))
                .header("x-test-auth-actor-id", "general-manager-contract")
                .header("x-test-auth-role", "general_manager")
                .header("x-test-auth-location-id", location_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let context = response_json(context_response).await;
    let action = &context["hygiene_actions"][0];
    let action_id = action["id"].as_str().unwrap();
    let base = json!({
        "outcome": "completed",
        "actual_minutes": 6,
        "actor": {"id": "front-desk-lead-17", "persona": "front_desk_lead", "actor_role": "front_desk_lead"},
        "feedback": "Reviewed only the evidence bound to this action.",
        "source_refs": [{
            "system": "unrelated",
            "record_type": "invented_record",
            "record_id": "invented-1",
            "observed_at": "2026-06-17T00:00:00Z",
            "adapter_version": "invented-adapter-v1"
        }],
        "issue_refs": ["invented-issue"],
        "resolution_status_after_review": "acknowledged",
        "timestamp": "2026-06-17T12:15:00Z",
        "audit": {"correlation_id": "data-quality-hygiene:provenance-test"},
        "requested_side_effects": [],
        "idempotency_key": "outcome-provenance-validation-1"
    });

    let mismatched = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/data-quality-hygiene/actions/{action_id}/outcome"))
                .header("content-type", "application/json")
                .header("x-test-auth-actor-id", "front-desk-lead-17")
                .header("x-test-auth-role", "front_desk_lead")
                .header("x-test-auth-location-id", location_id)
                .body(Body::from(base.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        mismatched.status(),
        axum::http::StatusCode::UNPROCESSABLE_ENTITY
    );
    let payload = response_json(mismatched).await;
    assert_eq!(payload["outcome_persisted"], false);
    assert_eq!(
        payload["reasons"],
        json!([
            "issue_refs_do_not_match_action",
            "source_refs_do_not_match_action"
        ])
    );

    let mut malformed_body = base;
    malformed_body["source_refs"] = json!([{"record_id": "missing-system"}]);
    let malformed = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/data-quality-hygiene/actions/{action_id}/outcome"))
                .header("content-type", "application/json")
                .header("x-test-auth-actor-id", "front-desk-lead-17")
                .header("x-test-auth-role", "front_desk_lead")
                .header("x-test-auth-location-id", location_id)
                .body(Body::from(malformed_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        malformed.status(),
        axum::http::StatusCode::UNPROCESSABLE_ENTITY
    );
}
