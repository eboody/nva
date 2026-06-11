# CRM lifecycle stages and transition rules

Purpose: define operational, auditable lifecycle stages for pet-resort CRM and retention workflows. These rules classify a customer for internal workflow routing, suppression, review packets, and draft generation. They do not authorize autonomous customer-facing sends, discounts, refunds, booking/provider mutations, DNC changes, or complaint responses.

Source basis: `docs/workflows/crm-retention-parts/inputs.md`.

Status: draft operating definition for downstream CRM workflow design. Location-specific thresholds, consent policy, approved templates, quiet hours, over-contact windows, and authority matrices remain configurable policy inputs.

## Classification principles

1. Classify from trusted, scoped evidence only.
   - Scope every lifecycle decision to `location_id`, `customer_id`, relevant `pet_ids`, service line, timezone, source-system refs, policy refs, and evidence freshness.
   - Normalize provider data into semantic records or evidence refs before using it as CRM truth.
   - Preserve source quality as `trusted`, `untrusted`, `missing`, `stale`, `conflicting`, or `source-pending`.

2. Separate lifecycle state from send eligibility.
   - A customer may be classified as lead, repeat, lapsed, or VIP while still being ineligible for outreach because of DNC, consent gaps, quiet hours, over-contact, active complaint, incident, payment dispute, or missing data.
   - Stage assignment may create internal tasks, summaries, suppression reasons, or review packets even when customer-facing contact is blocked.

3. Prefer one primary operational stage plus secondary qualifiers.
   - Downstream workflow routing should select one primary stage using the priority order below.
   - Non-primary facts can be retained as qualifiers, for example `vip_qualifier=true` while primary stage is `active_booking` or `complaint_recovery`.

4. Missing or conflicting data is not filled by AI.
   - If a required fact is missing, stale, or conflicting, classify to the safest supported stage or create a human review task.
   - Do not infer consent, DNC absence, complaint resolution, completed service history, VIP status, lapsed thresholds, or eligibility from silence.

## Stage priority and hard stops

Priority order for primary stage selection:

1. `do_not_contact`
2. `complaint_recovery`
3. `active_booking`
4. `first_time_customer`
5. `vip`
6. `lapsed`
7. `repeat`
8. `lead`

Rationale:

- `do_not_contact` is the strongest suppression state. It blocks promotional, review-request, winback, rebooking, VIP, and routine retention automation regardless of all other lifecycle evidence.
- `complaint_recovery` is the second hard stop. It suppresses marketing, review requests, routine rebooking prompts, VIP appreciation, and promotional retention until explicitly resolved or cleared by an authorized human/process.
- `active_booking` takes precedence over promotional lifecycle stages because in-care or near-service customers need operational/staff context, not retention nudges.
- `first_time_customer` takes precedence over repeat/lapsed/VIP during the first confirmed or completed experience because onboarding, missing-info, and post-first-service review gates differ from ordinary retention.
- `vip` is a service/attention qualifier, but it never bypasses suppression, complaint, consent, capacity, payment, eligibility, or approval gates.
- `lapsed` takes precedence over ordinary repeat for retention routing when the configured inactivity/cadence window is met and no higher-priority hard stop applies.
- `repeat` applies when the customer has trusted multi-service history but no higher-priority stage applies.
- `lead` is the default only for identity candidates or prospects without trusted active/completed customer history.

Recommended classifier output:

```text
lifecycle_classification:
  primary_stage: one of lead|first_time_customer|active_booking|repeat|lapsed|vip|complaint_recovery|do_not_contact|needs_review
  secondary_qualifiers: []
  evidence_refs: []
  suppression_flags: []
  missing_or_conflicting_facts: []
  required_human_actions: []
  allowed_outputs: []
  blocked_outputs: []
  policy_refs: []
  audit_refs: []
```

Use `needs_review` only when evidence is too incomplete/conflicting to safely choose among operational stages. Do not use `needs_review` to bypass DNC or complaint evidence; those hard-stop stages should still win when present.

## Stage definitions

### 1. Lead

Operational meaning: a prospective or unconverted customer identity with an inquiry or lead record but no trusted confirmed booking, active stay/service, or completed service history in scope.

Entry criteria:

- Trusted evidence of one or more of:
  - `inquiry.received`, web form, portal inquiry, phone/SMS/email lead, walk-in lead, waitlist/request record, imported lead record, or manually created lead.
  - Customer/owner identity candidate with requested service/date/pet info or explicit missing-info needs.
- No trusted evidence of an active/confirmed reservation or completed service for the same scoped customer/location/pet.
- No higher-priority `do_not_contact` or `complaint_recovery` hard stop.

Exit criteria:

- Moves to `active_booking` when a reservation becomes confirmed, checked in, active/in-care, or otherwise service-committed by trusted provider/status evidence.
- Moves to `first_time_customer` when a first completed service is recorded or when a first confirmed booking policy treats newly confirmed first-time bookings as first-time customers.
- Moves to `do_not_contact` if opt-out/DNC/legal suppression is set.
- Moves to `complaint_recovery` if the lead interaction becomes an unresolved complaint, dispute, or manager recovery case.
- Remains lead, or becomes `needs_review`, if identity match, source ownership, or requested service facts are conflicting.

Required data signals:

- Customer/owner identity candidate and confidence.
- Source/channel and timestamp.
- Requested service/date/location/pet references, if available.
- Lead/inquiry status and owner/staff follow-up state.
- Contact preference plus consent/opt-out/DNC status by purpose/channel if any outreach is considered.
- Existing customer/service-history match check.

Missing-data behavior:

- Missing pet, service, date, vaccine, or eligibility data creates a missing-info task or reviewed draft, not a booking promise.
- Missing consent/DNC facts blocks marketing/rebooking-style outreach; internal follow-up tasks may still be created.
- Ambiguous duplicate identity creates a merge/review task before lifecycle escalation.

Allowed outputs:

- Internal follow-up task.
- Reviewed acknowledgement or missing-info draft.
- Staff-facing lead summary.

Blocked outputs without policy/human approval:

- Booking promise, availability promise, price/deposit promise, eligibility decision, promotion, or autonomous marketing send.

### 2. First-time customer

Operational meaning: a customer in their first trusted booking/service experience with the location/service scope, requiring onboarding, missing-info, and first-experience follow-up handling.

Entry criteria:

- Trusted evidence of exactly one first confirmed/active/completed reservation or service in scope, or a first completed service/stay for a customer/pet/location.
- No trusted prior completed service history for the same scoped customer/location/pet/service line, unless policy defines a cross-service first-time distinction.
- No higher-priority hard stop or active booking rule that should own the immediate workflow.

Exit criteria:

- Moves to `active_booking` while the first reservation is currently confirmed/checked-in/active/in-care.
- Moves to `repeat` after a subsequent completed reservation/service or trusted recurring pattern is established.
- Moves to `lapsed` if the first-time customer passes a configured inactivity/cadence threshold after completion.
- Moves to `complaint_recovery` for unresolved negative experience, incident concern, dispute, or manager flag.
- Moves to `do_not_contact` if DNC/opt-out/legal suppression is set.

Required data signals:

- First booking/service status and timestamps.
- Customer/pet/profile completeness and vaccine/document status.
- Location/service line.
- Trusted count of prior completed services/reservations.
- Contact policy facts: consent, opt-out, DNC, preferred channel, quiet-hours, over-contact state.
- Review gates, incidents, payment disputes, or care/behavior/medical flags.

Missing-data behavior:

- If service history is incomplete or imported raw, classify as first-time only when the first-service evidence is trusted and no contrary history exists; otherwise create a history-review task.
- Missing consent/quiet-hours/template blocks customer-facing onboarding or post-service outreach.
- Missing profile/vaccine/care facts creates staff tasks and suppresses unsupported customer-visible claims.

Allowed outputs:

- Staff-facing prep summary.
- Missing-info task.
- Reviewed post-first-service follow-up candidate.
- Reviewed review-request candidate only after completion and only if suppression/consent checks pass.

Blocked outputs without policy/human approval:

- Automated review request, retention offer, discount, package recommendation, or claim about future eligibility/availability.

### 3. Active booking

Operational meaning: a customer has a current or imminent reservation/service requiring operational support, care context, or approved service communication rather than promotional retention.

Entry criteria:

- Trusted reservation/service status such as requested with active staff handling, confirmed, checked-in, active/in-care, operating-day arrival/departure, checked-out with unresolved checkout/follow-up tasks, or active care/Pawgress evidence.
- Includes boarding, daycare, grooming, training, DaySpa, add-ons, and package/membership usage while the service event is live or operationally unresolved.
- No higher-priority `do_not_contact` or `complaint_recovery` hard stop. DNC may block non-required customer contact but does not erase internal active-booking context.

Exit criteria:

- Moves to `first_time_customer`, `repeat`, `vip`, or `lapsed` after checkout/completion once operational tasks, incidents, payment disputes, and required follow-ups are resolved.
- Moves to `complaint_recovery` if an incident/concern/negative sentiment/payment dispute becomes an unresolved recovery case.
- Moves to `do_not_contact` if DNC/suppression is set, while preserving operational exceptions only under approved policy.

Required data signals:

- Reservation/service id, status, service line, dates/times, location/timezone, pet/customer refs.
- Arrival/departure/check-in/check-out timestamps where relevant.
- Care, medical, medication, allergy, behavior, incident, watchlist, task, and Pawgress/update evidence.
- Payment/deposit/checkout/cancellation/no-show flags.
- Contact policy facts and customer-message approval status.

Missing-data behavior:

- Missing or conflicting reservation status creates provider/status review before customer-facing claims.
- Missing care evidence blocks daily-update or Pawgress claims; internal task/summary may note the gap.
- Missing consent/DNC/quiet-hours blocks non-operational messages and routine retention sends.

Allowed outputs:

- Staff summary and risk-forward operational context.
- Task recommendations.
- Daily update or Pawgress draft from approved evidence, routed through required review.
- Suppression reason for promotional/review/rebooking outreach.

Blocked outputs:

- Promotional retention, VIP appreciation, review requests, and routine rebooking prompts while in-care or while unresolved operational/payment/care/incident facts exist.
- Autonomous booking/provider mutation or customer-visible care claim without approved evidence/review.

### 4. Repeat

Operational meaning: a customer with trusted multi-visit/service history or recurring usage pattern, eligible for ordinary retention/rebooking analysis when no higher-priority stage applies.

Entry criteria:

- Trusted evidence of multiple completed reservations/services in scope, or a policy-defined recurring daycare/grooming/training/boarding pattern.
- No active booking needing operational handling.
- No lapsed threshold met.
- No higher-priority DNC/complaint hard stop.

Exit criteria:

- Moves to `active_booking` when a new booking becomes confirmed/active/in-care.
- Moves to `lapsed` after a configured service-specific inactivity/cadence window.
- Moves to `vip` if configured VIP criteria are met and no higher-priority stage applies.
- Moves to `complaint_recovery` for unresolved concerns/disputes/negative sentiment.
- Moves to `do_not_contact` when DNC/suppression is set.

Required data signals:

- Completed service/reservation history count and service lines.
- Recurrence/cadence anchors such as grooming interval, daycare attendance pattern, boarding seasonality, package/membership use, or training completion.
- Last completed service date and relevant pet/location scope.
- Consent/DNC/quiet-hours/over-contact facts for any outreach.
- Incidents, complaints, payment disputes, care/behavior flags, cancellations/no-shows.

Missing-data behavior:

- Incomplete/stale/conflicting history creates history-review rather than inferred cadence or value.
- Missing lapsed thresholds means do not label as lapsed; keep repeat with `lapsed_candidate_unconfigured` qualifier if useful.
- Missing consent/template suppresses customer-facing retention output.

Allowed outputs:

- Staff-facing customer context.
- Reviewed rebooking, package, or retention draft candidate grounded in trusted history and approved policy.
- Suppression/no-action decision when checks fail.

Blocked outputs without approval:

- Discounts, credits, package adjustments, payment/deposit exceptions, or autonomous campaign sends.

### 5. Lapsed

Operational meaning: a formerly active/repeat/first-time customer has no active or completed booking within a configured service-specific cadence/inactivity window and may warrant reviewed winback/rebooking handling.

Entry criteria:

- Trusted prior completed service/reservation history.
- No active/confirmed booking in scope.
- Current date exceeds a configured lapsed threshold for the relevant service line, customer segment, location, and/or known cadence.
- Examples of threshold inputs:
  - Grooming: ordinary cadence can be 2-8 weeks only when supported by completed grooming history and local policy.
  - Daycare/boarding/training: thresholds remain configurable policy inputs.
- No higher-priority DNC, complaint recovery, active booking, or first-time condition.

Exit criteria:

- Moves to `active_booking` when a new booking is confirmed/active.
- Moves to `repeat` after a new completed service re-establishes normal activity and no lapsed threshold applies.
- Moves to `complaint_recovery` if winback context reveals unresolved complaint/negative sentiment/dispute.
- Moves to `do_not_contact` if DNC/suppression is set.
- Moves to `needs_review` if the threshold was applied from stale/conflicting history.

Required data signals:

- Last completed service/reservation date by service line and pet/customer/location.
- Configured lapsed threshold and policy version.
- Active/confirmed booking absence check.
- Known cadence/recurrence/package/membership context.
- Consent, opt-out, DNC, quiet-hours, over-contact, recent contact history.
- Complaint/incident/payment/no-show/cancellation/sentiment suppression checks.

Missing-data behavior:

- Missing threshold: do not classify primary stage as lapsed; retain repeat/first-time and record `lapsed_threshold_missing`.
- Missing active-booking check: suppress winback and create status-review task.
- Missing consent/DNC/complaint checks: suppress outreach; internal review packet only.

Allowed outputs:

- Staff-reviewed winback/rebooking candidate.
- Internal lapsed-candidate queue item.
- No-action/suppression reason.

Blocked outputs:

- Autonomous winback, promotion, review request, or rebooking send without all consent, suppression, template, timing, idempotency, and approval gates.

### 6. VIP

Operational meaning: a high-value, high-frequency, loyal, package/member, or manager-designated customer warranting staff awareness or concierge-style handling, subject to all suppression and approval gates.

Entry criteria:

- Trusted evidence satisfying configured VIP policy such as:
  - Frequent daycare attendance or recurring schedule.
  - High lifetime service volume or spend proxy, if approved and privacy-safe.
  - Repeated boarding/holiday stays.
  - Active package/membership/loyalty status.
  - Multi-service engagement across boarding/daycare/grooming/training.
  - Authorized staff/manager VIP tag.
- No higher-priority DNC, complaint recovery, active booking, or first-time rule.

Exit criteria:

- Moves to `active_booking` during current service events.
- Moves to `lapsed` when configured lapsed criteria exceed VIP precedence for retention routing.
- Moves to `complaint_recovery` for unresolved concerns/disputes/negative sentiment.
- Moves to `do_not_contact` if DNC/suppression is set.
- Removes VIP qualifier only by configured policy expiration or authorized human override.

Required data signals:

- VIP policy criteria and version.
- Supporting history/package/membership/tag evidence.
- Customer/pet/location scope.
- Consent/DNC/complaint/incident/payment/over-contact checks for any outreach.
- Human approval for any perk, discount, offer, package adjustment, or special exception.

Missing-data behavior:

- Missing VIP threshold/policy: do not assign primary VIP from intuition; create `vip_candidate_needs_review` if history suggests loyalty.
- Missing spend/package data cannot be invented from service count unless policy allows that proxy.
- Missing consent/suppression checks blocks appreciation/retention outreach.

Allowed outputs:

- Staff-facing VIP context.
- Reviewed appreciation or concierge follow-up candidate.
- Manager task for possible recognition/offer review.

Blocked outputs:

- Any bypass of DNC, consent, complaint, capacity, eligibility, payment, or approval gates.
- Autonomous discount, credit, waiver, retention offer, or package change.

### 7. Complaint recovery

Operational meaning: an unresolved complaint, concern, incident fallout, negative sentiment, refund/payment dispute, or manager recovery case exists and must be handled through manager/staff review before routine CRM automation resumes.

Entry criteria:

- Trusted or credible evidence of one or more of:
  - Inbound complaint or customer concern.
  - Negative sentiment requiring review from email/SMS/call note/public review/staff note.
  - Incident-related owner concern, care/safety/medical/behavior issue, or service failure claim.
  - Refund, credit, charge, payment, deposit, cancellation, no-show, or policy dispute.
  - Manager/admin complaint/recovery flag or task.
  - Public review needing response.
- Complaint status is unresolved, unreviewed, pending response, pending manager action, or resolution status is missing/conflicting.
- DNC does not erase complaint recovery internally, but `do_not_contact` has higher primary priority for contact suppression.

Exit criteria:

- Authorized manager/staff marks the complaint/recovery case resolved, cleared, or closed with audit refs, resolution summary, allowed follow-up posture, and any remaining suppression state.
- Related incident/payment/refund/service commitments are completed or explicitly transferred to another accountable workflow.
- Customer-facing recovery response, if any, has been approved/sent or intentionally not sent by authorized human decision.
- If DNC/opt-out remains active after resolution, primary stage becomes `do_not_contact` rather than ordinary retention.
- If no DNC and ordinary criteria apply after resolution, reclassify by priority order based on active booking, first-time, VIP, lapsed, repeat, or lead evidence.

Required data signals:

- Complaint/concern source refs and timestamps.
- Customer, pet, reservation/service, location, staff/task owner where known.
- Issue category, severity, status, due date, and manager owner.
- Related incident/payment/refund/review/public-response refs.
- Prior commitments and response history.
- Suppression state and audit refs.

Missing-data behavior:

- Missing resolution status means unresolved.
- Missing source details does not clear suppression; create manager triage task with available refs.
- Conflicting sentiment or duplicate complaint records route to manager review.
- Missing consent does not prevent internal complaint tasking; it blocks non-required customer-facing outreach.

Allowed outputs:

- Manager task with source refs, issue summary, affected pet/service context, owner, due date, proposed next steps, and suppression state.
- Empathetic response draft for manager review only.
- Internal recovery timeline and evidence packet.

Hard stops and blocked outputs:

- Suppress marketing, review requests, routine rebooking prompts, VIP outreach, package upsell, winback, and promotional retention automation until explicitly resolved/cleared.
- No autonomous customer-facing complaint response.
- No admission of liability, diagnosis, blame, refund/credit promise, discount/waiver, policy exception, or concealment of concerning facts without authorized approval.

### 8. Do-not-contact

Operational meaning: a customer/account/channel/purpose is under opt-out, unsubscribe, DNC, legal/privacy suppression, manager/admin suppression, or equivalent contact prohibition. This is a hard-stop suppression state for CRM/retention automation.

Entry criteria:

- Trusted evidence of one or more of:
  - Opt-out, unsubscribe, STOP reply, DNC flag, legal/privacy hold, retention hold, manager/admin suppression, channel failure suppression, or compliance suppression.
  - Consent revoked for the relevant channel/purpose.
  - Authorized suppression event such as `do-not-contact set`.
- Scope must be explicit where possible: customer/account, channel, destination, purpose, location, service line, pet, and effective time.
- If scope is ambiguous, apply the most conservative suppression compatible with policy until reviewed.

Exit criteria:

- Only authorized human/admin or deterministic consent-management process clears/changes DNC with audit refs, source evidence, scope, actor, timestamp, and policy basis.
- If cleared, reclassify using the normal priority order and retain audit history.
- Channel-specific or purpose-specific opt-in may permit only that channel/purpose if policy explicitly supports it; do not infer broad permission from narrow consent.

Required data signals:

- Suppression type, source, timestamp, actor/source system, audit ref.
- Scope: customer/account, channel, destination, purpose, location/service/pet if relevant.
- Consent/opt-out status by channel and purpose.
- Legal/privacy/manager hold details and expiration/review dates where applicable.
- Operational-message exception policy, if any.

Missing-data behavior:

- Missing consent is not identical to explicit DNC, but for marketing/review/rebooking/winback it blocks outbound automation and should be recorded as a suppression flag.
- Ambiguous DNC scope should suppress broadly and create a consent/suppression review task.
- Missing audit/source for a DNC-like flag does not permit outreach; route to authorized review.

Allowed outputs:

- Internal-only notes, suppression reasons, and staff/admin review tasks.
- Operational/legal-required customer contact only under separately approved policy, with purpose-specific audit.

Hard stops and blocked outputs:

- No promotional, review-request, rebooking, winback, VIP, package, or routine retention drafts/sends unless policy explicitly allows internal draft creation without delivery and labels it blocked.
- No AI/autonomous DNC clearing, consent changes, duplicate merges affecting suppression, or suppression-scope narrowing.

## Transition rule matrix

| From | Trigger | To | Required gate |
| --- | --- | --- | --- |
| any | DNC/opt-out/legal/privacy/admin suppression set | do_not_contact | Authorized source/audit; apply conservative scope |
| any except do_not_contact | unresolved complaint/concern/dispute/negative sentiment/manager recovery flag | complaint_recovery | Manager triage; suppress promotional/retention automations |
| lead/repeat/lapsed/VIP/first-time | reservation confirmed/checked-in/active/in-care or operationally unresolved checkout | active_booking | Trusted reservation status; no unsupported customer-visible claims |
| lead | first confirmed or completed booking/service with no prior trusted history | first_time_customer or active_booking | Identity/history confidence; profile/missing-info review |
| first_time_customer | second completed service or recurring pattern established | repeat | Trusted completed history and policy scope |
| repeat/VIP/first-time | configured inactivity/cadence threshold exceeded and no active booking | lapsed | Threshold policy version; active-booking absence; suppression checks |
| repeat/lapsed | VIP criteria met | VIP | VIP policy/version or authorized manager tag |
| lapsed | new booking confirmed/active | active_booking | Trusted reservation status |
| complaint_recovery | authorized resolution/clearance with audit refs | reclassify by priority | Manager/staff approval; remaining suppressions preserved |
| do_not_contact | authorized suppression clearance/scope change | reclassify by priority | Admin/authorized consent-management audit only |
| any | required facts missing/conflicting and no hard-stop evidence | needs_review or safest supported stage | Create human review task; suppress risky outbound effects |

## Human approval and override rules

Human approval is required for:

- Complaint recovery responses, public-review responses, or any customer-facing recovery commitment.
- Discounts, refunds, credits, waivers, retention offers, package adjustments, loyalty perks, deposit exceptions, forfeiture changes, or payment actions.
- DNC/opt-out/consent changes, suppression scope narrowing, duplicate merges affecting suppression, legal/privacy holds, and policy/template changes.
- Booking/provider mutations, reservation confirmations/modifications, capacity/overbooking exceptions, package enrollment, and eligibility decisions.
- Sensitive medical, vaccine, medication, allergy, behavior, incident, safety, legal/liability, service-denial, payment-dispute, cancellation/no-show, or policy-exception language.
- Lifecycle overrides when evidence is stale, conflicting, imported-only, or materially disputed.

Override requirements:

- Record actor, role/authority, timestamp, policy version, reason, before/after stage, scope, source refs, and expiration/review date if applicable.
- Overrides cannot bypass `do_not_contact` or unresolved `complaint_recovery` for promotional/retention automation.
- A manager may mark a complaint resolved only with an auditable resolution/closure state; ordinary positive sentiment or new booking does not automatically clear complaint recovery.
- A staff/admin may label a customer VIP only if policy permits the role; VIP override does not authorize offers or suppressions to be bypassed.
- AI may recommend an override review packet but must not apply the override autonomously.

## Automation posture by stage

| Stage | Internal tasks/summaries | Drafts for review | Autonomous sends/actions |
| --- | --- | --- | --- |
| lead | Allowed | Allowed for acknowledgement/missing-info when facts support it | Blocked unless future deterministic policy approves |
| first_time_customer | Allowed | Allowed for onboarding/post-service/review candidates after checks | Blocked unless future deterministic policy approves |
| active_booking | Allowed | Allowed for operational updates from approved evidence | Promotional/review/rebooking automation blocked |
| repeat | Allowed | Allowed for rebooking/retention candidates after checks | Blocked unless future deterministic policy approves |
| lapsed | Allowed | Allowed for winback/rebooking candidates after checks | Blocked unless future deterministic policy approves |
| VIP | Allowed | Allowed for appreciation/concierge candidates after checks | Offers/perks/actions blocked without approval |
| complaint_recovery | Required when evidence exists | Manager-review-only recovery draft | Customer-facing recovery and promotional automations blocked |
| do_not_contact | Allowed internally | Blocked for CRM/retention delivery; internal blocked-draft only if useful | Blocked except approved operational/legal-required policy |

## Implementation checklist for downstream workflows

Before producing a customer-facing CRM/retention candidate, verify:

1. Primary lifecycle stage selected by priority order.
2. Hard-stop checks completed: DNC/opt-out/legal suppression, complaint recovery, incident/concern, payment/refund dispute, care/medical/behavior review, active booking/in-care, over-contact, quiet hours, consent, template approval.
3. Required data signals are trusted, fresh, scoped, and non-conflicting.
4. Missing facts are represented as source gaps and routed to suppression, staff review, manager review, or no action.
5. Message purpose, channel, recipient, policy refs, template/copy, timing, idempotency key, audit refs, and opt-out handling are explicit.
6. Human approval gates are attached for all sensitive, promotional, recovery, offer, payment, booking, or suppression-affecting outputs.

Conservative default: if any stage, consent, suppression, source evidence, template, timing, idempotency, or approval fact is missing, the workflow may create an internal task, staff-facing summary, suppression reason, or review packet only. It must not send or mutate customer/provider state autonomously.
