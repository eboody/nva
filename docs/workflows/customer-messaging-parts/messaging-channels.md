# Messaging channels

Purpose: define the outbound customer communication channels the Customer Messaging Agent may target when creating drafts, approval packets, suppression reasons, and future approved send commands. This artifact is constrained by `docs/workflows/customer-messaging-parts/inputs.md`; it does not approve new integrations, autonomous sends, provider writes, payment actions, booking promises, or customer-portal script changes.

Status: draft channel contract for downstream Customer Messaging Agent definition cards. The conservative MVP posture is draft/review first: channel availability and customer preference are inputs, not legal consent or authority to send.

## Channel selection rules

Use channels only after the workflow has a normalized customer, destination reference, contact preference, consent/opt-out/quiet-hours state if available, prior delivery state, and an approved execution path for the message class.

General rules:

- Current supported outbound text channels are email, SMS, and customer portal/in-app/portal notices.
- WhatsApp is later optional only; it is not in the current `ContactChannel` enum or repo-local docs and must not be treated as available in MVP.
- Phone calls are not an AI outbound text channel in this artifact. They may appear as staff call tasks or future call scripts, but no automated call behavior is defined here.
- `Customer.preferred_contact` and `ContactChannel` are preferences/availability signals only. Do not infer consent, legal permission, quiet-hours eligibility, or template approval from them.
- Failed delivery should create a staff/manual retry task, reviewed replacement draft, or suppression reason. Do not silently switch channels without policy, consent, and audit.
- Infrastructure retries may retry the exact approved payload through the exact approved send path; content edits, channel changes, recipients changes, or policy changes require a new draft and approval.
- If no eligible channel exists, or required channel facts are missing, return a `no_send`/suppression reason and a staff follow-up task instead of filling gaps or guessing.

## Required channel fields for any draft or send candidate

Every channel-specific draft or future send command should carry these shared fields:

| Field | Requirement |
|---|---|
| `channel` | One of `Email`, `Sms`, or `Portal` for MVP. `WhatsApp` is not accepted until a later design adds it. |
| `customer_id` | Required canonical customer identity. |
| `destination_ref` | Required typed reference for the destination: email address ref, mobile phone ref, or portal account/provider customer ref. Avoid raw PII in prompts/logs when a ref is enough. |
| `location_id` / timezone | Required for location policy, display timing, quiet-hours checks, and audit. |
| `message_kind` | Required semantic intent, such as inquiry acknowledgement, missing-info request, daily/Pawgress update, grooming reminder, training follow-up, payment/deposit reminder, incident notice, checkout follow-up, or review-request candidate. |
| `subject_entities` | Required refs to relevant pet, reservation, document, incident, payment, task, or workflow event as applicable. |
| `source_evidence_refs` | Required refs to normalized records, approved staff notes, policy snapshots, provider/webhook evidence, or reviewed source text. Do not embed raw provider payloads unless explicitly redacted for review. |
| `consent_or_suppression_state` | Required before a customer-facing send candidate. If absent/ambiguous, draft-only plus staff review or no-send. |
| `approval_state` | Required: draft-only, customer-message approval required, manager approval required, approved-for-send token, or disallowed/suppressed. |
| `idempotency_key` | Required for any approved send path; key by semantic target, message kind, evidence/version, channel, and destination, not by raw body text. |
| `fallback_behavior` | Required explicit behavior for missing destination, opt-out, quiet hours, provider failure, stale facts, duplicate send, and unsafe content. |

## Email

### Intended use cases

Email is the best default for longer, structured, or attachment-adjacent customer communications where the customer has an email destination and policy permits contact. Source-derived candidate uses include:

- Inquiry/lead acknowledgement and follow-up when no booking promise is made.
- Missing information requests for pet profile details, preparation instructions, signed agreements, documents, or vaccine proof.
- Booking triage follow-up, waitlist/offer/confirmation-needed drafts, and policy explanations after human approval.
- Daily/Pawgress updates when the content is too long for SMS or includes a richer summary.
- Grooming reminders, prep instructions, rebooking follow-up, and lapsed-cadence winback drafts.
- Training parent follow-up, homework/next steps, program completion, and check-in drafts.
- Payment/deposit/balance reminders when source-backed and review-gated.
- Checkout/final report/follow-up and review-request candidates when eligibility/suppression checks pass.
- Incident/customer notices only through manager/customer-message approval.

### Constraints

- Email address availability is not consent. Consent, opt-out, suppression, quiet-hours, and location policy must be checked before sending.
- Email bodies, recipients, threads, and provider payloads are PII/provider content; prompts/logs should use evidence refs or redacted excerpts.
- Gingr `email_sent` webhook data is an observation of provider activity, not proof that the Customer Messaging Agent may send or that content was approved.
- Do not use email to communicate unverified medical, behavior, incident, payment/refund, booking availability, cancellation, waiver, discount, or policy-exception facts without the corresponding review gate.
- Do not attach or paste raw documents, OCR, payment payloads, internal staff notes, or high-PII provider JSON into generated email copy.

### Required fields

Email-specific draft/send candidates require:

- `to_email_ref`: typed customer email/contact ref; raw email should be minimized outside delivery boundary.
- Optional `cc/bcc` only if a later policy explicitly authorizes household/co-owner or staff copies. Default: none.
- `subject`: required.
- `body`: required customer-visible body.
- `reply_handling`: where replies land and whether replies create staff review tasks.
- `template_id` or `template_category`: required for deterministic sends; optional but recommended for drafts.
- Evidence refs for every customer-visible fact.

### Length and format expectations

- Subject: short, factual, and specific; avoid urgency language unless source-backed and approved. Recommended maximum about 80 characters.
- Body: plain-language paragraphs or short bullets. Keep routine emails concise; longer final reports may be structured but should still avoid raw internal notes.
- Include customer/pet names only when sourced and appropriate. Include dates/times with location timezone context.
- Use warm, operationally clear language; no unsupported promises, legal/medical conclusions, staff blame, or pressure.

### Subject needed

Yes. Email must carry a customer-safe subject line. If no safe subject can be generated from approved facts, the result should be draft-only and require review.

### Safe fallback behavior

- Missing/invalid email: do not switch automatically; create a staff follow-up task or evaluate another channel only under explicit policy.
- Consent/opt-out/quiet-hours unknown or negative: suppress send and create review/staff task.
- Provider/email failure or bounce: record delivery failure, preserve the approved payload, and route to staff/manual retry or reviewed replacement draft.
- Stale or conflicting source facts: suppress/send-block and request evidence review.

### Auto-send posture

Normally draft-only for MVP. A future deterministic auto-send path may be considered only for narrow, low-risk message classes with fixed template, verified destination, consent, suppression checks, approved policy, idempotency, provider audit, and no sensitive/policy-exception content. Incident, complaint, payment exception, refund/waiver, medical, behavior, booking confirmation/denial, and policy-exception emails should remain human-reviewed.

## SMS

### Intended use cases

SMS is for short, time-sensitive, low-detail prompts when the customer has a mobile phone destination and policy permits contact. Candidate uses include:

- Brief inquiry acknowledgement or "we received your request" drafts after approval if policy allows.
- Missing-information nudges that point the customer to staff/portal/email for details.
- Appointment or grooming reminder summaries where exact timing and template are approved.
- Arrival/check-in/check-out or checkout follow-up prompts when source-backed and low risk.
- Daily/Pawgress update notification that a fuller update is available by portal/email, rather than a sensitive full narrative.
- Delivery-failure follow-up or reply-review prompts routed to staff, not automatic conversation handling.

### Constraints

- Mobile phone availability is not SMS consent. SMS requires explicit consent/opt-out/quiet-hours and approved provider/send path.
- SMS is too short for nuanced medical, behavior, incident, payment dispute, refund, complaint, rejection, or policy-exception explanations. Route those to email/portal drafts with review or staff call tasks.
- Avoid sensitive details in SMS because lock-screen previews and shared phones can expose content.
- Do not continue multi-turn SMS conversations autonomously. Customer replies should create review/follow-up tasks unless a future policy defines a narrow deterministic reply handler.
- Do not use SMS for attachments, raw links to sensitive documents, card/payment secrets, or internal/provider identifiers.

### Required fields

SMS-specific draft/send candidates require:

- `mobile_phone_ref`: typed customer mobile phone/contact ref; raw number minimized outside delivery boundary.
- `body`: required; no subject.
- `consent_snapshot_ref`: required for sends.
- `quiet_hours_decision`: required for sends or schedule candidates.
- `short_link_ref` only if a later approved link policy exists; avoid arbitrary raw URLs in generated text.
- `reply_handling`: required; defaults to staff review task.
- `template_id` or `template_category`: required for deterministic sends; optional but recommended for drafts.

### Length and format expectations

- No subject.
- Keep content concise and single-purpose. Recommended maximum is one carrier segment target (about 160 GSM-7 characters) for routine reminders; multipart SMS should be exceptional and reviewed.
- Use plain text only; avoid emojis unless a later brand/template policy explicitly approves them.
- Include only the minimal fact needed: pet/customer-friendly greeting, reason, date/time if needed, and next action.
- If a message needs nuance, evidence, multiple pets, policy explanation, or sensitive context, use a staff-reviewed email/portal draft instead.

### Subject needed

No. SMS has body only. Do not simulate a subject line inside the message.

### Safe fallback behavior

- Missing/invalid mobile phone: do not switch automatically; create staff follow-up or evaluated channel replacement only under explicit policy.
- Consent/opt-out/quiet-hours unknown or negative: suppress send and create review/staff task.
- Message too long or sensitive: downgrade to draft-only and recommend email/portal/staff review.
- Provider failure, bounce, carrier rejection, or no receipt: record failure and route to staff/manual retry or reviewed replacement draft; do not regenerate altered SMS text on retry.
- Customer reply: create staff/customer-reply review task unless a future deterministic handler is approved.

### Auto-send posture

Draft-only by default. SMS is the most likely future candidate for narrow deterministic sends (for example, receipt-only acknowledgements or routine reminders), but only with explicit consent, quiet-hours enforcement, fixed template, source-backed facts, suppression checks, idempotency, and provider audit. Sensitive, exception-bearing, or customer-specific judgment messages should normally remain draft-only/human-approved.

## Customer portal / in-app / portal notices

### Intended use cases

Portal notices are for customer-facing updates inside the customer portal/provider app context when a portal account/provider customer ref exists and policy permits the notice. Candidate uses include:

- Portal-visible missing-information, document, signature, vaccine, or profile-completion prompts.
- Reservation/request status notices and follow-up tasks that avoid making unapproved booking promises.
- Daily/Pawgress update availability notices or portal-hosted update drafts after staff review.
- Checkout/final report, receipt, or follow-up prompts when unresolved incidents/payment/care facts are cleared.
- Review-request eligibility notices if suppression checks pass.
- Staff-reviewed incident, care, or policy notices when the portal is the approved delivery surface.

### Constraints

- Current repo sources include `ContactChannel::Portal`, `PortalAccountRef`, Gingr Customer Portal docs, and portal JavaScript event observations, but they do not approve installing scripts or sending portal messages from this agent.
- Portal browser/JavaScript events are observational signals only. They are not authoritative operational state and cannot by themselves justify a send, booking mutation, payment action, or document decision.
- Portal notice mechanics, provider APIs, inbox semantics, read receipts, push notifications, and retention behavior are not fully defined in current repo artifacts.
- Portal content may still be customer-facing PII. It must follow the same source-grounding, approval, redaction, and audit requirements as email/SMS.
- Do not rely on portal notice as a safe fallback when email/SMS consent is missing unless portal-only contact policy is explicitly modeled.

### Required fields

Portal-specific draft/send candidates require:

- `portal_account_ref` or provider/customer portal destination ref.
- `notice_title` or display heading if the portal surface supports one.
- `body`: required customer-visible content.
- `placement` or `surface` if known, such as inbox, reservation detail, document task, daily update, or checkout/final report. If unknown, mark unresolved instead of guessing.
- `read_receipt_or_status_ref` only if provider semantics exist.
- `reply_or_action_handling`: required; defaults to staff/provider review task.
- `template_id` or `template_category`: required for deterministic sends; optional but recommended for drafts.

### Length and format expectations

- Notice title/heading: short and factual; recommended maximum about 60 characters.
- Body: can be longer than SMS but should still be concise, structured, and customer-safe. Use bullets for task lists or next steps.
- Include dates/times with location timezone context. Do not expose raw provider IDs, staff-only notes, or internal review commentary.
- If the portal notice links to forms/documents/payments, the link/action must come from an approved portal/provider workflow, not generated free text.

### Subject needed

Not an email subject, but a portal notice usually needs a title/heading if the surface supports one. If the surface does not support titles, omit it and keep the body self-contained.

### Safe fallback behavior

- Missing portal account/ref or unknown portal surface: create staff follow-up or choose another channel only under explicit policy.
- Portal provider/API unavailable: record provider-unavailable suppression/failure and route to staff/manual retry or reviewed replacement draft.
- Ambiguous portal event/state: do not send or mutate; request provider/state reconciliation.
- Content needs document/medical/behavior/incident/payment/manager review: draft-only with required gate.

### Auto-send posture

Draft-only for MVP. Portal notices may become deterministic for low-risk task prompts after a concrete portal delivery adapter, consent/portal-only policy, template, placement, read/reply handling, and audit semantics are approved. Portal script installation or browser event handling is not in scope for this messaging channel definition.

## WhatsApp (future/later option only)

WhatsApp is not a supported MVP outbound channel. It is absent from the current repo-local `ContactChannel` enum and from the source docs checked in the input packet.

WhatsApp may be considered later only if it is useful for a specific pilot and a separate design defines:

- A typed `ContactChannel`/destination model and provider adapter.
- Explicit WhatsApp consent, opt-out, quiet-hours, template/session-window, and locale semantics.
- Approved message templates and template-variable validation where provider rules require them.
- Provider delivery, failure, reply, and audit events.
- Safe fallback behavior that does not silently substitute SMS/email/portal without consent and review.
- Whether AI may only draft WhatsApp copy or whether any deterministic provider send path is allowed.

Until that design exists, any customer preference or staff note mentioning WhatsApp should produce a staff follow-up/task or unsupported-channel suppression reason, not an outbound send candidate.

## Cross-channel fallback and approval matrix

| Condition | Safe behavior |
|---|---|
| Preferred channel unavailable | Do not guess. Evaluate alternate channel only if consent, destination, policy, and audit path are present; otherwise create staff follow-up. |
| Consent/opt-out/quiet-hours missing | Draft-only or no-send with staff/customer-contact review task. |
| Sensitive content present | Require the applicable gate: customer-message, manager, medical document, behavior, refund/deposit exception, incident, complaint, or legal/privacy review. |
| Provider delivery failure | Preserve approved payload, record failure, create manual retry/replacement review task; do not silently alter content/channel. |
| Duplicate semantic target | Suppress as duplicate unless a newer event/evidence version and approval justify a replacement. |
| Stale/conflicting source facts | Suppress/send-block and request evidence reconciliation. |
| No approved deterministic send path | Produce draft, review packet, suppression reason, or staff task only. |

Default auto-send posture by channel:

| Channel | MVP send posture |
|---|---|
| Email | Normally draft-only/human-approved. Future deterministic sends only for narrow low-risk templates. |
| SMS | Draft-only by default; possible future deterministic routine reminders/acknowledgements with strict consent, quiet-hours, template, idempotency, and audit. |
| Portal | Draft-only until portal delivery adapter, placement semantics, consent policy, and audit are approved. |
| WhatsApp | Unsupported/future optional only. |

## Implementation implications

- Model channel selection as a policy decision with explicit suppression reasons, not a helper that picks the first available address/number.
- Keep draft creation separate from send execution. `DraftMessage`/`DraftCustomerMessage` is not a send.
- Store approved sends in a durable outbox/approved-action record with immutable approved payload, approval actor, policy version, destination ref, idempotency key, provider response refs, and audit events.
- Treat provider/webhook/browser events as boundary inputs until verified and semantically mapped.
- Make fallback behavior explicit on every result so staff can see whether the agent drafted, suppressed, escalated, or requested manual follow-up.
