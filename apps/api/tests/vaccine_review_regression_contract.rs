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
    actor_id: &str,
) -> (axum_http::StatusCode, serde_json::Value) {
    let response = app
        .oneshot(
            axum_http::request::Builder::new()
                .method(method)
                .uri(uri)
                .header(axum_http::header::CONTENT_TYPE, "application/json")
                .header("x-test-auth-actor-id", actor_id)
                .header("x-test-auth-role", "medical_reviewer")
                .header("x-test-auth-location-id", LOCAL_LOCATION_ID)
                .body(Body::from(body.to_string()))
                .expect("request builds"),
        )
        .await
        .expect("api request should return a typed response instead of panicking");
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

async fn upload_vaccine_document(app: axum::Router) -> serde_json::Value {
    let (status, payload) = request_json_on(
        app,
        axum_http::Method::POST,
        "/vaccine-documents/uploads",
        json!({
            "pet_id": "00000000-0000-0000-0000-000000000101",
            "customer_id": "00000000-0000-0000-0000-000000000201",
            "filename": "rabies-certificate.txt",
            "mime_type": "text/plain",
            "content": "Rabies vaccine administered 2026-01-15 expires 2027-01-15 for Miso",
            "uploaded_by_staff_id": "medical-reviewer-42"
        }),
        "medical-reviewer-42",
    )
    .await;
    assert_eq!(status, axum_http::StatusCode::CREATED);
    payload
}

#[tokio::test]
async fn vaccine_review_packet_rejects_repeated_or_conflicting_decisions_without_overwriting_eligibility()
 {
    let app = http::router_with_test_auth_state(http::VaccineDocumentState::default());
    let upload = upload_vaccine_document(app.clone()).await;
    let review_packet_id = upload["review_packet"]["id"]
        .as_str()
        .expect("review packet id");

    let (approved_status, approved) = request_json_on(
        app.clone(),
        axum_http::Method::POST,
        &format!("/vaccine-documents/review-packets/{review_packet_id}/approve"),
        json!({
            "reviewed_by_staff_id": "medical-reviewer-42",
            "reason": "source document matches rabies policy"
        }),
        "medical-reviewer-42",
    )
    .await;
    assert_eq!(approved_status, axum_http::StatusCode::OK);
    assert_eq!(approved["review_packet"]["status"], "approved");
    assert_eq!(approved["eligibility"]["rabies_current"], true);

    let (conflict_status, conflict) = request_json_on(
        app,
        axum_http::Method::POST,
        &format!("/vaccine-documents/review-packets/{review_packet_id}/reject"),
        json!({
            "reviewed_by_staff_id": "medical-reviewer-42",
            "reason": "second reviewer tries to reverse the already consumed packet"
        }),
        "medical-reviewer-42",
    )
    .await;

    assert_eq!(conflict_status, axum_http::StatusCode::CONFLICT);
    assert_eq!(conflict["accepted"], false);
    assert_eq!(
        conflict["error"]["code"],
        "vaccine_review_packet_already_decided"
    );
    assert_eq!(conflict["existing_decision"], "approved");
    assert_eq!(conflict["attempted_decision"], "rejected");
    assert_eq!(conflict["eligibility"]["rabies_current"], true);
    assert_eq!(conflict["review_packet"]["status"], "approved");
}

#[tokio::test]
async fn unknown_vaccine_review_packet_returns_typed_not_found_error_instead_of_panic() {
    let app = http::router_with_test_auth_state(http::VaccineDocumentState::default());

    let (status, payload) = request_json_on(
        app,
        axum_http::Method::POST,
        "/vaccine-documents/review-packets/00000000-0000-0000-0000-00000000dead/approve",
        json!({
            "reviewed_by_staff_id": "medical-reviewer-42",
            "reason": "unknown packet should not panic"
        }),
        "medical-reviewer-42",
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::NOT_FOUND);
    assert_eq!(payload["accepted"], false);
    assert_eq!(payload["error"]["code"], "vaccine_review_packet_not_found");
    assert_eq!(
        payload["review_packet_id"],
        "00000000-0000-0000-0000-00000000dead"
    );
}

#[test]
fn vaccine_review_transition_preflights_every_linked_record_before_its_first_mutation() {
    let source = include_str!("../src/http.rs");
    let transition = source
        .split("fn apply_vaccine_review_decision(")
        .nth(1)
        .expect("review transition exists");
    let first_mutation = transition
        .find("self.review_packets\n            .get_mut")
        .expect("review transition eventually mutates the packet");

    for required_preflight in [
        "self.documents.get(&document_id)",
        "self.extractions.get(&document_id)",
        "self.eligibility.get(&pet_id)",
    ] {
        let position = transition
            .find(required_preflight)
            .expect("linked workflow record must be validated before mutation");
        assert!(position < first_mutation);
    }
}
