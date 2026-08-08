use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::http;
use tower::ServiceExt;

async fn request_json(
    method: axum_http::Method,
    uri: &str,
) -> (axum_http::StatusCode, serde_json::Value) {
    let response = http::router_with_state(http::VaccineDocumentState::default())
        .oneshot(
            axum_http::request::Builder::new()
                .method(method)
                .uri(uri)
                .header("x-request-id", "api-contract-test-request")
                .header("x-correlation-id", "api-contract-test-correlation")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("information lifespan request succeeds");

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload = serde_json::from_slice(&body).expect("json payload");

    (status, payload)
}

#[tokio::test]
async fn information_lifespan_run_endpoint_returns_report_artifact_and_visible_proof() {
    let (status, payload) =
        request_json(axum_http::Method::POST, "/v0/demo/information-lifespan/run").await;

    assert_eq!(status, axum_http::StatusCode::OK);
    assert_eq!(
        payload["api_contract"]["workflow"],
        "information_lifespan_demo_run"
    );
    assert_eq!(payload["correlation_id"], "info-lifespan-demo-2026-06-29");
    assert_eq!(
        payload["observability"]["request_id"],
        "api-contract-test-request"
    );
    assert_eq!(payload["trace"]["synthetic_data_only"], true);
    assert_eq!(
        payload["trace"]["provider_payloads_are_source_evidence_only"],
        true
    );
    assert_eq!(payload["trace"]["live_side_effects_allowed"], false);
    assert!(payload["trace"]["stages"].as_array().unwrap().len() >= 8);
    assert!(
        payload["trace"]["db_proof_entries"]
            .as_array()
            .unwrap()
            .len()
            >= 6
    );
    assert!(
        payload["trace"]["calculations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| {
                entry["name"] == "estimated_labor_minutes_saved" && entry["result"] == "42"
            })
    );
    assert_eq!(
        payload["final_report"]["artifact_ref"],
        "artifact://manager-daily-report/synthetic-2026-06-29"
    );
    assert!(
        payload["processor_proof"]["contract_version"]
            .as_str()
            .unwrap()
            .contains("information_lifespan_hermes_processor_output")
    );
    assert!(
        payload["safety"]["review_gates"]
            .as_array()
            .unwrap()
            .iter()
            .all(|gate| { gate["locked"] == true })
    );
    assert_eq!(payload["safety"]["live_side_effects_allowed"], false);
}

#[tokio::test]
async fn information_lifespan_report_endpoint_replays_same_artifact_by_correlation_id() {
    let (status, payload) = request_json(
        axum_http::Method::GET,
        "/v0/demo/information-lifespan/info-lifespan-demo-2026-06-29/report",
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::OK);
    assert_eq!(
        payload["api_contract"]["workflow"],
        "information_lifespan_demo_report"
    );
    assert_eq!(payload["correlation_id"], "info-lifespan-demo-2026-06-29");
    assert_eq!(
        payload["final_report"]["title"],
        "Manager Daily Report — synthetic 2026-06-29"
    );
    assert_eq!(
        payload["network_proof"][0]["path"],
        "/demo/information-lifespan/run"
    );
    assert!(
        payload["deferred_production_work"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| { item.as_str().unwrap().contains("live NVA/Gingr access") })
    );
}

#[tokio::test]
async fn information_lifespan_report_endpoint_rejects_unknown_correlation_id() {
    let (status, payload) = request_json(
        axum_http::Method::GET,
        "/v0/demo/information-lifespan/not-the-demo/report",
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::NOT_FOUND);
    assert_eq!(
        payload["error"]["code"],
        "information_lifespan_run_not_found"
    );
    assert_eq!(payload["live_side_effects_allowed"], false);
}
