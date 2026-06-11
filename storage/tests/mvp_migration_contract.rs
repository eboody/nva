const MVP_MIGRATION: &str = include_str!("../../migrations/0001_mvp_foundation.sql");

#[test]
fn mvp_migration_defines_core_tables_and_relationships() {
    for table in [
        "customers",
        "pets",
        "reservations",
        "documents",
        "vaccine_records",
        "vaccine_extractions",
        "pet_eligibility_projections",
        "operational_tasks",
        "care_notes",
        "incidents",
        "messages",
        "payment_deposit_projections",
        "workflow_events",
        "workflow_results",
        "review_packets",
        "approval_records",
        "outbox_records",
        "object_metadata",
        "audit_events",
    ] {
        assert!(
            MVP_MIGRATION.contains(&format!("CREATE TABLE IF NOT EXISTS {table}")),
            "missing core MVP table {table}"
        );
    }

    assert!(MVP_MIGRATION.contains("REFERENCES customers"));
    assert!(MVP_MIGRATION.contains("REFERENCES pets"));
    assert!(MVP_MIGRATION.contains("REFERENCES reservations"));
    assert!(MVP_MIGRATION.contains("REFERENCES documents"));
    assert!(MVP_MIGRATION.contains("vaccine_extraction_schema_version"));
    assert!(MVP_MIGRATION.contains("medical_document_uncertainty_policy_requires_staff_review"));
}

#[test]
fn mvp_migration_represents_audit_and_workflow_write_paths() {
    assert!(MVP_MIGRATION.contains("CREATE TRIGGER audit_events_append_only_update"));
    assert!(MVP_MIGRATION.contains("CREATE TRIGGER audit_events_append_only_delete"));
    assert!(MVP_MIGRATION.contains("RAISE EXCEPTION 'audit_events is append-only'"));
    assert!(MVP_MIGRATION.contains("outbox_records"));
    assert!(MVP_MIGRATION.contains("workflow_events"));
    assert!(MVP_MIGRATION.contains("workflow_results"));
    assert!(MVP_MIGRATION.contains("approval_records"));
}

#[test]
fn mvp_migration_persists_all_canonical_review_gates() {
    for persisted_gate in [
        "manager_approval",
        "medical_document_review",
        "behavior_review",
        "customer_message_approval",
        "refund_or_deposit_exception",
    ] {
        assert!(
            MVP_MIGRATION.contains(persisted_gate),
            "migration must accept canonical review gate {persisted_gate}"
        );
    }
}

#[test]
fn mvp_migration_rejects_invalid_incident_review_gates_and_incoherent_approval_decisions() {
    assert!(MVP_MIGRATION.contains("incidents_required_review_gates_valid"));
    assert!(
        MVP_MIGRATION.contains("WHERE review_gate_is_valid(required_review_gate.gate) IS NOT TRUE")
    );
    assert!(MVP_MIGRATION.contains("approval_records_decision_integrity"));
    assert!(MVP_MIGRATION.contains("status IN ('approved', 'rejected')"));
    assert!(MVP_MIGRATION.contains("decided_by_actor_kind IS NOT NULL"));
    assert!(MVP_MIGRATION.contains("decided_at IS NOT NULL"));
}
