# Rebooking workflow rules

Purpose: define actionable, conservative rebooking workflow rules using reservation and service history. This artifact turns CRM/retention inputs into deterministic candidate decisions, review packets, staff tasks, and draft-only customer prompts. It does not authorize autonomous booking, provider writes, discounts, incentives, payment actions, capacity promises, eligibility decisions, or customer-facing sends.

Status: draft workflow policy for downstream CRM/retention implementation. It builds on `docs/workflows/crm-retention-parts/inputs.md`, customer messaging policy, booking triage inputs, staff operations inputs, and the PetSuites boarding/daycare/grooming domain implication docs. Exact local thresholds, templates, consent rules, quiet hours, package terms, peak calendars, and manager authority remain configurable policy inputs.

## Source anchors

Use this workflow with the constraints in:

- `docs/workflows/crm-retention-parts/inputs.md` - lifecycle states, CRM input packet, suppression posture, brand voice, approval gates, and configurable unknowns.
- `docs/workflows/customer-messaging-parts/send-draft-approval-policy.md` and `docs/workflows/customer-messaging-parts/inputs.md` - draft/send separation, customer-safe wording, consent, quiet-hours, over-contact, idempotency, and outbound chain of custody.
- `docs/workflows/booking-triage-parts/inputs.md` - reservation lifecycle, booking/capacity/payment/holiday signals, cancellation/no-show implications, and booking/provider-write boundaries.
- `docs/workflows/staff-operations-parts/inputs.md` - checkout/customer-follow-up, Pawgress/customer-summary, staff-task, incident/care, and manager-review inputs.
- `docs/domain/petsuites/grooming/implications/04-rebooking-cadence-every-2-8-weeks.md` - grooming cadence source, ordinary 2-8 week band, completed-service anchors, no-show/deposit restrictions, and approval boundaries.
- `docs/domain/petsuites/daycare/implications/06-daily-recurring-attendance.md` - recurring daycare attendance, package/membership opportunities, missed-visit review, and occurrence/eligibility/capacity boundaries.
- PetSuites boarding service-domain docs - completed stays, holiday/peak demand, room/suite availability, minimum-stay/deposit/capacity limits, and no availability promise without approval.

## Core stance

The rebooking workflow is recommendation-first and review-gated. It may identify that a customer or pet is likely due for boarding, daycare, or grooming follow-up; it may not promise availability, modify reservations, enroll packages, apply discounts, or send outreach unless a separate approved messaging and execution path exists.

Default output order:

1. Suppression/no-action reason when contact or safety policy blocks outreach.
2. Internal staff task when history is useful but customer-facing outreach is not safe.
3. Review packet when a human decision is required.
4. Draft customer prompt only when facts, timing, consent, and template path are clean.
5. Approved send or provider mutation only through a later deterministic approval/outbox path, never from this workflow alone.

## Input packet

Each evaluation consumes a source-backed `crm_retention_input` or equivalent packet with these fields:

- Scope: `location_id`, `customer_id`, `pet_ids`, timezone, source-system refs, policy refs/versions.
- Contact policy: preferred channel, available destinations, consent by channel/purpose, opt-out/unsubscribe/DNC/legal suppression, quiet-hours state, delivery suppression, last contact by purpose/channel.
- Reservation history: requested/offered/confirmed/checked-in/checked-out/completed/cancelled/no-show/waitlisted/rejected statuses with timestamps and service line.
- Boarding history: completed stays, dates/nights, pets, accommodation/service variants, holiday/peak flags, add-ons, checkout state, future confirmed reservations, cancellation/no-show/deposit flags.
- Daycare history: attendance dates, weekday/frequency pattern, recurrence plan if any, package/membership indicators, missed visits, no-shows, eligibility/group-play status, incident restrictions.
- Grooming history: completed service anchors, service/breed/coat/style where known, groomer/customer cadence preference, ordinary 2-8 week interval evidence, missed/cancelled appointments, future appointments.
- Risk context: incidents, care/medical/behavior flags, complaints, negative sentiment, service-recovery status, payment/refund/dispute state, manager holds, DNC/suppression overrides.
- Audit context: source freshness, confidence/trust markers, conflicts, prior approvals, prior rebooking prompts, idempotency refs, open staff/manager tasks.

Raw provider payloads, unreviewed free text, inferred sentiment, or model memory may appear only as evidence refs with source/freshness/confidence; they are not direct permission to contact.

## Eligibility and suppression rules

Evaluate suppression before timing or personalization. Any hard stop below returns `suppressed` or `internal_task_only` with a reason and audit refs.

### Hard suppression: no customer-facing draft/send

- Do-not-contact, unsubscribe, opt-out, legal/privacy suppression, deceased/closed account suppression, or missing consent for the intended channel/purpose.
- Active complaint, unresolved negative sentiment, unresolved service recovery, refund/payment dispute, chargeback, public-review escalation, or manager/customer-experience hold.
- Open incident, injury/illness/safety event, aggression/behavior restriction, group-play suspension, medication/care ambiguity, or medical/vaccine/document issue that could make routine outreach insensitive or inaccurate.
- Customer currently in active care/checked-in state for the same pet unless the prompt is an approved operational checkout/prep draft; suppress promotional rebooking during active care.
- Unknown, stale, conflicting, or imported-only history when cadence would be inferred rather than source-backed.
- Prior failed delivery or channel mismatch without an approved alternate-channel policy.
- Over-contact limit reached for customer, pet, channel, purpose, or service line.
- Future confirmed booking already exists for the same pet/service/cadence window.
- Location policy disabled for the service line, channel, template, season, or customer segment.

### Review-gated conditions

These may produce a staff or manager task, not routine outreach:

- Repeated cancellations, late cancellations, no-shows, deposit restrictions, forfeiture questions, or cancellation-policy enforcement.
- Holiday/peak boarding with limited capacity, minimum stays, manager holds, waitlists, or deposit/prepayment rules.
- Daycare eligibility uncertainty, package/membership ambiguity, missed-visit threshold, group-play reinstatement, or care-lane change.
- Grooming cadence outside the ordinary 2-8 week band unless supported by groomer recommendation, customer preference, or manager approval.
- Complaint recovery marked resolved but not yet cleared for re-entry into routine marketing/rebooking flows.
- Any recommendation involving discounts, credits, waivers, loyalty rewards, package changes, retention incentives, or exceptions.
- Sensitive customer-facing wording about behavior, medical care, incident history, payment status, no-shows, denials, or restrictions.

### Eligible candidate baseline

A rebooking candidate may proceed to timing evaluation only when:

- Customer, pet, location, service line, and source identity are high-confidence and non-conflicting.
- The relevant history anchor is completed or otherwise explicitly policy-approved as a positive signal.
- No hard suppression applies.
- Consent/quiet-hours/over-contact facts are present for the intended draft/send purpose, or the output is internal-only.
- Prior prompt idempotency does not show an equivalent prompt already drafted/sent in the same policy window.

## Timing windows

Timing windows are policy-configurable. The defaults below are conservative candidate windows, not promises to send.

### Boarding rebooking prompts

Primary signals:

- Previous completed boarding stays by pet/customer/location.
- Repeated annual or seasonal patterns such as spring break, summer vacation, Thanksgiving, winter holidays, long weekends, or local school breaks.
- Prior holiday/peak stays, waitlists, deposit behavior, minimum-stay constraints, and lead time between booking and check-in.
- Future confirmed boarding reservations and recent cancellations/no-shows.

Candidate windows:

- Post-checkout follow-up: 3-10 days after clean checkout may create a staff-reviewed draft mentioning future planning only if no incident, complaint, payment dispute, or unresolved follow-up exists.
- Annual/seasonal repeat: 45-90 days before the next observed holiday/peak or recurring travel window, adjusted by the customer's historical booking lead time when known.
- Peak/holiday reminder: 60-120 days before location-defined peak periods when the customer has prior peak stays or high likelihood, but output should be a planning prompt, not an availability promise.
- Lapsed boarding: after the customer's ordinary boarding season/window has passed by a configurable grace interval, create a staff-reviewed winback/rebooking candidate only if recovery and consent gates pass.

Boarding-specific constraints:

- Do not state availability, room/suite type, price, deposit amount, minimum stay, cancellation rule, or booking confirmation unless those facts are separately approved and current.
- Holiday/peak prompts should prefer `manager_or_front_desk_review` when capacity is tight, waitlist-heavy, held, or stale.
- Multiple-pet households require same-household context and pet-specific eligibility/care flags; do not imply all pets can board together unless approved.

### Daycare rebooking and attendance prompts

Primary signals:

- Attendance frequency by weekday, week, and month.
- Recurring plan, package, or membership status when represented as typed data.
- Drop-off reliability, missed visits, no-shows, customer-requested skips, and package/membership renewal boundaries.
- Eligibility/group-play status, incident restrictions, care-lane changes, and capacity patterns.

Candidate windows:

- Habit maintenance: if a pet has a stable weekly pattern and no future occurrence/reservation exists for the next expected day, evaluate 1-3 business days before that expected attendance date.
- Missed expected drop-off: if an expected attendance day is missed, wait at least 1 business day and route to staff review; do not guilt or pressure the customer.
- Package/membership planning: when package balance, expiration, or membership term approaches a policy threshold, create a review packet 7-21 days before the threshold; any sale, discount, package change, or enrollment remains human-approved.
- Lapsed daycare: after a configurable number of missed expected attendance cycles, create a staff-reviewed check-in candidate if eligibility, incident, complaint, and contact gates pass.

Daycare-specific constraints:

- Do not auto-confirm recurring attendance or materialize reservations from a habit alone.
- Do not consume package visits, charge, enroll, renew, discount, or promise capacity.
- Suppress routine prompts during group-play suspension, unresolved incidents, stale vaccine/eligibility state, or care/behavior review.
- For uncertain recurrence, output a staff task to confirm the schedule rather than a customer prompt.

### Grooming rebooking prompts

Primary signals:

- Last completed grooming service as cadence anchor.
- Groomer recommendation, customer preference, service/coat/breed/style guidance where known, and location policy.
- Ordinary PetSuites grooming recurrence signal: 2-8 weeks when supported by completed grooming history.
- Missed/cancelled grooming appointments, no-show/deposit restrictions, care/product flags, and future appointments.

Candidate windows:

- Due-soon: 7-14 days before the target date derived from the trusted cadence anchor.
- Due-now: from target date through a configurable grace period.
- Overdue: after the grace period, create a staff-reviewed prompt or task; avoid shame/pressure language.
- Post-service prebook: at checkout or 1-3 days after clean completed service, create a staff/groomer-reviewed recommendation if the next cadence was recorded.
- Lapsed grooming: if the last service is beyond the ordinary 2-8 week band by a configurable threshold and no next appointment exists, create a staff-reviewed reactivation candidate only if suppression gates pass.

Grooming-specific constraints:

- A cancelled/no-show appointment is not a positive cadence anchor; use the last completed service and separately surface the missed appointment for review.
- Breed/service-specific cadence is allowed only when represented as trusted policy, groomer recommendation, or customer preference; otherwise use safe defaults or staff review.
- Service suitability, matting, skin/product, medical, handling, pricing, and deposit language require groomer/staff/manager review as applicable.

## Personalization inputs and safe defaults

Allowed personalization should be factual, minimal, and source-backed:

- Customer and pet names.
- Service line and last completed service/stay/attendance date.
- Prior approved cadence source, such as groomer-recommended interval or observed weekday habit.
- Holiday/season label only from location policy or observed prior pattern.
- Preferred channel and timezone when consent permits.
- Staff-friendly rationale: history anchor, expected window, suppression checks, and open risks.

Unsafe or gated personalization:

- Pricing, discounts, offers, loyalty value, package savings, deposit/cancellation enforcement, or financial hardship assumptions.
- Medical, behavior, incident, complaint, refund, no-show, or restriction details in routine customer copy.
- Emotional pressure, guilt, scarcity claims, or claims that a pet "needs" a service unless approved by staff/groomer policy.
- AI-inferred sentiment, household travel plans, or customer value tier without approved source and review.

Safe defaults when data is incomplete:

- Missing cadence: staff review instead of inferred due date.
- Missing consent: internal task only.
- Missing template: draft/review only, no send.
- Missing capacity or appointment windows: ask customer to contact staff or create a staff callback task; do not propose exact availability.
- Conflicting history: history-review task with source refs.
- Unknown over-contact policy: suppress automation and create no more than an internal review candidate.

## Over-contact and consent constraints

The workflow must maintain counters by customer, pet, service line, purpose, channel, and policy window. Exact limits are configurable; conservative defaults should prevent spam rather than maximize outreach.

Minimum constraints:

- Do not send or draft duplicate prompts for the same customer/pet/service/history anchor/policy window.
- Separate operational reminders from marketing/rebooking/winback purposes; consent for one purpose does not imply consent for another.
- Respect channel-specific quiet hours in the location/customer timezone.
- Do not silently switch channels after failed delivery, opt-out, or missing destination.
- Suppress lower-priority retention prompts when an operational message, incident follow-up, complaint recovery, payment issue, or manager task is active.
- For multi-service customers, bundle or prioritize prompts through staff review rather than sending separate boarding/daycare/grooming nudges in close succession.

Suggested conservative counters until policy is approved:

- At most one rebooking/retention prompt candidate per customer per rolling 14 days across all service lines.
- At most one service-line-specific prompt per pet/service per cadence window.
- At most one lapsed/winback candidate per customer per rolling 60-90 days.
- Post-complaint re-entry requires explicit manager clearance, not merely elapsed time.

## Human approval gates

The workflow must explicitly surface required gates on every output.

### Staff/front-desk approval

Required before:

- Customer-facing routine rebooking draft is sent.
- Boarding/daycare/grooming windows are proposed to a customer.
- Missing history, conflicting records, or uncertain recurrence is resolved.
- Channel changes, manual callbacks, or contact correction are attempted.

### Groomer/daycare/lead approval

Required before:

- Grooming cadence outside ordinary defaults or based on coat/service nuance is used.
- Daycare eligibility, group-play readiness, recurring attendance exceptions, or care-lane implications are discussed.
- Care/behavior/medical-adjacent facts affect the recommendation or draft.

### Manager/payment approval

Required before:

- Discounts, incentives, retention offers, package adjustments, credits, refunds, waivers, deposit exceptions, forfeiture changes, or loyalty rewards are offered or mentioned.
- Cancellation/no-show restrictions, payment disputes, complaints, incidents, negative sentiment, denials/restrictions, or policy exceptions affect outreach.
- Holiday/peak capacity pressure, waitlist promotion, manager holds, overbooking, or special accommodations are part of the customer-facing plan.
- Complaint recovery is marked safe for routine rebooking again.

### Product/policy approval

Required before:

- Any auto-send category is enabled.
- Templates, variables, timing windows, over-contact limits, quiet-hours policy, consent model, or idempotency rules are changed.
- The system moves from draft/review outputs to customer-facing delivery or provider mutations.

## Decision algorithm

For each customer/pet/service line:

1. Build the evidence packet and mark fields trusted, missing, stale, conflicting, or untrusted.
2. Apply hard suppression. If blocked, record `Suppressed(reason, source_refs, policy_version)` and stop customer-facing output.
3. Check future bookings/appointments/occurrences and prior prompts for idempotency. Stop if the need is already satisfied or duplicate.
4. Classify service-line signal:
   - Boarding: seasonal/holiday/lead-time pattern, post-checkout clean follow-up, or lapsed window.
   - Daycare: recurring attendance habit, missed expected attendance, package/membership threshold, or lapsed habit.
   - Grooming: completed-service cadence anchor, due/overdue/lapsed status, or post-service prebook recommendation.
5. Determine review gates from risk flags, source confidence, cadence source, capacity/payment/eligibility state, and offer/discount content.
6. Choose output mode:
   - `no_action` when not due or already booked.
   - `suppressed` when a hard stop applies.
   - `internal_task` for missing/conflicting facts or consent-blocked useful follow-up.
   - `review_packet` for staff/groomer/manager decision.
   - `draft_customer_prompt` only when facts and template path are clean and review-gated as needed.
7. Create audit and idempotency refs keyed on customer, pet, location, service line, purpose, history anchor, timing window, policy version, and channel.

## Output contract

Each output should be structured, not only prose:

```text
rebooking_decision:
  decision_id
  location_id
  customer_id
  pet_ids[]
  service_line: boarding | daycare | grooming
  purpose: post_checkout | seasonal_rebook | peak_planning | habit_followup | missed_visit | package_review | grooming_due | lapsed_winback
  timing_window: starts_on, target_on, expires_on, timezone, policy_ref
  status: no_action | suppressed | internal_task | review_required | draft_ready | approved_send_pending_outbox
  evidence_refs[]
  source_quality: trusted | missing | stale | conflicting | untrusted
  suppression_reasons[]
  review_gates[]
  allowed_actions[]
  disallowed_actions[]
  personalization_fields[]
  template_ref_or_missing_reason
  idempotency_key
  audit_refs[]
```

Allowed actions are recommendation artifacts only: create staff task, create review packet, draft message, suppress/no action, or schedule future evaluation. Disallowed by default: send message, book reservation, modify reservation, charge/refund/waive, apply discount, enroll package/membership, clear incident/complaint/DNC, or change consent.

## Acceptance checklist

A rebooking rule is ready for implementation only if it answers:

- What completed history anchor or policy signal triggered it?
- What timing window and timezone apply?
- What facts are required, and how are missing/stale/conflicting facts handled?
- Which consent, DNC, quiet-hours, over-contact, and prior-delivery gates apply?
- Which incident/complaint/payment/cancellation/no-show/care restrictions suppress or escalate it?
- Does the output avoid availability, booking, pricing, discount, package, or policy-exception promises?
- Which human role must approve customer-facing copy, discounts/offers, or sensitive cases?
- What idempotency and audit key prevents duplicate or regenerated outreach?

Conservative downstream rule: when in doubt, suppress customer-facing output and create a source-backed staff or manager review task. The workflow should make the next safe human action obvious without over-automating sensitive retention cases.
