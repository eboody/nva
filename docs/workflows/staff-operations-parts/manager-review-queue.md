# Manager review queues

Purpose: define the manager-review queue and escalation workflow for operational safety, quality, and customer-facing approvals. This is a modeling artifact for staff-operations design; it does not authorize live reservation updates, care decisions, payment actions, customer messages, provider writes, or autonomous task creation in production.

Source basis:

- `docs/workflows/staff-operations-parts/inputs.md` is the canonical staff-operations input packet for this file.
- Current task primitives are `operations::StaffTask`, `StaffTaskStatus::{Open, InProgress, Blocked, NeedsManagerReview, Completed, Cancelled}`, `StaffTaskPriority::{Low, Normal, High, Critical}`, `StaffTaskAssignment::{Unassigned, Staff, Role}`, and `StaffTaskSource::{Reservation, Pet, Customer, DailyBrief, WorkflowEvent, StaffCreated}`.
- Manager attention currently triggers for blocked/manager-review status, high/critical priority, and incident, medication, or document-review task kinds.
- AI/workflow workers may read, summarize, detect gaps, draft internal tasks/messages, and recommend review gates, but must not act as autonomous medical, safety, payment, eligibility, capacity-exception, or customer-message authorities.

## Core rule

A manager-review queue item is an explicit approval or escalation object, not a generic note. Every item must identify the subject, source trigger, owner role, priority/severity, due semantics, allowed next actions, audit evidence, customer-visible outcome, and closure evidence.

When source facts or policy are missing, stale, contradictory, sensitive, or provider-unverified, the workflow must create a review state or draft internal task. It must not silently mark the work ready, infer policy, send a customer message, mutate provider state, or treat an AI summary as authority.

## Shared queue item contract

Every manager-review queue entry should carry these fields regardless of category:

| Field | Requirement |
| --- | --- |
| Queue category | One of `Incident`, `RejectedAiOutput`, `SpecialCareBooking`, `DocumentUncertainty`, `ApprovalDraft`, `Complaint`. |
| Subject | Typed subject: reservation, stay, pet, customer, location, operating day, staff task, workflow event, or draft message. Avoid untraceable free-text subjects. |
| Source trigger | The event or detection path that created the review item: staff escalation, blocked task, workflow event, AI draft rejection, document review result, customer complaint, capacity/ratio exception, daily brief risk, check-in/out packet, or provider/import conflict. |
| Source evidence | Stable references to documents, task IDs, reservation/pet/customer IDs, policy snapshots, staff notes, audit events, attachments, extracted facts, and the current trusted/untrusted/conflicting source state. |
| Required fields | Category-specific facts required before a manager can decide. Missing required fields keep the item in `NeedsInfo` or `Blocked`, not `Approved`. |
| Priority/severity | Operational priority (`Low`, `Normal`, `High`, `Critical`) plus semantic severity where useful: routine, time-sensitive, sensitive, safety-critical, legal/compliance/privacy, or customer-blocking. |
| Owner role | Primary role accountable for disposition. Manager/admin owns final approval; lead/front desk/care staff may be assigned info-gathering sub-tasks. |
| SLA/due semantics | A due basis, not a magic constant: before arrival, before medication window, immediately for safety, before customer send, before checkout/release, before capacity returns to inventory, or within approved location policy. |
| Allowed actions | The manager and delegated staff actions allowed for that queue state. Actions must be typed and auditable. |
| Forbidden actions | Actions the queue item explicitly cannot perform without another approved workflow, such as payment movement, provider writes, autonomous customer sends, or medical judgment. |
| Audit/compliance | Actor, timestamp, role, decision, reason, policy/source version, before/after state, and evidence references required for every disposition. |
| Customer-visible outcome | What, if anything, may eventually be visible to the customer after approval. Drafts remain non-visible until approved and sent through an allowed path. |
| Closure evidence | The concrete evidence that lets the item close: manager decision, staff evidence, customer response, verified document, amended draft, incident follow-up, provider action reference, or explicit suppression reason. |

Recommended queue states:

- `Open`: created with enough context to route.
- `NeedsInfo`: missing required fields or supporting evidence.
- `InReview`: manager or delegated reviewer is actively deciding.
- `DelegatedForCollection`: waiting on front desk, care staff, lead, customer, vet, or provider evidence.
- `Approved`: manager approved a bounded action or draft.
- `Rejected`: manager rejected the proposed action/draft and captured reason.
- `Suppressed`: manager decided no customer-visible or provider-visible action should happen.
- `Escalated`: escalated to vet/emergency, legal/compliance/privacy, payment reconciliation, engineering/integration owner, or higher manager.
- `Closed`: closure evidence is complete and audit record is retained.
- `Cancelled`: item was superseded by a later verified state or duplicate review item.

## Shared prioritization and SLA rules

Use priority as a review-ordering signal, not as authority to skip evidence.

| Priority | Use for | Due semantics |
| --- | --- | --- |
| `Critical` | Animal safety, injury/escape/bite/aggression, medication ambiguity inside the medication window, medical distress, customer threat/legal/privacy signal, unsafe capacity/ratio exception, or checkout/release blocked by safety facts. | Immediate manager/lead escalation; customer-facing or provider mutation remains blocked until authorized. |
| `High` | Arrival/check-in blocked today, same-day daycare eligibility uncertainty, special-care booking before stay start, unresolved complaint, sensitive customer-message draft, document uncertainty blocking eligibility, or payment/policy exception with customer impact. | Before the affected operational milestone: arrival, playgroup entry, medication/feed window, customer send, checkout, or booking acceptance. |
| `Normal` | Non-urgent document ambiguity, future special-care booking, routine approval draft, non-safety rejected AI output, ordinary customer follow-up requiring manager tone review. | Before the next scheduled customer interaction or policy-defined review window. |
| `Low` | Retrospective quality review, training data feedback, non-customer-visible draft improvement, post-close audit cleanup. | Backlog/review cadence; must not block live operations unless reclassified. |

SLA/due values must be derived from source policy, location timezone, service date/time, medication/feeding window, checkout time, message-send window, or incident severity. If no approved timing policy exists, record `due_basis = unresolved_policy` and route to manager rather than inventing an SLA.

## Queue 1: incidents

### Source triggers

- `IncidentFollowUp` staff task, care watchlist item, daily brief risk, staff-created incident note, shift-handoff item, customer complaint about an incident, or provider/imported incident flag.
- Signals include injury, illness, bite/aggression, escape/lost pet, medication/feeding exception with possible harm, facility hazard, staff/customer safety issue, property damage, or a sensitive behavior event.
- AI may flag incident-like language or summarize source evidence, but it cannot classify final severity or send customer incident messages autonomously.

### Required fields

- Location, operating day, event time or discovered-at time, reporting actor, affected pet/customer/reservation/staff if known.
- Incident type, observed facts, immediate safety state, care actions already taken, unresolved risk, and whether vet/emergency escalation may be needed.
- Source evidence: staff note, photo/media reference if allowed, task/evidence IDs, witness/staff names or role IDs, provider references, and any customer statement.
- Customer-contact state: not contacted, draft pending, contacted by staff/manager, customer response pending, or suppressed with reason.
- Review gates: medical/vet, behavior/group-play, legal/compliance/privacy, payment/refund/waiver, provider mutation, or customer-message approval.

### Priority/severity

- `Critical`: active safety risk, injury/medical distress, bite/aggression, escape/lost pet, medication error, suspected neglect/abuse, or hazard requiring immediate action.
- `High`: customer-impacting incident, potential service restriction, group-play suspension/reinstatement, possible refund/waiver request, or incident message due before checkout.
- `Normal`: resolved minor incident needing manager sign-off before customer summary or internal closure.
- `Low`: retrospective quality review with no live safety/customer impact.

### Owner role

- Manager/admin owns final disposition and customer-facing approval.
- Lead staff may triage and collect immediate evidence.
- Kennel/playgroup staff provide observation and care evidence.
- Vet/emergency, legal/compliance/privacy, or payment reconciliation are external/specialist escalations when indicated.

### SLA/due semantics

- Immediate for active safety, medical, escape/lost pet, bite/aggression, facility hazard, or medication error.
- Before checkout/release if the incident must be truthfully communicated or affects release conditions.
- Before future group-play eligibility, reinstatement, or special-care booking decisions when behavior/safety is implicated.
- Before any customer-facing incident message or refund/waiver/credit discussion.

### Allowed actions

- Assign immediate staff/lead safety actions and evidence collection.
- Mark customer-message drafts as pending manager approval, approved, rejected, or suppressed.
- Suspend group play or require staff/manager review for future play eligibility when policy allows.
- Escalate to vet/emergency, legal/compliance/privacy, payment review, or engineering/provider support.
- Create follow-up tasks for care monitoring, staff debrief, customer follow-up, cleaning/hazard remediation, or provider record update.

### Forbidden actions

- AI or ordinary staff cannot close an incident using a summary alone.
- Do not send incident, medical, legal, refund, waiver, or fault-admitting messages without manager approval.
- Do not reinstate group play after a safety/behavior incident without the required review gate.
- Do not promise refunds, credits, medical outcomes, legal conclusions, or provider record changes from the review item alone.

### Audit/compliance requirements

- Retain actor, timestamp, source evidence, severity changes, triage steps, manager decision, escalation targets, customer-contact decision, and closure reason.
- Preserve source media/document references without unnecessary PII exposure.
- Record any suppression of customer-visible content and the reason.
- Record any provider/status/message/payment action as a separate audited execution event, not merely as queue closure.

### Customer-visible outcomes

- Approved incident explanation, care update, apology/service recovery message, behavior/group-play restriction note, checkout handoff, or follow-up commitment.
- Customer-visible content may also be intentionally suppressed when manager records why no outbound message is appropriate.

### Closure evidence

- Manager-approved disposition plus evidence that immediate safety actions are complete or escalated.
- Customer communication sent/suppressed with approved reason where applicable.
- Follow-up tasks created or completed for care monitoring, vet/emergency, group-play status, cleaning/hazard, provider record, payment/recovery, or staff debrief.

## Queue 2: rejected AI outputs

### Source triggers

- Staff or manager rejects an AI summary, draft task, draft customer message, document extraction, playgroup suggestion, reservation-status suggestion, daily update, or incident/complaint summary.
- Deterministic validator flags missing citation, unsupported claim, unsafe tone, stale evidence, untrusted source, privacy issue, policy overreach, hallucinated fact, or unauthorized action.
- Customer-facing send path blocks an AI-authored draft because review gates are unresolved.

### Required fields

- AI output ID or workflow event, prompt/input snapshot reference where retained, generated output, intended recipient/audience, intended action, and current workflow state.
- Rejection actor, rejection reason, unsafe/incorrect spans where possible, source facts that contradict or fail to support the output, and category of failure.
- Whether customer-facing, staff-facing, provider-facing, payment-sensitive, medical/safety-sensitive, or training-only.
- Corrective path: rewrite, suppress, escalate, request missing info, update extraction, update policy/template, or engineering bug.

### Priority/severity

- `Critical`: output could cause unsafe care, medical/medication error, unauthorized provider/payment action, privacy leak, legal/fault admission, or customer-facing misinformation if released.
- `High`: customer-facing draft or operational task blocked today by unsafe/unsupported content.
- `Normal`: internal draft is unusable but no live customer/safety milestone is blocked.
- `Low`: quality feedback for later prompt/template/model improvement.

### Owner role

- Manager/admin owns high-risk, customer-facing, safety, payment, legal/compliance/privacy, and policy-overreach rejections.
- Lead/front desk/care staff may correct factual source packets for routine operational drafts.
- Engineering/integration owner handles repeatable extraction, mapping, citation, privacy, or validator defects.

### SLA/due semantics

- Before any customer send, provider write, care task creation, playgroup suggestion acceptance, reservation-status mutation, or payment-sensitive action.
- Before affected arrival/check-in, medication/feed window, daily update send window, checkout, or complaint response.
- Backlog cadence for purely retrospective training feedback.

### Allowed actions

- Reject and suppress the output.
- Request revised draft using cited sources and narrower instructions.
- Convert the issue to a missing-info/document/incident/complaint review item if the rejection reveals an operational risk.
- Approve a corrected draft only after evidence and review gates are satisfied.
- File engineering/policy/template feedback for repeated failure modes.

### Forbidden actions

- Do not let the AI self-approve a rejected output.
- Do not treat regenerated text as safe without preserving the original rejection reason and re-reviewing the new draft.
- Do not use AI confidence as evidence.
- Do not silently delete rejections that affected safety, payment, legal/privacy, customer-facing, or provider-facing workflows.

### Audit/compliance requirements

- Retain rejected output reference, reviewer, reason, decision, replacement/suppression result, and whether the draft was customer-visible before rejection.
- Preserve enough input/output context for audit and debugging while respecting data minimization and PII/payment/medical sensitivity.
- Link any engineering, policy, prompt, or template follow-up to the rejection.

### Customer-visible outcomes

- Usually none: the rejected output remains internal.
- If the rejected output delayed a customer update, the visible result is a manager-approved replacement message, not the rejection itself.
- If incorrect content was already sent, escalate to complaint/incident/privacy workflow as appropriate.

### Closure evidence

- Output suppressed, corrected and approved, or converted to another queue item.
- Reviewer reason and final disposition recorded.
- Follow-up engineering/template/policy action created when the failure is systematic.

## Queue 3: special-care bookings

### Source triggers

- Reservation/request/stay includes medication, feeding complexity, allergies, medical condition, senior/puppy/kitten care, behavior concerns, individual play/care-lane needs, special accommodation, vet/emergency constraints, or customer-requested exception.
- Check-in prep, daily brief, care profile, document review, customer note, provider import, or staff task identifies care ambiguity or policy exception.
- Playgroup eligibility, capacity/staffing/ratio, overbooking/waitlist release, or group-play reinstatement requires manager approval.

### Required fields

- Reservation, customer, pet, service kind, dates/times, location, current reservation status, and affected operational milestone.
- Care requirements: medication name/dose/schedule/source if applicable, feeding instructions, allergies, medical conditions, handling notes, temperament/group-play status, spay/neuter/vaccine relevance, vet/emergency contact state, and required staff skill/coverage.
- Capacity/labor impact: accommodation/room/care-lane, staff ratio, schedule, special equipment, separation needs, or manager hold.
- Evidence state: verified, missing, ambiguous, conflicting, stale, customer-provided only, staff-reviewed, vet/provider-confirmed.
- Decision requested: accept/decline/request info/waitlist/assign care lane/require individual play/manager hold/override/reinstate/suspend.

### Priority/severity

- `Critical`: same-day active stay with unsafe or unclear medication/medical/safety instructions.
- `High`: arrival/check-in today or tomorrow, playgroup decision due today, capacity/ratio exception, or booking acceptance depends on manager review.
- `Normal`: future booking with complete but special care packet awaiting approval.
- `Low`: future informational review with no acceptance/capacity/customer milestone yet.

### Owner role

- Manager/admin owns booking acceptance exceptions, capacity/ratio overrides, group-play reinstatement/suspension/override, policy exceptions, and high-risk customer wording.
- Lead staff/care team may review executable care plan details and collect missing evidence.
- Front desk may collect routine missing documents or customer clarification but cannot approve medical/safety/capacity exceptions.

### SLA/due semantics

- Before accepting/confirming a booking if special care affects eligibility, capacity, staffing, or policy exception.
- Before arrival/check-in if care instructions affect readiness.
- Before medication/feeding/playgroup window during active stay.
- Before customer message promising acceptance, special handling, group play, or exception.

### Allowed actions

- Approve with explicit constraints, such as individual care lane, no group play, medication double-check, manager hold, extra staff review, or arrival instructions.
- Reject/decline or keep waitlisted when policy or safety cannot be satisfied.
- Request customer/vet/provider documentation or clarification.
- Create staff tasks for care-plan review, medication verification, feeding clarification, playgroup assessment, room/accommodation prep, or customer follow-up.
- Attach manager-approved customer-message draft or internal check-in packet.

### Forbidden actions

- AI cannot infer executable medication instructions from vague notes.
- AI cannot default unknown/stale/conflicting temperament, vaccine, spay/neuter, care, incident, staff-coverage, or capacity facts to group play.
- Do not promise acceptance, special handling, staff availability, group play, medical outcomes, or capacity exceptions without manager approval.
- Do not mutate provider reservation status or booking acceptance from the review item alone.

### Audit/compliance requirements

- Record source evidence, decision, constraints, policy basis, manager actor, due milestone, and downstream staff tasks.
- Retain care-plan changes and group-play/care-lane decisions with reason and effective dates.
- Record customer-facing promises separately from internal constraints.

### Customer-visible outcomes

- Approved acceptance/decline/request-info/waitlist message.
- Approved check-in instructions or special-care acknowledgement.
- Approved restriction language such as individual care instead of group play, if required and customer-safe.

### Closure evidence

- Manager decision plus care packet state: accepted with constraints, declined, waitlisted, missing-info requested, or escalated.
- Staff tasks created/assigned for any continuing care obligations.
- Customer communication sent/suppressed with approved reason when applicable.

## Queue 4: document uncertainty

### Source triggers

- `DocumentReview` task, uploaded file, vaccine proof extraction, OCR/import conflict, missing required document, stale document, untrusted screenshot/email, unsupported species/service requirement, or staff/customer dispute about eligibility.
- AI extraction has low confidence or conflicting structured fields.
- Check-in readiness, booking acceptance, playgroup/daycare eligibility, or stay continuation is blocked by document status.

### Required fields

- Pet, customer, reservation/stay, location, service kind, document type, upload/import source, received-at time, and current eligibility impact.
- Extracted fields and source spans: vaccine name/type, administration date, expiration/due date, provider/vet, lot/reference if present, species/pet match, and any signature/clinic proof if relevant.
- Source state: verified, missing, ambiguous, conflicting, stale, illegible, wrong pet, wrong location/service, customer-claimed only, or provider-confirmed.
- Required policy snapshot or unresolved policy marker; avoid hard-coding vaccine requirements from public context.
- Decision requested: approve, reject, request replacement, mark pending, staff verify with clinic/vet, or block service/playgroup/check-in.

### Priority/severity

- `High`: same-day arrival, daycare/playgroup eligibility, check-in blocked, or document uncertainty creates safety/policy risk.
- `Normal`: future reservation/document packet needs review before confirmation.
- `Low`: cleanup of non-blocking document metadata or retrospective extraction quality.
- `Critical`: only when document uncertainty is tied to immediate animal/public safety or active stay risk.

### Owner role

- Manager/admin or approved document/vaccine reviewer owns final eligibility-impacting approval/rejection.
- Front desk may collect replacements or clinic contact details.
- AI may extract and summarize but cannot verify final vaccine/document eligibility.

### SLA/due semantics

- Before booking confirmation or arrival/check-in when document status affects eligibility.
- Before daycare/group-play entry when vaccine/eligibility proof affects safety.
- Before customer message that states a document is accepted/rejected or service is unavailable because of it.

### Allowed actions

- Approve/reject document status within approved policy and role authority.
- Request replacement, clearer copy, vet/clinic confirmation, or customer clarification.
- Mark reservation/pet as pending document review or blocked for eligibility where policy allows.
- Create front-desk/customer-follow-up tasks.
- Attach customer-safe draft explaining missing/uncertain document needs for review.

### Forbidden actions

- AI cannot mark vaccine/document proof verified or eligibility approved from OCR alone.
- Do not infer expiration dates, vaccine sufficiency, pet identity, or clinic validity when evidence is absent or ambiguous.
- Do not send rejection/eligibility-denial messages without approved wording and review gate.
- Do not write provider eligibility/status unless an approved execution path exists.

### Audit/compliance requirements

- Retain original document reference, extraction result, reviewer decision, policy/version, source conflicts, replacement requests, and final status.
- Record whether the decision affected booking, check-in, daycare/playgroup, or checkout.
- Minimize exposed medical/document details in customer-facing summaries.

### Customer-visible outcomes

- Approved request for replacement or clarification.
- Approved acceptance/rejection/pending explanation.
- Approved service eligibility consequence only when policy and manager/staff authority support it.

### Closure evidence

- Verified document decision or explicit pending/blocked state with follow-up task.
- Replacement requested/received, clinic/vet confirmation attached, or service eligibility action recorded.
- Customer-facing message sent/suppressed with approved reason.

## Queue 5: approval drafts / customer-message drafts

### Source triggers

- AI or staff drafts a daily/Pawgress update, incident message, complaint response, document request, booking acceptance/decline, special-care instruction, checkout summary, payment/deposit reminder, policy-exception explanation, grooming/training follow-up, or customer support response.
- Draft includes sensitive content: medical, medication, allergy, behavior, incident, safety, payment, eligibility, refusal, cancellation, capacity/waitlist, legal/privacy, or policy exception.
- Deterministic send path requires human approval before external delivery.

### Required fields

- Draft ID, intended recipient/customer, channel, location, reservation/stay/pet/customer context, language/tone, and send deadline if any.
- Source facts with citations: approved care evidence, document status, payment/policy status, incident disposition, booking/capacity status, staff notes, or manager decisions.
- Sensitivity tags: medical, behavior, incident, payment, complaint, eligibility/refusal, legal/privacy, child/minor not expected here, or ordinary update.
- Requested approval: approve/send, approve with edits, reject/suppress, request more info, route to another queue, or escalate.
- Recipient-safety and data-minimization check: no unnecessary PII, payment/provider secrets, internal staff blame, unsupported claims, or unapproved promises.

### Priority/severity

- `Critical`: draft involves active safety/medical/legal/privacy issue or already-sent incorrect content requiring urgent correction.
- `High`: customer-facing send due before arrival, daily update window, checkout, incident follow-up, complaint response, or booking decision.
- `Normal`: routine but non-template customer message awaiting review.
- `Low`: internal wording review or future-dated low-risk draft.

### Owner role

- Manager/admin owns sensitive, policy-exception, incident, complaint, payment/refund/waiver, rejection/refusal, or legal/privacy drafts.
- Front desk/lead may review routine factual drafts only when policy grants that authority.
- AI may draft but cannot self-approve or send unless a separate deterministic send path has fixed facts, recipient, template, and send condition.

### SLA/due semantics

- Before any external send.
- Before the operational milestone the message affects: booking acceptance, arrival/check-in, daily update window, checkout, complaint response, incident follow-up, or payment deadline.
- If no approved send timing exists, keep due basis unresolved rather than inventing a response SLA.

### Allowed actions

- Approve as written, approve with edits, reject, suppress, request more evidence, or route to incident/document/special-care/complaint/payment review.
- Attach approved draft to a deterministic send workflow if available.
- Record manager-approved tone, constraints, and source facts for future templates.
- Create customer-follow-up task for manual send/call.

### Forbidden actions

- Do not send customer-facing messages autonomously from the queue item.
- Do not include unverified facts, internal speculation, staff blame, raw provider/payment payloads, medical claims, legal conclusions, refund promises, or capacity/booking promises without authority.
- Do not turn a draft approval into provider writes, payment movement, reservation confirmation/cancellation, or eligibility changes.

### Audit/compliance requirements

- Retain draft version history, source facts, reviewer, edits, approval/rejection/suppression reason, final message reference, and send actor/tool if sent.
- Preserve explicit reason when customer-visible content is suppressed.
- Record any post-send correction as a complaint/incident/privacy workflow if applicable.

### Customer-visible outcomes

- Approved customer message, call script, email/SMS/app message, or intentional no-send outcome.
- Customer sees only approved final content, not internal review comments or rejected draft text.

### Closure evidence

- Approved/sent reference, approved/manual-call task completion, rejected/suppressed decision, or routed queue item.
- Evidence that final content matches approved facts and channel/recipient.

## Queue 6: complaints

### Source triggers

- Customer message/call/review/social/imported note expresses dissatisfaction, dispute, service-quality concern, incident concern, billing/refund complaint, staff conduct issue, missing update, booking/policy objection, or care concern.
- Front desk/care staff escalates a customer interaction.
- AI sentiment/keyword detection may flag possible complaint but cannot decide final service recovery, refund, waiver, fault, or legal response.

### Required fields

- Customer, reservation/stay, pet(s), location, channel, received-at time, staff recipient if known, complaint text/reference, and current response state.
- Complaint category: care/safety, incident, document/eligibility, booking/capacity/waitlist, payment/refund/waiver, staff conduct, communication/daily update, grooming/training/add-on, policy exception, legal/privacy, or other.
- Source evidence and related records: incident/task/document/payment/draft/staff notes/provider references.
- Desired customer outcome if stated, operational impact, deadline pressure, and whether service recovery/payment exception is requested.
- Review gates: manager, incident, payment/refund, legal/compliance/privacy, staff HR/ops, provider/integration, or executive escalation.

### Priority/severity

- `Critical`: active safety risk, legal/regulatory threat, privacy breach, suspected fraud/abuse, public escalation with sensitive facts, or already-sent harmful misinformation.
- `High`: unresolved complaint affecting active stay, checkout, booking acceptance, refund/waiver request, incident allegation, or customer deadline today.
- `Normal`: ordinary service complaint needing manager response.
- `Low`: retrospective feedback with no active customer-response deadline or safety/payment issue.

### Owner role

- Manager/admin owns complaint disposition and customer response.
- Front desk may acknowledge receipt using approved non-committal wording if policy allows.
- Payment reconciliation owns factual payment investigation, but manager owns refund/waiver/credit decisions.
- Legal/compliance/privacy or higher management owns threats, privacy, regulatory, fraud, abuse, or high-risk public-review matters.

### SLA/due semantics

- Immediate for safety, legal/privacy, active-stay, or public-escalation risks.
- Before checkout/release if complaint affects release, billing, incident disclosure, or customer trust.
- Before any promised response time; if no promise exists, use approved location complaint-response policy or mark unresolved.
- Before refund/waiver/credit/customer promise is communicated.

### Allowed actions

- Acknowledge receipt with approved non-committal language.
- Collect evidence, link related incident/payment/document/draft/special-care items, and assign staff fact-finding tasks.
- Approve customer response, apology/service recovery within policy, operational correction, refund/waiver/credit escalation, or suppression/no-action reason.
- Escalate to legal/compliance/privacy, payment reconciliation, staff/HR channel, engineering/provider support, or higher manager.

### Forbidden actions

- Do not admit fault, promise refund/credit/waiver, alter charges, disclose staff discipline, disclose internal notes, or make legal/medical claims without the right approval.
- Do not let AI respond directly to complaints outside an approved deterministic template and send path.
- Do not close a complaint as resolved because sentiment looks calmer or because an AI summary says it is resolved.

### Audit/compliance requirements

- Retain complaint source, category, related records, evidence collected, reviewer decisions, response drafts/final sends, service recovery decisions, and escalation references.
- Separate factual investigation from customer-facing response and payment/provider execution.
- Minimize sensitive internal/staff/legal/payment data in customer-visible output.

### Customer-visible outcomes

- Approved acknowledgement, investigation update, final response, service recovery offer, refund/waiver decision, policy explanation, or no-action/suppression decision where appropriate.
- Customer-visible messaging should be factual, empathetic, source-backed, and avoid promises outside approved authority.

### Closure evidence

- Manager disposition plus final customer response or explicit suppression/no-contact reason.
- Related incident/payment/document/provider/staff follow-up items closed or intentionally left open with owner/due state.
- Any service recovery, refund, credit, waiver, or provider action recorded through its own approved/audited workflow.

## Escalation workflow

1. Detect and draft. Staff, system validators, AI workers, provider imports, daily briefs, and customer channels may create draft review items with source refs and uncertainty labels.
2. Normalize. Convert raw signals into one of the six queue categories with typed subject, source trigger, priority, owner, due basis, and required fields.
3. Route. Assign manager/admin for final decision. Assign lead/front desk/care staff sub-tasks only for evidence collection or ordinary factual preparation.
4. Freeze unsafe execution. Until review closes, block customer-facing sends, provider writes, payment/refund/waiver actions, booking acceptance exceptions, group-play reinstatement/override, and care-plan execution where required evidence is ambiguous.
5. Review. Manager either approves, approves with constraints, rejects, suppresses, requests info, delegates collection, or escalates to specialist review.
6. Execute through bounded paths. Approved external actions must happen through their own audited message, provider, payment, or staff-task workflow. Queue approval is not itself execution.
7. Close with evidence. Closure requires the category-specific evidence, decision reason, audit event, and any child task or execution reference.
8. Feed quality loops. Rejected AI outputs, repeated document conflicts, complaint themes, and incident patterns may create engineering/policy/template follow-up tasks, but they must not expand automation authority without approval.

## Cross-category routing rules

- Incident plus complaint: keep the incident queue item as the safety/fact source of truth and link a complaint item for customer response/service recovery.
- Document uncertainty plus special-care booking: document queue decides proof/eligibility status; special-care queue decides care/capacity/acceptance constraints.
- Rejected AI output plus approval draft: the rejected-output item records model/draft failure; a new or revised approval-draft item carries the candidate customer message.
- Complaint plus payment/refund/waiver request: complaint queue owns customer relationship; payment/reconciliation or manager payment review owns financial truth and approved money movement.
- Special-care booking plus incident: incident item decides safety/behavior consequences; special-care item decides future stay constraints, group-play status, or booking acceptance.

## Implementation implications

- Model queue category, state, priority, severity, owner, due basis, source refs, evidence refs, review gate, allowed actions, customer-visible outcome, and closure evidence as semantic fields, not a single notes blob.
- Preserve current `StaffTask` compatibility by allowing manager queue items to link to one or more `StaffTask`s and by using `NeedsManagerReview`/`Blocked` for staff-task projections.
- Do not overuse `StaffTaskKind::CustomerFollowUp` for all customer-facing work; approval drafts need draft state, reviewer, source facts, and send/suppression disposition.
- Keep queue approval distinct from execution. Sending a message, mutating a reservation/provider record, charging/refunding, or changing eligibility should produce a separate audited event.
- Treat all AI-authored content as draft/suggestion unless a future approved policy narrows it to deterministic, verified, template-bound behavior.
- Require source freshness and conflict handling. Stale, missing, or contradictory facts route to `NeedsInfo`/`Escalated`, not `Approved`.

## Open policy questions

- Exact SLA values by queue category, service line, operating hours, and location timezone.
- Which pilot roles may approve routine document status or routine customer messages without manager involvement.
- Whether manager review queue items are a separate aggregate or a specialized projection over `StaffTask` plus workflow events.
- Which customer-message templates are deterministic enough for non-manager approval or eventual automated send paths.
- Which provider/Gingr write-backs are in MVP scope and how queue approvals map to bounded execution tools.
- Retention/redaction rules for incident media, complaint evidence, medical/document data, and rejected AI outputs.
