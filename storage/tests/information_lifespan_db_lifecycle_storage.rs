const FOUNDATION_MIGRATION: &str = include_str!("../../migrations/0001_mvp_foundation.sql");
const DATA_QUALITY_MIGRATION: &str = FOUNDATION_MIGRATION;
const LOCAL_DEMO_SEED: &str = include_str!("../../fixtures/seed/local-demo.sql");

#[test]
fn information_lifespan_projection_exposes_correlation_linked_db_lifecycle() {
    assert!(
        DATA_QUALITY_MIGRATION
            .contains("CREATE OR REPLACE VIEW information_lifespan_db_lifecycle_proof"),
        "Piece 2 needs a queryable lifecycle proof view rather than only prose refs"
    );

    for column_or_join in [
        "we.payload->>'correlation_id' AS correlation_id",
        "we.payload->>'source_import_run_id' AS source_import_run_id",
        "sir.source_system",
        "source_quality_issue_refs",
        "manager_daily_brief_outcome_id",
        "review_packet_id",
        "approval_record_id",
        "audit_event_ids",
        "information_lifespan_db_lifecycle_proof.v1",
    ] {
        assert!(
            DATA_QUALITY_MIGRATION.contains(column_or_join),
            "information-lifespan DB proof view must expose {column_or_join}"
        );
    }
}

#[test]
fn local_demo_seed_writes_information_lifespan_rows_for_each_lifecycle_table() {
    for expected in [
        "info-lifespan-demo-2026-06-29",
        "source_import_run:info-lifespan-demo-2026-06-29",
        "workflow_event:manager-daily-report:2026-06-29",
        "review_packet:vaccine-near-expiry:8101",
        "manager_daily_brief_outcome:synthetic-2026-06-29",
        "audit_lineage:info-lifespan-demo-2026-06-29",
        "information_lifespan_db_lifecycle_proof",
        "raw_provider_payloads_redacted_or_referenced_only",
        "live_side_effects_allowed', false",
    ] {
        assert!(
            LOCAL_DEMO_SEED.contains(expected),
            "local demo seed must contain deterministic information-lifespan proof token {expected}"
        );
    }

    for write_target in [
        "INSERT INTO source_import_runs",
        "INSERT INTO source_quality_issues",
        "INSERT INTO workflow_events",
        "INSERT INTO workflow_results",
        "INSERT INTO review_packets",
        "INSERT INTO approval_records",
        "INSERT INTO manager_daily_brief_outcomes",
        "INSERT INTO audit_events",
    ] {
        assert!(
            LOCAL_DEMO_SEED.contains(write_target),
            "local demo seed must write/read through {write_target} for Piece 2 proof"
        );
    }
}

#[test]
fn foundation_tables_already_hold_review_audit_and_report_outcome_rows() {
    for existing_table in [
        "CREATE TABLE IF NOT EXISTS workflow_events",
        "CREATE TABLE IF NOT EXISTS workflow_results",
        "CREATE TABLE IF NOT EXISTS review_packets",
        "CREATE TABLE IF NOT EXISTS approval_records",
        "CREATE TABLE IF NOT EXISTS audit_events",
        "CREATE TABLE IF NOT EXISTS manager_daily_brief_outcomes",
    ] {
        assert!(
            FOUNDATION_MIGRATION.contains(existing_table),
            "Piece 2 should reuse existing schema surface: {existing_table}"
        );
    }
}
