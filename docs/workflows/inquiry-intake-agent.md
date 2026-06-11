# Inquiry intake agent workflow

Purpose: define the integrated workflow contract for the `inquiry-intake` agent. This document synthesizes the completed source-normalization, extraction-schema, triage-routing, draft-reply, internal-task, and test-fixture handoffs into the implementation artifact for `inquiry.received` / `WorkflowEventType::InquiryReceived`.

Status: integrated draft. This document is not approval for production LLM use, customer-facing auto-sends, booking/provider mutations, payment actions, vaccine/medical/behavior decisions, group-play eligibility, or policy exceptions. It defines the conservative packet, output, task, draft, review, and fixture contracts that implementation and review cards should encode.

## Source artifacts synthesized

Primary inquiry-intake parts:

- `docs/workflows/inquiry-intake-agent-parts/input-sources-and-normalization.md`
- `docs/workflows/inquiry-intake-agent-parts/extraction-schema.md`
- `docs/workflows/inquiry-intake-parts/triage-routing.md`
- `docs/workflows/inquiry-intake-reply-templates.md`
- `docs/workflows/inquiry-intake-internal-tasks.md`
- `docs/workflows/inquiry-intake-test-cases-fixtures.md`

Integration anchors:

- `docs/workflows/inquiry-intake-inputs.md`
- `docs/workflows/staff-operations-parts/inputs.md`
- `docs/architecture/pet-resort-workflow-events.md`
- `docs/architecture/pet-resort-data-model.md`
- `docs/architecture/pet-resort-ai-runtime.md`
- `docs/architecture/agent-permissions-by-workflow.md`
- `docs/architecture/agent-prompt-packet.md`
- `docs/architecture/pet-resort-ai-runtime-structured-output.md`
- `docs/architecture/ai-runtime-memory-context-policy.md`
- `docs/workflows/workflow-event-idempotency-replay.md`
- `domain/src/entities.rs`, `domain/src/workflow.rs`, `domain/src/agents.rs`, `domain/src/tools.rs`, `domain/src/operations.rs`, and `domain/src/policy.rs` as current Rust/domain anchors.

## 1. Scope and assumptions

### In scope

The inquiry-intake agent receives a normalized `inquiry.received` workflow event and may:

- Extract source-backed owner, contact, pet, requested service, requested dates/times, special-care, temperament, vaccine-mention, urgency, missing-info, ambiguity, conflict, and risk fields.
- Classify the inquiry into a conservative triage route.
- Recommend internal staff/review task drafts.
- Draft customer follow-up copy under `CustomerMessageApproval`.
- Produce source citations, idempotency keys, safe audit details, redaction notes, and human-review reasons.
- Recommend candidate-only mappings to lead/customer/pet/reservation/task/event records.

### Out of scope / forbidden

The inquiry-intake agent must not:

- Confirm, offer, reject, cancel, waitlist, or mutate a reservation.
- Promise availability, hold space, assign rooms/runs, quote price/deposit/payment instructions, waive/refund/discount, or move waitlists.
- Approve vaccine, medical, medication, allergy, behavior, group-play, safety, or service eligibility.
- Create or merge ambiguous customer/pet records as canonical truth.
- Write live Gingr/provider state or send customer messages.
- Treat customer free text, transcripts, screenshots, attachments, provider payloads, or AI confidence as authoritative policy/medical/payment/behavior truth.

A `Completed` result means only that the agent produced a validated internal recommendation/draft packet. It does not mean any underlying booking, vaccine, care, behavior, provider, payment, or customer-message action is approved.

### Product-map assumption

The requested canonical product-map artifact `docs/product/pet-resort-product-map.md` is not present in this repo. Until it is restored or synthesized, use `docs/workflows/staff-operations-parts/inputs.md` and `docs/workflows/inquiry-intake-inputs.md` as the product-scope anchors. Preserve this as a known gap in implementation tickets rather than inventing location policy.

Current product shape assumed by the source docs:

- Internal staff/manager workflow and operations tool for a single pet resort or small resort group.
- Core services: dog boarding, cat boarding, dog daycare/day play, day boarding/individual play, grooming/bathing/DaySpa, and training where offered.
- MVP emphasis: intake/missing-info handling, records, vaccine/document review queues, staff tasking, capacity/availability snapshots, audit events, and draft messages/summaries.
- V1 posture: manual/review-gated for final booking, vaccine, group-play/behavior, payment, provider write, and high-risk customer-message actions.

## 2. `inquiry.received` normalization

All inbound inquiry evidence must normalize into one semantic event:

```text
WorkflowEventType::InquiryReceived / inquiry.received
```

The event means evidence of interest, missing intake information, or lead follow-up was received. It does not mean a reservation exists, availability is known, a booking can be promised, a message can be sent, or provider state can be mutated.

### Normalization layers

1. Source boundary record:
   - Accept raw form/SMS/email/call/chat/provider/staff evidence into boundary storage.
   - Preserve source metadata, raw payload refs, hashes, redaction profile, transcript/document refs, verification state, and quarantine status.
2. Canonical inquiry evidence:
   - Extract minimized, typed, source-specific fields with provenance, idempotency, contact/consent metadata, attachments/transcript refs, and uncertainty.
3. Semantic workflow event:
   - Create/update one `inquiry.received` event/prompt packet with minimized payload, policy context, review gates, source refs, and audit refs.

### Required event envelope

Every normalized inquiry event should include:

| Field | Requirement |
| --- | --- |
| `event_id` | Stable workflow event id after source-event dedupe. |
| `event_type` | `InquiryReceived` in Rust and `inquiry.received` in external/docs vocabulary. |
| `occurred_at` | Source timestamp when reliable; otherwise receiver timestamp with uncertainty. |
| `source_kind` | `website_form`, `sms`, `email`, `phone_transcript`, `chat_widget`, `customer_portal`, `provider_webhook`, `provider_poll`, or `staff_created`. |
| `actor` | Customer/owner for self-submitted evidence; staff for manual entry/call summary; system for verified provider imports. |
| `location_id` | Required for policy, capacity, contact, and timezone scope. Unknown location routes to missing-info/staff review. |
| `subject` | `WorkflowSubject::Customer(customer_id)` when deterministically matched; otherwise `WorkflowSubject::External { provider, id }`. |
| `related_ids` | Customer, pet, reservation/request, provider lead, source message, attachment, transcript, audit, document, and policy refs when present. |
| `policy_context` | Allowed actions, automation level, required review gates, contact-permission refs, redaction/context policy refs, and location intake policy snapshot. |
| `payload` | Minimized `InquiryReceivedPayload` below. Raw bodies stay referenced, not copied wholesale. |
| `result` | `WorkflowResult<InquiryIntakeOutput>` after the agent runs. |

### Required normalized payload

The direct event payload should include:

- Identity/provenance: `source_kind`, `source_system`, `source_record_id`, optional delivery id, source version, timestamps, source confidence, source evidence refs, raw storage ref, canonical source fingerprint.
- Location: `location_id`, source-provided site/campaign, location match confidence, unresolved-location reason.
- Customer/lead: optional `customer_id`, external lead id, supplied name, supplied contact refs, identity match confidence, duplicate/conflict markers.
- Contact channel: inbound channel, requested/preferred channel, reply-to/destination ref, consent/opt-in/opt-out/quiet-hours state if known, and explicit `consent_unknown` when not modeled.
- Pet summary: optional pet ids, supplied names/species/breed/age/sex/spay-neuter/size, pet count, missing-pet-info checklist, no promotion of medical/vaccine truth from free text.
- Request intent: requested services, dates/times, add-ons, flexibility notes, campaign/referrer, unsupported/ambiguous service markers.
- Message/excerpt: minimal redacted excerpt or summary, language/locale, urgency/sentiment only as non-authoritative hints.
- Attachments/transcripts: refs, metadata, scan/extraction/redaction status, prompt-use approval state.
- Review/risk: review gates, sensitive categories, missing/ambiguous/conflicting fields, policy-blocked actions, risk flags.
- Audit: ingest actor/system, normalizer version, redaction profile/version, policy snapshot, prompt-field manifest, data-fetch manifest, audit refs.

### Source-specific handling

| Source | Current anchor | Normalized handling |
| --- | --- | --- |
| Website form | `ReservationSource::WebsiteForm` | Preserve form id/version, landing/referrer/campaign, submitted/received timestamps, location, service/date/pet/contact fields, consent checkbox text/version, attachment refs, spam/bot signals. Actor is customer unless staff/kiosk path. |
| SMS | `ReservationSource::Sms`, `ContactChannel::Sms` | Preserve provider message id/thread, inbound/destination numbers as refs, timestamps, opt-out/provider suppression, MMS refs, minimal body excerpt. Channel availability is not legal permission to send. |
| Email | `ReservationSource::Email`, `ContactChannel::Email` | Preserve message/thread ids, mailbox, from/to/reply-to refs, subject, timestamps, security status, attachment refs, sanitized body refs. Do not pass whole inbox/thread history. |
| Phone transcript | `ReservationSource::PhoneTranscript`, `ContactChannel::Phone` | Preserve call/transcript ids, provider, direction, numbers as refs, timestamps, transcript/recording refs, diarization confidence, staff notes, consent metadata. Low confidence or ambiguous speaker attribution routes to review. |
| Chat widget | current enum gap | Required product source, but no current `ReservationSource`, `ContactChannel`, or `DeliveryChannel` variant exists. Preserve `source_kind=chat_widget`; if forced through `WebsiteForm` for storage compatibility, keep explicit semantic metadata and `chat_source_unmapped` flag. |
| Customer portal/provider lead | `ReservationSource::Portal(PortalProvider)`, `PortalProvider::Gingr` | Provider/browser lead signals map to `inquiry.received` only after signature/verification, source normalization, identity reconciliation, and raw-payload quarantine. |
| Staff-created/manual | `ReservationSource::StaffCreated` | Preserve staff actor and original channel/source refs if known; manual entry must not erase customer consent uncertainty. |

### Idempotency

Recommended source key:

```text
source_event_key = v1:{location_id}:InquiryReceived:{primary_subject}:{source_kind}:{source_fingerprint}
```

Where `primary_subject` is `customer:{customer_id}` when matched, otherwise `external:{source_system}:{source_record_id}`. Prefer upstream IDs such as form submission id, SMS provider id, email message id, call/transcript id, chat session/message id, verified webhook delivery id, or provider lead id. If none exist, fingerprint the minimal verified tuple: location + source kind + normalized contact ref hash + source timestamp bucket + service/date intent + body/transcript hash.

Separate side-effect keys:

```text
internal_task = v1:{location_id}:internal_task:inquiry_follow_up:{customer_or_external}:{semantic_reason}:{policy_version}
draft = v1:{location_id}:draft_customer_message:inquiry_follow_up:{customer_or_external}:{evidence_set_hash}:{template_or_copy_version}:{policy_version}
send = v1:{location_id}:outbound_customer_message:{approved_draft_id}:{approved_draft_version}:{recipient_id}:{channel}:inquiry_follow_up:{approval_id}
```

Draft creation and outbound send are separate. Replays may rebuild summaries, missing-info checklists, task recommendations, and drafts; they must not send customer messages without a separate approved send path.

## 3. Runtime integration and prompt packet

Production default should follow the app-owned durable queue architecture from `docs/architecture/pet-resort-ai-runtime.md`:

```text
source evidence / verified provider event / staff action
  -> app/integration adapter authenticates and normalizes
  -> app writes WorkflowEvent + idempotency key + policy snapshot + queue record
  -> worker claims queue item
  -> context builder constructs AgentPromptPacket<InquiryIntakeInput>
  -> AgentRuntime invokes Hermes/AI
  -> parser/validator returns WorkflowResult<InquiryIntakeOutput> or FailedSafely
  -> app persists result, audit, task/message drafts, and review requests
  -> deterministic policy + human approvals decide side effects
```

### Minimum prompt packet

```text
InquiryIntakePromptPacket {
  workflow_name: "inquiry-intake",
  workflow_version,
  event: WorkflowEvent { event_id, InquiryReceived, occurred_at, actor, location_id, subject, policy_context },
  source: {
    source_kind,
    source_system,
    source_record_id,
    source_event_key,
    source_submitted_at,
    source_received_at,
    evidence_refs,
    redaction_profile,
    confidence,
  },
  normalized_payload: {
    customer_or_external_ref,
    supplied_contact_refs_minimized,
    contact_channel_metadata,
    requested_services,
    requested_date_window,
    pet_summary_minimized,
    missing_field_candidates,
    redacted_message_excerpt_or_summary,
    attachment_or_transcript_refs,
    risk_or_review_flags,
  },
  policy_and_permissions: {
    allowed_actions: [ReadEntities, ExtractStructuredData, CreateInternalTask, DraftCustomerMessage?, FlagRisk],
    forbidden_actions,
    review_gates,
    contact_permission_policy_ref,
    location_intake_policy_ref,
  },
  output_contract: "WorkflowResult<InquiryIntakeOutput>",
}
```

Fetch-by-ID may retrieve only approved lead/customer/pet/request/policy snippets: customer/lead record, pet names/species, requested service/date, prior approved contact preference, approved policy snippets, and same-inquiry message refs. It should avoid full histories, whole message threads, raw documents, raw provider payloads, unredacted contact exports, payment data, staff-only notes, and unrelated records.

### Permission posture

For inquiry intake, read permissions are limited to location/service catalog, approved intake requirements, identity candidates, contact preferences, pet basics, requested service/date/source channel, same-inquiry evidence, and dedupe audit refs. Write/recommend permissions are limited to structured extraction, missing-info summary, duplicate/ambiguity flags, internal task drafts, follow-up message drafts, risk flags, and workflow/audit result records.

Default customer-message automation is:

- `QueueForReview` for low-risk routine missing-info drafts.
- `DraftOnly` or `NeverAutoSend` for sensitive, ambiguous, denial/refusal-adjacent, payment, policy, medical/vaccine, behavior/safety, legal, or complaint language.
- No AI-controlled direct sends.
- Future deterministic auto-send may be considered only for fixed approved templates, verified facts, known recipient/channel/consent/suppression, exact trigger conditions, idempotency, and audit.

## 4. Extraction schema

The workflow-specific structured output is `InquiryIntakeExtraction` under a `WorkflowResult<InquiryIntakeOutput>` envelope.

### Shared envelope

```json
{
  "schema_name": "InquiryIntakeOutput",
  "schema_version": "2026-06-11",
  "workflow_name": "inquiry-intake",
  "event_id": "workflow_event_id",
  "subject": { "type": "customer_or_external", "id": "..." },
  "status": "Completed | NeedsHumanReview | NeedsMoreInformation | RejectedByPolicy | FailedSafely",
  "summary": "Operator-safe intake summary.",
  "structured_output": { "...": "InquiryIntakeExtraction" },
  "recommended_actions": [],
  "risk_flags": [],
  "verification": [],
  "uncertainty": [],
  "missing_inputs": [],
  "approval_requirements": [],
  "human_review_reason": null,
  "safe_log": {}
}
```

Status mapping:

- `Completed`: safe internal extraction/recommendation packet only; no external side effect approved.
- `NeedsMoreInformation`: missing-info follow-up needed.
- `NeedsHumanReview`: named gate is required for sensitive/ambiguous/conflicting content.
- `RejectedByPolicy`: requested action/output conflicts with deterministic policy.
- `FailedSafely`: packet/runtime/output could not be validated safely.

### Top-level structured output

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
  "triage": {},
  "draft_messages": [],
  "internal_tasks": [],
  "validation": {},
  "record_mapping": {},
  "source_refs": []
}
```

### Field evidence wrapper

Any fact that can affect routing, tasks, drafts, record mapping, reservation status suggestions, eligibility, or customer-facing copy must use evidence metadata rather than a bare value:

```json
{
  "value": "normalized value or null",
  "state": "known | unknown | ambiguous | conflicting | not_applicable | redacted",
  "confidence": "high | medium | low | none",
  "evidence": [
    {
      "source_ref": "source_message:msg_123:excerpt_2",
      "span": "char:42-68 or field:owner_name",
      "quote": "redacted minimal excerpt if policy permits",
      "source_trust_state": "customer_reported | staff_entered | provider_verified | provider_unverified | system_derived | unknown"
    }
  ],
  "notes": "short operator note, no unnecessary PII"
}
```

Rules:

- `confidence` is extraction confidence only; it is never approval authority.
- `known`, `ambiguous`, and `conflicting` values need evidence refs or deterministic source refs.
- `unknown` means absent from checked sources, not merely omitted by the model.
- `conflicting` values must not be silently resolved; downstream review decides.
- Quotes must be omitted/redacted for unnecessary PII, full raw messages, medical/payment/provider/secrets, or policy-prohibited content.

### Required extraction domains

The output must explicitly cover these domains, even when values are unknown:

1. Source:
   - `source_kind`, `reservation_source_mapping`, source ids, provider refs, submitted/received time, actor kind, verification state.
   - `source_kind=chat_widget` maps to `reservation_source_mapping=unmapped_chat_widget` until domain vocabulary is added or an explicit product mapping is approved.
2. Owner:
   - Customer id or external lead id, name, household/co-owner if present, identity match state, matched ids, conflict reasons.
   - Do not create/merge from name similarity alone.
3. Contact:
   - Email/phone/channel refs as minimized evidence, preferred channel, contact channel mapping, consent state, quiet-hours/suppression state.
   - Contact route is follow-up evidence, not legal send authority.
4. Pets:
   - One pet slot per mentioned animal; pet id if matched; name/species/breed/size/age/sex/spay-neuter; mapping state; per-pet missing info and risk flags.
   - Unknown species must not default to dog.
5. Requested services:
   - Service slots mapped only to existing `ServiceKind` values where clear: `Boarding`, `DayPlay`, `DayBoarding`, `Grooming`, `Training`, `DaySpa`.
   - Ambiguous or unsupported service terms remain candidates/missing-info/review.
6. Dates and times:
   - Date/time slots with timezone, date type, interpretation notes, and service/pet cross-refs.
   - Relative dates require source timestamp and location timezone before concrete operational use.
7. Special needs and care notes:
   - Feeding, medication, allergy, medical, mobility, anxiety, behavior, emergency/vet contacts, and other care claims as review evidence.
   - Do not diagnose, execute care instructions, or promise accommodations.
8. Temperament/behavior:
   - Reported temperament signals and review hints.
   - Aggression, bites, fights, escape risk, severe anxiety, guarding, group-play uncertainty, or incidents require `BehaviorReview`/manager review as appropriate.
9. Vaccine mention/status:
   - Mention state, per-pet claims/doc refs, trusted status, and review hint.
   - Owner statements and uploads are evidence for vaccine-document workflow; they are not accepted vaccine facts.
10. Urgency:
   - Routing hint and queue priority based on dates, customer distress, safety/medical/legal/complaint language, repeated follow-up, or operational deadlines.
   - Urgency never authorizes side effects or skipped gates.
11. Missing info:
   - Checklist entries such as owner name, contact method, pet identity/species/breed/size, service, dates/times, vaccine proof, special-needs details, temperament details, consent/preference, location, and other policy inputs.
12. Ambiguities/conflicts:
   - Explicit records for ambiguous/conflicting/unknown facts, candidates, required resolution path, and blocked consumers.

### Validation rules

Validator requirements:

1. Envelope workflow/event/subject/location must match prompt packet.
2. Enum fields must use typed values, not arbitrary prose.
3. Every known/ambiguous/conflicting extracted fact affecting mapping, tasks, drafts, review, or status suggestions has source refs.
4. Pet/service/date slots have unique ids and valid cross-references.
5. `source_kind=chat_widget` is not silently mapped to `WebsiteForm` or `Portal` without preserving the gap.
6. `Completed` is valid only for safe internal packets and still does not authorize sends or mutations.
7. `NeedsMoreInformation` requires missing-info entries or a review reason.
8. `NeedsHumanReview` requires a human-review reason and gate.
9. Customer drafts require `CustomerMessageApproval` unless a later deterministic send policy is cited.
10. Raw provider JSON, unredacted threads/transcripts, raw OCR/documents, payment/card data, webhook signatures, secrets, and unrelated PII are invalid in structured output.

## 5. Triage categories and routing

Use the seven-category routing vocabulary from `docs/workflows/inquiry-intake-parts/triage-routing.md` as the canonical workflow output vocabulary:

1. `spam`
2. `unsupported`
3. `behavior_review`
4. `special_care_review`
5. `vaccine_docs_needed`
6. `missing_information`
7. `ready_for_booking_review`

Apply categories in that precedence order. Higher-precedence categories may emit secondary flags/tasks for lower-precedence issues, but the primary route is the first matching category.

### Category matrix

| Category | Trigger | Lead/status suggestion | Typical outputs | Gates |
| --- | --- | --- | --- | --- |
| `ready_for_booking_review` | Real supported inquiry with enough owner/contact/location/service/date/pet facts and no unresolved vaccine, care, behavior, unsupported, or spam issue. | `ReadyForBookingReview`; reservation suggestion `Requested` or keep current with booking-triage task; never `Offered`/`Confirmed`. | Booking-triage task/event draft, intake summary, optional acknowledgement draft. | Staff approval before availability/offer/waitlist/denial/confirmation/deposit/provider mutation; `CustomerMessageApproval` for drafts. |
| `missing_information` | Supported inquiry but routine required facts are absent, stale, contradictory, or not confidently mapped. | `NeedsMoreInformation`; reservation suggestion `MissingInfo`. | Follow-up task, missing-info checklist, safe customer draft. | `CustomerMessageApproval`; staff approval for status/record changes; manager/privacy review for identity/consent conflict. |
| `vaccine_docs_needed` | Vaccine proof/docs are missing, ambiguous, stale, unverified, or attached and needing review. | `NeedsVaccineDocuments`; reservation suggestion `VaccinePending`. | Document/vaccine review task, document upload/request draft, risk flags. | `MedicalDocumentReview` for acceptance/expiry/source; `CustomerMessageApproval`; manager for exceptions/disputes. |
| `special_care_review` | Medical, medication, allergy, feeding, mobility, isolation, senior/special-needs, illness, care-lane, or handling ambiguity. | `NeedsSpecialCareReview`; reservation suggestion `SpecialReview`. | Care/medical review task, clarifying questions, safe draft. | Care lead/manager approval before accepting/rejecting care fit, executable care tasks, special-care fees, medical/care copy. |
| `behavior_review` | Aggression, bite/fight/attack, growling/snapping/lunging, selectivity, guarding, escape risk, severe anxiety, prior incident, suspension, group-play uncertainty, or similar safety flag. | `NeedsBehaviorReview`; reservation suggestion `SpecialReview` with behavior review reason. | Behavior/manager task, risk flags, draft only after review. | `BehaviorReview`, often `ManagerApproval`, and `CustomerMessageApproval`; no auto-send with behavior/aggression flags. |
| `unsupported` | Real but outside service/location/workflow scope, emergency/vet/legal/payment/vendor/non-customer request, or unsupported species/action. | `UnsupportedInquiry`; no reservation mutation by default. | Unsupported-route packet, staff/manager review, optional neutral draft. | Manager/admin for denial/sensitive redirect; `CustomerMessageApproval`; no final refusal/referral unless approved. |
| `spam` | Junk, phishing, credential/payment-token request, malware/link farm, bot gibberish, duplicate flood, or no plausible pet-service intent. | `SpamSuppressed`; no reservation status. | Suppression audit or admin/security task. | Human/admin before blocking known customer, reporting, preserving legal/security evidence, or replying. |

### Secondary flags

Preserve all concerns as secondary flags even when primary category is higher precedence:

- Missing/contact/identity: `missing_contact`, `missing_pet_identity`, `missing_service`, `missing_date_range`, `identity_conflict`, `contact_consent_unknown`.
- Vaccine/docs: `vaccine_document_missing`, `vaccine_document_unverified`, `document_uploaded_needs_extraction`.
- Special care: `medical_or_medication_review_required`, `allergy_or_feeding_review_required`, `special_handling_or_isolation`, `vet_clarification_needed`.
- Behavior/safety: `aggression_or_bite_history`, `group_play_review_required`, `escape_risk`, `unresolved_incident`, `behavior_source_unverified`.
- Unsupported/spam/security: `unsupported_service_or_species`, `unsupported_location`, `emergency_or_veterinary_redirect_needed`, `vendor_or_non_customer`, `spam_or_phishing`, `credential_or_payment_secret_risk`, `abusive_or_threatening_content`.

### Conflict preserved from source cards

The triage-routing artifact defines canonical categories as `ready_for_booking_review`, `missing_information`, `vaccine_docs_needed`, `special_care_review`, `behavior_review`, `unsupported`, and `spam`.

The test-fixture artifact uses an earlier/fixture-oriented vocabulary: `ready_for_staff_intake_review`, `missing_info`, `service_scope_review`, `behavior_review`, `medical_or_vaccine_review`, and `grooming_lead`.

Recommended resolution:

- Use the seven-category triage-routing vocabulary as canonical workflow output.
- Keep fixture aliases as test labels or migration aliases:
  - `ready_for_staff_intake_review` -> `ready_for_booking_review` when enough facts exist for staff/booking triage and no higher-risk route remains.
  - `missing_info` -> `missing_information`.
  - `service_scope_review` -> `unsupported` when truly out of scope; otherwise `missing_information` for unclear service.
  - `medical_or_vaccine_review` -> `vaccine_docs_needed` for document/proof issues, or `special_care_review` for medical/care ambiguity.
  - `grooming_lead` -> `ready_for_booking_review` or `missing_information` with `service_kind=Grooming` and a grooming-specific internal task; do not create a separate primary category unless product later requires a distinct routing behavior.
- Tests should assert both canonical category and optional fixture alias during migration to avoid hiding the disagreement.

## 6. Draft reply patterns

All customer replies are drafts by default and require `CustomerMessageApproval` unless a future deterministic auto-send policy approves an exact template/fact/recipient/channel/trigger/suppression/audit path. AI-authored free text is not auto-sendable by default.

### Global drafting rules

Tone:

- Warm, pet-parent-friendly, operationally clear.
- Concise and easy for staff to review.
- Truthful about what is known, unknown, and under review.
- Reassuring without promising space, eligibility, price, timing, or outcome.
- Neutral for behavior, medical, vaccine, and special-care topics.

Every draft should carry:

- `template_id` and version/policy snapshot.
- Channel and recipient ref, not raw contact where avoidable.
- Evidence refs for factual claims.
- Review gates, at least `CustomerMessageApproval`.
- Automation level (`QueueForReview`, `DraftOnly`, or `NeverAutoSend`).
- Forbidden-claims checklist.
- Draft idempotency key.

### Template intents

| Template id | Intent | Default automation | Additional gates |
| --- | --- | --- | --- |
| `inquiry_missing_info_v1` | Ask for routine missing owner/pet/service/date/contact facts. | `QueueForReview`; `DraftOnly` if sensitive. | `ManagerApproval` for policy/complaint/refusal/special accommodation; `BehaviorReview` for behavior; `MedicalDocumentReview` for document ambiguity. |
| `inquiry_vaccine_request_v1` | Request vaccine proof or clearer documentation. | `QueueForReview` for simple proof request; `DraftOnly` when ambiguous or eligibility-affecting. | `MedicalDocumentReview`; manager for waiver/exception/refusal/urgent check-in. |
| `inquiry_availability_pending_v1` | Acknowledge request and staff availability review without promise. | `QueueForReview`; `DraftOnly` for peak/capacity/special-care/behavior/policy cases. | Manager, `BehaviorReview`, `MedicalDocumentReview` as triggered. |
| `inquiry_waitlist_v1` | Staff-reviewed waitlist-related wording only. | `DraftOnly` by default. | `ManagerApproval`; behavior/document gates if eligibility-dependent. |
| `inquiry_special_care_followup_v1` | Ask for care/medical/behavior/special-handling clarification. | `DraftOnly`. | `ManagerApproval`, `MedicalDocumentReview`, `BehaviorReview` as applicable. |
| `inquiry_unsupported_service_v1` | Neutral unsupported/out-of-scope follow-up. | `DraftOnly`; possible `QueueForReview` only after policy approval. | `ManagerApproval` for denial/refusal/complaint; behavior/document gates if eligibility-dependent. |

Never include in an inquiry reply draft unless a later authorized workflow supplies approved wording:

- “Confirmed,” “booked,” “approved,” “cleared,” “we have space,” “space is held,” or “you are all set.”
- Final vaccine, medical, medication, allergy, document, behavior, group-play, or service eligibility decisions.
- Price/deposit/payment/refund/waiver/discount/policy exception statements.
- Provider status changes, waitlist movement, denial/refusal, or referral promises.
- Exact SLA/response deadline unless sourced from approved location policy.

### Behavior/aggression copy rule

If the inquiry mentions aggression, bites, fights, guarding, escape risk, severe anxiety, safety, group-play eligibility, incidents, or similar behavior flags:

- Primary route is `behavior_review` unless a higher security/spam/unsupported rule applies.
- Preserve exact source signal internally with evidence refs; do not hide or soften it in staff packets.
- Customer copy should use neutral language such as “additional care/handling details” or “our team can review the best fit” unless a behavior/manager reviewer approved more specific wording.
- No auto-send. Customer-facing wording that mentions aggression, bites, incidents, restrictions, safety, eligibility, acceptance, or denial requires `BehaviorReview` plus `CustomerMessageApproval`, and manager approval for severe/sensitive cases.

## 7. Internal task creation

Inquiry-intake may draft internal tasks and may live-create them only if a later task-creation policy explicitly approves the exact trigger, task kind, assignment, idempotency, evidence, and tests. Safe default is `DraftOnly` / `RecommendedAction::InternalTask`.

### Shared task contract

Every task draft should include:

- `location_id`, workflow event id, source event key, policy version.
- Task kind and inquiry-specific `task_intent`.
- Internal title/body with checklist, blocked actions, and completion instructions.
- Owner role/queue, due basis, due time, priority, and priority rationale.
- Source/evidence refs and related lead/customer/pet/reservation/document/message ids.
- Required payload fields and completion evidence.
- Creation policy: `DraftOnly`, `AutoCreateAllowed(policy_ref)`, or `RequiresReview(gate)`.
- Idempotency key and duplicate-handling action.
- Emitted events/audit observations.

### Task intents

| Intent | Trigger | Owner/role | Default priority | Completion evidence | Current model mapping |
| --- | --- | --- | --- | --- | --- |
| `call_customer` | Phone follow-up requested/needed; missing facts better resolved synchronously; phone transcript incomplete; conflicting identity/contact/consent. | Front desk; lead/manager for sensitive issues. | Normal; high for near-term/repeated/customer-waiting; critical only for escalation workflows. | Staff actor, timestamp, call outcome, contact ref, structured facts collected, unresolved questions. | `CustomerFollowUp { reason }`. |
| `verify_docs` | Vaccine/document proof missing/uploaded/stale/ambiguous/source-unverified; document linked to inquiry. | Front desk/document reviewer; manager for disputes/exceptions. | Normal; high when near-term or blocking. | Reviewer, document/evidence ids, policy version, disposition, extracted candidates, follow-up needed. | `DocumentReview { pet_id }` when pet mapped; otherwise follow-up/review packet. |
| `manager_review` | Policy/capacity/waitlist/payment/complaint/legal/behavior/medical/source conflict or sensitive customer wording. | Manager by default. | High by default; critical for safety/legal/emergency; normal only for future low-risk policy clarification. | Manager actor, reviewed refs, decision/disposition, approved next actions/wording, remaining blocked actions. | Nearest current task kind with `NeedsManagerReview`, often `CustomerFollowUp` + manager assignment. |
| `check_availability` | Service/date/pet count sufficient for staff to check capacity/calendar, but no availability promise exists. | Front desk, groomer, trainer, lead staff, or manager depending on service/exception. | Normal; high for near-term/high-value/repeated/full-ish; critical only for operational escalation. | Staff actor, snapshot/calendar refs checked, result category, statement no customer promise was made unless separately approved. | `CustomerFollowUp` or booking/readiness packet until availability task kind exists. |
| `request_behavior_notes` | Dayplay/group-play/boarding/play/training/special-handling/behavior risk; pet profile lacks current temperament/behavior notes. | Front desk for collection; lead/kennel/playgroup staff for assessment; manager for aggression/bite/incident/denial. | Normal; high for near-term or known ambiguity; critical for incident/safety escalation. | Collected notes, staff/lead/manager review state, uncertainty refs, downstream tasks/drafts. | `CustomerFollowUp` for collection; `PlaygroupAssessment { pet_id }` when pet exists and policy requires. |

### Task dedupe

Use a task key shaped like:

```text
task_key = v1:{location_id}:internal_task:{task_intent}:{domain_subject}:{semantic_reason}:{policy_version}
```

Do not create duplicates when an open task with the same key exists. Instead attach new evidence, update the task body/checklist for new non-conflicting facts, and emit an update audit observation. Conflicting facts create/update manager/reconciliation work rather than overwriting.

## 8. Human approval gates

The following gates must be explicit data in output, task, draft, and audit records.

### Customer auto-send / outbound gate

Default: no inquiry-intake auto-send.

Any customer-facing send requires a separate approved send path with:

- Approved draft id/version or fixed deterministic template id/version.
- Recipient/channel/contact ref.
- Consent/opt-out/quiet-hours/suppression checks.
- Approval actor/id when required.
- Policy version and idempotency key.
- Provider response refs and audit event.

A future deterministic auto-send path may be considered only for narrow receipt-only or missing-safe-fact templates when all facts are verified and no sensitive flags, consent gaps, source conflicts, or policy exceptions exist.

### Behavior/aggression gate

Required for any aggression, bite, fight, attack, guarding, escape risk, severe anxiety, incident, suspension, group-play eligibility, safety, or similar behavior signal:

- `BehaviorReview` is required before behavior/group-play/service eligibility conclusions.
- `ManagerApproval` is required for bite/aggression/severe/sensitive/denial/restriction/reinstatement/customer-dispute wording.
- `CustomerMessageApproval` is required before any customer-facing copy.
- No behavior/aggression customer reply is auto-sent by inquiry intake.

### Medical/vaccine/special-care gate

- `MedicalDocumentReview` is required for vaccine document acceptance, expiry/freshness, source/licensed-vet proof, and eligibility-affecting vaccine copy.
- Care lead/manager approval is required before accepting/rejecting special-care fit, executable medication/care plans, medical advice, special-care fees, or accommodations.
- Customer claims and OCR outputs remain evidence until reviewed.

### Booking/provider/payment gate

Staff/manager approval and downstream workflow execution are required before:

- Availability, waitlist, offer, confirmation, denial, cancellation, check-in/out, room/capacity hold, or provider write.
- Deposit/payment collection, refund, waiver, discount, credit, forfeiture, price/rate, or payment exception.
- Any provider/Gingr mutation.

## 9. Integration points

### Product map

- Current product-map path is missing. Cite `docs/workflows/staff-operations-parts/inputs.md` and `docs/workflows/inquiry-intake-inputs.md` until restored.
- Product scope drives service catalog, roles, approval posture, and unresolved policy questions.
- Do not infer local pricing, service availability, vaccine requirements, staff SLAs, ratio rules, or customer copy from public context without approved policy.

### Data model

Current anchors:

- `Customer`: identity/contact/preferred channel; contact preference is not consent.
- `Pet`: animal identity, species, profile/care/temperament; AI cannot clear high-risk facts.
- `Reservation`: candidate/request mapping only; statuses relevant to intake include `Inquiry`, `Requested`, `MissingInfo`, `VaccinePending`, `SpecialReview`, `Waitlisted`, `Offered`, and `Confirmed`, but inquiry intake must not progress to `Offered`/`Confirmed`.
- `ServiceKind`: `Boarding`, `DayPlay`, `DayBoarding`, `Grooming`, `Training`, `DaySpa`.
- `ReservationSource`: `Portal`, `WebsiteForm`, `PhoneTranscript`, `Sms`, `Email`, `StaffCreated`; chat is missing.
- `WorkflowResult`, `WorkflowStatus`, `RecommendedAction`, `PolicyContext`, and `AllowedAction` provide result and permission envelopes.
- `StaffTask`, `StaffTaskKind`, `StaffRole`, and task lifecycle anchor internal task drafts.

Known model gaps to carry forward:

1. No first-class inquiry/lead aggregate exists beyond `WorkflowSubject::External`, reservation `Inquiry` status, provider lead refs, and operations lead surfaces.
2. Chat widget source/channel/delivery vocabulary is missing.
3. Concrete consent/opt-out/quiet-hours model is not fully represented.
4. Source-message/transcript/attachment evidence schemas need implementation detail.
5. Multi-pet booking aggregation remains open: one reservation with pet segments vs linked child reservations.
6. Current `StaffTaskKind` lacks dedicated `CallCustomer`, `ManagerReview`, `CheckAvailability`, and `RequestBehaviorNotes` variants; use `task_intent` metadata or add typed variants later.

### Workflow events

Primary event:

- `inquiry.received`: normalized inbound inquiry event; can produce intake summary, missing-info task drafts, message drafts, risk flags, and status `NeedsMoreInformation` or `Completed` as safe recommendation.

Possible derived events/task audit observations:

- `booking.triage_needed` after staff/intake packet is ready for booking/capacity review.
- `document.uploaded` / `vaccine.extraction_needed` when attachment/evidence needs vaccine/document handling.
- `internal_task.proposed`, `internal_task.created`, `internal_task.updated`, `internal_task.completed`, `manager_review.requested`, `behavior_review.requested`, `document.review_requested`, `draft_customer_message.approval_requested` as implementation-local audit/event names where appropriate.

Do not add new event vocabulary when a task kind, audit action, or result subtype is enough. Add event vocabulary only when routing/policy behavior differs.

### AI runtime

The app owns:

- Source authentication/normalization.
- Event acceptance and idempotency.
- Prompt-packet construction.
- Context minimization/redaction.
- Runtime invocation and model config.
- Output parsing/schema validation/retry.
- Deterministic policy validation.
- Persistence, audit, approvals, and side-effect execution.

The runtime returns untrusted recommendations. Invalid, over-broad, unsafe, or policy-violating output becomes `FailedSafely`, `RejectedByPolicy`, or `NeedsHumanReview`; it must not execute side effects.

Safe logs may include ids, workflow/schema names, policy snapshot, source refs, statuses, review gates, field/category names, validation results, and redacted excerpt refs. They must exclude raw customer messages, full contact PII, raw documents/OCR, medication details, behavior/incident narratives, payment data, provider payloads, secrets, and hidden model reasoning.

## 10. Test cases and fixture expectations

Implement fixture scenarios under the shared AI-runtime fixture harness, with inquiry-specific files eventually placed under:

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

Responses can live under:

```text
fixtures/ai-runtime/responses/inquiry-intake/<scenario-id>.valid.json
fixtures/ai-runtime/responses/inquiry-intake/<scenario-id>.policy_violation.json
```

### Required fixture matrix

| Fixture id | Required case | Source | Canonical expected category | Key expectations |
| --- | --- | --- | --- | --- |
| `inquiry_complete_boarding` | Complete inquiry | Website form | `ready_for_booking_review` if contact/location policy accepts supplied facts; otherwise `missing_information` for contact/vaccine/doc gap. | Extract owner/pet/service/dates; keep vaccine claim non-authoritative; draft acknowledgement only; no availability/booking/vaccine approval. |
| `inquiry_vague_sms` | Vague inquiry | SMS | `missing_information` | Ask for owner, pet, service, exact dates, pet basics, contact preference; no price/availability quote. |
| `inquiry_multiple_pets_mixed_needs` | Multiple pets | Email | `special_care_review` when medication/care review is primary; otherwise `missing_information` with medical/vaccine secondary flags. | Keep per-pet slots/missing info; medication and vaccines are review evidence; no same-room promise. |
| `inquiry_missing_dates_daycare` | Missing dates / chat gap | Chat widget | `missing_information` | Preserve `chat_source_unmapped`; ask for contact handoff, puppy age, dates/cadence, service lane; no group-play eligibility. |
| `inquiry_anxiety_boarding` | Anxiety/special handling | Phone transcript | `behavior_review` | Flag anxiety/special-handling; request safe clarifying details; no individual-care/group-play/acceptance promise. |
| `inquiry_bite_history_dayplay` | Bite history | Staff-created phone note | `behavior_review` | Require `BehaviorReview`, `ManagerApproval`, and `CustomerMessageApproval`; no group-play approval/denial or sensitive auto-send. |
| `inquiry_cat_boarding_vaccine_mentions` | Cat boarding with vaccine mention | Portal/provider lead | `vaccine_docs_needed` unless special-care issue outranks; secondary cat boarding lead. | Cat-specific extraction; vaccine proof/request review; no dog group-play assumption or vaccine acceptance. |
| `inquiry_grooming_only` | Grooming-only | Email | `missing_information` or `ready_for_booking_review` with `service_kind=Grooming` and grooming task; do not use separate primary category in canonical output. | Extract grooming/bath/nail trim; ask owner/date/time/groom details; no lodging/daycare reservation or appointment promise. |
| `inquiry_unsupported_service_pet_sitting` | Unsupported service | Website form | `unsupported` | Identify in-home pet sitting as out of scope if policy confirms; create staff/manager review; no invented service/referral/final refusal. |

### Golden assertions

Every scenario should assert:

- Event type is `InquiryReceived` / `inquiry.received` and source/evidence refs are preserved.
- `structured_output` separates source, owner, contact, pets, request, missing info, vaccine/medical/behavior claims, risk flags, tasks, drafts, and record mapping.
- Customer claims about vaccines, health, behavior, medication, pricing, availability, and prior approvals remain non-authoritative evidence.
- Multi-pet cases keep per-pet associations and review gates.
- Cat cases do not enter dog group-play assumptions.
- Grooming-only cases do not create lodging/daycare tasks unless later evidence asks for them.
- Unsupported-service cases route to staff/manager review instead of inventing service coverage.
- `recommended_actions` contain only allowed internal tasks, review requests, and draft messages.
- Customer replies require `CustomerMessageApproval`; sensitive replies also carry `BehaviorReview`, `MedicalDocumentReview`, and/or `ManagerApproval`.
- No output confirms booking/appointment, promises availability/price/room, approves vaccines, decides group-play eligibility, sends customer messages, mutates providers, or writes final customer/pet/reservation truth from ambiguous evidence.
- Safe logs include ids, categories, workflow name, policy snapshot, source refs, status, and review gates; they exclude raw bodies, exact contact values, vaccine/OCR text, medication details, behavior/incident narrative, and full draft text.

## 11. Implementation checklist

Before implementation is considered ready:

1. Decide or create the product-map artifact path, or explicitly keep `staff-operations-parts/inputs.md` as the source of product scope.
2. Choose lead/inquiry aggregate strategy: first-class inquiry/lead entity vs `WorkflowSubject::External` + reservation `Inquiry` candidate.
3. Add or explicitly map chat widget source/channel/delivery vocabulary.
4. Model contact consent, opt-out, quiet-hours, and delivery suppression beyond `Customer.preferred_contact`.
5. Define source-message, transcript, attachment, document, and evidence refs for website form/SMS/email/phone/chat/provider lead.
6. Encode `InquiryIntakeOutput` / `InquiryIntakeExtraction` schema and validator with evidence wrappers, slot ids, cross-refs, review gates, and raw-data denylist.
7. Encode canonical triage categories and migration aliases for fixture vocabulary.
8. Encode draft template ids/metadata and forbidden-claims checks.
9. Encode task intents, dedupe keys, owner/priority/due defaults, and current `StaffTaskKind` mappings.
10. Build fake-runtime fixtures and tests before live Hermes/LLM wiring.
11. Configure prompt-packet field allowlist, redaction profile, model/provider/retention policy, and tool permissions for production review.
12. Keep all customer sends, provider writes, payment actions, booking status changes, vaccine approvals, behavior/group-play decisions, and sensitive customer copy behind deterministic policy and human approval gates.

## Conservative rule

When evidence is missing, stale, conflicting, unverified, unsupported by the current model, or sensitive, the inquiry-intake workflow must create a review state, missing-info task, or draft recommendation rather than treating the inquiry as ready. The workflow may safely extract, classify, summarize, draft, and recommend; it must not turn untrusted inquiry evidence into booking, payment, provider, medical, vaccine, behavior, eligibility, or customer-message authority.
