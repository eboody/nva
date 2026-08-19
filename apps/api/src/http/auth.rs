use super::*;

pub(super) fn authorization_error_payload(
    rejection: authentication::Rejection,
    workflow: &'static str,
    persisted_field: &'static str,
) -> Value {
    let mut payload = json!({
        "api_contract": api_dto_contract_payload(workflow),
        "accepted": false,
    });
    payload[persisted_field] = Value::Bool(false);
    merge_error_envelope(
        payload,
        PublicApiError::new(rejection.into(), ErrorContext::new("missing_request_id")),
    )
}

impl authentication::Rejection {
    pub(super) fn status_code(self) -> StatusCode {
        PublicApiError::new(self.into(), ErrorContext::new("missing_request_id")).status_code()
    }
}

#[derive(Clone, Copy)]
pub(super) enum AuthenticationSource {
    Production,
    SyntheticTestHeaders,
}

pub(super) async fn attach_authenticated_actor(
    State(source): State<AuthenticationSource>,
    mut request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let context = match source {
        AuthenticationSource::Production => authentication::Context::missing(),
        AuthenticationSource::SyntheticTestHeaders => {
            authentication::Context::from_test_headers(request.headers())
        }
    };
    request.extensions_mut().insert(context);
    next.run(request).await
}
