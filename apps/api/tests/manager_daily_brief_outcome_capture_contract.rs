use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::http;
use serde_json::json;
use tower::ServiceExt;

async fn post_outcome(
    action_id: &str,
    body: serde_json::Value,
) -> (axum_http::StatusCode, serde_json::Value) {
    let actor_id = body["actor"]["id"].as_str().map(str::to_owned);
    let mut builder = axum_http::request::Builder::new()
        .method(axum_http::Method::POST)
        .uri(format!(
            "/v0/manager-daily-brief/actions/{action_id}/outcome"
        ))
        .header(axum_http::header::CONTENT_TYPE, "application/json");
    if let Some(actor_id) = actor_id {
        builder = builder
            .header("x-test-auth-actor-id", actor_id)
            .header("x-test-auth-role", "front_desk_lead")
            .header(
                "x-test-auth-location-id",
                "00c0ffee-0000-0000-0000-000000000001",
            );
    }

    let response = http::router_with_test_auth_state(http::VaccineDocumentState::default())
        .oneshot(
            builder
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
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("json outcome response payload");
    if status == axum_http::StatusCode::UNPROCESSABLE_ENTITY {
        assert!(payload["error"]["code"].is_string());
        assert!(payload["error"]["message"].is_string());
        assert!(payload["error"]["safe_error_class"].is_string());
        assert!(payload["error"]["details"].is_array());
        assert!(payload["request_id"].is_string());
        assert!(payload.get("correlation_id").is_some());
        assert_eq!(payload["live_side_effects"], "disabled");
    }

    (status, payload)
}

async fn get_manager_daily_brief_context() -> serde_json::Value {
    let response = http::router_with_test_auth_state(http::VaccineDocumentState::default())
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::GET)
                .uri(
                    "/agent/context/manager-daily-brief?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
                )
                .header("x-test-auth-actor-id", "general-manager-17")
                .header("x-test-auth-role", "general_manager")
                .header(
                    "x-test-auth-location-id",
                    "00c0ffee-0000-0000-0000-000000000001",
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
            "operating_day": "2026-06-17"
        },
        "requested_side_effects": [],
        "idempotency_key": "manager-daily-brief-outcome-1"
    })
}

async fn post_outcome_with_state(
    state: http::VaccineDocumentState,
    action_id: &str,
    body: serde_json::Value,
) -> (axum_http::StatusCode, serde_json::Value) {
    let actor_id = body["actor"]["id"].as_str().expect("actor id");
    let response = http::router_with_test_auth_state(state)
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::POST)
                .uri(format!(
                    "/v0/manager-daily-brief/actions/{action_id}/outcome"
                ))
                .header(axum_http::header::CONTENT_TYPE, "application/json")
                .header("x-test-auth-actor-id", actor_id)
                .header("x-test-auth-role", "front_desk_lead")
                .header(
                    "x-test-auth-location-id",
                    "00c0ffee-0000-0000-0000-000000000001",
                )
                .body(Body::from(body.to_string()))
                .expect("request builds"),
        )
        .await
        .expect("outcome request succeeds");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).expect("json response"),
    )
}

#[tokio::test]
async fn manager_daily_brief_outcome_replay_reuses_the_atomic_record() {
    let state = http::VaccineDocumentState::default();
    let action = manager_daily_brief_action_by_kind("resolve_checkout_exception").await;
    let action_id = action["id"].as_str().expect("action id");
    let mut body = outcome_body();
    body["source_refs"] = action["source_refs"].clone();

    let first = post_outcome_with_state(state.clone(), action_id, body.clone()).await;
    let replay = post_outcome_with_state(state, action_id, body).await;

    assert_eq!(first.0, axum_http::StatusCode::CREATED);
    assert_eq!(replay.0, axum_http::StatusCode::OK);
    assert_eq!(replay.1["idempotent_replay"], true);
    assert_eq!(
        replay.1["reported_labor_evidence"]["persisted_outcome_count"],
        1
    );
}

#[tokio::test]
async fn manager_daily_brief_outcome_key_reuse_with_payload_drift_conflicts() {
    let state = http::VaccineDocumentState::default();
    let action = manager_daily_brief_action_by_kind("resolve_checkout_exception").await;
    let action_id = action["id"].as_str().expect("action id");
    let mut body = outcome_body();
    body["source_refs"] = action["source_refs"].clone();
    let mut drift = body.clone();
    drift["feedback"] = json!("Different semantic outcome payload.");

    let first = post_outcome_with_state(state.clone(), action_id, body).await;
    let conflict = post_outcome_with_state(state, action_id, drift).await;

    assert_eq!(first.0, axum_http::StatusCode::CREATED);
    assert_eq!(conflict.0, axum_http::StatusCode::CONFLICT);
    assert_eq!(conflict.1["error"]["code"], "idempotency_conflict");
}

#[tokio::test]
async fn manager_daily_brief_outcome_capture_persists_staff_feedback_as_reported_labor_evidence() {
    let action = manager_daily_brief_action_by_kind("resolve_checkout_exception").await;
    let action_id = action["id"].as_str().expect("action id");
    let mut body = outcome_body();
    body["source_refs"] = action["source_refs"].clone();

    let (status, payload) = post_outcome(action_id, body).await;

    assert_eq!(status, axum_http::StatusCode::CREATED);
    assert_eq!(payload["outcome_record"]["action_id"], action_id);
    assert_eq!(payload["outcome_record"]["outcome"], "completed");
    assert_eq!(payload["outcome_record"]["before_minutes"], 20);
    assert_eq!(payload["outcome_record"]["actual_minutes"], 12);
    assert_eq!(
        payload["outcome_record"]["source_refs"],
        action["source_refs"]
    );
    assert_eq!(
        payload["outcome_record"]["actor"]["persona"],
        "front_desk_lead"
    );
    assert_eq!(
        payload["outcome_record"]["audit"]["correlation_id"],
        "manager-daily-brief:00c0ffee-0000-0000-0000-000000000001:2026-06-17"
    );

    assert_eq!(
        payload["reported_labor_evidence"]["reported_estimated_minutes_difference"],
        action["labor_impact"]["reported_estimated_minutes_difference"]
    );
    assert_eq!(
        payload["reported_labor_evidence"]["reported_actual_minutes_spent"],
        12
    );
    assert_eq!(
        payload["reported_labor_evidence"]["grouping"]["location_id"],
        "00c0ffee-0000-0000-0000-000000000001"
    );
    assert_eq!(
        payload["reported_labor_evidence"]["grouping"]["operating_day"],
        "2026-06-17"
    );
    assert_eq!(
        payload["reported_labor_evidence"]["grouping"]["action_kind"],
        "resolve_checkout_exception"
    );
    assert_eq!(
        payload["reported_labor_evidence"]["grouping"]["owner_persona"],
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
async fn manager_daily_brief_outcome_capture_rejects_provenance_not_bound_to_the_action() {
    let action = manager_daily_brief_action_by_kind("resolve_checkout_exception").await;
    let action_id = action["id"].as_str().expect("action id");
    let mut body = outcome_body();
    body["source_refs"][0]["record_id"] = json!("borrowed-record-999");

    let (status, payload) = post_outcome(action_id, body).await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["outcome_persisted"], false);
    assert_eq!(
        payload["reasons"],
        json!(["source_refs_do_not_match_action"])
    );
}

#[tokio::test]
async fn manager_daily_brief_outcome_capture_rejects_missing_source_refs() {
    let action = manager_daily_brief_action_by_kind("resolve_checkout_exception").await;
    let action_id = action["id"].as_str().expect("action id");
    let mut body = outcome_body();
    body["source_refs"] = json!([]);

    let (status, payload) = post_outcome(action_id, body).await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["accepted"], false);
    assert_eq!(payload["outcome_persisted"], false);
    assert_eq!(payload["reasons"], json!(["missing_source_refs"]));
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
async fn manager_daily_brief_outcome_capture_rejects_nil_reporting_location() {
    let action = manager_daily_brief_action_by_kind("resolve_checkout_exception").await;
    let mut body = outcome_body();
    body["source_refs"] = action["source_refs"].clone();
    body["reporting"]["location_id"] = json!("00000000-0000-0000-0000-000000000000");

    let (status, payload) =
        post_outcome(action["id"].as_str().expect("action id is string"), body).await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["live_side_effects"], "disabled");
    assert_eq!(payload["error"]["safe_error_class"], "validation_failed");
    assert!(payload.get("outcome_persisted").is_none());
}

#[tokio::test]
async fn manager_daily_brief_outcome_capture_rejects_non_rfc3339_timestamps() {
    let action = manager_daily_brief_action_by_kind("resolve_checkout_exception").await;
    let mut body = outcome_body();
    body["source_refs"] = action["source_refs"].clone();
    body["timestamp"] = json!("2026-06-17 13:15:00");

    let (status, payload) =
        post_outcome(action["id"].as_str().expect("action id is string"), body).await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["live_side_effects"], "disabled");
}

#[tokio::test]
async fn manager_daily_brief_outcome_capture_rejects_zero_actual_minutes() {
    let action = manager_daily_brief_action_by_kind("resolve_checkout_exception").await;
    let action_id = action["id"].as_str().expect("action id");
    let mut body = outcome_body();
    body["actual_minutes"] = json!(0);

    let (status, payload) = post_outcome(action_id, body).await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["live_side_effects"], "disabled");
    assert_eq!(payload["error"]["safe_error_class"], "validation_failed");
    assert!(payload.get("accepted").is_none());
    assert!(payload.get("outcome_persisted").is_none());
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
