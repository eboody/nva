-- Core MVP data model for the pet resort agent foundation.
-- Encodes launch-critical owners, pets, reservations, evidence, review, workflow,
-- outbox, object metadata, and append-only audit surfaces.
-- Deferred DB surfaces intentionally not claimed by this foundation migration:
-- auth/session/role/location authorization; infra metrics snapshots and dashboard read models;
-- durable job leasing and worker ownership; real provider source snapshots.
-- Durable processing lineage and safety model:
-- workflow_events are accepted before worker processing and carry the source/idempotency fact.
-- workflow_results are reviewable worker output, not execution proof or external delivery authority.
-- outbox_records require a matching approved approval_record before any publishable work can exist.
-- audit_events preserve workflow/outbox lineage append-only for review and replay evidence.
-- worker MVP remains fake deterministic and side-effect stubbed; no customer/provider/payment adapter is live.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE OR REPLACE FUNCTION review_gate_is_valid(gate text)
RETURNS boolean
LANGUAGE sql
IMMUTABLE
AS $$
    SELECT gate IN (
        'manager_approval',
        'medical_document_review',
        'behavior_review',
        'customer_message_approval',
        'refund_or_deposit_exception'
    );
$$;

CREATE OR REPLACE FUNCTION review_gates_are_valid(required_review_gates text[])
RETURNS boolean
LANGUAGE sql
IMMUTABLE
AS $$
    SELECT NOT EXISTS (
        SELECT 1
        FROM unnest(required_review_gates) AS required_review_gate(gate)
        WHERE review_gate_is_valid(required_review_gate.gate) IS NOT TRUE
    );
$$;

CREATE TABLE IF NOT EXISTS locations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    brand text NOT NULL,
    name text NOT NULL,
    timezone text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS customers (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    full_name text NOT NULL CHECK (length(trim(full_name)) > 0),
    email text,
    mobile_phone text,
    preferred_contact text NOT NULL CHECK (preferred_contact IN ('email', 'sms', 'phone', 'portal')),
    portal_provider text,
    portal_customer_id text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS pets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id uuid NOT NULL REFERENCES customers(id),
    name text NOT NULL CHECK (length(trim(name)) > 0),
    species text NOT NULL CHECK (species IN ('dog', 'cat', 'other')),
    species_other text,
    birth_date date,
    sex text CHECK (sex IN ('female', 'male', 'unknown')),
    spay_neuter_status text NOT NULL CHECK (spay_neuter_status IN ('spayed', 'neutered', 'intact', 'unknown')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS reservations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    location_id uuid NOT NULL REFERENCES locations(id),
    customer_id uuid NOT NULL REFERENCES customers(id),
    service text NOT NULL CHECK (service IN ('boarding', 'day_play', 'day_boarding', 'grooming', 'training', 'day_spa')),
    status text NOT NULL CHECK (status IN ('inquiry', 'requested', 'missing_info', 'vaccine_pending', 'special_review', 'waitlisted', 'offered', 'confirmed', 'checked_in', 'active', 'checked_out', 'cancelled', 'rejected')),
    starts_at timestamptz NOT NULL,
    ends_at timestamptz NOT NULL,
    source text NOT NULL CHECK (source IN ('portal', 'website_form', 'phone_transcript', 'sms', 'email', 'staff_created')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (ends_at > starts_at)
);

CREATE TABLE IF NOT EXISTS reservation_pets (
    reservation_id uuid NOT NULL REFERENCES reservations(id) ON DELETE CASCADE,
    pet_id uuid NOT NULL REFERENCES pets(id),
    PRIMARY KEY (reservation_id, pet_id)
);

CREATE TABLE IF NOT EXISTS documents (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    location_id uuid NOT NULL REFERENCES locations(id),
    subject_kind text NOT NULL CHECK (subject_kind IN ('customer', 'pet', 'reservation', 'incident')),
    subject_id uuid NOT NULL,
    classification text NOT NULL CHECK (classification IN ('vaccine_proof', 'waiver', 'photo', 'medical_record', 'incident_evidence', 'other')),
    source text NOT NULL CHECK (source IN ('customer_upload', 'staff_scan', 'staff_upload', 'email_ingest', 'provider_poll', 'provider_webhook', 'migration_import', 'unknown')),
    uploaded_by_actor_kind text NOT NULL CHECK (uploaded_by_actor_kind IN ('customer', 'staff', 'manager', 'system', 'agent')),
    uploaded_by_actor_id text,
    uploaded_at timestamptz NOT NULL,
    filename text NOT NULL CHECK (length(trim(filename)) > 0),
    mime_type text NOT NULL CHECK (length(trim(mime_type)) > 0),
    content_length_bytes bigint NOT NULL CHECK (content_length_bytes > 0),
    sha256 text NOT NULL CHECK (sha256 ~ '^[0-9a-fA-F]{64}$'),
    storage_bucket text NOT NULL CHECK (length(trim(storage_bucket)) > 0),
    storage_key text NOT NULL CHECK (length(trim(storage_key)) > 0),
    storage_version text NOT NULL CHECK (length(trim(storage_version)) > 0),
    virus_scan_status text NOT NULL CHECK (virus_scan_status IN ('pending', 'passed', 'failed', 'unsupported')),
    pii_redaction_status text NOT NULL CHECK (pii_redaction_status IN ('not_required', 'pending', 'redacted', 'failed')),
    verification_status text NOT NULL CHECK (verification_status IN ('received', 'quarantined_rejected', 'extracting', 'extraction_failed', 'awaiting_review', 'verified', 'rejected', 'superseded', 'archived')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS object_metadata (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id uuid REFERENCES documents(id),
    bucket text NOT NULL CHECK (length(trim(bucket)) > 0),
    object_key text NOT NULL CHECK (length(trim(object_key)) > 0),
    version text,
    content_type text,
    content_length_bytes bigint CHECK (content_length_bytes IS NULL OR content_length_bytes > 0),
    sha256 text CHECK (sha256 IS NULL OR sha256 ~ '^[0-9a-fA-F]{64}$'),
    retention_policy text,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (bucket, object_key, version)
);

CREATE TABLE IF NOT EXISTS vaccine_records (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    pet_id uuid NOT NULL REFERENCES pets(id),
    vaccine_name text NOT NULL CHECK (length(trim(vaccine_name)) > 0),
    source_document_id uuid NOT NULL REFERENCES documents(id),
    status text NOT NULL CHECK (status IN ('suggested_extracted', 'pending_review', 'verified_current', 'verified_expired', 'rejected', 'exception_requested', 'exception_approved', 'superseded')),
    effective_on date NOT NULL,
    expires_on date,
    review_gate text NOT NULL CHECK (review_gate_is_valid(review_gate)),
    reviewed_by_actor_kind text CHECK (reviewed_by_actor_kind IN ('customer', 'staff', 'manager', 'system', 'agent')),
    reviewed_by_actor_id text,
    reviewed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (expires_on IS NULL OR expires_on >= effective_on)
);

CREATE TABLE IF NOT EXISTS vaccine_extractions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id uuid NOT NULL REFERENCES documents(id),
    vaccine_record_id uuid REFERENCES vaccine_records(id),
    vaccine_extraction_schema_version text NOT NULL CHECK (vaccine_extraction_schema_version = 'vaccine_extraction.v1'),
    vaccine_name text NOT NULL CHECK (length(trim(vaccine_name)) > 0),
    effective_on date,
    expires_on date,
    confidence numeric(4, 3) NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    uncertainty_policy text NOT NULL CHECK (uncertainty_policy = 'medical_document_uncertainty_policy_requires_staff_review'),
    auto_accept_threshold numeric(4, 3) NOT NULL CHECK (auto_accept_threshold >= 0 AND auto_accept_threshold <= 1),
    raw_text_ref text NOT NULL CHECK (length(trim(raw_text_ref)) > 0),
    extraction_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (expires_on IS NULL OR effective_on IS NULL OR expires_on >= effective_on)
);

CREATE TABLE IF NOT EXISTS pet_eligibility_projections (
    pet_id uuid PRIMARY KEY REFERENCES pets(id),
    rabies_current boolean NOT NULL DEFAULT false,
    source_vaccine_record_id uuid REFERENCES vaccine_records(id),
    status text NOT NULL CHECK (status IN ('awaiting_medical_document_review', 'eligible_from_approved_vaccine_document', 'ineligible_after_rejected_vaccine_document')),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS operational_tasks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    location_id uuid REFERENCES locations(id),
    reservation_id uuid REFERENCES reservations(id),
    customer_id uuid REFERENCES customers(id),
    pet_id uuid REFERENCES pets(id),
    title text NOT NULL CHECK (length(trim(title)) > 0),
    task_kind text NOT NULL,
    status text NOT NULL CHECK (status IN ('draft', 'open', 'assigned', 'blocked', 'completed', 'cancelled')),
    priority text NOT NULL CHECK (priority IN ('low', 'normal', 'high', 'urgent')),
    assigned_to_staff_id text,
    due_at timestamptz,
    created_by_actor_kind text NOT NULL CHECK (created_by_actor_kind IN ('customer', 'staff', 'manager', 'system', 'agent')),
    created_by_actor_id text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS care_notes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_kind text NOT NULL CHECK (subject_kind IN ('pet', 'reservation', 'incident')),
    subject_id uuid NOT NULL,
    kind text NOT NULL CHECK (kind IN ('feeding', 'medication', 'medical', 'behavior', 'grooming', 'training', 'general')),
    visibility text NOT NULL CHECK (visibility IN ('internal_only', 'customer_visible', 'customer_visible_after_review')),
    body text NOT NULL CHECK (length(trim(body)) > 0),
    author_actor_kind text NOT NULL CHECK (author_actor_kind IN ('customer', 'staff', 'manager', 'system', 'agent')),
    author_actor_id text,
    recorded_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS incidents (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    location_id uuid NOT NULL REFERENCES locations(id),
    primary_subject_kind text NOT NULL CHECK (primary_subject_kind IN ('pet', 'reservation', 'customer', 'location')),
    primary_subject_id uuid NOT NULL,
    category text NOT NULL CHECK (category IN ('injury', 'altercation', 'behavior', 'medication', 'escape', 'property', 'customer_service', 'other')),
    severity text NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    status text NOT NULL CHECK (status IN ('reported', 'needs_manager_review', 'investigation_open', 'customer_message_review', 'resolved', 'closed', 'reopened', 'legal_hold')),
    reported_by_actor_kind text NOT NULL CHECK (reported_by_actor_kind IN ('customer', 'staff', 'manager', 'system', 'agent')),
    reported_by_actor_id text,
    reported_at timestamptz NOT NULL,
    summary text NOT NULL CHECK (length(trim(summary)) > 0),
    required_review_gates text[] NOT NULL DEFAULT '{}',
    CONSTRAINT incidents_required_review_gates_valid CHECK (review_gates_are_valid(required_review_gates)),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS messages (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_kind text NOT NULL CHECK (subject_kind IN ('customer', 'pet', 'reservation', 'incident', 'approval')),
    subject_id uuid NOT NULL,
    direction text NOT NULL CHECK (direction IN ('inbound_received', 'outbound_draft', 'outbound_queued', 'outbound_sent')),
    channel text NOT NULL CHECK (channel IN ('email', 'sms', 'phone_note', 'portal', 'internal')),
    status text NOT NULL CHECK (status IN ('draft_created', 'approval_requested', 'approved_to_queue', 'queued', 'send_attempted', 'delivered', 'failed', 'suppressed', 'cancelled')),
    body_ref text NOT NULL CHECK (length(trim(body_ref)) > 0),
    approval_gate text CHECK (approval_gate IS NULL OR review_gate_is_valid(approval_gate)),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- payment_deposit_projections retain provider-observed state history: multiple rows for one
-- reservation are legal when they represent distinct provider payment ids or observations.
-- They are projections of payment/deposit facts, not a one-row execution authority ledger.
CREATE TABLE IF NOT EXISTS payment_deposit_projections (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    reservation_id uuid NOT NULL REFERENCES reservations(id),
    provider text NOT NULL,
    provider_payment_id text,
    status text NOT NULL CHECK (status IN ('not_required', 'required', 'authorized', 'captured', 'refunded', 'failed', 'waived')),
    amount_minor_units bigint CHECK (amount_minor_units IS NULL OR amount_minor_units >= 0),
    currency text CHECK (currency IS NULL OR currency IN ('usd')),
    due_at timestamptz,
    synced_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS workflow_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_name text NOT NULL CHECK (length(trim(workflow_name)) > 0),
    event_kind text NOT NULL CHECK (length(trim(event_kind)) > 0),
    subject_kind text NOT NULL,
    subject_id uuid NOT NULL,
    idempotency_key text NOT NULL UNIQUE,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    occurred_at timestamptz NOT NULL DEFAULT now(),
    recorded_at timestamptz NOT NULL DEFAULT now()
);

-- workflow_results retain one row per processing attempt. Retries and later reviewable results
-- may therefore coexist for one workflow_event_id; created_at and row id preserve their history.
CREATE TABLE IF NOT EXISTS workflow_results (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_event_id uuid NOT NULL REFERENCES workflow_events(id),
    status text NOT NULL CHECK (status IN ('succeeded', 'failed', 'needs_review', 'deferred', 'cancelled')),
    result jsonb NOT NULL DEFAULT '{}'::jsonb,
    error_code text,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS review_packets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_kind text NOT NULL,
    subject_id uuid NOT NULL,
    gate text NOT NULL CHECK (review_gate_is_valid(gate)),
    status text NOT NULL CHECK (status IN ('draft', 'ready_for_review', 'in_review', 'approved', 'rejected', 'cancelled')),
    evidence_document_ids uuid[] NOT NULL DEFAULT '{}',
    workflow_event_id uuid REFERENCES workflow_events(id),
    reviewed_action_id text CHECK (reviewed_action_id IS NULL OR length(trim(reviewed_action_id)) > 0),
    created_by_actor_kind text NOT NULL CHECK (created_by_actor_kind IN ('customer', 'staff', 'manager', 'system', 'agent')),
    created_by_actor_id text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS approval_records (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    target_kind text NOT NULL CHECK (target_kind IN ('reservation', 'document', 'vaccine_record', 'incident', 'message')),
    target_id uuid NOT NULL,
    gate text NOT NULL CHECK (review_gate_is_valid(gate)),
    status text NOT NULL CHECK (status IN ('approval_requested', 'approved', 'rejected', 'cancelled', 'superseded')),
    requested_by_actor_kind text NOT NULL CHECK (requested_by_actor_kind IN ('customer', 'staff', 'manager', 'system', 'agent')),
    requested_by_actor_id text,
    requested_at timestamptz NOT NULL,
    decided_by_actor_kind text CHECK (decided_by_actor_kind IN ('customer', 'staff', 'manager', 'system', 'agent')),
    decided_by_actor_id text,
    decided_by_actor_persona text CHECK (decided_by_actor_persona IN ('general_manager', 'assistant_general_manager', 'front_desk_lead', 'front_desk_agent', 'regional_operator', 'operations_analyst')),
    legacy_persona_missing boolean NOT NULL DEFAULT false,
    decided_at timestamptz,
    review_packet_id uuid REFERENCES review_packets(id),
    CONSTRAINT approval_records_decision_integrity CHECK (
        (
            status IN ('approved', 'rejected')
            AND decided_by_actor_kind IS NOT NULL
            AND decided_by_actor_id IS NOT NULL
            AND decided_by_actor_persona IS NOT NULL
            AND NOT legacy_persona_missing
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
    ),
    CONSTRAINT approval_records_decider_persona_kind_integrity CHECK (
        legacy_persona_missing
        OR decided_by_actor_kind IS NULL
        OR (decided_by_actor_kind = 'manager' AND decided_by_actor_persona IN ('general_manager', 'assistant_general_manager', 'regional_operator'))
        OR (decided_by_actor_kind = 'staff' AND decided_by_actor_persona IN ('front_desk_lead', 'front_desk_agent', 'operations_analyst'))
    ),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
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

CREATE TABLE IF NOT EXISTS manager_daily_brief_outcomes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_event_id uuid NOT NULL REFERENCES workflow_events(id),
    approval_record_id uuid NOT NULL REFERENCES approval_records(id),
    action_id text NOT NULL CHECK (length(trim(action_id)) > 0),
    outcome text NOT NULL CHECK (outcome IN ('completed', 'deferred', 'suppressed_by_manager', 'source_fact_was_wrong')),
    actor_id text NOT NULL CHECK (length(trim(actor_id)) > 0),
    actor_persona text NOT NULL CHECK (actor_persona IN ('general_manager', 'assistant_general_manager', 'front_desk_lead', 'front_desk_agent')),
    feedback text NOT NULL DEFAULT '',
    owner_persona text NOT NULL CHECK (owner_persona IN ('general_manager', 'assistant_general_manager', 'front_desk_lead', 'front_desk_agent')),
    action_kind text NOT NULL CHECK (action_kind IN ('review_demand_against_staffing_plan', 'resolve_checkout_exception', 'approve_retention_follow_up_draft', 'investigate_source_data_quality_issue', 'review_capacity_labor_recommendation')),
    before_minutes integer NOT NULL CHECK (before_minutes > 0),
    actual_minutes integer NOT NULL CHECK (actual_minutes > 0),
    reported_estimated_minutes_difference integer NOT NULL CHECK (reported_estimated_minutes_difference >= 0),
    location_id uuid NOT NULL REFERENCES locations(id),
    operating_day date NOT NULL,
    source_refs jsonb NOT NULL DEFAULT '[]'::jsonb,
    correlation_id text NOT NULL CHECK (length(trim(correlation_id)) > 0),
    recorded_at timestamptz NOT NULL DEFAULT now(),
    schema_version integer NOT NULL DEFAULT 1 CHECK (schema_version = 1),
    CONSTRAINT manager_daily_brief_outcomes_action_id_key UNIQUE (action_id)
);

CREATE TABLE IF NOT EXISTS data_quality_hygiene_outcomes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_event_id uuid NOT NULL REFERENCES workflow_events(id),
    approval_record_id uuid NOT NULL REFERENCES approval_records(id),
    action_id text NOT NULL CHECK (length(trim(action_id)) > 0),
    outcome text NOT NULL CHECK (outcome IN ('completed', 'deferred', 'suppressed_by_manager', 'source_fact_was_wrong', 'not_actionable')),
    actor_id text NOT NULL CHECK (length(trim(actor_id)) > 0),
    actor_persona text NOT NULL CHECK (actor_persona IN ('general_manager', 'assistant_general_manager', 'front_desk_lead', 'front_desk_agent', 'regional_operator', 'operations_analyst')),
    feedback text NOT NULL DEFAULT '',
    issue_refs jsonb NOT NULL DEFAULT '[]'::jsonb,
    resolution_status_after_review text NOT NULL CHECK (resolution_status_after_review IN ('open', 'acknowledged', 'ignored', 'repaired', 'superseded')),
    owner_persona text NOT NULL CHECK (owner_persona IN ('general_manager', 'assistant_general_manager', 'front_desk_lead', 'front_desk_agent', 'regional_operator', 'operations_analyst')),
    action_kind text NOT NULL CHECK (action_kind IN ('investigate_missing_source_evidence', 'reconcile_duplicate_customer_or_pet_candidate', 'complete_missing_pet_or_customer_profile_fields', 'review_stale_vaccination_source_freshness', 'normalize_ambiguous_service_line_naming', 'review_checkout_or_unclosed_reservation_evidence', 'escalate_sensitive_or_quarantined_payload', 'review_payment_state_conflict')),
    before_minutes integer NOT NULL CHECK (before_minutes > 0),
    actual_minutes integer NOT NULL CHECK (actual_minutes > 0),
    reported_estimated_minutes_difference integer NOT NULL CHECK (reported_estimated_minutes_difference >= 0),
    location_id uuid NOT NULL REFERENCES locations(id),
    operating_day date NOT NULL,
    source_refs jsonb NOT NULL DEFAULT '[]'::jsonb,
    correlation_id text NOT NULL CHECK (length(trim(correlation_id)) > 0),
    recorded_at timestamptz NOT NULL DEFAULT now(),
    schema_version integer NOT NULL DEFAULT 1 CHECK (schema_version = 1),
    CONSTRAINT data_quality_hygiene_outcomes_action_id_key UNIQUE (action_id)
);

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

-- One approval may issue exactly one internal handoff capability. This row is the
-- durable, relational image of the opaque application authority. It binds the
-- exact topic, gate, target, and payload before an outbox row can be admitted.
CREATE TABLE IF NOT EXISTS approval_outbox_bindings (
    approval_record_id uuid PRIMARY KEY REFERENCES approval_records(id),
    topic text NOT NULL,
    review_gate text NOT NULL CHECK (review_gate_is_valid(review_gate)),
    aggregate_kind text NOT NULL CHECK (aggregate_kind IN ('reservation', 'document', 'vaccine_record', 'incident', 'message')),
    aggregate_id uuid NOT NULL,
    payload jsonb NOT NULL,
    authorized_by_actor_kind text NOT NULL CHECK (authorized_by_actor_kind IN ('staff', 'manager')),
    authorized_by_actor_id text NOT NULL CHECK (length(trim(authorized_by_actor_id)) > 0),
    authorized_by_actor_persona text NOT NULL CHECK (authorized_by_actor_persona IN ('general_manager', 'assistant_general_manager', 'front_desk_lead', 'front_desk_agent', 'regional_operator', 'operations_analyst')),
    authorized_at timestamptz NOT NULL,
    consumed_by_outbox_id uuid UNIQUE,
    CONSTRAINT approval_outbox_bindings_actor_persona_kind_integrity CHECK (
        (authorized_by_actor_kind = 'manager' AND authorized_by_actor_persona IN ('general_manager', 'assistant_general_manager', 'regional_operator'))
        OR (authorized_by_actor_kind = 'staff' AND authorized_by_actor_persona IN ('front_desk_lead', 'front_desk_agent', 'operations_analyst'))
    ),
    CONSTRAINT approval_outbox_bindings_internal_topic_is_closed CHECK (
        topic IN (
            'internal.data_quality_hygiene.reviewed_handoff',
            'internal.site_finance.reviewed_handoff'
        )
    )
);

CREATE TABLE IF NOT EXISTS outbox_records (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    idempotency_key text NOT NULL,
    approval_record_id uuid NOT NULL REFERENCES approval_records(id),
    topic text NOT NULL,
    review_gate text NOT NULL CHECK (review_gate_is_valid(review_gate)),
    aggregate_kind text NOT NULL CHECK (aggregate_kind IN ('reservation', 'document', 'vaccine_record', 'incident', 'message')),
    aggregate_id uuid NOT NULL,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    status text NOT NULL CHECK (status IN ('pending', 'claimed', 'published', 'failed', 'dead_letter')),
    available_at timestamptz NOT NULL DEFAULT now(),
    claimed_at timestamptz,
    published_at timestamptz,
    failure_count integer NOT NULL DEFAULT 0 CHECK (failure_count >= 0),
    last_error text,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT outbox_records_approval_record_id_key UNIQUE (approval_record_id),
    CONSTRAINT outbox_records_idempotency_key_key UNIQUE (idempotency_key),
    CONSTRAINT outbox_internal_handoff_topic_is_closed CHECK (
        topic IN (
            'internal.data_quality_hygiene.reviewed_handoff',
            'internal.site_finance.reviewed_handoff'
        )
    ),
    CONSTRAINT outbox_records_status_timestamp_integrity CHECK (
        (status = 'pending' AND claimed_at IS NULL AND published_at IS NULL)
        OR (status = 'claimed' AND claimed_at IS NOT NULL AND published_at IS NULL)
        OR (status = 'published' AND claimed_at IS NOT NULL AND published_at IS NOT NULL)
        OR (status = 'failed' AND failure_count > 0 AND last_error IS NOT NULL AND published_at IS NULL)
        OR (status = 'dead_letter' AND failure_count > 0 AND last_error IS NOT NULL AND published_at IS NULL)
    )
);

-- Consumption points at the durable row whose validated insertion consumed the
-- binding. The relation is added after both tables exist to avoid making trigger
-- nesting depth or caller-controlled session state part of the authority model.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'approval_outbox_bindings_consumed_outbox_fkey'
          AND conrelid = 'approval_outbox_bindings'::regclass
    ) THEN
        ALTER TABLE approval_outbox_bindings
            ADD CONSTRAINT approval_outbox_bindings_consumed_outbox_fkey
            FOREIGN KEY (consumed_by_outbox_id)
            REFERENCES outbox_records(id);
    END IF;
END;
$$;

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

CREATE OR REPLACE FUNCTION enforce_outbox_approval_record_is_approved()
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
    THEN
        RAISE EXCEPTION 'outbox_records require a matching approved approval_record';
    END IF;

    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION enforce_outbox_internal_handoff_binding()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    binding approval_outbox_bindings%ROWTYPE;
BEGIN
    SELECT * INTO binding
    FROM approval_outbox_bindings
    WHERE approval_record_id = NEW.approval_record_id
    FOR UPDATE;

    IF binding.approval_record_id IS NULL
        OR binding.consumed_by_outbox_id IS NOT NULL
        OR binding.topic <> NEW.topic
        OR binding.review_gate <> NEW.review_gate
        OR binding.aggregate_kind <> NEW.aggregate_kind
        OR binding.aggregate_id <> NEW.aggregate_id
        OR binding.payload <> NEW.payload
    THEN
        RAISE EXCEPTION 'outbox_records require one matching unconsumed approval_outbox_binding';
    END IF;

    UPDATE approval_outbox_bindings
    SET consumed_by_outbox_id = NEW.id
    WHERE approval_record_id = NEW.approval_record_id;

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS outbox_records_approved_approval_record ON outbox_records;
CREATE TRIGGER outbox_records_approved_approval_record
    BEFORE INSERT OR UPDATE OF approval_record_id, aggregate_kind, aggregate_id, review_gate ON outbox_records
    FOR EACH ROW EXECUTE FUNCTION enforce_outbox_approval_record_is_approved();

DROP TRIGGER IF EXISTS outbox_records_internal_handoff_binding ON outbox_records;
CREATE TRIGGER outbox_records_internal_handoff_binding
    AFTER INSERT ON outbox_records
    FOR EACH ROW EXECUTE FUNCTION enforce_outbox_internal_handoff_binding();

CREATE OR REPLACE FUNCTION reject_outbox_authority_identity_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'outbox_records are durable history and cannot be deleted';
    END IF;

    IF OLD.id <> NEW.id
        OR OLD.idempotency_key <> NEW.idempotency_key
        OR OLD.aggregate_kind <> NEW.aggregate_kind
        OR OLD.aggregate_id <> NEW.aggregate_id
        OR OLD.topic <> NEW.topic
        OR OLD.payload <> NEW.payload
        OR OLD.approval_record_id <> NEW.approval_record_id
        OR OLD.review_gate <> NEW.review_gate
        OR OLD.available_at <> NEW.available_at
        OR OLD.created_at <> NEW.created_at
    THEN
        RAISE EXCEPTION 'outbox authority identity is immutable after admission';
    END IF;

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS outbox_records_authority_identity_immutable_update ON outbox_records;
CREATE TRIGGER outbox_records_authority_identity_immutable_update
    BEFORE UPDATE OF id, idempotency_key, aggregate_kind, aggregate_id, topic, payload,
        approval_record_id, review_gate, available_at, created_at ON outbox_records
    FOR EACH ROW EXECUTE FUNCTION reject_outbox_authority_identity_mutation();

DROP TRIGGER IF EXISTS outbox_records_durable_history_delete ON outbox_records;
CREATE TRIGGER outbox_records_durable_history_delete
    BEFORE DELETE ON outbox_records
    FOR EACH ROW EXECUTE FUNCTION reject_outbox_authority_identity_mutation();

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
            EXISTS (
                SELECT 1
                FROM approval_outbox_bindings
                WHERE approval_record_id = OLD.id
            )
            OR EXISTS (
                SELECT 1
                FROM outbox_records
                WHERE approval_record_id = OLD.id
            )
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

CREATE TABLE IF NOT EXISTS audit_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    occurred_at timestamptz NOT NULL DEFAULT now(),
    actor_kind text NOT NULL CHECK (actor_kind IN ('customer', 'staff', 'manager', 'system', 'agent')),
    actor_id text,
    subject_kind text NOT NULL CHECK (subject_kind IN ('customer', 'pet', 'reservation', 'location', 'document', 'vaccine_record', 'care_note', 'incident', 'message', 'approval', 'workflow_event', 'external')),
    subject_id text NOT NULL,
    action text NOT NULL CHECK (length(trim(action)) > 0),
    workflow_event_id uuid REFERENCES workflow_events(id),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    recorded_at timestamptz NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION reject_audit_events_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'audit_events is append-only';
END;
$$;

DROP TRIGGER IF EXISTS audit_events_append_only_update ON audit_events;
CREATE TRIGGER audit_events_append_only_update
    BEFORE UPDATE ON audit_events
    FOR EACH ROW EXECUTE FUNCTION reject_audit_events_mutation();

DROP TRIGGER IF EXISTS audit_events_append_only_delete ON audit_events;
CREATE TRIGGER audit_events_append_only_delete
    BEFORE DELETE ON audit_events
    FOR EACH ROW EXECUTE FUNCTION reject_audit_events_mutation();

CREATE INDEX IF NOT EXISTS pets_customer_id_idx ON pets(customer_id);
CREATE INDEX IF NOT EXISTS reservations_customer_id_idx ON reservations(customer_id);
CREATE INDEX IF NOT EXISTS reservation_pets_pet_id_idx ON reservation_pets(pet_id);
CREATE INDEX IF NOT EXISTS documents_subject_idx ON documents(subject_kind, subject_id);
CREATE INDEX IF NOT EXISTS vaccine_records_pet_id_idx ON vaccine_records(pet_id);
CREATE INDEX IF NOT EXISTS incidents_location_status_idx ON incidents(location_id, status);
CREATE INDEX IF NOT EXISTS messages_subject_idx ON messages(subject_kind, subject_id);
CREATE INDEX IF NOT EXISTS workflow_events_subject_idx ON workflow_events(subject_kind, subject_id);
CREATE INDEX IF NOT EXISTS outbox_records_status_available_idx ON outbox_records(status, available_at);
CREATE INDEX IF NOT EXISTS audit_events_subject_idx ON audit_events(subject_kind, subject_id);
