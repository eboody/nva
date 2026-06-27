use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::http;
use tower::ServiceExt;

async fn get_json(uri: &str) -> (axum_http::StatusCode, serde_json::Value) {
    let response = http::router_with_state(http::VaccineDocumentState::default())
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::GET)
                .uri(uri)
                .header("x-request-id", "read-model-contract-req-001")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("read-model request succeeds");

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload = serde_json::from_slice(&body).expect("json read-model payload");

    (status, payload)
}

#[tokio::test]
async fn read_model_routes_keep_fallback_safe_and_label_configured_storage_truthfully() {
    unsafe {
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("MINIO_ENDPOINT");
    }

    let (status, payload) = get_json("/v0/read-models/source-quality-backlog").await;

    assert_eq!(status, axum_http::StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        payload["api_contract"]["workflow"],
        "source_quality_backlog_read_model"
    );
    assert_eq!(payload["read_model"]["name"], "source_quality_backlog");
    assert_eq!(payload["read_model"]["source"], "postgres_view");
    assert_eq!(payload["data_posture"]["safe_synthetic_data"], true);
    assert_eq!(payload["data_posture"]["live_side_effects_allowed"], false);
    assert_eq!(
        payload["data_posture"]["provider_payload_passthrough"],
        false
    );
    assert_eq!(payload["database"]["status"], "not_configured");
    assert_eq!(payload["records"].as_array().unwrap().len(), 0);

    unsafe {
        std::env::set_var(
            "DATABASE_URL",
            "postgres://configured-for-readiness.test/pet_resort",
        );
        std::env::set_var("MINIO_ENDPOINT", "http://127.0.0.1:9000");
    }

    let (status, payload) = get_json("/readyz").await;

    assert_eq!(status, axum_http::StatusCode::OK);
    assert_eq!(payload["database"], "configured_not_verified");
    assert_eq!(payload["object_storage"], "env_configured_not_verified");
    assert_eq!(
        payload["workflow_repository"]["active_adapter"],
        "in_memory"
    );
    assert_eq!(
        payload["workflow_repository"]["postgres_adapter"],
        "env_configured_not_verified"
    );

    unsafe {
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("MINIO_ENDPOINT");
    }
}
