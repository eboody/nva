use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::http;
use serde_json::json;
use tower::ServiceExt;

async fn json_request(
    method: axum_http::Method,
    uri: &str,
    payload: serde_json::Value,
) -> (axum_http::StatusCode, serde_json::Value) {
    let response = http::router()
        .oneshot(
            axum_http::request::Builder::new()
                .method(method)
                .uri(uri)
                .header(axum_http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload.to_string()))
                .expect("request builds"),
        )
        .await
        .expect("workflow request succeeds");
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("json workflow payload");
    (status, payload)
}

#[tokio::test]
async fn vaccine_document_upload_persists_extraction_review_and_audit_evidence() {
    let (status, payload) = json_request(
        axum_http::Method::POST,
        "/vaccine-documents/uploads",
        json!({
            "pet_id": "00000000-0000-0000-0000-000000000101",
            "customer_id": "00000000-0000-0000-0000-000000000201",
            "filename": "  rabies-certificate.txt  ",
            "mime_type": "text/plain",
            "content": "Rabies vaccine administered 2026-01-15 expires 2027-01-15 for Miso",
            "uploaded_by_staff_id": "front-desk-demo"
        }),
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::CREATED);
    assert_eq!(payload["document"]["classification"], "vaccine_proof");
    assert_eq!(
        payload["document"]["verification_status"],
        "awaiting_review"
    );
    assert!(
        payload["document"]["storage_key"]
            .as_str()
            .unwrap()
            .contains("vaccine-documents/pets/00000000-0000-0000-0000-000000000101/")
    );
    assert_eq!(
        payload["extraction"]["schema_version"],
        "vaccine_extraction.v1"
    );
    assert_eq!(payload["extraction"]["vaccine_name"], "Rabies");
    assert_eq!(payload["extraction"]["confidence"], 0.78);
    assert_eq!(payload["vaccine_record"]["status"], "pending_review");
    assert_eq!(payload["review_packet"]["gate"], "medical_document_review");
    assert_eq!(payload["eligibility"]["rabies_current"], false);
    assert!(
        payload["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["action"] == "document.received")
    );
    assert!(
        payload["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["action"] == "vaccine_record.review_requested")
    );
}

#[tokio::test]
async fn staff_approval_updates_vaccine_eligibility_and_preserves_document_audit_lineage() {
    let (_, upload) = json_request(
        axum_http::Method::POST,
        "/vaccine-documents/uploads",
        json!({
            "pet_id": "00000000-0000-0000-0000-000000000101",
            "customer_id": "00000000-0000-0000-0000-000000000201",
            "filename": "rabies-certificate.txt",
            "mime_type": "text/plain",
            "content": "Rabies vaccine administered 2026-01-15 expires 2027-01-15 for Miso",
            "uploaded_by_staff_id": "front-desk-demo"
        }),
    )
    .await;
    let review_packet_id = upload["review_packet"]["id"]
        .as_str()
        .expect("review packet id");

    let (status, approved) = json_request(
        axum_http::Method::POST,
        &format!("/vaccine-documents/review-packets/{review_packet_id}/approve"),
        json!({ "reviewed_by_staff_id": "manager-demo" }),
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::OK);
    assert_eq!(approved["document"]["verification_status"], "verified");
    assert_eq!(approved["vaccine_record"]["status"], "verified_current");
    assert_eq!(approved["eligibility"]["rabies_current"], true);
    assert_eq!(approved["approval"]["status"], "approved");
    assert_eq!(approved["document"]["id"], upload["document"]["id"]);
    assert!(
        approved["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["action"] == "approval.decision.recorded")
    );
    assert!(
        approved["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["action"] == "pet.eligibility.updated")
    );
}

#[tokio::test]
async fn staff_rejection_keeps_pet_ineligible_and_marks_extracted_record_rejected() {
    let (_, upload) = json_request(
        axum_http::Method::POST,
        "/vaccine-documents/uploads",
        json!({
            "pet_id": "00000000-0000-0000-0000-000000000101",
            "customer_id": "00000000-0000-0000-0000-000000000201",
            "filename": "unclear-rabies.txt",
            "mime_type": "text/plain",
            "content": "unclear vaccine paperwork maybe expired",
            "uploaded_by_staff_id": "front-desk-demo"
        }),
    )
    .await;
    let review_packet_id = upload["review_packet"]["id"]
        .as_str()
        .expect("review packet id");

    let (status, rejected) = json_request(
        axum_http::Method::POST,
        &format!("/vaccine-documents/review-packets/{review_packet_id}/reject"),
        json!({ "reviewed_by_staff_id": "manager-demo", "reason": "name/date mismatch" }),
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::OK);
    assert_eq!(rejected["document"]["verification_status"], "rejected");
    assert_eq!(rejected["vaccine_record"]["status"], "rejected");
    assert_eq!(rejected["eligibility"]["rabies_current"], false);
    assert_eq!(rejected["approval"]["status"], "rejected");
}
