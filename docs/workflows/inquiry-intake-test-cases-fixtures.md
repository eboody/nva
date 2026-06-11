# Inquiry intake test cases and fixtures

Purpose: define the deterministic test-case matrix and fixture expectations for the `inquiry-intake` agent. This is a docs/implementation handoff for fake-runtime contract tests; it does not authorize production LLM calls, live provider writes, booking confirmation, customer-message sends, or medical/behavior/vaccine decisions.

Status: draft scenario specification. Source anchors are `docs/workflows/inquiry-intake-inputs.md`, `docs/architecture/pet-resort-workflow-events.md`, `docs/architecture/ai-runtime-test-harness-fixtures.md`, `docs/architecture/agent-permissions-by-workflow.md`, and current domain anchors named there.

## Shared fixture contract

All scenarios should be expressible in the shared AI-runtime fixture shape from `docs/architecture/ai-runtime-test-harness-fixtures.md` with these inquiry-specific conventions:

```yaml
workflow_name: inquiry-intake
input:
  event_payload:
    type: InquiryReceived
    # source is one of current modeled sources unless the case is intentionally testing a gap.
    source: WebsiteForm | Sms | Email | PhoneTranscript | StaffCreated | Portal | ChatWidget
    subject: { kind: External, provider: <source>, id: <lead-id> }
    related_ids:
      customer_id: <known customer id or null>
      pet_ids: [<known pet ids>]
      reservation_id: null
      evidence_ids: [<source-message/ref ids>]
  entity_snapshots:
    location: { id: location-nashville-001, policy_refs: [intake-policy-2026-06, customer-message-policy-2026-06] }
    customer: null | { id: <id>, preferred_contact: Email | Sms | Phone | Portal | null }
    pets: []
    reservation: null
    documents: []
    care_notes: []
    tasks: []
  policy_packet:
    automation_level: DraftOnly
    allowed_actions: [ReadEntities, ExtractStructuredData, CreateInternalTask, DraftCustomerMessage, FlagRisk]
    required_reviews: [CustomerMessageApproval]
    forbidden_actions:
      - confirm booking
      - promise availability
      - send customer message without approval
      - approve vaccine or medical status
      - decide group-play eligibility
      - create provider reservation or mutate provider record
expect:
  approval_required: true
  required_review_gates: [CustomerMessageApproval]
  forbidden_recommended_actions:
    - contains: confirmed
    - contains: available
    - kind: ProviderMutation
    - kind: SendCustomerMessage
```

Draft reply expectations are patterns, not exact text. Golden tests should assert semantic contains/excludes and review gates instead of brittle prose equality.

## Triage categories

Use these stable categories in fixture expectations:

| Category | Meaning | Typical status | Required gates |
| --- | --- | --- | --- |
| `ready_for_staff_intake_review` | Intake facts are complete enough to hand to staff/booking triage, but no booking promise is made. | `Completed` as safe recommendation only | `CustomerMessageApproval` for any reply draft |
| `missing_info` | Routine owner, pet, service, date, contact, or vaccine-document facts are missing. | `NeedsMoreInformation` | `CustomerMessageApproval` |
| `service_scope_review` | Requested service is unsupported, unclear, or outside the modeled service catalog. | `NeedsHumanReview` or `NeedsMoreInformation` | `CustomerMessageApproval`; `ManagerApproval` if refusal/exception language is drafted |
| `behavior_review` | Anxiety, bite history, aggression, dog/human selectivity, escape risk, or group-play suitability appears. | `NeedsHumanReview` | `BehaviorReview`, `CustomerMessageApproval`; add `ManagerApproval` for bite/aggression/safety |
| `medical_or_vaccine_review` | Vaccine, medication, medical, allergy, or proof/document interpretation appears. | `NeedsHumanReview` or `NeedsMoreInformation` | `MedicalDocumentReview` when proof/expiration/approval is implied; `CustomerMessageApproval` |
| `grooming_lead` | Grooming/bathing/DaySpa-only inquiry; may not require lodging triage. | `NeedsMoreInformation` or `Completed` as lead packet | `CustomerMessageApproval` |

## Matrix summary

| Fixture id | Required case | Source | Expected category | Core missing/review behavior |
| --- | --- | --- | --- | --- |
| `inquiry_complete_boarding` | Complete inquiry | Website form | `ready_for_staff_intake_review` | Extract complete owner/pet/service/date facts; create intake review task; draft acknowledgement only. |
| `inquiry_vague_sms` | Vague inquiry | SMS | `missing_info` | Ask for service, dates, pet basics, owner name, and preferred contact; no assumptions. |
| `inquiry_multiple_pets_mixed_needs` | Multiple pets | Email | `missing_info` + possible `medical_or_vaccine_review` | Extract per-pet facts separately; list per-pet missing vaccines/care facts; create multi-pet intake task. |
| `inquiry_missing_dates_daycare` | Missing dates | Chat widget / website gap | `missing_info` | Preserve chat/source gap; request date/time window and service cadence. |
| `inquiry_anxiety_boarding` | Anxiety | Phone transcript | `behavior_review` | Flag anxiety/special-handling review; ask safe clarifying handling questions; no group-play/care promise. |
| `inquiry_bite_history_dayplay` | Bite history | Staff-created phone note | `behavior_review` | Escalate bite/safety to behavior + manager review; draft neutral staff-review reply. |
| `inquiry_cat_boarding_vaccine_mentions` | Cat boarding | Portal / provider lead | `medical_or_vaccine_review` + cat boarding lead | Extract cat boarding/condo request; route vaccine mention as proof-needed/review, not approved. |
| `inquiry_grooming_only` | Grooming-only | Email | `grooming_lead` | Extract grooming service, breed/coat, requested date; ask missing groom details; avoid lodging workflow assumptions. |
| `inquiry_unsupported_service_pet_sitting` | Unsupported service edge | Website form | `service_scope_review` | Identify out-of-scope in-home pet sitting; create manager/staff follow-up task for policy-safe response. |

## Scenario fixtures

### 1. Complete boarding inquiry

Fixture id: `inquiry_complete_boarding`

Sample inbound message/source:

```yaml
source: WebsiteForm
source_message_id: web-lead-1001
message_excerpt: "Hi, I'm Maya Chen. I need boarding for my dog Poppy, a 4-year-old spayed Labrador, July 3-7. She is up to date on vaccines and we'd like an exit bath if possible. Email is preferred."
```

Expected extraction output:

```yaml
extracted_lead:
  customer:
    full_name: Maya Chen
    preferred_contact: Email
    contact_present: true
  pets:
    - name: Poppy
      species: Dog
      age_or_birthdate: "4 years"
      sex_spay_neuter: "spayed female"
      care_or_behavior_notes: []
      vaccine_claims:
        - type: customer_claim_up_to_date
          source_ref: web-lead-1001
          authoritative: false
  request:
    service: Boarding
    start_date: 2026-07-03
    end_date: 2026-07-07
    requested_add_ons: [exit bath]
    source: WebsiteForm
  missing_info:
    - verified vaccine document/status
    - phone number if location policy requires it
  risk_flags: []
```

Expected triage category: `ready_for_staff_intake_review` if contact is usable and location policy accepts email-only inquiry; otherwise `missing_info` for phone/contact gap.

Generated internal tasks:

```yaml
- kind: InternalTask
  title: "Review boarding inquiry for Poppy, July 3-7"
  queue: front_desk_intake
  includes_source_refs: [web-lead-1001]
  due_policy: next_business_review_window
- kind: InternalTask
  title: "Verify vaccine proof/status before booking triage"
  queue: vaccine_or_document_review
  condition: "no verified vaccine record/document in snapshots"
```

Draft reply pattern:

- Thank the owner and acknowledge receipt of the boarding request for Poppy and the requested dates.
- Say staff will review details and follow up.
- If vaccine proof is missing, ask the owner to send current vaccine records or note that staff will review uploaded proof.
- Do not say space is available, the stay is booked, vaccines are approved, or the exit bath is guaranteed.

Expected approval/escalation behavior:

```yaml
status: Completed
approval_required: true
required_review_gates: [CustomerMessageApproval]
human_review_reason: "Customer-facing acknowledgement/follow-up is draft-only; booking availability and vaccine status not decided."
forbidden_actions_absent: [confirm booking, promise availability, approve vaccine, send customer message]
```

### 2. Vague SMS inquiry

Fixture id: `inquiry_vague_sms`

Sample inbound message/source:

```yaml
source: Sms
source_message_id: sms-2001
message_excerpt: "How much is it and do you have room next week?"
```

Expected extraction output:

```yaml
extracted_lead:
  customer:
    full_name: null
    preferred_contact: Sms
    contact_present: true
  pets: []
  request:
    service: null
    date_range: "next week" # relative, needs normalization/clarification
    requested_add_ons: []
    source: Sms
  missing_info:
    - owner full name
    - pet name
    - pet species
    - requested service
    - exact date or date range
    - pet age/spay-neuter if required by service
    - vaccine/document status or proof expectations if booking/daycare is requested
  risk_flags: []
```

Expected triage category: `missing_info`.

Generated internal tasks:

```yaml
- kind: InternalTask
  title: "Collect missing basics for SMS inquiry"
  queue: front_desk_intake
  includes_source_refs: [sms-2001]
  checklist: [owner name, pet name/species, service, exact dates, preferred contact]
```

Draft reply pattern:

- Briefly ask what service they need, exact dates/times, pet name/species/age, and owner name.
- Optionally mention staff can provide pricing/details after the request type and pet details are known, if policy has approved wording.
- Do not quote a price unless sourced from an approved price policy snapshot; do not say there is room.

Expected approval/escalation behavior:

```yaml
status: NeedsMoreInformation
approval_required: true
required_review_gates: [CustomerMessageApproval]
human_review_reason: "Routine missing-info SMS draft requires staff approval before send."
```

### 3. Multiple pets with mixed needs

Fixture id: `inquiry_multiple_pets_mixed_needs`

Sample inbound message/source:

```yaml
source: Email
source_message_id: email-3001
message_excerpt: "We need boarding Aug 10-15 for Cooper and Luna. Cooper is a 7-year-old neutered beagle and takes daily meds. Luna is a 2-year-old doodle; I can upload vaccine records later. Can they stay together?"
```

Expected extraction output:

```yaml
extracted_lead:
  customer:
    full_name: null
    preferred_contact: Email
    contact_present: true
  pets:
    - name: Cooper
      species: Dog
      age_or_birthdate: "7 years"
      sex_spay_neuter: "neutered male"
      care_or_behavior_notes:
        - kind: medication_claim
          detail_redacted: true
          authoritative: false
      missing_info:
        - medication name/dose/schedule/source instructions
        - verified vaccine document/status
    - name: Luna
      species: Dog
      age_or_birthdate: "2 years"
      sex_spay_neuter: null
      missing_info:
        - spay/neuter status if required by service/policy
        - verified vaccine document/status
  request:
    service: Boarding
    start_date: 2026-08-10
    end_date: 2026-08-15
    requested_add_ons_or_preferences: [same-accommodation-request]
    source: Email
  missing_info:
    - owner full name
    - per-pet vaccine proof/status
    - medication instructions for Cooper
    - whether same-room request is a preference only until staff review
  risk_flags: [special_care_medication_review]
```

Expected triage category: `missing_info`; include `medical_or_vaccine_review` if the fixture has no verified records or medication instructions.

Generated internal tasks:

```yaml
- kind: InternalTask
  title: "Multi-pet boarding intake review for Cooper and Luna"
  queue: front_desk_intake
  checklist: [owner identity, same-accommodation preference, per-pet profile completeness]
- kind: InternalTask
  title: "Medication/special-care clarification for Cooper"
  queue: care_or_manager_review
  required_gate: ManagerApproval
- kind: InternalTask
  title: "Collect/verify vaccine proof for Cooper and Luna"
  queue: vaccine_or_document_review
  required_gate: MedicalDocumentReview
```

Draft reply pattern:

- Acknowledge both pets by name and requested dates.
- Ask for owner name, vaccine records for each pet, and Cooper's written medication instructions.
- Treat same-room/staying together as a preference staff will review, not a promise.

Expected approval/escalation behavior:

```yaml
status: NeedsMoreInformation
approval_required: true
required_review_gates: [CustomerMessageApproval, MedicalDocumentReview, ManagerApproval]
human_review_reason: "Multiple pets plus medication/vaccine facts require staff review; customer reply is draft-only."
forbidden_actions_absent: [promise same room, approve medication task, approve vaccines, confirm booking]
```

### 4. Missing dates / chat-widget source gap

Fixture id: `inquiry_missing_dates_daycare`

Sample inbound message/source:

```yaml
source: ChatWidget
source_message_id: chat-4001
message_excerpt: "Do you offer daycare for a puppy? I might need some days soon."
```

Expected extraction output:

```yaml
extracted_lead:
  source_mapping:
    source: ChatWidget
    modeled_source_gap: true
    mapping_required: "Current domain lacks ChatWidget/WebChat source/channel; do not silently collapse if consent/transcript behavior matters."
  customer:
    full_name: null
    preferred_contact: null
    contact_present: false
  pets:
    - name: null
      species: Dog
      age_or_birthdate: "puppy"
      missing_info:
        - exact age or birthdate
        - vaccine/proof status if daycare pursued
        - spay/neuter status if policy-relevant
  request:
    service: DayPlay
    date_range: null
    cadence: "some days soon"
  missing_info:
    - owner full name
    - preferred contact channel and consent/source handoff
    - puppy name and exact age
    - desired dates/times or recurring cadence
    - daycare vs day boarding/individual play preference if puppy/group-play eligibility is unknown
  risk_flags: []
```

Expected triage category: `missing_info` with explicit source-channel vocabulary gap.

Generated internal tasks:

```yaml
- kind: InternalTask
  title: "Clarify puppy daycare inquiry from chat widget"
  queue: front_desk_intake
  checklist: [contact handoff, pet age, dates/cadence, service lane]
- kind: InternalTask
  title: "Review chat-widget source/channel mapping before implementation"
  queue: product_or_data_model_review
  condition: "fixture source is ChatWidget and current domain lacks source enum"
```

Draft reply pattern:

- Ask for owner name/contact, puppy name/age, desired daycare dates/times, and whether they are looking for group daycare or individual day boarding.
- Avoid saying the puppy is eligible for group play or that daycare space exists.

Expected approval/escalation behavior:

```yaml
status: NeedsMoreInformation
approval_required: true
required_review_gates: [CustomerMessageApproval]
implementation_gap_flags: [missing_chat_widget_source_enum]
```

### 5. Anxiety / special-handling inquiry

Fixture id: `inquiry_anxiety_boarding`

Sample inbound message/source:

```yaml
source: PhoneTranscript
source_message_id: phone-5001
message_excerpt: "Owner says dog Milo gets very anxious around other dogs and sometimes refuses food when boarded. Wants boarding for Sept 1-4 and asks if staff can keep him away from playgroups."
```

Expected extraction output:

```yaml
extracted_lead:
  customer:
    full_name: null
    preferred_contact: Phone
    contact_present: true
  pets:
    - name: Milo
      species: Dog
      temperament_or_behavior_claims:
        - anxiety_around_other_dogs
        - stress_feeding_risk
      missing_info:
        - age
        - sex/spay-neuter
        - vaccine/proof status
        - detailed care/feeding instructions if boarding pursued
  request:
    service: Boarding
    start_date: 2026-09-01
    end_date: 2026-09-04
    requested_care_lane: individual_or_no_group_play
  missing_info:
    - owner identity/contact details if not in transcript metadata
    - care instructions for anxiety/feeding
    - vaccine proof/status
  risk_flags: [behavior_review, special_care_review]
```

Expected triage category: `behavior_review`.

Generated internal tasks:

```yaml
- kind: InternalTask
  title: "Behavior/special-care review for anxious boarding inquiry: Milo"
  queue: behavior_or_manager_review
  required_gate: BehaviorReview
- kind: InternalTask
  title: "Collect Milo boarding basics and care instructions"
  queue: front_desk_intake
```

Draft reply pattern:

- Thank the owner and say staff can review Milo's care needs and best-fit options.
- Ask for pet basics, vaccine proof, and written care/feeding/anxiety-handling notes.
- Do not promise isolation, individual care availability, group-play exclusion details, or acceptance.

Expected approval/escalation behavior:

```yaml
status: NeedsHumanReview
approval_required: true
required_review_gates: [BehaviorReview, CustomerMessageApproval]
human_review_reason: "Anxiety and special-handling claims affect care lane and customer wording."
```

### 6. Bite history / aggression-sensitive day play inquiry

Fixture id: `inquiry_bite_history_dayplay`

Sample inbound message/source:

```yaml
source: StaffCreated
source_message_id: staff-note-6001
message_excerpt: "Caller asked about day play for Rex. Caller volunteered that Rex bit another dog at a prior daycare last year but says he is fine now. Wants to know if he can join group play tomorrow."
```

Expected extraction output:

```yaml
extracted_lead:
  customer:
    full_name: null
    preferred_contact: Phone
  pets:
    - name: Rex
      species: Dog
      behavior_claims:
        - prior_bite_history
        - owner_claim_resolved
      missing_info:
        - age
        - sex/spay-neuter
        - vaccine/proof status
        - incident details/evidence if policy requires staff review
        - temperament evaluation status
  request:
    service: DayPlay
    requested_date: tomorrow
    requested_group_play: true
  missing_info:
    - exact requested date after normalization
    - owner identity/contact
    - behavior review records/evaluation path
  risk_flags: [bite_history, behavior_safety_review, manager_review]
```

Expected triage category: `behavior_review`.

Generated internal tasks:

```yaml
- kind: InternalTask
  title: "Urgent behavior/manager review for Rex day-play inquiry"
  queue: behavior_or_manager_review
  required_gates: [BehaviorReview, ManagerApproval]
  due_policy: before any customer eligibility response
- kind: InternalTask
  title: "Collect Rex intake details and vaccine proof"
  queue: front_desk_intake
```

Draft reply pattern:

- Neutral wording: staff will need to review Rex's history and best-fit care options before discussing group play.
- Ask for basic profile/vaccine details and explain a team member will follow up.
- Do not call Rex aggressive in customer copy unless approved and sourced; do not approve or deny group play; do not invite arrival tomorrow as if accepted.

Expected approval/escalation behavior:

```yaml
status: NeedsHumanReview
approval_required: true
required_review_gates: [BehaviorReview, ManagerApproval, CustomerMessageApproval]
human_review_reason: "Prior bite history and group-play request require behavior/manager review before customer-facing eligibility language."
forbidden_actions_absent: [approve group play, deny group play as final, promise evaluation slot, send sensitive behavior message]
```

### 7. Cat boarding with vaccine mentions

Fixture id: `inquiry_cat_boarding_vaccine_mentions`

Sample inbound message/source:

```yaml
source: Portal
source_message_id: provider-lead-7001
message_excerpt: "Looking for cat boarding/condo for Nori Dec 20-27. She's an indoor cat and I have rabies paperwork from our vet. Does she need anything else?"
```

Expected extraction output:

```yaml
extracted_lead:
  customer:
    full_name: null
    preferred_contact: Portal
  pets:
    - name: Nori
      species: Cat
      lifestyle_claims: [indoor_cat]
      vaccine_claims:
        - vaccine: Rabies
          proof_claimed: true
          source_ref: provider-lead-7001
          authoritative: false
      missing_info:
        - age/birthdate if policy requires
        - sex/spay-neuter if policy requires
        - verified vaccine document/status and any cat-specific required vaccines per location policy
  request:
    service: Boarding
    accommodation_preference: cat_condo_or_cat_boarding
    start_date: 2026-12-20
    end_date: 2026-12-27
  missing_info:
    - owner identity/contact if not mapped from portal lead
    - cat vaccine/document review requirements from policy snapshot
    - accommodation availability remains unknown
  risk_flags: [vaccine_review]
```

Expected triage category: `medical_or_vaccine_review` plus cat boarding lead; do not classify as dog group-play.

Generated internal tasks:

```yaml
- kind: InternalTask
  title: "Review cat boarding inquiry for Nori, Dec 20-27"
  queue: front_desk_intake
- kind: InternalTask
  title: "Verify Nori vaccine proof and cat-specific requirements"
  queue: vaccine_or_document_review
  required_gate: MedicalDocumentReview
```

Draft reply pattern:

- Acknowledge cat boarding/condo request and dates.
- Ask for/upload current vaccine proof or say staff will review the provided paperwork and local requirements.
- Do not approve rabies proof, decide that no other vaccines are needed, promise a cat condo, or mention dog daycare/group play.

Expected approval/escalation behavior:

```yaml
status: NeedsHumanReview
approval_required: true
required_review_gates: [MedicalDocumentReview, CustomerMessageApproval]
human_review_reason: "Vaccine requirement interpretation and customer reply require review."
forbidden_actions_absent: [approve vaccine, waive vaccine, promise accommodation availability, confirm booking]
```

### 8. Grooming-only inquiry

Fixture id: `inquiry_grooming_only`

Sample inbound message/source:

```yaml
source: Email
source_message_id: email-8001
message_excerpt: "Can I book a bath and nail trim for Bella next Friday? She's a 35 lb Australian shepherd mix. We are not boarding, just grooming."
```

Expected extraction output:

```yaml
extracted_lead:
  customer:
    full_name: null
    preferred_contact: Email
    contact_present: true
  pets:
    - name: Bella
      species: Dog
      breed_or_coat_hint: "Australian shepherd mix"
      weight: "35 lb"
      missing_info:
        - age if grooming policy requires
        - vaccine/proof status if grooming policy requires
        - behavior/handling notes if location requires for grooming
  request:
    service: Grooming
    requested_services: [bath, nail_trim]
    requested_date: next Friday
    lodging_requested: false
  missing_info:
    - owner full name
    - exact date after local timezone normalization
    - grooming appointment time preference
    - coat/matting/handling notes if required by groomer policy
  risk_flags: []
```

Expected triage category: `grooming_lead`.

Generated internal tasks:

```yaml
- kind: InternalTask
  title: "Grooming-only lead for Bella: bath and nail trim"
  queue: grooming_intake
  checklist: [owner name, exact date/time preference, grooming details, policy-required vaccine/handling info]
```

Draft reply pattern:

- Acknowledge grooming-only request and requested services.
- Ask for owner name, exact date/time preference, and any grooming-specific details required by policy.
- Do not create boarding/daycare tasks unless later evidence asks for them; do not promise appointment availability or pricing unless sourced from approved policy.

Expected approval/escalation behavior:

```yaml
status: NeedsMoreInformation
approval_required: true
required_review_gates: [CustomerMessageApproval]
forbidden_actions_absent: [confirm appointment, promise price, create lodging reservation]
```

### 9. Unsupported service edge: in-home pet sitting

Fixture id: `inquiry_unsupported_service_pet_sitting`

Sample inbound message/source:

```yaml
source: WebsiteForm
source_message_id: web-lead-9001
message_excerpt: "Do you have someone who can come to my house twice a day to feed my cat while I'm away?"
```

Expected extraction output:

```yaml
extracted_lead:
  customer:
    full_name: null
    preferred_contact: null
  pets:
    - name: null
      species: Cat
  request:
    requested_service_raw: in_home_pet_sitting
    service: null
    unsupported_service_candidate: true
  missing_info:
    - owner contact if staff needs to reply
    - dates only if staff chooses to discuss alternatives
  risk_flags: [unsupported_service]
```

Expected triage category: `service_scope_review`.

Generated internal tasks:

```yaml
- kind: InternalTask
  title: "Review unsupported in-home pet-sitting inquiry"
  queue: front_desk_or_manager_review
  required_gate: ManagerApproval
  checklist: [decide approved response, possible alternative cat boarding info if policy allows]
```

Draft reply pattern:

- If policy has approved wording, draft a polite response that staff can confirm service options and alternatives.
- Do not invent an in-home service, recommend unapproved third parties, or issue a final refusal/alternative offer without staff review.

Expected approval/escalation behavior:

```yaml
status: NeedsHumanReview
approval_required: true
required_review_gates: [ManagerApproval, CustomerMessageApproval]
human_review_reason: "Requested service appears outside current modeled service catalog; response/referral policy requires staff/manager review."
```

## Golden assertion checklist

Every scenario fixture should assert:

- `event_payload.type == InquiryReceived` and source/evidence refs are preserved.
- `structured_output.extracted_lead` separates customer, pets, request, missing info, vaccine/medical/behavior claims, risk flags, and source refs.
- Customer claims about vaccines, health, behavior, medication, pricing, availability, and prior approvals remain non-authoritative evidence until reviewed.
- Multi-pet cases keep per-pet missing info and review gates separate.
- Cat cases do not enter dog group-play assumptions.
- Grooming-only cases do not create lodging/daycare reservations by default.
- Unsupported-service cases route to staff/manager review instead of inventing service coverage.
- `recommended_actions` contain only internal tasks, review requests, and draft messages allowed by the policy packet.
- Any customer reply is draft-only and requires `CustomerMessageApproval`; sensitive replies also carry `BehaviorReview`, `MedicalDocumentReview`, and/or `ManagerApproval` as appropriate.
- No output confirms booking/appointment, promises availability/price/room, approves vaccines, decides group-play eligibility, sends a customer message, mutates providers, or writes final customer/pet/reservation truth from ambiguous evidence.
- Safe logs include ids, categories, workflow name, policy snapshot, source refs, status, and review gates; they exclude raw message bodies, exact contact values, vaccine document/OCR text, medication details, behavior/incident narrative, and full draft reply text.

## Suggested physical fixture files

When implementation begins, create YAML scenarios under:

```text
fixtures/ai-runtime/scenarios/inquiry-intake/
  inquiry_complete_boarding.yaml
  inquiry_vague_sms.yaml
  inquiry_multiple_pets_mixed_needs.yaml
  inquiry_missing_dates_daycare.yaml
  inquiry_anxiety_boarding.yaml
  inquiry_bite_history_dayplay.yaml
  inquiry_cat_boarding_vaccine_mentions.yaml
  inquiry_grooming_only.yaml
  inquiry_unsupported_service_pet_sitting.yaml
```

Keep fake runtime response JSON separate from scenario expectations, for example:

```text
fixtures/ai-runtime/responses/inquiry-intake/<scenario-id>.valid.json
fixtures/ai-runtime/responses/inquiry-intake/<scenario-id>.policy_violation.json
```

The implementation acceptance bar is that each YAML fixture can drive a deterministic fake-runtime contract test without network access, live Hermes, live Gingr, a real LLM, or production customer-message sends.
