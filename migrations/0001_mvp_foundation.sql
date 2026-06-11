-- Core MVP data model for the pet resort agent foundation.
-- Encodes launch-critical owners, pets, reservations, evidence, review, workflow,
-- outbox, object metadata, and append-only audit surfaces.

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
    decided_at timestamptz,
    review_packet_id uuid REFERENCES review_packets(id),
    CONSTRAINT approval_records_decision_integrity CHECK (
        (
            status IN ('approved', 'rejected')
            AND decided_by_actor_kind IS NOT NULL
            AND decided_at IS NOT NULL
        )
        OR
        (
            status NOT IN ('approved', 'rejected')
            AND decided_by_actor_kind IS NULL
            AND decided_by_actor_id IS NULL
            AND decided_at IS NULL
        )
    ),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS outbox_records (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    topic text NOT NULL CHECK (length(trim(topic)) > 0),
    aggregate_kind text NOT NULL CHECK (length(trim(aggregate_kind)) > 0),
    aggregate_id uuid NOT NULL,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    status text NOT NULL CHECK (status IN ('pending', 'claimed', 'published', 'failed', 'dead_letter')),
    available_at timestamptz NOT NULL DEFAULT now(),
    claimed_at timestamptz,
    published_at timestamptz,
    failure_count integer NOT NULL DEFAULT 0 CHECK (failure_count >= 0),
    last_error text,
    created_at timestamptz NOT NULL DEFAULT now()
);

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
