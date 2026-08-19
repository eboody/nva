use super::*;

pub(super) async fn site_finance_agent_context(
    Protected(authentication): Protected,
    Extension(request_trace): Extension<RequestTraceEvidence>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_read(
        &authentication,
        authentication::Read::SiteFinance,
        Some(local_data_quality_hygiene_location_id().get()),
        None,
    ) {
        return PublicApiError::new(
            rejection.into(),
            ErrorContext::new(request_trace.request_id()),
        )
        .into_response();
    }
    let slice = match app_site_finance::fixture_site_period_projection() {
        Ok(slice) => slice,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "api_contract": api_dto_contract_payload("site_finance_recommendation_packet"),
                    "safe_error_class": "fixture_validation_failed",
                    "error": error.to_string(),
                    "live_side_effects_allowed": false
                })),
            )
                .into_response();
        }
    };
    let review_packet_id = serde_json::to_value(slice.action().review_packet_id())
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "site-finance-review:00c0ffee:2026-06".to_owned());
    let audit_event_id = serde_json::to_value(slice.action().audit_event_id())
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "audit:site-finance-review:00c0ffee:2026-06".to_owned());
    let correlation_id = "site-finance:00c0ffee:2026-06";

    (
        StatusCode::OK,
        Json(json!({
            "api_contract": api_dto_contract_payload("site_finance_recommendation_packet"),
            "workflow": {
                "name": "site_finance_recommendation_outcome",
                "version": "local-site-finance-context-v1"
            },
            "projection": {
                "data_quality_status": slice.projection().data_quality_status(),
                "net_revenue": slice.projection().net_revenue().ok(),
                "variance": slice.projection().variance(),
                "source_ref_count": slice.action().source_record_refs().len()
            },
            "recommendation": {
                "review_gate": slice.recommendation().required_review_gate(),
                "blocks_financial_mutation": slice.recommendation().blocks_financial_mutation(),
                "kind": slice.recommendation().recommendation()
            },
            "action": {
                "legal_action": slice.action().legal_action(),
                "allows_financial_mutation": slice.action().allows_financial_mutation(),
                "review_packet_id": review_packet_id,
                "audit_event_id": audit_event_id
            },
            "outcome": {
                "reviewed_action_evidence_claimable": slice.reviewed_action_evidence_outcome().can_support_value_claim(),
                "correlated_evidence_claimable": slice.correlated_evidence_outcome().can_support_value_claim()
            },
            "safety": {
                "live_side_effects_allowed": false,
                "customer_messages_allowed": false,
                "provider_writes_allowed": false,
                "financial_mutations_allowed": false
            },
            "observability": workflow_observability_payload(correlation_id, &request_trace)
        })),
    )
        .into_response()
}
