# AI runtime test harness fixtures

Purpose: define deterministic fixture shapes and golden assertions for testing the app-owned AI/Hermes runtime boundary before wiring any production runtime, provider adapter, or live worker. These fixtures are contract tests for prompt-packet assembly, model-output validation, retry behavior, audit-safe logging, and policy enforcement.

Status: draft implementation handoff. This document does not approve production automation. The implementation team should be able to build all tests with deterministic fake agents and static JSON/YAML fixtures before any live Hermes/LLM invocation exists.

## Test harness goal

The runtime test harness should prove that the application can:

1. Convert a normalized `WorkflowEvent` plus entity snapshots and policy packet into a stable `AgentPromptPacket<T>`.
2. Call a deterministic fake runtime that returns scripted model output.
3. Validate the output into a `WorkflowResult<T>`-style envelope.
4. Reject forbidden actions, unsafe claims, malformed output, or missing approval gates.
5. Retry once for validation-correctable malformed output, then fail safely if the second result is still invalid.
6. Persist/audit only redacted safe logs that preserve traceability without leaking raw PII, vaccine OCR, medication details, incident narratives, payment refs, or customer-message bodies.

The harness should not test model quality. It should test the deterministic boundary around the model.

## Fixture directory layout

Recommended repo layout:

```text
domain/tests/ai_runtime_contracts.rs            # Rust contract tests for schema/policy validation
fixtures/ai-runtime/
  README.md                                    # how to add scenarios
  schemas/
    prompt-packet.schema.json                  # serialized AgentPromptPacket-like shape
    workflow-result.schema.json                # common WorkflowResult envelope
    scenario-expected.schema.json              # expected assertions shape
  scenarios/
    inquiry_incomplete_owner_pet.yaml
    vaccine_missing_or_expired.yaml
    booking_no_availability.yaml
    behavior_risk_review.yaml
    medication_special_care_request.yaml
    incident_report.yaml
    daily_note_staff_update.yaml
    validation_retry_once.yaml
    validation_retry_exhausted.yaml
    safe_log_redaction.yaml
```

Fixtures may start as YAML for readability. Tests should load them into typed DTOs and then convert to domain types; do not branch on raw YAML strings inside policy code.

## Canonical fixture shape

Each scenario fixture should use the same top-level shape:

```yaml
scenario_id: inquiry_incomplete_owner_pet
workflow_name: inquiry-intake
purpose: "New inquiry with incomplete owner/pet details routes to missing-info draft."
source_refs:
  event_catalog: docs/architecture/pet-resort-workflow-events.md#inquiryreceived
  domain_contract: docs/architecture/domain-contract-skeleton.md#safety-and-automation-boundary-contracts

input:
  event_payload:
    event_id: 00000000-0000-0000-0000-000000000101
    type: InquiryReceived
    occurred_at: "2026-06-11T12:00:00Z"
    source: customer_portal
    actor: { kind: Customer, customer_id: "customer-alpha" }
    location_id: "location-nashville-001"
    subject: { kind: External, provider: customer_portal, id: lead-101 }
    related_ids: { customer_id: null, pet_ids: [], reservation_id: null, evidence_ids: [lead-message-101] }
    payload_ref: lead-message-101
  entity_snapshots:
    location: { id: "location-nashville-001", brand: "PetSuites", policy_refs: [vaccine-policy-2026-06, intake-policy-2026-06] }
    customer: null
    pets: []
    reservation: null
    documents: []
    care_notes: []
    capacity: null
    tasks: []
  policy_packet:
    policy_snapshot_id: policy-snapshot-2026-06-11-a
    automation_level: DraftOnly
    allowed_actions: [ReadEntities, ExtractStructuredData, CreateInternalTask, DraftCustomerMessage, FlagRisk]
    required_reviews: [CustomerMessageApproval]
    forbidden_actions:
      - confirm booking
      - promise availability
      - send customer message without approval
      - approve medical or vaccine facts
    data_handling:
      redact_fields: [owner_phone, owner_email, raw_message_body, medication_name, medication_dose, incident_narrative, payment_reference, raw_ocr_text]
      safe_log_fields: [scenario_id, event_id, event_type, workflow_name, location_id, policy_snapshot_id, validation_status, retry_count]
  expected_output_schema:
    result_envelope: WorkflowResult<InquiryIntakeOutput>
    structured_output_required: true
    allowed_statuses: [NeedsMoreInformation, NeedsHumanReview, FailedSafely]
    required_fields:
      - status
      - summary
      - structured_output.missing_info
      - recommended_actions
      - verification

fake_runtime:
  first_response_ref: responses/inquiry_incomplete_owner_pet.valid.json
  retry_response_ref: null

expect:
  status: NeedsMoreInformation
  approval_required: true
  required_review_gates: [CustomerMessageApproval]
  missing_info:
    - owner full name
    - owner preferred contact channel
    - pet name
    - pet species
    - requested service/date range
  required_recommended_actions:
    - kind: InternalTask
      contains: "Collect missing owner and pet details"
    - kind: DraftMessage
      customer_safe: true
  forbidden_recommended_actions:
    - kind: UpdateStatus
    - contains: "confirmed"
    - contains: "space is available"
  risk_flags: []
  verification_contains:
    - "No booking availability or eligibility decision made"
    - "Customer message is draft-only"
  safe_log_assertions:
    includes: [scenario_id, event_id, workflow_name, policy_snapshot_id, validation_status]
    excludes: [owner_phone, owner_email, raw_message_body]
```

### Required fixture sections

- `scenario_id`: stable snake_case id used in snapshot names and test names.
- `workflow_name`: one baseline agent/workflow name such as `inquiry-intake`, `booking-triage`, `vaccine-document`, `daily-care-update`, or `incident-escalation`.
- `input.event_payload`: normalized semantic event payload, not raw Gingr/customer-portal webhook JSON.
- `input.entity_snapshots`: minimal current-world state the runtime may read. Use explicit `null`/empty lists for missing records so tests prove missing-data behavior.
- `input.policy_packet`: the location-scoped policy snapshot, allowed actions, forbidden actions, review gates, and data-handling rules.
- `input.expected_output_schema`: schema name and required shape the validator enforces.
- `fake_runtime`: scripted first/second responses. A live model is never required for these tests.
- `expect`: golden assertions independent of exact prose wording.

## Common expected output contract

Every validated result should normalize to:

```yaml
status: Completed | NeedsHumanReview | RejectedByPolicy | NeedsMoreInformation | FailedSafely
summary: non-empty, <= 500 chars
structured_output: scenario-specific object or null only when status is FailedSafely
recommended_actions:
  - InternalTask | DraftMessage | UpdateStatus | RequestHumanReview
risk_flags: []
verification:
  - non-empty evidence/review notes
human_review_reason: null | non-empty reason
metadata:
  approval_required: boolean
  required_review_gates: [ManagerApproval | MedicalDocumentReview | BehaviorReview | CustomerMessageApproval | RefundOrDepositException]
  missing_info: [semantic missing-info item]
  forbidden_action_violations: []
  redaction_profile: safe-log-v1
```

If the current Rust `WorkflowResult<T>` does not yet include `metadata`, test DTOs can keep `approval_required`, `missing_info`, and redaction facts in a scenario-specific `structured_output` while preserving the common envelope above.

## Golden assertion categories

Each scenario test should assert these categories rather than brittle full-text equality:

1. Prompt packet construction
   - Contains the expected `workflow_name`, normalized `WorkflowEvent`, entity snapshot ids, policy snapshot id, allowed actions, forbidden actions, and output schema name.
   - Does not contain raw provider webhook JSON, raw OCR text, full message bodies, payment refs, or unredacted contact/medical/incident details in log/debug views.

2. Safe action boundaries
   - No action outside `policy_packet.allowed_actions` appears in `recommended_actions`.
   - No `UpdateStatus` appears unless the policy permits status suggestion and the result labels it as a suggestion requiring approval.
   - No provider mutation, payment/refund/waiver, booking confirmation, vaccine approval, incident closure, group-play reinstatement, or customer send is treated as completed by the AI result.

3. Approval gates
   - `approval_required` is true whenever any customer-facing draft, medical/vaccine ambiguity, behavior risk, incident, payment/deposit exception, capacity exception, or policy exception appears.
   - `required_review_gates` includes the exact gates implied by the policy packet and scenario facts.
   - `human_review_reason` is present for `NeedsHumanReview` and policy-sensitive `NeedsMoreInformation` cases.

4. Missing information
   - Missing-info lists use semantic labels, not free-form model guesswork.
   - Missing owner/pet/profile/vaccine/capacity/care/incident fields are explicitly listed when the scenario depends on them.
   - The draft message asks only for missing/clarifying information and does not promise outcome.

5. Safe draft behavior
   - Draft customer messages are present only when `DraftCustomerMessage` is allowed.
   - Drafts avoid diagnosis, blame, legal admissions, payment promises, booking confirmations, vaccine approvals, and final incident conclusions.
   - Drafts include human-review status either in metadata or by routing through `RequestHumanReview(CustomerMessageApproval)`.

6. Validation failures and retry-once behavior
   - Malformed JSON/schema results trigger exactly one retry with a validation-error repair prompt.
   - Policy-violating results may retry once only if the failure is repairable by removing/rewriting the output; unrecoverable forbidden action attempts fail safely.
   - After the retry is exhausted, the runtime returns `FailedSafely`, creates/requests an internal review task where allowed, logs the validation errors safely, and does not persist unsafe recommended actions.

7. Redaction/safe logs
   - Safe logs include ids, event type, workflow name, policy snapshot id, validation status, retry count, result status, and review gates.
   - Safe logs exclude owner email/phone/address, raw customer body, raw OCR text, medication name/dose/schedule, allergy/condition details, incident narrative, payment reference/provider payload, and full customer-message drafts.
   - Debug output for care, medication, temperament, incident, and document snapshots should be redacted or summarized.

## Required scenario fixtures

### 1. New inquiry with incomplete owner/pet details

Scenario id: `inquiry_incomplete_owner_pet`

Event payload:
- Event type: `InquiryReceived` / `inquiry.received`.
- Subject: external lead or provisional customer.
- Payload refs: customer free-text inquiry evidence, requested service/date if known.

Entity snapshots:
- Location exists with intake and customer-message approval policy refs.
- Customer missing or incomplete: no full name, no validated contact preference, or no portal account.
- Pet missing or incomplete: no species/name/age/spay-neuter and no vaccine profile.
- No reservation exists yet, or only an external lead exists.

Policy packet:
- `automation_level: DraftOnly`.
- Allowed actions: `ReadEntities`, `ExtractStructuredData`, `CreateInternalTask`, optional `DraftCustomerMessage`, `FlagRisk`.
- Required review: `CustomerMessageApproval`.
- Forbidden: confirm booking, promise availability, mark pet eligible, send customer message.

Expected output schema:
- `InquiryIntakeOutput` with `extracted_lead`, `missing_info`, `draft_follow_up`, `internal_task`, and `risk_flags`.

Golden assertions:
- Status is `NeedsMoreInformation` or `NeedsHumanReview`.
- `missing_info` includes owner identity/contact and pet basics.
- Any message is a draft follow-up asking for missing facts only.
- No reservation status update, availability promise, eligibility decision, or send action.
- Safe logs omit raw message body and contact values.

### 2. Missing vaccine document or expired vaccine

Scenario id: `vaccine_missing_or_expired`

Event payload:
- Event type: `VaccineDocumentUploaded` / `document.uploaded` for upload, or `BookingTriageNeeded` / `vaccine.extraction_needed` for derived review.
- Subject: pet, with related customer/reservation/document ids.

Entity snapshots:
- Pet and reservation exist.
- One of:
  - no vaccine document for required vaccines;
  - document exists but has no extracted record;
  - extracted vaccine date is expired;
  - OCR/source is ambiguous or not verified as approved proof.

Policy packet:
- Required review: `MedicalDocumentReview`.
- Allowed actions: `ReadEntities`, `ExtractStructuredData`, `CreateInternalTask`, `FlagRisk`, optional draft follow-up.
- Forbidden: final approve vaccine, mark reservation confirmed, waive vaccine requirement, infer vet source from OCR confidence.

Expected output schema:
- `VaccineDocumentReviewOutput` with `required_vaccines`, `document_status`, `missing_or_expired`, `review_task`, and optional customer-safe draft.

Golden assertions:
- Status is `NeedsMoreInformation` for missing document or `NeedsHumanReview` for expired/ambiguous proof.
- `approval_required: true` and gate includes `MedicalDocumentReview`.
- Required internal task is document/vaccine review.
- Any customer draft asks for updated proof or says staff will review; it does not declare the pet cleared or not cleared.
- Safe logs exclude raw OCR text and document content.

### 3. Full booking / no availability

Scenario id: `booking_no_availability`

Event payload:
- Event type: `BookingTriageNeeded` derived from `BookingRequested`.
- Subject: reservation.

Entity snapshots:
- Customer, pet, and reservation exist.
- Capacity/availability snapshot shows requested service/date/accommodation is full, stale, or missing sellable space.
- Optional waitlist policy ref exists.

Policy packet:
- Required review: `ManagerApproval` for overbooking/capacity exception and `CustomerMessageApproval` for any customer-facing waitlist/alternate-date copy.
- Allowed actions: `ReadEntities`, `CreateInternalTask`, `SuggestReservationStatus`, `DraftCustomerMessage`, `FlagRisk`.
- Forbidden: invent availability, confirm booking, overbook, create provider hold, charge deposit, send denial/waitlist message.

Expected output schema:
- `BookingTriageOutput` with `availability_decision`, `suggested_status`, `alternatives`, `review_gates`, `draft_customer_message`.

Golden assertions:
- Result status is `NeedsHumanReview` or `Completed` only as a recommendation/draft; it must not imply provider mutation.
- Suggested status may be `Waitlisted` or `SpecialReview`, but must be labeled as suggestion requiring approval.
- No booking confirmation, deposit charge, or provider write action.
- Missing/stale capacity creates `NeedsHumanReview` or `NeedsMoreInformation`, not invented availability.
- Safe logs include capacity snapshot id but not raw provider payload.

### 4. Aggression or behavior-risk review

Scenario id: `behavior_risk_review`

Event payload:
- Event type: `PetProfileCreated`, `DailyNoteCreated`, `IncidentCreated`, or `BookingTriageNeeded` depending on source.
- Subject: pet or reservation.

Entity snapshots:
- Pet has temperament profile with unknown, not-candidate, stale, or concerning behavior observation.
- Optional incident refs or staff notes exist as evidence ids.
- Reservation requests daycare/group play, boarding with play, or another service affected by behavior.

Policy packet:
- Required review: `BehaviorReview`; also `ManagerApproval` if there is prior incident/reinstatement/override risk.
- Allowed actions: `ReadEntities`, `CreateInternalTask`, `SuggestPlayEligibility`, `FlagRisk`, optional draft message.
- Forbidden: approve group play, reinstate group play, diagnose aggression cause, hide concerning facts, send sensitive behavior message without approval.

Expected output schema:
- `BehaviorRiskReviewOutput` with `play_eligibility_suggestion`, `evidence_refs`, `review_task`, `customer_message_draft`, and `risk_flags`.

Golden assertions:
- `approval_required: true` with `BehaviorReview`.
- Suggested play eligibility is review-gated and conservative (`NeedsStaffReview`, `IndividualCareLane`, or equivalent), not final group-play approval.
- Risk flags include behavior/safety review when evidence indicates aggression or incident risk.
- Draft customer copy avoids labels like "aggressive" unless sourced and approved; it can say staff needs to review best play/care fit.
- Safe logs exclude full behavior note and incident narrative.

### 5. Medication/special-care request

Scenario id: `medication_special_care_request`

Event payload:
- Event type: `BookingRequested`, `DailyNoteCreated`, or `BookingTriageNeeded`.
- Subject: reservation or pet.

Entity snapshots:
- Pet care profile contains medication, allergy, feeding, medical condition, or special-handling request.
- Details may be incomplete: medication name present but no dose/schedule, or customer note says "needs meds" without structured instructions.
- Reservation exists for boarding/daycare with stay date.

Policy packet:
- Required review: `ManagerApproval` or medical/care staff review vocabulary if added later; for current enum use `ManagerApproval` plus `CustomerMessageApproval` for outbound drafts.
- Allowed actions: `ReadEntities`, `ExtractStructuredData`, `CreateInternalTask`, `DraftCustomerMessage`, `FlagRisk`.
- Forbidden: infer executable medication instructions, mark medication task complete, diagnose, approve medical exception, send sensitive message without approval.

Expected output schema:
- `SpecialCareReviewOutput` with `care_requirements`, `missing_care_info`, `review_task`, `customer_clarification_draft`, and `risk_flags`.

Golden assertions:
- Missing dose/schedule/source is listed as missing information.
- Internal task routes to medication/special-care review before stay or before administration.
- Any customer draft asks for written instructions/vet/source clarification without advising treatment.
- No care task is marked executable or completed by AI.
- Safe logs redact medication names, doses, schedules, allergies, and medical conditions.

### 6. Incident report

Scenario id: `incident_report`

Event payload:
- Event type: `IncidentCreated` / `incident.created`.
- Subject: pet, reservation, or external incident until mapped.
- Related ids: incident, pet, reservation, customer, evidence/media/staff-task refs.

Entity snapshots:
- Incident record contains category, severity hint, time/place, involved parties, immediate actions, and evidence refs.
- Customer notification may be unknown, pending, or already completed.
- Pet profile/reservation may need behavior/care status update review.

Policy packet:
- Required reviews: `ManagerApproval`, `CustomerMessageApproval`, and `BehaviorReview` when play eligibility may be affected.
- Allowed actions: `ReadEntities`, `CreateInternalTask`, `FlagRisk`, optional `DraftCustomerMessage`, optional `SuggestPlayEligibility` as review-gated suggestion.
- Forbidden: close incident, assign liability, diagnose, hide facts, reinstate group play, send owner message without manager approval.

Expected output schema:
- `IncidentTriageOutput` with `incident_summary`, `missing_fields`, `severity_review`, `manager_tasks`, `owner_message_draft`, and `risk_flags`.

Golden assertions:
- Status is `NeedsHumanReview`.
- Manager/lead follow-up task is recommended.
- Missing fields include any absent immediate action/customer-notification/vet-care/witness details required by policy.
- Owner draft, if present, is explicitly draft-only and neutral.
- No incident closure, liability statement, or final medical/behavior conclusion.
- Safe logs include incident id/severity category but exclude full narrative/media/raw witness notes.

### 7. Daily note / staff care update

Scenario id: `daily_note_staff_update`

Event payload:
- Event type: `DailyNoteCreated` or derived `DailyUpdateNeeded`.
- Subject: reservation or pet.

Entity snapshots:
- Active reservation/stay exists.
- Staff care notes and optional media refs exist.
- Notes may include ordinary happy-path care evidence or sensitive medical/behavior/incident content.
- Prior daily update state may be absent, draft, approved, sent, or suppressed.

Policy packet:
- Required review: `CustomerMessageApproval`; plus `ManagerApproval`, `MedicalDocumentReview`, or `BehaviorReview` if notes include sensitive issues.
- Allowed actions: `ReadEntities`, `SummarizeCareNotes`, `CreateInternalTask`, `DraftCustomerMessage`, `FlagRisk`.
- Forbidden: auto-send Pawgress/daily update, hide concerning facts, diagnose, include unreviewed sensitive facts, mark care complete.

Expected output schema:
- `DailyUpdateDraftOutput` with `source_note_refs`, `safe_summary`, `excluded_sensitive_refs`, `draft_message`, `review_task`, and `risk_flags`.

Golden assertions:
- Ordinary notes can produce a warm draft with `CustomerMessageApproval` gate.
- Sensitive notes are either excluded with review reason or routed to manager/staff review before customer copy.
- No outbound send, no care completion, no diagnosis, and no suppression of required incident/safety facts without review.
- Safe logs exclude full staff note body and full draft customer message; include note ids and sensitivity classification.

## Validation failure fixtures

### Validation retry once: repairable malformed output

Scenario id: `validation_retry_once`

Setup:
- Use any low-risk inquiry fixture.
- Fake runtime first response is malformed or schema-invalid, e.g. missing `status`, invalid enum value, or `recommended_actions` as a string.
- Fake runtime second response is valid and policy-compliant.

Assertions:
- Runtime performs exactly two total attempts.
- Retry prompt includes validation errors and the same policy packet; it does not reveal redacted raw context in logs.
- Final result is the second valid result.
- Audit/safe log records first validation failure, retry count `1`, final validation success, and no unsafe action persisted from the first response.

### Validation retry exhausted: still invalid or unsafe

Scenario id: `validation_retry_exhausted`

Setup:
- Fake runtime first response contains forbidden action such as `confirm booking` or `send customer message`.
- Fake runtime second response still contains a forbidden action or remains schema-invalid.

Assertions:
- Runtime performs exactly one retry and then returns `FailedSafely`.
- No unsafe recommended action is persisted.
- If allowed, a review/internal task is created or recommended that names validation failure generically, e.g. "AI runtime output requires manual review".
- Safe log includes validation error codes/categories, retry count, and failed-safe status without storing the forbidden customer-message body or raw model text.

## Redaction and safe-log fixture

Scenario id: `safe_log_redaction`

Input should intentionally include:
- Owner name, phone, email, address.
- Raw customer message body.
- Medication name/dose/schedule.
- Allergy/medical-condition detail.
- Behavior/incident narrative.
- Vaccine OCR raw text.
- Payment/provider reference.

Assertions:
- Prompt packet may contain the minimal policy-needed semantic facts when required, but debug/safe-log rendering must redact or reference evidence ids instead of dumping raw sensitive content.
- Safe log includes only `scenario_id`, `event_id`, `event_type`, `workflow_name`, `location_id`, `policy_snapshot_id`, `allowed_actions`, `required_reviews`, `validation_status`, `retry_count`, `result_status`, and opaque evidence ids.
- Safe log excludes exact contact values, medication dose/schedule, raw OCR, incident narrative, payment refs, and full draft customer message.
- Structured validation errors quote field paths and error categories, not sensitive values.

## Implementation test plan

### Test 1: Fixture schema loads all scenarios

Objective: every YAML fixture conforms to the shared scenario fixture schema.

Files:
- Create: `fixtures/ai-runtime/schemas/scenario-expected.schema.json`
- Create: `fixtures/ai-runtime/scenarios/*.yaml`
- Create/modify: `domain/tests/ai_runtime_contracts.rs`

Assertions:
- Scenario ids are unique.
- Required sections exist: `input.event_payload`, `input.entity_snapshots`, `input.policy_packet`, `input.expected_output_schema`, `fake_runtime`, `expect`.
- Unknown workflow/action/review enum names fail fixture loading.

### Test 2: Prompt packet construction is deterministic

Objective: the app-owned adapter builds the same prompt packet from the same fixture every time.

Assertions:
- Stable normalized JSON snapshot for prompt packet, excluding nondeterministic timestamps if not provided by fixture.
- Packet includes policy snapshot id, allowed actions, required reviews, forbidden actions, output schema name, and entity snapshot ids.
- Packet excludes raw provider payloads and stores only evidence refs where the scenario says refs should be used.

### Test 3: Golden output validation passes for valid fixture responses

Objective: scripted fake-agent responses are validated into `WorkflowResult<T>` and checked against scenario expectations.

Assertions:
- Result status is in `allowed_statuses`.
- Required review gates and approval-required flag match expectations.
- Missing-info, recommended-actions, risk flags, and verification notes satisfy semantic contains/excludes assertions.
- Forbidden recommended actions are absent.

### Test 4: Policy violations fail closed

Objective: model output cannot bypass deterministic policy.

Assertions:
- Output containing a forbidden action, absent approval gate, unsafe status update, final vaccine approval, incident closure, group-play reinstatement, or customer send fails validation.
- Validator either repairs via retry once or returns `FailedSafely` after the retry.
- No unsafe action reaches the persistent `WorkflowResult`/audit sink.

### Test 5: Retry-once behavior is exact

Objective: validation-correctable output retries once, never zero times and never unbounded.

Assertions:
- `validation_retry_once` attempts exactly twice and succeeds with second response.
- `validation_retry_exhausted` attempts exactly twice and returns `FailedSafely`.
- Retry audit includes error categories and retry count, not raw sensitive payload values.

### Test 6: Redaction and safe logging

Objective: logs and debug strings are safe even when prompt inputs and fake outputs contain sensitive facts.

Assertions:
- Denylist values from the fixture are absent from safe logs and validation errors.
- Allowed ids/categories are present for traceability.
- Model raw output is never logged in full; if stored for forensic review later, it must be in a restricted evidence store outside ordinary app logs.

## Suggested typed helper APIs

The implementation does not need these exact names, but tests should end up with equivalent seams:

```rust
struct AiRuntimeScenario {
    scenario_id: ScenarioId,
    workflow_name: agent::Name,
    event: workflow::WorkflowEvent,
    entity_snapshots: EntitySnapshots,
    policy_packet: PolicyPacket,
    expected_output_schema: OutputSchemaExpectation,
    fake_runtime: FakeRuntimeScript,
    expect: GoldenExpectations,
}

trait AgentRuntime {
    fn run_structured(&self, packet: AgentPromptPacket<RuntimeInput>) -> RuntimeAttempt;
}

struct RuntimeHarness<R: AgentRuntime> {
    runtime: R,
    validator: WorkflowResultValidator,
    audit_sink: SafeAuditSink,
}

impl<R: AgentRuntime> RuntimeHarness<R> {
    fn run_scenario(&self, scenario: &AiRuntimeScenario) -> HarnessResult;
}
```

Core invariant: `AgentRuntime` returns untrusted output. Only `WorkflowResultValidator` can produce a trusted `WorkflowResult<T>` or a `FailedSafely` result.

## Open implementation decisions

These are implementation decisions, not blockers to writing deterministic tests:

- Whether fixtures live as YAML only, JSON only, or YAML source plus generated JSON snapshots.
- Whether scenario-specific output types live in `domain`, a future `runtime` crate, or only test DTOs until product workflows stabilize.
- Whether approval/missing-info metadata becomes part of `WorkflowResult<T>` or remains in each scenario-specific `structured_output`.
- Whether safe logs are asserted through a real logger capture, a structured `SafeAuditSink`, or both.
- Whether live Hermes integration is tested later behind ignored/smoke tests; it should not be required for these contract tests.

## Acceptance checklist

The implementation card is ready when tests can prove:

- All seven required business scenarios have fixtures with event payload, entity snapshots, policy packet, expected output schema, fake runtime response, and golden assertions.
- Every scenario asserts no forbidden action and correct approval-required/review-gate behavior.
- Missing-info and safe draft behavior are asserted for inquiry, vaccine, booking, medication/care, incident, and daily-update cases.
- Validation failure and retry-once behavior are covered by dedicated fixtures.
- Redaction/safe-log assertions cover PII, medical/care, behavior/incident, vaccine OCR, payment refs, raw provider payloads, and draft message bodies.
- No test requires network access, live Hermes, live Gingr, or a real LLM.
