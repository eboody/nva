# Customer messaging agent

> Successor route: this is a detailed specification/supporting-proof artifact, not the current reader spine. Start with the [docs successor and archive map](../design/successor-archive-map.md#older-workflow-and-specification-docs), [workflow-to-entity map](../design/workflow-to-entity-navigation-map.md), and [operator workflow index](operator/README.md) before using this page for current claims.

Status: integration artifact synthesized from the Customer Messaging Agent definition cards. This document is a workflow/specification artifact only. It does not authorize live customer/member-facing sends, provider writes, reservation changes, payment actions, refunds, waivers, medical/vaccine decisions, incident closure, policy exceptions, or customer-portal script changes.

Source artifacts:

- `docs/workflows/customer-messaging-parts/inputs.md`
- `docs/workflows/customer-messaging-parts/messaging-channels.md`
- `docs/workflows/customer-messaging-parts/message-categories.md`
- `docs/workflows/customer-messaging-parts/send-draft-approval-policy.md`
- `docs/workflows/customer-messaging-parts/message-generation-schema.md`
- `docs/workflows/customer-messaging-parts/tone-and-compliance-rules.md`

## Central messaging spec

The Customer Messaging Agent is a draft/review packet generator first. It receives normalized workflow events and source evidence, selects or suppresses a customer-message category, prepares channel-appropriate draft copy when safe, and returns structured proof of facts used, approval gates, suppression reasons, and audit/idempotency metadata.

Default authority:

- Allowed by default: read normalized records; summarize source evidence; generate draft customer copy; generate internal staff tasks; generate manager/reviewer packets; generate suppression/no-send reasons; identify missing facts and required approvals.
- Not allowed by default: customer-facing send, provider/system mutation, reservation confirmation/rejection, room/group assignment, payment charge/refund/waiver/discount, medical/vaccine/document eligibility decision, behavior/group-play eligibility decision, incident disposition, customer-portal script installation, or policy exception.
- A draft is not a send. Current domain primitives such as `RecommendedAction::DraftMessage` or `OperationsAction::DraftCustomerMessage` are reviewable recommendations only.
- Model confidence, urgency, customer pressure, positive sentiment, or channel availability is never authority to send or approve.

Canonical inputs for every message decision:

- Workflow envelope: workflow event id/type/time, location id/timezone, actor, subject refs, allowed actions, automation level, policy context, required review gates, idempotency scope, and audit refs.
- Customer/contact: customer id, display name, preferred contact channel, email/mobile/portal destination refs, redacted destination display when needed for review, consent/opt-out state, quiet-hours state, delivery suppression/failure state, and reply-handling path.
- Pet/reservation/service: pet ids and names, species/service kind, reservation/stay/check-in/check-out context, requested or approved dates/times, add-ons, documents, hard stops, care profile, feeding/medication/allergy notes, behavior/incident flags, and source freshness.
- Evidence and policies: normalized domain records, verified provider mappings, policy snapshots, approved staff notes, manager/reviewer decisions, payment/provider refs, prior draft/send/outbox refs, and redacted source excerpts when needed.
- Missing/unsafe inputs: absent consent model, unknown quiet hours, stale/conflicting provider facts, unreviewed OCR/raw documents, raw provider JSON, raw email bodies, card/payment secrets, API keys, webhook signatures, or high-PII payloads must not be used as customer-visible truth.

Required outputs:

- A typed structured output object using the message generation schema below.
- Zero or more customer-message drafts, each with category, channel, recipient, subject/body where appropriate, fact citations, forbidden-claim checks, approval state, and audit refs.
- Suppression/no-send reasons when no safe customer copy should be produced.
- Internal staff/reviewer tasks when information, approval, or source reconciliation is required.
- Risk flags and verification notes for any sensitive, ambiguous, stale, conflicting, or unsupported facts.

## Send, draft, and approval rules

### Core stance

All customer-facing messages are draft/review outputs unless both unresolved gates below are explicitly approved and the exact category/channel/template/fact set is covered by deterministic policy.

A customer-facing send is allowed only when all of these are true:

1. The message category is in an approved auto-send category set for the location and channel.
2. The final copy is an approved deterministic template or template-bound variant.
3. Required facts are present, current, source-backed, non-conflicting, and approved for customer copy.
4. Recipient, destination, channel, consent/opt-out state, quiet-hours policy, and prior delivery suppression state pass.
5. No sensitive, exceptional, ambiguous, or manager-gated language is present.
6. Idempotency, audit, outbox, retry, provider response, and immutable approved-payload records are available.

If any condition is missing, stale, ambiguous, conflicting, unsupported, or outside policy, the agent must not send. It should draft, suppress, or route to the proper human review path.

### Explicit unresolved human approval gates

These are open gates, not approved operational facts:

- Gate A — approved auto-send category set: product/operations leadership must approve the exact categories, workflow triggers, templates, variables, channels, consent/quiet-hours rules, evidence requirements, suppression conditions, idempotency scope, retry behavior, audit fields, and change owner before any automated delivery is enabled. Until this gate is approved, all customer-facing messages are drafts or staff tasks.
- Gate B — legal/medical-sensitive language policy: management and legal/compliance where applicable must approve language policy for medical, medication, vaccine/document eligibility, injury, safety, behavior/aggression, incidents, privacy, liability, refunds/waivers, rejections/declines, or policy exceptions. Until this gate is approved, the agent may not auto-send any legal/medical-sensitive language; it may only prepare internal summaries and manager-reviewed drafts with evidence refs and risk flags.

### Send-mode taxonomy

- `draft_for_review`: default customer-message posture; copy can be reviewed and edited by staff/manager.
- `internal_review_packet`: no customer copy is sent; evidence, risks, and suggested wording are routed to an approver.
- `staff_call_task`: phone/manual contact is needed; not an AI outbound text send.
- `deterministic_send_candidate`: possible only after Gate A, fixed template policy, consent/quiet-hours/suppression checks, safe category, idempotency, and audit are proven.
- `suppressed` / `no_send`: no customer copy should be produced or sent because channel, consent, facts, policy, sensitivity, or suppression checks fail.

### Approval routing

- `CustomerMessageApproval`: default for customer/member-facing drafts.
- `ManagerApproval`: incidents, safety-sensitive content, capacity/ratio/holiday/waitlist/booking promises, complaints, public responses, declines/rejections/restrictions, policy exceptions, sensitive tone, and most final customer-message approvals not covered by future deterministic policy.
- `MedicalDocumentReview`: vaccine/document proof, medication/allergy/medical ambiguity, care instruction uncertainty, and any message that states document or medical eligibility status.
- `BehaviorReview`: temperament, group-play, bite/aggression, restrictions, reinstatement, behavior-sensitive eligibility, and safety-impacting behavior facts.
- `RefundOrDepositException`: refunds, waivers, credits, discounts, forfeitures, payment disputes, deposit exceptions, amount conflicts, and payment-sensitive exceptions.
- Legal/compliance/privacy review: legal threats, liability/fault language, regulatory/privacy/security exposure, raw PII/payment leaks, public responses with sensitive facts, and Gate B categories when local policy requires specialist review.
- Engineering/integration owner: missing send adapter, webhook verification failures, stale/missing policy snapshots, validator defects, template/version mismatches, idempotency conflicts, provider delivery inconsistencies, or unsupported channel behavior.

### Retry and outbox rules

- Approved sends must use a durable approved-action/outbox record with immutable approved payload, approval actor, policy version, destination ref, idempotency key, provider response refs, and audit events.
- Infrastructure retries may retry delivery of the exact approved payload only.
- Content edits, channel changes, recipient changes, policy changes, regenerated copy, or altered facts require a new draft and new approval.
- Error/dead-letter views shown to staff/engineering must be redacted and must not expose exact outbound body, raw provider content, payment secrets, or high-PII payloads unless an approved secure review surface requires it.

## Output schema

Each generated customer-message draft or no-send result should be embedded in the workflow `structured_output`. The current domain `DraftMessage` primitive may carry only channel/body; the richer object below must remain available for review, validation, and audit.

Initial schema version: `customer_message_generation.v1`.

```json
{
  "schema_version": "customer_message_generation.v1",
  "draft_id": "draft_123",
  "workflow_event_id": "evt_456",
  "category": "inquiry",
  "send_mode": "draft_for_review",
  "channel": "email",
  "recipient": {
    "customer_id": "customer:123",
    "display_name": "Jordan Lee",
    "destination_ref": "customer:123.email.primary",
    "destination_redacted": "jo***@example.com",
    "portal_account_ref": null,
    "consent_state": "allowed",
    "quiet_hours_state": "send_window_open",
    "delivery_suppression_state": "none"
  },
  "subject": "Question about Milo's boarding request",
  "body": "Hi Jordan — thanks for reaching out about Milo's boarding request. We have the request in review and our team will follow up if we need any additional details before confirming next steps.",
  "requires_approval": true,
  "approval_reason": "CustomerMessageApproval required for customer-facing inquiry drafts in manual-v1.",
  "required_approvals": ["CustomerMessageApproval"],
  "facts_used": [],
  "forbidden_claims_checked": [],
  "confirmed_facts": [],
  "requested_information": [],
  "sensitive_language": [],
  "omitted_or_suppressed_facts": [],
  "reviewer_notes": [],
  "risk_flags": [],
  "validation": {
    "status": "needs_human_review",
    "validator_version": "customer_message_validator.v1",
    "failures": [],
    "warnings": []
  },
  "audit": {
    "policy_version": "policy_or_snapshot_ref",
    "template_key": "inquiry.receipt_or_followup.v1",
    "template_version": null,
    "source_evidence_refs": [],
    "input_snapshot_refs": [],
    "idempotency_key": "semantic-key",
    "prior_draft_or_send_refs": [],
    "outbox_ref": null
  },
  "fallback_behavior": {
    "missing_destination": "staff_follow_up_task",
    "unknown_consent": "suppress_or_review",
    "quiet_hours_closed": "hold_or_review_only_if_policy_exists",
    "provider_failure": "manual_retry_or_reviewed_replacement",
    "stale_or_conflicting_facts": "suppress_and_request_reconciliation"
  }
}
```

Field rules:

- `schema_version`, `workflow_event_id`, `category`, `send_mode`, `channel`, `recipient`, `requires_approval`, `approval_reason`, `required_approvals`, `facts_used`, `forbidden_claims_checked`, `validation`, `audit`, and `fallback_behavior` are required for any draft or send candidate.
- `channel` is one of `email`, `sms`, or `portal`. `phone` is staff call-task only unless a later schema explicitly authorizes call scripts. `whatsapp` is invalid until a typed consent/provider/template/audit design exists.
- Email requires a non-empty `subject`; SMS requires `subject = null`; portal may include a title/subject only when the surface supports it.
- `body` is required for customer-visible drafts. Suppression/no-send outputs may use an internal no-send rationale instead of projecting a `DraftMessage`.
- `requires_approval` defaults to `true`. It may be `false` only when deterministic send authority is proven by approved category/template/facts/channel/consent/quiet-hours/suppression/idempotency/audit policy.
- `approval_reason` must name the specific approval gate or missing authority when `requires_approval = true`.
- `facts_used` must cite every customer-visible factual claim with source ref, source kind, trust state, sensitivity, freshness, and whether the fact is approved for customer copy.
- `forbidden_claims_checked` must cover baseline forbidden classes and category-specific risks. Any `blocked` check suppresses or rewrites the draft; any `requires_approval` check must be reflected in approval state; any `not_checked` check fails send readiness.
- `confirmed_facts`, `requested_information`, `sensitive_language`, and `omitted_or_suppressed_facts` must remain separate so reviewers can see what is known, what is being asked, what requires special approval, and what was intentionally withheld.

Baseline forbidden claim classes:

- Unsupported fact or hallucination.
- Booking confirmation, acceptance, availability, capacity, room/group, or waitlist promise.
- Payment charge, refund, waiver, deposit exception, discount, price, package, or payment-status claim without approved evidence.
- Medical diagnosis, vaccine/document eligibility determination, medication instruction, treatment advice, allergy/medical conclusion, or veterinary advice.
- Behavior, group-play, safety, aggression, restriction, or reinstatement determination without approval.
- Incident fault, liability, legal, privacy, staff-blame, or confidential-detail claim.
- Provider write, reservation status change, system mutation, or customer-portal script behavior.
- Policy exception, location-specific rule, deadline, cancellation rule, or no-show/refundability language without policy snapshot and approval.
- Unauthorized consent, opt-out, quiet-hours bypass, channel switch, duplicate send, or provider retry change.
- Raw provider/payment/OCR/webhook secret exposure or unnecessary PII disclosure.
- Unsupported reassurance, outcome guarantee, service recovery promise, review pressure, or public-response claim.

## Channel rules

### Shared channel requirements

Every channel candidate must include:

- `channel`: `email`, `sms`, or `portal` for MVP.
- Customer id and typed destination reference.
- Location id/timezone for policy, quiet-hours, display timing, and audit.
- Message category and subject entity refs.
- Source evidence refs for every visible fact.
- Consent/opt-out/suppression state before any send candidate.
- Approval state and required gates.
- Idempotency key for approved send paths.
- Explicit fallback behavior for missing destination, opt-out, quiet hours, provider failure, stale facts, duplicate target, and unsafe content.

Channel selection must be a policy decision, not a helper that chooses the first available email or phone. Customer preference and channel availability are inputs only; they are not legal permission, quiet-hours eligibility, or template approval.

### Email

Intended uses: longer or structured inquiry follow-up, missing information requests, booking/waitlist/offer drafts, daily/Pawgress updates, grooming reminders, training parent follow-up, payment/deposit drafts, checkout/final report/follow-up, review-request candidates, and incident/customer notices after approval.

Rules:

- Requires `to_email_ref`, `subject`, `body`, `reply_handling`, template key/category when available, and evidence refs for all customer-visible facts.
- Email address availability is not consent.
- Email bodies, recipients, threads, and provider payloads are PII/provider content; prompts/logs should use evidence refs or redacted excerpts.
- Do not attach or paste raw documents, OCR, payment payloads, internal notes, or high-PII provider JSON.
- Normally draft-only in MVP. Future deterministic sends only for narrow low-risk templates with verified destination, consent, suppression checks, approved policy, idempotency, provider audit, and no sensitive/exception content.

### SMS

Intended uses: short, time-sensitive, low-detail prompts such as brief receipt acknowledgements, missing-information nudges, routine reminders, check-in/out prompts, daily-update availability notices, or reply-review prompts when policy permits.

Rules:

- Requires `mobile_phone_ref`, `body`, `consent_snapshot_ref` for sends, `quiet_hours_decision`, `reply_handling`, and template key/category when available.
- `subject` must be `null`.
- Keep content concise, plain text, and single-purpose; routine reminders should target one carrier segment when practical.
- Avoid sensitive details because lock-screen previews and shared phones can expose content.
- Do not conduct autonomous multi-turn SMS conversations. Replies create staff/customer-reply review tasks unless a future deterministic handler is approved.
- Draft-only by default. Future deterministic SMS is possible only for narrow routine templates with consent, quiet hours, suppression, idempotency, provider audit, and no sensitive judgment.

### Customer portal / in-app / portal notices

Intended uses: portal-visible missing-information/document/profile prompts, reservation/request status notices, daily/Pawgress update availability, checkout/final report prompts, review-request eligibility notices, and staff-reviewed incident/care/policy notices when portal is the approved surface.

Rules:

- Requires `portal_account_ref` or provider/customer portal destination ref, body, reply/action handling, placement/surface if known, and notice title/heading when the surface supports one.
- Current repo sources establish portal refs/events but do not approve installing scripts or sending portal messages from this agent.
- Portal browser/JavaScript events are observational signals only and cannot justify sends, booking mutations, payment actions, or document decisions by themselves.
- Portal mechanics, provider APIs, inbox semantics, read receipts, push notifications, and retention are not fully defined; unknown placement/mechanics must be marked unresolved.
- Draft-only until portal delivery adapter, placement semantics, consent policy, template, read/reply handling, and audit semantics are approved.

### Phone and WhatsApp

- Phone is not an AI outbound text channel in this artifact. It may produce staff call tasks or future call scripts only if separately scoped.
- WhatsApp is unsupported/future-only. It is absent from the current `ContactChannel` enum and source docs. Any WhatsApp preference should create staff follow-up or unsupported-channel suppression until typed consent, provider, template/session-window, locale, delivery/failure/reply, and audit semantics exist.

### Cross-channel fallback

- Do not silently switch channels after a missing destination, opt-out, quiet-hours conflict, provider failure, or delivery bounce.
- Alternate channel evaluation requires explicit policy, consent, destination, suppression checks, approval state, and audit.
- Provider retry may resend only the exact approved payload through the exact approved path.
- Stale/conflicting source facts, duplicate semantic target, or unsafe content must suppress/send-block and route to staff/reviewer reconciliation.

## Category taxonomy

Current source-derived categories are candidates, not approved production templates.

| Category | Typical triggers | Default mode | Required approvals and caveats |
| --- | --- | --- | --- |
| `inquiry` | `inquiry.received`, verified `lead_created`, website/portal/email/SMS/phone/staff-created lead | Draft for review; later deterministic receipt-only acknowledgement may be considered | `CustomerMessageApproval` by default. No booking, availability, price/deposit, vaccine/eligibility, policy exception, or provider mutation promise. |
| `missing_info` | Missing intake, pet profile, reservation, document/vaccine, contact preference, signature/agreement, care detail, payment reference, or source evidence | Draft for review; narrow deterministic templates may be considered for low-risk collectable fields | `CustomerMessageApproval`; `MedicalDocumentReview` for vaccine/medical ambiguity; manager/payment gates for policy-sensitive or payment asks. Ask only for specific missing items. |
| `vaccine_reminder` | Document upload/extraction, vaccine pending, booking triage hard stop or review task | Draft for review | `MedicalDocumentReview` for accepting/denying vaccine facts; `CustomerMessageApproval` for copy; `ManagerApproval` for disputes/status impact. Upload is not verification; no medical advice. |
| `booking_offer_confirmation` | `booking.confirmation_needed`, staff-approved offer, readiness candidate, provider state with approved execution path | Draft for review; deterministic only after approved booking state/template facts | `CustomerMessageApproval` always; `ManagerApproval` for capacity/waitlist/special-care/denial/policy copy; payment/document/behavior gates as applicable. Separate offer from confirmed booking. |
| `deposit_request` | Required/unpaid/failed/expired deposit or balance/deposit collection task | Draft for review unless exact deterministic payment-reminder path exists | `CustomerMessageApproval`; `RefundOrDepositException` for waivers/refunds/forfeitures/discounts/disputes; manager gate for stale/missing policy or consequences. No threats, raw card data, or unsupported deadlines. |
| `waitlist` | Waitlisted triage outcome, capacity/staffing/holiday block, waitlist fill opportunity | Draft for review | `CustomerMessageApproval`; `ManagerApproval` for capacity exceptions, overbooking, priority disputes, denial/rejection, or policy exceptions. No promise of position, timing, availability, or priority unless modeled and approved. |
| `pre_arrival` | Upcoming approved reservation/check-in prep/missing requirement reminder | Draft for review; low-risk deterministic reminders may be considered after Gate A | `CustomerMessageApproval`; medical/vaccine/medication/behavior/payment/policy-sensitive content needs matching gate. Do not imply unresolved issues are cleared. |
| `daily_update` | `daily_note.created`, `daily_update.needed`, care notes/media/Pawgress task | Draft for staff review | `CustomerMessageApproval`; manager/care/medical/behavior review for incident, safety, medication, health, behavior, media/privacy, or missing evidence. No fabricated cheerful filler. |
| `incident_draft` | Incident event/report/follow-up task or daily update with incident facts | Draft-only, review-required | `ManagerApproval` plus `CustomerMessageApproval` for all owner-facing copy; medical/behavior/legal/privacy gates as needed. No autonomous owner incident send. |
| `checkout` | `checkout.completed`, final report, receipt/rebooking/follow-up candidate | Draft for review; receipt/report deterministic only with approved source/template path | `CustomerMessageApproval`; payment/manager gates for balances/refunds/waivers; incident/medical/behavior review for unresolved facts. Do not hide unresolved issues. |
| `review_request` | `review_request.eligible`, clean checkout/final follow-up, reputation workflow after suppression checks | Draft for review or deterministic only after strict eligibility/suppression policy | `CustomerMessageApproval`; `ManagerApproval` for complaints, negative sentiment, incidents, payment disputes, unresolved care issues, or public-response implications. Suppress when the ask would be insensitive. |
| `suppression_no_send` | No safe channel/facts/policy/category, opt-out/quiet-hours/suppression, unsupported channel, stale/conflicting source, blocked forbidden claim | Suppressed/no-send plus internal task or review packet | Name missing or unsafe condition; do not project internal rationale as customer copy. |

Preserved caveats:

- No approved template catalog, consent/opt-out model, quiet-hours policy, or WhatsApp support exists in current repo sources.
- Public PetSuites-style details are not pilot-approved local policy. Exact prices, timing, refund/deposit language, no-show rules, ratios, reminders, and customer-facing copy remain location/policy data.
- Low-risk categories can still smuggle commitments; inquiry, missing-info, and pre-arrival copy must not imply availability, eligibility, vaccine acceptance, payment status, or policy commitments.
- Booking, deposit, waitlist, checkout, and review-request copy can change customer expectations and therefore needs trusted source facts and appropriate review.
- Daily updates must not smooth over incidents, health/medication exceptions, behavior concerns, media/privacy issues, or missing evidence.
- Incident drafts are always review-gated and may require manual phone/emergency procedures outside this agent.

## Tone and compliance rules

Customer copy must be warm, pet-parent-friendly, operationally clear, concise, and truthful.

Rules:

1. Ground every customer-visible statement in approved facts. Do not fill gaps from general pet-care knowledge, public brand assumptions, previous drafts, model confidence, likely business practice, or customer pressure.
2. Never provide diagnosis, veterinary advice, medication instructions, vaccine/document eligibility conclusions, or medical interpretations unless exact approved reviewer wording is supplied for customer copy.
3. Do not guarantee outcomes, booking availability, room/group placement, refunds, payment timing, grooming/training results, response times, special handling, group-play eligibility, waitlist movement, incident resolution, or policy exceptions unless explicitly approved.
4. Avoid blame, fault admission, legal/liability conclusions, threats, staff/customer/pet shaming, or public-response language without manager/legal/privacy approval.
5. Keep channel fit: SMS short and single-purpose; email structured and concise; portal tied to the active surface; phone only as staff task unless separately scoped.
6. Separate confirmed facts, requested information, sensitive language, omitted/suppressed facts, review gates, and channel-fit notes in the review packet.
7. Escalate sensitive or ambiguous content instead of smoothing it over. Warm tone must not hide concerning facts or convert uncertainty into reassurance.
8. Minimize data. Do not expose raw provider JSON, raw OCR, raw email bodies, unredacted documents, internal notes, payment provider refs, other customers/pets, staff debate, legal/compliance notes, or unrelated history.

Worker pre-send/review checklist:

- Every factual claim maps to a trusted source ref, policy snapshot, approved evidence ref, or manager-approved text.
- Missing, ambiguous, stale, conflicting, likely, or assumed details are omitted from customer copy and captured internally.
- The action remains within draft/review/suppression/internal-task authority unless an explicit deterministic send policy covers it.
- Required review gates are present.
- Medical/safety, promise, blame/legal, tone/channel, privacy, and audit/idempotency checks pass.
- If any check is unknown or fails, output a review/missing-info/suppression reason instead of customer-ready copy.

Reviewer approval checklist:

- Final text matches approved source facts and does not add model-invented facts.
- Sensitive wording has the right approval owner.
- Channel is permitted by contact preference, consent/opt-out, suppression state, quiet-hours/location policy, and provider/send-path policy.
- Any promised next step, deadline, refund, booking status, service availability, care accommodation, special handling, or policy exception is explicitly authorized.
- Approval records preserve reviewer, timestamp, final text/version, source refs, edits, reason, policy version, idempotency key, and send/suppression disposition.
- Retries resend only exact approved payloads; any content change creates a new draft and approval.

## Validation and implementation notes

- Implement deterministic validators before any review/send projection: JSON schema validation, channel/recipient policy validation, consent/quiet-hours/suppression validation, fact-to-copy coverage, forbidden-claim coverage, sensitivity/review-gate validation, length/channel formatting, redaction checks, template/version checks, and idempotency checks.
- Prefer a dedicated future domain type such as `GeneratedCustomerMessageDraft` rather than overloading `RecommendedAction::DraftMessage`, because the current primitive loses recipient, subject, approval, citation, and forbidden-claim proof.
- Persist generated outputs with stable draft id, workflow event id, policy version, prompt/input snapshot ref, model/version ref, validator version, template/version, idempotency key, source evidence refs, reviewer decisions, and outbox/send refs when applicable.
- Use semantic idempotency keys based on category, subject refs, workflow event/evidence version, template version, policy version, channel, and destination; never key by raw message text alone.
- Treat provider/webhook/browser events as boundary inputs until verified, HMAC-checked where applicable, sanitized, and mapped into normalized domain records.
- Queue/retry/dead-letter infrastructure must preserve policy gates and redaction boundaries. Infrastructure failure must not weaken review requirements, regenerate approved payloads, silently change channels, or expose high-PII fields.

## Unresolved items for product/operations

The current source cards intentionally leave these unresolved rather than inventing operational policy:

- Approved auto-send category set, including exact category names, triggers, channels, templates, variables, suppression conditions, idempotency, retry behavior, and owner for template/policy changes.
- Legal/medical-sensitive language policy, including who approves wording for medical/vaccine/document, medication/allergy, injury/incident/safety, behavior/aggression, privacy/liability, refunds/waivers, rejections/declines, and policy exceptions.
- Consent, opt-out, quiet-hours, frequency cap, and channel-specific contact permission model.
- Approved production template catalog and per-location brand copy.
- Portal delivery adapter, placement semantics, read/reply handling, retention, portal-only contact policy, and audit behavior.
- WhatsApp support, if any, including typed channel model, consent, provider, templates/session windows, locale, reply handling, and audit.
- Dedicated message aggregate/outbox/review UI fields beyond the current minimal draft-message primitive.
