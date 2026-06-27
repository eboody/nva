use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::http;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn health_endpoint_reports_safe_local_service_identity() {
    let response = http::router()
        .oneshot(
            axum_http::request::Builder::new()
                .uri("/healthz")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("health request succeeds");

    assert_eq!(response.status(), axum_http::StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("json health payload");

    assert_eq!(payload["service"], "pet-resort-api");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["live_side_effects"], "disabled");
}

#[tokio::test]
async fn readiness_endpoint_keeps_mvp_dependencies_explicitly_stubbed() {
    unsafe {
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("MINIO_ENDPOINT");
    }

    let response = http::router()
        .oneshot(
            axum_http::request::Builder::new()
                .uri("/readyz")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("readiness request succeeds");

    assert_eq!(response.status(), axum_http::StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("json readiness payload");

    assert_eq!(payload["database"], "not_configured");
    assert_eq!(payload["object_storage"], "not_configured");
    assert_eq!(payload["agent_runtime"], "fake_deterministic");
    assert_eq!(
        payload["observability"]["request_correlation"],
        "x_request_id_and_x_correlation_id_response_headers_with_workflow_payload_fields"
    );
    assert_eq!(
        payload["observability"]["workflow_correlation"],
        "local_workflow_correlation_ids_only"
    );
    assert_eq!(
        payload["observability"]["local_request_metrics"],
        "api_request_span_fields_and_aggregate_summary_only"
    );
    assert_eq!(
        payload["observability"]["metrics_scope"],
        "aggregate_local_counters_and_labor_rollups"
    );
    assert_eq!(
        payload["observability"]["production_gap"],
        "no_durable_traces_queue_dashboard_or_alerting"
    );
}

#[tokio::test]
async fn inquiry_submission_creates_review_gated_intake_record() {
    let app = http::router_with_state(http::VaccineDocumentState::default());
    let response = app
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::POST)
                .uri("/inquiries")
                .header(axum_http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "source_event_key": "contract-inquiry-001",
                        "location_id": "location_local",
                        "customer": {
                            "full_name": "Avery Chen",
                            "email": "avery@example.test",
                            "phone": "555-0101"
                        },
                        "pet": {
                            "name": "Miso",
                            "species": "dog"
                        },
                        "service": "boarding",
                        "requested_dates": {
                            "start": "2026-07-03",
                            "end": "2026-07-07"
                        },
                        "message": "Miso needs boarding over the holiday. Do you need vaccine records?"
                    })
                    .to_string(),
                ))
                .expect("request builds"),
        )
        .await
        .expect("inquiry request succeeds");

    assert_eq!(response.status(), axum_http::StatusCode::CREATED);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("json intake payload");

    assert_eq!(payload["event"]["event_type"], "inquiry.received");
    assert_eq!(payload["lead"]["customer_name"], "Avery Chen");
    assert_eq!(payload["lead"]["pet_name"], "Miso");
    assert_eq!(payload["lead"]["service"], "boarding");
    assert_eq!(payload["draft_reply"]["status"], "draft_created");
    assert_eq!(payload["draft_reply"]["live_send_allowed"], false);
    assert!(
        payload["draft_reply"]["body"]
            .as_str()
            .expect("draft body string")
            .contains("Thanks Avery")
    );
    assert_eq!(payload["task"]["status"], "open");
    assert_eq!(
        payload["audit_events"][0]["action"],
        "inquiry.received.normalized"
    );
}

#[tokio::test]
async fn inquiry_intake_records_are_visible_to_staff_review_queue() {
    let app = http::router_with_state(http::VaccineDocumentState::default());
    let submit_response = app
        .clone()
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::POST)
                .uri("/inquiries")
                .header(axum_http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "source_event_key": "contract-inquiry-002",
                        "location_id": "location_local",
                        "customer": {"full_name": "Riley Patel", "email": "riley@example.test"},
                        "pet": {"name": "Juniper", "species": "dog"},
                        "service": "day_play",
                        "message": "Can Juniper come for day play next week?"
                    })
                    .to_string(),
                ))
                .expect("request builds"),
        )
        .await
        .expect("inquiry request succeeds");
    assert_eq!(submit_response.status(), axum_http::StatusCode::CREATED);

    let queue_response = app
        .oneshot(
            axum_http::request::Builder::new()
                .uri("/staff/inquiries")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("staff queue request succeeds");

    assert_eq!(queue_response.status(), axum_http::StatusCode::OK);
    let body = queue_response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("json staff queue payload");

    assert_eq!(
        payload["records"][0]["event"]["event_type"],
        "inquiry.received"
    );
    assert_eq!(
        payload["records"][0]["lead"]["customer_name"],
        "Riley Patel"
    );
    assert_eq!(
        payload["records"][0]["draft_reply"]["live_send_allowed"],
        false
    );
    assert_eq!(payload["records"][0]["task"]["kind"], "missing_info_review");
}

#[tokio::test]
async fn request_trace_echoes_safe_request_and_correlation_ids_without_payload_logging() {
    let response = http::router()
        .oneshot(
            axum_http::request::Builder::new()
                .uri("/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17")
                .header("x-request-id", "ops-readiness-req-001")
                .header("x-correlation-id", "ops-readiness-corr-001")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("context request succeeds");

    assert_eq!(response.status(), axum_http::StatusCode::OK);
    assert_eq!(
        response.headers().get("x-request-id").unwrap(),
        "ops-readiness-req-001"
    );
    assert_eq!(
        response.headers().get("x-correlation-id").unwrap(),
        "ops-readiness-corr-001"
    );
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("json context payload");

    assert_eq!(
        payload["observability"]["request_id"],
        "ops-readiness-req-001"
    );
    assert_eq!(
        payload["observability"]["request_correlation_id"],
        "ops-readiness-corr-001"
    );
    assert_eq!(payload["observability"]["payload_logging"], "disabled");
    assert_eq!(
        payload["observability"]["safe_error_class"],
        "not_applicable"
    );
    assert!(payload.to_string().contains("ops-readiness-corr-001"));
    assert!(!payload.to_string().contains("Miso needs boarding"));
}

#[tokio::test]
async fn metrics_summary_separates_local_proof_from_production_observability_gaps() {
    let response = http::router()
        .oneshot(
            axum_http::request::Builder::new()
                .uri("/ops/metrics/summary")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("metrics request succeeds");

    assert_eq!(response.status(), axum_http::StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("json metrics payload");

    assert_eq!(
        payload["api_request_metrics"]["scope"],
        "local_runtime_only"
    );
    assert_eq!(
        payload["api_request_metrics"]["payload_logging"],
        "disabled"
    );
    assert_eq!(
        payload["api_request_metrics"]["safe_error_classes"],
        json!(["validation_failed", "not_found", "not_applicable"])
    );
    assert_eq!(
        payload["observability_gap"],
        json!({
            "production_traces": "not_configured",
            "durable_request_metrics": "not_configured",
            "dashboard_or_alerting": "not_configured"
        })
    );
}
