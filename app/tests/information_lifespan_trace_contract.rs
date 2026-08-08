use app::information_lifespan as trace;

#[test]
fn deterministic_trace_fixture_covers_full_information_lifespan() {
    let envelope = trace::mock_gingr_manager_daily_report_trace();

    assert_eq!(
        envelope.correlation_id().as_str(),
        "info-lifespan-demo-2026-06-29"
    );
    assert_eq!(
        envelope.source_system(),
        trace::SourceSystem::MockGingrReadOnlyFixture
    );
    assert_eq!(envelope.stages().len(), 8);

    let stage_kinds: Vec<_> = envelope.stages().iter().map(|stage| stage.kind()).collect();
    assert_eq!(
        stage_kinds,
        vec![
            trace::StageKind::SourceEvidenceReceived,
            trace::StageKind::ProviderDtoPreserved,
            trace::StageKind::NormalizedNvaModels,
            trace::StageKind::DatabaseProjectionProof,
            trace::StageKind::HermesProcessorRun,
            trace::StageKind::CalculationApplied,
            trace::StageKind::ReviewGateLocked,
            trace::StageKind::ManagerDailyReportArtifact,
        ]
    );

    assert_eq!(envelope.source_payloads().len(), 3);
    assert!(
        envelope
            .source_payloads()
            .iter()
            .any(|payload| payload.payload_kind() == trace::SourcePayloadKind::Reservation)
    );
    assert!(
        envelope
            .source_payloads()
            .iter()
            .any(|payload| payload.payload_kind() == trace::SourcePayloadKind::CareNote)
    );
    assert!(
        envelope
            .source_payloads()
            .iter()
            .any(|payload| payload.payload_kind() == trace::SourcePayloadKind::Vaccine)
    );

    assert!(
        envelope
            .stages()
            .iter()
            .flat_map(|stage| stage.model_paths())
            .any(|path| path.path() == "gingr::response::ReservationRecord")
    );
    assert!(
        envelope
            .stages()
            .iter()
            .flat_map(|stage| stage.model_paths())
            .any(|path| path.path() == "app::manager_daily_brief::Packet")
    );
    assert!(envelope.network_proof_entries().iter().any(|entry| {
        entry.method() == trace::HttpMethod::Post
            && entry.path() == "/demo/information-lifespan/run"
    }));
    let database_stage = envelope
        .stages()
        .iter()
        .find(|stage| stage.kind() == trace::StageKind::DatabaseProjectionProof)
        .expect("trace has database/projection proof stage");
    assert!(
        !database_stage.simulated(),
        "Piece 2 wires DB refs into deterministic local Postgres seed rows"
    );
    assert!(database_stage.why_simulated().is_none());
    let hermes_stage = envelope
        .stages()
        .iter()
        .find(|stage| stage.kind() == trace::StageKind::HermesProcessorRun)
        .expect("trace has Hermes processor proof stage");
    assert!(
        !hermes_stage.simulated(),
        "Piece 3 adds a Docker Compose hermes-processor bridge with deterministic output"
    );
    assert!(hermes_stage.why_simulated().is_none());
    assert!(hermes_stage.model_paths().iter().any(|path| {
        path.path() == "apps::hermes_processor::processor"
            && path.role().contains("Docker Compose service")
    }));
    assert!(envelope.db_proof_entries().len() >= 6);
    assert!(envelope.db_proof_entries().iter().any(|entry| {
        entry.table_or_view() == "information_lifespan_db_lifecycle_proof"
            && entry.proof_ref() == "correlation_id:info-lifespan-demo-2026-06-29"
    }));
    assert!(envelope.db_proof_entries().iter().any(|entry| {
        entry.table_or_view() == "manager_daily_brief_outcomes"
            && entry.proof_ref().contains("info-lifespan-demo-2026-06-29")
    }));
    assert!(envelope.calculations().iter().any(|calculation| {
        calculation.name() == "estimated_labor_minutes_saved" && calculation.result() == "42"
    }));
    assert!(envelope.safety_gates().iter().all(|gate| gate.locked()));
    assert!(envelope.safety_gates().iter().any(|gate| {
        gate.gate() == trace::SafetyGateKind::ProviderWriteLocked && gate.locked()
    }));
    let artifact_stage = envelope
        .stages()
        .iter()
        .find(|stage| stage.kind() == trace::StageKind::ManagerDailyReportArtifact)
        .expect("trace has final Manager Daily Report artifact stage");
    assert!(
        !artifact_stage.simulated(),
        "Piece 4 exposes the final artifact through the local API/report payload"
    );
    assert!(artifact_stage.why_simulated().is_none());
    assert!(artifact_stage.model_paths().iter().any(|path| {
        path.path() == "apps::api::http::information_lifespan_run_payload"
            && path.role().contains("API renderer")
    }));
    assert_eq!(
        envelope.final_artifact().artifact_kind(),
        trace::FinalArtifactKind::ManagerDailyReport
    );
}

#[test]
fn mocked_gingr_payloads_remain_source_evidence_not_product_truth() {
    let envelope = trace::mock_gingr_manager_daily_report_trace();

    assert!(envelope.uses_synthetic_data_only());
    assert!(envelope.provider_payloads_are_source_evidence_only());
    assert!(!envelope.live_side_effects_allowed());

    for source_payload in envelope.source_payloads() {
        assert_eq!(
            source_payload.authority(),
            trace::PayloadAuthority::ProviderEvidenceOnly
        );
        assert!(
            source_payload
                .raw_payload_ref()
                .starts_with("fixture://mock-gingr/")
        );
    }

    let serialized =
        serde_json::to_value(&envelope).expect("trace fixture serializes for API/UI reuse");
    assert_eq!(
        serialized["correlation_id"],
        serde_json::Value::String("info-lifespan-demo-2026-06-29".to_owned())
    );
    assert_eq!(
        serialized["live_side_effects_allowed"],
        serde_json::Value::Bool(false)
    );
    assert_eq!(
        serialized["source_payloads"][0]["payload"]["synthetic"],
        true
    );
    assert_eq!(
        serialized["final_artifact"]["title"],
        serde_json::Value::String("Manager Daily Report — synthetic 2026-06-29".to_owned())
    );
}
