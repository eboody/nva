use axum::{body::Body, http as axum_http};
use http_body_util::BodyExt;
use pet_resort_api::http;
use tower::ServiceExt;

async fn get_json(uri: &str) -> (axum_http::StatusCode, serde_json::Value) {
    let response = http::router()
        .oneshot(
            axum_http::request::Builder::new()
                .method(axum_http::Method::GET)
                .uri(uri)
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("agent context request succeeds");

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let payload = serde_json::from_slice(&body).expect("json context payload");

    (status, payload)
}

#[tokio::test]
async fn manager_daily_brief_agent_context_returns_source_grounded_read_only_packet() {
    let (status, payload) = get_json(
        "/v0/agent/context/manager-daily-brief?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17",
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::OK);
    assert_eq!(payload["workflow"]["name"], "manager_daily_brief");
    assert_eq!(
        payload["workflow"]["version"],
        "local-manager-daily-brief-context-v1"
    );
    assert_eq!(
        payload["location_id"],
        "00c0ffee-0000-0000-0000-000000000001"
    );
    assert_eq!(payload["operating_day"], "2026-06-17");

    assert_eq!(payload["service_demand_facts"].as_array().unwrap().len(), 1);
    assert_eq!(
        payload["service_demand_facts"][0]["source_refs"][0]["system"],
        "gingr"
    );
    assert_eq!(
        payload["checkout_completion_exceptions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        payload["crm_retention_opportunities"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        payload["manager_brief_actions"].as_array().unwrap().len(),
        3
    );
    assert!(payload["source_refs"].as_array().unwrap().len() >= 3);

    assert!(
        payload["allowed_agent_actions"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("rank_manager_actions"))
    );
    assert!(
        payload["allowed_agent_actions"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("estimate_labor_minutes_saved"))
    );
    assert!(
        payload["blocked_actions"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("mutate_provider_or_pms_record"))
    );
    assert!(
        payload["blocked_actions"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("send_customer_message"))
    );

    assert_eq!(
        payload["audit"]["correlation_id"],
        "manager-daily-brief:00c0ffee-0000-0000-0000-000000000001:2026-06-17"
    );
    assert!(
        payload["audit"]["context_packet_id"]
            .as_str()
            .unwrap()
            .starts_with("manager-daily-brief-context:")
    );
}

#[tokio::test]
async fn manager_daily_brief_agent_context_reports_missing_facts_as_typed_data_quality_issues() {
    let (status, payload) = get_json(
        "/v0/agent/context/manager-daily-brief?location_id=00c0ffee-0000-0000-0000-000000000002&operating_day=2026-06-18",
    )
    .await;

    assert_eq!(status, axum_http::StatusCode::OK);
    assert!(
        payload["service_demand_facts"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        payload["checkout_completion_exceptions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        payload["crm_retention_opportunities"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        payload["manager_brief_actions"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let issue_kinds = payload["data_quality_issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["kind"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(issue_kinds.contains(&"missing_service_demand_fact"));
    assert!(issue_kinds.contains(&"missing_checkout_completion_packet"));
    assert!(issue_kinds.contains(&"missing_crm_retention_packet"));
    assert!(
        payload["blocked_actions"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("hide_source_data_quality_issue"))
    );
}
