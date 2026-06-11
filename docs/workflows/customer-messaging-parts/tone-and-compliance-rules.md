# Tone and compliance rules

Purpose: define the tone, safety boundaries, review criteria, and example phrasing for Customer Messaging Agent drafts. This is a drafting/review artifact only. It does not authorize autonomous customer sends, provider writes, payment actions, reservation changes, medical/vaccine decisions, incident closure, or policy exceptions.

Status: draft rules derived from `docs/workflows/customer-messaging-parts/inputs.md`, `docs/workflows/staff-operations-parts/manager-review-queue.md`, and `docs/architecture/agent-permissions-by-workflow.md`.

## Tone and compliance rules

### 1. Ground every customer-visible statement in approved facts

Customer messages must use only facts available in the current, trusted message packet:

- confirmed domain facts: customer, pet, reservation, service, location, date/time, channel, policy snapshot, verified status, approved staff note, approved care evidence, or approved manager decision;
- requested information: missing fields or documents the business is asking the customer to provide;
- manager-approved sensitive language: final wording for incidents, complaints, payment/refund/waiver topics, eligibility/refusal, medical/vaccine/document decisions, behavior/playgroup restrictions, legal/privacy concerns, or policy exceptions.

Do not fill gaps from general pet-care knowledge, public brand assumptions, previous drafts, model confidence, likely business practice, or customer pressure. If a fact is missing, stale, conflicting, ambiguous, or source-unverified, the draft must either omit it from customer copy or route it as an internal review/missing-info item.

Allowed:

- "We received Luna's boarding request for July 12 and are reviewing the details."
- "Could you please send a clearer copy of Luna's rabies record so our team can finish reviewing her file?"
- "Our manager has reviewed the update below and approved this wording for today."

Disallowed:

- "Luna is confirmed for July 12" when the booking is only requested or `confirmation_needed`.
- "Your deposit failed because your card was declined" without a trusted payment-provider status and approved payment wording.
- "Luna is all set on vaccines" when the upload is unreviewed OCR or a pending document.

### 2. Never provide diagnosis, veterinary advice, or medical conclusions

Messages may summarize approved observations and request approved documents, but must not diagnose, predict recovery, recommend treatment, interpret vaccine sufficiency, or replace staff/vet review.

Allowed:

- "Our team noticed Max did not finish breakfast this morning, so we are keeping an eye on him and will update you if we need anything else."
- "Please upload an updated vaccine record so our team can review it before check-in."
- "For health-specific questions, please contact your veterinarian."

Disallowed:

- "Max probably has an upset stomach."
- "This vaccine record is valid for daycare" unless an authorized reviewer has approved that exact eligibility status.
- "Give Bella half her usual dose tonight" or any medication instruction not explicitly approved for customer-facing communication.

### 3. Do not guarantee outcomes, availability, refunds, timing, or policy exceptions

Use cautious, operational language unless the source packet contains explicit approval for the promise. Avoid promises about space, group play, grooming finish time, training outcomes, refund amount/timing, special handling, staff availability, waitlist movement, incident resolution, or review/public-response outcome.

Allowed:

- "We will review availability and follow up with next steps."
- "Your request is on our waitlist, and our team will contact you if a spot opens."
- "A manager is reviewing the refund request and will follow up once the review is complete."

Disallowed:

- "We guarantee a spot will open."
- "Your refund will arrive tomorrow" without approved payment/refund authority and provider evidence.
- "Charlie will be able to join group play" before the required behavior/eligibility review.

### 4. Avoid blame, fault admission, legal conclusions, or staff-shaming language

Customer-safe messages should be empathetic and factual without assigning blame to staff, customers, pets, vendors, or systems. Incident, complaint, legal/privacy, and payment-dispute language requires manager approval before customer-facing use.

Allowed:

- "We are sorry for the concern this caused. Our manager is reviewing the details and will follow up with you."
- "During playtime, staff observed an interaction involving Milo and another dog. Milo is safe, and a manager will review the details before we share next steps."
- "We found a mismatch in the reservation details and are checking it now."

Disallowed:

- "Our staff forgot to feed Milo."
- "Your dog caused the incident."
- "The system messed up your payment."
- "We are legally responsible" or "we are not liable" unless legal/manager-approved language is provided.

### 5. Keep phrasing concise, warm, and channel-appropriate

Use a calm, helpful, pet-parent-friendly voice. The message should be easy for a customer to act on and should not expose internal workflow mechanics.

Channel guidance:

- SMS: short, direct, one main ask, no long explanations, no sensitive detail unless approved for SMS.
- Email: structured paragraphs or bullets when the customer needs details, documents, or prep steps.
- Portal/app message: concise status/update language tied to the active reservation or pet.
- Phone/call-script drafts: staff-facing talking points only unless a later workflow authorizes outbound call scripts.

Allowed SMS style:

- "Hi Jordan, could you upload Bella's updated rabies record before tomorrow's check-in? Our team needs it to finish review. Thank you!"

Allowed email style:

- "Hi Jordan,\n\nWe received Bella's boarding request for July 12-15. To continue review, please send her updated rabies record and confirm her feeding instructions.\n\nOnce our team reviews those details, we can follow up with next steps.\n\nThank you!"

Disallowed:

- Long SMS messages with multiple policy caveats, internal queue terms, or sensitive incident detail.
- Overly casual wording for safety/payment/complaint topics, such as "No worries, we'll definitely make it right!"
- Robotic or threatening phrasing, such as "Failure to comply will result in cancellation" unless an approved policy template requires exact wording.

### 6. Separate confirmed facts, requested information, and manager-approved sensitive language

Every draft/review packet should preserve these three categories separately so reviewers can see what the model knows versus what it is asking versus what has special approval.

Required draft packet fields:

- `confirmed_facts`: customer-safe facts with source refs, policy refs, reviewer refs, or trusted domain IDs;
- `requested_information`: customer asks that do not imply unverified conclusions;
- `sensitive_language`: any incident, medical, behavior, payment, refund, refusal, complaint, eligibility, legal/privacy, or policy-exception wording, with approval actor/ref or `requires_manager_approval`;
- `omitted_or_suppressed_facts`: facts intentionally withheld from customer copy because they are unverified, unnecessary, too sensitive, or not approved;
- `review_gates`: required gates such as `CustomerMessageApproval`, `ManagerApproval`, `MedicalDocumentReview`, `BehaviorReview`, or `RefundOrDepositException`;
- `channel_fit`: why the copy is suitable for the chosen channel or why a different channel/manual call is recommended.

Customer copy should not mention source refs, queue state, model uncertainty, raw provider payloads, staff debate, or rejected drafts. Those belong in internal review notes.

### 7. Escalate sensitive or ambiguous content instead of smoothing it over

A warm tone must not hide concerning facts or convert uncertainty into reassurance. Escalate or suppress customer copy when required evidence or approval is absent.

Always route to review before customer-facing use when the draft involves:

- injury, illness, medication, allergy, vaccine/document eligibility, behavior, group play, safety, escape/lost pet, bite/aggression, or incident details;
- complaints, bad reviews, public responses, legal/privacy language, staff conduct concerns, or already-sent incorrect content;
- payment failures, deposits, refunds, credits, waivers, forfeitures, discounts, disputes, or policy exceptions;
- booking acceptance/rejection, waitlist release, cancellation, capacity/overbooking, eligibility/refusal, or special-care promises;
- missing, stale, conflicting, or provider-unverified facts.

Allowed internal note:

- "Suppressed from customer copy: staff note says 'possible limp' but no manager/medical review has approved customer-facing wording. Route to `ManagerApproval` before send."

Disallowed customer copy:

- "Daisy may be limping but should be fine" without approved observation wording and escalation state.

### 8. Respect privacy and data minimization

Customer messages should include only what the recipient needs to understand the status or next action. Do not include raw internal notes, staff names unless approved, other pets/customers, internal room/camera/location details, payment provider references, webhook/OCR text, unredacted documents, legal/compliance notes, or unrelated history.

Allowed:

- "Please send a clearer copy of Bella's vaccine record."

Disallowed:

- "Our OCR could not read page 2 of IMG_3391 and the webhook payload says..."
- "Another dog in playgroup bit Max" unless incident wording has manager approval and privacy review.

## Worker pre-send checklist

A worker must answer "yes" to each applicable check before placing copy into a customer-message approval queue, and must not send autonomously unless a separate deterministic send policy explicitly covers the exact message class.

1. Source grounding: Does every factual claim in the customer copy map to a trusted source ref, approved evidence ref, policy snapshot, or manager-approved text?
2. No invented facts: Are all missing, ambiguous, stale, conflicting, likely, or assumed details omitted from customer copy and captured as internal notes or missing-info asks?
3. Authority: Is the intended action within the agent's allowed actions (`DraftMessage`, review packet, suppression reason, or internal task) rather than a send, provider write, payment action, reservation change, eligibility decision, or policy exception?
4. Review gates: Are all required gates included for customer-facing, sensitive, or ambiguous content?
5. Medical/safety boundary: Does the copy avoid diagnosis, veterinary advice, medication instructions, vaccine eligibility conclusions, and unsupported safety conclusions?
6. Promise boundary: Does the copy avoid guarantees about booking, availability, refunds, payment timing, service outcomes, group play, special handling, response time, or policy exceptions unless explicitly approved?
7. Blame/legal boundary: Does the copy avoid fault admission, liability/legal conclusions, staff/customer/pet blame, threats, or shaming language?
8. Tone/channel fit: Is the draft warm, concise, direct, and appropriately short or structured for SMS, email, portal, or staff call-script use?
9. Privacy: Does the draft avoid unnecessary PII, raw provider/payment/document/OCR data, internal notes, staff debate, other-customer/pet facts, and sensitive details not needed by the recipient?
10. Auditability: Does the packet carry draft ID/category, recipient/channel, source refs, review gates, suppression reasons, idempotency basis, and reviewer-needed decision?

If any answer is "no" or "unknown", output a review/missing-info/suppression reason instead of customer-ready copy.

## Reviewer approval checklist

A reviewer should approve customer-facing copy only when:

1. The final text matches approved source facts and does not add facts introduced by the model.
2. Sensitive wording has the right approval owner: manager/admin for incident, complaint, policy exception, payment/refund/waiver/refusal/legal/privacy; medical/document reviewer for document/vaccine eligibility; behavior reviewer for group-play/temperament restrictions.
3. The requested send channel is permitted by contact preference, consent/opt-out, suppression state, quiet-hours/location policy, and provider/send-path policy.
4. The customer-visible message is complete enough to be useful but does not expose unnecessary internal or sensitive data.
5. Any promised next step, deadline, refund, booking status, service availability, care accommodation, or special handling is explicitly authorized.
6. The approval record preserves reviewer, timestamp, final text/version, source refs, edits, reason, policy version, idempotency key, and send/suppression disposition.
7. Retries will resend only the exact approved payload; any content change requires a new draft and approval.

## Common rewrite patterns

| Unsafe phrasing | Safer draft pattern |
| --- | --- |
| "Your booking is confirmed." | "We received your booking request and are reviewing the details." |
| "Your dog failed the behavior test." | "Our team needs a manager review before confirming the best care plan for [pet]." |
| "The vaccine is expired." | "Our team needs an updated vaccine record before we can finish review." |
| "We will refund you tomorrow." | "A manager is reviewing the refund request and will follow up once the review is complete." |
| "The groomer is running late." | "We need a little more time to finish [pet]'s appointment today. We'll follow up with an updated pickup window." |
| "Your card was declined." | "We need help resolving the payment on file. Please contact us or update your payment method through the approved link." |
| "Max is sick." | "Our team noticed a health-related concern for Max and is escalating it for review. We will contact you with approved next steps." |
| "The staff made a mistake." | "We are reviewing what happened and will follow up with you after a manager has reviewed the details." |

## Output requirements for Customer Messaging Agent definitions

Customer Messaging Agent outputs that include customer copy should contain:

- final/customer draft text by channel;
- internal reviewer notes that identify confirmed facts, requested information, sensitive language, omitted/suppressed facts, and open questions;
- required review gates and automation level (`QueueForReview`, `DraftOnly`, `NeverAutoSend`, or, later, `DeterministicAutoSendOnly` for approved fixed templates);
- risk flags for invented fact risk, medical/safety, payment/refund, behavior, incident, complaint/legal/privacy, booking/eligibility promise, tone, and privacy;
- deterministic validation result: pass, fail, or needs human review, with reasons;
- audit/idempotency metadata for draft creation and eventual approved send path.

Default stance: low-risk routine operational messages may be queued for review; sensitive or ambiguous messages remain draft-only or never-auto-send. No AI-authored message is customer-visible until the appropriate approved policy and review/send path says it is.
