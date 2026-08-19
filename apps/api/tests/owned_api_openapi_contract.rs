use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::{http, public_contract};
use serde_json::{Value, json};
use tower::ServiceExt;

const OPENAPI: &str = include_str!("../openapi/owned-operations-v1.openapi.json");

fn openapi_field_is_nullable(schema: &Value) -> bool {
    schema["type"]
        .as_array()
        .is_some_and(|types| types.iter().any(|kind| kind == "null"))
}

async fn get_json(uri: &str) -> (axum_http::StatusCode, Value) {
    let response = http::router_with_test_auth_state(http::VaccineDocumentState::default())
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::GET)
                .uri(uri)
                .header("x-test-auth-actor-id", "general-manager-17")
                .header("x-test-auth-role", "general_manager")
                .header(
                    "x-test-auth-location-id",
                    "00c0ffee-0000-0000-0000-000000000001",
                )
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("owned v1 get request succeeds");

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload = serde_json::from_slice(&body).expect("json response payload");

    (status, payload)
}

async fn post_json(uri: &str, body: Value) -> (axum_http::StatusCode, Value) {
    let trusted_actor = body["actor"]["id"]
        .as_str()
        .map(|actor_id| (actor_id, "front_desk_lead"))
        .or_else(|| {
            body["submitted_by"]
                .as_str()
                .map(|actor_id| (actor_id, "general_manager"))
        })
        .or_else(|| {
            (body.get("context_packet_id").is_some() && body.get("actions").is_some())
                .then_some(("front-desk-lead-17", "front_desk_lead"))
        })
        .unwrap_or(("general-manager-17", "general_manager"));
    let builder = axum_http::request::Builder::new()
        .method(axum_http::Method::POST)
        .uri(uri)
        .header(axum_http::header::CONTENT_TYPE, "application/json")
        .header("x-test-auth-actor-id", trusted_actor.0)
        .header("x-test-auth-role", trusted_actor.1)
        .header(
            "x-test-auth-location-id",
            "00c0ffee-0000-0000-0000-000000000001",
        );

    let response = http::router_with_test_auth_state(http::VaccineDocumentState::default())
        .oneshot(
            builder
                .body(Body::from(body.to_string()))
                .expect("request builds"),
        )
        .await
        .expect("owned v1 post request succeeds");

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload = serde_json::from_slice(&body).expect("json response payload");

    (status, payload)
}

#[test]
fn checked_openapi_artifact_names_owned_v1_operations_and_safe_schemas() {
    let spec: Value = serde_json::from_str(OPENAPI).expect("checked OpenAPI json parses");

    assert_eq!(spec["openapi"], "3.1.0");
    assert_eq!(
        spec["info"]["title"],
        "NVA Pet Resorts Owned Operations API"
    );
    assert_eq!(spec["info"]["version"], "0.1.0");
    assert!(
        !OPENAPI.contains("information_lifespan_trace.v0"),
        "canonical v1 OpenAPI must not advertise a v0 information-lifespan schema tag"
    );
    assert_eq!(
        spec["components"]["schemas"]["InformationLifespanTraceSchemaVersion"]["enum"],
        json!(["information_lifespan_trace.v1"])
    );

    let paths = spec["paths"].as_object().expect("paths object");
    for route in [
        "/v1/healthz",
        "/v1/readyz",
        "/v1/ops/metrics/summary",
        "/v1/agent/context/manager-daily-brief",
        "/v1/agent/context/data-quality-hygiene",
        "/v1/agent/drafts/data-quality-hygiene",
        "/v1/data-quality-hygiene/actions/{action_id}/outcome",
        "/v1/data-quality-hygiene/outcomes/summary",
        "/v1/read-models/source-quality-backlog",
    ] {
        assert!(paths.contains_key(route), "missing v1 route {route}");
    }

    let schemas = spec["components"]["schemas"]
        .as_object()
        .expect("component schemas object");
    for schema_name in [
        "ApiContractMetadata",
        "RequestMetadata",
        "SourceRef",
        "ReviewGateRef",
        "BlockedAction",
        "AuditRef",
        "ErrorEnvelope",
        "DataQualityHygieneContextResponse",
        "DataQualityHygieneDraftSubmissionRequest",
        "DataQualityHygieneDraftSubmissionResponse",
        "DataQualityHygieneOutcomeCaptureRequest",
        "DataQualityHygieneOutcomeCaptureResponse",
        "DataQualityHygieneOutcomeSummaryResponse",
        "ManagerDailyBriefContextResponse",
        "SourceQualityBacklogResponse",
        "ReadModelDatabaseStatus",
        "ReadinessResponse",
        "OpsMetricsSummaryResponse",
    ] {
        assert!(
            schemas.contains_key(schema_name),
            "missing schema {schema_name}"
        );
    }

    assert_eq!(
        schemas["ApiContractMetadata"]["properties"]["provider_boundary"]["const"],
        "evidence_refs_only"
    );
    assert_eq!(
        schemas["ApiContractMetadata"]["properties"]["live_side_effects"]["const"],
        "disabled"
    );
    assert_eq!(
        schemas["ManagerDailyBriefOutcomeReporting"]["properties"]["location_id"]["format"],
        "uuid"
    );
    assert_eq!(
        schemas["ManagerDailyBriefOutcomeReporting"]["properties"]["operating_day"]["format"],
        "date"
    );
    assert_eq!(
        schemas["ManagerDailyBriefOutcomeAudit"]["properties"]["correlation_id"]["minLength"],
        1
    );
}

#[test]
fn data_quality_outcome_response_has_closed_nested_runtime_schemas() {
    let spec: Value = serde_json::from_str(OPENAPI).expect("checked OpenAPI json parses");
    let schemas = &spec["components"]["schemas"];
    let response = &schemas["DataQualityHygieneOutcomeCaptureResponse"];

    assert_eq!(response["additionalProperties"], false);
    for (field, schema_name) in [
        ("outcome_record", "DataQualityHygieneOutcomeRecord"),
        (
            "reported_labor_evidence",
            "DataQualityHygieneReportedLaborEvidence",
        ),
        (
            "local_demo_readiness",
            "DataQualityHygieneLocalDemoReadiness",
        ),
        (
            "storage_projection_proof",
            "DataQualityHygieneStorageProjectionProof",
        ),
        ("observability", "DataQualityHygieneOutcomeObservability"),
        ("audit", "DataQualityHygieneOutcomeAudit"),
    ] {
        assert_eq!(
            response["properties"][field]["$ref"],
            format!("#/components/schemas/{schema_name}")
        );
        assert_eq!(schemas[schema_name]["additionalProperties"], false);
        assert!(schemas[schema_name]["required"].is_array());
    }
    assert!(
        response["required"]
            .as_array()
            .unwrap()
            .contains(&json!("idempotent_replay"))
    );
    let outcome_record = &schemas["DataQualityHygieneOutcomeRecord"];
    assert_eq!(
        outcome_record["properties"]["authority_disposition"]["const"],
        "needs_review"
    );
    assert_eq!(outcome_record["properties"]["claimable"]["const"], false);
    assert_eq!(
        outcome_record["properties"]["outcome"]["enum"],
        json!([
            "reported_completed",
            "reported_deferred",
            "reported_suppressed_by_manager",
            "reported_source_fact_was_wrong",
            "reported_not_actionable"
        ])
    );

    let summary_response = &schemas["DataQualityHygieneOutcomeSummaryResponse"];
    assert_eq!(summary_response["additionalProperties"], false);
    assert_eq!(
        summary_response["properties"]["summary"]["$ref"],
        "#/components/schemas/DataQualityHygieneOutcomeSummary"
    );
    assert_eq!(
        summary_response["properties"]["audit"]["$ref"],
        "#/components/schemas/DataQualityHygieneOutcomeSummaryAudit"
    );
}

#[test]
fn checked_openapi_marks_authenticated_mutations_and_their_fail_closed_responses() {
    let spec: Value = serde_json::from_str(OPENAPI).expect("checked OpenAPI json parses");
    let outcome = &spec["paths"]["/v1/data-quality-hygiene/actions/{action_id}/outcome"]["post"];

    assert_eq!(outcome["security"], json!([{"StaffSessionCookie": []}]));
    for status in ["401", "403"] {
        assert_eq!(
            outcome["responses"][status]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/ErrorEnvelope"
        );
    }
}

#[test]
fn data_quality_validation_schema_requires_the_runtime_error_envelope() {
    let spec: Value = serde_json::from_str(OPENAPI).expect("checked OpenAPI json parses");
    let schema = &spec["components"]["schemas"]["DataQualityHygieneOutcomeValidationResponse"];
    assert_eq!(schema["additionalProperties"], false);
    let required = schema["required"].as_array().unwrap();
    for field in [
        "error",
        "request_id",
        "live_side_effects",
        "accepted",
        "outcome_persisted",
        "reasons",
    ] {
        assert!(required.contains(&Value::String(field.to_owned())));
    }
    assert_eq!(schema["properties"]["error"]["additionalProperties"], false);
}

#[test]
fn runtime_contract_manifest_matches_openapi_fields_requiredness_nullability_enums_and_refs() {
    let spec: Value = serde_json::from_str(OPENAPI).expect("checked OpenAPI json parses");
    for runtime in public_contract::runtime_schema_contracts() {
        let schema = &spec["components"]["schemas"][runtime.name];
        assert!(schema.is_object(), "OpenAPI is missing {}", runtime.name);
        assert_eq!(
            schema["additionalProperties"], false,
            "{} must reject unknown fields",
            runtime.name
        );
        assert_eq!(
            schema["required"],
            json!(runtime.required),
            "{} requiredness drift",
            runtime.name
        );
        for field in runtime.fields {
            let openapi = &schema["properties"][field.name];
            assert!(
                openapi.is_object(),
                "{}.{} is missing",
                runtime.name,
                field.name
            );
            let nested_ref = openapi
                .get("$ref")
                .or_else(|| openapi.get("items").and_then(|items| items.get("$ref")))
                .and_then(Value::as_str);
            assert_eq!(
                nested_ref, field.nested_ref,
                "{}.{} nested ref drift",
                runtime.name, field.name
            );
            assert_eq!(
                openapi_field_is_nullable(openapi),
                field.nullable,
                "{}.{} nullability drift",
                runtime.name,
                field.name
            );
            assert_eq!(
                openapi.get("format").and_then(Value::as_str),
                field.format,
                "{}.{} format drift",
                runtime.name,
                field.name
            );
            assert_eq!(
                openapi.get("minimum").and_then(Value::as_u64),
                field.minimum,
                "{}.{} numeric minimum drift",
                runtime.name,
                field.name
            );
            assert_eq!(
                openapi.get("minLength").and_then(Value::as_u64),
                field.min_length,
                "{}.{} minimum length drift",
                runtime.name,
                field.name
            );
            if !field.enum_values.is_empty() {
                assert_eq!(
                    openapi["enum"],
                    json!(field.enum_values),
                    "{}.{} enum drift",
                    runtime.name,
                    field.name
                );
            }
        }
    }
}

#[test]
fn runtime_route_inventory_matches_openapi_including_manager_daily_brief_mutations() {
    let spec: Value = serde_json::from_str(OPENAPI).expect("checked OpenAPI json parses");
    let documented = spec["paths"].as_object().expect("paths object");
    for route in [
        "/v1/agent/context/data-quality-hygiene",
        "/v1/agent/context/manager-daily-brief",
        "/v1/agent/context/permissioned-knowledge",
        "/v1/agent/context/site-finance",
        "/v1/agent/drafts/data-quality-hygiene",
        "/v1/agent/drafts/manager-daily-brief",
        "/v1/data-quality-hygiene/actions/{action_id}/outcome",
        "/v1/data-quality-hygiene/outcomes/summary",
        "/v1/demo/information-lifespan/run",
        "/v1/demo/information-lifespan/{correlation_id}/report",
        "/v1/healthz",
        "/v1/inquiries",
        "/v1/manager-daily-brief/actions/{action_id}/outcome",
        "/v1/metrics",
        "/v1/ops/metrics/summary",
        "/v1/read-models/source-quality-backlog",
        "/v1/readyz",
        "/v1/staff/inquiries",
        "/v1/vaccine-documents/review-packets/{review_packet_id}/approve",
        "/v1/vaccine-documents/review-packets/{review_packet_id}/reject",
        "/v1/vaccine-documents/uploads",
    ] {
        assert!(
            documented.contains_key(route),
            "OpenAPI missing runtime route {route}"
        );
    }
    assert_eq!(
        documented["/v1/manager-daily-brief/actions/{action_id}/outcome"]["post"]["requestBody"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/ManagerDailyBriefOutcomeCaptureRequest"
    );
}

#[test]
fn manager_daily_brief_outcome_schema_exposes_reported_nonclaimable_disposition() {
    let spec: Value = serde_json::from_str(OPENAPI).expect("checked OpenAPI json parses");
    let schema = &spec["components"]["schemas"]["ManagerDailyBriefOutcomeRecord"];

    assert_eq!(schema["additionalProperties"], false);
    assert!(
        schema["required"]
            .as_array()
            .expect("required fields")
            .iter()
            .any(|field| field == "authority_disposition")
    );
    assert!(
        schema["required"]
            .as_array()
            .expect("required fields")
            .iter()
            .any(|field| field == "claimable")
    );
    assert_eq!(
        schema["properties"]["authority_disposition"]["const"],
        "needs_review"
    );
    assert_eq!(schema["properties"]["claimable"]["const"], false);
    assert_eq!(
        schema["properties"]["outcome"]["enum"],
        json!([
            "reported_completed",
            "reported_deferred",
            "reported_suppressed_by_manager",
            "reported_source_fact_was_wrong"
        ])
    );
}

#[test]
fn manager_daily_brief_errors_advertise_the_actual_closed_workflow_envelope() {
    let spec: Value = serde_json::from_str(OPENAPI).expect("checked OpenAPI json parses");
    let operation = &spec["paths"]["/v1/manager-daily-brief/actions/{action_id}/outcome"]["post"];
    for status in ["401", "403", "409", "422"] {
        assert_eq!(
            operation["responses"][status]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/ManagerDailyBriefOutcomeErrorResponse"
        );
    }
    let schema = &spec["components"]["schemas"]["ManagerDailyBriefOutcomeErrorResponse"];
    assert_eq!(schema["additionalProperties"], false);
    for field in [
        "error",
        "request_id",
        "live_side_effects",
        "api_contract",
        "accepted",
        "outcome_persisted",
        "reasons",
        "blocked_actions",
    ] {
        assert!(schema["properties"].get(field).is_some(), "missing {field}");
    }
}

#[test]
fn public_contract_dtos_serialize_owned_boundary_and_error_posture() {
    use pet_resort_api::error::{ErrorContext, ErrorKind, PublicApiError};

    let metadata = public_contract::ApiContractMetadata::operations_v1("data-quality-hygiene");
    let error = PublicApiError::new(
        ErrorKind::Validation {
            details: vec![public_contract::ErrorDetail::field(
                "actions[0].requested_side_effects".to_owned(),
                "customer_send_requires_review_and_live_sends_are_disabled".to_owned(),
            )],
        },
        ErrorContext::new("req_contract_test").with_correlation_id("data-quality-hygiene:test"),
    )
    .envelope();

    let metadata_json = serde_json::to_value(metadata).expect("metadata serializes");
    let error_json = serde_json::to_value(error).expect("error envelope serializes");

    assert_eq!(metadata_json["owner"], "pet_resort_api");
    assert_eq!(metadata_json["boundary"], "api_runtime_dto");
    assert_eq!(metadata_json["provider_boundary"], "evidence_refs_only");
    assert_eq!(metadata_json["live_side_effects"], "disabled");
    assert_eq!(error_json["error"]["safe_error_class"], "validation_failed");
    assert_eq!(error_json["live_side_effects"], "disabled");
}

#[tokio::test]
async fn v1_routes_expose_safe_runtime_readiness_and_data_quality_context() {
    let (health_status, health) = get_json("/v1/healthz").await;
    assert_eq!(health_status, axum_http::StatusCode::OK);
    assert_eq!(health["live_side_effects"], "disabled");
    assert_eq!(
        health["api_contract"]["provider_boundary"],
        "evidence_refs_only"
    );

    let (ready_status, ready) = get_json("/v1/readyz").await;
    assert_eq!(ready_status, axum_http::StatusCode::OK);
    assert_eq!(ready["workflow_repository"]["active_adapter"], "in_memory");
    assert_eq!(ready["live_customer_messaging"], "disabled");
    assert_eq!(ready["live_provider_writes"], "disabled");

    let (manager_context_status, manager_context) = get_json(
        "/v1/agent/context/manager-daily-brief?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
    )
    .await;
    assert_eq!(manager_context_status, axum_http::StatusCode::OK);
    assert_eq!(manager_context["workflow"]["name"], "manager_daily_brief");
    assert!(
        !manager_context["manager_brief_actions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        manager_context["blocked_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("mutate_provider_or_pms_record"))
    );
    assert!(
        manager_context["blocked_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("change_staff_schedule"))
    );

    let (context_status, context) = get_json(
        "/v1/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
    )
    .await;
    assert_eq!(context_status, axum_http::StatusCode::OK);
    assert_eq!(context["workflow"]["name"], "data-quality-hygiene");
    assert_eq!(context["live_side_effects_allowed"], false);
    assert!(
        context["blocked_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("mutate_provider_or_pms_record"))
    );
    assert!(
        context["blocked_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("hide_or_auto_resolve_source_ambiguity"))
    );
}

#[tokio::test]
async fn v1_data_quality_draft_rejection_preserves_safe_error_shape() {
    let context = get_json(
        "/v1/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
    )
    .await
    .1;
    let action = context["hygiene_actions"][0].clone();
    let (status, payload) = post_json(
        "/v1/agent/drafts/data-quality-hygiene",
        json!({
            "context_packet_id": context["audit"]["context_packet_id"],
            "correlation_id": "data-quality-hygiene:test-v1-draft",
            "actions": [{
                "action_id": action["id"],
                "kind": action["kind"],
                "source_refs": action["source_refs"],
                "issue_refs": action["issue_refs"],
                "review_gates": action["review_gates"],
                "requested_side_effects": ["send_customer_message"],
                "attempted_ambiguity_resolution": true
            }]
        }),
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["live_side_effects_allowed"], false);
    assert_eq!(
        payload["validation"]["safe_error_class"],
        "validation_failed"
    );
    assert!(
        payload["rejected_actions"][0]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("blocked_side_effect_requested"))
    );
    assert!(
        payload["rejected_actions"][0]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("attempted_ambiguity_hiding"))
    );
}

#[tokio::test]
async fn v1_read_model_route_returns_safe_fallback_when_database_is_not_configured() {
    unsafe {
        std::env::remove_var("DATABASE_URL");
    }

    let (status, payload) =
        get_json("/v1/read-models/source-quality-backlog?location_id=local").await;

    assert_eq!(status, axum_http::StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        payload["api_contract"]["workflow"],
        "source_quality_backlog_read_model"
    );
    assert_eq!(payload["read_model"]["name"], "source_quality_backlog");
    assert_eq!(payload["database"]["status"], "not_configured");
    assert_eq!(payload["data_posture"]["live_side_effects_allowed"], false);
    assert_eq!(payload["data_posture"]["provider_writes_allowed"], false);
    assert_eq!(payload["records"], json!([]));

    let spec: Value = serde_json::from_str(OPENAPI).expect("checked OpenAPI json parses");
    let route_responses =
        &spec["paths"]["/v1/read-models/source-quality-backlog"]["get"]["responses"];
    assert_eq!(
        route_responses["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/SourceQualityBacklogResponse"
    );
    assert_eq!(
        route_responses["503"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/SourceQualityBacklogResponse"
    );
}

#[tokio::test]
async fn v1_success_payloads_include_openapi_required_contract_fields() {
    let spec: Value = serde_json::from_str(OPENAPI).expect("checked OpenAPI json parses");
    let schemas = &spec["components"]["schemas"];

    let context = get_json(
        "/v1/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
    )
    .await
    .1;
    let action = context["hygiene_actions"][0].clone();

    let (draft_status, draft) = post_json(
        "/v1/agent/drafts/data-quality-hygiene",
        json!({
            "context_packet_id": context["audit"]["context_packet_id"],
            "correlation_id": "data-quality-hygiene:test-v1-contract-success",
            "actions": [{
                "action_id": action["id"],
                "kind": action["kind"],
                "source_refs": action["source_refs"],
                "issue_refs": action["issue_refs"],
                "review_gates": action["review_gates"],
                "requested_side_effects": [],
                "attempted_ambiguity_resolution": false
            }]
        }),
    )
    .await;
    assert_eq!(draft_status, axum_http::StatusCode::CREATED);
    assert_required_fields_present(
        &schemas["DataQualityHygieneDraftSubmissionResponse"],
        &draft,
    );

    let (summary_status, summary) = get_json(
        "/v1/data-quality-hygiene/outcomes/summary?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
    )
    .await;
    assert_eq!(summary_status, axum_http::StatusCode::OK);
    assert_required_fields_present(
        &schemas["DataQualityHygieneOutcomeSummaryResponse"],
        &summary,
    );

    let outcome_uri = format!(
        "/v1/data-quality-hygiene/actions/{}/outcome",
        action["id"].as_str().expect("action id is a string")
    );
    let (outcome_status, outcome) = post_json(
        &outcome_uri,
        json!({
            "outcome": "completed",
            "actual_minutes": 10,
            "actor": { "id": "front-desk:test", "persona": "front_desk_lead", "actor_role": "front_desk_lead" },
            "feedback": "Resolved duplicate aliases after manager review.",
            "source_refs": action["source_refs"],
            "issue_refs": action["issue_refs"],
            "reported_resolution_status": "repaired",
            "timestamp": "2026-06-17T16:00:00Z",
            "audit": { "correlation_id": "data-quality-hygiene:test-v1-outcome" },
            "requested_side_effects": [],
            "idempotency_key": "openapi-contract-outcome-1"
        }),
    )
    .await;
    assert_eq!(outcome_status, axum_http::StatusCode::CREATED);
    assert_required_fields_present(
        &schemas["DataQualityHygieneOutcomeCaptureResponse"],
        &outcome,
    );
}

#[test]
fn manager_outcome_response_openapi_binds_concrete_nested_contracts() {
    let spec: Value = serde_json::from_str(OPENAPI).expect("checked OpenAPI json parses");
    let schemas = &spec["components"]["schemas"];
    let response = &schemas["ManagerDailyBriefOutcomeCaptureResponse"];

    assert_eq!(
        response["properties"]["outcome_record"]["$ref"],
        "#/components/schemas/ManagerDailyBriefOutcomeRecord"
    );
    assert_eq!(
        response["properties"]["reported_labor_evidence"]["$ref"],
        "#/components/schemas/ManagerDailyBriefReportedLaborEvidence"
    );
    assert_eq!(
        response["properties"]["audit"]["$ref"],
        "#/components/schemas/ManagerDailyBriefOutcomeRecordedAudit"
    );
    for name in [
        "ManagerDailyBriefOutcomeRecord",
        "ManagerDailyBriefLaborGrouping",
        "ManagerDailyBriefReportedLaborEvidence",
        "ManagerDailyBriefOutcomeRecordedAudit",
    ] {
        assert!(schemas[name]["additionalProperties"] == false);
        assert!(
            schemas[name]["required"]
                .as_array()
                .is_some_and(|fields| !fields.is_empty())
        );
    }
}

fn assert_required_fields_present(schema: &Value, payload: &Value) {
    for required in schema["required"]
        .as_array()
        .expect("schema has required array")
    {
        let field = required.as_str().expect("required field is string");
        assert!(
            payload.get(field).is_some(),
            "payload is missing required OpenAPI field {field}: {payload}"
        );
    }
}
