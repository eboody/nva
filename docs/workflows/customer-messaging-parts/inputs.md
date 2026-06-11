# Customer messaging inputs

Purpose: canonicalize the source inputs downstream Customer Messaging Agent definition cards should use. This is an input packet, not live operating policy. It does not authorize sending customer/member-facing messages, changing reservations, changing provider state, collecting payments, or installing customer-portal scripts.

Status: draft source inventory and constraints note for downstream definitions. Facts below come from the repo-local docs and Rust domain surfaces checked for this card; missing sources are called out explicitly.

## Source inventory

Primary repo sources checked:

- `docs/architecture/pet-resort-workflow-events.md` — canonical workflow-event envelope, allowed AI/workflow actions, event-to-message implications, and conservative approval posture for customer-facing outbound sends.
- `docs/data-model/workflow-queue-retry-dead-letter.md` — durable queue/outbox model for workflow results, draft messages, approved customer sends, retries, dead letters, idempotency, and redacted failure visibility.
- `docs/workflows/staff-operations-parts/inputs.md` — product posture, manual-v1 approval gates, staff roles, daily update/Pawgress state, checkout/customer-follow-up surfaces, and current domain-data summary.
- `docs/workflows/payments-pricing.md` and `docs/workflows/payments-pricing-parts/ai-boundaries.md` — payment/deposit/customer-reminder truth, authority, execution, escalation, and redaction boundaries.
- `docs/integrations/gingr/sdk-readiness-review.md` — Gingr v0 readiness, read-only/prototype boundary, sandbox gates, high-PII/privacy caveats, and human-reviewed execution-only surfaces.
- `docs/integrations/gingr/sdk-webhooks.md` — Gingr webhook event names, `email_sent` payload caveat, HMAC verification/typestate gate, retry semantics, PII/sanitization rules, and raw-payload quarantine.
- `docs/integrations/gingr/sdk-customer-portal-js.md` — customer-portal JavaScript boundary, portal event names, privacy caveats, and distinction between read API, webhooks, and browser/customer-facing portal scripts.
- `docs/domain/petsuites/boarding/service-domain-map.md` — public PetSuites/NVA service vocabulary, Pawgress Report/customer-update surface, add-ons/upsell wording constraints, and draft-only AI posture.
- `docs/domain/petsuites/grooming/implications/06-customer-reminders.md` — detailed grooming reminder model: timing triggers, consent/channel inputs, reminder kinds, draft/audit state, failed-delivery handling, and review gates.
- `docs/domain/petsuites/training/implications/05-parent-follow-up.md` — training parent follow-up model: trainer evidence, homework/outcome wording, re-enrollment/package ambiguity, and manager/trainer/customer-message gates.
- `docs/domain/petsuites/daycare/implications/04-incident-tracking.md` — incident owner-notice constraints, safety/medical/behavior escalation, approved-customer-notice properties, and no autonomous incident messaging.
- `domain/src/entities.rs` — current customer, contact-channel, portal-account, pet, reservation, service, add-on, hard-stop, audit, and actor surfaces.
- `domain/src/customer.rs` — current customer name/email/phone value objects.
- `domain/src/workflow.rs` — current `WorkflowEvent`, `WorkflowEventType`, `PolicyContext`, `AllowedAction`, `WorkflowResult`, `RecommendedAction::DraftMessage`, and draft message channel/body value objects.
- `domain/src/operations.rs` — current customer follow-up reasons, operations actions, staff-task kinds, daily-update-draft task kind, and manager-attention behavior.
- `domain/src/agents.rs` — baseline agent specs and their customer-message/manager review gates, especially `inquiry-intake`, `daily-care-update`, `incident-escalation`, `lead-conversion`, `grooming-rebooking`, and `reputation-triage`.
- `domain/src/policy.rs` — current review gates: `ManagerApproval`, `MedicalDocumentReview`, `BehaviorReview`, `CustomerMessageApproval`, and `RefundOrDepositException`.
- `integrations/gingr/src/mapping/customer.rs` — current Gingr owner-to-customer candidate mapping and fallback contact-channel selection: email if present, else SMS if mobile phone present, else portal.

Missing/caveated sources:

- No repo-local `docs/product/pet-resort-product-map.md` was found. Use existing workflow/data-model handoffs and PetSuites/NVA domain docs as provisional product context until the product map is copied into the repo.
- No final, dedicated Customer Messaging Agent definition artifact or approved message-template catalog was found in this repo before this card. Downstream cards should treat template categories below as source-derived candidates, not approved production templates.
- No dedicated consent/opt-out/quiet-hours data model was found beyond `Customer.preferred_contact`, optional email/mobile/portal account, and repeated implication-doc references to consent/contact policy. Do not infer legal contact permission from channel availability alone.
- WhatsApp is not present in the current `ContactChannel` enum or repo docs checked. Treat it as a later optional channel only after a typed consent/provider design exists.
- Public PetSuites-style details are not pilot-approved local policy. Exact prices, timing, refund/deposit language, no-show rules, ratios, reminders, and customer-facing copy remain location/policy data.

## Brand voice and customer-facing style constraints

Canonical style available from current sources is functional rather than a formal brand book:

- Tone should be warm, pet-parent-friendly, operationally clear, and truthful. `domain/src/agents.rs` describes daily-care updates as "warm customer-safe update drafts with risk flags," and PetSuites docs use customer-facing concepts like Pawgress Reports, parent follow-up, reminders, and pet-care updates.
- Use plain language tied to approved source facts: customer/pet names, service, date/time, approved policy, staff-reviewed care evidence, and next action.
- Prefer reassurance without promises: do not promise availability, booking confirmation, refund timing, policy exceptions, diagnosis, outcome guarantees, group-play eligibility, or payment state unless the corresponding trusted source and approval path exist.
- For sensitive events, preserve important facts without blame-shifting, diagnosis, legal/liability claims, or hiding concerning facts; incident/customer-notice docs require manager/staff review before customer-facing language.
- Payment/deposit copy must be minimal, factual, non-threatening, and source-backed. Avoid threats, legal claims, discounts/waivers, altered terms, or speculative reasons for payment issues.
- If a required fact is missing, stale, ambiguous, or location-specific, the draft should say so internally and route to review rather than filling the gap in customer copy.

## Communication channels

Current canonical channel set:

- Email: present in `entities::ContactChannel::Email`, `customer::Email`, and Gingr webhook `email_sent` observations. Email bodies and recipients are PII and provider content; sanitize before display or model use.
- SMS: present in `entities::ContactChannel::Sms`, `customer::Phone`, reservation source `Sms`, and staff/customer intake references. SMS sends need contact permission and approved provider/send path; channel availability is not consent.
- Portal: present in `entities::ContactChannel::Portal`, `PortalAccountRef`, Gingr Customer Portal docs, portal JavaScript events, and customer document/reservation/payment/upload capabilities. Portal browser events are observational signals, not authoritative operational state.
- Phone exists in `ContactChannel::Phone` and intake sources but is likely a staff/call-task channel, not an AI-generated outbound text channel unless a later card scopes call scripts or transcripts.
- WhatsApp is out of scope for the current model because no repo-local type/docs establish it. Add later only with explicit consent, provider, template, and audit semantics.

Channel-selection constraints:

- Use `Customer.preferred_contact` only as a preference/input, not legal authorization.
- Required channel facts for a send candidate should include channel, destination reference, consent/opt-out status, quiet-hours/location policy, prior delivery failure/suppression state, and approval/send path.
- Failed delivery should create staff/manual retry work or a reviewed replacement send; do not silently switch channels without policy and audit.

## Approval policy and human gates

Conservative MVP default from workflow/staff/payment/docs:

- AI/workflow workers may read normalized records, extract structured suggestions, summarize approved evidence, draft internal tasks, draft customer-message copy, suggest reservation status, suggest play eligibility, and flag risks.
- Customer-facing outbound sends are human-approval-gated until the product owner explicitly approves a deterministic send path for a narrow message class.
- Provider/system mutations are human-approval-gated unless a later policy explicitly approves exact action, source evidence, actor authority, and adapter path.
- Medical, vaccine, medication, allergy, behavior, incident, safety, payment, refund, waiver, cancellation, overbooking, waitlist, and policy-exception facts require staff/manager review before becoming customer-facing or provider-mutating actions.

Current named review gates:

- `CustomerMessageApproval` — default for customer/member-facing drafts.
- `ManagerApproval` — required for incidents, complaints, safety-sensitive content, capacity/ratio exceptions, public responses, and many operational exceptions.
- `MedicalDocumentReview` — required for vaccine/medical document ambiguity and care/medical source proof.
- `BehaviorReview` — required for temperament/group-play/behavior facts where eligibility or safety is affected.
- `RefundOrDepositException` — required for refunds, waivers, deposit exceptions, forfeitures, discounts/credits, payment disputes, and payment-sensitive exceptions.

Execution model:

- A draft message is not a send. `workflow::RecommendedAction::DraftMessage` and `operations::OperationsAction::DraftCustomerMessage` produce reviewable copy only.
- Approved sends should use a separate `workflow_outbox`/approved action record with immutable approved payload, approval actor, policy version, idempotency key, provider response refs, and audit events.
- Send retries may retry delivery of the exact approved payload; content/policy changes require a new draft and new approval.

## Existing/expected template and message categories

No approved template catalog was found. Source-derived candidate categories for downstream design:

- Inquiry/lead acknowledgement or follow-up: from `inquiry.received`, `lead_created`, `LeadNeedsResponse`, `inquiry-intake`, `lead-conversion`. Draft-only unless deterministic receipt-only acknowledgement is later approved.
- Missing information requests: pet profile, contact preference, reservation details, prep instructions, signatures/legal agreements, care details, and document/vaccine proof.
- Booking triage/follow-up: missing-info, vaccine-pending, special-review, waitlist, offer/confirmation-needed, denial/rejection-by-policy explanation. No booking promise without approval.
- Booking confirmation candidate: only from `booking.confirmation_needed`; human-approved before promising confirmed space, collecting/charging payment, or writing provider status.
- Daily/Pawgress update: from `daily_note.created`/`daily_update.needed`, care notes/media, boarding/daycare active-stay policy. Sensitive care/medical/behavior/incident content requires staff/manager review or suppression.
- Grooming reminders and rebooking: confirmation, prep instructions, 48-hour reminder, morning-of reminder, rebooking due, lapsed-cadence winback, missing prep info, failed delivery/reply review. No calendar booking, discount, or send without approval.
- Training parent follow-up: progress update, homework/next steps, program completion, 30-day check-in, parent-requested clarification, re-enrollment/package opportunity. Requires trainer/source evidence and review.
- Payment/deposit/balance reminders: missing deposit, unpaid/partial/failed payment, cancellation/refundability explanation, payment reconciliation packet. Draft-only unless pre-approved deterministic template/facts/condition exist.
- Checkout/final report/follow-up: receipts/final report/Pawgress/rebooking/review-eligibility tasks; payment, incident, belongings, care-task, or unresolved service facts can block or escalate.
- Incident/customer notice: owner notice, manager packet, urgent contact candidate, restriction/safety follow-up. Always review-gated; no diagnosis/liability/unsupported outcome claims.
- Reputation/review request or public-response packet: only after eligibility/suppression checks; suppress or manager-review on incidents, complaints, payment disputes, unresolved issues, or negative sentiment.
- Upsell/revenue opportunity drafts: exit bath, grooming rebooking, daycare package, training consult, holiday boarding waitlist fill. Must remain staff-reviewed recommendations and avoid pressure or unsupported eligibility/pricing.

## Data model facts available to message generation

Currently available semantic facts include:

- Identity and scope: `LocationId`, `CustomerId`, `PetId`, `ReservationId`, `StaffId`, `ManagerId`, `ActorRef`, `Location`, `Brand`, timezone, location policy refs.
- Customer/contact: full name, optional email, optional mobile phone, preferred contact channel, optional portal account/provider external customer id.
- Pet/profile: name, species, birth date, sex, spay/neuter status, temperament profile, behavior observations, staff notes, care profile, feeding instructions, medications, allergies, medical conditions, emergency/vet contacts.
- Reservation/service: service kind (`Boarding`, `DayPlay`, `DayBoarding`, `Grooming`, `Training`, `DaySpa`), status, start/end, source (`Portal`, `WebsiteForm`, `PhoneTranscript`, `Sms`, `Email`, `StaffCreated`), deposit, add-ons, hard stops.
- Operations: operating day, occupancy/labor/arrival-departure snapshots, customer follow-ups, pet care watchlist, revenue opportunities, operations risks/actions, staff tasks and assignments.
- Workflow: event id/type/time/actor/location/subject/policy context, allowed actions, automation level, review gates, workflow result status, summary, structured output, recommended actions, risk flags, verification notes, human-review reason.
- Queue/outbox: queue row, workflow kind, retry/dead-letter state, required review gates, approval record id, approved action id, payload/result refs, idempotency scope/key, audit events, provider response refs.
- Provider/Gingr boundary: verified webhook envelope metadata, event/entity types, email_data/recipients after verification, read-only API candidates, portal events as observational signals. Raw provider JSON remains boundary evidence until mapped.

Important gaps to keep explicit:

- Consent/opt-out/quiet-hours and delivery preference are referenced as required policy facts but are not fully modeled in current Rust types.
- Message body/template/draft persistence is currently represented minimally by workflow message channel/body and queue/outbox refs; a dedicated message aggregate may be needed downstream.
- Exact approved templates, localization, legal disclaimers, and per-location brand copy are not present.

## Workflow event triggers relevant to messaging

Messaging-relevant event types from the architecture and current Rust enum:

- `inquiry.received` / `InquiryReceived`: draft acknowledgement/follow-up; no booking promise.
- `pet_profile.created` / `PetProfileCreated` and owner/customer changes: possible missing-profile/document follow-up drafts; medical/behavior facts require review.
- `document.uploaded` / `VaccineDocumentUploaded`: document receipt or missing-info drafts only after review; vaccine truth is not automatic.
- `booking.requested` / `BookingRequested`: booking triage drafts, follow-up tasks, missing-info/customer reply drafts; no acceptance/rejection/confirmation without review.
- `booking.triage_needed` / `BookingTriageNeeded`: follow-up, waitlist, offer, denial, payment/document/capacity tasks as recommendations.
- `booking.confirmation_needed` / `BookingConfirmationNeeded`: draft confirmation packet; human approval before customer promise/provider write/payment action.
- `daily_note.created` / `DailyNoteCreated` and `daily_update.needed` / `DailyUpdateNeeded`: draft daily/Pawgress/customer updates from approved evidence.
- `incident.created` / `IncidentCreated`: manager/owner draft packets; all incident-related customer messages approval-gated.
- `checkout.completed` / `CheckoutCompleted`: final report, follow-up, receipt/rebooking/review-request candidates; unresolved payment/care/incident facts gate output.
- `review_request.eligible` / `ReviewRequestEligible`: review-request draft or suppression reason; eligibility must check incidents, sentiment, payment disputes, unresolved follow-up, and contact history.
- Additional Rust enum values: `CustomerRegistered`, `MembershipChanged`, `LoyaltyCreditAvailable` exist, but detailed messaging policy/templates were not found in docs checked; downstream cards should not invent send behavior for them.

Provider/browser triggers:

- Gingr webhooks: `check_in`, `check_out`, `checking_in`, `checking_out`, `email_sent`, `owner_created`, `owner_edited`, `animal_created`, `animal_edited`, `incident_created`, `incident_edited`, `lead_created`. They are boundary triggers only after HMAC verification and semantic mapping.
- Customer Portal JS events: `reservation_created`, `owner_created`, `lead_created`. They are frontend observational/analytics signals and must not be treated as authoritative operational state.

## AI/Hermes/runtime constraints for generated customer messages

- Generated messages must be source-grounded: include only facts from normalized domain records, approved evidence, policy snapshots, trusted payment/provider refs, staff/manager approvals, or explicitly reviewed source text.
- Raw provider payloads, raw OCR, raw email bodies, unredacted customer free text, card/payment secrets, webhook signatures, API keys, and high-PII payloads must stay in boundary storage and be referenced by evidence ids/refs after redaction.
- Prompt packets should include event, subject, location/policy scope, allowed actions, required review gates, source evidence refs, suppression reasons, and output schema name. They should not include unnecessary raw PII.
- AI output should be a structured `WorkflowResult` with status, summary, structured output, recommended actions, risk flags, verification notes, and optional human-review reason; drafts are one recommended action, not execution.
- Use deterministic validators/policies for authority. Model confidence, urgency, sentiment, or customer pressure is never authority to send, approve, charge, refund, confirm, cancel, waive, or mutate provider state.
- Queue/retry semantics must preserve policy gates. Infrastructure retries must not weaken review requirements or regenerate approved payloads.
- Idempotency should use semantic keys per draft/send/template category and event/source evidence, not raw message text.
- Dead-letter and error messages shown to staff/manager/engineering must be safe and redacted; do not expose exact approved outbound body in error fields.

## Downstream definition implications

The Customer Messaging Agent should be defined as a draft/review packet generator first:

1. Inputs: semantic event + normalized customer/pet/reservation/service/policy/evidence refs + channel/preference/consent facts + prior message/delivery state.
2. Outputs: one or more typed message drafts, internal tasks, suppression/no-send reasons, review requests, and audit/idempotency metadata.
3. Non-goals by default: no autonomous sends, no provider writes, no portal script changes, no payment action, no booking promise, no medical/behavior/safety determination, and no policy exception.
4. Narrow deterministic sends can be introduced only by later policy for specific categories with fixed template, recipient/channel, facts, consent, condition, idempotency, audit, and opt-out/suppression behavior.
