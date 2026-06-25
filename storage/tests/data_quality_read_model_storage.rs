use storage::operations::{
    AffectedEntityKindCode, DataQualityFreshnessCode, DataQualityIssueKindCode,
    DataQualityIssueRecord, DataQualitySensitivityCode, DataQualitySeverityCode,
    DataQualitySourceImportModeCode, DataQualitySourceImportRunRecord,
    DataQualitySourceImportStatusCode, DataQualitySyncGapKindCode, DataQualitySyncGapRecord,
    DataQualitySyncGapStatusCode, DataQualityWorkflowBlockingCode, ImportFreshnessRow,
    ReviewGateCode, SourceQualityBacklogRow, StoredSourceRecordRef,
};
use strum::VariantArray;

const DATA_QUALITY_READ_MODEL_MIGRATION: &str =
    include_str!("../../migrations/0002_data_quality_read_models.sql");

#[test]
fn data_quality_read_model_migration_declares_durable_source_quality_tables_and_views() {
    for table in ["source_import_runs", "source_quality_issues", "sync_gaps"] {
        assert!(
            DATA_QUALITY_READ_MODEL_MIGRATION
                .contains(&format!("CREATE TABLE IF NOT EXISTS {table}")),
            "missing durable data-quality table {table}"
        );
    }

    for view in [
        "source_quality_backlog",
        "data_quality_hygiene_labor_outcomes",
        "audit_lineage",
        "import_freshness",
    ] {
        assert!(
            DATA_QUALITY_READ_MODEL_MIGRATION.contains(&format!("CREATE OR REPLACE VIEW {view}")),
            "missing data-quality BI read model {view}"
        );
    }

    for invariant in [
        "provider_writes_allowed=false",
        "customer_messages_allowed=false",
        "live_delivery_allowed=false",
        "raw provider payloads are redacted or referenced, not exposed",
        "source_refs jsonb NOT NULL DEFAULT '[]'::jsonb",
        "review_gate text NOT NULL CHECK (review_gate_is_valid(review_gate))",
        "workflow_blocking text NOT NULL CHECK (workflow_blocking IN ('blocking', 'non_blocking'))",
        "resolution_status text NOT NULL CHECK (resolution_status IN",
        "latest_outcome_id",
        "projection_version",
    ] {
        assert!(
            DATA_QUALITY_READ_MODEL_MIGRATION.contains(invariant),
            "migration must preserve data-quality read-model invariant: {invariant}"
        );
    }
}

#[test]
fn data_quality_issue_record_codecs_preserve_bi_backlog_dimensions_and_lineage() {
    let issue = source_quality_issue_record();

    assert_eq!(issue.issue_ref, "dq-missing-vaccine-42");
    assert_eq!(
        issue.workflow_blocking,
        DataQualityWorkflowBlockingCode::Blocking
    );
    assert_eq!(issue.review_gate, ReviewGateCode::ManagerApproval);
    assert_eq!(issue.source_refs.len(), 1);

    let decoded = DataQualityIssueRecord::decode_json(&issue.encode_json().unwrap()).unwrap();
    assert_eq!(decoded, issue);
}

#[test]
fn source_quality_backlog_row_exposes_bi_safe_dimensions_without_raw_provider_payloads() {
    let issue = source_quality_issue_record();
    let backlog_row = SourceQualityBacklogRow::from_issue(
        issue,
        Some("dqh-outcome-1".to_owned()),
        "source_quality_backlog.v1".to_owned(),
        vec![
            "raw_payload_redacted".to_owned(),
            "live_side_effects_disabled".to_owned(),
        ],
    );

    assert_eq!(backlog_row.location_id, "location-1");
    assert_eq!(
        backlog_row.issue_kind,
        DataQualityIssueKindCode::MissingSourceEvidence
    );
    assert_eq!(backlog_row.severity, DataQualitySeverityCode::High);
    assert_eq!(backlog_row.freshness, DataQualityFreshnessCode::Stale);
    assert_eq!(
        backlog_row.sensitivity,
        DataQualitySensitivityCode::MedicalOrVaccination
    );
    assert_eq!(
        backlog_row.latest_outcome_id.as_deref(),
        Some("dqh-outcome-1")
    );
    assert_eq!(backlog_row.source_refs[0].record_id, "pet-vaccine-42");
    assert!(
        backlog_row
            .caveats
            .contains(&"raw_payload_redacted".to_owned())
    );
    assert!(
        backlog_row
            .caveats
            .contains(&"live_side_effects_disabled".to_owned())
    );
}

#[test]
fn import_freshness_row_caveats_failed_or_rejected_source_imports() {
    let import_run = DataQualitySourceImportRunRecord::builder()
        .id("import-run-1".to_owned())
        .source_system("gingr".to_owned())
        .adapter_version("gingr-v0-readonly".to_owned())
        .location_id("location-1".to_owned())
        .mode(DataQualitySourceImportModeCode::ReadOnlySnapshot)
        .status(DataQualitySourceImportStatusCode::CompletedWithRejections)
        .started_at("2026-06-17T00:00:00Z".to_owned())
        .completed_at("2026-06-17T00:05:00Z".to_owned())
        .record_count(200)
        .rejected_count(3)
        .safe_error_class("redacted_validation_error".to_owned())
        .redaction_posture("raw_payload_redacted".to_owned())
        .created_at("2026-06-17T00:05:01Z".to_owned())
        .build();
    let sync_gap = DataQualitySyncGapRecord::builder()
        .id("sync-gap-1".to_owned())
        .source_system("gingr".to_owned())
        .source_ref(source_ref())
        .location_id("location-1".to_owned())
        .gap_kind(DataQualitySyncGapKindCode::MissingExpectedRecord)
        .severity(DataQualitySeverityCode::Medium)
        .detected_at("2026-06-17T00:06:00Z".to_owned())
        .age_seconds(3600)
        .status(DataQualitySyncGapStatusCode::Open)
        .workflow_event_id("workflow-event-1".to_owned())
        .safe_error_class("redacted_missing_record".to_owned())
        .created_at("2026-06-17T00:06:00Z".to_owned())
        .updated_at("2026-06-17T00:06:00Z".to_owned())
        .build();

    let freshness = ImportFreshnessRow::from_import_runs_and_sync_gaps(
        "gingr",
        "location-1",
        &[import_run],
        &[sync_gap],
        "import_freshness.v1".to_owned(),
    );

    assert_eq!(freshness.source_system, "gingr");
    assert_eq!(
        freshness.last_completed_at.as_deref(),
        Some("2026-06-17T00:05:00Z")
    );
    assert_eq!(freshness.rejected_count, 3);
    assert_eq!(freshness.open_gap_count, 1);
    assert_eq!(
        freshness.adapter_version.as_deref(),
        Some("gingr-v0-readonly")
    );
    assert!(
        freshness
            .caveats
            .contains(&"source_import_had_rejections".to_owned())
    );
    assert!(freshness.caveats.contains(&"open_sync_gaps".to_owned()));
}

#[test]
fn data_quality_read_model_codes_roundtrip_through_strum_variant_metadata() {
    for code in DataQualityIssueKindCode::VARIANTS {
        assert_eq!(code.to_string().parse(), Ok(*code));
    }
    for code in DataQualitySeverityCode::VARIANTS {
        assert_eq!(code.to_string().parse(), Ok(*code));
    }
    for code in DataQualityFreshnessCode::VARIANTS {
        assert_eq!(code.to_string().parse(), Ok(*code));
    }
    for code in DataQualitySensitivityCode::VARIANTS {
        assert_eq!(code.to_string().parse(), Ok(*code));
    }
    for code in AffectedEntityKindCode::VARIANTS {
        assert_eq!(code.to_string().parse(), Ok(*code));
    }
    for code in DataQualityWorkflowBlockingCode::VARIANTS {
        assert_eq!(code.to_string().parse(), Ok(*code));
    }
    for code in DataQualitySourceImportStatusCode::VARIANTS {
        assert_eq!(code.to_string().parse(), Ok(*code));
    }
    for code in DataQualitySyncGapStatusCode::VARIANTS {
        assert_eq!(code.to_string().parse(), Ok(*code));
    }
}

fn source_quality_issue_record() -> DataQualityIssueRecord {
    DataQualityIssueRecord::builder()
        .issue_ref("dq-missing-vaccine-42".to_owned())
        .location_id("location-1".to_owned())
        .affected_entity_kind(AffectedEntityKindCode::Pet)
        .affected_entity_id("pet-42".to_owned())
        .field_path("vaccination.rabies.expires_on".to_owned())
        .issue_kind(DataQualityIssueKindCode::MissingSourceEvidence)
        .severity(DataQualitySeverityCode::High)
        .freshness(DataQualityFreshnessCode::Stale)
        .sensitivity(DataQualitySensitivityCode::MedicalOrVaccination)
        .workflow_blocking(DataQualityWorkflowBlockingCode::Blocking)
        .owner_persona("front_desk_lead".to_owned())
        .review_gate(ReviewGateCode::ManagerApproval)
        .resolution_status(storage::operations::DataQualityResolutionStatusCode::Open)
        .source_refs(vec![source_ref()])
        .workflow_event_id("workflow-event-1".to_owned())
        .created_at("2026-06-17T00:00:00Z".to_owned())
        .updated_at("2026-06-17T00:00:00Z".to_owned())
        .build()
}

fn source_ref() -> StoredSourceRecordRef {
    StoredSourceRecordRef::builder()
        .system("gingr".to_owned())
        .record_type("pet_vaccination".to_owned())
        .record_id("pet-vaccine-42".to_owned())
        .observed_at("2026-06-17T00:00:00Z".to_owned())
        .adapter_version("gingr-v0-readonly".to_owned())
        .build()
}
