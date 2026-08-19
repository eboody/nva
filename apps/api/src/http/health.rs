use super::*;

pub(super) async fn healthz() -> Json<HealthPayload> {
    Json(HealthPayload {
        api_contract: api_dto_contract("runtime_health"),
        service: "pet-resort-api",
        status: "ok",
        live_side_effects: "disabled",
    })
}

pub(super) async fn readyz(State(state): State<VaccineDocumentState>) -> Json<ReadinessPayload> {
    let observability = state.observability.config().readiness();
    Json(ReadinessPayload {
        api_contract: api_dto_contract("runtime_readiness"),
        service: "pet-resort-api",
        database: database_readiness_status(),
        object_storage: object_storage_readiness_status(),
        agent_runtime: "fake_deterministic",
        workflow_repository: workflow_repository_readiness_payload(),
        observability: ObservabilityReadinessPayload {
            request_correlation: "x_request_id_and_x_correlation_id_response_headers_with_workflow_payload_fields",
            workflow_correlation: "local_workflow_correlation_ids_only",
            local_request_metrics: "prometheus_bounded_route_status_and_duration_series",
            metrics_scope: "bounded_route_method_status_class_without_payload_or_actor_labels",
            production_gap: if state.observability.config().production().is_some() {
                "external_stack_configured_but_live_health_is_not_verified_by_this_endpoint"
            } else {
                "no_durable_traces_queue_dashboard_or_alerting"
            },
            durable_traces: observability.durable_traces,
            production_metrics: observability.production_metrics,
            dashboard: observability.dashboard,
            alerting: observability.alerting,
        },
        live_customer_messaging: "disabled",
        live_provider_writes: "disabled",
    })
}

pub(super) async fn prometheus_metrics(
    State(state): State<VaccineDocumentState>,
) -> axum::response::Response {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        state.observability.render_prometheus(),
    )
        .into_response()
}

pub(super) async fn ops_metrics_summary(
    State(state): State<VaccineDocumentState>,
    Protected(authentication): Protected,
    Extension(request_trace): Extension<RequestTraceEvidence>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_read(
        &authentication,
        authentication::Read::OperationalMetrics,
        Some(local_data_quality_hygiene_location_id().get()),
        None,
    ) {
        return PublicApiError::new(
            rejection.into(),
            ErrorContext::new(request_trace.request_id()),
        )
        .into_response();
    }
    let store = state.store.lock().await;
    let manager_daily_brief =
        manager_daily_brief_labor_rollup(store.manager_daily_brief_outcomes.outcomes());
    let data_quality_hygiene =
        data_quality_hygiene_labor_rollup(store.data_quality_hygiene_outcomes.outcomes());
    let counters = store.runtime_counters();

    Json(OpsMetricsSummaryPayload {
        api_contract: api_dto_contract("ops_metrics_summary"),
        api_request_metrics: ApiRequestMetricsPayload {
            scope: "local_runtime_only",
            request_id_source: "x_request_id_or_generated_uuid",
            correlation_id_source: "x_correlation_id_or_generated_uuid",
            payload_logging: "disabled",
            safe_error_classes: vec!["validation_failed", "not_found", "not_applicable"],
        },
        product_labor_metrics: ProductLaborMetricsPayload {
            manager_daily_brief,
            data_quality_hygiene,
        },
        local_runtime_counters: LocalRuntimeCountersPayload {
            inquiry_count: counters.inquiry_count,
            review_packet_count: counters.review_packet_count,
            audit_event_count: counters.audit_event_count,
            outcome_count: counters.outcome_count,
            data_quality_hygiene_outbox_candidate_count: counters.internal_outbox_candidate_count,
            data_quality_hygiene_review_gated_outbox_count: counters
                .review_gated_internal_outbox_count,
            production_queue_adapter: "not_configured",
        },
        safety: MetricsSafetyPayload {
            granularity: "aggregate_only",
            contains_customer_pii: false,
            contains_provider_payloads: false,
            live_side_effects: "disabled",
        },
        observability_gap: ObservabilityGapPayload {
            production_traces: "not_configured",
            durable_request_metrics: "not_configured",
            dashboard_or_alerting: "not_configured",
        },
        production_metrics_plan: vec![
            "request_latency",
            "error_rate",
            "queue_depth",
            "dead_letter_count",
            "review_sla",
            "outbox_failures",
            "worker_lease_age",
        ],
    })
    .into_response()
}

pub(super) fn manager_daily_brief_labor_rollup(
    records: &[storage::operations::ManagerDailyBriefOutcomeRecord],
) -> LaborOutcomeRollupPayload {
    let reported_actual_minutes_spent = records.iter().fold(0u16, |total, record| {
        total.saturating_add(record.actual_minutes.get())
    });

    LaborOutcomeRollupPayload {
        metric_source: "manager_daily_brief_outcome_records",
        reported_outcome_count: records.len(),
        reported_actual_minutes_spent,
    }
}

pub(super) fn data_quality_hygiene_labor_rollup(
    records: &[storage::operations::DataQualityHygieneOutcomeRecord],
) -> LaborOutcomeRollupPayload {
    let reported_actual_minutes_spent = records.iter().fold(0u16, |total, record| {
        total.saturating_add(record.actual_minutes.get())
    });

    LaborOutcomeRollupPayload {
        metric_source: "data_quality_hygiene_outcome_records",
        reported_outcome_count: records.len(),
        reported_actual_minutes_spent,
    }
}
