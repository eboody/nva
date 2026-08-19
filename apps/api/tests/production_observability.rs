use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::{
    http,
    observability::{
        ObservabilityConfig, ObservabilityConfigError, ObservabilityRuntime, TelemetryMode,
    },
};
use tower::ServiceExt;

#[test]
fn production_telemetry_configuration_is_complete_or_fails_closed() {
    let missing = ObservabilityConfig::try_from_pairs([
        ("PET_RESORT_TELEMETRY_MODE", "production"),
        (
            "PET_RESORT_OBSERVABILITY_DASHBOARD_URL",
            "http://127.0.0.1:3002/d/pet-resort-api",
        ),
        (
            "PET_RESORT_OBSERVABILITY_ALERT_POLICY",
            "nva-platform/pet-resort-api",
        ),
    ]);
    assert_eq!(missing, Err(ObservabilityConfigError::MissingOtlpEndpoint));

    let configured = ObservabilityConfig::try_from_pairs([
        ("PET_RESORT_TELEMETRY_MODE", "production"),
        ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://127.0.0.1:4317"),
        (
            "PET_RESORT_OBSERVABILITY_DASHBOARD_URL",
            "http://127.0.0.1:3002/d/pet-resort-api",
        ),
        (
            "PET_RESORT_OBSERVABILITY_ALERT_POLICY",
            "nva-platform/pet-resort-api",
        ),
    ])
    .unwrap();

    assert_eq!(configured.mode(), TelemetryMode::Production);
    let readiness = configured.readiness();
    assert_eq!(readiness.durable_traces, "configured_otlp");
    assert_eq!(readiness.production_metrics, "configured_prometheus");
    assert_eq!(readiness.dashboard, "configured_grafana");
    assert_eq!(readiness.alerting, "configured_prometheus_rules");
}

#[tokio::test]
async fn metrics_endpoint_exports_bounded_prometheus_request_series() {
    let runtime = ObservabilityRuntime::new(ObservabilityConfig::local());
    let state = http::VaccineDocumentState::default().with_observability(runtime);
    let app = http::router_with_test_auth_state(state);

    for _ in 0..2 {
        let response = app
            .clone()
            .oneshot(
                axum_http::Request::builder()
                    .uri("/v1/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum_http::StatusCode::OK);
    }

    let response = app
        .oneshot(
            axum_http::Request::builder()
                .uri("/v1/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum_http::StatusCode::OK);
    assert_eq!(
        response.headers()[axum_http::header::CONTENT_TYPE],
        "text/plain; version=0.0.4; charset=utf-8"
    );
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("pet_resort_api_requests_total"));
    assert!(body.contains("method=\"GET\""));
    assert!(body.contains("route=\"/v1/healthz\""));
    assert!(body.contains("status_class=\"2xx\"} 2"));
    assert!(body.contains("pet_resort_api_request_duration_seconds_bucket"));
}
