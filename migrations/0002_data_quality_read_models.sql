-- Data-Quality Hygiene durable source/import/read-model slice.
-- This migration adds owned operational evidence and BI-safe read models for one workflow.
-- It does not authorize live side effects: provider_writes_allowed=false,
-- customer_messages_allowed=false, live_delivery_allowed=false.
-- raw provider payloads are redacted or referenced, not exposed through these read models.

CREATE TABLE IF NOT EXISTS source_import_runs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    source_system text NOT NULL CHECK (length(trim(source_system)) > 0),
    adapter_version text NOT NULL CHECK (length(trim(adapter_version)) > 0),
    location_id uuid REFERENCES locations(id),
    tenant_id text,
    mode text NOT NULL CHECK (mode IN ('read_only_snapshot', 'dry_run_mapping')),
    status text NOT NULL CHECK (status IN ('pending', 'completed', 'completed_with_rejections', 'failed')),
    started_at timestamptz NOT NULL,
    completed_at timestamptz,
    record_count integer NOT NULL DEFAULT 0 CHECK (record_count >= 0),
    rejected_count integer NOT NULL DEFAULT 0 CHECK (rejected_count >= 0),
    safe_error_class text,
    redaction_posture text NOT NULL CHECK (length(trim(redaction_posture)) > 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (completed_at IS NULL OR completed_at >= started_at)
);

CREATE TABLE IF NOT EXISTS source_quality_issues (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_ref text NOT NULL CHECK (length(trim(issue_ref)) > 0),
    location_id uuid REFERENCES locations(id),
    tenant_id text,
    affected_entity_kind text NOT NULL CHECK (affected_entity_kind IN ('customer', 'pet', 'reservation', 'location', 'source_record')),
    affected_entity_id text NOT NULL CHECK (length(trim(affected_entity_id)) > 0),
    field_path text NOT NULL CHECK (length(trim(field_path)) > 0),
    issue_kind text NOT NULL CHECK (issue_kind IN ('missing_source_evidence', 'duplicate_entity_candidate', 'missing_required_field', 'stale_source_freshness', 'ambiguous_service_line_naming', 'unclosed_reservation_evidence', 'sensitive_payload_quarantine', 'payment_state_conflict')),
    severity text NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    freshness text NOT NULL CHECK (freshness IN ('current', 'stale', 'unknown')),
    sensitivity text NOT NULL CHECK (sensitivity IN ('operational_metadata', 'customer_or_pet_profile', 'medical_or_vaccination', 'payment_state', 'quarantined')),
    workflow_blocking text NOT NULL CHECK (workflow_blocking IN ('blocking', 'non_blocking')),
    owner_persona text NOT NULL CHECK (length(trim(owner_persona)) > 0),
    review_gate text NOT NULL CHECK (review_gate_is_valid(review_gate)),
    resolution_status text NOT NULL CHECK (resolution_status IN ('open', 'acknowledged', 'ignored', 'repaired', 'superseded')),
    source_refs jsonb NOT NULL DEFAULT '[]'::jsonb,
    workflow_event_id uuid REFERENCES workflow_events(id),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    resolved_at timestamptz,
    CONSTRAINT source_quality_issues_issue_ref_key UNIQUE (issue_ref),
    CHECK (jsonb_typeof(source_refs) = 'array'),
    CHECK (resolved_at IS NULL OR resolved_at >= created_at)
);

CREATE TABLE IF NOT EXISTS sync_gaps (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    source_system text NOT NULL CHECK (length(trim(source_system)) > 0),
    source_ref jsonb,
    location_id uuid REFERENCES locations(id),
    tenant_id text,
    gap_kind text NOT NULL CHECK (gap_kind IN ('missing_expected_record', 'stale_expected_record', 'mapping_uncertain', 'adapter_failure')),
    severity text NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    detected_at timestamptz NOT NULL,
    age_seconds bigint NOT NULL CHECK (age_seconds >= 0),
    status text NOT NULL CHECK (status IN ('open', 'acknowledged', 'resolved', 'superseded')),
    workflow_event_id uuid REFERENCES workflow_events(id),
    safe_error_class text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (source_ref IS NULL OR jsonb_typeof(source_ref) = 'object')
);

CREATE INDEX IF NOT EXISTS source_import_runs_location_status_idx
    ON source_import_runs(location_id, status, started_at);
CREATE INDEX IF NOT EXISTS source_import_runs_source_started_idx
    ON source_import_runs(source_system, started_at);
CREATE INDEX IF NOT EXISTS source_quality_issues_location_resolution_severity_idx
    ON source_quality_issues(location_id, resolution_status, severity);
CREATE INDEX IF NOT EXISTS source_quality_issues_workflow_event_idx
    ON source_quality_issues(workflow_event_id);
CREATE INDEX IF NOT EXISTS source_quality_issues_source_refs_gin_idx
    ON source_quality_issues USING gin(source_refs);
CREATE INDEX IF NOT EXISTS sync_gaps_location_status_severity_idx
    ON sync_gaps(location_id, status, severity);
CREATE INDEX IF NOT EXISTS sync_gaps_source_detected_idx
    ON sync_gaps(source_system, detected_at);

CREATE OR REPLACE VIEW source_quality_backlog AS
SELECT
    sqi.issue_ref,
    sqi.location_id,
    sqi.tenant_id,
    sqi.affected_entity_kind,
    sqi.affected_entity_id,
    sqi.field_path,
    sqi.issue_kind,
    sqi.severity,
    sqi.freshness,
    sqi.sensitivity,
    sqi.workflow_blocking,
    sqi.owner_persona,
    sqi.review_gate,
    sqi.resolution_status,
    sqi.source_refs,
    sqi.workflow_event_id,
    latest_outcome.id AS latest_outcome_id,
    'source_quality_backlog.v1'::text AS projection_version,
    ARRAY_REMOVE(ARRAY[
        CASE WHEN sqi.sensitivity IN ('medical_or_vaccination', 'payment_state', 'quarantined') THEN 'raw_payload_redacted' END,
        CASE WHEN sqi.resolution_status = 'open' THEN 'review_pending' END,
        CASE WHEN sqi.freshness IN ('stale', 'unknown') THEN 'source_stale' END,
        'live_side_effects_disabled'
    ], NULL)::text[] AS caveats
FROM source_quality_issues sqi
LEFT JOIN LATERAL (
    SELECT dqh.id
    FROM data_quality_hygiene_outcomes dqh
    WHERE dqh.issue_refs ? sqi.issue_ref
    ORDER BY dqh.recorded_at DESC
    LIMIT 1
) latest_outcome ON TRUE;

CREATE OR REPLACE VIEW data_quality_hygiene_labor_outcomes AS
SELECT
    dqh.id,
    dqh.location_id,
    dqh.operating_day,
    dqh.action_kind,
    dqh.owner_persona,
    dqh.actor_persona,
    dqh.outcome,
    dqh.resolution_status_after_review,
    dqh.before_minutes,
    dqh.actual_minutes,
    dqh.estimated_minutes_saved,
    GREATEST(dqh.before_minutes - dqh.actual_minutes, 0) AS actual_minutes_saved,
    dqh.issue_refs,
    dqh.source_refs,
    dqh.workflow_event_id,
    dqh.approval_record_id,
    dqh.correlation_id,
    'data_quality_hygiene_labor_outcomes.v1'::text AS projection_version,
    ARRAY['live_side_effects_disabled']::text[] AS caveats
FROM data_quality_hygiene_outcomes dqh;

CREATE OR REPLACE VIEW audit_lineage AS
SELECT
    we.payload->>'correlation_id' AS correlation_id,
    we.payload->>'request_id' AS request_id,
    we.id AS workflow_event_id,
    rp.id AS review_packet_id,
    ar.id AS approval_record_id,
    dqh.id AS outcome_id,
    ob.id AS outbox_id,
    ARRAY_AGG(ae.id ORDER BY ae.occurred_at) FILTER (WHERE ae.id IS NOT NULL) AS audit_event_ids,
    'audit_lineage.v1'::text AS projection_version
FROM workflow_events we
LEFT JOIN review_packets rp ON rp.workflow_event_id = we.id
LEFT JOIN approval_records ar ON ar.review_packet_id = rp.id
LEFT JOIN data_quality_hygiene_outcomes dqh ON dqh.workflow_event_id = we.id
LEFT JOIN outbox_records ob ON ob.approval_record_id = ar.id
LEFT JOIN audit_events ae ON ae.workflow_event_id = we.id
GROUP BY we.id, rp.id, ar.id, dqh.id, ob.id;

CREATE OR REPLACE VIEW import_freshness AS
WITH import_rollup AS (
    SELECT
        source_system,
        location_id,
        MAX(completed_at) FILTER (WHERE status IN ('completed', 'completed_with_rejections')) AS last_completed_at,
        (ARRAY_AGG(adapter_version ORDER BY started_at DESC))[1] AS adapter_version,
        SUM(record_count) AS record_count,
        SUM(rejected_count) AS rejected_count,
        COUNT(*) FILTER (WHERE status = 'failed') AS failed_import_count
    FROM source_import_runs
    GROUP BY source_system, location_id
), gap_rollup AS (
    SELECT
        source_system,
        location_id,
        COUNT(*) FILTER (WHERE status = 'open') AS open_gap_count
    FROM sync_gaps
    GROUP BY source_system, location_id
)
SELECT
    ir.source_system,
    ir.location_id,
    ir.last_completed_at,
    ir.adapter_version,
    ir.record_count,
    ir.rejected_count,
    ir.failed_import_count,
    COALESCE(gr.open_gap_count, 0) AS open_gap_count,
    'import_freshness.v1'::text AS projection_version,
    ARRAY_REMOVE(ARRAY[
        CASE WHEN ir.rejected_count > 0 THEN 'source_import_had_rejections' END,
        CASE WHEN ir.failed_import_count > 0 THEN 'source_import_failed' END,
        CASE WHEN COALESCE(gr.open_gap_count, 0) > 0 THEN 'open_sync_gaps' END
    ], NULL)::text[] AS caveats
FROM import_rollup ir
LEFT JOIN gap_rollup gr
    ON gr.source_system = ir.source_system
   AND (gr.location_id = ir.location_id OR (gr.location_id IS NULL AND ir.location_id IS NULL));
