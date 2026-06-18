use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::http;
use serde_json::json;
use tower::ServiceExt;

async fn post_outcome(
    action_id: &str,
    body: serde_json::Value,
) -> (axum_http::StatusCode, serde_json::Value) {
    let response = http::router()
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::POST)
                .uri(format!("/manager-daily-brief/actions/{action_id}/outcome"))
                .header(axum_http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .expect("request builds"),
        )
        .await
        .expect("outcome request succeeds");

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload = serde_json::from_slice(&body).expect("json outcome response payload");

    (status, payload)
}

async fn get_manager_daily_brief_context() -> serde_json::Value {
    let response = http::router()
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::GET)
                .uri(
                    "/agent/context/manager-daily-brief?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
                )
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("context request succeeds");

    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();

    serde_json::from_slice(&body).expect("json context payload")
}

async fn manager_daily_brief_action_by_kind(kind: &str) -> serde_json::Value {
    let context = get_manager_daily_brief_context().await;
    context["manager_brief_actions"]
        .as_array()
        .expect("manager brief actions array")
        .iter()
        .find(|action| action["kind"] == kind)
        .expect("manager brief action exists")
        .clone()
}

fn source_ref() -> serde_json::Value {
    json!({
        "system": "gingr",
        "record_type": "reservation",
        "record_id": "reservation-4242",
        "observed_at": "2026-06-17T12:00:00Z",
        "adapter_version": "local-manager-daily-brief-outcome-fixture-v1"
    })
}

fn outcome_body() -> serde_json::Value {
    json!({
        "outcome": "completed",
        "actual_minutes": 12,
        "actor": {
            "id": "front-desk-lead-17",
            "persona": "front_desk_lead"
        },
        "feedback": "Resolved before checkout rush; brief saved a manual open-stay audit.",
        "source_refs": [source_ref()],
        "timestamp": "2026-06-17T13:15:00Z",
        "audit": {
            "correlation_id": "manager-daily-brief:00c0ffee-0000-0000-0000-000000000001:2026-06-17"
        },
        "reporting": {
            "location_id": "00c0ffee-0000-0000-0000-000000000001",
            "operating_day": "2026-06-17",
            "action_kind": "approve_retention_follow_up_draft",
            "owner_persona": "general_manager",
            "before_minutes": 999,
            "estimated_minutes_saved": 999
        },
        "requested_side_effects": []
    })
}

#[tokio::test]
async fn manager_daily_brief_outcome_capture_persists_staff_feedback_as_labor_savings_evidence() {
    let action = manager_daily_brief_action_by_kind("resolve_checkout_exception").await;
    let action_id = action["id"].as_str().expect("action id");

    let (status, payload) = post_outcome(action_id, outcome_body()).await;

    assert_eq!(status, axum_http::StatusCode::CREATED);
    assert_eq!(payload["outcome_record"]["action_id"], action_id);
    assert_eq!(payload["outcome_record"]["outcome"], "completed");
    assert_eq!(payload["outcome_record"]["before_minutes"], 20);
    assert_eq!(payload["outcome_record"]["actual_minutes"], 12);
    assert_eq!(
        payload["outcome_record"]["actor"]["persona"],
        "front_desk_lead"
    );
    assert_eq!(
        payload["outcome_record"]["audit"]["correlation_id"],
        "manager-daily-brief:00c0ffee-0000-0000-0000-000000000001:2026-06-17"
    );

    assert_eq!(
        payload["labor_savings_evidence"]["estimated_minutes_saved"],
        action["labor_impact"]["minutes_saved"]
    );
    assert_eq!(payload["labor_savings_evidence"]["actual_minutes_saved"], 8);
    assert_eq!(
        payload["labor_savings_evidence"]["grouping"]["location_id"],
        "00c0ffee-0000-0000-0000-000000000001"
    );
    assert_eq!(
        payload["labor_savings_evidence"]["grouping"]["operating_day"],
        "2026-06-17"
    );
    assert_eq!(
        payload["labor_savings_evidence"]["grouping"]["action_kind"],
        "resolve_checkout_exception"
    );
    assert_eq!(
        payload["labor_savings_evidence"]["grouping"]["owner_persona"],
        "front_desk_lead"
    );
    assert_eq!(payload["live_side_effects_allowed"], false);
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
            .contains(&json!("send_customer_message"))
    );
}

#[tokio::test]
async fn manager_daily_brief_outcome_capture_rejects_attempted_live_side_effects() {
    for blocked_side_effect in [
        "send_customer_message",
        "mutate_provider_or_pms_record",
        "change_staff_schedule",
        "move_refund_discount_or_payment",
        "hide_source_data_quality_issue",
    ] {
        let mut body = outcome_body();
        body["requested_side_effects"] = json!([blocked_side_effect]);

        let (status, payload) = post_outcome("checkout-exception-reservation-4242", body).await;

        assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(payload["accepted"], false);
        assert_eq!(payload["outcome_persisted"], false);
        assert_eq!(
            payload["reasons"],
            json!([format!("blocked_side_effect:{blocked_side_effect}")])
        );
    }
}

#[tokio::test]
async fn manager_daily_brief_outcome_capture_rejects_unknown_side_effects_fail_closed() {
    let mut body = outcome_body();
    body["requested_side_effects"] = json!(["invent_new_live_side_effect"]);

    let (status, payload) = post_outcome("checkout-exception-reservation-4242", body).await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["accepted"], false);
    assert_eq!(payload["outcome_persisted"], false);
    assert_eq!(
        payload["reasons"],
        json!(["unsupported_side_effect:invent_new_live_side_effect"])
    );
}

#[tokio::test]
async fn manager_daily_brief_outcome_capture_rejects_zero_actual_minutes() {
    let action = manager_daily_brief_action_by_kind("resolve_checkout_exception").await;
    let action_id = action["id"].as_str().expect("action id");
    let mut body = outcome_body();
    body["actual_minutes"] = json!(0);

    let (status, payload) = post_outcome(action_id, body).await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["accepted"], false);
    assert_eq!(payload["outcome_persisted"], false);
    assert_eq!(
        payload["reasons"],
        json!(["actual_minutes_must_be_greater_than_zero"])
    );
}

#[tokio::test]
async fn manager_daily_brief_outcome_capture_rejects_unknown_action_ids() {
    let (status, payload) = post_outcome("fabricated-action-id", outcome_body()).await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["accepted"], false);
    assert_eq!(payload["outcome_persisted"], false);
    assert_eq!(
        payload["reasons"],
        json!(["unknown_manager_daily_brief_action_id"])
    );
}
