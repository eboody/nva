use axum::{
    Router,
    body::Body,
    http::{self as axum_http, Method, StatusCode},
};
use http_body_util::BodyExt;
use pet_resort_api::http;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use tower::ServiceExt;

const REQUEST_ID: &str = "route-family-contract-request";
const CORRELATION_ID: &str = "route-family-contract-correlation";
const SENSITIVE_BODY_MARKER: &str = "must-not-be-reflected-private-payload";
const SENSITIVE_HEADER_MARKER: &str = "must-not-be-reflected-private-header";
const SENSITIVE_QUERY_MARKER: &str = "must-not-be-reflected-private-query";
const SENSITIVE_PATH_MARKER: &str = "must-not-be-reflected-private-path";

#[test]
fn http_boundary_is_composed_from_bounded_capability_modules() {
    let root = include_str!("../src/http.rs");
    assert!(
        root.lines().count() < 500,
        "http.rs must be composition glue below 500 physical lines"
    );

    for (module, source) in [
        ("auth", include_str!("../src/http/auth.rs")),
        ("health", include_str!("../src/http/health.rs")),
        ("inquiry", include_str!("../src/http/inquiry.rs")),
        ("checkout", include_str!("../src/http/checkout.rs")),
        (
            "contract_observation",
            include_str!("../src/http/contract_observation.rs"),
        ),
        ("data_quality", include_str!("../src/http/data_quality.rs")),
        (
            "manager_brief",
            include_str!("../src/http/manager_brief.rs"),
        ),
        ("site_finance", include_str!("../src/http/site_finance.rs")),
        ("workflow", include_str!("../src/http/workflow.rs")),
        ("dto", include_str!("../src/http/dto/mod.rs")),
        (
            "error_mapping",
            include_str!("../src/http/error_mapping.rs"),
        ),
        ("router", include_str!("../src/http/router.rs")),
        ("state", include_str!("../src/http/state.rs")),
        ("vaccine", include_str!("../src/http/vaccine.rs")),
    ] {
        assert!(
            root.contains(&format!("mod {module};")),
            "http.rs must compose the {module} capability module"
        );
        assert!(
            source.lines().count() <= 1_500,
            "http/{module} exceeds the 1,500-line module ceiling"
        );
    }
}

#[test]
fn denial_safety_is_proved_by_runtime_observation_not_source_spelling() {
    let characterization = include_str!("http_route_family_characterization.rs");
    let prohibited_source_count = ["source.", "matches(\".", "attempt", "(\")"].concat();

    assert!(
        !characterization.contains(&prohibited_source_count),
        "live-effect denial safety must be proved by runtime observation, not source-substring counting"
    );
}

struct RouteContract {
    family: &'static str,
    method: Method,
    uri: &'static str,
    body: &'static str,
}

async fn request_on(
    router: Router,
    method: Method,
    uri: &str,
    body: impl Into<Body>,
) -> (StatusCode, axum_http::HeaderMap, Vec<u8>) {
    let response = router
        .oneshot(
            axum_http::Request::builder()
                .method(method)
                .uri(uri)
                .header("x-request-id", REQUEST_ID)
                .header("x-correlation-id", CORRELATION_ID)
                .header("x-private-contract-probe", SENSITIVE_HEADER_MARKER)
                .header(axum_http::header::CONTENT_TYPE, "application/json")
                .body(body.into())
                .expect("route-family contract request builds"),
        )
        .await
        .expect("route-family contract request completes");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("route-family response body collects")
        .to_bytes()
        .to_vec();
    (status, headers, bytes)
}

async fn request(
    method: Method,
    uri: &str,
    body: impl Into<Body>,
) -> (StatusCode, axum_http::HeaderMap, Vec<u8>) {
    request_on(
        http::router_with_test_auth_state(http::VaccineDocumentState::default()),
        method,
        uri,
        body,
    )
    .await
}

fn with_sensitive_query(uri: &str) -> String {
    let separator = if uri.contains('?') { '&' } else { '?' };
    format!("{uri}{separator}sensitive_probe={SENSITIVE_QUERY_MARKER}")
}

fn response_text(headers: &axum_http::HeaderMap, bytes: &[u8]) -> String {
    let mut response = String::from_utf8_lossy(bytes).into_owned();
    for (name, value) in headers {
        response.push_str(name.as_str());
        response.push_str(value.to_str().unwrap_or("<non-text-header>"));
    }
    response
}

fn protected_route_contracts() -> Vec<RouteContract> {
    vec![
        RouteContract {
            family: "inquiry intake",
            method: Method::POST,
            uri: "/v1/inquiries",
            body: SENSITIVE_BODY_MARKER,
        },
        RouteContract {
            family: "inquiry staff queue",
            method: Method::GET,
            uri: "/v1/staff/inquiries",
            body: "",
        },
        RouteContract {
            family: "manager brief context",
            method: Method::GET,
            uri: "/v1/agent/context/manager-daily-brief?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
            body: "",
        },
        RouteContract {
            family: "manager brief canonical context",
            method: Method::GET,
            uri: "/v1/agent/context/manager-daily-brief?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
            body: "",
        },
        RouteContract {
            family: "manager brief draft",
            method: Method::POST,
            uri: "/v1/agent/drafts/manager-daily-brief",
            body: SENSITIVE_BODY_MARKER,
        },
        RouteContract {
            family: "manager brief outcome",
            method: Method::POST,
            uri: "/v1/manager-daily-brief/actions/must-not-be-reflected-private-path/outcome",
            body: SENSITIVE_BODY_MARKER,
        },
        RouteContract {
            family: "manager brief canonical outcome",
            method: Method::POST,
            uri: "/v1/manager-daily-brief/actions/must-not-be-reflected-private-path/outcome",
            body: SENSITIVE_BODY_MARKER,
        },
        RouteContract {
            family: "data quality context",
            method: Method::GET,
            uri: "/v1/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
            body: "",
        },
        RouteContract {
            family: "data quality canonical context",
            method: Method::GET,
            uri: "/v1/agent/context/data-quality-hygiene?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
            body: "",
        },
        RouteContract {
            family: "data quality draft",
            method: Method::POST,
            uri: "/v1/agent/drafts/data-quality-hygiene",
            body: SENSITIVE_BODY_MARKER,
        },
        RouteContract {
            family: "data quality canonical draft",
            method: Method::POST,
            uri: "/v1/agent/drafts/data-quality-hygiene",
            body: SENSITIVE_BODY_MARKER,
        },
        RouteContract {
            family: "data quality outcome",
            method: Method::POST,
            uri: "/v1/data-quality-hygiene/actions/must-not-be-reflected-private-path/outcome",
            body: SENSITIVE_BODY_MARKER,
        },
        RouteContract {
            family: "data quality canonical outcome",
            method: Method::POST,
            uri: "/v1/data-quality-hygiene/actions/must-not-be-reflected-private-path/outcome",
            body: SENSITIVE_BODY_MARKER,
        },
        RouteContract {
            family: "data quality summary",
            method: Method::GET,
            uri: "/v1/data-quality-hygiene/outcomes/summary?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
            body: "",
        },
        RouteContract {
            family: "data quality canonical summary",
            method: Method::GET,
            uri: "/v1/data-quality-hygiene/outcomes/summary?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
            body: "",
        },
        RouteContract {
            family: "permissioned knowledge context",
            method: Method::GET,
            uri: "/v1/agent/context/permissioned-knowledge?location_id=00c0ffee-0000-0000-0000-000000000001&service=boarding&role=general_manager&section=check-in.required-documents",
            body: "",
        },
        RouteContract {
            family: "permissioned knowledge canonical context",
            method: Method::GET,
            uri: "/v1/agent/context/permissioned-knowledge?location_id=00c0ffee-0000-0000-0000-000000000001&service=boarding&role=general_manager&section=check-in.required-documents",
            body: "",
        },
        RouteContract {
            family: "site finance context",
            method: Method::GET,
            uri: "/v1/agent/context/site-finance",
            body: "",
        },
        RouteContract {
            family: "site finance canonical context",
            method: Method::GET,
            uri: "/v1/agent/context/site-finance",
            body: "",
        },
        RouteContract {
            family: "operational metrics summary",
            method: Method::GET,
            uri: "/v1/ops/metrics/summary",
            body: "",
        },
        RouteContract {
            family: "operational metrics canonical summary",
            method: Method::GET,
            uri: "/v1/ops/metrics/summary",
            body: "",
        },
        RouteContract {
            family: "source quality read model",
            method: Method::GET,
            uri: "/v1/read-models/source-quality-backlog",
            body: "",
        },
        RouteContract {
            family: "information lifespan run",
            method: Method::POST,
            uri: "/v1/demo/information-lifespan/run",
            body: SENSITIVE_BODY_MARKER,
        },
        RouteContract {
            family: "information lifespan canonical run",
            method: Method::POST,
            uri: "/v1/demo/information-lifespan/run",
            body: SENSITIVE_BODY_MARKER,
        },
        RouteContract {
            family: "information lifespan report",
            method: Method::GET,
            uri: "/v1/demo/information-lifespan/must-not-be-reflected-private-path/report",
            body: "",
        },
        RouteContract {
            family: "information lifespan canonical report",
            method: Method::GET,
            uri: "/v1/demo/information-lifespan/must-not-be-reflected-private-path/report",
            body: "",
        },
        RouteContract {
            family: "vaccine document upload",
            method: Method::POST,
            uri: "/v1/vaccine-documents/uploads",
            body: SENSITIVE_BODY_MARKER,
        },
        RouteContract {
            family: "vaccine approval",
            method: Method::POST,
            uri: "/v1/vaccine-documents/review-packets/00000000-0000-4000-8000-000000000001/approve",
            body: SENSITIVE_BODY_MARKER,
        },
        RouteContract {
            family: "vaccine rejection",
            method: Method::POST,
            uri: "/v1/vaccine-documents/review-packets/00000000-0000-4000-8000-000000000001/reject",
            body: SENSITIVE_BODY_MARKER,
        },
    ]
}

#[test]
fn every_protected_handler_uses_the_authenticating_entry_probe_before_its_body() {
    let source = concat!(
        include_str!("../src/http/workflow.rs"),
        include_str!("../src/http/health.rs"),
        include_str!("../src/http/inquiry.rs"),
        include_str!("../src/http/manager_brief.rs"),
        include_str!("../src/http/data_quality.rs"),
        include_str!("../src/http/checkout.rs"),
        include_str!("../src/http/site_finance.rs"),
        include_str!("../src/http/vaccine.rs"),
    );
    let contract_observation = include_str!("../src/http/contract_observation.rs");
    for handler in [
        "source_quality_backlog",
        "run_information_lifespan_demo",
        "replay_information_lifespan_report",
        "ops_metrics_summary",
        "submit_inquiry",
        "staff_inquiries",
        "capture_manager_daily_brief_action_outcome",
        "submit_manager_daily_brief_agent_draft",
        "data_quality_hygiene_agent_context",
        "permissioned_knowledge_agent_context",
        "site_finance_agent_context",
        "submit_data_quality_hygiene_agent_draft",
        "capture_data_quality_hygiene_action_outcome",
        "data_quality_hygiene_outcome_summary",
        "manager_daily_brief_agent_context",
        "upload_vaccine_document",
        "approve_vaccine_document",
        "reject_vaccine_document",
    ] {
        let signature = source
            .split_once(&format!("async fn {handler}("))
            .unwrap_or_else(|| panic!("missing protected handler {handler}"))
            .1
            .split_once(") ->")
            .unwrap_or_else(|| panic!("missing signature terminator for {handler}"))
            .0;
        assert!(
            signature.contains("Protected(authentication): Protected"),
            "{handler} must use the handler-local authenticating entry probe"
        );
    }

    let authentication = contract_observation
        .find("authentication::authenticate(&context)")
        .expect("protected extractor authenticates");
    let entry = contract_observation
        .find("entries.0.fetch_add(1")
        .expect("protected extractor records handler entry");
    assert!(
        authentication < entry,
        "the protected extractor must authenticate before recording handler entry"
    );
}

#[tokio::test]
async fn canonical_v1_health_readiness_and_metrics_preserve_payload_contracts() {
    for uri in ["/v1/healthz"] {
        let (status, headers, bytes) = request(Method::GET, uri, Body::empty()).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert_eq!(headers["x-request-id"], REQUEST_ID, "{uri}");
        assert_eq!(headers["x-correlation-id"], CORRELATION_ID, "{uri}");
        let payload: Value = serde_json::from_slice(&bytes).expect("health response is json");
        assert_eq!(
            payload,
            json!({
                "api_contract": {
                    "owner": "pet_resort_api",
                    "boundary": "api_runtime_dto",
                    "schema_version": "pet_resort_api.runtime.v1",
                    "workflow": "runtime_health",
                    "provider_boundary": "evidence_refs_only",
                    "live_side_effects": "disabled"
                },
                "service": "pet-resort-api",
                "status": "ok",
                "live_side_effects": "disabled"
            }),
            "{uri}"
        );
    }

    for uri in ["/v1/readyz"] {
        let (status, _, bytes) = request(Method::GET, uri, Body::empty()).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        let payload: Value = serde_json::from_slice(&bytes).expect("readiness response is json");
        assert_eq!(payload["api_contract"]["workflow"], "runtime_readiness");
        assert_eq!(payload["live_customer_messaging"], "disabled");
        assert_eq!(payload["live_provider_writes"], "disabled");
    }

    for uri in ["/v1/metrics"] {
        let (status, headers, bytes) = request(Method::GET, uri, Body::empty()).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert_eq!(
            headers[axum_http::header::CONTENT_TYPE],
            "text/plain; version=0.0.4; charset=utf-8",
            "{uri}"
        );
        let text = String::from_utf8(bytes).expect("metrics response is utf-8 text");
        assert!(text.contains("pet_resort_api_requests_total"), "{uri}");
    }
}

#[tokio::test]
async fn removed_unversioned_product_routes_are_not_executable_aliases() {
    for uri in [
        "/healthz",
        "/readyz",
        "/metrics",
        "/ops/metrics/summary",
        "/inquiries",
        "/staff/inquiries",
        "/agent/context/manager-daily-brief",
        "/agent/drafts/manager-daily-brief",
        "/manager-daily-brief/actions/action-1/outcome",
        "/agent/context/data-quality-hygiene",
        "/agent/context/permissioned-knowledge",
        "/agent/context/site-finance",
        "/agent/drafts/data-quality-hygiene",
        "/data-quality-hygiene/actions/action-1/outcome",
        "/data-quality-hygiene/outcomes/summary",
        "/demo/information-lifespan/run",
        "/demo/information-lifespan/correlation/report",
        "/vaccine-documents/uploads",
        "/vaccine-documents/review-packets/review-1/approve",
        "/vaccine-documents/review-packets/review-1/reject",
    ] {
        let (status, _, _) = request(Method::GET, uri, Body::empty()).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
    }
}

#[tokio::test]
async fn every_protected_route_rejects_missing_authentication_without_reflecting_request_inputs() {
    let state = http::VaccineDocumentState::default();
    let baseline = state.persisted_record_count().await;
    let handler_entries = state.protected_handler_entry_count();
    let denied_live_effect_intents = state.denied_live_effect_intent_count();
    let router = http::router_with_test_auth_state(state.clone());
    for contract in protected_route_contracts() {
        let uri = with_sensitive_query(contract.uri);
        let (status, headers, bytes) = request_on(
            router.clone(),
            contract.method.clone(),
            &uri,
            Body::from(contract.body),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{}", contract.family);
        assert_eq!(headers["x-request-id"], REQUEST_ID, "{}", contract.family);
        assert_eq!(
            headers["x-correlation-id"], CORRELATION_ID,
            "{}",
            contract.family
        );
        let payload: Value =
            serde_json::from_slice(&bytes).expect("protected route error response is json");
        assert_eq!(
            payload["error"]["code"], "missing_trusted_actor_context",
            "{}",
            contract.family
        );
        assert_eq!(
            payload["error"]["safe_error_class"], "authentication_failed",
            "{}",
            contract.family
        );
        assert_eq!(
            payload["request_id"], "missing_request_id",
            "{}",
            contract.family
        );
        assert_eq!(
            payload["correlation_id"],
            Value::Null,
            "{}",
            contract.family
        );
        assert_eq!(
            payload["live_side_effects"], "disabled",
            "{}",
            contract.family
        );
        let response = response_text(&headers, &bytes);
        for (kind, sentinel) in [
            ("body", SENSITIVE_BODY_MARKER),
            ("header", SENSITIVE_HEADER_MARKER),
            ("query", SENSITIVE_QUERY_MARKER),
            ("path", SENSITIVE_PATH_MARKER),
        ] {
            assert!(
                !response.contains(sentinel),
                "{} reflected a sensitive request {kind}",
                contract.family
            );
        }
    }
    assert_eq!(
        state.persisted_record_count().await,
        baseline,
        "authentication failures must not persist any workflow record"
    );
    assert_eq!(
        state.protected_handler_entry_count(),
        handler_entries,
        "authentication failures must not enter a protected handler"
    );
    assert_eq!(
        state.denied_live_effect_intent_count(),
        denied_live_effect_intents,
        "authentication failures must not reach the live-effect denial boundary"
    );
}

#[tokio::test]
async fn wrong_methods_and_automatic_head_preserve_axum_routing_and_public_error_contracts() {
    let state = http::VaccineDocumentState::default();
    let baseline = state.persisted_record_count().await;
    let router = http::router_with_test_auth_state(state.clone());
    for (method, uri, expected_status, expected_allow) in [
        (
            Method::POST,
            "/v1/healthz",
            StatusCode::METHOD_NOT_ALLOWED,
            "GET,HEAD",
        ),
        (
            Method::OPTIONS,
            "/v1/healthz",
            StatusCode::METHOD_NOT_ALLOWED,
            "GET,HEAD",
        ),
        (
            Method::GET,
            "/v1/inquiries",
            StatusCode::METHOD_NOT_ALLOWED,
            "POST",
        ),
        (
            Method::OPTIONS,
            "/v1/inquiries",
            StatusCode::METHOD_NOT_ALLOWED,
            "POST",
        ),
    ] {
        let (status, headers, bytes) =
            request_on(router.clone(), method.clone(), uri, Body::empty()).await;
        assert_eq!(status, expected_status, "{method} {uri}");
        assert_eq!(
            headers[axum_http::header::ALLOW],
            expected_allow,
            "{method} {uri}"
        );
        let payload: Value = serde_json::from_slice(&bytes).expect("405 response is json");
        assert_eq!(payload["error"]["code"], "internal_error", "{method} {uri}");
        assert_eq!(
            payload["error"]["safe_error_class"], "internal_error",
            "{method} {uri}"
        );
        assert_eq!(payload["request_id"], REQUEST_ID, "{method} {uri}");
        assert_eq!(payload["correlation_id"], CORRELATION_ID, "{method} {uri}");
        assert_eq!(payload["live_side_effects"], "disabled", "{method} {uri}");
    }

    let (status, headers, bytes) =
        request_on(router, Method::HEAD, "/v1/healthz", Body::empty()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers["x-request-id"], REQUEST_ID);
    assert_eq!(headers["x-correlation-id"], CORRELATION_ID);
    assert!(
        bytes.is_empty(),
        "Axum's automatic HEAD must suppress the GET body"
    );
    assert_eq!(
        state.persisted_record_count().await,
        baseline,
        "method-routing errors and HEAD must not persist workflow records"
    );
}

#[tokio::test]
async fn unknown_routes_use_the_stable_not_found_error_envelope_without_side_effect_authority() {
    let state = http::VaccineDocumentState::default();
    let baseline = state.persisted_record_count().await;
    let (status, _, bytes) = request_on(
        http::router_with_test_auth_state(state.clone()),
        Method::GET,
        "/v1/not-an-owned-route",
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let payload: Value = serde_json::from_slice(&bytes).expect("fallback response is json");
    assert_eq!(payload["error"]["code"], "not_found");
    assert_eq!(payload["error"]["safe_error_class"], "not_found");
    assert_eq!(payload["error"]["details"][0]["field"], "path");
    assert_eq!(
        payload["error"]["details"][0]["reason"],
        "/v1/not-an-owned-route"
    );
    assert_eq!(payload["request_id"], REQUEST_ID);
    assert_eq!(payload["correlation_id"], Value::Null);
    assert_eq!(payload["live_side_effects"], "disabled");
    assert_eq!(state.persisted_record_count().await, baseline);
}

#[test]
fn canonical_openapi_route_inventory_remains_exact_before_route_family_moves() {
    let owned_v1_routes = [
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
    ];

    let spec: Value =
        serde_json::from_str(include_str!("../openapi/owned-operations-v1.openapi.json"))
            .expect("checked OpenAPI document parses");
    let documented = spec["paths"].as_object().expect("OpenAPI paths object");
    let documented_paths = documented.keys().map(String::as_str).collect::<Vec<_>>();
    assert_eq!(documented_paths, owned_v1_routes);

    let manifest = [
        ("/v1/healthz", "get", false),
        ("/v1/readyz", "get", false),
        ("/v1/metrics", "get", false),
        ("/v1/ops/metrics/summary", "get", false),
        ("/v1/agent/context/manager-daily-brief", "get", false),
        (
            "/v1/manager-daily-brief/actions/{action_id}/outcome",
            "post",
            true,
        ),
        ("/v1/agent/context/data-quality-hygiene", "get", false),
        ("/v1/agent/drafts/data-quality-hygiene", "post", true),
        (
            "/v1/data-quality-hygiene/actions/{action_id}/outcome",
            "post",
            true,
        ),
        ("/v1/data-quality-hygiene/outcomes/summary", "get", false),
        ("/v1/agent/context/permissioned-knowledge", "get", false),
        ("/v1/agent/context/site-finance", "get", false),
        ("/v1/demo/information-lifespan/run", "post", false),
        (
            "/v1/demo/information-lifespan/{correlation_id}/report",
            "get",
            false,
        ),
        ("/v1/read-models/source-quality-backlog", "get", false),
    ];
    let mut expected_operations = BTreeMap::<&str, BTreeSet<&str>>::new();
    for (path, method, _) in manifest {
        expected_operations.entry(path).or_default().insert(method);
    }
    const HTTP_OPERATION_KEYS: [&str; 8] = [
        "get", "put", "post", "delete", "options", "head", "patch", "trace",
    ];
    for (path, expected) in expected_operations {
        let actual = documented[path]
            .as_object()
            .expect("OpenAPI path item")
            .keys()
            .map(String::as_str)
            .filter(|key| HTTP_OPERATION_KEYS.contains(key))
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected, "HTTP operation-key drift for {path}");
    }

    for (path, method, authenticated) in manifest {
        let operation = &documented[path][method];
        assert!(operation.is_object(), "missing {method} {path}");
        if authenticated {
            assert_eq!(
                operation["security"],
                json!([{"StaffSessionCookie": []}]),
                "{method} {path}"
            );
            for status in ["401", "403"] {
                assert!(
                    operation["responses"].get(status).is_some(),
                    "{method} {path} missing {status} response"
                );
            }
        } else {
            assert!(operation.get("security").is_none(), "{method} {path}");
        }
    }
}
