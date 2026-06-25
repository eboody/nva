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
        "manager_daily_brief_outcomes",
        "data_quality_hygiene_outcomes",
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
fn mvp_migration_documents_durable_processing_lineage_without_live_side_effects() {
    for invariant in [
        "workflow_events are accepted before worker processing",
        "workflow_results are reviewable worker output, not execution proof",
        "outbox_records require a matching approved approval_record",
        "audit_events preserve workflow/outbox lineage append-only",
        "worker MVP remains fake deterministic and side-effect stubbed",
    ] {
        assert!(
            MVP_MIGRATION.contains(invariant),
            "migration should document durable processing invariant: {invariant}"
        );
    }
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
    assert!(MVP_MIGRATION.contains("decided_by_actor_id IS NOT NULL"));
    assert!(MVP_MIGRATION.contains("decided_at IS NOT NULL"));
}

#[test]
fn mvp_migration_prevents_autonomous_outbox_side_effects_without_approved_review() {
    assert!(
        MVP_MIGRATION.contains("approval_record_id uuid NOT NULL REFERENCES approval_records(id)")
    );
    assert!(MVP_MIGRATION.contains("enforce_outbox_approval_record_is_approved"));
    assert!(MVP_MIGRATION.contains("approval_record.status <> 'approved'"));
    assert!(MVP_MIGRATION.contains("approval_record.target_kind <> NEW.aggregate_kind"));
    assert!(MVP_MIGRATION.contains("approval_record.target_id <> NEW.aggregate_id"));
    assert!(MVP_MIGRATION.contains("approval_record.gate <> NEW.review_gate"));
    assert!(MVP_MIGRATION.contains("prevent_approval_change_with_open_outbox_records"));
    assert!(MVP_MIGRATION.contains("status IN ('pending', 'claimed')"));
    assert!(MVP_MIGRATION.contains("outbox_records_status_timestamp_integrity"));
    assert!(MVP_MIGRATION.contains("outbox_records_idempotency_key_key"));
}

#[test]
fn mvp_migration_persists_owned_outcome_and_labor_projections() {
    for table in [
        "manager_daily_brief_outcomes",
        "data_quality_hygiene_outcomes",
    ] {
        assert!(
            MVP_MIGRATION.contains(&format!("CREATE TABLE IF NOT EXISTS {table}")),
            "missing owned outcome/labor projection table {table}"
        );
    }

    for invariant in [
        "workflow_event_id uuid NOT NULL REFERENCES workflow_events(id)",
        "approval_record_id uuid NOT NULL REFERENCES approval_records(id)",
        "estimated_minutes_saved integer NOT NULL CHECK (estimated_minutes_saved >= 0)",
        "actual_minutes integer NOT NULL CHECK (actual_minutes > 0)",
        "source_refs jsonb NOT NULL DEFAULT '[]'::jsonb",
        "correlation_id text NOT NULL CHECK (length(trim(correlation_id)) > 0)",
        "action_kind text NOT NULL CHECK (action_kind IN",
        "owner_persona text NOT NULL CHECK (owner_persona IN",
    ] {
        assert!(
            MVP_MIGRATION.contains(invariant),
            "migration should preserve outcome/labor invariant: {invariant}"
        );
    }
}

#[test]
fn mvp_migration_declares_deferred_database_surfaces_without_pretending_live_access() {
    for deferred_surface in [
        "auth/session/role/location authorization",
        "infra metrics snapshots and dashboard read models",
        "durable job leasing and worker ownership",
        "real provider source snapshots",
    ] {
        assert!(
            MVP_MIGRATION.contains(deferred_surface),
            "migration comments should name deferred DB surface: {deferred_surface}"
        );
    }
}
