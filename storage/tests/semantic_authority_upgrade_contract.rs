use std::{fs, path::PathBuf};

fn migration() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join("../migrations/0003_semantic_authority_upgrade.sql"))
        .expect("semantic authority upgrade migration must remain readable")
}

#[test]
fn semantic_authority_upgrade_adds_current_columns_and_constraints_to_existing_schemas() {
    let sql = migration();

    for required in [
        "ALTER TABLE review_packets",
        "ADD COLUMN IF NOT EXISTS reviewed_action_id",
        "ALTER TABLE approval_records",
        "ADD COLUMN IF NOT EXISTS decided_by_actor_persona",
        "ADD COLUMN IF NOT EXISTS legacy_persona_missing boolean",
        "WHERE legacy_persona_missing IS NULL",
        "approval_records_legacy_persona_marker_guard",
        "legacy approval persona marker is migration-owned and immutable",
        "Persona-less terminal rows from the deployed legacy shape remain historical",
        "ADD COLUMN IF NOT EXISTS authorized_by_actor_persona",
        "approval_record.decided_by_actor_persona IS DISTINCT FROM binding.authorized_by_actor_persona",
        "approval_record.decided_by_actor_persona <> NEW.authorized_by_actor_persona",
        "OLD.authorized_by_actor_persona <> NEW.authorized_by_actor_persona",
        "NEW.decided_by_actor_persona IS DISTINCT FROM OLD.decided_by_actor_persona",
        "decided_by_actor_id, decided_by_actor_persona, decided_at ON approval_records",
        "approval_records_decider_persona_kind_integrity",
        "approval_outbox_bindings_actor_persona_kind_integrity",
        "ALTER TABLE manager_daily_brief_outcomes",
        "ALTER TABLE data_quality_hygiene_outcomes",
        "ADD COLUMN IF NOT EXISTS schema_version integer NOT NULL DEFAULT 1",
        "ALTER COLUMN location_id SET NOT NULL",
        "status NOT IN ('approved', 'rejected')",
        "requested_at <= decided_at",
        "review_capacity_labor_recommendation",
        "UPDATE manager_daily_brief_outcomes",
        "UPDATE data_quality_hygiene_outcomes",
        "SET workflow_event_id = workflow_event_id",
    ] {
        assert!(
            sql.contains(required),
            "forward migration must retain required upgrade fragment: {required}"
        );
    }
}

#[test]
fn semantic_authority_upgrade_installs_exact_lineage_and_immutability_guards_idempotently() {
    let sql = migration();

    for trigger in [
        "manager_daily_brief_outcomes_approval_workflow_binding",
        "data_quality_hygiene_outcomes_approval_workflow_binding",
        "manager_daily_brief_outcomes_immutable",
        "data_quality_hygiene_outcomes_immutable",
        "approval_records_outcome_lineage_immutable",
        "review_packets_outcome_lineage_immutable",
        "workflow_events_outcome_lineage_immutable",
    ] {
        assert!(
            sql.contains(&format!("DROP TRIGGER IF EXISTS {trigger}")),
            "forward migration must replace {trigger} idempotently"
        );
        assert!(
            sql.contains(&format!("CREATE TRIGGER {trigger}")),
            "forward migration must install {trigger}"
        );
    }

    for required in [
        "FOR UPDATE",
        "approval_review_packet.reviewed_action_id <> NEW.action_id",
        "approval.decided_by_actor_persona <> NEW.actor_persona",
        "reviewed_workflow_event.payload->'source_refs' IS DISTINCT FROM NEW.source_refs",
        "reviewed outcome evidence is immutable after admission",
    ] {
        assert!(
            sql.contains(required),
            "forward migration must preserve authority invariant: {required}"
        );
    }
}
