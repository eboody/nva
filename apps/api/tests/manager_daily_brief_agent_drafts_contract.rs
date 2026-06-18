use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::http;
use serde_json::json;
use tower::ServiceExt;

async fn post_json(body: serde_json::Value) -> (axum_http::StatusCode, serde_json::Value) {
    let response = http::router()
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::POST)
                .uri("/agent/drafts/manager-daily-brief")
                .header(axum_http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .expect("request builds"),
        )
        .await
        .expect("agent draft request succeeds");

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload = serde_json::from_slice(&body).expect("json draft response payload");

    (status, payload)
}

fn valid_source_ref() -> serde_json::Value {
    json!({
        "system": "gingr",
        "record_type": "service_demand_forecast",
        "record_id": "demand:00c0ffee-0000-0000-0000-000000000001:2026-06-17",
        "observed_at": "2026-06-17T12:00:00Z",
        "adapter_version": "local-manager-daily-brief-fixture-v1"
    })
}

fn accepted_demand_draft_body() -> serde_json::Value {
    json!({
        "context_packet_id": "manager-daily-brief-context:00c0ffee-0000-0000-0000-000000000001:2026-06-17",
        "correlation_id": "manager-daily-brief:00c0ffee-0000-0000-0000-000000000001:2026-06-17",
        "submitted_by": "hermes-agent",
        "actions": [
            {
                "id": "draft-demand-staffing-1",
                "kind": "review_demand_against_staffing_plan",
                "recommendation": "Review demand against the staffing plan before morning drop-off.",
                "source_refs": [valid_source_ref()],
                "review_gates": ["manager_approval"],
                "requested_side_effects": []
            }
        ]
    })
}

#[tokio::test]
async fn manager_daily_brief_agent_drafts_accepts_source_grounded_review_gated_recommendations() {
    let (status, payload) = post_json(accepted_demand_draft_body()).await;

    assert_eq!(status, axum_http::StatusCode::CREATED);
    assert_eq!(payload["validation"]["status"], "accepted");
    assert_eq!(payload["accepted_actions"].as_array().unwrap().len(), 1);
    assert_eq!(
        payload["accepted_actions"][0]["kind"],
        "review_demand_against_staffing_plan"
    );
    assert_eq!(payload["accepted_actions"][0]["showable_to_manager"], true);
    assert_eq!(payload["rejected_actions"].as_array().unwrap().len(), 0);
    assert_eq!(payload["live_side_effects_allowed"], false);
}

#[tokio::test]
async fn manager_daily_brief_agent_drafts_rejects_unsupported_action_kinds() {
    let mut body = accepted_demand_draft_body();
    body["actions"][0]["kind"] = json!("autonomously_rewrite_schedule");

    let (status, payload) = post_json(body).await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["validation"]["status"], "rejected");
    assert_eq!(payload["accepted_actions"].as_array().unwrap().len(), 0);
    assert_eq!(
        payload["rejected_actions"][0]["reasons"],
        json!(["unsupported_action_kind"])
    );
}

#[tokio::test]
async fn manager_daily_brief_agent_drafts_rejects_actions_without_source_refs() {
    let mut body = accepted_demand_draft_body();
    body["actions"][0]["source_refs"] = json!([]);

    let (status, payload) = post_json(body).await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["accepted_actions"].as_array().unwrap().len(), 0);
    assert!(
        payload["rejected_actions"][0]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("missing_source_refs"))
    );
}

#[tokio::test]
async fn manager_daily_brief_agent_drafts_rejects_missing_or_wrong_review_gates() {
    let mut missing_gate_body = accepted_demand_draft_body();
    missing_gate_body["actions"][0]["review_gates"] = json!([]);

    let (missing_status, missing_payload) = post_json(missing_gate_body).await;
    assert_eq!(missing_status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        missing_payload["rejected_actions"][0]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("missing_required_review_gate:manager_approval"))
    );

    let mut wrong_gate_body = accepted_demand_draft_body();
    wrong_gate_body["actions"][0]["review_gates"] = json!(["customer_message_approval"]);

    let (wrong_status, wrong_payload) = post_json(wrong_gate_body).await;
    assert_eq!(wrong_status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        wrong_payload["rejected_actions"][0]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("missing_required_review_gate:manager_approval"))
    );
}

#[tokio::test]
async fn manager_daily_brief_agent_drafts_rejects_live_side_effect_attempts() {
    for blocked_side_effect in [
        "send_customer_message",
        "mutate_provider_or_pms_record",
        "change_staff_schedule",
        "move_refund_discount_or_payment",
    ] {
        let mut body = accepted_demand_draft_body();
        body["actions"][0]["requested_side_effects"] = json!([blocked_side_effect]);

        let (status, payload) = post_json(body).await;

        assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(payload["accepted_actions"].as_array().unwrap().len(), 0);
        assert!(
            payload["rejected_actions"][0]["reasons"]
                .as_array()
                .unwrap()
                .contains(&json!(format!("blocked_side_effect:{blocked_side_effect}")))
        );
    }
}

#[tokio::test]
async fn manager_daily_brief_agent_drafts_rejects_unknown_requested_side_effects_fail_closed() {
    let mut body = accepted_demand_draft_body();
    body["actions"][0]["requested_side_effects"] = json!(["invent_new_live_side_effect"]);

    let (status, payload) = post_json(body).await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["validation"]["status"], "rejected");
    assert_eq!(payload["accepted_actions"].as_array().unwrap().len(), 0);
    assert_eq!(payload["rejected_actions"][0]["showable_to_manager"], false);
    assert_eq!(
        payload["rejected_actions"][0]["live_side_effects_allowed"],
        false
    );
    assert_eq!(
        payload["rejected_actions"][0]["reasons"],
        json!(["unsupported_side_effect:invent_new_live_side_effect"])
    );
}

#[tokio::test]
async fn manager_daily_brief_agent_drafts_preserves_only_allowed_recommendations() {
    let body = json!({
        "context_packet_id": "manager-daily-brief-context:00c0ffee-0000-0000-0000-000000000001:2026-06-17",
        "correlation_id": "manager-daily-brief:00c0ffee-0000-0000-0000-000000000001:2026-06-17",
        "submitted_by": "hermes-agent",
        "actions": [
            {
                "id": "retention-draft-1",
                "kind": "approve_retention_follow_up_draft",
                "recommendation": "Review this draft-only retention follow-up for approval.",
                "source_refs": [valid_source_ref()],
                "review_gates": ["customer_message_approval"],
                "requested_side_effects": []
            },
            {
                "id": "schedule-change-1",
                "kind": "autonomously_rewrite_schedule",
                "recommendation": "Move two staff members without manager approval.",
                "source_refs": [valid_source_ref()],
                "review_gates": ["manager_approval"],
                "requested_side_effects": ["change_staff_schedule"]
            }
        ]
    });

    let (status, payload) = post_json(body).await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["validation"]["status"], "partially_accepted");
    assert_eq!(payload["accepted_actions"].as_array().unwrap().len(), 1);
    assert_eq!(
        payload["accepted_actions"][0]["kind"],
        "approve_retention_follow_up_draft"
    );
    assert_eq!(payload["rejected_actions"].as_array().unwrap().len(), 1);
}
