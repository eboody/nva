# Incident intake input form

Purpose: define the staff-facing intake form for incident reports that feed the Incident/Escalation Agent. This form captures source facts, evidence references, and unresolved questions. It does not authorize diagnosis, owner-facing sends, incident closure, eligibility changes, provider writes, or restriction clearance.

Status: draft workflow definition. Requiredness below is intentionally conservative until location-approved incident SOPs, severity mappings, attachment retention rules, and owner-notice SLAs are finalized.

## Intake principles

1. Capture observations, not diagnosis.
   - Staff may record what they saw, heard, smelled, measured, or were told by a named source.
   - Staff must not enter medical diagnoses, treatment instructions, blame, liability statements, or unsupported intent.
2. Preserve raw facts separately from customer-safe wording.
   - Internal narrative, observed signs, and witness notes are not automatically owner-facing copy.
   - Owner-message drafts are later review artifacts and require human approval.
3. Prefer explicit unknowns over guessed details.
   - If a required field is unknown, staff must mark `Unknown` and explain what follow-up is needed.
   - Missing required details create follow-up prompts/tasks; the AI must not fabricate them.
4. Store media by reference.
   - Photos, video, scanned notes, and documents should be uploaded to approved storage and represented as attachment references with metadata.
   - Do not paste raw blobs or broad media histories into runtime context.
5. Keep an audit trail.
   - Every submission, edit, attachment upload, status change, notification decision, and human approval should be attributable to actor, timestamp, source, and reason.

## Severity/type vocabulary for form behavior

The form supports staff triage using these provisional severities. The AI may suggest a candidate severity for review later, but it must not finalize medium/high/emergency classifications or downgrade serious reports.

| Severity | Meaning for intake | Human review posture |
| --- | --- | --- |
| `Low` | Internal note or minor operational issue with no injury, health ambiguity, behavior escalation, owner-notice need, restriction, or policy-sensitive concern. | May still need staff completion. Owner-facing draft, if any, requires approval. |
| `Medium` | Owner notice likely, manager/lead review likely, repeated behavior/care issue, minor injury/health observation, unresolved staff action, or unclear facts. | Manager/lead review required before final classification, owner copy, or closure. |
| `High` | Bite/aggression hard stop, injury/health concern, medication/care deviation, escape/near-miss, suspension/restriction, legal/liability-sensitive issue, or customer complaint connected to safety/care. | Manager approval required. Restrictions and review gates remain active until resolved. |
| `Emergency` | Active safety, severe injury/illness signs, vet/emergency escalation, heat stress, allergic reaction concern, missing/escaped pet, or any urgent condition requiring immediate human response. | Immediate human escalation required. AI may only summarize, flag, and prompt for missing facts. |

Incident type values should be multi-select because one incident can span categories:

- Behavior/safety: bite, attempted bite, fight, aggressive display, mounting, guarding, chase escalation, barrier reactivity, human selectivity, stress/inability to settle.
- Health/injury: wound, limp/lameness, vomiting/diarrhea, allergic reaction concern, heat/cold stress, abnormal breathing, collapse, seizure-like activity, pain response, other observed health sign.
- Care-plan deviation: missed medication, wrong/late feeding, allergy/food exposure, care-instruction deviation, document/vaccine ambiguity.
- Facility/supervision: escape attempt, gate/door failure, ratio/capacity concern, sanitation/cleaning hazard, equipment/facility hazard, incompatible playgroup mix.
- Owner/customer-reported: owner reports issue at pickup or after pickup, complaint related to care/safety/health/behavior.
- Other reviewed incident: anything staff believes needs documented review.

## Requiredness matrix

Legend:

- `R`: required before the intake can be submitted as a complete report for this severity/type.
- `C`: conditionally required when the condition applies.
- `O`: optional but useful.
- `F`: required follow-up if unknown; report may submit as incomplete only if staff marks unknown and explains follow-up.

| Section / field | Low | Medium | High | Emergency | Conditional rule |
| --- | --- | --- | --- | --- | --- |
| Reporter/staff identity | R | R | R | R | Use authenticated staff actor when possible. |
| Source of report | R | R | R | R | Staff-observed, manager, provider import, owner/customer, witness, system/import. |
| Operating location | R | R | R | R | Required for routing, audit, policy snapshot, and manager review. |
| Operating day / shift | R | R | R | R | Needed for shift handoff and daily brief. |
| Observed-at date/time | R | R | R | R | If exact time unknown, capture approximate time plus uncertainty. |
| Submitted-at timestamp | R | R | R | R | System-generated. |
| Where it happened | R | R | R | R | Area/room/yard/kennel/vehicle/front desk/etc.; if unknown, mark unknown. |
| Incident type/category | R | R | R | R | Multi-select; choose best available type and `other reviewed` when unsure. |
| Candidate severity | R | R | R | R | Staff triage value; AI/human review may change it. |
| What happened - factual summary | R | R | R | R | Observed/source-grounded narrative only. |
| Pets involved | C | R | R | R | Required for any pet-specific incident. Include role and uncertainty. |
| Owner/customer involved | O | C | C | C | Required when owner/customer reported, witnessed, must be notified, or contact is needed. |
| Reservation/attendance/stay reference | C | R | R | R | Required when incident occurred during a service/stay or affects active booking. |
| Staff involved | C | R | R | R | Required when staff observed, intervened, had assigned care, or are witnesses. |
| Immediate action taken | F | R | R | R | Required for medium/high/emergency and any injury, behavior, care, escape, or owner-notice incident. |
| Current pet/care state | O | R | R | R | Where pet is now, care mode/rest/isolation, pickup/vet status, active restriction. |
| Observed signs/symptoms/behavior | C | R | R | R | Required for health, injury, behavior, stress, safety, and emergency categories. No diagnosis. |
| Injury/body location details | O | C | R | R | Required when injury/health concern is present. Describe visible facts only. |
| Care-plan/medication/allergy details | O | C | R | R | Required for missed medication, feeding/allergy exposure, or care-instruction deviation. |
| Facility/equipment details | O | C | R | R | Required for facility/supervision/escape/sanitation hazards. |
| Photos/attachments | O | C | C | C | Required-by-policy unknown; strongly recommended for injury, property/facility issue, medication/care proof, owner report, and serious behavior. If absent for high/emergency, explain why. |
| Witnesses | O | C | C | C | Required when another staff/customer/vendor saw or reported material facts. If none/unknown, state that. |
| Owner notification status | O | R | R | R | Required for owner-notice-likely, injury/health, care deviation, behavior/safety, high/emergency, or customer-reported incidents. |
| Manager notification status | O | R | R | R | Required for medium/high/emergency, incomplete required facts, active restrictions, owner-message need, and policy-sensitive incidents. |
| Follow-up needs | C | R | R | R | Required when anything remains unresolved, unknown, review-gated, task-worthy, or time-sensitive. |
| Active restrictions / temporary flags | O | C | R | R | Required when group play, kennel/rest lane, handling, feeding, medication, pickup/vet, or eligibility may change. |
| Requested owner-message draft? | O | O | O | O | Draft only; never approval or send. Must link to `CustomerMessageApproval`. |
| Source/evidence refs | C | R | R | R | Required for imported/provider reports, owner reports, photos, staff notes, and edits. |

## Form sections and fields

### 1. Report header

- `incident_intake_id` (system-generated, required)
  - Stable intake/report identifier used by audit, attachment refs, tasks, and manager review.
- `reporter_actor` (required)
  - Authenticated staff/manager/system/customer actor reference when available.
- `reporter_display_name` (required when actor id unavailable)
  - Free-text fallback for witness/customer/imported reports.
- `report_source` (required)
  - Enum: `staff_observed`, `manager_reported`, `lead_reported`, `owner_reported`, `customer_after_pickup`, `provider_import`, `system_detected`, `witness_reported`, `other`.
- `source_record_refs` (conditional)
  - Provider webhook id, staff note id, task id, owner message id, uploaded document id, or audit/source reference.
- `location_id` / `location_name` (required)
- `operating_day` (required)
- `shift_or_service_window` (required when available)
- `submitted_at` (system-generated, required)
- `last_edited_at` and `last_edited_by` (system-generated on edit)

### 2. What happened, when, where

- `observed_at` (required)
  - Exact timestamp if known.
- `observed_at_precision` (required)
  - Enum: `exact`, `approximate`, `unknown_time_known_day`, `unknown`.
- `observed_at_note` (conditional)
  - Required when precision is not exact.
- `where_happened` (required)
  - Area, room, yard, kennel, lobby, grooming, vehicle, playgroup, etc.
- `service_context` (conditional)
  - Boarding, day play, day boarding, grooming, training, day spa, pickup/dropoff, after-pickup report, other.
- `reservation_or_attendance_refs` (required for active service incidents)
- `incident_types` (required, multi-select)
- `candidate_severity` (required)
- `what_happened_summary` (required)
  - Staff-facing factual summary. Separate chronology from interpretation. Avoid diagnosis/blame.
- `timeline_entries` (optional for low; required for high/emergency or complex events)
  - Repeating entries: time/approximation, actor/source, observed fact, action taken, evidence ref.

### 3. Pets involved

Required for any pet-specific incident and for all medium/high/emergency reports.

For each pet:

- `pet_id` / `pet_name` (required when known)
- `customer_id` / owner contact reference (conditional)
- `reservation_id` / attendance reference (conditional)
- `role_in_incident` (required)
  - Enum: `primary_subject`, `other_pet_involved`, `injured_or_observed_signs`, `reported_aggressor`, `reported_recipient`, `bystander_affected`, `unknown_role`.
- `role_uncertainty_note` (conditional when role is uncertain or source-reported)
- `current_location_or_care_mode` (required for medium/high/emergency)
  - Example: with staff, rest lane, individual care, owner picked up, vet/emergency, unknown.
- `active_restriction_candidate` (conditional)
  - Group play hold, handling caution, feeding/allergy review, medical/care review, pickup/vet follow-up, other. Candidate only until approved.

### 4. Staff involved

Required when staff observed, intervened, were assigned care, received the report, or are witnesses. Required for all medium/high/emergency reports unless truly unknown.

For each staff member:

- `staff_id` / `staff_name` (required when known)
- `role_in_incident` (required)
  - Reporter, observer, intervened, assigned caregiver, manager/lead notified, witness, other.
- `action_or_observation_summary` (conditional)
- `needs_statement_or_follow_up` (optional for low; conditional for medium/high/emergency)

### 5. Immediate action taken

Required for medium/high/emergency and for any behavior/safety, injury/health, care-plan, owner-notice, or facility/supervision incident. Low reports may submit incomplete only if `no_immediate_action_needed` or `unknown_follow_up_required` is selected.

- `immediate_action_taken` (required/conditional per matrix)
  - Examples: separated pets, moved pet to rest lane, cleaned area, checked pet visually, notified lead, called manager, owner pickup requested, vet/emergency escalation started, medication/care correction initiated, facility hazard isolated.
- `action_taken_by` (conditional)
- `action_taken_at` (conditional)
- `current_status_after_action` (required for medium/high/emergency)
- `open_safety_or_care_concern` (required)
  - Boolean plus details when true.
- `urgent_human_attention_needed` (required for high/emergency)
  - Boolean; if true, route immediately to manager/lead workflow.

### 6. Observed signs, symptoms, and behavior

Required for health/injury/behavior/safety/emergency categories. Staff should describe observations without diagnosis or treatment advice.

- `observed_behavior` (conditional)
  - Controlled vocabulary plus notes: growling, snap, bite, attempted bite, stiff posture, guarding, chase escalation, mounting, barrier reactivity, hiding, shaking, stress panting, inability to settle, other observed behavior.
- `observed_health_or_injury_signs` (conditional)
  - Visible wound, bleeding, swelling, lameness/limp, vomiting, diarrhea, coughing, abnormal breathing, heat stress signs, allergic-reaction concern, lethargy, pain response, seizure-like activity, other observed sign.
- `body_location_observed` (conditional for visible injury/signs)
- `duration_or_frequency` (conditional when known)
- `severity_observation_note` (conditional for medium/high/emergency)
- `staff_measurements` (optional/conditional)
  - Only objective values actually taken under approved facility practice; do not invent or infer.
- `not_diagnosis_acknowledged` (required checkbox when health/injury fields are used)
  - Text: "I am recording observed signs only, not diagnosing or giving medical advice."

### 7. Photos and attachments

Attachments are optional for low incidents unless required by local policy, but conditionally expected for injury/health observations, facility/equipment hazards, care-plan deviations, owner/customer reports, serious behavior events, and high/emergency incidents. If a high/emergency report has no attachments, staff must explain why.

For each attachment:

- `attachment_id` / storage reference (system-generated, required)
- `attachment_type` (required)
  - Photo, video, scanned document, owner message screenshot, provider record, staff note, external report, other.
- `captured_at` (conditional)
- `uploaded_at` and `uploaded_by` (system-generated)
- `subject_refs` (conditional)
  - Pet, incident area, facility/equipment, document, message, or other subject.
- `description` (required)
  - Factual description of what the attachment shows; no diagnosis/blame.
- `sensitivity` (required)
  - Internal-only, manager-review, customer-share-review-needed, legal/privacy-sensitive.
- `redaction_needed` (required)
- `owner_share_allowed` (required default `unknown_requires_review`)
  - No attachment is owner-shareable merely because it was uploaded.
- `retention_policy_ref` (conditional until policy exists)
  - Use location/media-retention policy id when available; otherwise mark unknown.

Attachment handling expectations:

- Store media in approved object/document storage; incident records carry references and metadata only.
- Restrict runtime prompts to scoped metadata and approved derived observations unless a future data-sent-to-runtime gate explicitly permits raw media.
- Preserve original upload metadata and do not overwrite originals; edits/redactions create derivative refs.
- Log access, download, redaction, derivative creation, deletion/retention decisions, and any owner-share approval.
- Owner-facing message drafts may mention that photos/attachments exist only if staff/manager chooses to include that fact; sending media requires separate approval.

### 8. Witnesses

Required when a material witness exists or when the incident source is witness/customer-reported. If no witnesses are known, staff should explicitly select `no_known_witnesses` for medium/high/emergency reports.

For each witness:

- `witness_type` (required)
  - Staff, manager/lead, customer/owner, vendor, visitor, unknown.
- `witness_actor_ref` or `witness_name/contact_ref` (conditional)
- `witness_statement_summary` (conditional)
  - What they reported seeing/hearing; separate from staff-observed facts.
- `statement_record_ref` (optional/conditional)
- `needs_follow_up` (conditional)

### 9. Owner notification status

Required for medium/high/emergency, owner-notice-likely, injury/health, behavior/safety, care deviation, customer-reported, and any incident where owner contact may be required. This section records status; it does not send messages.

- `owner_notification_status` (required/conditional)
  - Enum: `not_needed_candidate`, `not_started`, `draft_needed`, `draft_created_pending_review`, `manager_review_required_before_owner_notice`, `owner_notified_by_staff`, `owner_notified_by_manager`, `unable_to_reach_owner`, `unknown`.
- `owner_notification_required_reason` (conditional)
- `owner_contact_refs` (conditional)
- `owner_notification_time` (conditional if already notified)
- `owner_notification_actor` (conditional if already notified)
- `owner_notification_channel` (conditional if already notified)
  - Phone, in-person, email, SMS, portal/app, other.
- `owner_message_draft_requested` (optional)
- `owner_message_draft_constraints` (conditional if draft requested)
  - Facts to include, facts needing manager review, unknowns, tone constraints, legal/medical caution.
- `customer_message_approval_gate` (system-required when draft requested)
  - Must be `CustomerMessageApproval`; manager approval also required for sensitive/severe cases.

### 10. Manager notification status

Required for medium/high/emergency, incomplete required facts, active restrictions, owner-message need, policy-sensitive incidents, medical/care ambiguity, behavior/eligibility impact, facility/safety concerns, and any staff uncertainty.

- `manager_notification_status` (required/conditional)
  - Enum: `not_needed_candidate`, `not_started`, `lead_notified`, `manager_notified`, `manager_review_requested`, `manager_review_in_progress`, `manager_approved_action`, `unable_to_reach_manager`, `unknown`.
- `manager_or_lead_ref` (conditional)
- `manager_notification_time` (conditional)
- `manager_notification_channel` (conditional)
- `manager_review_reason` (conditional)
- `approval_or_decision_refs` (conditional)
  - Approval records, task ids, audit ids; not free-text claims alone.

### 11. Follow-up needs

Required for medium/high/emergency and whenever facts, notifications, attachments, witness statements, owner draft, restrictions, or closure remain unresolved.

- `follow_up_needed` (required)
- `follow_up_items` (conditional)
  - Repeating items: type, description, owner role/person, due time, priority, related pet/incident/attachment/source refs, review gate.
- `recommended_task_kinds` (optional/conditional)
  - Incident follow-up, document review, playgroup assessment, daily update draft, customer follow-up, medication/care review, facility/safety check, manager review.
- `blocking_unknowns` (conditional)
- `active_review_gates` (conditional)
  - ManagerApproval, MedicalDocumentReview, BehaviorReview, CustomerMessageApproval, plus proposed incident-specific gates if later approved.
- `closure_blockers` (conditional)
  - Missing required facts, unresolved owner notice, unresolved manager review, active restriction, open staff task, unresolved medical/care ambiguity, attachment/witness follow-up.

### 12. Staff attestations

- `facts_are_source_grounded` (required checkbox)
  - "I have recorded observed or reported facts and marked unknowns instead of guessing."
- `no_diagnosis_or_liability_statement` (required checkbox for health/injury/legal-sensitive reports)
  - "I have not entered a diagnosis, treatment instruction, blame assignment, or liability/legal conclusion."
- `urgent_path_used_if_needed` (required for emergency candidate)
  - "If immediate human response was needed, I used the approved emergency/manager escalation path outside this AI form."
- `submission_notes` (optional)

## AI validation and missing-field prompts

The AI may return staff prompts to complete the intake. Prompts must be specific, neutral, and limited to missing or inconsistent fields. The AI must not pressure staff to soften facts or choose a lower severity.

### General missing required fields

- "I can save this as an incomplete incident report, but I still need the observed date/time or an `unknown time` note before it can be reviewed. What time did this happen, or should I mark the time as unknown and ask for follow-up?"
- "Please add where the incident happened, such as yard, kennel, lobby, grooming, vehicle, or `unknown`."
- "Please select at least one incident type so the report can route correctly. If none fit, choose `other reviewed incident` and describe it."
- "Please add a factual summary of what happened. Use what was observed or reported; avoid diagnosis, blame, or guesses."

### Pet and reservation prompts

- "Which pet or pets were involved? If a pet is unknown, add `unknown pet` and describe what is known so staff can follow up."
- "This incident is marked medium/high/emergency, so each involved pet needs a current care state: with staff, rest lane, individual care, owner picked up, vet/emergency, or unknown."
- "Please link the reservation/attendance record for the active service, or mark that no reservation reference is available yet."
- "The pet's role is unclear. Was this pet the primary subject, injured/observed, another pet involved, a bystander affected, or is the role unknown?"

### Staff and witness prompts

- "Who observed or received the report? Add the staff member, manager, witness, or mark the source as unknown."
- "You listed a witness but no witness summary. Please add what the witness reported, or mark `needs follow-up`."
- "For a high/emergency incident, please identify who took immediate action or mark that action-taker is unknown and needs follow-up."

### Immediate action prompts

- "This incident type requires immediate-action documentation. What was done right away, by whom, and what is the pet/current area status now?"
- "You selected an emergency candidate. Confirm whether the approved urgent human escalation path has been used outside this AI form."
- "There is an open safety/care concern but no follow-up item. Please add a follow-up task or mark the concern resolved with source evidence."

### Observed signs/behavior prompts

- "Please describe observed signs or behavior without diagnosis. For example: `limping on rear left leg`, `small visible scrape`, `vomited once`, `growled and snapped`, or `panting heavily`."
- "You selected injury/health concern. Please add body location if visible, or mark body location unknown."
- "You selected care-plan/medication/allergy issue. Please add the care instruction involved, what happened, and who needs to review it."
- "The note sounds like a diagnosis or treatment instruction. Please rewrite it as observed facts only, or move the medical decision to manager/vet follow-up."

### Attachment prompts

- "For this high/emergency or injury/facility incident, are there photos or attachments? Upload them by reference, or explain why none are available."
- "Please add a factual description for each attachment and mark whether redaction or manager review is needed."
- "Owner sharing for attachments defaults to `unknown_requires_review`. Do not mark media as shareable unless an approved policy/manager decision says so."

### Notification prompts

- "Owner notification status is required for this incident. Choose not started, draft needed, pending manager review, owner notified, unable to reach owner, or unknown."
- "Manager notification status is required because this report is medium/high/emergency or has unresolved required facts. Choose the current status and add a manager/lead reference if known."
- "You requested an owner-message draft. The draft will require CustomerMessageApproval and may require ManagerApproval; please add any facts that must be reviewed before owner-facing wording is drafted."

### Follow-up and closure prompts

- "This report still has required unknowns. Add follow-up needs for each unknown, or mark why no follow-up is needed."
- "An active restriction or review gate is present. The incident cannot be closed until the gate is resolved with approval/audit evidence."
- "There is owner notice pending but no follow-up task. Add an owner-notification follow-up or explain why owner notice is not needed."

## Audit trail expectations

The intake form must produce or preserve audit events for:

- Initial incident intake creation.
- Each staff edit, including changed fields, editor, timestamp, and reason when supplied.
- Attachment upload, replacement, redaction, derivative creation, access/download when required, and retention/deletion decisions.
- Source/provider imports and semantic mapping from untrusted provider payloads into typed fields.
- AI validation prompts and staff responses to missing-field prompts.
- AI-generated summaries, candidate classifications, owner-message drafts, and manager review packets.
- Human approvals/rejections for owner messages, medium/high/emergency classifications, restrictions, provider writes, and closure.
- Owner notification proof when performed by staff/manager: actor, channel, time, source/proof reference, and message/draft approval reference.
- Manager notification proof and review decisions.
- Follow-up task creation, assignment, status changes, completion evidence, and closure blockers.

Audit records should distinguish:

- Source facts vs AI summaries.
- Staff-entered observations vs owner/witness/provider-reported claims.
- Drafts/recommendations vs approved actions.
- Attachment references vs raw media.
- Unknown/missing facts vs completed fields.

## Downstream support requirements

### Manager review

The form supports manager review by collecting:

- Severity/type candidate and evidence.
- Current pet/care state and active restrictions.
- Immediate action, unresolved safety/care concerns, and urgent flags.
- Staff/witness/source refs.
- Attachments and sensitivity markers.
- Owner/manager notification status.
- Follow-up items, review gates, and closure blockers.

Manager review packets should show unknowns and contradictions explicitly rather than presenting the report as complete.

### Owner-message draft review

The form supports owner-message drafting by preserving:

- Customer-safe candidates separate from raw/internal narrative.
- Source-grounded observed facts and uncertainty.
- Explicit no-diagnosis/no-liability boundaries.
- Notification status, channel context, and owner contact refs.
- Attachment share status and redaction requirements.
- Required approval gates before any send.

Owner-facing drafts must never include staff-only notes, unsupported blame, diagnosis/treatment advice, legal conclusions, or raw media unless separately approved.

### Post-incident review

The form supports post-incident review by preserving:

- Timeline entries, actions taken, and care/status changes.
- Involved pets, staff, witnesses, reservations, and location/shift.
- Incident type/severity evolution and human decisions.
- Restrictions, follow-up tasks, and closure blockers.
- Attachment/evidence refs and audit history.
- Owner and manager notification proof.
- Open questions and lessons learned fields for later review.

Post-incident review may recommend process improvements or policy updates, but closure, restriction clearance, eligibility changes, provider writes, and owner-facing conclusions remain approval-gated.

## Minimal structured payload sketch

This sketch is illustrative; implementation should map to canonical domain types and storage adapters when those exist.

```yaml
incident_intake:
  incident_intake_id: string
  source:
    reporter_actor_ref: string | null
    reporter_display_name: string | null
    report_source: enum
    source_record_refs: [string]
  context:
    location_id: string
    operating_day: date
    shift_or_service_window: string | null
    submitted_at: datetime
    observed_at: datetime | null
    observed_at_precision: enum
    observed_at_note: string | null
    where_happened: string
    service_context: enum | null
    reservation_or_attendance_refs: [string]
  classification_candidate:
    incident_types: [enum]
    candidate_severity: low | medium | high | emergency
    active_review_gates: [enum]
  narrative:
    what_happened_summary: string
    timeline_entries:
      - time: datetime | null
        time_precision: enum
        actor_or_source_ref: string | null
        observed_fact: string
        action_taken: string | null
        evidence_refs: [string]
  involved_pets:
    - pet_id: string | null
      pet_name: string | null
      customer_id: string | null
      reservation_id: string | null
      role_in_incident: enum
      role_uncertainty_note: string | null
      current_location_or_care_mode: string | null
      active_restriction_candidate: string | null
  involved_staff:
    - staff_id: string | null
      staff_name: string | null
      role_in_incident: enum
      action_or_observation_summary: string | null
      needs_statement_or_follow_up: boolean
  immediate_action:
    summary: string | null
    action_taken_by: string | null
    action_taken_at: datetime | null
    current_status_after_action: string | null
    open_safety_or_care_concern: boolean
    urgent_human_attention_needed: boolean
  observations:
    observed_behavior: [enum]
    observed_health_or_injury_signs: [enum]
    body_location_observed: string | null
    duration_or_frequency: string | null
    objective_measurements: [string]
    no_diagnosis_acknowledged: boolean
  attachments:
    - attachment_id: string
      attachment_type: enum
      captured_at: datetime | null
      uploaded_at: datetime
      uploaded_by: string
      subject_refs: [string]
      description: string
      sensitivity: enum
      redaction_needed: boolean
      owner_share_allowed: unknown_requires_review | no | approved_ref
      retention_policy_ref: string | null
  witnesses:
    - witness_type: enum
      witness_ref_or_name: string | null
      witness_statement_summary: string | null
      statement_record_ref: string | null
      needs_follow_up: boolean
  notifications:
    owner_notification_status: enum
    owner_notification_refs: [string]
    owner_message_draft_requested: boolean
    manager_notification_status: enum
    manager_notification_refs: [string]
  follow_up:
    follow_up_needed: boolean
    follow_up_items: [string]
    blocking_unknowns: [string]
    closure_blockers: [string]
  attestations:
    facts_are_source_grounded: boolean
    no_diagnosis_or_liability_statement: boolean
    urgent_path_used_if_needed: boolean | null
```

## Non-goals and hard boundaries

- The form does not decide final severity for medium/high/emergency incidents.
- The form does not send owner messages or approve owner-message drafts.
- The form does not diagnose, recommend treatment, assign liability, or suppress material facts.
- The form does not close incidents, clear restrictions, reinstate group play, or complete staff tasks without approved human evidence.
- The form does not define final legal/media retention policy; it records policy refs or unknowns for review.
