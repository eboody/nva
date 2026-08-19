use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::{
    http,
    public_contract::{
        DataQualityAction, DataQualityCandidate, DataQualityHygieneDraftSubmissionRequest,
        DataQualityHygieneOutcomeCaptureRequest, DataQualityHygieneSubmittedAction,
        DataQualityIssue, ReportedLaborEstimateEvidence,
    },
};
use serde_json::{Value, json};
use tower::ServiceExt;

const LOCATION_ID: &str = "00c0ffee-0000-0000-0000-000000000001";

#[test]
fn data_quality_public_dto_debug_redacts_issue_action_and_lineage_details() {
    let issue = DataQualityIssue {
        kind: "private-kind-8675309".to_owned(),
        severity: "private-severity-8675309".to_owned(),
        workflow_blocking: true,
        detail: Some("private-issue-detail-8675309".to_owned()),
        source_refs: vec![],
    };
    let candidate = DataQualityCandidate {
        id: "private-candidate-8675309".to_owned(),
        kind: "private-candidate-kind-8675309".to_owned(),
        issue: issue.clone(),
        source_refs: vec![],
        source_freshness: "private-freshness-8675309".to_owned(),
        sensitivity: "private-sensitivity-8675309".to_owned(),
    };
    let action = DataQualityAction {
        id: "private-action-8675309".to_owned(),
        kind: "private-action-kind-8675309".to_owned(),
        priority: "private-priority-8675309".to_owned(),
        owner_persona: "private-owner-8675309".to_owned(),
        removed_manual_work: "private-manual-work-8675309".to_owned(),
        rationale: "private-rationale-8675309".to_owned(),
        source_refs: vec![],
        issue_refs: vec!["private-issue-ref-8675309".to_owned()],
        review_gates: vec!["private-review-gate-8675309".to_owned()],
        labor_impact: ReportedLaborEstimateEvidence {
            before_minutes: 12,
            after_minutes: 8,
            reported_estimated_minutes_difference: 4,
        },
        live_side_effects_allowed: false,
    };
    let submitted = DataQualityHygieneSubmittedAction {
        action_id: "private-submitted-action-8675309".to_owned(),
        kind: "private-submitted-kind-8675309".to_owned(),
        source_refs: vec![],
        issue_refs: vec!["private-submitted-issue-8675309".to_owned()],
        review_gates: vec!["private-submitted-gate-8675309".to_owned()],
        requested_side_effects: vec!["private-side-effect-8675309".to_owned()],
        attempted_ambiguity_resolution: true,
    };
    let submission = DataQualityHygieneDraftSubmissionRequest {
        context_packet_id: "private-context-8675309".to_owned(),
        correlation_id: "private-correlation-8675309".to_owned(),
        actions: vec![submitted.clone()],
        idempotency_key: Some("private-idempotency-8675309".to_owned()),
    };

    let debug = format!("{issue:?}{candidate:?}{action:?}{submitted:?}{submission:?}");
    for marker in [
        "DataQualityIssue([REDACTED])",
        "DataQualityCandidate([REDACTED])",
        "DataQualityAction([REDACTED])",
        "DataQualityHygieneSubmittedAction([REDACTED])",
        "DataQualityHygieneDraftSubmissionRequest([REDACTED])",
    ] {
        assert!(debug.contains(marker), "missing redaction marker {marker}");
    }
    for private in [
        "private-issue-detail-8675309",
        "private-action-8675309",
        "private-context-8675309",
        "private-correlation-8675309",
        "private-side-effect-8675309",
    ] {
        assert!(!debug.contains(private));
    }
}

async fn request(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Value,
) -> (axum_http::StatusCode, Value) {
    let response = app
        .oneshot(
            axum_http::Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", "application/json")
                .header("x-test-auth-actor-id", "front-desk-lead-17")
                .header("x-test-auth-role", "front_desk_lead")
                .header("x-test-auth-location-id", LOCATION_ID)
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

async fn hygiene_action(app: axum::Router) -> Value {
    let (status, context) = request(
        app,
        "GET",
        &format!(
            "/v1/agent/context/data-quality-hygiene?location_id={LOCATION_ID}&operating_day=2026-06-17"
        ),
        Value::Null,
    )
    .await;
    assert_eq!(status, axum_http::StatusCode::OK);
    context["hygiene_actions"][0].clone()
}

fn outcome(action: &Value) -> Value {
    json!({
        "outcome": "completed",
        "actual_minutes": 6,
        "actor": {
            "id": "front-desk-lead-17",
            "persona": "front_desk_lead",
            "actor_role": "front_desk_lead"
        },
        "feedback": "Reviewed the source-backed issue.",
        "source_refs": action["source_refs"],
        "issue_refs": action["issue_refs"],
        "reported_resolution_status": "acknowledged",
        "timestamp": "2026-06-17T12:15:00Z",
        "audit": {"correlation_id": "data-quality-hygiene:strict-contract"},
        "requested_side_effects": [],
        "idempotency_key": "dqh-outcome-contract-0001"
    })
}

#[test]
fn canonical_outcome_request_is_closed_and_requires_documented_arrays() {
    let action = json!({
        "source_refs": [{
            "system": "gingr",
            "record_type": "reservation",
            "record_id": "r-1",
            "observed_at": "2026-06-17T00:00:00Z",
            "adapter_version": "readonly-v1"
        }],
        "issue_refs": ["issue-1"]
    });
    let valid = outcome(&action);
    assert!(
        serde_json::from_value::<DataQualityHygieneOutcomeCaptureRequest>(valid.clone()).is_ok()
    );
    let mut malformed_timestamp = valid.clone();
    malformed_timestamp["timestamp"] = json!("2026-06-17 12:15:00");
    assert!(
        serde_json::from_value::<DataQualityHygieneOutcomeCaptureRequest>(malformed_timestamp)
            .is_err(),
        "OpenAPI date-time fields must reject non-RFC3339 input at runtime"
    );

    for missing in ["source_refs", "issue_refs", "requested_side_effects"] {
        let mut payload = valid.clone();
        payload.as_object_mut().unwrap().remove(missing);
        assert!(
            serde_json::from_value::<DataQualityHygieneOutcomeCaptureRequest>(payload).is_err(),
            "{missing} must be required"
        );
    }

    for (path, field) in [
        ("request", "surprise"),
        ("actor", "trusted_by_client"),
        ("audit", "raw_payload"),
    ] {
        let mut payload = valid.clone();
        match path {
            "request" => payload[field] = json!(true),
            nested => payload[nested][field] = json!(true),
        }
        assert!(
            serde_json::from_value::<DataQualityHygieneOutcomeCaptureRequest>(payload).is_err(),
            "unknown {path} field must fail closed"
        );
    }
}

#[test]
fn canonical_outcome_request_rejects_open_codes_and_invalid_idempotency_keys() {
    let action = json!({"source_refs": [{
        "system": "gingr", "record_type": "reservation", "record_id": "r-1",
        "observed_at": "2026-06-17T00:00:00Z", "adapter_version": "readonly-v1"
    }], "issue_refs": ["issue-1"]});
    let valid = outcome(&action);
    for (field, invalid) in [
        ("outcome", "mostly_completed"),
        ("reported_resolution_status", "maybe_fixed"),
    ] {
        let mut payload = valid.clone();
        payload[field] = json!(invalid);
        assert!(
            serde_json::from_value::<DataQualityHygieneOutcomeCaptureRequest>(payload).is_err()
        );
    }
    let mut invalid_persona = valid.clone();
    invalid_persona["actor"]["persona"] = json!("regional_superhero");
    assert!(
        serde_json::from_value::<DataQualityHygieneOutcomeCaptureRequest>(invalid_persona).is_err()
    );

    for invalid_key in [
        "",
        " leading-space",
        "contains secret whitespace",
        &"x".repeat(129),
    ] {
        let mut payload = valid.clone();
        payload["idempotency_key"] = json!(invalid_key);
        assert!(
            serde_json::from_value::<DataQualityHygieneOutcomeCaptureRequest>(payload).is_err()
        );
    }
}

#[tokio::test]
async fn outcome_capture_consumes_actor_role_and_is_really_idempotent() {
    let app = http::router_with_test_auth_state(Default::default());
    let action = hygiene_action(app.clone()).await;
    let uri = format!(
        "/v1/data-quality-hygiene/actions/{}/outcome",
        action["id"].as_str().unwrap()
    );
    let payload = outcome(&action);

    let (created_status, created) = request(app.clone(), "POST", &uri, payload.clone()).await;
    assert_eq!(created_status, axum_http::StatusCode::CREATED);
    assert_eq!(created["outcome_persisted"], true);
    assert_eq!(created["outcome_record"]["outcome"], "reported_completed");
    assert_eq!(
        created["outcome_record"]["authority_disposition"],
        "needs_review"
    );
    assert_eq!(created["outcome_record"]["claimable"], false);
    let persisted_count = created["reported_labor_evidence"]["persisted_outcome_count"].clone();
    let projection_count = created["reported_labor_evidence"]["persisted_projection_count"].clone();

    let (replay_status, replay) = request(app.clone(), "POST", &uri, payload.clone()).await;
    assert_eq!(replay_status, axum_http::StatusCode::OK);
    assert_eq!(replay["idempotent_replay"], true);
    assert_eq!(
        replay["reported_labor_evidence"]["persisted_outcome_count"],
        persisted_count
    );
    assert_eq!(
        replay["reported_labor_evidence"]["persisted_projection_count"],
        projection_count
    );

    let mut drift = payload;
    drift["feedback"] = json!("Consequentially changed feedback.");
    let (conflict_status, conflict) = request(app.clone(), "POST", &uri, drift).await;
    assert_eq!(conflict_status, axum_http::StatusCode::CONFLICT);
    assert_eq!(conflict["error"]["code"], "idempotency_conflict");

    let mut forged_role = outcome(&action);
    forged_role["idempotency_key"] = json!("dqh-outcome-contract-0002");
    forged_role["actor"]["actor_role"] = json!("general_manager");
    let (forbidden_status, forbidden) = request(app, "POST", &uri, forged_role).await;
    assert_eq!(forbidden_status, axum_http::StatusCode::FORBIDDEN);
    assert_eq!(forbidden["outcome_persisted"], false);
}

fn inquiry(key: &str) -> Value {
    json!({
        "source_event_key": key,
        "location_id": LOCATION_ID,
        "customer": {"full_name": "Avery Chen", "email": "avery@example.test", "phone": null},
        "pet": {"name": "Miso", "species": "dog"},
        "service": "boarding",
        "requested_dates": {"start": "2026-07-03", "end": "2026-07-07"},
        "message": "Need boarding information.",
        "contact_attempts": [{
            "attempted_at": "2026-07-03T14:05:00Z",
            "channel": "email",
            "purpose": "lead_response",
            "outcome": "drafted_for_review",
            "message_ref": "message-draft:1"
        }],
        "simulated_conversion": {
            "reservation_id": "reservation:simulated-1",
            "converted_at": "2026-07-03T15:30:00Z",
            "attribution_source": "website_form"
        }
    })
}

#[tokio::test]
async fn inquiry_provenance_is_all_or_nothing_and_never_manufactured() {
    let app = http::router_with_test_auth_state(Default::default());
    let no_provenance = inquiry("inquiry-no-provenance");
    let (created_status, created) =
        request(app.clone(), "POST", "/v1/inquiries", no_provenance).await;
    assert_eq!(created_status, axum_http::StatusCode::CREATED);
    assert!(
        created.get("provenance").is_none(),
        "no source receipt may be invented"
    );

    let mut partial = inquiry("inquiry-partial-provenance");
    partial["source_system"] = json!("mock_gingr");
    let (partial_status, partial_response) = request(app, "POST", "/v1/inquiries", partial).await;
    assert_eq!(partial_status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        partial_response["classification"],
        "incomplete_source_provenance"
    );
}

#[tokio::test]
async fn inquiry_replay_matching_covers_complete_provenance_attempts_and_conversion() {
    let app = http::router_with_test_auth_state(Default::default());
    let mut complete = inquiry("inquiry-complete-replay");
    complete["source_system"] = json!("mock_gingr");
    complete["provider_model_path"] = json!("gingr::webhook::LeadInquirySubmitted");
    complete["raw_payload_ref"] = json!("fixture://lead-1.json");
    complete["received_at"] = json!("2026-07-03T14:00:00Z");

    let (created_status, _) = request(app.clone(), "POST", "/v1/inquiries", complete.clone()).await;
    assert_eq!(created_status, axum_http::StatusCode::CREATED);

    for mutate in ["provenance", "attempt", "conversion"] {
        let mut drift = complete.clone();
        match mutate {
            "provenance" => drift["raw_payload_ref"] = json!("fixture://changed.json"),
            "attempt" => drift["contact_attempts"][0]["outcome"] = json!("changed"),
            "conversion" => {
                drift["simulated_conversion"]["reservation_id"] = json!("reservation:changed")
            }
            _ => unreachable!(),
        }
        let (status, response) = request(app.clone(), "POST", "/v1/inquiries", drift).await;
        assert_eq!(
            status,
            axum_http::StatusCode::CONFLICT,
            "{mutate} must be consequential"
        );
        assert_eq!(response["classification"], "idempotency_payload_drift");
    }
}

#[test]
fn data_quality_draft_rejects_unknown_fields_at_root_and_nested_action() {
    let valid = json!({
        "context_packet_id": "context-1",
        "correlation_id": "correlation-1",
        "actions": [{
            "action_id": "action-1",
            "kind": "review_stale_source",
            "source_refs": [],
            "issue_refs": [],
            "review_gates": [],
            "requested_side_effects": [],
            "attempted_ambiguity_resolution": false
        }],
        "idempotency_key": "draft-key-1"
    });
    assert!(
        serde_json::from_value::<DataQualityHygieneDraftSubmissionRequest>(valid.clone()).is_ok()
    );

    let mut root_extra = valid.clone();
    root_extra["ignored_contract_field"] = json!(true);
    assert!(
        serde_json::from_value::<DataQualityHygieneDraftSubmissionRequest>(root_extra).is_err()
    );

    let mut nested_extra = valid;
    nested_extra["actions"][0]["trusted_by_client"] = json!(true);
    assert!(
        serde_json::from_value::<DataQualityHygieneDraftSubmissionRequest>(nested_extra).is_err()
    );
}

#[tokio::test]
async fn inquiry_unknown_fields_fail_closed_at_every_public_record_boundary() {
    let app = http::router_with_test_auth_state(Default::default());
    for path in [
        "request",
        "customer",
        "pet",
        "requested_dates",
        "contact_attempt",
        "conversion",
    ] {
        let mut payload = inquiry(&format!("unknown-{path}"));
        match path {
            "request" => payload["surprise"] = json!(true),
            "contact_attempt" => payload["contact_attempts"][0]["surprise"] = json!(true),
            "conversion" => payload["simulated_conversion"]["surprise"] = json!(true),
            nested => payload[nested]["surprise"] = json!(true),
        }
        let (status, _) = request(app.clone(), "POST", "/v1/inquiries", payload).await;
        assert_eq!(
            status,
            axum_http::StatusCode::UNPROCESSABLE_ENTITY,
            "unknown {path} field"
        );
    }
}
