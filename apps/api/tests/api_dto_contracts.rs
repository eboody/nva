use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::http;
use serde_json::json;
use tower::ServiceExt;

async fn request_json(
    method: axum_http::Method,
    uri: &str,
    body: Option<serde_json::Value>,
) -> (axum_http::StatusCode, serde_json::Value) {
    let app = http::router_with_state(http::VaccineDocumentState::default());
    let (status, _, payload) = request_json_on(app, method, uri, body, None).await;
    (status, payload)
}

async fn request_json_with_headers(
    method: axum_http::Method,
    uri: &str,
    body: Option<serde_json::Value>,
    request_id: Option<&str>,
) -> (
    axum_http::StatusCode,
    axum_http::HeaderMap,
    serde_json::Value,
) {
    let app = http::router_with_state(http::VaccineDocumentState::default());
    request_json_on(app, method, uri, body, request_id).await
}

async fn request_json_on(
    app: axum::Router,
    method: axum_http::Method,
    uri: &str,
    body: Option<serde_json::Value>,
    request_id: Option<&str>,
) -> (
    axum_http::StatusCode,
    axum_http::HeaderMap,
    serde_json::Value,
) {
    let mut builder = axum_http::request::Builder::new().method(method).uri(uri);
    if let Some(request_id) = request_id {
        builder = builder.header("x-request-id", request_id);
    }
    let request_body = if let Some(body) = body {
        builder = builder.header(axum_http::header::CONTENT_TYPE, "application/json");
        Body::from(body.to_string())
    } else {
        Body::empty()
    };

    let response = app
        .oneshot(builder.body(request_body).expect("request builds"))
        .await
        .expect("api contract request succeeds");
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload = serde_json::from_slice(&body).expect("json api payload");

    (status, headers, payload)
}

fn assert_product_owned_runtime_dto_contract(payload: &serde_json::Value, workflow: &str) {
    assert_eq!(payload["api_contract"]["owner"], "pet_resort_api");
    assert_eq!(payload["api_contract"]["boundary"], "api_runtime_dto");
    assert_eq!(payload["api_contract"]["workflow"], workflow);
    assert_eq!(
        payload["api_contract"]["provider_payload_passthrough"],
        false
    );
    assert_eq!(
        payload["api_contract"]["provider_dto_boundary"],
        "provider_evidence_only"
    );
}

#[tokio::test]
async fn health_and_readiness_payloads_label_themselves_as_product_owned_runtime_dtos() {
    let (health_status, health) = request_json(axum_http::Method::GET, "/healthz", None).await;
    assert_eq!(health_status, axum_http::StatusCode::OK);
    assert_product_owned_runtime_dto_contract(&health, "runtime_health");
    assert_eq!(health["live_side_effects"], "disabled");

    let (ready_status, ready) = request_json(axum_http::Method::GET, "/readyz", None).await;
    assert_eq!(ready_status, axum_http::StatusCode::OK);
    assert_product_owned_runtime_dto_contract(&ready, "runtime_readiness");
    assert_eq!(ready["live_customer_messaging"], "disabled");
    assert_eq!(ready["live_provider_writes"], "disabled");
    assert_eq!(ready["workflow_repository"]["active_adapter"], "in_memory");
    assert_eq!(
        ready["workflow_repository"]["postgres_adapter"],
        "planned_same_contract"
    );
    assert_eq!(
        ready["workflow_repository"]["contract"].as_array().unwrap(),
        &[
            json!("workflow_events"),
            json!("review_packets"),
            json!("audit_events"),
            json!("outcomes"),
            json!("documents")
        ]
    );
}

#[tokio::test]
async fn api_requests_echo_safe_request_ids_for_route_status_tracing() {
    let (status, headers, payload) = request_json_with_headers(
        axum_http::Method::GET,
        "/healthz",
        None,
        Some("req-test-safe-123"),
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::OK);
    assert_product_owned_runtime_dto_contract(&payload, "runtime_health");
    assert_eq!(
        headers
            .get("x-request-id")
            .expect("request id response header present"),
        "req-test-safe-123"
    );
}

#[tokio::test]
async fn agent_workflow_packets_carry_correlation_without_payload_secrets() {
    let (status, headers, context) = request_json_with_headers(
        axum_http::Method::GET,
        "/agent/context/manager-daily-brief?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
        None,
        Some("req-workflow-safe-456"),
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::OK);
    assert_product_owned_runtime_dto_contract(&context, "manager_daily_brief");
    assert_eq!(
        context["observability"]["request_id"],
        "req-workflow-safe-456"
    );
    assert_eq!(
        context["observability"]["correlation_id"],
        context["audit"]["correlation_id"]
    );
    assert_eq!(
        headers
            .get("x-request-id")
            .expect("request id response header present"),
        "req-workflow-safe-456"
    );
    assert!(context["observability"]["sensitive_payload_logging"].is_null());
}

#[tokio::test]
async fn vaccine_document_workflow_payload_contract_preserves_review_gate_and_audit_safety() {
    let (status, payload) = request_json(
        axum_http::Method::POST,
        "/vaccine-documents/uploads",
        Some(json!({
            "pet_id": "00000000-0000-0000-0000-000000000101",
            "customer_id": "00000000-0000-0000-0000-000000000201",
            "filename": "rabies-certificate.txt",
            "mime_type": "text/plain",
            "content": "Rabies vaccine administered 2026-01-15 expires 2027-01-15 for Miso",
            "uploaded_by_staff_id": "front-desk-demo"
        })),
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::CREATED);
    assert_product_owned_runtime_dto_contract(&payload, "vaccine_document_review");
    assert_eq!(payload["review_packet"]["gate"], "medical_document_review");
    assert_eq!(payload["review_packet"]["status"], "ready_for_review");
    assert_eq!(payload["eligibility"]["rabies_current"], false);
    assert!(
        payload["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| {
                event["action"] == "vaccine_record.review_requested"
                    && event["metadata"]["review_packet_id"].is_string()
            })
    );
}

#[tokio::test]
async fn manager_daily_brief_payload_contract_preserves_review_gates_labor_and_disabled_side_effects()
 {
    let (status, context) = request_json(
        axum_http::Method::GET,
        "/agent/context/manager-daily-brief?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
        None,
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::OK);
    assert_product_owned_runtime_dto_contract(&context, "manager_daily_brief");
    assert!(
        context["manager_brief_actions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|action| {
                action["required_review_gates"]
                    .as_array()
                    .is_some_and(|gates| !gates.is_empty())
                    && action["labor_impact"]["before_minutes"].is_number()
                    && action["labor_impact"]["minutes_saved"].is_number()
            })
    );
    assert!(
        context["blocked_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("mutate_provider_or_pms_record"))
    );

    let action = &context["manager_brief_actions"][0];
    let (draft_status, draft) = request_json(
        axum_http::Method::POST,
        "/agent/drafts/manager-daily-brief",
        Some(json!({
            "context_packet_id": context["audit"]["context_packet_id"],
            "correlation_id": context["audit"]["correlation_id"],
            "submitted_by": "hermes-agent",
            "actions": [{
                "id": action["id"],
                "kind": action["kind"],
                "recommendation": "Review this source-grounded action before staff execution.",
                "source_refs": action["source_facts"][0]["source_refs"],
                "review_gates": action["required_review_gates"],
                "requested_side_effects": []
            }]
        })),
    )
    .await;

    assert_eq!(draft_status, axum_http::StatusCode::CREATED);
    assert_product_owned_runtime_dto_contract(&draft, "manager_daily_brief_agent_draft");
    assert_eq!(
        draft["accepted_actions"][0]["live_side_effects_allowed"],
        false
    );
    assert_eq!(
        draft["audit"]["event"],
        "manager_daily_brief_agent_draft_validated"
    );
}

#[tokio::test]
async fn ops_metrics_summary_counts_safe_local_state_without_prometheus_overbuild() {
    let app = http::router_with_state(http::VaccineDocumentState::default());

    let (initial_status, _, initial) = request_json_on(
        app.clone(),
        axum_http::Method::GET,
        "/ops/metrics/summary",
        None,
        None,
    )
    .await;
    assert_eq!(initial_status, axum_http::StatusCode::OK);
    assert_product_owned_runtime_dto_contract(&initial, "ops_metrics_summary");
    assert_eq!(initial["safety"]["live_side_effects"], "disabled");
    assert_eq!(initial["local_runtime_counters"]["inquiry_count"], 0);
    assert_eq!(initial["local_runtime_counters"]["review_packet_count"], 0);
    assert_eq!(initial["local_runtime_counters"]["audit_event_count"], 0);
    assert_eq!(initial["local_runtime_counters"]["outcome_count"], 0);

    let (inquiry_status, _, _) = request_json_on(
        app.clone(),
        axum_http::Method::POST,
        "/inquiries",
        Some(json!({
            "source_event_key": "web-inquiry-metrics-smoke-1",
            "location_id": "00c0ffee-0000-0000-0000-000000000001",
            "customer": {"full_name": "Metrics Smoke", "email": "metrics@example.test", "phone": null},
            "pet": {"name": "Maple", "species": "dog"},
            "service": "boarding",
            "requested_dates": {"start": "2026-07-01", "end": "2026-07-04"},
            "message": "Need boarding details."
        })),
        None,
    )
    .await;
    assert_eq!(inquiry_status, axum_http::StatusCode::CREATED);

    let (upload_status, _, _) = request_json_on(
        app.clone(),
        axum_http::Method::POST,
        "/vaccine-documents/uploads",
        Some(json!({
            "pet_id": "00000000-0000-0000-0000-000000000301",
            "customer_id": "00000000-0000-0000-0000-000000000401",
            "filename": "metrics-rabies-certificate.txt",
            "mime_type": "text/plain",
            "content": "Rabies vaccine administered 2026-01-15 expires 2027-01-15 for Maple",
            "uploaded_by_staff_id": "front-desk-metrics-smoke"
        })),
        None,
    )
    .await;
    assert_eq!(upload_status, axum_http::StatusCode::CREATED);

    let (context_status, _, context) = request_json_on(
        app.clone(),
        axum_http::Method::GET,
        "/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
        None,
        None,
    )
    .await;
    assert_eq!(context_status, axum_http::StatusCode::OK);
    let action = &context["hygiene_actions"][0];
    let (outcome_status, _, _) = request_json_on(
        app.clone(),
        axum_http::Method::POST,
        &format!(
            "/data-quality-hygiene/actions/{}/outcome",
            action["id"].as_str().unwrap()
        ),
        Some(json!({
            "outcome": "completed",
            "actual_minutes": 9,
            "actor": {"id": "front-desk-lead-17", "persona": "front_desk_lead"},
            "feedback": "Recorded aggregate metrics smoke outcome without touching Gingr.",
            "source_refs": action["source_refs"],
            "issue_refs": action["issue_refs"],
            "resolution_status_after_review": "acknowledged",
            "timestamp": "2026-06-17T13:15:00Z",
            "audit": {"correlation_id": context["audit"]["correlation_id"]}
        })),
        None,
    )
    .await;
    assert_eq!(outcome_status, axum_http::StatusCode::CREATED);

    let (metrics_status, _, metrics) = request_json_on(
        app,
        axum_http::Method::GET,
        "/ops/metrics/summary",
        None,
        None,
    )
    .await;
    assert_eq!(metrics_status, axum_http::StatusCode::OK);
    assert_product_owned_runtime_dto_contract(&metrics, "ops_metrics_summary");
    assert_eq!(metrics["local_runtime_counters"]["inquiry_count"], 1);
    assert_eq!(metrics["local_runtime_counters"]["review_packet_count"], 1);
    assert_eq!(metrics["local_runtime_counters"]["audit_event_count"], 3);
    assert_eq!(metrics["local_runtime_counters"]["outcome_count"], 1);
    assert_eq!(
        metrics["product_labor_metrics"]["data_quality_hygiene"]["completed_count"],
        1
    );
    assert_eq!(
        metrics["production_metrics_plan"].as_array().unwrap(),
        &[
            json!("request_latency"),
            json!("error_rate"),
            json!("queue_depth"),
            json!("dead_letter_count"),
            json!("review_sla"),
            json!("outbox_failures"),
            json!("worker_lease_age")
        ]
    );
}

#[tokio::test]
async fn data_quality_hygiene_payload_contract_preserves_review_packet_status_and_labor_outcome_shape()
 {
    let (status, context) = request_json(
        axum_http::Method::GET,
        "/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
        None,
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::OK);
    assert_product_owned_runtime_dto_contract(&context, "data_quality_hygiene");
    assert_eq!(context["live_side_effects_allowed"], false);
    assert!(
        context["hygiene_actions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|action| {
                action["review_gates"]
                    .as_array()
                    .is_some_and(|gates| !gates.is_empty())
                    && action["labor_impact"]["estimated_minutes_saved"].is_number()
            })
    );

    let action = &context["hygiene_actions"][0];
    let (outcome_status, outcome) = request_json(
        axum_http::Method::POST,
        &format!(
            "/data-quality-hygiene/actions/{}/outcome",
            action["id"].as_str().unwrap()
        ),
        Some(json!({
            "outcome": "completed",
            "actual_minutes": 9,
            "actor": {"id": "front-desk-lead-17", "persona": "front_desk_lead"},
            "feedback": "Prepared source-grounded cleanup task for manager review without touching Gingr.",
            "source_refs": action["source_refs"],
            "issue_refs": action["issue_refs"],
            "resolution_status_after_review": "acknowledged",
            "timestamp": "2026-06-17T13:15:00Z",
            "audit": {"correlation_id": context["audit"]["correlation_id"]}
        })),
    )
    .await;

    assert_eq!(outcome_status, axum_http::StatusCode::CREATED);
    assert_product_owned_runtime_dto_contract(&outcome, "data_quality_hygiene_outcome");
    assert_eq!(outcome["outcome_persisted"], true);
    assert_eq!(outcome["outcome_record"]["actual_minutes"], 9);
    assert_eq!(outcome["live_side_effects_allowed"], false);
    assert_eq!(
        outcome["audit"]["event"],
        "data_quality_hygiene_outcome_recorded"
    );
}
