use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::http;
use serde_json::json;
use tower::ServiceExt;

async fn get_json_with_state(
    state: http::VaccineDocumentState,
    uri: &str,
) -> (axum_http::StatusCode, serde_json::Value) {
    let response = http::router_with_state(state)
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::GET)
                .uri(uri)
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("stateful data-quality hygiene summary request succeeds");

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload = serde_json::from_slice(&body).expect("json summary payload");

    (status, payload)
}

async fn post_json_with_state(
    state: http::VaccineDocumentState,
    uri: &str,
    body: serde_json::Value,
) -> (axum_http::StatusCode, serde_json::Value) {
    let response = http::router_with_state(state)
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::POST)
                .uri(uri)
                .header(axum_http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .expect("request builds"),
        )
        .await
        .expect("stateful data-quality hygiene post request succeeds");

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
        payload["outcome_record"]["actor"]["persona"],
        "front_desk_lead"
    );
    let persisted_source_ref = &payload["outcome_record"]["source_refs"][0];
    let action_source_ref = &action["source_refs"][0];
    assert_eq!(persisted_source_ref["system"], action_source_ref["system"]);
    assert_eq!(
        persisted_source_ref["record_id"],
        action_source_ref["record_id"]
    );
    assert_eq!(persisted_source_ref["record_type"], "source_record");
    assert!(persisted_source_ref["observed_at"].is_string());
    assert!(persisted_source_ref["adapter_version"].is_string());
    assert_eq!(
        payload["outcome_record"]["issue_refs"],
        action["issue_refs"]
    );
    assert_eq!(
        payload["outcome_record"]["resolution_status_after_review"],
        "acknowledged"
    );
    assert_eq!(payload["live_side_effects_allowed"], false);
    assert_eq!(
        payload["observability"]["correlation_id"],
        context["audit"]["correlation_id"]
    );
    assert_eq!(
        payload["observability"]["workflow_event_id"],
        payload["storage_projection_proof"]["workflow_event_id"]
    );
    assert_eq!(
        payload["observability"]["outbox_candidate_id"],
        payload["storage_projection_proof"]["outbox_candidate"]["id"]
    );
    assert_eq!(
        payload["observability"]["what_happened"],
        "reviewed_outcome_recorded_and_internal_outbox_candidate_created"
    );
    assert_eq!(
        payload["observability"]["what_was_blocked"],
        json!([
            "provider_writes",
            "customer_sends",
            "payments",
            "schedule_changes"
        ])
    );
    assert_eq!(
        payload["observability"]["production_next_step"],
        "durable_worker_leasing_retry_dead_letter_metrics_and_approved_adapter_execution"
    );
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
    assert_eq!(payload["local_demo_readiness"]["mode"], "local_demo_only");
    assert_eq!(
        payload["local_demo_readiness"]["workflow_repository"],
        "in_memory_typed_storage_projection"
    );
    assert_eq!(
        payload["local_demo_readiness"]["live_provider_writes"],
        "disabled"
    );
    assert_eq!(
        payload["local_demo_readiness"]["live_customer_sends"],
        "disabled"
    );
    assert_eq!(payload["local_demo_readiness"]["payments"], "disabled");
    assert!(
        payload["storage_projection_proof"]["workflow_event_id"]
            .as_str()
            .unwrap()
            .starts_with("dqh-workflow-event:")
    );
    assert!(
        payload["storage_projection_proof"]["review_packet_id"]
            .as_str()
            .unwrap()
            .starts_with("dqh-review-packet:")
    );
    assert!(
        payload["storage_projection_proof"]["approval_record_id"]
            .as_str()
            .unwrap()
            .starts_with("dqh-approval:")
    );
    assert_eq!(
        payload["storage_projection_proof"]["workflow_result_status"],
        "succeeded"
    );
    assert_eq!(
        payload["storage_projection_proof"]["review_gate"],
        "manager_approval"
    );
    assert_eq!(payload["storage_projection_proof"]["audit_event_count"], 2);
    assert_eq!(
        payload["storage_projection_proof"]["outbox_candidate"]["topic"],
        "internal.data_quality_hygiene.reviewed_handoff"
    );
    assert_eq!(
        payload["storage_projection_proof"]["outbox_candidate"]["internal_handoff_only"],
        true
    );
    assert_eq!(
        payload["storage_projection_proof"]["outbox_candidate"]["live_delivery_allowed"],
        false
    );
    assert_eq!(
        payload["storage_projection_proof"]["outbox_candidate"]["status"],
        "pending"
    );
}

#[tokio::test]
async fn data_quality_hygiene_outcome_capture_rejects_missing_source_or_issue_refs() {
    let context = data_quality_context().await;
    let action = &context["hygiene_actions"][0];
    let action_id = action["id"].as_str().unwrap();
    let base_body = json!({
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

    let mut without_source_refs = base_body.clone();
    without_source_refs["source_refs"] = json!([]);
    let (status, payload) = post_json(
        &format!("/data-quality-hygiene/actions/{action_id}/outcome"),
        without_source_refs,
    )
    .await;
    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["accepted"], false);
    assert_eq!(payload["outcome_persisted"], false);
    assert_eq!(payload["reasons"], json!(["missing_source_refs"]));
    assert_eq!(payload["live_side_effects_allowed"], false);

    let mut without_issue_refs = base_body;
    without_issue_refs["issue_refs"] = json!([]);
    let (status, payload) = post_json(
        &format!("/data-quality-hygiene/actions/{action_id}/outcome"),
        without_issue_refs,
    )
    .await;
    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["accepted"], false);
    assert_eq!(payload["outcome_persisted"], false);
    assert_eq!(
        payload["reasons"],
        json!(["missing_data_quality_issue_refs"])
    );
    assert_eq!(payload["live_side_effects_allowed"], false);
}

#[tokio::test]
async fn data_quality_hygiene_outcome_summary_reports_reviewed_minutes_and_provenance() {
    let state = http::VaccineDocumentState::default();
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

    let (status, capture_payload) = post_json_with_state(
        state.clone(),
        &format!("/data-quality-hygiene/actions/{action_id}/outcome"),
        body,
    )
    .await;
    assert_eq!(status, axum_http::StatusCode::CREATED);
    let grouping = &capture_payload["labor_savings_evidence"]["grouping"];
    let correlation_id = capture_payload["outcome_record"]["audit"]["correlation_id"]
        .as_str()
        .unwrap();
    let summary_uri = format!(
        "/data-quality-hygiene/outcomes/summary?location_id={}&operating_day={}&correlation_id={}",
        grouping["location_id"].as_str().unwrap(),
        grouping["operating_day"].as_str().unwrap(),
        correlation_id
    );

    let (status, payload) = get_json_with_state(state.clone(), &summary_uri).await;

    assert_eq!(status, axum_http::StatusCode::OK);
    assert_eq!(payload["summary"]["reviewed_outcome_count"], 1);
    assert_eq!(payload["summary"]["completed_count"], 1);
    assert_eq!(payload["summary"]["deferred_count"], 0);
    assert_eq!(payload["summary"]["wrong_source_count"], 0);
    assert_eq!(payload["summary"]["not_actionable_count"], 0);
    assert_eq!(
        payload["summary"]["total_estimated_minutes_saved"],
        action["labor_impact"]["estimated_minutes_saved"]
    );
    assert_eq!(payload["summary"]["total_actual_minutes_spent"], 9);
    assert!(
        payload["summary"]["completed_actual_minutes_saved"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert_eq!(
        payload["summary"]["source_refs"][0]["system"],
        action["source_refs"][0]["system"]
    );
    assert_eq!(
        payload["summary"]["source_refs"][0]["record_id"],
        action["source_refs"][0]["record_id"]
    );
    assert_eq!(payload["summary"]["issue_refs"], action["issue_refs"]);
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
            .contains(&json!("move_refund_discount_or_payment"))
    );
    assert!(
        payload["blocked_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("change_staff_schedule"))
    );

    let (status, metrics_payload) = get_json_with_state(state, "/ops/metrics/summary").await;
    assert_eq!(status, axum_http::StatusCode::OK);
    assert_eq!(
        metrics_payload["local_runtime_counters"]["data_quality_hygiene_outbox_candidate_count"],
        1
    );
    assert_eq!(
        metrics_payload["local_runtime_counters"]["data_quality_hygiene_review_gated_outbox_count"],
        1
    );
    assert_eq!(
        metrics_payload["local_runtime_counters"]["production_queue_adapter"],
        "not_configured"
    );
}
