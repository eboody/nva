-- Safe synthetic Data-Quality Hygiene demo data for local Postgres only.
-- This file does not contain live NVA/Gingr data and never enables provider writes or customer sends.

WITH location_row AS (
    INSERT INTO locations (id, brand, name, timezone)
    VALUES (
        '00c0ffee-0000-0000-0000-000000000001',
        'PetSuites',
        'PetSuites Safe Synthetic Demo',
        'America/New_York'
    )
    ON CONFLICT (id) DO UPDATE
    SET brand = EXCLUDED.brand,
        name = EXCLUDED.name,
        timezone = EXCLUDED.timezone,
        updated_at = now()
    RETURNING id
), workflow_row AS (
    INSERT INTO workflow_events (
        id,
        workflow_name,
        event_kind,
        subject_kind,
        subject_id,
        idempotency_key,
        payload,
        occurred_at
    )
    VALUES (
        '10c0ffee-0000-0000-0000-000000000001',
        'data-quality-hygiene',
        'context_created',
        'location',
        '00c0ffee-0000-0000-0000-000000000001',
        'local-demo:data-quality-hygiene:2026-06-17',
        jsonb_build_object(
            'correlation_id', 'data-quality-hygiene:local-demo:2026-06-17',
            'request_id', 'local-demo-read-model-seed',
            'safe_synthetic_data', true,
            'provider_writes_allowed', false,
            'customer_messages_allowed', false
        ),
        '2026-06-17T12:00:00Z'
    )
    ON CONFLICT (idempotency_key) DO UPDATE
    SET payload = EXCLUDED.payload
    RETURNING id
), review_packet_row AS (
    INSERT INTO review_packets (
        id,
        subject_kind,
        subject_id,
        gate,
        status,
        workflow_event_id,
        created_by_actor_kind,
        created_by_actor_id
    )
    VALUES (
        '20c0ffee-0000-0000-0000-000000000001',
        'location',
        '00c0ffee-0000-0000-0000-000000000001',
        'manager_approval',
        'approved',
        (SELECT id FROM workflow_row),
        'manager',
        'synthetic-general-manager'
    )
    ON CONFLICT (id) DO UPDATE
    SET status = EXCLUDED.status,
        workflow_event_id = EXCLUDED.workflow_event_id,
        updated_at = now()
    RETURNING id
), approval_row AS (
    INSERT INTO approval_records (
        id,
        target_kind,
        target_id,
        gate,
        status,
        requested_by_actor_kind,
        requested_by_actor_id,
        requested_at,
        decided_by_actor_kind,
        decided_by_actor_id,
        decided_at,
        review_packet_id
    )
    VALUES (
        '30c0ffee-0000-0000-0000-000000000001',
        'message',
        '40c0ffee-0000-0000-0000-000000000001',
        'manager_approval',
        'approved',
        'agent',
        'fake-deterministic-agent',
        '2026-06-17T12:05:00Z',
        'manager',
        'synthetic-general-manager',
        '2026-06-17T12:10:00Z',
        (SELECT id FROM review_packet_row)
    )
    ON CONFLICT (id) DO UPDATE
    SET status = EXCLUDED.status,
        review_packet_id = EXCLUDED.review_packet_id,
        updated_at = now()
    RETURNING id
)
INSERT INTO audit_events (
    id,
    occurred_at,
    actor_kind,
    actor_id,
    subject_kind,
    subject_id,
    action,
    workflow_event_id,
    metadata
)
VALUES (
    '50c0ffee-0000-0000-0000-000000000001',
    '2026-06-17T12:11:00Z',
    'manager',
    'synthetic-general-manager',
    'workflow_event',
    (SELECT id::text FROM workflow_row),
    'data_quality_hygiene.reviewed_without_live_side_effects',
    (SELECT id FROM workflow_row),
    jsonb_build_object('safe_synthetic_data', true, 'live_side_effects_allowed', false)
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO source_import_runs (
    id,
    source_system,
    adapter_version,
    location_id,
    tenant_id,
    mode,
    status,
    started_at,
    completed_at,
    record_count,
    rejected_count,
    redaction_posture
)
VALUES (
    '60c0ffee-0000-0000-0000-000000000001',
    'gingr_snapshot_synthetic',
    'local-demo-adapter-v1',
    '00c0ffee-0000-0000-0000-000000000001',
    'local-demo',
    'read_only_snapshot',
    'completed_with_rejections',
    '2026-06-17T11:45:00Z',
    '2026-06-17T11:50:00Z',
    42,
    2,
    'raw_provider_payloads_redacted_or_referenced_only'
)
ON CONFLICT (id) DO UPDATE
SET record_count = EXCLUDED.record_count,
    rejected_count = EXCLUDED.rejected_count,
    status = EXCLUDED.status;

INSERT INTO source_quality_issues (
    id,
    issue_ref,
    location_id,
    tenant_id,
    affected_entity_kind,
    affected_entity_id,
    field_path,
    issue_kind,
    severity,
    freshness,
    sensitivity,
    workflow_blocking,
    owner_persona,
    review_gate,
    resolution_status,
    source_refs,
    workflow_event_id
)
VALUES
    (
        '70c0ffee-0000-0000-0000-000000000001',
        'synthetic-dq-001',
        '00c0ffee-0000-0000-0000-000000000001',
        'local-demo',
        'pet',
        'synthetic-pet-miso',
        'vaccination.rabies.expires_on',
        'stale_source_freshness',
        'high',
        'stale',
        'medical_or_vaccination',
        'blocking',
        'front_desk_lead',
        'manager_approval',
        'open',
        '[{"source_system":"gingr_snapshot_synthetic","source_record_ref":"redacted-vaccine-row-001","raw_payload":"redacted"}]'::jsonb,
        '10c0ffee-0000-0000-0000-000000000001'
    ),
    (
        '70c0ffee-0000-0000-0000-000000000002',
        'synthetic-dq-002',
        '00c0ffee-0000-0000-0000-000000000001',
        'local-demo',
        'customer',
        'synthetic-customer-avery',
        'profile.mobile_phone',
        'missing_required_field',
        'medium',
        'unknown',
        'customer_or_pet_profile',
        'non_blocking',
        'front_desk_agent',
        'manager_approval',
        'acknowledged',
        '[{"source_system":"gingr_snapshot_synthetic","source_record_ref":"redacted-customer-row-014","raw_payload":"redacted"}]'::jsonb,
        '10c0ffee-0000-0000-0000-000000000001'
    )
ON CONFLICT (issue_ref) DO UPDATE
SET resolution_status = EXCLUDED.resolution_status,
    source_refs = EXCLUDED.source_refs,
    updated_at = now();

INSERT INTO data_quality_hygiene_outcomes (
    id,
    workflow_event_id,
    approval_record_id,
    action_id,
    outcome,
    actor_id,
    actor_persona,
    feedback,
    issue_refs,
    resolution_status_after_review,
    owner_persona,
    action_kind,
    before_minutes,
    actual_minutes,
    estimated_minutes_saved,
    location_id,
    operating_day,
    source_refs,
    correlation_id,
    recorded_at
)
VALUES (
    '80c0ffee-0000-0000-0000-000000000001',
    '10c0ffee-0000-0000-0000-000000000001',
    '30c0ffee-0000-0000-0000-000000000001',
    'synthetic-data-quality-action-001',
    'completed',
    'front-desk-lead-synthetic',
    'front_desk_lead',
    'Prepared manager-reviewed cleanup handoff without mutating Gingr or messaging a customer.',
    '["synthetic-dq-001"]'::jsonb,
    'acknowledged',
    'front_desk_lead',
    'review_stale_vaccination_source_freshness',
    30,
    9,
    21,
    '00c0ffee-0000-0000-0000-000000000001',
    '2026-06-17',
    '[{"source_system":"gingr_snapshot_synthetic","source_record_ref":"redacted-vaccine-row-001","raw_payload":"redacted"}]'::jsonb,
    'data-quality-hygiene:local-demo:2026-06-17',
    '2026-06-17T13:15:00Z'
)
ON CONFLICT (action_id) DO UPDATE
SET outcome = EXCLUDED.outcome,
    actual_minutes = EXCLUDED.actual_minutes,
    issue_refs = EXCLUDED.issue_refs,
    source_refs = EXCLUDED.source_refs;

INSERT INTO sync_gaps (
    id,
    source_system,
    source_ref,
    location_id,
    tenant_id,
    gap_kind,
    severity,
    detected_at,
    age_seconds,
    status,
    workflow_event_id,
    safe_error_class
)
VALUES (
    '90c0ffee-0000-0000-0000-000000000001',
    'gingr_snapshot_synthetic',
    '{"source_record_ref":"redacted-vaccine-row-001"}'::jsonb,
    '00c0ffee-0000-0000-0000-000000000001',
    'local-demo',
    'stale_expected_record',
    'high',
    '2026-06-17T11:55:00Z',
    86400,
    'open',
    '10c0ffee-0000-0000-0000-000000000001',
    'safe_synthetic_stale_source'
)
ON CONFLICT (id) DO UPDATE
SET status = EXCLUDED.status,
    age_seconds = EXCLUDED.age_seconds;
