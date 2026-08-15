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
), demo_workflow_event_inserted AS (
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
        'location',
        demo_location.id,
        'local-demo-data-quality-hygiene-001',
        jsonb_build_object(
            'request_id', 'local-demo-request-001',
            'correlation_id', 'local-demo-data-quality-hygiene-001',
            'location_id', demo_location.id::text,
            'operating_day', '2026-06-20',
            'issue_refs', jsonb_build_array('SQI-LOCAL-001'),
            'source_refs', jsonb_build_array(jsonb_build_object(
                'system', 'gingr-readonly-fixture',
                'record_type', 'pet_vaccine_record',
                'record_id', 'pet:local-miso:vaccines',
                'observed_at', '2026-06-20T13:00:00Z',
                'adapter_version', 'local-demo-readonly-v1'
            )),
            'source_system', 'gingr-readonly-fixture',
            'live_side_effects_allowed', false
        ),
        '2026-06-20T13:00:00Z'::timestamptz
    FROM demo_pet, demo_location
    ON CONFLICT (id) DO NOTHING
    RETURNING id, subject_id
), demo_workflow_event AS (
    SELECT id, subject_id FROM demo_workflow_event_inserted
    UNION ALL
    SELECT id, subject_id FROM workflow_events
    WHERE id = '00000000-0000-4000-8000-000000000401'
    LIMIT 1
), demo_review_packet_inserted AS (
    INSERT INTO review_packets (
        id,
        subject_kind,
        subject_id,
        gate,
        status,
        workflow_event_id,
        reviewed_action_id,
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
        'DQM-LOCAL-001',
        'agent',
        'agent.data-quality-hygiene.fake_deterministic'
    FROM demo_workflow_event
    ON CONFLICT (id) DO NOTHING
    RETURNING id, subject_id
), demo_review_packet AS (
    SELECT id, subject_id FROM demo_review_packet_inserted
    UNION ALL
    SELECT id, subject_id FROM review_packets
    WHERE id = '00000000-0000-4000-8000-000000000501'
    LIMIT 1
), demo_approval_inserted AS (
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
        decided_by_actor_persona,
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
        'general_manager',
        '2026-06-20T13:02:00Z'::timestamptz,
        demo_review_packet.id
    FROM demo_review_packet
    ON CONFLICT (id) DO NOTHING
    RETURNING id, target_id
), demo_approval AS (
    SELECT id, target_id FROM demo_approval_inserted
    UNION ALL
    SELECT id, target_id FROM approval_records
    WHERE id = '00000000-0000-4000-8000-000000000601'
    LIMIT 1
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
            'system', 'gingr-readonly-fixture',
            'record_type', 'pet_vaccine_record',
            'record_id', 'pet:local-miso:vaccines',
            'observed_at', '2026-06-17T13:00:00Z',
            'adapter_version', 'local-demo-adapter-v1'
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
        reported_estimated_minutes_difference,
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
        'local-demo-manager',
        'general_manager',
        'Synthetic demo: acknowledged missing vaccine source evidence and routed internal handoff only.',
        jsonb_build_array('SQI-LOCAL-001'),
        'acknowledged',
        'front_desk_lead',
        'investigate_missing_source_evidence',
        22,
        7,
        15,
        demo_location.id,
        '2026-06-20'::date,
        jsonb_build_array(jsonb_build_object(
            'system', 'gingr-readonly-fixture',
            'record_type', 'pet_vaccine_record',
            'record_id', 'pet:local-miso:vaccines',
            'observed_at', '2026-06-20T13:00:00Z',
            'adapter_version', 'local-demo-readonly-v1'
        )),
        'local-demo-data-quality-hygiene-001',
        '2026-06-20T13:03:00Z'::timestamptz
    FROM demo_workflow_event, demo_approval, demo_location
    ON CONFLICT (id) DO NOTHING
    RETURNING id
), demo_binding AS (
    INSERT INTO approval_outbox_bindings (
        approval_record_id,
        topic,
        review_gate,
        aggregate_kind,
        aggregate_id,
        payload,
        authorized_by_actor_kind,
        authorized_by_actor_id,
        authorized_by_actor_persona,
        authorized_at
    )
    SELECT
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
        'manager',
        'local-demo-manager',
        'general_manager',
        '2026-06-20T13:02:00Z'::timestamptz
    FROM demo_approval
    ON CONFLICT (approval_record_id) DO NOTHING
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
ON CONFLICT (id) DO NOTHING;

-- Piece 2 information-lifespan proof rows: synthetic local-only DB lifecycle for the
-- Manager Daily Report trace. These rows are deterministic and queryable through
-- information_lifespan_db_lifecycle_proof; they do not contain real Gingr/NVA data
-- and keep live_side_effects_allowed=false.
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
), information_lifespan_import_run AS (
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
        '00000000-0000-4000-8000-00000000a801',
        'mock_gingr_readonly_fixture',
        'information-lifespan.local-demo.v1',
        demo_location.id,
        'local-demo',
        'read_only_snapshot',
        'completed',
        '2026-06-29T12:55:00Z'::timestamptz,
        '2026-06-29T12:56:00Z'::timestamptz,
        3,
        0,
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
), information_lifespan_workflow_event_inserted AS (
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
        '00000000-0000-4000-8000-00000000a901',
        'information_lifespan_manager_daily_report',
        'manager_daily_report.trace_replayed',
        'location',
        demo_location.id,
        'workflow_event:manager-daily-report:2026-06-29',
        jsonb_build_object(
            'request_id', 'local-demo-information-lifespan-run-001',
            'correlation_id', 'info-lifespan-demo-2026-06-29',
            'location_id', demo_location.id::text,
            'operating_day', '2026-06-29',
            'source_refs', jsonb_build_array(
                jsonb_build_object('system', 'mock_gingr_readonly_fixture', 'record_type', 'reservation', 'record_id', 'fixture://mock-gingr/reservations/9001001.json', 'observed_at', '2026-06-29T12:55:00Z', 'adapter_version', 'local-demo-adapter-v1'),
                jsonb_build_object('system', 'mock_gingr_readonly_fixture', 'record_type', 'care_note', 'record_id', 'fixture://mock-gingr/care-notes/9001001-feeding.json', 'observed_at', '2026-06-29T12:55:00Z', 'adapter_version', 'local-demo-adapter-v1'),
                jsonb_build_object('system', 'mock_gingr_readonly_fixture', 'record_type', 'vaccine_record', 'record_id', 'fixture://mock-gingr/vaccines/8101-rabies.json', 'observed_at', '2026-06-29T12:55:00Z', 'adapter_version', 'local-demo-adapter-v1')
            ),
            'source_system', 'mock_gingr_readonly_fixture',
            'source_import_run_id', (SELECT id::text FROM information_lifespan_import_run),
            'source_import_proof_ref', 'source_import_run:info-lifespan-demo-2026-06-29',
            'workflow_event_proof_ref', 'workflow_event:manager-daily-report:2026-06-29',
            'audit_lineage_proof_ref', 'audit_lineage:info-lifespan-demo-2026-06-29',
            'source_payload_refs', jsonb_build_array(
                'fixture://mock-gingr/reservations/9001001.json',
                'fixture://mock-gingr/care-notes/9001001-feeding.json',
                'fixture://mock-gingr/vaccines/8101-rabies.json'
            ),
            'live_side_effects_allowed', false,
            'provider_writes_allowed', false,
            'customer_messages_allowed', false,
            'payment_actions_allowed', false
        ),
        '2026-06-29T13:00:00Z'::timestamptz
    FROM demo_location
    ON CONFLICT (id) DO NOTHING
    RETURNING id, subject_id
), information_lifespan_workflow_event AS (
    SELECT id, subject_id FROM information_lifespan_workflow_event_inserted
    UNION ALL
    SELECT id, subject_id FROM workflow_events
    WHERE id = '00000000-0000-4000-8000-00000000a901'
    LIMIT 1
), information_lifespan_issue AS (
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
        '00000000-0000-4000-8000-00000000aa01',
        'source_quality_issue:vaccine-near-expiry:8101',
        demo_location.id,
        'local-demo',
        'pet',
        'synthetic-pet-8101',
        'vaccine.rabies.expires_on',
        'stale_source_freshness',
        'medium',
        'current',
        'medical_or_vaccination',
        'blocking',
        'front_desk_lead',
        'manager_approval',
        'open',
        jsonb_build_array(
            jsonb_build_object(
                'system', 'mock_gingr_readonly_fixture',
                'record_type', 'vaccine_record',
                'record_id', 'fixture://mock-gingr/vaccines/8101-rabies.json',
                'observed_at', '2026-06-29T13:06:00Z',
                'adapter_version', 'local-demo-information-lifespan-v1'
            )
        ),
        information_lifespan_workflow_event.id
    FROM demo_location, information_lifespan_workflow_event
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
), information_lifespan_review_packet_inserted AS (
    INSERT INTO review_packets (
        id,
        subject_kind,
        subject_id,
        gate,
        status,
        workflow_event_id,
        reviewed_action_id,
        created_by_actor_kind,
        created_by_actor_id
    )
    SELECT
        '00000000-0000-4000-8000-00000000ab01',
        'message',
        '00000000-0000-4000-8000-00000000b401',
        'manager_approval',
        'approved',
        information_lifespan_workflow_event.id,
        'manager_daily_brief_outcome:synthetic-2026-06-29',
        'agent',
        'agent.information-lifespan.local-demo'
    FROM information_lifespan_workflow_event
    ON CONFLICT (id) DO NOTHING
    RETURNING id, subject_id
), information_lifespan_review_packet AS (
    SELECT id, subject_id FROM information_lifespan_review_packet_inserted
    UNION ALL
    SELECT id, subject_id FROM review_packets
    WHERE id = '00000000-0000-4000-8000-00000000ab01'
    LIMIT 1
), information_lifespan_approval_inserted AS (
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
        decided_by_actor_persona,
        decided_at,
        review_packet_id
    )
    SELECT
        '00000000-0000-4000-8000-00000000ac01',
        'message',
        '00000000-0000-4000-8000-00000000b401',
        'manager_approval',
        'approved',
        'agent',
        'agent.information-lifespan.local-demo',
        '2026-06-29T13:01:00Z'::timestamptz,
        'manager',
        'local-demo-general-manager',
        'general_manager',
        '2026-06-29T13:02:00Z'::timestamptz,
        information_lifespan_review_packet.id
    FROM information_lifespan_review_packet
    ON CONFLICT (id) DO NOTHING
    RETURNING id
), information_lifespan_approval AS (
    SELECT id FROM information_lifespan_approval_inserted
    UNION ALL
    SELECT id FROM approval_records
    WHERE id = '00000000-0000-4000-8000-00000000ac01'
    LIMIT 1
), information_lifespan_result AS (
    INSERT INTO workflow_results (id, workflow_event_id, status, result)
    SELECT
        '00000000-0000-4000-8000-00000000ad01',
        information_lifespan_workflow_event.id,
        'needs_review',
        jsonb_build_object(
            'correlation_id', 'info-lifespan-demo-2026-06-29',
            'source_snapshots', 3,
            'normalized_facts', 3,
            'workflow_packets', 1,
            'review_gates', 5,
            'audit_events', 2,
            'reported_estimated_labor_minutes_difference', 42,
            'live_side_effects_allowed', false,
            'review_required_before_customer_or_provider_action', true
        )
    FROM information_lifespan_workflow_event
    ON CONFLICT (id) DO UPDATE
    SET workflow_event_id = EXCLUDED.workflow_event_id,
        status = EXCLUDED.status,
        result = EXCLUDED.result
    RETURNING id
), information_lifespan_outcome AS (
    INSERT INTO manager_daily_brief_outcomes (
        id,
        workflow_event_id,
        approval_record_id,
        action_id,
        outcome,
        actor_id,
        actor_persona,
        feedback,
        owner_persona,
        action_kind,
        before_minutes,
        actual_minutes,
        reported_estimated_minutes_difference,
        location_id,
        operating_day,
        source_refs,
        correlation_id,
        recorded_at
    )
    SELECT
        '00000000-0000-4000-8000-00000000ae01',
        information_lifespan_workflow_event.id,
        information_lifespan_approval.id,
        'manager_daily_brief_outcome:synthetic-2026-06-29',
        'deferred',
        'local-demo-general-manager',
        'general_manager',
        'Synthetic demo: Manager Daily Report packet is ready for review; no live send, provider write, payment, schedule, or medical action occurred.',
        'general_manager',
        'investigate_source_data_quality_issue',
        60,
        18,
        42,
        information_lifespan_workflow_event.subject_id,
        '2026-06-29'::date,
        jsonb_build_array(
            jsonb_build_object('system', 'mock_gingr_readonly_fixture', 'record_type', 'reservation', 'record_id', 'fixture://mock-gingr/reservations/9001001.json', 'observed_at', '2026-06-29T12:55:00Z', 'adapter_version', 'local-demo-adapter-v1'),
            jsonb_build_object('system', 'mock_gingr_readonly_fixture', 'record_type', 'care_note', 'record_id', 'fixture://mock-gingr/care-notes/9001001-feeding.json', 'observed_at', '2026-06-29T12:55:00Z', 'adapter_version', 'local-demo-adapter-v1'),
            jsonb_build_object('system', 'mock_gingr_readonly_fixture', 'record_type', 'vaccine_record', 'record_id', 'fixture://mock-gingr/vaccines/8101-rabies.json', 'observed_at', '2026-06-29T12:55:00Z', 'adapter_version', 'local-demo-adapter-v1')
        ),
        'info-lifespan-demo-2026-06-29',
        '2026-06-29T13:03:00Z'::timestamptz
    FROM information_lifespan_workflow_event, information_lifespan_approval
    ON CONFLICT (action_id) DO NOTHING
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
SELECT
    audit_row.id,
    audit_row.occurred_at,
    audit_row.actor_kind,
    audit_row.actor_id,
    audit_row.subject_kind,
    audit_row.subject_id,
    audit_row.action,
    audit_row.workflow_event_id,
    audit_row.metadata
FROM (
    SELECT
        '00000000-0000-4000-8000-00000000af01'::uuid AS id,
        '2026-06-29T13:02:00Z'::timestamptz AS occurred_at,
        'agent'::text AS actor_kind,
        'agent.information-lifespan.local-demo'::text AS actor_id,
        'workflow_event'::text AS subject_kind,
        (SELECT id::text FROM information_lifespan_workflow_event) AS subject_id,
        'information_lifespan.db_projection_rows_written'::text AS action,
        (SELECT id FROM information_lifespan_workflow_event) AS workflow_event_id,
        jsonb_build_object(
            'correlation_id', 'info-lifespan-demo-2026-06-29',
            'source_import_run', 'source_import_run:info-lifespan-demo-2026-06-29',
            'review_packet', 'review_packet:vaccine-near-expiry:8101',
            'manager_daily_brief_outcome', 'manager_daily_brief_outcome:synthetic-2026-06-29',
            'live_side_effects_allowed', false
        ) AS metadata
    UNION ALL
    SELECT
        '00000000-0000-4000-8000-00000000af02'::uuid,
        '2026-06-29T13:04:00Z'::timestamptz,
        'manager'::text,
        'local-demo-general-manager'::text,
        'workflow_event'::text,
        (SELECT id::text FROM information_lifespan_workflow_event),
        'information_lifespan.manager_daily_report_review_required'::text,
        (SELECT id FROM information_lifespan_workflow_event),
        jsonb_build_object(
            'correlation_id', 'info-lifespan-demo-2026-06-29',
            'audit_lineage', 'audit_lineage:info-lifespan-demo-2026-06-29',
            'customer_messages_allowed', false,
            'provider_writes_allowed', false,
            'medical_decisions_allowed', false
        )
) AS audit_row
ON CONFLICT (id) DO NOTHING;
