# Inquiry intake draft reply templates

Purpose: define safe draft reply patterns for the `inquiry-intake` / `lead-conversion` workflow. These templates are source-grounded drafting guidance only. They do not approve autonomous customer-facing sends, booking/provider mutations, waitlist placement, medical/vaccine decisions, payment actions, or policy exceptions.

Status: draft template catalog for downstream workflow and review cards. Templates stay `QueueForReview` / `DraftOnly` unless final product, security, and operations review approves narrower deterministic auto-send boundaries for exact templates, facts, recipients, channels, consent, suppression rules, idempotency, and audit.

## Source anchors

- `docs/workflows/inquiry-intake-inputs.md` — canonical inquiry input constraints, channel mapping, allowed actions, forbidden actions, and customer-message boundaries.
- `docs/workflows/customer-messaging-parts/inputs.md` — customer messaging style, channel, approval, consent, sensitive-topic, and runtime constraints.
- `docs/architecture/agent-permissions-by-workflow.md` — intake and messaging permission matrix, automation labels, review gates, and redaction rules.
- `docs/architecture/agent-prompt-packet.md` — structured prompt packet, allowed/forbidden actions, verification, escalation, and customer-message draft validation.
- `domain/src/agents.rs`, `domain/src/workflow.rs`, `domain/src/tools.rs`, `domain/src/policy.rs` — baseline agent specs, `AllowedAction::DraftCustomerMessage`, `WorkflowResult`, `RecommendedAction::DraftMessage`, `DeliveryChannel`, and review-gate vocabulary.

## Global drafting rules

### Tone

Use a warm, pet-parent-friendly, operationally clear tone:

- Friendly and helpful, not salesy or pressuring.
- Concise enough for staff to review quickly.
- Truthful about what is known, unknown, and still under review.
- Reassuring without promising space, eligibility, pricing, timing, or outcome.
- Neutral for behavior, medical, vaccine, or special-care topics: no diagnosis, blame, labels, or implied final decisions.

### Required variable placeholders

Use stable placeholders in template text rather than live values in this catalog:

- `{{customer_first_name}}`
- `{{pet_name}}`
- `{{pet_names}}`
- `{{service_label}}` — boarding, daycare/day play, day boarding, grooming/bathing, training, DaySpa, etc.
- `{{requested_date_or_range}}`
- `{{location_name}}`
- `{{staff_contact_label}}` — e.g. front desk, reservations team, care team.
- `{{missing_items_list}}`
- `{{vaccine_items_list}}`
- `{{document_upload_link_or_instructions}}`
- `{{availability_review_window}}` — e.g. "after our team reviews the request"; do not invent SLA.
- `{{waitlist_context}}` — human-approved factual context only, not a promise.
- `{{special_care_topics}}` — customer-safe summary like "medication details" or "extra handling notes".
- `{{unsupported_service_label}}`
- `{{safe_alternative_or_referral}}` — only if approved by policy/staff.

If a placeholder is missing, stale, contradictory, or not approved for customer copy, the draft should either omit that sentence or return `NeedsMoreInformation` / `NeedsHumanReview` instead of inventing content.

### Required draft metadata

Every generated draft should carry metadata for review and idempotency:

- `template_id` from this document.
- `template_version` / policy snapshot version.
- `channel` and `recipient_ref`, not raw contact value unless the review UI requires it.
- `evidence_refs` for each factual claim.
- `review_gates`, at least `CustomerMessageApproval`; add `ManagerApproval`, `MedicalDocumentReview`, or `BehaviorReview` when triggered.
- `automation_level`: default `QueueForReview`; use `DraftOnly` / `NeverAutoSend` for sensitive or ambiguous topics.
- `forbidden_claims_checked`: availability, booking confirmation, vaccine clearance, eligibility, payment, policy exception, provider write, send action.
- `idempotency_key` including subject/event/source/template/policy version.

## Channel-specific notes

### SMS

- Use the shortest safe version, usually 1-3 concise sentences.
- Do not include long lists of medical/vaccine details if they can be moved to email/portal instructions.
- Include a human review path before send. Channel availability is not consent.
- Avoid sensitive specifics when a shared phone could expose medical, behavior, incident, or payment details.
- Do not silently switch to SMS after email failure unless contact policy and approval allow it.

### Email

- Use a clear subject line and structured body.
- Better for multiple missing items, vaccine document instructions, and special-care follow-up.
- Keep factual claims tied to approved evidence; do not include internal notes, raw OCR, raw customer free text, or provider payload details.
- Attachments/links must come from approved document upload or portal instructions.

### Chat / web widget

- Current repo model does not yet define a distinct chat source/channel enum. Treat chat copy as provisional until the product model chooses `ChatWidget`/`WebChat` or a deliberate mapping.
- Use brief conversational copy and avoid collecting high-risk details directly in chat unless the UX has consent, retention, and escalation policy.
- If special care, behavior/aggression, medical, vaccine, payment, or urgent safety content appears, draft a handoff to staff review rather than continuing automated chat.

## What must never be promised automatically

No inquiry reply draft may automatically promise, imply, or execute:

- Confirmed booking, held space, room/run assignment, check-in/check-out action, cancellation, rejection, or provider status change.
- Availability or waitlist movement/release.
- Vaccine, medical, medication, allergy, or document approval.
- Daycare/group-play/playgroup eligibility, behavior clearance, or safety decision.
- Payment/deposit collection, refund, waiver, discount, credit, forfeiture, or pricing exception.
- Policy exception, manager approval, special accommodation approval, or guaranteed outcome.
- Exact response deadline unless sourced from an approved location policy.
- Customer message send, mark-as-sent state, or contact attempt before an approved send path executes.

## Approval and send policy assumptions

Default policy before final review:

- Templates are drafts, not sends.
- Low-risk routine replies may be queued for staff review (`QueueForReview`) when all facts are present and non-sensitive.
- Sensitive, ambiguous, or denial/refusal-adjacent replies are `DraftOnly` or `NeverAutoSend` and need explicit staff/manager approval of final text.
- Any outbound send requires a separate approved action with recipient/channel, exact payload/version, approval actor, policy version, opt-out/suppression check, provider response refs, and audit event.
- Deterministic auto-send can be considered only later for narrow receipt-only or missing-info classes with fixed text and verified facts. AI-authored free text is not auto-sendable by default.

## Behavior/aggression flag handling

Inquiry content may mention biting, aggression, reactivity, escape attempts, anxiety, resource guarding, fights, muzzle use, separation anxiety, special handling, or prior daycare rejection. Handle these as routing signals, not customer-facing labels.

Rules:

- Flag `BehaviorReview` when the inquiry mentions behavior/aggression, group-play/daycare eligibility, safety, bite/fight history, escape risk, staff handling constraints, or conflicting temperament claims.
- Customer copy should use neutral language such as "additional care/handling details" or "a few follow-up questions so our team can review the best fit" unless staff has approved more specific wording.
- Do not say the pet is aggressive, unsafe, banned, cleared, approved for group play, or ineligible unless an authorized reviewer has supplied approved wording.
- Do not promise private play, special handling, medication administration, or one-on-one care availability automatically.
- If there is urgent safety or incident content, suppress customer draft auto-send and create/recommend internal review/escalation instead.

## Template pattern: missing info

Template id: `inquiry_missing_info_v1`

Use when an inquiry lacks required intake facts such as pet name/species, dates, service, contact preference, pickup/drop-off window, care notes, or required document reference. Do not use when the missing fact is a sensitive decision that needs manager/medical/behavior review before asking the customer.

Automation level: `QueueForReview` by default; `DraftOnly` if missing items include medical, medication, allergy, behavior, incident, payment, policy exception, or complaint details.

Review gates:

- Always: `CustomerMessageApproval`.
- Add `ManagerApproval` for policy exception, complaint, refusal, or special accommodation language.
- Add `BehaviorReview` for behavior/aggression/group-play topics.
- Add `MedicalDocumentReview` for vaccine/medical-document ambiguity.

SMS draft:

> Hi {{customer_first_name}}, thanks for reaching out to {{location_name}} about {{service_label}} for {{pet_name}}. To help our team review the request, could you send {{missing_items_list}}? We’ll take a look once we have those details.

Email draft:

Subject: A few details needed for {{pet_name}}’s {{service_label}} request

> Hi {{customer_first_name}},
>
> Thanks for reaching out to {{location_name}} about {{service_label}} for {{pet_name}} on {{requested_date_or_range}}.
>
> To help our team review the request, could you please send the following details?
>
> - {{missing_items_list}}
>
> Once we have those details, our team can review the request and follow up with next steps.
>
> Thank you,
> {{staff_contact_label}}

Chat draft:

> Thanks! To help our team review {{pet_name}}’s {{service_label}} request, we still need {{missing_items_list}}. You can share that here, and we’ll route it for staff review.

Never include:

- "You’re all set," "we have space," "your reservation is booked," or "approved."
- Speculative policy, price, or timing details.

## Template pattern: vaccine request

Template id: `inquiry_vaccine_request_v1`

Use when the intake packet indicates vaccine proof is missing or clearer documentation is needed. The draft may request documents; it must not decide whether vaccines are valid, complete, expired, waived, or sufficient.

Automation level: `QueueForReview` for simple missing proof; `DraftOnly` when OCR/document evidence is ambiguous, eligibility/check-in is affected, or policy-specific denial language would be needed.

Review gates:

- Always: `CustomerMessageApproval`.
- Add `MedicalDocumentReview` when vaccine records/documents are interpreted, unclear, expired/stale, mismatched, or eligibility-affecting.
- Add `ManagerApproval` if a waiver/exception, refusal, or urgent check-in problem is involved.

SMS draft:

> Hi {{customer_first_name}}, thanks for your inquiry for {{pet_name}}. Before our team can finish reviewing the {{service_label}} request, we still need vaccine documentation for {{vaccine_items_list}}. You can send it using {{document_upload_link_or_instructions}}.

Email draft:

Subject: Vaccine documentation needed for {{pet_name}}

> Hi {{customer_first_name}},
>
> Thank you for your {{service_label}} inquiry for {{pet_name}}.
>
> Our team still needs vaccine documentation for:
>
> - {{vaccine_items_list}}
>
> Please upload or send the documentation here: {{document_upload_link_or_instructions}}.
>
> Once it is received, it will be reviewed by the appropriate team member. This message is only a request for documentation and does not confirm vaccine approval or booking status.
>
> Thank you,
> {{staff_contact_label}}

Chat draft:

> We still need vaccine documentation for {{vaccine_items_list}} before staff can finish reviewing {{pet_name}}’s request. Please upload/send it using {{document_upload_link_or_instructions}}.

Never include:

- "Vaccines are approved," "cleared for daycare/boarding," "accepted," or "waived."
- Medical interpretation from OCR or customer text without medical/document review.

## Template pattern: availability pending

Template id: `inquiry_availability_pending_v1`

Use when an inquiry has enough basic details to route to availability/capacity/staff review but availability has not been approved or is stale/unknown. This pattern acknowledges receipt and sets a review expectation without promising space.

Automation level: `QueueForReview`; `DraftOnly` for peak/holiday, capacity exception, special care, behavior, medical, or policy-exception cases.

Review gates:

- Always: `CustomerMessageApproval`.
- Add `ManagerApproval` for capacity exceptions, peak/holiday constraints, overbooking risk, policy exceptions, or special accommodations.
- Add `BehaviorReview` / `MedicalDocumentReview` as triggered by pet facts.

SMS draft:

> Hi {{customer_first_name}}, thanks for the {{service_label}} request for {{pet_name}} on {{requested_date_or_range}}. Our team has the request and will review availability and any needed next steps before following up.

Email draft:

Subject: We received {{pet_name}}’s {{service_label}} request

> Hi {{customer_first_name}},
>
> Thanks for sending {{pet_name}}’s {{service_label}} request for {{requested_date_or_range}}.
>
> Our team will review availability and the details provided, then follow up with next steps. This message confirms we received the request; it does not confirm a booking or hold space.
>
> Thank you,
> {{staff_contact_label}}

Chat draft:

> Thanks — we received the {{service_label}} request for {{pet_name}} on {{requested_date_or_range}}. Staff will review availability and follow up with next steps. This does not confirm the booking yet.

Never include:

- "We have availability," "space is held," "confirmed," "guaranteed," or room/capacity specifics unless approved by the booking workflow.

## Template pattern: waitlist

Template id: `inquiry_waitlist_v1`

Use only when a human-approved or deterministic booking/operations fact says the request should be treated as waitlist-related. The inquiry agent must not place a pet on a waitlist, remove from waitlist, release a spot, or promise movement. If the waitlist state is merely suggested by the model or stale capacity, use availability-pending or manager review instead.

Automation level: `DraftOnly` by default; possible `QueueForReview` only after a verified waitlist recommendation/status and approved wording exist. Not auto-sendable before final review.

Review gates:

- Always: `CustomerMessageApproval`.
- Add `ManagerApproval` when waitlist placement/release, denial, peak/holiday policy, capacity exception, or customer disappointment/complaint language is involved.
- Add `BehaviorReview` / `MedicalDocumentReview` if waitlist status depends on eligibility, behavior, or vaccine readiness.

SMS draft:

> Hi {{customer_first_name}}, thanks for your {{service_label}} request for {{pet_name}} on {{requested_date_or_range}}. Our team needs to review the request in relation to current availability/waitlist status and will follow up with next steps.

Email draft:

Subject: Update needed for {{pet_name}}’s {{service_label}} request

> Hi {{customer_first_name}},
>
> Thank you for your {{service_label}} request for {{pet_name}} on {{requested_date_or_range}}.
>
> Our team needs to review the request against current availability and any waitlist information before we can provide next steps. {{waitlist_context}}
>
> We’ll follow up once staff has reviewed it. This message does not confirm a reservation, hold a space, or guarantee waitlist movement.
>
> Thank you,
> {{staff_contact_label}}

Chat draft:

> Thanks for checking. Staff needs to review {{pet_name}}’s request against current availability/waitlist information before giving next steps. This does not confirm a space or waitlist movement yet.

Never include:

- "You are on the waitlist" unless the source evidence says so and staff-approved language allows it.
- "A spot will open," "you are next," "we can fit you in," or denial/refusal language without manager-approved policy copy.

## Template pattern: special-care follow-up

Template id: `inquiry_special_care_followup_v1`

Use when the inquiry mentions medication, allergies, feeding complexity, mobility support, anxiety, separation issues, senior care, medical conditions, behavior/aggression, escape risk, individual play, extra handling, or other special-care topics. This template asks for more information and routes staff review; it does not approve care, diagnose, or promise accommodation.

Automation level: `DraftOnly` by default because special-care content often triggers medical, behavior, safety, or manager review. Low-sensitivity extra detail requests may later be `QueueForReview` if final policy permits.

Review gates:

- Always: `CustomerMessageApproval`.
- Add `ManagerApproval` for special accommodations, safety risk, service eligibility, staff/capacity implications, or policy exceptions.
- Add `MedicalDocumentReview` for medication, allergy, medical condition, vaccine/document, or veterinary proof topics.
- Add `BehaviorReview` for aggression, bite/fight history, reactivity, group-play/daycare eligibility, escape risk, or handling constraints.

SMS draft:

> Hi {{customer_first_name}}, thanks for sharing those details about {{pet_name}}. To help our team review the best fit for {{service_label}}, could you send a little more information about {{special_care_topics}}? Staff will review before confirming any next steps.

Email draft:

Subject: Follow-up details for {{pet_name}}’s care needs

> Hi {{customer_first_name}},
>
> Thank you for sharing information about {{pet_name}} and the {{service_label}} request for {{requested_date_or_range}}.
>
> To help our team review the best fit and any care considerations, could you please provide more detail about:
>
> - {{special_care_topics}}
>
> Our team will review the information before confirming next steps. This message does not approve a service, accommodation, medication plan, or group-play/daycare eligibility.
>
> Thank you,
> {{staff_contact_label}}

Chat draft:

> Thanks for letting us know. Because {{pet_name}} may need a little extra review for {{special_care_topics}}, staff should look at the details before confirming next steps. Could you share the key information here, or would you prefer a team member to follow up?

Behavior/aggression-safe variant:

> Thanks for sharing that context about {{pet_name}}. To help our team review the safest fit, could you provide more details about {{special_care_topics}}? Staff will review before confirming any daycare, group-play, or boarding next steps.

Never include:

- "We can accommodate," "approved for group play," "not aggressive," "safe," "cleared," "banned," or diagnosis/treatment advice.
- Medication administration promises or medical care instructions.

## Template pattern: unsupported service

Template id: `inquiry_unsupported_service_v1`

Use when the requested service is outside the location’s supported service catalog or outside the product’s current scope. The draft must be based on a trusted service catalog/policy snapshot. If the unsupported status is uncertain, route to staff review instead.

Automation level: `DraftOnly` by default because this is denial/refusal-adjacent and location-specific. May become `QueueForReview` for simple factual redirects after policy approval. Not auto-sendable before final review.

Review gates:

- Always: `CustomerMessageApproval`.
- Add `ManagerApproval` when the message denies service, mentions policy exceptions, addresses complaints, or could affect customer relationship.
- Add `BehaviorReview` / `MedicalDocumentReview` if unsupported status depends on behavior, medical/vaccine, or safety eligibility rather than service catalog.

SMS draft:

> Hi {{customer_first_name}}, thanks for reaching out to {{location_name}}. It looks like {{unsupported_service_label}} may not be a service we can offer through this inquiry path. Our team can review and follow up with appropriate next steps{{safe_alternative_or_referral}}.

Email draft:

Subject: Follow-up on your request for {{unsupported_service_label}}

> Hi {{customer_first_name}},
>
> Thanks for contacting {{location_name}} about {{unsupported_service_label}}.
>
> Based on the service information available to this workflow, that request may be outside the services we can offer through this inquiry path. Our team should review the details and follow up with appropriate next steps{{safe_alternative_or_referral}}.
>
> Thank you,
> {{staff_contact_label}}

Chat draft:

> Thanks for asking about {{unsupported_service_label}}. That may be outside what this inquiry path supports, so staff should review and follow up with the best next step{{safe_alternative_or_referral}}.

Never include:

- Final refusal, legal/policy language, pricing, competitor claims, or referral promises unless approved by location policy/staff.
- Eligibility denial based on behavior, vaccine, medical, or safety facts without the appropriate review gate.

## Suggested structured output shape

Downstream implementation can represent these templates as a typed draft packet rather than free-form copy:

```yaml
template_id: inquiry_missing_info_v1
template_version: "2026-06-11"
intent: missing_info_request
channel: Email
recipient_ref: contact:cust_123:email:primary
automation_level: QueueForReview
review_gates:
  - CustomerMessageApproval
subject: "A few details needed for {{pet_name}}’s {{service_label}} request"
body: "<rendered draft body>"
placeholders_used:
  customer_first_name: evidence:customer:cust_123:v4
  pet_name: evidence:pet:pet_456:v2
  missing_items_list: evidence:intake_gap:evt_789
forbidden_claims_checked:
  booking_confirmation: true
  availability_promise: true
  vaccine_clearance: true
  behavior_eligibility: true
  payment_or_policy_exception: true
evidence_refs:
  - evidence:inquiry:evt_789:redacted
  - policy:loc_123:message_tone:2026-06-01
human_review_reason: "Customer-facing draft requires approval before send."
idempotency_key: customer-message-draft:inquiry:evt_789:inquiry_missing_info_v1:policy_2026-06-01
```

## Final review questions before any auto-send boundary

Before any of these patterns can move beyond draft/review, final review must answer:

1. Which exact template ids, if any, are eligible for deterministic auto-send?
2. Which fields/facts must be present, fresh, trusted, and validator-checked?
3. Which channels are eligible, and how are consent, opt-out, quiet hours, failed delivery, and suppression represented?
4. Who may approve template text, policy changes, recipient/channel selection, and send execution?
5. What risk flags force suppression or human review?
6. How are duplicate inquiries, replays, and customer replies deduped so customers are not spammed?
7. How are chat-widget source/channel semantics represented in the Rust/domain model?
8. How are audit logs redacted while preserving enough evidence for operations review?
