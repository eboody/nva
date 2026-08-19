use super::*;

pub(super) async fn source_quality_backlog(
    Protected(authentication): Protected,
    Extension(request_trace): Extension<RequestTraceEvidence>,
) -> axum::response::Response {
    let location_id = match authentication::actor_location_id(&authentication) {
        Ok(location_id) => location_id,
        Err(rejection) => {
            return PublicApiError::new(
                rejection.into(),
                ErrorContext::new(request_trace.request_id()),
            )
            .into_response();
        }
    };
    if let Err(rejection) = authentication::authorize_read(
        &authentication,
        authentication::Read::SourceQualityBacklog,
        Some(location_id),
        None,
    ) {
        return PublicApiError::new(
            rejection.into(),
            ErrorContext::new(request_trace.request_id()),
        )
        .into_response();
    }
    let Some(database_url) = configured_database_url() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(source_quality_backlog_payload(
                ReadModelDatabasePayload {
                    status: "not_configured",
                    adapter: "tokio_postgres",
                    error: Some("DATABASE_URL is not configured; in-memory workflow routes remain available".to_owned()),
                },
                Vec::new(),
            )),
        )
            .into_response();
    };

    let repository = storage::workflow_repository::PostgresSourceQualityBacklog::new(database_url);
    match repository
        .prioritized_items_for_location(entities::LocationId::new(location_id))
        .await
    {
        Ok(records) => (
            StatusCode::OK,
            Json(source_quality_backlog_payload(
                ReadModelDatabasePayload {
                    status: "connected",
                    adapter: "tokio_postgres",
                    error: None,
                },
                records,
            )),
        )
            .into_response(),
        Err(_error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(source_quality_backlog_payload(
                ReadModelDatabasePayload {
                    status: "query_failed",
                    adapter: "tokio_postgres",
                    error: Some("postgres read-model query failed; details redacted".to_owned()),
                },
                Vec::new(),
            )),
        )
            .into_response(),
    }
}

pub(super) fn source_quality_backlog_payload(
    database: ReadModelDatabasePayload,
    records: Vec<workflow_repository::source_quality_backlog::Item>,
) -> SourceQualityBacklogPayload {
    SourceQualityBacklogPayload {
        api_contract: api_dto_contract("source_quality_backlog_read_model"),
        read_model: ReadModelDescriptorPayload {
            name: "source_quality_backlog",
            source: "postgres_view",
            projection_version: "source_quality_backlog.v1",
        },
        data_posture: ReadModelDataPosturePayload {
            safe_synthetic_data: true,
            live_side_effects_allowed: false,
            provider_payload_passthrough: false,
            provider_writes_allowed: false,
            customer_messages_allowed: false,
        },
        database,
        records,
    }
}

pub(super) fn configured_database_url() -> Option<String> {
    env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty() && !value.contains("***"))
}

pub(super) fn database_url_configured() -> bool {
    configured_database_url().is_some()
}

pub(super) fn minio_env_configured() -> bool {
    env::var("MINIO_ENDPOINT")
        .ok()
        .is_some_and(|value| !value.trim().is_empty())
}

pub(super) async fn run_information_lifespan_demo(
    Protected(authentication): Protected,
    Extension(request_trace): Extension<RequestTraceEvidence>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_source_ingest(
        &authentication,
        authentication::Mutation::InformationLifespanDemo,
        local_data_quality_hygiene_location_id().get(),
    ) {
        return (
            rejection.status_code(),
            Json(authorization_error_payload(
                rejection,
                "information_lifespan_demo_run",
                "run_persisted",
            )),
        )
            .into_response();
    }

    let trace = information_lifespan_fixture::mock_gingr_manager_daily_report_trace();
    let correlation_id = trace.correlation_id().as_str().to_owned();
    let processor_proof = information_lifespan_processor_proof().await;

    tracing::info!(
        workflow = "information_lifespan_demo_run",
        correlation_id,
        synthetic_data_only = true,
        live_side_effects_allowed = false,
        "information lifespan demo replay returned manager report artifact"
    );

    (
        StatusCode::OK,
        Json(information_lifespan_run_payload(
            "information_lifespan_demo_run",
            &trace,
            &request_trace,
            processor_proof,
        )),
    )
        .into_response()
}

pub(super) async fn replay_information_lifespan_report(
    Protected(authentication): Protected,
    Path(correlation_id): Path<String>,
    Extension(request_trace): Extension<RequestTraceEvidence>,
) -> axum::response::Response {
    if let Err(rejection) = authentication::authorize_read(
        &authentication,
        authentication::Read::InformationLifespanReport,
        Some(local_data_quality_hygiene_location_id().get()),
        None,
    ) {
        return PublicApiError::new(
            rejection.into(),
            ErrorContext::new(request_trace.request_id()),
        )
        .into_response();
    }
    let trace = information_lifespan_fixture::mock_gingr_manager_daily_report_trace();
    if correlation_id != trace.correlation_id().as_str() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": {
                    "code": "information_lifespan_run_not_found",
                    "message": "Only the deterministic synthetic information-lifespan demo run is available in the local replay API"
                },
                "requested_correlation_id": correlation_id,
                "available_correlation_id": trace.correlation_id().as_str(),
                "synthetic_data_only": true,
                "live_side_effects_allowed": false
            })),
        )
            .into_response();
    }

    let processor_proof = information_lifespan_processor_proof().await;
    (
        StatusCode::OK,
        Json(information_lifespan_run_payload(
            "information_lifespan_demo_report",
            &trace,
            &request_trace,
            processor_proof,
        )),
    )
        .into_response()
}

pub(super) async fn information_lifespan_processor_proof() -> Value {
    let output_dir = env::var("INFORMATION_LIFESPAN_OUTPUT_DIR")
        .unwrap_or_else(|_| ".var/information-lifespan".to_owned());
    let output_path = format!("{output_dir}/processor-output.json");
    match std::fs::read_to_string(&output_path) {
        Ok(contents) => serde_json::from_str::<Value>(&contents).unwrap_or_else(|error| {
            information_lifespan_processor_fallback(
                "processor_output_parse_failed",
                Some(output_path),
                Some(error.to_string()),
            )
        }),
        Err(error) => information_lifespan_processor_fallback(
            "processor_output_not_present",
            Some(output_path),
            Some(error.to_string()),
        ),
    }
}

pub(super) fn information_lifespan_processor_fallback(
    status: &'static str,
    output_path: Option<String>,
    error: Option<String>,
) -> Value {
    json!({
        "contract_version": "information_lifespan_hermes_processor_output.v1.local_api_fallback",
        "status": status,
        "output_path": output_path,
        "error": error,
        "simulated": true,
        "why_simulated": "The API replay can return the deterministic app trace without starting Docker; run scripts/demo_information_lifespan.sh to refresh the real processor artifact.",
        "processor": {
            "service": "hermes-processor",
            "runtime_status": "processor_artifact_unavailable_to_api"
        },
        "calculations": {
            "reported_estimated_labor_minutes_difference": 42
        },
        "final_report": {
            "artifact_ref": "artifact://manager-daily-report/synthetic-2026-06-29",
            "manager_actions": [
                "review demand against staffing plan",
                "review vaccine/care exception gates"
            ]
        },
        "live_side_effects_allowed": false,
        "review_gates": [
            {"gate": "provider_write_locked", "locked": true},
            {"gate": "customer_send_locked", "locked": true},
            {"gate": "medical_review_required", "locked": true},
            {"gate": "schedule_change_locked", "locked": true},
            {"gate": "payment_movement_locked", "locked": true}
        ]
    })
}

pub(super) fn information_lifespan_run_payload(
    workflow: &'static str,
    trace: &information_lifespan::TraceEnvelope,
    request_trace: &RequestTraceEvidence,
    processor_proof: Value,
) -> Value {
    json!({
        "api_contract": api_dto_contract(workflow),
        "correlation_id": trace.correlation_id().as_str(),
        "observability": workflow_observability_payload(trace.correlation_id().as_str(), request_trace),
        "trace": trace,
        "source_model_proof": {
            "source_system": trace.source_system(),
            "source_evidence": trace.source_evidence(),
            "provider_payloads_are_source_evidence_only": trace.provider_payloads_are_source_evidence_only()
        },
        "db_proof": trace.db_proof_entries(),
        "processor_proof": processor_proof,
        "calculations": trace.calculations(),
        "safety": {
            "synthetic_data_only": trace.uses_synthetic_data_only(),
            "live_side_effects_allowed": trace.live_side_effects_allowed(),
            "review_gates": trace.safety_gates(),
            "locked_side_effects": [
                "provider_pms_writes",
                "customer_sends",
                "medical_or_vaccine_acceptance",
                "schedule_or_staffing_changes",
                "payments_refunds_discounts"
            ]
        },
        "network_proof": trace.network_proof_entries(),
        "log_proof": trace.log_proof_entries(),
        "final_report": trace.final_artifact(),
        "deferred_production_work": [
            "live NVA/Gingr access remains intentionally absent; all source evidence is synthetic/mock read-only fixture data",
            "real customer sends, provider/PMS writes, payments/refunds/discounts, schedule changes, and medical/safety decisions remain locked behind human/system-of-record review gates",
            "the API replay reads the processor artifact when present and otherwise labels its local fallback instead of pretending Docker or an LLM ran"
        ]
    })
}
