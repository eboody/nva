use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::http;
use serde_json::json;
use tower::ServiceExt;

async fn get_json(uri: &str) -> (axum_http::StatusCode, serde_json::Value) {
    let response = http::router()
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::GET)
                .uri(uri)
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("data-quality hygiene context request succeeds");

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload = serde_json::from_slice(&body).expect("json context payload");

    (status, payload)
}

async fn post_json(
    uri: &str,
    body: serde_json::Value,
) -> (axum_http::StatusCode, serde_json::Value) {
    let response = http::router()
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::POST)
                .uri(uri)
                .header(axum_http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .expect("request builds"),
        )
        .await
        .expect("data-quality hygiene post request succeeds");

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

async fn data_quality_context() -> serde_json::Value {
    let (status, payload) = get_json(
        "/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
    )
    .await;
    assert_eq!(status, axum_http::StatusCode::OK);
    payload
}

#[tokio::test]
async fn data_quality_hygiene_context_returns_source_grounded_internal_cleanup_packet() {
    let payload = data_quality_context().await;

    assert_eq!(payload["workflow"]["name"], "data-quality-hygiene");
    assert_eq!(
        payload["workflow"]["version"],
        "data-quality-hygiene-context-v1"
    );
    assert_eq!(
        payload["location_id"],
        "00c0ffee-0000-0000-0000-000000000001"
    );
    assert_eq!(payload["operating_day"], "2026-06-17");
    assert_eq!(payload["prepared_for"], "general_manager");
    assert!(payload["candidates"].as_array().unwrap().len() >= 2);
    assert!(payload["hygiene_actions"].as_array().unwrap().len() >= 2);
    assert_eq!(payload["labor_savings_estimate"]["before_minutes"], 55);
    assert_eq!(payload["labor_savings_estimate"]["after_minutes"], 22);
    assert_eq!(
        payload["labor_savings_estimate"]["estimated_minutes_saved"],
        33
    );

    assert!(
        payload["allowed_agent_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("draft_internal_cleanup_task"))
    );
    assert!(
        payload["allowed_agent_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("preserve_ambiguity_for_review"))
    );
    assert!(
        payload["blocked_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("mutate_provider_or_pms_record"))
    );
    assert!(
        payload["blocked_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("hide_or_auto_resolve_source_ambiguity"))
    );
    assert!(
        payload["audit"]["context_packet_id"]
            .as_str()
            .unwrap()
            .starts_with("data-quality-hygiene-context:")
    );
    assert!(
        payload["live_side_effects_allowed"]
            .as_bool()
            .is_some_and(|allowed| !allowed)
    );
}

#[tokio::test]
async fn data_quality_hygiene_drafts_reject_blocked_side_effects_and_ambiguity_hiding() {
    let context = data_quality_context().await;
    let action = &context["hygiene_actions"][0];
    let body = json!({
        "context_packet_id": context["audit"]["context_packet_id"],
        "correlation_id": context["audit"]["correlation_id"],
        "actions": [
            {
                "action_id": action["id"],
                "kind": action["kind"],
                "source_refs": action["source_refs"],
                "issue_refs": action["issue_refs"],
                "review_gates": action["review_gates"],
                "requested_side_effects": ["send_customer_message"],
                "attempted_ambiguity_resolution": true
            }
        ]
    });

    let (status, payload) = post_json("/agent/drafts/data-quality-hygiene", body).await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["validation"]["status"], "rejected");
    assert_eq!(payload["accepted_actions"].as_array().unwrap().len(), 0);
    assert_eq!(payload["live_side_effects_allowed"], false);
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
async fn data_quality_hygiene_outcome_capture_records_labor_evidence_without_provider_writes() {
    let context = data_quality_context().await;
    let action = &context["hygiene_actions"][0];
    let action_id = action["id"].as_str().unwrap();
    let body = json!({
        "outcome": "completed",
        "actual_minutes": 9,
        "actor": {
            "id": "front-desk-lead-17",
            "persona": "front_desk_lead"
        },
        "feedback": "Prepared source-grounded cleanup task for manager review without touching Gingr.",
        "source_refs": action["source_refs"],
        "issue_refs": action["issue_refs"],
        "resolution_status_after_review": "acknowledged",
        "timestamp": "2026-06-17T13:15:00Z",
        "audit": {
            "correlation_id": context["audit"]["correlation_id"]
        },
        "requested_side_effects": []
    });

    let (status, payload) = post_json(
        &format!("/data-quality-hygiene/actions/{action_id}/outcome"),
        body,
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::CREATED);
    assert_eq!(payload["outcome_record"]["action_id"], action_id);
    assert_eq!(payload["outcome_record"]["outcome"], "completed");
    assert_eq!(payload["outcome_record"]["actual_minutes"], 9);
    assert_eq!(
        payload["outcome_record"]["resolution_status_after_review"],
        "acknowledged"
    );
    assert_eq!(payload["live_side_effects_allowed"], false);
    assert!(
        payload["blocked_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("mutate_provider_or_pms_record"))
    );
    assert!(
        payload["labor_savings_evidence"]["actual_minutes_saved"]
            .as_u64()
            .unwrap()
            > 0
    );
}
