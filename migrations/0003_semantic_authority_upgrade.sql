BEGIN;

-- Forward-only semantic-authority upgrade for installations that already applied 0001.
-- Existing incompatible rows are rejected rather than assigned invented authority metadata.

ALTER TABLE review_packets
    ADD COLUMN IF NOT EXISTS reviewed_action_id text;
ALTER TABLE review_packets
    DROP CONSTRAINT IF EXISTS review_packets_reviewed_action_id_check;
ALTER TABLE review_packets
    ADD CONSTRAINT review_packets_reviewed_action_id_check
    CHECK (reviewed_action_id IS NULL OR length(trim(reviewed_action_id)) > 0);

ALTER TABLE approval_records
    ADD COLUMN IF NOT EXISTS decided_by_actor_persona text;
ALTER TABLE approval_records
    ADD COLUMN IF NOT EXISTS legacy_persona_missing boolean;
UPDATE approval_records
SET legacy_persona_missing = (
    status IN ('approved', 'rejected')
    AND decided_by_actor_persona IS NULL
)
WHERE legacy_persona_missing IS NULL;
ALTER TABLE approval_records
    ALTER COLUMN legacy_persona_missing SET DEFAULT false;
ALTER TABLE approval_records
    ALTER COLUMN legacy_persona_missing SET NOT NULL;
ALTER TABLE approval_records
    DROP CONSTRAINT IF EXISTS approval_records_decided_by_actor_persona_check;
ALTER TABLE approval_records
    ADD CONSTRAINT approval_records_decided_by_actor_persona_check
    CHECK (decided_by_actor_persona IN (
        'general_manager',
        'assistant_general_manager',
        'front_desk_lead',
        'front_desk_agent',
        'regional_operator',
        'operations_analyst'
    ));
ALTER TABLE approval_records
    DROP CONSTRAINT IF EXISTS approval_records_decision_integrity;
ALTER TABLE approval_records
    ADD CONSTRAINT approval_records_decision_integrity CHECK (
        (
            status IN ('approved', 'rejected')
            AND decided_by_actor_kind IS NOT NULL
            AND decided_by_actor_id IS NOT NULL
            AND (
                (decided_by_actor_persona IS NOT NULL AND NOT legacy_persona_missing)
                OR (decided_by_actor_persona IS NULL AND legacy_persona_missing)
            )
            AND decided_at IS NOT NULL
            AND requested_at <= decided_at
        )
        OR
        (
            status NOT IN ('approved', 'rejected')
            AND decided_by_actor_kind IS NULL
            AND decided_by_actor_id IS NULL
            AND decided_by_actor_persona IS NULL
            AND NOT legacy_persona_missing
            AND decided_at IS NULL
        )
    );
-- Persona-less terminal rows from the deployed legacy shape remain historical
-- evidence only. The migration-owned marker grandfathers only rows that existed
-- before this upgrade; new inserts and marker changes are rejected below.
ALTER TABLE approval_records
    DROP CONSTRAINT IF EXISTS approval_records_decider_persona_kind_integrity;
ALTER TABLE approval_records
    ADD CONSTRAINT approval_records_decider_persona_kind_integrity CHECK (
        legacy_persona_missing
        OR decided_by_actor_kind IS NULL
        OR (decided_by_actor_kind = 'manager' AND decided_by_actor_persona IN ('general_manager', 'assistant_general_manager', 'regional_operator'))
        OR (decided_by_actor_kind = 'staff' AND decided_by_actor_persona IN ('front_desk_lead', 'front_desk_agent', 'operations_analyst'))
    );

CREATE OR REPLACE FUNCTION reject_approval_legacy_persona_marker_forgery()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF (TG_OP = 'INSERT' AND NEW.legacy_persona_missing)
       OR (TG_OP = 'UPDATE' AND NEW.legacy_persona_missing IS DISTINCT FROM OLD.legacy_persona_missing)
    THEN
        RAISE EXCEPTION 'legacy approval persona marker is migration-owned and immutable';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS approval_records_legacy_persona_marker_guard ON approval_records;
CREATE TRIGGER approval_records_legacy_persona_marker_guard
    BEFORE INSERT OR UPDATE OF legacy_persona_missing ON approval_records
    FOR EACH ROW EXECUTE FUNCTION reject_approval_legacy_persona_marker_forgery();

-- Durable outbox admission must preserve the exact review persona as well as
-- actor kind/id. Existing bindings without this provenance are incompatible
-- legacy authority and deliberately make the upgrade fail closed.
ALTER TABLE approval_outbox_bindings
    ADD COLUMN IF NOT EXISTS authorized_by_actor_persona text;
ALTER TABLE approval_outbox_bindings
    DROP CONSTRAINT IF EXISTS approval_outbox_bindings_authorized_by_actor_persona_check;
ALTER TABLE approval_outbox_bindings
    ADD CONSTRAINT approval_outbox_bindings_authorized_by_actor_persona_check
    CHECK (authorized_by_actor_persona IN (
        'general_manager',
        'assistant_general_manager',
        'front_desk_lead',
        'front_desk_agent',
        'regional_operator',
        'operations_analyst'
    ));
ALTER TABLE approval_outbox_bindings
    DROP CONSTRAINT IF EXISTS approval_outbox_bindings_actor_persona_kind_integrity;
ALTER TABLE approval_outbox_bindings
    ADD CONSTRAINT approval_outbox_bindings_actor_persona_kind_integrity CHECK (
        (authorized_by_actor_kind = 'manager' AND authorized_by_actor_persona IN ('general_manager', 'assistant_general_manager', 'regional_operator'))
        OR (authorized_by_actor_kind = 'staff' AND authorized_by_actor_persona IN ('front_desk_lead', 'front_desk_agent', 'operations_analyst'))
    );

DO $$
DECLARE
    binding approval_outbox_bindings%ROWTYPE;
    approval_record approval_records%ROWTYPE;
BEGIN
    FOR binding IN SELECT * FROM approval_outbox_bindings LOOP
        SELECT * INTO approval_record
        FROM approval_records
        WHERE id = binding.approval_record_id;

        IF approval_record.id IS NULL
            OR approval_record.status <> 'approved'
            OR approval_record.decided_by_actor_persona IS DISTINCT FROM binding.authorized_by_actor_persona
        THEN
            RAISE EXCEPTION 'legacy approval_outbox_binding lacks exact approved reviewer persona provenance';
        END IF;
    END LOOP;
END;
$$;

ALTER TABLE approval_outbox_bindings
    ALTER COLUMN authorized_by_actor_persona SET NOT NULL;

CREATE OR REPLACE FUNCTION enforce_approval_outbox_binding_authority()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    approval_record approval_records%ROWTYPE;
BEGIN
    SELECT * INTO approval_record
    FROM approval_records
    WHERE id = NEW.approval_record_id
    FOR UPDATE;

    IF approval_record.id IS NULL
        OR approval_record.status <> 'approved'
        OR approval_record.target_kind <> NEW.aggregate_kind
        OR approval_record.target_id <> NEW.aggregate_id
        OR approval_record.gate <> NEW.review_gate
        OR approval_record.decided_by_actor_kind <> NEW.authorized_by_actor_kind
        OR approval_record.decided_by_actor_id <> NEW.authorized_by_actor_id
        OR approval_record.decided_by_actor_persona <> NEW.authorized_by_actor_persona
        OR approval_record.decided_at <> NEW.authorized_at
    THEN
        RAISE EXCEPTION 'approval_outbox_bindings require the matching approved decision authority';
    END IF;

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS approval_outbox_bindings_authority_guard ON approval_outbox_bindings;
CREATE TRIGGER approval_outbox_bindings_authority_guard
    BEFORE INSERT ON approval_outbox_bindings
    FOR EACH ROW EXECUTE FUNCTION enforce_approval_outbox_binding_authority();

CREATE OR REPLACE FUNCTION reject_approval_outbox_binding_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'approval_outbox_bindings are immutable and cannot be deleted';
    END IF;

    IF OLD.approval_record_id <> NEW.approval_record_id
        OR OLD.topic <> NEW.topic
        OR OLD.review_gate <> NEW.review_gate
        OR OLD.aggregate_kind <> NEW.aggregate_kind
        OR OLD.aggregate_id <> NEW.aggregate_id
        OR OLD.payload <> NEW.payload
        OR OLD.authorized_by_actor_kind <> NEW.authorized_by_actor_kind
        OR OLD.authorized_by_actor_id <> NEW.authorized_by_actor_id
        OR OLD.authorized_by_actor_persona <> NEW.authorized_by_actor_persona
        OR OLD.authorized_at <> NEW.authorized_at
        OR OLD.consumed_by_outbox_id IS NOT NULL
        OR NEW.consumed_by_outbox_id IS NULL
        OR NOT EXISTS (
            SELECT 1
            FROM outbox_records
            WHERE id = NEW.consumed_by_outbox_id
              AND approval_record_id = OLD.approval_record_id
              AND topic = OLD.topic
              AND review_gate = OLD.review_gate
              AND aggregate_kind = OLD.aggregate_kind
              AND aggregate_id = OLD.aggregate_id
              AND payload = OLD.payload
        )
    THEN
        RAISE EXCEPTION 'approval_outbox_bindings permit only one relationally proven consumption transition';
    END IF;

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS approval_outbox_bindings_immutable_update ON approval_outbox_bindings;
CREATE TRIGGER approval_outbox_bindings_immutable_update
    BEFORE UPDATE ON approval_outbox_bindings
    FOR EACH ROW EXECUTE FUNCTION reject_approval_outbox_binding_mutation();

DROP TRIGGER IF EXISTS approval_outbox_bindings_immutable_delete ON approval_outbox_bindings;
CREATE TRIGGER approval_outbox_bindings_immutable_delete
    BEFORE DELETE ON approval_outbox_bindings
    FOR EACH ROW EXECUTE FUNCTION reject_approval_outbox_binding_mutation();

CREATE OR REPLACE FUNCTION prevent_approval_change_with_open_outbox_records()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF (
        OLD.status = 'approved'
        AND (
            NEW.status <> 'approved'
            OR NEW.target_kind <> OLD.target_kind
            OR NEW.target_id <> OLD.target_id
            OR NEW.gate <> OLD.gate
            OR NEW.decided_by_actor_kind IS DISTINCT FROM OLD.decided_by_actor_kind
            OR NEW.decided_by_actor_id IS DISTINCT FROM OLD.decided_by_actor_id
            OR NEW.decided_by_actor_persona IS DISTINCT FROM OLD.decided_by_actor_persona
            OR NEW.decided_at IS DISTINCT FROM OLD.decided_at
        )
        AND (
            EXISTS (SELECT 1 FROM approval_outbox_bindings WHERE approval_record_id = OLD.id)
            OR EXISTS (SELECT 1 FROM outbox_records WHERE approval_record_id = OLD.id)
        )
    ) THEN
        RAISE EXCEPTION 'cannot change approval after an approval_outbox_binding or outbox_record exists';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS approval_records_open_outbox_guard ON approval_records;
CREATE TRIGGER approval_records_open_outbox_guard
    BEFORE UPDATE OF status, target_kind, target_id, gate,
        decided_by_actor_kind, decided_by_actor_id, decided_by_actor_persona, decided_at ON approval_records
    FOR EACH ROW EXECUTE FUNCTION prevent_approval_change_with_open_outbox_records();

ALTER TABLE manager_daily_brief_outcomes
    ADD COLUMN IF NOT EXISTS schema_version integer NOT NULL DEFAULT 1;
ALTER TABLE manager_daily_brief_outcomes
    DROP CONSTRAINT IF EXISTS manager_daily_brief_outcomes_schema_version_check;
ALTER TABLE manager_daily_brief_outcomes
    ADD CONSTRAINT manager_daily_brief_outcomes_schema_version_check
    CHECK (schema_version = 1);
ALTER TABLE manager_daily_brief_outcomes
    DROP CONSTRAINT IF EXISTS manager_daily_brief_outcomes_action_kind_check;
ALTER TABLE manager_daily_brief_outcomes
    ADD CONSTRAINT manager_daily_brief_outcomes_action_kind_check CHECK (action_kind IN (
        'review_demand_against_staffing_plan',
        'resolve_checkout_exception',
        'approve_retention_follow_up_draft',
        'investigate_source_data_quality_issue',
        'review_capacity_labor_recommendation'
    ));
ALTER TABLE manager_daily_brief_outcomes
    ALTER COLUMN location_id SET NOT NULL;

ALTER TABLE data_quality_hygiene_outcomes
    ADD COLUMN IF NOT EXISTS schema_version integer NOT NULL DEFAULT 1;
ALTER TABLE data_quality_hygiene_outcomes
    DROP CONSTRAINT IF EXISTS data_quality_hygiene_outcomes_schema_version_check;
ALTER TABLE data_quality_hygiene_outcomes
    ADD CONSTRAINT data_quality_hygiene_outcomes_schema_version_check
    CHECK (schema_version = 1);
ALTER TABLE data_quality_hygiene_outcomes
    ALTER COLUMN location_id SET NOT NULL;

-- Reviewed outcome evidence must be backed by one immutable approved decision whose packet binds
-- the exact owned workflow event, action id, target, and gate. This does not invent provider
-- relationships; it closes only NVA-owned review lineage.
CREATE OR REPLACE FUNCTION enforce_outcome_approval_workflow_binding()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    approval approval_records%ROWTYPE;
    approval_review_packet review_packets%ROWTYPE;
    reviewed_workflow_event workflow_events%ROWTYPE;
    source_ref jsonb;
    current_issue_ref jsonb;
    expected_actor_kind text;
BEGIN
    -- UPDATE-strength locks serialize outcome admission with every mutable lineage row. KEY SHARE
    -- is insufficient because it is compatible with non-key updates and admits a check-then-mutate race.
    SELECT *
      INTO approval
      FROM approval_records
     WHERE id = NEW.approval_record_id
     FOR UPDATE;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'outcome approval must be the exact approved workflow, action, target, and gate lineage';
    END IF;

    SELECT *
      INTO approval_review_packet
      FROM review_packets
     WHERE id = approval.review_packet_id
     FOR UPDATE;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'outcome approval must be the exact approved workflow, action, target, and gate lineage';
    END IF;

    SELECT *
      INTO reviewed_workflow_event
      FROM workflow_events
     WHERE id = NEW.workflow_event_id
     FOR UPDATE;

    expected_actor_kind := CASE NEW.actor_persona
        WHEN 'general_manager' THEN 'manager'
        WHEN 'assistant_general_manager' THEN 'manager'
        WHEN 'regional_operator' THEN 'manager'
        WHEN 'front_desk_lead' THEN 'staff'
        WHEN 'front_desk_agent' THEN 'staff'
        WHEN 'operations_analyst' THEN 'staff'
        ELSE NULL
    END;

    IF NOT FOUND
       OR approval.status <> 'approved'
       OR approval_review_packet.status <> 'approved'
       OR approval_review_packet.workflow_event_id IS NULL
       OR approval_review_packet.workflow_event_id <> NEW.workflow_event_id
       OR approval_review_packet.reviewed_action_id IS NULL
       OR approval_review_packet.reviewed_action_id <> NEW.action_id
       OR approval.target_kind <> approval_review_packet.subject_kind
       OR approval.target_id <> approval_review_packet.subject_id
       OR approval.gate <> approval_review_packet.gate
       OR NEW.schema_version <> 1
       OR NEW.id = '00000000-0000-0000-0000-000000000000'::uuid
       OR NEW.location_id = '00000000-0000-0000-0000-000000000000'::uuid
       OR NEW.workflow_event_id = '00000000-0000-0000-0000-000000000000'::uuid
       OR approval.review_packet_id = '00000000-0000-0000-0000-000000000000'::uuid
       OR NEW.approval_record_id = '00000000-0000-0000-0000-000000000000'::uuid
       OR approval.requested_at > approval.decided_at
       OR reviewed_workflow_event.occurred_at > approval.requested_at
       OR approval.decided_at > NEW.recorded_at
       OR approval.decided_by_actor_id <> NEW.actor_id
       OR approval.decided_by_actor_persona IS NULL
       OR approval.decided_by_actor_persona <> NEW.actor_persona
       OR approval.decided_by_actor_kind <> expected_actor_kind
       OR (approval.gate = 'manager_approval' AND approval.decided_by_actor_kind <> 'manager')
       OR NEW.operating_day > NEW.recorded_at::date
       OR jsonb_typeof(NEW.source_refs) <> 'array'
       OR jsonb_array_length(NEW.source_refs) = 0
       OR reviewed_workflow_event.payload->'source_refs' IS DISTINCT FROM NEW.source_refs
       OR reviewed_workflow_event.payload->>'correlation_id' IS DISTINCT FROM NEW.correlation_id
       OR reviewed_workflow_event.payload->>'location_id' IS NULL
       OR reviewed_workflow_event.payload->>'location_id' <> NEW.location_id::text
       OR reviewed_workflow_event.subject_kind <> 'location'
       OR reviewed_workflow_event.subject_id <> NEW.location_id
       OR reviewed_workflow_event.payload->>'operating_day' IS DISTINCT FROM NEW.operating_day::text
       OR (
            TG_TABLE_NAME = 'data_quality_hygiene_outcomes'
            AND (
                jsonb_typeof(to_jsonb(NEW)->'issue_refs') <> 'array'
                OR jsonb_array_length(to_jsonb(NEW)->'issue_refs') = 0
                OR reviewed_workflow_event.payload->'issue_refs' IS DISTINCT FROM to_jsonb(NEW)->'issue_refs'
            )
       )
       OR (
            TG_TABLE_NAME = 'manager_daily_brief_outcomes'
            AND NOT (
                (
                    reviewed_workflow_event.workflow_name = 'manager_daily_brief'
                    AND reviewed_workflow_event.event_kind = 'outcome_capture'
                )
                OR (
                    reviewed_workflow_event.workflow_name = 'information_lifespan_manager_daily_report'
                    AND reviewed_workflow_event.event_kind = 'manager_daily_report.trace_replayed'
                )
            )
       )
       OR (
            TG_TABLE_NAME = 'data_quality_hygiene_outcomes'
            AND (
                reviewed_workflow_event.workflow_name <> 'data_quality_hygiene'
                OR reviewed_workflow_event.event_kind NOT IN (
                    'context_created',
                    'source_quality_issue.detected',
                    'outcome_capture'
                )
            )
       ) THEN
        RAISE EXCEPTION 'outcome approval must be the exact approved workflow, action, target, gate, actor, scope, time, and source lineage';
    END IF;

    FOR source_ref IN SELECT value FROM jsonb_array_elements(NEW.source_refs)
    LOOP
        IF jsonb_typeof(source_ref) <> 'object'
           OR jsonb_typeof(source_ref->'system') <> 'string'
           OR length(trim(source_ref->>'system')) = 0
           OR jsonb_typeof(source_ref->'record_type') <> 'string'
           OR length(trim(source_ref->>'record_type')) = 0
           OR jsonb_typeof(source_ref->'record_id') <> 'string'
           OR length(trim(source_ref->>'record_id')) = 0
           OR trim(source_ref->>'record_id') = '0'
           OR jsonb_typeof(source_ref->'observed_at') <> 'string'
           OR (source_ref->>'observed_at') !~ '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$'
           OR (source_ref->>'observed_at')::timestamptz > reviewed_workflow_event.occurred_at
           OR (source_ref->>'observed_at')::timestamptz > approval.requested_at
           OR (source_ref->>'observed_at')::timestamptz > approval.decided_at
           OR (source_ref->>'observed_at')::timestamptz > NEW.recorded_at
           OR jsonb_typeof(source_ref->'adapter_version') <> 'string'
           OR length(trim(source_ref->>'adapter_version')) = 0
           OR trim(source_ref->>'adapter_version') = '0'
           OR (SELECT count(*) FROM jsonb_object_keys(source_ref)) <> 5 THEN
            RAISE EXCEPTION 'outcome source references must be exact typed provenance';
        END IF;
    END LOOP;

    IF TG_TABLE_NAME = 'data_quality_hygiene_outcomes' THEN
        FOR current_issue_ref IN SELECT value FROM jsonb_array_elements(to_jsonb(NEW)->'issue_refs')
        LOOP
            IF jsonb_typeof(current_issue_ref) <> 'string'
               OR length(trim(current_issue_ref #>> '{}')) = 0
               OR trim(current_issue_ref #>> '{}') = '0' THEN
                RAISE EXCEPTION 'outcome issue references must be nonempty semantic identifiers';
            END IF;
        END LOOP;
    END IF;

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS manager_daily_brief_outcomes_approval_workflow_binding ON manager_daily_brief_outcomes;
CREATE TRIGGER manager_daily_brief_outcomes_approval_workflow_binding
BEFORE INSERT OR UPDATE OF workflow_event_id, approval_record_id, action_id ON manager_daily_brief_outcomes
FOR EACH ROW EXECUTE FUNCTION enforce_outcome_approval_workflow_binding();

DROP TRIGGER IF EXISTS data_quality_hygiene_outcomes_approval_workflow_binding ON data_quality_hygiene_outcomes;
CREATE TRIGGER data_quality_hygiene_outcomes_approval_workflow_binding
BEFORE INSERT OR UPDATE OF workflow_event_id, approval_record_id, action_id ON data_quality_hygiene_outcomes
FOR EACH ROW EXECUTE FUNCTION enforce_outcome_approval_workflow_binding();

-- A repeated migration run must revalidate admitted rows before recreating the
-- immutable guards. These DDL statements hold table locks for this transaction,
-- so there is no interval in which another session can rewrite outcome history.
DROP TRIGGER IF EXISTS manager_daily_brief_outcomes_immutable ON manager_daily_brief_outcomes;
DROP TRIGGER IF EXISTS data_quality_hygiene_outcomes_immutable ON data_quality_hygiene_outcomes;

-- Validate every outcome admitted before this migration under the same exact
-- lineage function before installing immutable-history guards. Assigning each
-- binding column to itself is deliberate: it fires the UPDATE OF trigger without
-- inventing or repairing missing authority metadata. Any incompatible legacy row
-- aborts the migration transaction.
UPDATE manager_daily_brief_outcomes
SET workflow_event_id = workflow_event_id,
    approval_record_id = approval_record_id,
    action_id = action_id;

UPDATE data_quality_hygiene_outcomes
SET workflow_event_id = workflow_event_id,
    approval_record_id = approval_record_id,
    action_id = action_id;

CREATE OR REPLACE FUNCTION prevent_reviewed_outcome_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'reviewed outcome evidence is immutable after admission';
END;
$$;

DROP TRIGGER IF EXISTS manager_daily_brief_outcomes_immutable ON manager_daily_brief_outcomes;
CREATE TRIGGER manager_daily_brief_outcomes_immutable
BEFORE UPDATE OR DELETE ON manager_daily_brief_outcomes
FOR EACH ROW EXECUTE FUNCTION prevent_reviewed_outcome_mutation();

DROP TRIGGER IF EXISTS data_quality_hygiene_outcomes_immutable ON data_quality_hygiene_outcomes;
CREATE TRIGGER data_quality_hygiene_outcomes_immutable
BEFORE UPDATE OR DELETE ON data_quality_hygiene_outcomes
FOR EACH ROW EXECUTE FUNCTION prevent_reviewed_outcome_mutation();

CREATE OR REPLACE FUNCTION prevent_outcome_approval_lineage_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM manager_daily_brief_outcomes WHERE approval_record_id = OLD.id
        UNION ALL
        SELECT 1 FROM data_quality_hygiene_outcomes WHERE approval_record_id = OLD.id
    ) THEN
        RAISE EXCEPTION 'approval lineage referenced by an outcome is immutable';
    END IF;
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS approval_records_outcome_lineage_immutable ON approval_records;
CREATE TRIGGER approval_records_outcome_lineage_immutable
BEFORE UPDATE OR DELETE ON approval_records
FOR EACH ROW EXECUTE FUNCTION prevent_outcome_approval_lineage_mutation();

CREATE OR REPLACE FUNCTION prevent_outcome_review_packet_lineage_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF EXISTS (
        SELECT 1
          FROM approval_records approval_record
          JOIN manager_daily_brief_outcomes outcome
            ON outcome.approval_record_id = approval_record.id
         WHERE approval_record.review_packet_id = OLD.id
        UNION ALL
        SELECT 1
          FROM approval_records approval_record
          JOIN data_quality_hygiene_outcomes outcome
            ON outcome.approval_record_id = approval_record.id
         WHERE approval_record.review_packet_id = OLD.id
    ) THEN
        RAISE EXCEPTION 'review packet lineage referenced by an outcome is immutable';
    END IF;
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS review_packets_outcome_lineage_immutable ON review_packets;
CREATE TRIGGER review_packets_outcome_lineage_immutable
BEFORE UPDATE OR DELETE ON review_packets
FOR EACH ROW EXECUTE FUNCTION prevent_outcome_review_packet_lineage_mutation();

CREATE OR REPLACE FUNCTION prevent_outcome_workflow_event_lineage_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM manager_daily_brief_outcomes WHERE workflow_event_id = OLD.id
        UNION ALL
        SELECT 1 FROM data_quality_hygiene_outcomes WHERE workflow_event_id = OLD.id
    ) THEN
        RAISE EXCEPTION 'workflow event lineage referenced by an outcome is immutable';
    END IF;
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS workflow_events_outcome_lineage_immutable ON workflow_events;
CREATE TRIGGER workflow_events_outcome_lineage_immutable
BEFORE UPDATE OR DELETE ON workflow_events
FOR EACH ROW EXECUTE FUNCTION prevent_outcome_workflow_event_lineage_mutation();

COMMIT;
