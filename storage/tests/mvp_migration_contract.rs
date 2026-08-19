const MVP_MIGRATION: &str = include_str!("../../migrations/0001_mvp_foundation.sql");

#[test]
fn clean_slate_schema_has_one_canonical_migration_without_upgrade_bridges() {
    let migration_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../migrations");
    let migrations = std::fs::read_dir(migration_dir)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "sql")
        })
        .collect::<Vec<_>>();

    assert_eq!(
        migrations.len(),
        1,
        "clean-slate databases have one schema owner"
    );
    assert!(!MVP_MIGRATION.contains("ALTER TABLE"));
    assert!(!MVP_MIGRATION.contains("legacy_persona_missing"));
    assert!(!MVP_MIGRATION.contains("resolution_status_after_review"));
}

use storage::operations::ManagerDailyBriefActionKindCode;
use strum::VariantArray;

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
    assert!(MVP_MIGRATION.contains("decided_by_actor_persona IS NOT NULL"));
    assert!(MVP_MIGRATION.contains("approval_records_decider_persona_kind_integrity"));
    assert!(MVP_MIGRATION.contains("approval_outbox_bindings_actor_persona_kind_integrity"));
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
    assert!(MVP_MIGRATION.contains(
        "cannot change approval after an approval_outbox_binding or outbox_record exists"
    ));
    assert!(MVP_MIGRATION.contains("outbox_records_status_timestamp_integrity"));
    assert!(MVP_MIGRATION.contains("outbox_records_idempotency_key_key"));
}

#[test]
fn mvp_migration_relationally_closes_and_one_shot_binds_internal_outbox_admission() {
    assert!(MVP_MIGRATION.contains("approval_outbox_bindings"));
    assert!(MVP_MIGRATION.contains("outbox_records_approval_record_id_key"));
    assert!(MVP_MIGRATION.contains("outbox_internal_handoff_topic_is_closed"));
    assert!(MVP_MIGRATION.contains("enforce_outbox_internal_handoff_binding"));
    assert!(MVP_MIGRATION.contains("binding.topic <> NEW.topic"));
    assert!(MVP_MIGRATION.contains("binding.payload <> NEW.payload"));
    assert!(MVP_MIGRATION.contains("reject_approval_outbox_binding_mutation"));
    assert!(MVP_MIGRATION.contains("reject_outbox_authority_identity_mutation"));
    assert!(MVP_MIGRATION.contains("outbox_records_durable_history_delete"));
    assert!(MVP_MIGRATION.contains("FOR UPDATE"));
    assert!(MVP_MIGRATION.contains("AFTER INSERT ON outbox_records"));
    assert!(!MVP_MIGRATION.contains("pg_trigger_depth()"));
}

#[test]
fn mvp_migration_freezes_approval_relation_after_binding_or_outbox_admission() {
    let guard = MVP_MIGRATION
        .split("CREATE OR REPLACE FUNCTION prevent_approval_change_with_open_outbox_records()")
        .nth(1)
        .expect("approval immutability trigger function must exist")
        .split("CREATE TABLE IF NOT EXISTS audit_events")
        .next()
        .expect("guard function must precede audit events");

    assert!(guard.contains("WHERE approval_record_id = OLD.id"));
    assert!(!guard.contains("status IN ('pending', 'claimed')"));
    assert!(guard.contains(
        "cannot change approval after an approval_outbox_binding or outbox_record exists"
    ));
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
        "reported_estimated_minutes_difference integer NOT NULL CHECK (reported_estimated_minutes_difference >= 0)",
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
fn mvp_migration_supports_data_quality_hygiene_reviewed_internal_handoff_slice() {
    for invariant in [
        "workflow_name text NOT NULL CHECK (length(trim(workflow_name)) > 0)",
        "event_kind text NOT NULL CHECK (length(trim(event_kind)) > 0)",
        "idempotency_key text NOT NULL UNIQUE",
        "workflow_event_id uuid REFERENCES workflow_events(id)",
        "issue_refs jsonb NOT NULL DEFAULT '[]'::jsonb",
        "reported_resolution_status text NOT NULL CHECK (reported_resolution_status IN",
        "correlation_id text NOT NULL CHECK (length(trim(correlation_id)) > 0)",
        "status text NOT NULL CHECK (status IN ('pending', 'claimed', 'published', 'failed', 'dead_letter'))",
        "outbox_records require a matching approved approval_record",
    ] {
        assert!(
            MVP_MIGRATION.contains(invariant),
            "migration must support reviewed data-quality handoff invariant: {invariant}"
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

#[test]
fn manager_daily_brief_action_codes_have_exact_rust_sql_parity() {
    let action_constraint = MVP_MIGRATION
        .split("CREATE TABLE IF NOT EXISTS manager_daily_brief_outcomes")
        .nth(1)
        .expect("manager daily brief table exists")
        .split("before_minutes")
        .next()
        .expect("action constraint precedes labor evidence");

    for rust_code in ManagerDailyBriefActionKindCode::VARIANTS {
        let sql_literal = format!("'{}'", rust_code);
        assert!(
            action_constraint.contains(&sql_literal),
            "Rust-valid action code {rust_code} must be accepted by SQL"
        );
    }
}

#[test]
fn outcome_approval_evidence_is_exact_action_bound_approved_and_immutable() {
    for invariant in [
        "enforce_outcome_approval_workflow_binding",
        "approval.status <> 'approved'",
        "approval_review_packet.status <> 'approved'",
        "approval_review_packet.workflow_event_id <> NEW.workflow_event_id",
        "approval_review_packet.reviewed_action_id <> NEW.action_id",
        "approval.target_kind <> approval_review_packet.subject_kind",
        "approval.target_id <> approval_review_packet.subject_id",
        "approval.gate <> approval_review_packet.gate",
        "reviewed_workflow_event.workflow_name = 'manager_daily_brief'",
        "reviewed_workflow_event.workflow_name = 'information_lifespan_manager_daily_report'",
        "reviewed_workflow_event.workflow_name <> 'data_quality_hygiene'",
        "FOR UPDATE",
        "approval_records_outcome_lineage_immutable",
        "review_packets_outcome_lineage_immutable",
        "workflow_events_outcome_lineage_immutable",
        "schema_version integer NOT NULL DEFAULT 1 CHECK (schema_version = 1)",
        "approval.decided_at > NEW.recorded_at",
        "approval.decided_by_actor_id <> NEW.actor_id",
        "prevent_reviewed_outcome_mutation",
        "manager_daily_brief_outcomes_immutable",
        "data_quality_hygiene_outcomes_immutable",
        "manager_daily_brief_outcomes_approval_workflow_binding",
        "data_quality_hygiene_outcomes_approval_workflow_binding",
    ] {
        assert!(
            MVP_MIGRATION.contains(invariant),
            "missing exact outcome authority invariant: {invariant}"
        );
    }
}

#[test]
fn migration_explicitly_preserves_payment_projection_and_workflow_result_history() {
    assert!(
        MVP_MIGRATION
            .contains("payment_deposit_projections retain provider-observed state history")
    );
    assert!(MVP_MIGRATION.contains("workflow_results retain one row per processing attempt"));
}

#[test]
fn hygiene_outcomes_use_current_reported_resolution_vocabulary() {
    assert!(MVP_MIGRATION.contains("reported_resolution_status text NOT NULL"));
}
