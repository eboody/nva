use super::*;

pub(super) fn request_id_header() -> HeaderName {
    HeaderName::from_static("x-request-id")
}

pub(super) fn correlation_id_header() -> HeaderName {
    HeaderName::from_static("x-correlation-id")
}

pub(super) fn safe_request_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

pub(super) fn workflow_observability_payload(
    correlation_id: &str,
    request_trace: &RequestTraceEvidence,
) -> public_contract::WorkflowObservability {
    public_contract::WorkflowObservability {
        correlation_id: correlation_id.to_owned(),
        request_id: request_trace.request_id().to_owned(),
        request_correlation_id: request_trace.request_correlation_id().to_owned(),
        route_status_trace: "enabled".to_owned(),
        safe_error_class: "not_applicable".to_owned(),
        payload_logging: "disabled".to_owned(),
        sensitive_payload_logging: None,
    }
}

pub(super) async fn attach_request_trace(
    State(observability): State<ObservabilityRuntime>,
    mut request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let started = Instant::now();
    let method = request.method().as_str().to_owned();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or("unmatched")
        .to_owned();
    let request_trace = RequestTraceEvidence::from_request_headers(request.headers());
    request.extensions_mut().insert(request_trace.clone());

    let mut response = next.run(request).await;
    observability.record_request(
        &method,
        &route,
        response.status().as_u16(),
        started.elapsed(),
    );
    let request_id = HeaderValue::from_str(request_trace.request_id())
        .expect("generated or validated request id is a header value");
    response
        .headers_mut()
        .insert(request_id_header(), request_id);
    let correlation_id = HeaderValue::from_str(request_trace.request_correlation_id())
        .expect("generated or validated correlation id is a header value");
    response
        .headers_mut()
        .insert(correlation_id_header(), correlation_id);
    response
}

/// Builds the default staff-facing workflow router with deterministic in-memory state.
///
/// Exposed routes are safe runtime surfaces: health/readiness probes, staff inquiry
/// intake, vaccine document review, manager daily-brief packet/draft/outcome paths,
/// and data-quality hygiene packet/draft/outcome paths. They return DTO evidence and
/// review gates rather than performing live customer sends or provider writes.
pub fn router() -> Router {
    router_with_state(
        VACCINE_DOCUMENT_STATE
            .get_or_init(VaccineDocumentState::default)
            .clone(),
    )
}

/// Builds the staff-facing workflow router over caller-provided deterministic state.
///
/// Tests and local runtimes use this entrypoint to share state across requests while
/// preserving the same safety rules as [`router`]: handlers may draft, audit, and
/// persist projections, but live side effects remain blocked unless a future runtime
/// adapter adds explicit approval and provider gates.
pub fn router_with_state(state: VaccineDocumentState) -> Router {
    router_with_authentication(state, AuthenticationSource::Production)
}

/// Builds a deterministic test harness router whose synthetic actor headers are
/// promoted only when callers explicitly select this entrypoint.
///
/// Production entrypoints never call this helper; keeping it available in normal
/// library builds lets black-box integration tests exercise authentication without
/// Cargo feature unification changing whether the test suite compiles.
#[doc(hidden)]
pub fn router_with_test_auth_state(state: VaccineDocumentState) -> Router {
    router_with_authentication(state, AuthenticationSource::SyntheticTestHeaders)
}

pub(super) fn router_with_authentication(
    state: VaccineDocumentState,
    authentication_source: AuthenticationSource,
) -> Router {
    let observability = state.observability.clone();
    let protected_handler_entries = state.contract_observation.handler_entries();
    Router::new()
        .route("/v1/healthz", get(healthz))
        .route("/v1/readyz", get(readyz))
        .route("/v1/metrics", get(prometheus_metrics))
        .route("/v1/ops/metrics/summary", get(ops_metrics_summary))
        .route("/v1/inquiries", post(submit_inquiry))
        .route("/v1/staff/inquiries", get(staff_inquiries))
        .route(
            "/v1/agent/context/manager-daily-brief",
            get(manager_daily_brief_agent_context),
        )
        .route(
            "/v1/agent/drafts/manager-daily-brief",
            post(submit_manager_daily_brief_agent_draft),
        )
        .route(
            "/v1/manager-daily-brief/actions/{action_id}/outcome",
            post(capture_manager_daily_brief_action_outcome),
        )
        .route(
            "/v1/agent/context/permissioned-knowledge",
            get(permissioned_knowledge_agent_context),
        )
        .route(
            "/v1/agent/context/site-finance",
            get(site_finance_agent_context),
        )
        .route(
            "/v1/agent/context/data-quality-hygiene",
            get(data_quality_hygiene_agent_context),
        )
        .route(
            "/v1/agent/drafts/data-quality-hygiene",
            post(submit_data_quality_hygiene_agent_draft),
        )
        .route(
            "/v1/data-quality-hygiene/actions/{action_id}/outcome",
            post(capture_data_quality_hygiene_action_outcome),
        )
        .route(
            "/v1/data-quality-hygiene/outcomes/summary",
            get(data_quality_hygiene_outcome_summary),
        )
        .route(
            "/v1/read-models/source-quality-backlog",
            get(source_quality_backlog),
        )
        .route(
            "/v1/demo/information-lifespan/run",
            post(run_information_lifespan_demo),
        )
        .route(
            "/v1/demo/information-lifespan/{correlation_id}/report",
            get(replay_information_lifespan_report),
        )
        .route(
            "/v1/vaccine-documents/uploads",
            post(upload_vaccine_document),
        )
        .route(
            "/v1/vaccine-documents/review-packets/{review_packet_id}/approve",
            post(approve_vaccine_document),
        )
        .route(
            "/v1/vaccine-documents/review-packets/{review_packet_id}/reject",
            post(reject_vaccine_document),
        )
        .fallback(owned_operations_not_found)
        .with_state(state)
        .layer(Extension(protected_handler_entries))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let route = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str)
                        .unwrap_or("unknown");
                    let request_trace = request.extensions().get::<RequestTraceEvidence>();
                    let request_id = request_trace
                        .map(RequestTraceEvidence::request_id)
                        .unwrap_or("missing");
                    let correlation_id = request_trace
                        .map(RequestTraceEvidence::request_correlation_id)
                        .unwrap_or("missing");

                    tracing::info_span!(
                        "api_request",
                        http.method = %request.method(),
                        http.route = %route,
                        http.request_id = %request_id,
                        http.correlation_id = %correlation_id,
                        status_code = tracing::field::Empty,
                        duration_ms = tracing::field::Empty,
                        safe_error_class = tracing::field::Empty,
                        actor_source = "payload_or_unset",
                        location_id = tracing::field::Empty,
                        tenant_id = tracing::field::Empty,
                        payload_logging = "disabled"
                    )
                })
                .on_response(record_api_response_metrics),
        )
        .layer(middleware::from_fn(normalize_public_api_errors))
        .layer(middleware::from_fn_with_state(
            authentication_source,
            attach_authenticated_actor,
        ))
        .layer(middleware::from_fn_with_state(
            observability,
            attach_request_trace,
        ))
}
