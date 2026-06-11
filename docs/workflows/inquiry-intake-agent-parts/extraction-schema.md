# Inquiry intake extraction schema

Purpose: define the workflow-specific structured output schema for the `inquiry-intake` agent. This artifact is intended to feed the final `docs/workflows/inquiry-intake-agent.md` workflow definition. It does not authorize live booking, availability promises, customer sends, provider writes, vaccine approval, payment action, or policy exceptions.

Status: draft schema contract. It is aligned to the current `WorkflowEventType::InquiryReceived`, `AgentPromptPacket<T>`, and `WorkflowResult<T>` anchors, plus the proposed result-envelope conventions in `docs/architecture/workflow-result-envelope.md` and `docs/architecture/pet-resort-ai-runtime-structured-output.md`.

## Source anchors

Use these repo-local sources as the canonical constraints for this schema:

- `docs/workflows/inquiry-intake-inputs.md` — product scope, event/channel boundaries, customer-message constraints, idempotency, missing assumptions, and current Rust anchors.
- `docs/architecture/workflow-result-envelope.md` — result envelope semantics, status mapping, draft message/task behavior, verification, risk flags, and human-review invariants.
- `docs/architecture/pet-resort-ai-runtime-structured-output.md` — schema validation, parse/retry/fail-safe behavior, required source refs, uncertainty/missing-input conventions, and logging minimization.
- `docs/architecture/pet-resort-data-model.md` — canonical Customer/Pet/Reservation/Task/Message/Audit boundaries, AI output handling, multi-pet open question, and approval gates.
- `domain/src/entities.rs` — `Customer`, `Pet`, `Reservation`, `ContactChannel`, `ReservationSource`, `ServiceKind`, `ReservationStatus`, `HardStop`, `ActorRef`, and audit subjects/actions.
- `domain/src/workflow.rs` — `WorkflowEventType::InquiryReceived`, `WorkflowSubject`, `PolicyContext`, allowed actions, `WorkflowResult<T>`, `WorkflowStatus`, and `RecommendedAction`.
- `domain/src/agents.rs` — baseline `inquiry-intake` agent purpose, allowed tools, forbidden actions, and `CustomerMessageApproval` default gate.

Known source gaps to preserve explicitly:

- No final product-map artifact exists at `docs/product/pet-resort-product-map.md`.
- No dedicated Inquiry/Lead aggregate exists yet; the current choices are external lead subject, customer subject, and/or reservation in `Inquiry`/`MissingInfo` status.
- Chat widget is not represented in current `ReservationSource`, `ContactChannel`, or `DeliveryChannel` enums.
- Consent/opt-out/quiet-hours are named as required policy inputs but not fully modeled.
- Website form, email, SMS, phone, chat transcript, and source-message metadata schemas are not implemented.

## Envelope placement

The agent output should be a `WorkflowResult<InquiryIntakeExtraction>` style packet.

Minimum envelope expectations:

```json
{
  "schema_name": "InquiryIntakeExtraction",
  "schema_version": "2026-06-11",
  "workflow_name": "inquiry-intake",
  "event_id": "workflow_event_id",
  "subject": { "type": "customer_or_external", "id": "..." },
  "status": "completed | needs_human_review | needs_more_information | failed_safely | rejected_by_policy",
  "summary": "Operator-safe intake summary.",
  "structured_output": { "...": "InquiryIntakeExtraction" },
  "recommended_actions": [],
  "risk_flags": [],
  "verification": [],
  "human_review_reason": null
}
```

During migration to the proposed docs envelope, map current Rust status values as:

- `WorkflowStatus::Completed` -> `success` only for a safe extraction/review packet, not booking execution.
- `WorkflowStatus::NeedsMoreInformation` -> `blocked` / missing-info follow-up needed.
- `WorkflowStatus::NeedsHumanReview` -> review packet requires a named gate.
- `WorkflowStatus::RejectedByPolicy` -> no action or review depending on customer impact.
- `WorkflowStatus::FailedSafely` -> failed, no side effects.

## Workflow-specific structured output

Recommended top-level object:

```json
{
  "intake_id": "inq_... or null",
  "location_id": "loc_...",
  "source": {},
  "owner": {},
  "contact": {},
  "pets": [],
  "requested_services": [],
  "requested_dates": [],
  "special_needs": [],
  "temperament": [],
  "vaccine_status": {},
  "urgency": {},
  "missing_info": [],
  "ambiguities": [],
  "validation": {},
  "record_mapping": {},
  "source_refs": []
}
```

All extracted fact fields that can affect routing, tasks, drafts, record creation, reservation status, eligibility, or customer-facing copy must use the field-evidence wrapper below rather than a bare value.

### Field evidence wrapper

Use this shape for scalar facts, normalized enum candidates, and source-derived values:

```json
{
  "value": "normalized value or null",
  "state": "known | unknown | ambiguous | conflicting | not_applicable | redacted",
  "confidence": "high | medium | low | none",
  "evidence": [
    {
      "source_ref": "source_message:msg_123:excerpt_2",
      "span": "char:42-68 or line:3-4 or field:owner_name",
      "quote": "redacted minimal excerpt if policy permits",
      "source_trust_state": "customer_reported | staff_entered | provider_verified | provider_unverified | system_derived | unknown"
    }
  ],
  "notes": "short operator note, no unnecessary PII"
}
```

Rules:

- `confidence` is extraction confidence only; it is never approval authority.
- `state=known` requires at least one evidence item or a deterministic derived source ref.
- `state=unknown` means the field was absent from checked sources, not merely omitted by the model.
- `state=ambiguous` means a value may exist but needs interpretation, e.g. “next weekend” without timezone/date anchor.
- `state=conflicting` means multiple source-backed candidates disagree and downstream code must not choose silently.
- `quote` must be omitted or redacted when it would expose unnecessary PII, full raw messages, medical details beyond the review audience, payment data, raw provider payloads, secrets, or webhook material.

## Required fields

### 1. Source

`source` preserves where the inquiry came from and how much trust the application can place in it.

```json
{
  "source_kind": "website_form | sms | email | phone_transcript | chat_widget | portal | provider_lead | staff_created | unknown",
  "reservation_source_mapping": "WebsiteForm | Sms | Email | PhoneTranscript | Portal(Gingr) | StaffCreated | unmapped_chat_widget | unknown",
  "source_message_id": "msg_123 or null",
  "provider_ref": { "provider": "gingr", "external_id": "lead_123" },
  "submitted_at": { "value": "2026-06-11T13:00:00Z", "state": "known", "confidence": "high", "evidence": [] },
  "actor_kind": "customer | staff | system | provider | unknown",
  "verification_state": "verified_event | normalized_import | staff_entered | unverified_external | unknown"
}
```

Validation and mapping:

- `source_kind=chat_widget` is allowed in the schema but maps to `reservation_source_mapping=unmapped_chat_widget` until the domain model adds a typed chat source or explicitly maps it.
- Provider lead/webhook evidence can be used only after signature verification and semantic normalization; otherwise set `verification_state=unverified_external` and flag `provider_payload_unverified`.
- Preserve source ids for idempotency: `location + InquiryReceived + customer/external lead id + source_kind + lead/provider event id or normalized contact+submitted_at hash`.

### 2. Owner

Owner is the human/household making the inquiry. It may map to an existing `Customer` or remain an external lead candidate.

```json
{
  "customer_id": "cust_... or null",
  "external_lead_id": "lead_... or null",
  "full_name": { "value": "Jordan Lee", "state": "known", "confidence": "high", "evidence": [] },
  "household_or_co_owner": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] },
  "identity_match": {
    "state": "new_lead | matched_customer | possible_match | conflicting_match | unknown",
    "matched_customer_ids": [],
    "reason": "Email matched existing customer, or name/phone fuzzy only."
  }
}
```

Rules:

- Do not create or merge customer records solely from name similarity.
- Conflicting owner/contact/profile facts create a review item rather than overwriting canonical `Customer` fields.
- Customer name can be used in a draft only when source-backed and safe for the selected review audience.

### 3. Contact

Contact is the normalized way staff can follow up, not proof of legal send consent.

```json
{
  "email": { "value": "jordan@example.com", "state": "known", "confidence": "high", "evidence": [] },
  "phone": { "value": "+15555550123", "state": "known", "confidence": "medium", "evidence": [] },
  "preferred_channel": { "value": "email", "state": "known", "confidence": "medium", "evidence": [] },
  "contact_channel_mapping": "Email | Sms | Phone | Portal | unknown",
  "consent_state": "known_allowed | known_denied | unknown | not_modeled",
  "quiet_hours_or_suppression_state": "clear | suppressed | unknown | not_modeled"
}
```

Rules:

- At least one contact route is required to complete an inquiry follow-up packet; otherwise include `missing_info.field=contact_method`.
- `preferred_channel` maps to current `ContactChannel` only for preference. It is not legal authorization to send.
- SMS/phone/email availability must not bypass consent, opt-out, quiet-hours, suppression, or customer-message approval gates.

### 4. Pets

`pets` is an array because inquiries often mention multiple pets. Each pet has its own evidence, missing fields, risk flags, and mapping state.

```json
{
  "pet_slot_id": "pet_slot_1",
  "pet_id": "pet_... or null",
  "name": { "value": "Luna", "state": "known", "confidence": "high", "evidence": [] },
  "species": { "value": "dog", "state": "known", "confidence": "high", "evidence": [] },
  "breed": { "value": "goldendoodle", "state": "known", "confidence": "medium", "evidence": [] },
  "size": {
    "reported_weight": { "value": "42 lb", "state": "known", "confidence": "medium", "evidence": [] },
    "size_class": { "value": "medium", "state": "ambiguous", "confidence": "low", "evidence": [] },
    "derivation": "reported_by_owner | derived_from_weight | unknown"
  },
  "age_or_birth_date": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] },
  "sex": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] },
  "spay_neuter_status": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] },
  "mapping_state": "new_pet | matched_pet | possible_match | conflicting_match | unknown",
  "per_pet_missing_info": [],
  "per_pet_risk_flags": []
}
```

Rules:

- `species` accepted values should align with domain `Species::{Dog, Cat, Other}` plus a schema-level `unknown` state. Do not coerce unknown species to dog.
- Breed is descriptive unless a later policy requires breed review. If breed affects policy, route to review rather than branch on free text.
- Size can be reported text and/or derived class. Derived size must name its basis; do not derive from breed alone unless a policy explicitly permits it.
- For multi-pet inquiries, preserve per-pet service/date/special-need associations when the source provides them. If the message says “Luna and Max” but only one service/date applies ambiguously, mark the association ambiguous instead of duplicating confidently.

### 5. Requested service

Use an array so one inquiry can ask about boarding plus grooming, daycare trial plus training, etc.

```json
{
  "service_slot_id": "svc_1",
  "service_kind": { "value": "boarding", "state": "known", "confidence": "high", "evidence": [] },
  "service_mapping": "Boarding | DayPlay | DayBoarding | Grooming | Training | DaySpa | unknown | other",
  "requested_add_ons": [
    { "value": "exit_bath", "state": "known", "confidence": "medium", "evidence": [] }
  ],
  "pet_slot_ids": ["pet_slot_1"],
  "service_notes": { "value": "Interested in webcam suite", "state": "known", "confidence": "medium", "evidence": [] }
}
```

Rules:

- Normalize only to existing `ServiceKind` values where the intent is clear: `Boarding`, `DayPlay`, `DayBoarding`, `Grooming`, `Training`, `DaySpa`.
- Ambiguous service terms such as “day stay,” “camp,” “spa,” “play,” or “class” require either a confidence note or `missing_info.field=service_kind_confirmation`.
- The schema may identify likely service intent but must not promise availability, price, package eligibility, or booking acceptance.

### 6. Dates and times

Represent date/time as one or more requested windows with timezone and interpretation state.

```json
{
  "date_slot_id": "date_1",
  "applies_to_service_slot_ids": ["svc_1"],
  "applies_to_pet_slot_ids": ["pet_slot_1"],
  "start": { "value": "2026-07-03", "state": "known", "confidence": "high", "evidence": [] },
  "end": { "value": "2026-07-07", "state": "known", "confidence": "high", "evidence": [] },
  "time_window": { "value": "morning drop-off", "state": "known", "confidence": "medium", "evidence": [] },
  "timezone": { "value": "location_timezone", "state": "ambiguous", "confidence": "low", "evidence": [] },
  "date_type": "date_range | single_day | recurring | flexible | unknown",
  "interpretation_notes": []
}
```

Rules:

- Relative dates (“tomorrow,” “next weekend,” “over Thanksgiving”) require the source timestamp and location timezone before they can become concrete dates.
- Boarding ranges must keep start and end separate. Do not infer nights, check-in, checkout, or minimum-stay compliance in the extraction schema; downstream booking triage handles policy.
- If only a month/season/holiday is provided, use `date_type=flexible` or `unknown` and include missing info for exact dates/times.

### 7. Special needs and care notes

Special needs cover care, feeding, medication, allergy, medical, mobility, anxiety, handling, emergency/vet contact, and other non-routine requirements.

```json
{
  "need_id": "need_1",
  "pet_slot_ids": ["pet_slot_1"],
  "category": "feeding | medication | allergy | medical_condition | mobility | anxiety | behavior | emergency_contact | veterinarian_contact | other",
  "description": { "value": "takes medication with dinner", "state": "known", "confidence": "medium", "evidence": [] },
  "structured_details": {
    "medication_name": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] },
    "dose": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] },
    "schedule": { "value": "with dinner", "state": "known", "confidence": "medium", "evidence": [] }
  },
  "review_gate_hint": "medical_or_medication_review | behavior_review | manager_review | none"
}
```

Rules:

- Medical, allergy, medication, mobility, emergency, and veterinarian details are evidence for review, not executable care instructions.
- Vague medication mentions must create missing info for medication name/dose/schedule and a `medical_or_medication_review` route.
- Do not diagnose, reassure, or decide service eligibility from special-needs text.

### 8. Temperament

Temperament is extracted as owner/staff-reported evidence and review routing, not group-play eligibility.

```json
{
  "temperament_id": "temp_1",
  "pet_slot_ids": ["pet_slot_1"],
  "reported_summary": { "value": "friendly but nervous around large dogs", "state": "known", "confidence": "medium", "evidence": [] },
  "signals": [
    { "value": "nervous_around_large_dogs", "state": "known", "confidence": "medium", "evidence": [] }
  ],
  "group_play_relevance": "possibly_relevant | not_mentioned | needs_behavior_review | unknown",
  "review_gate_hint": "BehaviorReview | ManagerApproval | none"
}
```

Rules:

- Mentions of aggression, bite history, fear/anxiety, escape risk, resource guarding, intact status, in-heat status, or prior incident require risk flags/review.
- The inquiry-intake schema may say “needs behavior review” but must not clear or deny group play.

### 9. Vaccine mention/status

The intake agent only captures whether vaccines/documents are mentioned and what follow-up may be needed. It does not approve vaccine compliance.

```json
{
  "mention_state": "not_mentioned | mentioned_current | mentioned_missing | mentioned_expired | document_uploaded | unknown | conflicting",
  "per_pet": [
    {
      "pet_slot_id": "pet_slot_1",
      "mentioned_vaccines": [
        { "value": "rabies", "state": "known", "confidence": "medium", "evidence": [] }
      ],
      "document_refs": [],
      "owner_claim": { "value": "up to date", "state": "known", "confidence": "medium", "evidence": [] },
      "trusted_status": "not_evaluated | needs_document | needs_medical_document_review | source_backed_current | conflicting | unknown"
    }
  ],
  "review_gate_hint": "MedicalDocumentReview | none"
}
```

Rules:

- Owner statements like “shots are current” are customer-reported claims, not verified vaccine status.
- Uploaded records, screenshots, provider notes, and attachments are evidence refs for the vaccine-document workflow; they do not become accepted vaccine facts here.
- If vaccine status affects booking readiness or customer copy, include `risk_flags` such as `missing_required_vaccine_proof` or `vaccine_status_unverified`.

### 10. Urgency

Urgency is a routing hint for staff, not permission to skip review gates.

```json
{
  "level": "low | normal | high | urgent | emergency | unknown",
  "basis": [
    { "value": "needs boarding tomorrow", "state": "known", "confidence": "high", "evidence": [] }
  ],
  "recommended_queue_priority": "normal | elevated | immediate_staff_review | emergency_handoff",
  "review_gate_hint": "ManagerApproval | staff_review | emergency_protocol | none"
}
```

Rules:

- Near-term dates, repeated follow-ups, customer distress, safety/medical terms, or operational deadlines may raise urgency.
- Medical emergencies, injury, severe illness, aggression/safety threats, legal/regulatory threats, and active complaint language should route to manager/emergency protocol, not a normal lead workflow.
- Urgency never authorizes a booking promise, policy exception, live provider mutation, or unapproved send.

### 11. Missing info

`missing_info` is a normalized checklist for staff follow-up and task/draft generation.

```json
{
  "field": "owner_name | contact_method | pet_name | species | breed | size | requested_service | requested_dates | requested_times | vaccine_proof | special_needs_details | temperament_details | consent_or_preference | location | other",
  "applies_to": {
    "owner": true,
    "contact": false,
    "pet_slot_ids": ["pet_slot_1"],
    "service_slot_ids": ["svc_1"],
    "date_slot_ids": []
  },
  "required_for": ["lead_creation", "staff_follow_up", "booking_triage", "vaccine_document_review", "customer_message_draft"],
  "severity": "blocking | recommended | optional",
  "question_for_customer": "What dates are you hoping to book?",
  "internal_note": "Exact dates are needed before booking triage can check availability."
}
```

Minimum completeness gates for a safe follow-up packet:

- `owner.full_name` known or enough external lead identity to route staff review.
- At least one contact method or portal/provider lead ref for staff follow-up.
- At least one requested service or a missing-info item asking for service intent.
- At least one pet slot or a missing-info item asking for pet details.
- Date/time fields are either known/flexible by design or included in missing info.

### 12. Ambiguities and conflicts

Use explicit ambiguity records so consumers do not infer certainty from absent values.

```json
{
  "field": "requested_dates.start",
  "state": "ambiguous | conflicting | unknown",
  "candidates": [
    { "value": "2026-07-03", "source_ref": "source_message:msg_123:field:start_date" },
    { "value": "2026-07-04", "source_ref": "source_message:msg_123:free_text" }
  ],
  "required_resolution": "staff_review | customer_follow_up | provider_reconciliation | policy_decision",
  "blocks": ["booking_triage", "customer_message_send"]
}
```

Rules:

- Ambiguous/conflicting facts must also be reflected in envelope `uncertainty`/`missing_inputs` if the consuming envelope supports those fields.
- If a field is conflicting, do not populate a single normalized field as `known` unless it is clearly a display-only summary and cannot affect behavior.

## Validation rules

Schema-level validation:

1. `schema_name`, `workflow_name`, `event_id`, and `subject` must match the prompt packet/event.
2. `structured_output.location_id` must match `WorkflowEvent.location_id`.
3. `source.source_kind`, `requested_services[].service_mapping`, contact channel mappings, urgency, missing-info fields, and review-gate hints must be enums, not arbitrary strings.
4. Every known/conflicting/ambiguous extracted field that affects mapping, tasking, review, or drafts must include at least one `evidence.source_ref` or deterministic source reference.
5. `pets[].pet_slot_id`, `requested_services[].service_slot_id`, and `requested_dates[].date_slot_id` must be unique within the output.
6. Cross-references from services/dates/special-needs/temperament/vaccine rows to pet/service/date slots must resolve.
7. `source_kind=chat_widget` must not silently map to `WebsiteForm` or `Portal`; use `unmapped_chat_widget` or a future explicit enum.
8. `status=completed` is valid only when the output is safe to persist/display as an internal extraction packet. It still does not authorize sends or external mutations.
9. `status=needs_more_information` requires at least one `missing_info` item or a human review reason explaining the missing blocker.
10. `status=needs_human_review` requires `human_review_reason` and at least one review/risk/ambiguity basis.
11. Any `draft_message` or customer-facing follow-up recommendation must include `CustomerMessageApproval` unless a later deterministic send policy is cited.
12. Raw provider JSON, raw email threads, unredacted full transcripts, payment/card data, webhook signatures, secrets, and unrelated PII are invalid inside the structured output.

Domain validation before any downstream action:

- Customer/pet/reservation ids must be looked up by deterministic repositories; the model cannot invent ids.
- Customer merges, pet matches, and conflicting profile updates require review.
- Reservation creation/update is not performed by this schema. The mapping below creates proposed record drafts only.
- Vaccine, medical, behavior, payment, capacity, and booking decisions must be delegated to their own workflows/review gates.
- External provider writes and customer sends require separate approved commands and idempotency keys.

## Multi-pet handling

Required behavior:

- Represent each mentioned animal as a separate `pets[]` item even if some fields are missing.
- Preserve source language that indicates shared facts: “both dogs,” “for Luna and Max,” “same dates,” “one needs meds.”
- Use slot ids to attach requested services, dates, special needs, temperament facts, and vaccine mentions to the correct pet(s).
- If a source cannot establish whether a fact applies to one pet or all pets, set the per-fact association to ambiguous and add a missing-info item.
- Do not collapse multi-pet inquiries into one pet summary for booking triage. Downstream booking/policy workflows may need per-pet eligibility, care, vaccine, behavior, room/group, and pricing review.

Open design question to carry forward:

- The canonical data-model doc leaves open whether multi-pet bookings are one reservation with per-pet segments or linked child reservations. This extraction schema should therefore emit neutral pet/service/date slots and let the lead/reservation mapping layer choose the aggregate strategy.

## Unknown, ambiguous, and low-confidence values

Use these conventions consistently:

- Unknown: field absent from checked evidence. Include a missing-info item when required for follow-up or routing.
- Ambiguous: source has a possible value but normalization is uncertain, e.g. “daycare” vs “day boarding,” “this Friday,” “small dog,” “shots current.”
- Conflicting: two or more evidence items disagree, e.g. email says July 3, form field says July 4.
- Low confidence: extraction/parsing was weak but a candidate exists. Low confidence does not mean unknown; preserve the candidate as a candidate with review/missing info.
- Not applicable: field truly does not apply to the service/pet/source, e.g. checkout time for a simple grooming info request.
- Redacted: source exists but value is deliberately omitted from the runtime result/log for minimization.

Consumers must not treat omitted fields as no-risk. Required fields should be present with explicit state values.

## Risk flags

Suggested typed risk flags for inquiry intake:

- `customer_message_requires_review`
- `provider_payload_unverified`
- `raw_pii_redacted`
- `missing_contact_method`
- `missing_pet_identity`
- `missing_requested_service`
- `missing_requested_dates`
- `vaccine_status_unverified`
- `missing_required_vaccine_proof`
- `medical_or_medication_ambiguity`
- `behavior_review_needed`
- `possible_safety_or_incident_language`
- `urgent_near_term_request`
- `payment_sensitive_content`
- `policy_exception_requested`
- `availability_or_booking_promise_requested`
- `multi_pet_association_ambiguous`
- `chat_source_unmapped`

Risk flags route review and UI attention. They do not approve, deny, send, or mutate anything.

## Recommended actions, tasks, and drafts

Allowed recommendation types from inquiry extraction:

- Internal follow-up task draft for front desk/staff to collect missing information.
- Internal review task draft for medical/document, behavior, manager, or integration review when the source includes sensitive/ambiguous claims.
- Customer follow-up draft asking for missing non-sensitive intake information, with `CustomerMessageApproval` by default.
- Request human review for ambiguous identity/contact, source verification, medical/vaccine/behavior/payment/safety, or policy-exception content.
- Safe reservation status suggestion only as a draft/recommendation, typically `Inquiry`, `MissingInfo`, `VaccinePending`, or `SpecialReview`; never `Confirmed` from inquiry intake.

Forbidden recommendation behavior:

- No booking confirmation, rejection, waitlist movement, availability promise, room/group assignment, price/deposit quote, refund/waiver/discount, vaccine/medical approval, group-play eligibility clearance, or provider mutation.
- No customer-facing send command. Drafts are content artifacts only.

Example task draft mapping:

```json
{
  "kind": "customer_follow_up",
  "title": "Collect missing dates and vaccine proof for Luna inquiry",
  "assignment": "front_desk",
  "priority": "elevated",
  "creation_policy": "DraftOnly or AutoCreateAllowed(policy_ref) if later approved",
  "evidence_refs": ["workflow_event:evt_123", "source_message:msg_123:excerpt_1"],
  "dedupe_key": "internal_task:external:inquiry_follow_up:v1"
}
```

Example draft message posture:

```json
{
  "audience": "customer",
  "channel": "email",
  "message_kind": "inquiry_missing_info_follow_up",
  "body": "Hi Jordan — thanks for reaching out about boarding for Luna. To help our team review next steps, could you share the exact dates you have in mind and upload Luna's current vaccine record?",
  "send_policy": "RequiresApproval(CustomerMessageApproval)",
  "source_refs": ["workflow_event:evt_123", "pet_slot:pet_slot_1", "missing_info:requested_dates", "missing_info:vaccine_proof"],
  "claims_not_made": ["booking confirmed", "availability available", "vaccines approved", "price quoted"]
}
```

## Record mapping

The extraction output should include an explicit mapping section so downstream consumers can create review packets without re-interpreting prose.

```json
{
  "lead_record": {
    "action": "create_candidate | update_candidate | link_existing | no_action | needs_review",
    "subject": "WorkflowSubject::External or WorkflowSubject::Customer",
    "fields": {
      "owner_name": "owner.full_name",
      "contact_refs": ["contact.email", "contact.phone"],
      "source_kind": "source.source_kind",
      "source_refs": ["source_message:msg_123"]
    },
    "review_required": false,
    "reason": "New external inquiry with enough contact info for staff follow-up."
  },
  "customer_record": {
    "action": "create_candidate | link_existing | possible_match_review | no_action",
    "customer_id": null,
    "proposed_fields": ["full_name", "email", "mobile_phone", "preferred_contact"],
    "blocked_fields": ["consent_state"],
    "review_required": true
  },
  "pet_records": [
    {
      "pet_slot_id": "pet_slot_1",
      "action": "create_candidate | link_existing | possible_match_review | no_action",
      "pet_id": null,
      "proposed_fields": ["name", "species", "breed", "size_reported"],
      "review_required": false
    }
  ],
  "reservation_or_request": {
    "action": "create_inquiry_candidate | link_existing_reservation | defer_until_missing_info | no_action",
    "reservation_id": null,
    "proposed_status": "Inquiry | MissingInfo | VaccinePending | SpecialReview | none",
    "proposed_fields": ["location_id", "customer_id_or_external_lead", "pet_slots", "service", "requested_dates", "source"],
    "review_required": true,
    "reason": "No live reservation/provider mutation from inquiry-intake."
  },
  "tasks": [],
  "events": [
    {
      "event_type": "inquiry.received",
      "subject": "external_or_customer",
      "source_event_key_basis": "location + InquiryReceived + subject + source_kind + source_fingerprint",
      "audit_action": "WorkflowEventRecorded"
    }
  ]
}
```

Mapping rules:

- Lead/task/event records may be drafted or recommended from extraction; canonical customer/pet/reservation changes still require deterministic validators and approved repository/tool paths.
- If the product later adds a first-class Inquiry/Lead aggregate, map `lead_record` directly to that aggregate. Until then, preserve both `WorkflowSubject::External` and optional `ReservationStatus::Inquiry` mapping options.
- `Reservation` mapping is candidate-only at this stage. A downstream booking/request workflow decides whether to create a reservation in `Inquiry`/`Requested`/`MissingInfo` and whether provider writes are allowed.
- Task creation must use source-event dedupe and policy. Draft task suggestions are safe; live staff tasks require a defined task-creation policy.
- Audit/event records should store event ids, source refs, field/category lists, validation outcome, policy/review gates, and redacted evidence refs rather than raw transcripts.

## Example full structured output

```json
{
  "intake_id": null,
  "location_id": "loc_1",
  "source": {
    "source_kind": "website_form",
    "reservation_source_mapping": "WebsiteForm",
    "source_message_id": "form_123",
    "provider_ref": null,
    "submitted_at": { "value": "2026-06-11T13:00:00Z", "state": "known", "confidence": "high", "evidence": [{ "source_ref": "website_form:form_123:submitted_at", "span": "field:submitted_at", "source_trust_state": "customer_reported" }] },
    "actor_kind": "customer",
    "verification_state": "normalized_import"
  },
  "owner": {
    "customer_id": null,
    "external_lead_id": "lead_123",
    "full_name": { "value": "Jordan Lee", "state": "known", "confidence": "high", "evidence": [{ "source_ref": "website_form:form_123", "span": "field:owner_name", "source_trust_state": "customer_reported" }] },
    "household_or_co_owner": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] },
    "identity_match": { "state": "new_lead", "matched_customer_ids": [], "reason": "No deterministic customer match supplied in prompt packet." }
  },
  "contact": {
    "email": { "value": "jordan@example.com", "state": "known", "confidence": "high", "evidence": [{ "source_ref": "website_form:form_123", "span": "field:email", "source_trust_state": "customer_reported" }] },
    "phone": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] },
    "preferred_channel": { "value": "email", "state": "known", "confidence": "medium", "evidence": [{ "source_ref": "website_form:form_123", "span": "field:preferred_contact", "source_trust_state": "customer_reported" }] },
    "contact_channel_mapping": "Email",
    "consent_state": "not_modeled",
    "quiet_hours_or_suppression_state": "not_modeled"
  },
  "pets": [
    {
      "pet_slot_id": "pet_slot_1",
      "pet_id": null,
      "name": { "value": "Luna", "state": "known", "confidence": "high", "evidence": [{ "source_ref": "website_form:form_123", "span": "field:pet_name", "source_trust_state": "customer_reported" }] },
      "species": { "value": "dog", "state": "known", "confidence": "high", "evidence": [{ "source_ref": "website_form:form_123", "span": "field:species", "source_trust_state": "customer_reported" }] },
      "breed": { "value": "goldendoodle", "state": "known", "confidence": "medium", "evidence": [{ "source_ref": "website_form:form_123", "span": "field:breed", "source_trust_state": "customer_reported" }] },
      "size": { "reported_weight": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] }, "size_class": { "value": "medium", "state": "ambiguous", "confidence": "low", "evidence": [{ "source_ref": "website_form:form_123", "span": "field:size", "source_trust_state": "customer_reported" }] }, "derivation": "reported_by_owner" },
      "age_or_birth_date": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] },
      "sex": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] },
      "spay_neuter_status": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] },
      "mapping_state": "new_pet",
      "per_pet_missing_info": ["weight_or_size_confirmation", "spay_neuter_status"],
      "per_pet_risk_flags": []
    }
  ],
  "requested_services": [
    { "service_slot_id": "svc_1", "service_kind": { "value": "boarding", "state": "known", "confidence": "high", "evidence": [{ "source_ref": "website_form:form_123", "span": "field:service", "source_trust_state": "customer_reported" }] }, "service_mapping": "Boarding", "requested_add_ons": [], "pet_slot_ids": ["pet_slot_1"], "service_notes": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] } }
  ],
  "requested_dates": [
    { "date_slot_id": "date_1", "applies_to_service_slot_ids": ["svc_1"], "applies_to_pet_slot_ids": ["pet_slot_1"], "start": { "value": "2026-07-03", "state": "known", "confidence": "high", "evidence": [{ "source_ref": "website_form:form_123", "span": "field:start_date", "source_trust_state": "customer_reported" }] }, "end": { "value": "2026-07-07", "state": "known", "confidence": "high", "evidence": [{ "source_ref": "website_form:form_123", "span": "field:end_date", "source_trust_state": "customer_reported" }] }, "time_window": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] }, "timezone": { "value": "location_timezone", "state": "ambiguous", "confidence": "low", "evidence": [] }, "date_type": "date_range", "interpretation_notes": ["Timezone requires location policy/context before operational use."] }
  ],
  "special_needs": [],
  "temperament": [],
  "vaccine_status": { "mention_state": "mentioned_missing", "per_pet": [{ "pet_slot_id": "pet_slot_1", "mentioned_vaccines": [], "document_refs": [], "owner_claim": { "value": null, "state": "unknown", "confidence": "none", "evidence": [] }, "trusted_status": "needs_document" }], "review_gate_hint": "MedicalDocumentReview" },
  "urgency": { "level": "normal", "basis": [], "recommended_queue_priority": "normal", "review_gate_hint": "none" },
  "missing_info": [
    { "field": "vaccine_proof", "applies_to": { "owner": false, "contact": false, "pet_slot_ids": ["pet_slot_1"], "service_slot_ids": ["svc_1"], "date_slot_ids": [] }, "required_for": ["booking_triage", "vaccine_document_review"], "severity": "blocking", "question_for_customer": "Please upload Luna's current vaccine record.", "internal_note": "Vaccine proof is needed before downstream booking readiness review." }
  ],
  "ambiguities": [],
  "validation": { "completeness_state": "missing_info", "schema_warnings": ["consent policy not modeled"], "side_effect_authority": "none" },
  "record_mapping": { "lead_record": { "action": "create_candidate", "subject": "WorkflowSubject::External", "fields": { "owner_name": "owner.full_name", "contact_refs": ["contact.email"], "source_kind": "source.source_kind", "source_refs": ["website_form:form_123"] }, "review_required": false, "reason": "New external inquiry with enough contact info for staff follow-up." }, "customer_record": { "action": "create_candidate", "customer_id": null, "proposed_fields": ["full_name", "email", "preferred_contact"], "blocked_fields": ["consent_state"], "review_required": true }, "pet_records": [{ "pet_slot_id": "pet_slot_1", "action": "create_candidate", "pet_id": null, "proposed_fields": ["name", "species", "breed", "size_reported"], "review_required": false }], "reservation_or_request": { "action": "create_inquiry_candidate", "reservation_id": null, "proposed_status": "MissingInfo", "proposed_fields": ["location_id", "external_lead", "pet_slots", "service", "requested_dates", "source"], "review_required": true, "reason": "No live reservation/provider mutation from inquiry-intake." }, "tasks": [], "events": [{ "event_type": "inquiry.received", "subject": "external_or_customer", "source_event_key_basis": "location + InquiryReceived + subject + source_kind + source_fingerprint", "audit_action": "WorkflowEventRecorded" }] },
  "source_refs": ["workflow_event:evt_123", "website_form:form_123"]
}
```

## Final handoff summary for `inquiry-intake-agent.md`

Define `InquiryIntakeExtraction` as the `structured_output` under a `WorkflowResult` envelope for `WorkflowEventType::InquiryReceived`. The schema must preserve source refs, confidence, source spans, explicit unknown/ambiguous/conflicting states, per-pet slotting, missing-info checklist, risk flags, and candidate-only mapping to lead/customer/pet/reservation/task/event records. The agent may extract owner/contact/pet/service/date/special-need/temperament/vaccine-mention/urgency facts, propose internal follow-up/review tasks, and draft customer follow-up copy under `CustomerMessageApproval`; it must not confirm bookings, promise availability, approve vaccines/medical/behavior/payment/eligibility, mutate providers, or send customer messages without a separate approved side-effect path.
