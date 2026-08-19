use app::information_lifespan as trace;

#[test]
fn trace_component_accessors_preserve_exact_public_proof_values() {
    let model = trace::ModelPath::new("domain::reservation::StayFact", "normalized fact");
    assert_eq!(model.path(), "domain::reservation::StayFact");
    assert_eq!(model.role(), "normalized fact");

    let stage = trace::TraceStage::new(
        trace::StageKind::ProviderDtoPreserved,
        "Source DTO",
        true,
        Some("fixture only".to_owned()),
        vec![model],
    );
    assert_eq!(stage.kind(), trace::StageKind::ProviderDtoPreserved);
    assert_eq!(stage.title(), "Source DTO");
    assert!(stage.simulated());
    assert_eq!(stage.why_simulated(), Some("fixture only"));
    assert_eq!(stage.model_paths().len(), 1);

    let log = trace::LogProofEntry::new("info", "trace", "normalized");
    assert_eq!(log.level(), "info");
    assert_eq!(log.target(), "trace");
    assert_eq!(log.message(), "normalized");
    let db = trace::DbProofEntry::new("review_packets", "packet-7", "storage::ReviewPacket");
    assert_eq!(db.table_or_view(), "review_packets");
    assert_eq!(db.proof_ref(), "packet-7");
    assert_eq!(db.model_path(), "storage::ReviewPacket");
    let network =
        trace::NetworkProofEntry::new(trace::HttpMethod::Get, "/api/report/7", 200, "response-7");
    assert_eq!(network.method(), trace::HttpMethod::Get);
    assert_eq!(network.path(), "/api/report/7");
    assert_eq!(network.status(), 200);
    assert_eq!(network.response_ref(), "response-7");
    let calculation = trace::CalculationProof::new("labor", "10 - 3", "7");
    assert_eq!(calculation.name(), "labor");
    assert_eq!(calculation.expression(), "10 - 3");
    assert_eq!(calculation.result(), "7");
}

#[test]
fn observed_source_evidence_remains_non_authoritative_inside_the_trace_envelope() {
    let evidence = trace::ObservedSourceEvidence::new(
        trace::SourceEvidenceKind::Reservation,
        "reservation:7",
        serde_json::json!({"provider_status": "booked"}),
    );

    assert_eq!(
        evidence.evidence_kind(),
        trace::SourceEvidenceKind::Reservation
    );
    assert_eq!(evidence.source_ref(), "reservation:7");
    assert_eq!(evidence.value()["provider_status"], "booked");
    assert_eq!(
        format!("{evidence:?}"),
        "ObservedSourceEvidence([REDACTED])"
    );

    let correlation_id = trace::CorrelationId::new("trace-7");
    let envelope = trace::TraceEnvelope::builder()
        .schema_version(trace::TraceSchemaVersion::V1)
        .correlation_id(correlation_id.clone())
        .source_system(trace::SourceSystem::MockProviderReadOnlyFixture)
        .synthetic_data_only(true)
        .provider_payloads_are_source_evidence_only(true)
        .live_side_effects_allowed(false)
        .source_evidence(vec![evidence])
        .stages(Vec::new())
        .log_proof_entries(Vec::new())
        .db_proof_entries(Vec::new())
        .network_proof_entries(Vec::new())
        .calculations(Vec::new())
        .safety_gates(Vec::new())
        .final_artifact(trace::FinalArtifact::new(
            trace::FinalArtifactKind::ManagerDailyReport,
            "Manager Daily Report",
            "report:7",
            "Synthetic report",
        ))
        .build();

    assert_eq!(envelope.schema_version(), trace::TraceSchemaVersion::V1);
    assert_eq!(envelope.correlation_id(), &correlation_id);
    assert_eq!(
        envelope.source_system(),
        trace::SourceSystem::MockProviderReadOnlyFixture
    );
    assert!(envelope.uses_synthetic_data_only());
    assert!(envelope.provider_payloads_are_source_evidence_only());
    assert!(!envelope.live_side_effects_allowed());
    assert_eq!(envelope.source_evidence().len(), 1);
    assert!(envelope.stages().is_empty());
    assert!(envelope.log_proof_entries().is_empty());
    assert!(envelope.db_proof_entries().is_empty());
    assert!(envelope.network_proof_entries().is_empty());
    assert!(envelope.calculations().is_empty());
    assert!(envelope.safety_gates().is_empty());
    assert_eq!(
        envelope.final_artifact().artifact_kind(),
        trace::FinalArtifactKind::ManagerDailyReport
    );
}

#[test]
fn legacy_v0_trace_schema_is_not_a_current_contract() {
    let error =
        serde_json::from_str::<trace::TraceSchemaVersion>("\"information_lifespan.trace.v0\"")
            .expect_err("legacy v0 traces must not deserialize as the current schema");

    assert!(error.to_string().contains("information_lifespan_trace.v1"));
}
