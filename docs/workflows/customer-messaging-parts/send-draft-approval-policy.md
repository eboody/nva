# Send/draft/approval policy

Purpose: define the conservative customer-message send, draft, and approval policy for the Customer Messaging Agent. This policy narrows when automation may prepare or send customer-facing messages; it does not authorize provider writes, reservation status changes, payment actions, refunds, waivers, legal conclusions, medical judgments, or policy exceptions.

Status: draft policy artifact for downstream Customer Messaging Agent design. It is source-backed by `docs/workflows/customer-messaging-parts/inputs.md`, staff review queue policy, payment AI boundaries, and current workflow/domain constraints.

## Core stance

The Customer Messaging Agent is draft-first. It may auto-send only narrow routine message categories after a human has approved the category set, template, channel policy, evidence requirements, suppression rules, and send adapter. Everything else becomes a draft, review packet, staff task, suppression/no-send reason, or manager approval request.

A customer-facing send is allowed only when all of these are true:

1. The message category is explicitly in the approved auto-send category set for the location and channel.
2. The final customer copy is an approved deterministic template or template-bound variant for that category.
3. Required facts are present, current, source-backed, and non-conflicting.
4. Recipient, channel, consent/opt-out status, quiet-hours policy, and prior delivery suppression state are satisfied.
5. The message contains no sensitive, exceptional, ambiguous, or manager-gated language.
6. Idempotency, audit, outbox, retry, and provider-response records will preserve the exact approved payload and policy version.

If any condition is missing, stale, ambiguous, conflicting, or outside scope, the agent must not send. It should draft or suppress and route to the appropriate human review path.

## Human approval gates

### Gate A: approved auto-send category set

Product/operations leadership must approve the exact categories that may auto-send before any automated customer-facing delivery is enabled. The initial approved set should be limited to routine reminders, pre-arrival preparation messages, and review requests that satisfy the rules in this file.

Approval must specify:

- category name and allowed workflow event triggers;
- template id/version and allowed variables;
- allowed channels and consent/quiet-hours rules;
- required source facts and evidence references;
- suppression conditions;
- idempotency key scope and retry behavior;
- audit/outbox fields;
- role that owns template and policy changes.

Until this gate is approved, all customer-facing messages are drafts or staff tasks, not sends.

### Gate B: legal/medical-sensitive language policy

Management, and legal/compliance where applicable, must approve the policy for language that touches medical, medication, vaccine/document eligibility, injury, safety, behavior/aggression, incidents, privacy, liability, refunds/waivers, rejections/declines, or policy exceptions.

Default until approved: the agent may not auto-send any legal/medical-sensitive language. It may only prepare internal summaries and manager-reviewed drafts with evidence refs, risk flags, and suggested escalation path.

## Policy rules

| Rule | Send mode | Rationale | Main risk | Escalation path | Required evidence/facts |
| --- | --- | --- | --- | --- | --- |
| Routine reminder with approved template | Auto-send only after Gate A approval | Routine reminders can reduce staff workload when facts and copy are deterministic. | Sending at wrong time, to wrong recipient/channel, after opt-out, or with stale reservation facts. | Suppress and create front-desk/staff review task when facts, consent, quiet-hours, template, channel, or delivery state are not clean. Manager review if reminder touches exception, payment dispute, incident, complaint, medical/behavior issue, or rejection. | Approved category/template/policy version; workflow event trigger; customer/pet/reservation/location ids; service/date/time/timezone; channel and destination ref; consent/opt-out/quiet-hours state; prior send/delivery state; no unresolved risk flags. |
| Pre-arrival preparation message with approved template | Auto-send only after Gate A approval | Pre-arrival prep can safely communicate ordinary checklist or arrival instructions when local policy and reservation facts are fixed. | Implied booking confirmation, special-care promise, incorrect document/vaccine requirement, or unapproved policy statement. | Draft-only for missing documents, special care, vaccine uncertainty, care nuance, capacity/waitlist, payment ambiguity, or any booking-status promise. Manager/document/special-care review when eligibility, acceptance, decline, or care accommodation is affected. | Approved template and location policy snapshot; reservation status that permits pre-arrival contact; service/start time; ordinary prep facts; no conflicting/stale policy; channel consent; suppression checks; explicit exclusion of medical/legal/exception language. |
| Review request after checkout/completion | Auto-send only after Gate A approval and eligibility checks | Review requests are low-risk only when the stay closed cleanly and no unresolved issue should make the request insensitive. | Asking for a review after an incident, complaint, billing dispute, unresolved care issue, negative sentiment, or failed prior contact. | Suppress automatically on incidents, complaints, negative sentiment, payment dispute, unresolved follow-up, failed service recovery, or manager hold. Route to manager/reputation review if the customer history is mixed or ambiguous. | Checkout/completion event; customer/contact consent; eligible channel; no open incident/complaint/payment dispute/care follow-up; sentiment or service-recovery suppression state if available; prior review-request/contact history; template/version/audit refs. |
| Receipt-only acknowledgement for inquiries or uploads | Draft-only unless separately added to Gate A | Simple receipt acknowledgements may become safe later, but current sources do not approve a deterministic template catalog or consent model. | Customer may infer acceptance, booking confirmation, vaccine approval, document approval, or response SLA. | Staff/front-desk review for customer-facing copy; document/medical review if upload contains vaccine/medical facts; manager review for legal agreements, complaints, or sensitive content. | Source event id; customer/contact refs; what was received; approved non-committal wording; channel consent; no approval/acceptance claim; evidence ref for received item only, not interpreted truth. |
| Daily/Pawgress or routine care update | Draft-only by default | Care updates often contain free-text staff observations, media, behavior, medication, health, or incident-adjacent facts that need source review. | Unsupported reassurance, medical claim, missed concerning fact, privacy/media issue, or inaccurate behavior/care statement. | Lead/staff review for routine factual care content; manager review for injury/illness, medication, behavior/aggression, incident, complaint, sensitive media, or unresolved care issue. | Staff-reviewed care notes/media refs; pet/stay/date; approved facts only; sensitivity tags; media/customer-use permission if applicable; unresolved incident/medical/behavior/payment flags; reviewer role/decision for final send. |
| Grooming reminder or ordinary rebooking nudge | Draft-only unless category/template is approved under Gate A | Grooming reminders can be routine, but rebooking and prep messages may imply availability, pricing, service suitability, or policy. | Unsupported appointment availability, incorrect prep instructions, unapproved price/discount, or message after failed delivery/opt-out. | Staff review for routine grooming follow-up; manager review for cancellations, complaints, pricing/payment exception, safety/behavior issue, or service decline. | Grooming appointment/service facts; timing rule; approved prep/rebooking template; channel consent; prior delivery/reply state; no discount/price/exception language; no unresolved complaint or incident. |
| Training parent follow-up or homework | Draft-only | Training updates depend on trainer observations, progress language, homework, and package/re-enrollment context. | Overstating progress, promising outcomes, pressuring package sale, or misstating behavior/safety. | Trainer/staff review for ordinary progress/homework; manager review for behavior/aggression, safety restrictions, complaint, refund/package exception, or decline language. | Trainer-approved observations; program/session refs; homework/next-step facts; sensitivity tags; no outcome guarantee; channel consent; manager decision for sensitive or commercial exception content. |
| Missing-info request for routine administrative facts | Draft-only unless category/template is approved under Gate A | Requesting missing profile/contact/prep information can be routine, but missing facts often affect eligibility or booking commitments. | Asking for wrong information, exposing internal uncertainty, implying acceptance, or mishandling document/medical data. | Front-desk/staff review for ordinary missing info; document/medical review for vaccine/medical proof; manager review when missing info drives decline, waitlist, policy exception, or special care. | Specific missing fields; source state showing missing/stale/ambiguous; approved request wording; service/reservation context; channel consent; no eligibility/acceptance claim unless approved. |
| Booking confirmation, acceptance, waitlist, or availability language | Manager approval required unless a future separate policy explicitly allows a narrow staff-reviewed path | These messages change customer expectations and can imply capacity, eligibility, staffing, and payment/policy authority. | Overbooking, false promise, unapproved decline, missed vaccine/special-care/payment gate, or provider-state mismatch. | Manager/admin review; document/vaccine, behavior, special-care, capacity, or payment review as applicable. Provider mutation and customer send must be separate audited actions. | Reservation status; capacity/staffing/room facts; eligibility/document status; payment/deposit state; special-care/behavior flags; approved policy snapshot; manager decision; provider/write/send refs if executed. |
| Exceptions, special-care nuance, or ambiguous facts | Draft-only; manager approval if customer-facing meaning is material | Exceptions and ambiguity require judgment and often depend on location policy, care capability, or risk tolerance. | AI fills gaps, promises accommodation, gives unsafe care instruction, or hides uncertainty. | Route to manager/special-care/document/behavior/payment review depending on issue. Staff may collect missing evidence but cannot approve exceptions without authority. | Missing/ambiguous/conflicting fact list; source refs and freshness; affected pet/reservation/service; requested decision; risk flags; policy snapshot or unresolved-policy marker; proposed customer-safe draft if any. |
| Incidents, injuries, illness, medication errors, safety events, escape/lost pet, or facility hazards | Manager approval required | Incident-related messaging requires factual accuracy, safety escalation, tone control, and possible medical/legal/privacy sensitivity. | Unsafe advice, diagnosis, liability/fault admission, omitted concerning fact, privacy breach, or customer misinformation. | Immediate manager/lead escalation for active safety; vet/emergency, legal/compliance/privacy, behavior, or payment/refund review as indicated. Customer send only after approved incident disposition. | Incident record; time/location; observed facts; immediate care actions; safety state; witnesses/source media refs; customer-contact state; medical/vet/behavior/legal/payment gates; manager-approved final copy. |
| Aggression, bite, temperament, group-play restriction, reinstatement, or behavior concern | Manager/behavior approval required | Behavior language can affect safety, eligibility, future service access, and customer trust. | Misclassification, unsupported blame, unsafe group-play promise, or defamatory/insensitive wording. | Manager plus behavior/lead staff review; incident workflow if tied to an event; special-care review for future booking constraints. | Behavior evidence; incident/staff notes; current eligibility/care-lane/group-play state; prior behavior history; policy basis; manager/behavior decision; approved customer wording. |
| Refunds, waivers, credits, discounts, forfeitures, billing disputes, or payment-sensitive exceptions | Manager/payment approval required | Financial and policy-exception language requires trusted payment truth and authorized decision makers. | Unauthorized refund/waiver promise, incorrect amount, threats/legal claims, exposing payment data, or treating unverified customer claims as truth. | Staff/payment reconciliation for factual status; manager for refunds, disputes, waivers, discounts, forfeitures, complaints, amount conflicts, or exception wording; legal/compliance for chargeback/legal/privacy signals. | Trusted payment/policy refs; amount/currency/due/refundability only when verified; customer request if any; reconciliation state; manager/payment decision; redacted evidence; approved final wording. |
| Rejection, decline, denial, refusal, cancellation, restriction, or policy-enforcement language | Manager approval required | Negative or restrictive language has high customer-impact and may involve eligibility, capacity, safety, legal, or payment policy. | Unfair/unsupported denial, discriminatory or insensitive tone, legal exposure, or incorrect policy application. | Manager/admin review; document/medical/behavior/capacity/payment/legal review as applicable. Staff may collect facts but should not send unapproved decline language. | Decision requested and basis; source facts; policy snapshot; eligibility/capacity/payment/care gates; alternative options if approved; manager decision and approved customer-safe copy. |
| Medical, vaccine, medication, allergy, diagnosis, treatment, legal, liability, privacy, or compliance language | Manager approval required; specialist/legal/compliance review when applicable | These categories are sensitive and cannot be safely resolved by model confidence or raw documents. | Medical advice, wrong vaccine/document status, privacy breach, legal admission, regulatory issue, or unsafe care instruction. | MedicalDocumentReview, manager, vet/emergency, legal/compliance/privacy, or document reviewer depending on subject. No auto-send until Gate B and category-specific policy explicitly allow the exact wording class. | Verified document/care evidence; reviewer decision; policy/version; sensitivity tags; minimum necessary customer details; explicit no-diagnosis/no-legal-conclusion copy; approval actor/time/reason. |
| Complaints, negative sentiment, public-review response, or service recovery | Manager approval required except future approved non-committal receipt acknowledgement | Complaint responses may involve incidents, refunds, service recovery, staff conduct, public reputation, and legal/privacy risk. | Admitting fault, promising refund/credit, escalating publicly with sensitive facts, or responding from incomplete evidence. | Manager owns disposition; payment, incident, legal/privacy, staff/HR, or engineering/provider support as needed. Public responses need extra manager/legal/privacy caution. | Complaint source/ref; category; related records; desired outcome; evidence collected; service recovery/payment/legal gates; manager-approved response or suppression reason. |
| Failed delivery, customer reply, opt-out, or channel mismatch | Do not switch/send automatically unless approved channel policy allows it | Channel behavior can create consent, privacy, and duplicate-message problems. | Contacting through unapproved channel, ignoring opt-out, duplicate sends, or exposing sensitive content. | Staff/front-desk review for routine contact correction; manager/legal/privacy review for sensitive content, opt-out disputes, or repeated delivery failure. | Delivery/provider response refs; channel consent/opt-out state; destination refs; prior attempts; suppression state; approved alternative-channel policy; audit/idempotency refs. |

## Escalation routing

- Front desk / staff: routine factual review, missing ordinary administrative info, routine reminder/pre-arrival draft checks, contact correction, and source-fact collection.
- Lead/care staff: routine care-note validation, daily update evidence, trainer/groomer observations, care-plan source facts, and ordinary parent follow-up drafts.
- Manager/admin: incidents, aggression/behavior restrictions, refunds/waivers/credits/discounts/forfeitures, declines/rejections, complaints, policy exceptions, capacity/waitlist/booking promises, sensitive tone, and final customer-message approval where not pre-approved for auto-send.
- Document/medical reviewer: vaccine/document proof, medication/allergy/medical ambiguities, care-instruction uncertainty, and any message that states document or medical eligibility status.
- Payment reconciliation/provider operator: failed/unknown/partial/duplicate payment facts, provider-reference mismatches, webhook/payment conflicts, and amount/currency conflicts.
- Legal/compliance/privacy: legal threats, regulatory issues, liability/fault language, privacy/security exposure, raw PII/payment leak concerns, public response with sensitive facts, and any category covered by Gate B where local policy requires specialist review.
- Engineering/integration owner: missing approved send adapter, webhook verification failure, stale/missing policy snapshot, validator defect, template/version mismatch, idempotency conflict, or provider delivery inconsistency.

## Required send/audit contract

Every send candidate, draft, suppression, and review request should carry structured fields instead of relying on prose:

- `message_category` and `policy_version`.
- `send_mode`: `auto_send_candidate`, `draft_only`, `manager_approval_required`, or `suppressed`.
- `workflow_event_id`, `subject_refs`, and source/evidence refs.
- `recipient_ref`, `channel`, `destination_ref`, consent/opt-out/quiet-hours state, and delivery suppression state.
- `template_id`/`template_version` or draft body ref.
- required facts checklist with `verified`, `missing`, `ambiguous`, `conflicting`, or `stale` state.
- sensitivity tags and required review gates.
- reviewer/approval refs when not auto-send.
- idempotency key, outbox ref, provider response refs, retry/dead-letter state, and immutable approved payload ref for executed sends.
- suppression/no-send reason when output is withheld.

Infrastructure retries may retry delivery of the exact approved payload only. They must not regenerate copy, loosen review gates, switch channels, or update facts without a new policy decision and audit record.

## Implementation implications

- The default result for generated customer copy is a draft or review packet, not a send.
- The first production milestone should implement deterministic validators for the auto-send category set and legal/medical-sensitive language policy before enabling any send adapter.
- Auto-send should be opt-in by category, location, channel, template version, and policy version; it should not be inferred from `AllowedAction` or model confidence alone.
- Source freshness and conflict handling are blocking gates. The agent should prefer `needs_human_review` or `suppressed` over customer-facing copy when evidence is incomplete.
- Payment, provider, reservation, document, eligibility, and incident execution remain separate audited workflows. Message approval is not permission to mutate those systems.
- Future expansion of auto-send categories should require a new policy/version update, test fixtures for happy path and suppression cases, and manager/product approval recorded outside model output.
