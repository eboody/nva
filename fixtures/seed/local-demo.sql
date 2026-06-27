-- Synthetic local demo seed for Docker Compose. Safe, deterministic, and idempotent.
-- These rows prove the Data-Quality Hygiene read-model path without live NVA/Gingr access,
-- live customer sends, provider/PMS writes, payment movement, schedule changes, or medical decisions.

WITH demo_location AS (
    INSERT INTO locations (id, brand, name, timezone)
    VALUES (
        '00000000-0000-4000-8000-000000000101',
        'Local Demo',
        'Local/dev kennel',
        'America/New_York'
    )
    ON CONFLICT (id) DO UPDATE
    SET brand = EXCLUDED.brand,
        name = EXCLUDED.name,
        timezone = EXCLUDED.timezone,
        updated_at = now()
    RETURNING id
), demo_customer AS (
    INSERT INTO customers (id, full_name, email, mobile_phone, preferred_contact, portal_provider, portal_customer_id)
    VALUES (
        '00000000-0000-4000-8000-000000000201',
        'Avery Chen (synthetic)',
        'avery.local-demo@example.invalid',
        NULL,
        'email',
        'local-fixture',
        'local-customer-001'
    )
    ON CONFLICT (id) DO UPDATE
    SET full_name = EXCLUDED.full_name,
        email = EXCLUDED.email,
        preferred_contact = EXCLUDED.preferred_contact,
        portal_provider = EXCLUDED.portal_provider,
        portal_customer_id = EXCLUDED.portal_customer_id,
        updated_at = now()
    RETURNING id
), demo_pet AS (
    INSERT INTO pets (id, customer_id, name, species, birth_date, sex, spay_neuter_status)
    SELECT
        '00000000-0000-4000-8000-000000000301',
        demo_customer.id,
        'Miso',
        'dog',
        '2021-04-12'::date,
        'female',
        'spayed'
    FROM demo_customer
    ON CONFLICT (id) DO UPDATE
    SET customer_id = EXCLUDED.customer_id,
        name = EXCLUDED.name,
        species = EXCLUDED.species,
        birth_date = EXCLUDED.birth_date,
        sex = EXCLUDED.sex,
        spay_neuter_status = EXCLUDED.spay_neuter_status,
        updated_at = now()
    RETURNING id
), demo_workflow_event AS (
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
    SELECT
        '00000000-0000-4000-8000-000000000401',
        'data_quality_hygiene',
        'source_quality_issue.detected',
        'pet',
        demo_pet.id,
        'local-demo-data-quality-hygiene-001',
        jsonb_build_object(
            'request_id', 'local-demo-request-001',
            'correlation_id', 'local-demo-data-quality-hygiene-001',
            'source_system', 'gingr-readonly-fixture',
            'live_side_effects_allowed', false
        ),
        '2026-06-20T13:00:00Z'::timestamptz
    FROM demo_pet
    ON CONFLICT (id) DO UPDATE
    SET workflow_name = EXCLUDED.workflow_name,
        event_kind = EXCLUDED.event_kind,
        subject_kind = EXCLUDED.subject_kind,
        subject_id = EXCLUDED.subject_id,
        idempotency_key = EXCLUDED.idempotency_key,
        payload = EXCLUDED.payload,
        occurred_at = EXCLUDED.occurred_at
    RETURNING id, subject_id
), demo_review_packet AS (
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
    SELECT
        '00000000-0000-4000-8000-000000000501',
        'message',
        demo_workflow_event.id,
        'manager_approval',
        'approved',
        demo_workflow_event.id,
        'agent',
        'agent.data-quality-hygiene.fake_deterministic'
    FROM demo_workflow_event
    ON CONFLICT (id) DO UPDATE
    SET subject_kind = EXCLUDED.subject_kind,
        subject_id = EXCLUDED.subject_id,
        gate = EXCLUDED.gate,
        status = EXCLUDED.status,
        workflow_event_id = EXCLUDED.workflow_event_id,
        created_by_actor_kind = EXCLUDED.created_by_actor_kind,
        created_by_actor_id = EXCLUDED.created_by_actor_id,
        updated_at = now()
    RETURNING id, subject_id
), demo_approval AS (
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
    SELECT
        '00000000-0000-4000-8000-000000000601',
        'message',
        demo_review_packet.subject_id,
        'manager_approval',
        'approved',
        'agent',
        'agent.data-quality-hygiene.fake_deterministic',
        '2026-06-20T13:01:00Z'::timestamptz,
        'manager',
        'local-demo-manager',
        '2026-06-20T13:02:00Z'::timestamptz,
        demo_review_packet.id
    FROM demo_review_packet
    ON CONFLICT (id) DO UPDATE
    SET target_kind = EXCLUDED.target_kind,
        target_id = EXCLUDED.target_id,
        gate = EXCLUDED.gate,
        status = EXCLUDED.status,
        requested_by_actor_kind = EXCLUDED.requested_by_actor_kind,
        requested_by_actor_id = EXCLUDED.requested_by_actor_id,
        requested_at = EXCLUDED.requested_at,
        decided_by_actor_kind = EXCLUDED.decided_by_actor_kind,
        decided_by_actor_id = EXCLUDED.decided_by_actor_id,
        decided_at = EXCLUDED.decided_at,
        review_packet_id = EXCLUDED.review_packet_id,
        updated_at = now()
    RETURNING id, target_id
), demo_result AS (
    INSERT INTO workflow_results (id, workflow_event_id, status, result)
    SELECT
        '00000000-0000-4000-8000-000000000701',
        demo_workflow_event.id,
        'succeeded',
        jsonb_build_object(
            'review_gate', 'manager_approval',
            'live_delivery_allowed', false,
            'provider_writes_allowed', false,
            'customer_messages_allowed', false
        )
    FROM demo_workflow_event
    ON CONFLICT (id) DO UPDATE
    SET workflow_event_id = EXCLUDED.workflow_event_id,
        status = EXCLUDED.status,
        result = EXCLUDED.result
    RETURNING id
), demo_import_run AS (
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
        safe_error_class,
        redaction_posture
    )
    SELECT
        '00000000-0000-4000-8000-000000000801',
        'gingr-readonly-fixture',
        'local-demo.v0',
        demo_location.id,
        'local-demo',
        'read_only_snapshot',
        'completed_with_rejections',
        '2026-06-20T12:55:00Z'::timestamptz,
        '2026-06-20T12:56:00Z'::timestamptz,
        42,
        1,
        'not_applicable',
        'raw_provider_payloads_redacted_or_referenced_only'
    FROM demo_location
    ON CONFLICT (id) DO UPDATE
    SET source_system = EXCLUDED.source_system,
        adapter_version = EXCLUDED.adapter_version,
        location_id = EXCLUDED.location_id,
        tenant_id = EXCLUDED.tenant_id,
        mode = EXCLUDED.mode,
        status = EXCLUDED.status,
        started_at = EXCLUDED.started_at,
        completed_at = EXCLUDED.completed_at,
        record_count = EXCLUDED.record_count,
        rejected_count = EXCLUDED.rejected_count,
        safe_error_class = EXCLUDED.safe_error_class,
        redaction_posture = EXCLUDED.redaction_posture
    RETURNING id
), demo_issue AS (
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
    SELECT
        '00000000-0000-4000-8000-000000000901',
        'SQI-LOCAL-001',
        demo_location.id,
        'local-demo',
        'pet',
        demo_pet.id::text,
        'pet.vaccine_records.rabies.source_document',
        'missing_source_evidence',
        'high',
        'unknown',
        'medical_or_vaccination',
        'blocking',
        'front_desk_lead',
        'manager_approval',
        'acknowledged',
        jsonb_build_array(jsonb_build_object(
            'source_system', 'gingr-readonly-fixture',
            'source_ref', 'pet:local-miso:vaccines',
            'redaction', 'raw_payload_not_persisted'
        )),
        demo_workflow_event.id
    FROM demo_location, demo_pet, demo_workflow_event
    ON CONFLICT (issue_ref) DO UPDATE
    SET location_id = EXCLUDED.location_id,
        tenant_id = EXCLUDED.tenant_id,
        affected_entity_kind = EXCLUDED.affected_entity_kind,
        affected_entity_id = EXCLUDED.affected_entity_id,
        field_path = EXCLUDED.field_path,
        issue_kind = EXCLUDED.issue_kind,
        severity = EXCLUDED.severity,
        freshness = EXCLUDED.freshness,
        sensitivity = EXCLUDED.sensitivity,
        workflow_blocking = EXCLUDED.workflow_blocking,
        owner_persona = EXCLUDED.owner_persona,
        review_gate = EXCLUDED.review_gate,
        resolution_status = EXCLUDED.resolution_status,
        source_refs = EXCLUDED.source_refs,
        workflow_event_id = EXCLUDED.workflow_event_id,
        updated_at = now()
    RETURNING id, issue_ref
), demo_gap AS (
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
    SELECT
        '00000000-0000-4000-8000-000000001001',
        'gingr-readonly-fixture',
        jsonb_build_object('source_ref', 'pet:local-miso:vaccines'),
        demo_location.id,
        'local-demo',
        'missing_expected_record',
        'high',
        '2026-06-20T12:57:00Z'::timestamptz,
        86400,
        'open',
        demo_workflow_event.id,
        'not_applicable'
    FROM demo_location, demo_workflow_event
    ON CONFLICT (id) DO UPDATE
    SET source_system = EXCLUDED.source_system,
        source_ref = EXCLUDED.source_ref,
        location_id = EXCLUDED.location_id,
        tenant_id = EXCLUDED.tenant_id,
        gap_kind = EXCLUDED.gap_kind,
        severity = EXCLUDED.severity,
        detected_at = EXCLUDED.detected_at,
        age_seconds = EXCLUDED.age_seconds,
        status = EXCLUDED.status,
        workflow_event_id = EXCLUDED.workflow_event_id,
        safe_error_class = EXCLUDED.safe_error_class,
        updated_at = now()
    RETURNING id
), demo_outcome AS (
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
    SELECT
        '00000000-0000-4000-8000-000000001101',
        demo_workflow_event.id,
        demo_approval.id,
        'DQM-LOCAL-001',
        'completed',
        'local-demo-front-desk-lead',
        'front_desk_lead',
        'Synthetic demo: acknowledged missing vaccine source evidence and routed internal handoff only.',
        jsonb_build_object('SQI-LOCAL-001', true),
        'acknowledged',
        'front_desk_lead',
        'investigate_missing_source_evidence',
        22,
        7,
        15,
        demo_location.id,
        '2026-06-20'::date,
        jsonb_build_array(jsonb_build_object(
            'source_system', 'gingr-readonly-fixture',
            'source_ref', 'pet:local-miso:vaccines',
            'redaction', 'raw_payload_not_persisted'
        )),
        'local-demo-data-quality-hygiene-001',
        '2026-06-20T13:03:00Z'::timestamptz
    FROM demo_workflow_event, demo_approval, demo_location
    ON CONFLICT (id) DO UPDATE
    SET workflow_event_id = EXCLUDED.workflow_event_id,
        approval_record_id = EXCLUDED.approval_record_id,
        action_id = EXCLUDED.action_id,
        outcome = EXCLUDED.outcome,
        actor_id = EXCLUDED.actor_id,
        actor_persona = EXCLUDED.actor_persona,
        feedback = EXCLUDED.feedback,
        issue_refs = EXCLUDED.issue_refs,
        resolution_status_after_review = EXCLUDED.resolution_status_after_review,
        owner_persona = EXCLUDED.owner_persona,
        action_kind = EXCLUDED.action_kind,
        before_minutes = EXCLUDED.before_minutes,
        actual_minutes = EXCLUDED.actual_minutes,
        estimated_minutes_saved = EXCLUDED.estimated_minutes_saved,
        location_id = EXCLUDED.location_id,
        operating_day = EXCLUDED.operating_day,
        source_refs = EXCLUDED.source_refs,
        correlation_id = EXCLUDED.correlation_id,
        recorded_at = EXCLUDED.recorded_at
    RETURNING id
)
INSERT INTO outbox_records (
    id,
    idempotency_key,
    approval_record_id,
    topic,
    review_gate,
    aggregate_kind,
    aggregate_id,
    payload,
    status,
    available_at
)
SELECT
    '00000000-0000-4000-8000-000000001201',
    'local-demo-data-quality-hygiene-internal-handoff-001',
    demo_approval.id,
    'internal.data_quality_hygiene.reviewed_handoff',
    'manager_approval',
    'message',
    demo_approval.target_id,
    jsonb_build_object(
        'internal_handoff_only', true,
        'live_delivery_allowed', false,
        'provider_writes_allowed', false,
        'customer_messages_allowed', false,
        'payment_actions_allowed', false,
        'issue_refs', jsonb_build_array('SQI-LOCAL-001')
    ),
    'pending',
    '2026-06-20T13:04:00Z'::timestamptz
FROM demo_approval
ON CONFLICT (id) DO UPDATE
SET idempotency_key = EXCLUDED.idempotency_key,
    approval_record_id = EXCLUDED.approval_record_id,
    topic = EXCLUDED.topic,
    review_gate = EXCLUDED.review_gate,
    aggregate_kind = EXCLUDED.aggregate_kind,
    aggregate_id = EXCLUDED.aggregate_id,
    payload = EXCLUDED.payload,
    status = EXCLUDED.status,
    available_at = EXCLUDED.available_at;
