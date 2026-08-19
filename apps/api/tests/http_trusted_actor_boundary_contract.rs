use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::http;
use serde_json::json;
use tower::ServiceExt;

const LOCAL_LOCATION_ID: &str = "00c0ffee-0000-0000-0000-000000000001";

async fn request_json_on(
    app: axum::Router,
    method: axum_http::Method,
    uri: &str,
    body: serde_json::Value,
    trusted_actor: Option<TrustedActorHeaders<'_>>,
) -> (axum_http::StatusCode, serde_json::Value) {
    let mut builder = axum_http::request::Builder::new()
        .method(method)
        .uri(uri)
        .header(axum_http::header::CONTENT_TYPE, "application/json");

    if let Some(actor) = trusted_actor {
        builder = builder
            .header("x-test-auth-actor-id", actor.actor_id)
            .header("x-test-auth-role", actor.role)
            .header("x-test-auth-location-id", actor.location_id);
    }

    let response = app
        .oneshot(
            builder
                .body(Body::from(body.to_string()))
                .expect("request builds"),
        )
        .await
        .expect("trusted actor boundary request succeeds");
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload = serde_json::from_slice(&body).expect("json api payload");
    (status, payload)
}

#[derive(Clone, Copy)]
struct TrustedActorHeaders<'a> {
    actor_id: &'a str,
    role: &'a str,
    location_id: &'a str,
}

fn front_desk_lead() -> TrustedActorHeaders<'static> {
    TrustedActorHeaders {
        actor_id: "front-desk-lead-17",
        role: "front_desk_lead",
        location_id: LOCAL_LOCATION_ID,
    }
}

fn medical_reviewer() -> TrustedActorHeaders<'static> {
    TrustedActorHeaders {
        actor_id: "medical-reviewer-42",
        role: "medical_reviewer",
        location_id: LOCAL_LOCATION_ID,
    }
}

fn general_manager() -> TrustedActorHeaders<'static> {
    TrustedActorHeaders {
        actor_id: "general-manager-1",
        role: "general_manager",
        location_id: LOCAL_LOCATION_ID,
    }
}

fn vaccine_upload_body(uploaded_by_staff_id: &str) -> serde_json::Value {
    json!({
        "pet_id": "00000000-0000-0000-0000-000000000101",
        "customer_id": "00000000-0000-0000-0000-000000000201",
        "filename": "rabies-certificate.txt",
        "mime_type": "text/plain",
        "content": "Rabies vaccine administered 2026-01-15 expires 2027-01-15 for Miso",
        "uploaded_by_staff_id": uploaded_by_staff_id
    })
}

async fn upload_vaccine_document_with_actor(
    app: axum::Router,
    uploaded_by_staff_id: &str,
    trusted_actor: Option<TrustedActorHeaders<'_>>,
) -> (axum_http::StatusCode, serde_json::Value) {
    request_json_on(
        app,
        axum_http::Method::POST,
        "/v1/vaccine-documents/uploads",
        vaccine_upload_body(uploaded_by_staff_id),
        trusted_actor,
    )
    .await
}

async fn data_quality_context(app: axum::Router) -> serde_json::Value {
    let response = app
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::GET)
                .uri(format!(
                    "/v1/agent/context/data-quality-hygiene?location_id={LOCAL_LOCATION_ID}&operating_day=2026-06-17"
                ))
                .header("x-test-auth-actor-id", "general-manager-17")
                .header("x-test-auth-role", "general_manager")
                .header("x-test-auth-location-id", LOCAL_LOCATION_ID)
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("data-quality context request succeeds");
    assert_eq!(response.status(), axum_http::StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    serde_json::from_slice(&body).expect("json context payload")
}

fn data_quality_outcome_body(
    action: &serde_json::Value,
    actor_id_claim: &str,
    persona_claim: &str,
) -> serde_json::Value {
    json!({
        "outcome": "completed",
        "actual_minutes": 9,
        "actor": {"id": actor_id_claim, "persona": persona_claim, "actor_role": persona_claim},
        "feedback": "Prepared source-grounded cleanup task for manager review without touching Gingr.",
        "source_refs": action["source_refs"],
        "issue_refs": action["issue_refs"],
        "reported_resolution_status": "acknowledged",
        "timestamp": "2026-06-17T13:15:00Z",
        "audit": {"correlation_id": "trusted-actor-boundary-test"},
        "requested_side_effects": [],
        "idempotency_key": "trusted-actor-boundary-outcome-1"
    })
}

#[tokio::test]
async fn mutating_routes_reject_missing_trusted_actor_context_even_when_body_names_staff_actor() {
    let app = http::router_with_test_auth_state(http::VaccineDocumentState::default());

    let (upload_status, upload_payload) =
        upload_vaccine_document_with_actor(app.clone(), "front-desk-lead-17", None).await;
    assert_eq!(upload_status, axum_http::StatusCode::UNAUTHORIZED);
    assert_eq!(
        upload_payload["error"]["code"],
        "missing_trusted_actor_context"
    );
    assert_eq!(upload_payload["accepted"], false);

    let context = data_quality_context(app.clone()).await;
    let action = &context["hygiene_actions"][0];
    let (outcome_status, outcome_payload) = request_json_on(
        app,
        axum_http::Method::POST,
        &format!(
            "/v1/data-quality-hygiene/actions/{}/outcome",
            action["id"].as_str().unwrap()
        ),
        data_quality_outcome_body(action, "front-desk-lead-17", "front_desk_lead"),
        None,
    )
    .await;
    assert_eq!(outcome_status, axum_http::StatusCode::UNAUTHORIZED);
    assert_eq!(
        outcome_payload["error"]["code"],
        "missing_trusted_actor_context"
    );
    assert_eq!(outcome_payload["outcome_persisted"], false);
}

#[tokio::test]
async fn outcome_routes_authenticate_before_revealing_request_validation_results() {
    let app = http::router_with_test_auth_state(http::VaccineDocumentState::default());

    let context = data_quality_context(app.clone()).await;
    let hygiene_action = &context["hygiene_actions"][0];
    let mut invalid_hygiene_outcome =
        data_quality_outcome_body(hygiene_action, "front-desk-lead-17", "front_desk_lead");
    invalid_hygiene_outcome["source_refs"] = json!([]);
    let (hygiene_status, hygiene_payload) = request_json_on(
        app.clone(),
        axum_http::Method::POST,
        &format!(
            "/v1/data-quality-hygiene/actions/{}/outcome",
            hygiene_action["id"].as_str().unwrap()
        ),
        invalid_hygiene_outcome,
        None,
    )
    .await;
    assert_eq!(hygiene_status, axum_http::StatusCode::UNAUTHORIZED);
    assert_eq!(
        hygiene_payload["error"]["code"],
        "missing_trusted_actor_context"
    );

    let manager_context = request_json_on(
        app.clone(),
        axum_http::Method::GET,
        &format!(
            "/v1/agent/context/manager-daily-brief?location_id={LOCAL_LOCATION_ID}&operating_day=2026-06-17"
        ),
        json!(null),
        Some(general_manager()),
    )
    .await
    .1;
    let manager_action = &manager_context["manager_brief_actions"][0];
    let (manager_status, manager_payload) = request_json_on(
        app,
        axum_http::Method::POST,
        &format!(
            "/v1/manager-daily-brief/actions/{}/outcome",
            manager_action["id"].as_str().unwrap()
        ),
        json!({
            "outcome": "completed",
            "actual_minutes": 4,
            "actor": {"id": "front-desk-lead-17", "persona": "front_desk_lead"},
            "feedback": "Unauthenticated requests must not learn validation ordering.",
            "source_refs": [],
            "timestamp": "2026-06-17T12:00:00Z",
            "audit": {"correlation_id": "auth-first-contract"},
            "reporting": {"location_id": LOCAL_LOCATION_ID, "operating_day": "2026-06-17"},
            "requested_side_effects": []
        }),
        None,
    )
    .await;
    assert_eq!(manager_status, axum_http::StatusCode::UNAUTHORIZED);
    assert_eq!(
        manager_payload["error"]["code"],
        "missing_trusted_actor_context"
    );
}

#[tokio::test]
async fn every_post_mutation_authenticates_before_json_validation_or_state_access() {
    for uri in [
        "/v1/inquiries",
        "/v1/agent/drafts/manager-daily-brief",
        "/v1/agent/drafts/data-quality-hygiene",
        "/v1/data-quality-hygiene/actions/action-id/outcome",
        "/v1/manager-daily-brief/actions/action-id/outcome",
        "/v1/demo/information-lifespan/run",
        "/v1/vaccine-documents/uploads",
        "/v1/vaccine-documents/review-packets/00000000-0000-0000-0000-000000000001/approve",
        "/v1/vaccine-documents/review-packets/00000000-0000-0000-0000-000000000001/reject",
    ] {
        let response = http::router_with_test_auth_state(http::VaccineDocumentState::default())
            .oneshot(
                axum_http::request::Builder::new()
                    .method(axum_http::Method::POST)
                    .uri(uri)
                    .header(axum_http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{not-json"))
                    .expect("request builds"),
            )
            .await
            .expect("auth-first request succeeds");
        assert_eq!(response.status(), axum_http::StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
async fn trusted_role_cannot_be_replaced_by_a_body_persona_claim() {
    let app = http::router_with_test_auth_state(http::VaccineDocumentState::default());
    let context = data_quality_context(app.clone()).await;
    let action = &context["hygiene_actions"][0];
    let (status, payload) = request_json_on(
        app,
        axum_http::Method::POST,
        &format!(
            "/v1/data-quality-hygiene/actions/{}/outcome",
            action["id"].as_str().unwrap()
        ),
        data_quality_outcome_body(action, "general-manager-1", "front_desk_lead"),
        Some(general_manager()),
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::FORBIDDEN);
    assert_eq!(payload["error"]["code"], "body_actor_claim_mismatch");
    assert_eq!(payload["outcome_persisted"], false);
}

#[tokio::test]
async fn data_quality_draft_requires_trusted_actor_before_json_validation() {
    let response = http::router_with_test_auth_state(http::VaccineDocumentState::default())
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::POST)
                .uri("/v1/agent/drafts/data-quality-hygiene")
                .header(axum_http::header::CONTENT_TYPE, "application/json")
                .body(Body::from("{}"))
                .expect("request builds"),
        )
        .await
        .expect("auth-first draft request succeeds");
    assert_eq!(response.status(), axum_http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn inquiry_intake_rejects_missing_trusted_actor_context_before_mutating_runtime_state() {
    let (status, payload) = request_json_on(
        http::router_with_state(http::VaccineDocumentState::default()),
        axum_http::Method::POST,
        "/v1/inquiries",
        json!({
            "source_event_key": "unauthenticated-inquiry",
            "location_id": LOCAL_LOCATION_ID,
            "customer": {"full_name": "Casey", "email": "casey@example.test"},
            "pet": {"name": "Miso", "species": "dog"},
            "service": "boarding",
            "message": "Need boarding details."
        }),
        None,
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::UNAUTHORIZED);
    assert_eq!(payload["error"]["code"], "missing_trusted_actor_context");
}

#[tokio::test]
async fn inquiry_intake_rejects_explicit_nil_location_identity() {
    let (status, payload) = request_json_on(
        http::router_with_test_auth_state(http::VaccineDocumentState::default()),
        axum_http::Method::POST,
        "/v1/inquiries",
        json!({
            "source_event_key": "nil-location-inquiry",
            "location_id": "00000000-0000-0000-0000-000000000000",
            "customer": {"full_name": "Casey", "email": "casey@example.test"},
            "pet": {"name": "Miso", "species": "dog"},
            "service": "boarding",
            "message": "Need boarding details.",
            "contact_attempts": []
        }),
        Some(front_desk_lead()),
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["error"]["code"], "invalid_location_id");
}

#[tokio::test]
async fn production_router_never_promotes_client_supplied_test_headers_into_trusted_identity() {
    let app = http::router_with_state(http::VaccineDocumentState::default());

    let (status, payload) =
        upload_vaccine_document_with_actor(app, "medical-reviewer-42", Some(medical_reviewer()))
            .await;

    assert_eq!(status, axum_http::StatusCode::UNAUTHORIZED);
    assert_eq!(payload["error"]["code"], "missing_trusted_actor_context");
    assert_eq!(payload["accepted"], false);
}

#[tokio::test]
async fn trusted_actor_context_must_match_body_actor_claim_before_review_mutation() {
    let app = http::router_with_test_auth_state(http::VaccineDocumentState::default());

    let (forged_upload_status, forged_upload_payload) = upload_vaccine_document_with_actor(
        app.clone(),
        "forged-staff-id-from-body",
        Some(front_desk_lead()),
    )
    .await;
    assert_eq!(forged_upload_status, axum_http::StatusCode::FORBIDDEN);
    assert_eq!(
        forged_upload_payload["error"]["code"],
        "body_actor_claim_mismatch"
    );

    let (upload_status, upload_payload) = upload_vaccine_document_with_actor(
        app.clone(),
        "medical-reviewer-42",
        Some(medical_reviewer()),
    )
    .await;
    assert_eq!(upload_status, axum_http::StatusCode::CREATED);
    let review_packet_id = upload_payload["review_packet"]["id"].as_str().unwrap();

    let (forged_review_status, forged_review_payload) = request_json_on(
        app,
        axum_http::Method::POST,
        &format!("/v1/vaccine-documents/review-packets/{review_packet_id}/approve"),
        json!({
            "reviewed_by_staff_id": "forged-reviewer-from-body",
            "reason": "Body actor id must remain an unauthoritative claim."
        }),
        Some(medical_reviewer()),
    )
    .await;
    assert_eq!(forged_review_status, axum_http::StatusCode::FORBIDDEN);
    assert_eq!(
        forged_review_payload["error"]["code"],
        "body_actor_claim_mismatch"
    );
}

#[tokio::test]
async fn mutating_routes_reject_wrong_location_or_role_from_trusted_actor_context() {
    let wrong_location_actor = TrustedActorHeaders {
        actor_id: "front-desk-lead-17",
        role: "front_desk_lead",
        location_id: "11111111-1111-1111-1111-111111111111",
    };
    let app = http::router_with_test_auth_state(http::VaccineDocumentState::default());
    let context = data_quality_context(app.clone()).await;
    let action = &context["hygiene_actions"][0];

    let (wrong_location_status, wrong_location_payload) = request_json_on(
        app.clone(),
        axum_http::Method::POST,
        &format!(
            "/v1/data-quality-hygiene/actions/{}/outcome",
            action["id"].as_str().unwrap()
        ),
        data_quality_outcome_body(action, "front-desk-lead-17", "front_desk_lead"),
        Some(wrong_location_actor),
    )
    .await;
    assert_eq!(wrong_location_status, axum_http::StatusCode::FORBIDDEN);
    assert_eq!(
        wrong_location_payload["error"]["code"],
        "actor_location_not_authorized"
    );
    assert_eq!(wrong_location_payload["outcome_persisted"], false);

    let unauthorized_role = TrustedActorHeaders {
        actor_id: "front-desk-lead-17",
        role: "guest_care_attendant",
        location_id: LOCAL_LOCATION_ID,
    };
    let (wrong_role_status, wrong_role_payload) =
        upload_vaccine_document_with_actor(app, "front-desk-lead-17", Some(unauthorized_role))
            .await;
    assert_eq!(wrong_role_status, axum_http::StatusCode::FORBIDDEN);
    assert_eq!(
        wrong_role_payload["error"]["code"],
        "actor_role_not_authorized"
    );
}

#[tokio::test]
async fn staff_sensitive_reads_require_trusted_actor_context_and_reject_role_claims() {
    let app = http::router_with_test_auth_state(http::VaccineDocumentState::default());

    let (staff_status, staff_payload) = request_json_on(
        app.clone(),
        axum_http::Method::GET,
        "/v1/staff/inquiries",
        json!(null),
        None,
    )
    .await;
    assert_eq!(staff_status, axum_http::StatusCode::UNAUTHORIZED);
    assert_eq!(
        staff_payload["error"]["code"],
        "missing_trusted_actor_context"
    );

    let permissioned_uri = format!(
        "/v1/agent/context/permissioned-knowledge?location_id={LOCAL_LOCATION_ID}&service=boarding&role=general_manager&section=check-in.required-documents"
    );
    let (missing_status, missing_payload) = request_json_on(
        app.clone(),
        axum_http::Method::GET,
        &permissioned_uri,
        json!(null),
        None,
    )
    .await;
    assert_eq!(missing_status, axum_http::StatusCode::UNAUTHORIZED);
    assert_eq!(
        missing_payload["error"]["code"],
        "missing_trusted_actor_context"
    );

    let (forged_status, forged_payload) = request_json_on(
        app,
        axum_http::Method::GET,
        &permissioned_uri,
        json!(null),
        Some(front_desk_lead()),
    )
    .await;
    assert_eq!(forged_status, axum_http::StatusCode::FORBIDDEN);
    assert_eq!(forged_payload["error"]["code"], "body_actor_claim_mismatch");
}

#[tokio::test]
async fn operational_reads_reject_missing_trusted_actor_context() {
    let app = http::router_with_test_auth_state(http::VaccineDocumentState::default());
    let routes = [
        "/v1/read-models/source-quality-backlog",
        "/v1/demo/information-lifespan/info-lifespan-demo-2026-06-29/report",
        "/v1/ops/metrics/summary",
        "/v1/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
        "/v1/agent/context/site-finance",
        "/v1/data-quality-hygiene/outcomes/summary?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
        "/v1/agent/context/manager-daily-brief?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
    ];

    for route in routes {
        let (status, payload) = request_json_on(
            app.clone(),
            axum_http::Method::GET,
            route,
            json!(null),
            None,
        )
        .await;

        assert_eq!(status, axum_http::StatusCode::UNAUTHORIZED, "{route}");
        assert_eq!(
            payload["error"]["code"], "missing_trusted_actor_context",
            "{route}"
        );
    }
}
