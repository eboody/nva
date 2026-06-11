# Inquiry intake input sources and normalization

Purpose: define the inbound source contracts that normalize customer/lead inquiry evidence into one semantic `inquiry.received` workflow event. This document is a source-normalization part for the final `docs/workflows/inquiry-intake-agent.md` artifact; it is not live operating policy and does not authorize customer-facing sends, booking promises, provider writes, payment actions, medical/vaccine/behavior decisions, or production LLM access.

Status: draft source-backed normalization contract. Current Rust/domain anchors exist for `WorkflowEventType::InquiryReceived`, `ReservationSource::{Portal, WebsiteForm, PhoneTranscript, Sms, Email, StaffCreated}`, and `ContactChannel::{Email, Sms, Phone, Portal}`. Chat widget vocabulary is required by product scope but not currently represented in the domain enums; treat chat mapping as a known data-model gap rather than silently collapsing it.

## Source anchors

Use these repo-local anchors when synthesizing the final agent artifact:

- `docs/workflows/inquiry-intake-inputs.md` — canonical inquiry-intake constraints, source-channel map, customer-message boundaries, idempotency, runtime packet, and known gaps.
- `docs/architecture/pet-resort-workflow-events.md` — global event envelope and `inquiry.received` row: source triggers, actor/subject semantics, expected payload, policy context, allowed actions, and customer-message implications.
- `docs/workflows/workflow-event-idempotency-replay.md` — source event key and side-effect key guidance for `InquiryReceived`, drafts, sends, and internal follow-up tasks.
- `docs/architecture/ai-runtime-memory-context-policy.md` — minimal runtime packet, fetch-by-ID rules, context allowlist/denylist, contact-message privacy boundaries, audit/redaction expectations.
- `docs/workflows/customer-messaging-parts/inputs.md` — channel, consent/contact preference, draft/send separation, customer-message approval posture, and message-template category inputs.
- `domain/src/entities.rs` — current customer/pet/reservation/contact/source enums and actor/audit anchors.
- `domain/src/workflow.rs` — `WorkflowEvent`, `WorkflowEventType::InquiryReceived`, `WorkflowSubject`, `PolicyContext`, `AllowedAction`, and `WorkflowResult` surfaces.
- `domain/src/agents.rs` — baseline `inquiry-intake` and `lead-conversion` agent specs and default customer-message approval gate.
- `domain/src/tools.rs` — app-owned repository/runtime boundaries and draft-message tool surfaces.
- `docs/integrations/gingr/sdk-webhooks.md` and `docs/integrations/gingr/sdk-customer-portal-js.md` — provider/browser lead observations, verification boundaries, raw-payload quarantine, and portal event privacy caveats.

## Normalization objective

Every inbound inquiry source must be normalized through the same semantic event:

```text
WorkflowEventType::InquiryReceived / inquiry.received
```

The semantic event means: a customer, staff member, trusted system, or verified provider/browser boundary produced evidence of interest in services, missing intake information, or lead follow-up. It does not mean a reservation exists, availability is known, a booking can be promised, a message can be sent, or provider state can be mutated.

Normalization has three layers:

1. Source boundary record: raw form submission, SMS, email, call transcript, chat transcript, provider webhook, or staff-entered note is accepted into a boundary/evidence store with source metadata, raw-payload quarantine, hashes, and redaction profile.
2. Canonical inquiry evidence: trusted/minimized fields are extracted into a typed source-specific normalized record with provenance, idempotency fields, contact/consent metadata, attachment/transcript refs, and uncertainty markers.
3. Semantic workflow event: the canonical evidence creates or updates one `inquiry.received` event/prompt packet with a minimized payload, policy context, review gates, and citations/evidence refs.

## Required event envelope

The final artifact should require these event-level fields for every normalized inquiry:

| Field | Requirement |
| --- | --- |
| `event_id` | Stable workflow event id assigned after source-event dedupe. |
| `event_type` | Always `InquiryReceived` in Rust and `inquiry.received` in docs/external catalog. |
| `occurred_at` | Source submission/message/call/chat/provider timestamp when reliable; otherwise receiver timestamp plus uncertainty marker. |
| `source_kind` | Canonical source category such as `website_form`, `sms`, `email`, `phone_transcript`, `chat_widget`, `customer_portal`, `provider_webhook`, `provider_poll`, or `staff_created`. |
| `actor` | Customer/owner for self-submitted form/SMS/email/chat/portal; staff for manual entry or call transcription performed by staff; system for verified provider imports; agent only for derived/replay work, not original inquiry facts. |
| `location_id` | Required policy/capacity/contact scope. If unknown, route to `NeedsMoreInformation` or staff assignment instead of guessing. |
| `subject` | `WorkflowSubject::Customer(customer_id)` when deterministically matched; otherwise `WorkflowSubject::External { provider, id }` for unmapped lead/source record. |
| `related_ids` | Optional customer, pet, reservation, provider lead, source message, attachment, transcript, audit, and policy ids needed to explain the event. |
| `policy_context` | Allowed actions, automation level, required reviews, contact-permission policy refs, redaction/context policy refs, and any location intake policy snapshot. |
| `payload` | Minimized `InquiryReceivedPayload` shape below; raw source bodies stay referenced, not copied wholesale. |
| `result` | `WorkflowResult<InquiryIntakeOutput>` after the agent runs: summary, missing-info checklist, recommended tasks/drafts, risk flags, verification, and human-review reason if needed. |

## Required normalized payload

The final `InquiryReceivedPayload` should include these normalized fields regardless of source:

| Field group | Required normalized fields |
| --- | --- |
| Identity/provenance | `source_kind`, `source_system`, `source_record_id`, optional `source_delivery_id`, `source_version`, `source_received_at`, `source_submitted_at`, source confidence, source evidence refs, raw payload/storage ref, canonical source fingerprint. |
| Location | `location_id`, source-provided location/site/campaign where present, location match confidence, unresolved-location reason if unknown. |
| Customer/lead | optional `customer_id`, lead/external id, supplied customer name, supplied email/phone/portal handle refs, identity match confidence, duplicate/conflict markers. |
| Contact channel | `contact_channel` for the inbound source, requested/preferred channel if supplied, reply-to/destination ref, consent/opt-in/opt-out/quiet-hours state if known, and `consent_unknown` when source only proves channel availability. |
| Pet summary | Optional pet ids when matched; supplied pet names/species/breed/age/sex/spay-neuter summary where provided; pet-count; missing-pet-info checklist; no medical/vaccine truth promotion from free text. |
| Request intent | Requested services, date range/time window, boarding/daycare/grooming/training/DaySpa/add-on interest, flexibility notes, campaign/referrer when relevant, and unsupported/ambiguous service markers. |
| Message/excerpt | Minimal relevant redacted excerpt or structured summary, language/locale if known, urgency/sentiment only as non-authoritative triage signals, staff uncertainty notes. |
| Attachments/transcripts | Attachment refs, transcript refs, media/document metadata, content type, size/hash, redaction/extraction status, malware/safety scan status if available, and whether the content is approved for AI prompt use. |
| Review/risk | Required review gates, sensitive categories observed, missing/ambiguous/conflicting fields, policy-blocked actions, risk flags such as urgent safety/medical/payment/legal/customer complaint. |
| Audit | Ingest actor/system, normalizer version, redaction profile/version, policy snapshot/version, prompt-field manifest, data fetch manifest, and audit event refs. |

## Source-specific raw field contracts

### Website form

Current domain anchor: `ReservationSource::WebsiteForm`. Documentation trigger: lead form.

Raw/source fields to preserve:

- Form submission id, form id/version, landing page/referrer/UTM/campaign, submitted-at timestamp, receiver-at timestamp, source IP/user agent only if approved for fraud/debug audit and not sent to LLM by default.
- Location/site selection, service interest, date/time window, pet count and pet details, customer name, email, phone, preferred contact, free-text notes, consent checkbox text/version and boolean values.
- Attachment refs if the form permits files; never inline raw documents into the workflow payload.
- Bot/spam/honeypot/captcha status, if available, as boundary/risk metadata.

Normalize to:

- `source_kind = website_form`.
- `actor = Customer` unless submitted by a staff kiosk/manual form path.
- `contact_channel` is the channel requested/preferred by the form when explicit; otherwise use the provided reply-to fields as candidate contact refs, not consent proof.
- `source_record_id = form_submission_id` where available; otherwise canonical hash of location + normalized contact + submitted_at + form version + selected service/date.
- `subject = Customer` if deterministic match succeeds; otherwise `External { provider: "website_form", id: source_record_id }`.

Required review/edge handling:

- Spam/bot suspicion routes to human/review or suppression; do not create customer drafts automatically.
- Conflicting name/email/phone vs existing customer creates identity review rather than overwriting customer profile.
- Free-text service/medical/payment/behavior claims remain evidence, not authoritative facts.

### SMS

Current anchors: `ReservationSource::Sms`, `ContactChannel::Sms`, draft/send channel may be SMS in messaging tools.

Raw/source fields to preserve:

- Provider message id, conversation/thread id, inbound phone number, destination/location number, received timestamp, carrier/provider metadata, delivery direction, opt-out keywords/provider suppression status, media/MMS attachment refs, and prior-message refs only within the approved source window.
- Message text as raw boundary content plus a redacted minimal excerpt for the normalized payload.

Normalize to:

- `source_kind = sms`.
- `actor = Customer` for inbound customer text; `actor = Staff` only for staff-entered SMS summary/manual intake.
- `contact_channel = Sms`; `reply_to_ref` is the inbound phone/contact ref.
- `source_record_id = provider_message_id`; fallback fingerprint: location number + inbound phone hash + received_at + normalized body hash.
- `subject = Customer` if phone maps cleanly to one customer; otherwise external lead/message record.

Required review/edge handling:

- `STOP`, unsubscribe, opt-out, harassment, urgent safety, medical, refund/payment, complaint, or legal language must carry review/suppression flags.
- Channel availability is not legal permission to send; outbound SMS requires consent/contact policy and separate approved send key.
- Do not send full SMS history to the agent; include only the current message and minimal approved surrounding context if needed.

### Email

Current anchors: `ReservationSource::Email`, `ContactChannel::Email`, and Gingr `email_sent` as a provider observation, not inquiry truth by itself.

Raw/source fields to preserve:

- Message id, thread id, mailbox/account, from/to/cc/bcc refs after redaction, subject, sent/received timestamps, reply-to, provider labels/folders, SPF/DKIM/DMARC/security status if available, attachment refs, inline image refs, and sanitized text/html body refs.
- Raw MIME/html remains boundary evidence; normalized payload gets a short relevant excerpt or structured summary.

Normalize to:

- `source_kind = email`.
- `actor = Customer` for inbound customer email, `Staff` for staff-entered email summary, or `System` for verified provider email observation that is only evidence.
- `contact_channel = Email`; `reply_to_ref` is the approved email contact ref.
- `source_record_id = message_id`; fallback fingerprint: mailbox + from hash + received_at + subject/body canonical hash.
- `subject = Customer` if a sender/contact maps cleanly; otherwise external email lead/message record.

Required review/edge handling:

- Email attachments may contain vaccine/medical/payment/legal data; classify and store refs, but do not promote to intake truth or include raw contents in prompt context.
- Multi-message threads should be summarized/minimized; never pass whole inboxes or unrelated thread history.
- Sender spoofing/security failure lowers confidence and may require human review before customer/profile matching or drafts.

### Phone transcript

Current anchors: `ReservationSource::PhoneTranscript`, `ContactChannel::Phone`.

Raw/source fields to preserve:

- Call id, telephony provider id, inbound/outbound direction, caller/callee numbers, started/ended timestamps, duration, recording ref if retained, transcript ref, transcription provider/version, diarization/speaker confidence, staff participant id, manual notes, and consent/recording disclosure metadata if applicable.
- Transcript text should remain in transcript storage; normalized payload includes a redacted summary/excerpt and uncertainty markers.

Normalize to:

- `source_kind = phone_transcript`.
- `actor = Customer` when transcript captures customer call content directly; `actor = Staff` when staff manually enters a call summary; preserve both participants in source metadata.
- `contact_channel = Phone` for inbound channel. Do not assume phone is an outbound text/call automation channel.
- `source_record_id = call_id` or transcript id; fallback fingerprint: location line + caller hash + call_start + transcript hash.
- `subject = Customer` if caller/summary maps cleanly; otherwise external call lead/transcript record.

Required review/edge handling:

- Low transcript confidence, ambiguous speaker attribution, or staff uncertainty must route to `NeedsHumanReview` or `NeedsMoreInformation`.
- Sensitive medical/behavior/payment/legal/customer-complaint content should be flagged and excluded from customer-safe drafts unless reviewed.
- If call recording/transcription consent is absent or unknown, keep raw recording/transcript out of LLM context and use only staff-approved summary refs.

### Chat widget

Current gap: no `ReservationSource`, `ContactChannel`, or `DeliveryChannel` variant for chat widget exists in `domain/src/entities.rs`. Product scope requires this source, so downstream data-model work should add explicit vocabulary such as `ReservationSource::ChatWidget` and `ContactChannel::Chat`/`WebChat`, or document a deliberate mapping if the product treats chat as website-form evidence.

Raw/source fields to preserve:

- Chat session id, message ids, widget/provider id, page URL/referrer/campaign, visitor id/cookie/session ref if approved, customer-supplied name/email/phone, timestamps per message, staff/bot/agent participant ids, transcript ref, handoff state, offline form fields, consent/disclosure text/version, and attachment refs if supported.
- Raw transcript remains boundary evidence; normalized payload includes a relevant redacted excerpt and transcript summary.

Normalize to provisional contract until enum support exists:

- Preferred future `source_kind = chat_widget`.
- If implementation must use existing enums before data-model update, preserve a separate `source_kind = chat_widget` in source metadata and map `ReservationSource::WebsiteForm` only as a temporary storage compatibility value, never as semantic erasure.
- `actor = Customer` for visitor/customer messages, `Staff` for staff-entered transcript summaries, `System` for bot-generated lead handoff.
- `contact_channel = ChatWidget/WebChat` when model exists; otherwise mark `contact_channel_gap = true` and do not infer Email/SMS consent from chat-supplied contact fields.
- `source_record_id = chat_session_id` plus latest relevant message id; fallback fingerprint: location/widget + visitor/session hash + started_at/submitted_at + transcript hash.

Required review/edge handling:

- Bot transcripts, staff handoffs, and visitor free text must preserve speaker/source attribution.
- Chat consent for in-session reply does not imply later SMS/email permission unless explicit consent fields exist.
- Do not treat browser visitor ids/cookies as customer identity without deterministic account/contact matching and privacy approval.

### Customer portal / verified provider lead

Although the task asks for website form, SMS, email, phone transcript, and chat widget, the final artifact should keep the existing portal/provider lead boundary because current docs list it as a source trigger for `inquiry.received`.

Current anchors: `ReservationSource::Portal(PortalProvider)`, `ContactChannel::Portal`, `PortalProvider::Gingr`, Gingr webhook `lead_created`, portal JS `lead_created`.

Raw/source fields to preserve:

- Provider/webhook delivery id, provider event type, entity type/id, verified signature status, received timestamp, provider location/app id, raw payload ref, normalized provider lead id, customer/owner/animal ids where supplied, and verification/audit refs.
- Browser portal JS event name, session/app id, timestamp, and minimal allowlisted metadata only if a reviewed bridge exists; raw `e.detail` is personal data and must not be forwarded by default.

Normalize to:

- `source_kind = provider_webhook`, `customer_portal`, `provider_poll`, or `portal_browser_event` as appropriate.
- `actor = System` for verified server-side provider import; `Customer` for first-party customer portal submission if backend identity is known; browser event observations remain low-authority signals until reconciled.
- `source_record_id = verified provider delivery/event id or provider lead id`.
- `subject = Customer` if provider owner/customer identity maps; otherwise `External { provider: Gingr/Other, id: provider lead id }`.

Required review/edge handling:

- Unverified webhooks and portal browser events cannot drive semantic workflows directly.
- Provider `lead_created` maps to `inquiry.received` only after signature verification, source normalization, identity reconciliation, and raw-payload quarantine.
- Portal JavaScript is observational/customer-facing and must not mutate UI, collect expanded personal data, or forward raw form data without separate reviewed design.

## Idempotency and source IDs

Recommended source key for normalized inquiry events:

```text
source_event_key = v1:{location_id}:InquiryReceived:{primary_subject}:{source_kind}:{source_fingerprint}
```

Where:

- `primary_subject` is `customer:{customer_id}` when matched, otherwise `external:{source_system}:{source_record_id}`.
- `source_kind` is the canonical normalized source category, not the temporary storage enum if chat is mapped through a compatibility path.
- `source_fingerprint` should prefer a stable upstream/provider id: form submission id, SMS provider message id, email message id, call/transcript id, chat session/message id, verified webhook delivery id, provider lead id.
- When no upstream id exists, fingerprint the minimal verified tuple that defines the same inquiry: location + source kind + normalized contact ref hash + source submitted/received timestamp bucket + service/date intent + body/transcript hash. Do not hash whole raw payloads, HTML, or incidental metadata.

Duplicate behavior:

- Same source key and same canonical fingerprint: attach duplicate observation/audit and return existing processing status.
- Same contact/source/submitted timestamp with minor duplicate evidence: merge into one lead/review packet and attach source evidence refs.
- Same key with conflicting canonical fields: create identity/source reconciliation review; do not overwrite customer/pet/contact/reservation truth.
- Multiple sources from the same lead journey may converge on one internal follow-up task when the side-effect key matches.

Recommended side-effect keys that the final artifact should preserve:

```text
internal_task = v1:{location_id}:internal_task:inquiry_follow_up:{customer_or_external}:{semantic_reason}:{policy_version}
draft = v1:{location_id}:draft_customer_message:inquiry_follow_up:{customer_or_external}:{evidence_set_hash}:{template_or_copy_version}:{policy_version}
send = v1:{location_id}:outbound_customer_message:{approved_draft_id}:{approved_draft_version}:{recipient_id}:{channel}:inquiry_follow_up:{approval_id}
```

Draft creation and outbound send must remain separate. Replaying `inquiry.received` may rebuild the lead summary, missing-info checklist, task recommendation, or draft; it must not send a customer message without a separate approval/deterministic send path.

## Consent and contact-channel metadata

Every normalized inquiry should carry contact metadata as facts to evaluate, not as automatic send authority:

- `inbound_contact_channel`: how the inquiry arrived.
- `customer_preferred_contact_channel`: what the customer selected/stated, if any.
- `reply_to_contact_ref`: exact email/phone/portal/chat ref needed for a draft/send review, redacted/hashable in logs.
- `consent_status`: explicit opt-in, opt-out, prior approved consent, unknown, not applicable, or suppressed by provider/policy.
- `consent_source_ref`: checkbox version, provider consent field, staff note, prior customer preference, or policy record.
- `quiet_hours_or_timing_policy_ref`: if relevant for sends.
- `delivery_suppression_reasons`: opt-out, bounce/failure, STOP keyword, complaint, unknown consent, legal/policy gate, channel unsupported.

Channel rules:

- `Customer.preferred_contact` is a preference/input, not legal permission.
- A customer providing email or phone for identity/follow-up does not by itself approve marketing, SMS, or sensitive outbound communication.
- Chat in-session reply consent, if any, does not imply later SMS/email permission.
- Failed delivery or opt-out should create staff/manual retry work or reviewed replacement-channel recommendation; do not silently switch channels.

## Attachments, media, documents, and transcripts

Normalize attachments and transcripts by reference, not by embedding raw content:

| Content type | Normalized handling |
| --- | --- |
| Website/email/chat attachments | Store `attachment_ref`, filename/content-type/size/hash, source message id, scan status, extraction status, and sensitivity classification. Do not pass raw files to the inquiry agent. |
| MMS/media | Store media refs and minimal metadata; route photos/documents through approved media/document workflows when needed. |
| Phone/chat transcripts | Store transcript ref, transcript version/hash, speaker/diarization confidence, redacted excerpt, and uncertainty notes. |
| Vaccine/medical/payment/legal documents | Treat as sensitive boundary evidence; create document/review tasks when needed, but do not promote values into inquiry truth. |
| Screenshots/free-text claims | Evidence only; reconcile with trusted provider/domain records before policy/payment/booking decisions. |

The inquiry-intake prompt may include only a minimal redacted excerpt or structured summary needed to identify intent and missing information. Full bodies, full threads, raw OCR, raw provider payloads, recordings, images, and unredacted documents require a separate approved workflow and data-sent-to-runtime gate.

## Audit and redaction considerations

Required audit records for each normalized source:

- Source ingestion audit: source system, source record id, received/submitted timestamps, verification status, raw storage ref, raw payload hash, accepted/duplicate/rejected/conflict status.
- Normalization audit: normalizer version, canonical field list, redaction profile/version, source fingerprint, identity-match outcome, conflict markers, and policy snapshot ids.
- Runtime prompt manifest: event id, agent spec/version, output schema, field/category list, source/evidence refs, subject ids, allowed actions, review gates, and explicit omission/redaction categories. Store manifest/hashes/refs, not raw sensitive prompt text by default.
- Data fetch manifest: repository/tool, records fetched by ID, fields/categories returned, reason, policy version, actor/runtime identity, timestamp.
- Output/effect audit: workflow status, summary, citations, recommended tasks/drafts, risk flags, human-review reason, side-effect keys, approval ids if any, and validation result.

Redaction defaults:

- Logs should omit or hash customer email/phone, exact free text, transcript bodies, raw provider JSON, attachment content, payment refs, medical/vaccine details, staff-only notes, signatures/secrets, and browser/session identifiers unless an approved audit role needs them.
- Prompt/output raw retention should be disabled by default for production and separately approved if needed for debugging.
- Corrections should append new versions/audit records rather than rewriting prior source evidence used by a draft or decision.

## Inquiry-intake agent input packet

The final `docs/workflows/inquiry-intake-agent.md` artifact can use this minimum direct runtime packet:

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
    allowed_actions: [ReadEntities, CreateInternalTask, DraftCustomerMessage?, FlagRisk],
    forbidden_actions,
    review_gates,
    contact_permission_policy_ref,
    location_intake_policy_ref,
  },
  output_contract: "WorkflowResult<InquiryIntakeOutput>",
}
```

Fetch-by-ID may retrieve only approved inquiry/lead/customer/pet/request/policy snippets: customer/lead record, pet names/species, requested service/date, prior approved contact preference, and approved policy snippets. It should avoid full histories, whole message threads, raw documents, raw provider payloads, unredacted contact exports, payment data, and unrelated notes.

## Expected normalized output categories

The `inquiry-intake` agent should return a `WorkflowResult` whose structured output is limited to:

- Inquiry summary with citations/source refs.
- Missing-field checklist: customer contact, pet details, service/date details, vaccine/document need as a review/document request only, consent/preference uncertainty, location ambiguity, unsupported service/add-on ambiguity.
- Identity/reconciliation recommendation when customer/pet/source conflicts exist.
- Internal follow-up task recommendation with idempotency key and source evidence refs.
- Optional customer-message draft for acknowledgement/missing-info follow-up, explicitly `DraftOnly` or customer-message-approval gated.
- Risk flags and human-review reasons.
- Verification notes listing source confidence, redactions, unchecked sources, and policy gates.

It must not output:

- Booking confirmation, availability promise, waitlist/denial/final eligibility decision, price/deposit/payment instruction, refund/waiver/discount claim, vaccine/medical/behavior/group-play approval, provider mutation, or autonomous outbound send.

## Open gaps for downstream implementation

1. Add or explicitly map chat widget vocabulary in the domain model. Current enums do not include chat as a source/channel/delivery channel.
2. Define first-class inquiry/lead aggregate or decide whether inquiries become reservation requests immediately. Current model has `WorkflowSubject::External`, reservation `Inquiry` status, and provider lead refs, but no dedicated inquiry entity.
3. Define concrete consent/opt-out/quiet-hours data model beyond `Customer.preferred_contact`.
4. Define source-message/transcript/attachment evidence schemas and storage refs for website form, SMS, email, phone transcript, and chat.
5. Define identity matching/reconciliation confidence rules for supplied contact fields, provider owner ids, portal accounts, and duplicate leads.
6. Approve production data-sent-to-runtime field allowlist, model/provider, retention policy, and tool permissions before using real customer/pet/message data.
7. Decide whether a narrow deterministic receipt-only acknowledgement path exists. Until then, all inquiry acknowledgement/follow-up messages are drafts requiring human approval.

## Final synthesis rule

Normalize every inbound channel into `inquiry.received` only after preserving source provenance, idempotency, contact/consent metadata, evidence refs, redaction/audit manifests, and uncertainty. The normalized event may safely drive missing-info extraction, internal follow-up tasks, risk flags, and approval-gated draft replies. It must not erase source-specific consent/transcript/attachment differences, and it must not turn unverified free text into booking, payment, medical, vaccine, eligibility, or customer-message authority.
