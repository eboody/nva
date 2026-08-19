use super::*;

pub(super) async fn normalize_public_api_errors(
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let context = request
        .extensions()
        .get::<RequestTraceEvidence>()
        .map(|trace| {
            ErrorContext::new(trace.request_id())
                .with_correlation_id(trace.request_correlation_id())
        })
        .unwrap_or_else(|| ErrorContext::new("missing_request_id"));
    let response = next.run(request).await;
    let status = response.status();
    if !status.is_client_error() && !status.is_server_error() {
        return response;
    }

    let (parts, body) = response.into_parts();
    let bytes = match to_bytes(body, 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return PublicApiError::new(ErrorKind::Internal, context).into_response();
        }
    };
    let payload = serde_json::from_slice::<Value>(&bytes).unwrap_or(Value::Null);
    if payload["error"]["safe_error_class"].is_string()
        && payload["request_id"].is_string()
        && payload["live_side_effects"].is_string()
    {
        return Response::from_parts(parts, Body::from(bytes));
    }

    let mut error = PublicApiError::new(error_kind_for_response(status, &payload), context);
    if let Some(public_code) = payload["error"]["code"].as_str() {
        error = error.with_public_code(public_code);
    }
    let merged = merge_error_envelope(payload, error);
    (status, Json(merged)).into_response()
}

pub(super) fn merge_error_envelope(mut payload: Value, error: PublicApiError) -> Value {
    let envelope =
        serde_json::to_value(error.envelope()).expect("public error envelope always serializes");
    let Value::Object(envelope_fields) = envelope else {
        unreachable!("public error envelope serializes as an object")
    };
    let Value::Object(payload_fields) = &mut payload else {
        return Value::Object(envelope_fields);
    };
    payload_fields.extend(envelope_fields);
    payload
}

pub(super) fn error_kind_for_response(status: StatusCode, payload: &Value) -> ErrorKind {
    match status {
        StatusCode::UNAUTHORIZED => {
            ErrorKind::Authentication(AuthenticationFailure::MissingTrustedActorContext)
        }
        StatusCode::FORBIDDEN => {
            ErrorKind::Authorization(AuthorizationFailure::ActorRoleNotAuthorized)
        }
        StatusCode::CONFLICT
            if payload["classification"] == "idempotency_payload_drift"
                || payload["error"]["code"] == "idempotency_payload_drift" =>
        {
            ErrorKind::IdempotencyConflict
        }
        StatusCode::CONFLICT => ErrorKind::ReviewConflict,
        StatusCode::UNPROCESSABLE_ENTITY if response_reports_source_ambiguity(payload) => {
            ErrorKind::SourceAmbiguity
        }
        StatusCode::UNPROCESSABLE_ENTITY | StatusCode::BAD_REQUEST => ErrorKind::Validation {
            details: validation_details(payload),
        },
        StatusCode::NOT_FOUND => ErrorKind::NotFound {
            details: Vec::new(),
        },
        StatusCode::SERVICE_UNAVAILABLE | StatusCode::BAD_GATEWAY | StatusCode::GATEWAY_TIMEOUT => {
            ErrorKind::Unavailable
        }
        _ => ErrorKind::Internal,
    }
}

pub(super) fn response_reports_source_ambiguity(payload: &Value) -> bool {
    payload
        .get("reasons")
        .and_then(Value::as_array)
        .is_some_and(|reasons| {
            reasons.iter().any(|reason| {
                reason
                    .as_str()
                    .is_some_and(|reason| reason.contains("source") || reason.contains("issue_ref"))
            })
        })
}

pub(super) fn validation_details(payload: &Value) -> Vec<public_contract::ErrorDetail> {
    payload
        .get("reasons")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|reason| public_contract::ErrorDetail::field("request".to_owned(), reason.to_owned()))
        .collect()
}

pub(super) fn record_api_response_metrics(
    response: &Response<Body>,
    latency: Duration,
    span: &Span,
) {
    let status_code = response.status().as_u16();
    let duration_ms = latency.as_millis() as u64;
    let safe_error_class = safe_error_class_for_status(response.status());

    span.record("status_code", status_code);
    span.record("duration_ms", duration_ms);
    span.record("safe_error_class", safe_error_class);

    tracing::event!(
        parent: span,
        Level::INFO,
        status_code,
        duration_ms,
        safe_error_class,
        payload_logging = "disabled",
        "api_request completed"
    );
}

pub(super) fn safe_error_class_for_status(status: StatusCode) -> &'static str {
    match status {
        StatusCode::NOT_FOUND => "not_found",
        status if status.is_client_error() => "validation_failed",
        status if status.is_server_error() => "internal_error",
        _ => "not_applicable",
    }
}

pub(super) async fn owned_operations_not_found(request: Request<Body>) -> axum::response::Response {
    let request_id = request
        .extensions()
        .get::<RequestTraceEvidence>()
        .map(RequestTraceEvidence::request_id)
        .unwrap_or("missing_request_id")
        .to_owned();
    let path = request.uri().path().to_owned();

    PublicApiError::new(
        ErrorKind::NotFound {
            details: vec![public_contract::ErrorDetail::field("path".to_owned(), path)],
        },
        ErrorContext::new(request_id),
    )
    .into_response()
}
