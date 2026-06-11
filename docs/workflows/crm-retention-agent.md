# CRM and retention agent workflow

Status: canonical workflow artifact for the current CRM/retention workstream. This document synthesizes the completed part artifacts in `docs/workflows/crm-retention-parts/` plus related customer-messaging policy into one integration contract for product, data-model, task-model, and implementation planning.

This document is a workflow/specification artifact only. It does not authorize live customer-facing marketing automation, review requests, rebooking sends, complaint responses, public-review replies, discounts, refunds, credits, waivers, retention offers, booking/provider mutations, consent/DNC changes, or production campaign execution.

Anything marked candidate, draft, proposed, configurable, or review-gated requires explicit location/product/manager policy approval before it becomes production behavior. Customer-facing sends and offers remain gated unless a later deterministic policy fixes the category, template, channel, facts, suppression set, consent basis, quiet-hours/cadence, idempotency, audit, and execution adapter.

## Purpose and non-goals

The CRM and retention agent answers four operational questions:

1. What lifecycle stage best describes this customer/pet/location context right now?
2. Is a post-service review request appropriate, suppressed, or review-required?
3. Is a rebooking, reminder, winback, package, VIP, or retention opportunity appropriate, suppressed, or review-required?
4. What risk-forward staff history summary or complaint-recovery handoff is needed before any outreach?

Allowed outcomes:

- classify lifecycle stage from source-backed evidence;
- produce suppression/no-action reasons with source refs and policy version;
- create internal staff, manager, reputation, payment, or data-quality tasks;
- prepare review packets for marketing/reputation/rebooking/retention decisions;
- draft customer-facing copy only as review-gated output when facts and template path support it;
- produce staff-facing customer/pet history summaries with risk flags and citations;
- route complaint recovery cases to manager-owned review and maintain suppression while unresolved.

Non-goals and blocked actions:

- no autonomous customer-facing marketing, review-request, rebooking, winback, VIP, or promotional sends;
- no autonomous complaint response, public-review reply, apology commitment, legal statement, or service-recovery promise;
- no autonomous discount, refund, credit, waiver, retention offer, loyalty reward, package adjustment, deposit exception, or payment action;
- no booking confirmation, reservation modification, capacity/availability promise, waitlist promotion, package enrollment, eligibility decision, or provider write from CRM logic alone;
- no AI clearing of complaints, incidents, DNC/opt-out, consent conflicts, legal/privacy holds, payment disputes, duplicate identity conflicts, behavior restrictions, or source conflicts;
- no use of raw provider payloads, raw OCR, raw email/SMS bodies, unredacted free text, prior AI summaries, or model memory as direct business truth without normalized source refs.

## Source artifacts

Canonical CRM/retention part docs:

- `docs/workflows/crm-retention-parts/inputs.md`
- `docs/workflows/crm-retention-parts/lifecycle-stages.md`
- `docs/workflows/crm-retention-parts/review-request-workflow.md`
- `docs/workflows/crm-retention-parts/rebooking-workflow.md`
- `docs/workflows/crm-retention-parts/customer-history-summary-agent.md`
- `docs/workflows/crm-retention-parts/complaint-recovery-workflow.md`

Related messaging and policy anchors:

- `docs/workflows/customer-messaging-parts/inputs.md`
- `docs/workflows/customer-messaging-parts/send-draft-approval-policy.md`
- `docs/workflows/customer-messaging-parts/message-generation-schema.md`
- `docs/workflows/customer-messaging-parts/message-categories.md`
- `docs/workflows/customer-messaging-parts/messaging-channels.md`
- `docs/workflows/customer-messaging-parts/tone-and-compliance-rules.md`

Additional domain/workflow anchors referenced by the parts include staff operations, booking triage, incident escalation, payments/pricing, security/audit, boarding/daycare/grooming/training implication docs, and current domain contracts in `domain/src/entities.rs`, `domain/src/operations.rs`, `domain/src/workflow.rs`, `domain/src/policy.rs`, and `domain/src/agents.rs`.

Known caveats inherited from the source packet:

- No approved CRM/retention operating policy or template catalog exists yet.
- No dedicated consent/marketing/opt-out/quiet-hours/over-contact aggregate exists beyond preference and policy references.
- No dedicated typed do-not-contact model exists yet, but DNC/suppression must be treated as a hard-stop policy fact.
- Location-specific prices, discounts, deposit rules, no-show thresholds, blackout windows, review links, offers, and final brand copy remain policy data.
- Sentiment, complaint, public-review, and free-text signals require evidence refs, trust markers, and human review when risky or unresolved.

## Required inputs and assumptions

Every CRM/retention evaluation should consume a scoped, source-backed input packet rather than loose prose.

```text
crm_retention_input:
  scope:
    location_id
    customer_id
    pet_ids[]
    timezone
    policy_refs[]
    source_system_refs[]
  lifecycle_context:
    current_candidate_stages[]
    stage_evidence[]
    suppression_flags[]
    lifecycle_override_refs[]
  contact_policy:
    preferred_contact
    channels[]
    destination_refs[]
    consent_by_channel_and_purpose
    opt_outs
    dnc_state
    legal_privacy_or_manager_holds[]
    quiet_hours_policy
    over_contact_state
    prior_delivery_suppression
    last_contact_by_channel_and_purpose[]
  reservation_history:
    recent_and_relevant_reservations[]
    upcoming_or_active_reservations[]
    cancellations[]
    no_shows[]
    waitlists[]
    checkout_or_completion_refs[]
  service_history:
    boarding_stays[]
    daycare_attendance_pattern
    grooming_history
    training_history
    day_spa_or_addons[]
    packages_memberships[]
  cadence_and_seasonality:
    observed_boarding_patterns[]
    holiday_or_peak_policy_refs[]
    daycare_recurrence_refs[]
    grooming_cadence_refs[]
    customer_or_staff_timing_preferences[]
  risk_context:
    incidents[]
    open_concerns[]
    complaints[]
    manager_flags[]
    payment_or_refund_disputes[]
    medical_behavior_care_flags[]
    cancellation_or_no_show_restrictions[]
  sentiment_reputation:
    review_refs[]
    sentiment_evidence[]
    negative_signal_refs[]
    public_response_refs[]
    service_recovery_state
  opportunities:
    rebooking_candidates[]
    review_request_candidate
    vip_candidate
    lapsed_candidate
    package_or_offer_candidates[]
  approval_context:
    required_gates[]
    prior_approvals[]
    manager_task_refs[]
    audit_refs[]
    source_gaps[]
```

Required source-state assumptions:

- Every material field must be marked `trusted`, `untrusted`, `missing`, `stale`, `conflicting`, `imported_only`, or `source_pending`.
- Missing or conflicting evidence routes to suppression, staff review, manager review, data-quality review, or no action. The agent must not fill gaps by inference.
- `Customer.preferred_contact` is a preference, not legal permission. A message candidate needs purpose-specific consent/opt-out/DNC/quiet-hours/over-contact facts.
- Channel availability is not channel permission. Failed delivery does not authorize silent channel switching.
- AI sentiment, prior AI summaries, raw free text, OCR, provider notes, and model memory are not final operational truth without underlying evidence refs.
- Suppression/no-action is a valid terminal output and must record reason code, scope, policy version, source refs, and next owner where applicable.

## Lifecycle stages and precedence

Select one primary operational lifecycle stage using this priority order:

1. `do_not_contact`
2. `complaint_recovery`
3. `active_booking`
4. `first_time_customer`
5. `vip`
6. `lapsed`
7. `repeat`
8. `lead`

Secondary qualifiers may coexist with the primary stage, for example `vip_qualifier=true` while the primary stage is `active_booking` or `complaint_recovery`.

Recommended classifier output:

```text
lifecycle_classification:
  primary_stage: lead | first_time_customer | active_booking | repeat | lapsed | vip | complaint_recovery | do_not_contact | needs_review
  secondary_qualifiers[]
  evidence_refs[]
  suppression_flags[]
  missing_or_conflicting_facts[]
  allowed_outputs[]
  blocked_outputs[]
  required_human_actions[]
  policy_refs[]
  audit_refs[]
```

Stage definitions and safe outputs:

| Stage | Entry summary | Safe outputs | Blocked or gated outputs |
| --- | --- | --- | --- |
| `do_not_contact` | Opt-out, unsubscribe, DNC, legal/privacy hold, manager/admin suppression, consent revoked, or ambiguous suppression scope. | Internal-only notes, suppression reasons, consent/DNC review tasks, operational/legal-required contact only under separately approved policy. | Promotional, review, rebooking, winback, VIP, package, or routine retention drafts/sends; AI/autonomous consent or DNC changes. |
| `complaint_recovery` | Unresolved complaint, customer concern, negative sentiment, service recovery case, payment/refund dispute, incident-adjacent concern, manager flag, or public review needing response. | Manager task, evidence packet, manager-review-only response draft, active suppression record, recovery timeline. | Marketing, review requests, routine rebooking, VIP outreach, package upsell, public responses, complaint sends, offers/refunds until authorized clearance. |
| `active_booking` | Confirmed, checked-in, active/in-care, operating-day arrival/departure, active care/Pawgress, or unresolved checkout/follow-up. | Staff summary, operational tasks, approved-evidence update drafts, suppression for promotional/review/rebooking outreach. | Promotional retention, review request, routine rebooking prompt while in care or unresolved; unsupported care/payment/booking claims. |
| `first_time_customer` | First trusted confirmed/active/completed booking or first completed service in scope. | Staff prep, missing-info tasks, reviewed post-first-service follow-up candidates, review-request candidate after clean completion. | Automated review request, retention offer, discount, package recommendation, future eligibility/availability claim. |
| `vip` | Configured high-value/high-frequency/package/member/manager-tag evidence and no higher stop. | Staff-facing VIP context, reviewed appreciation/concierge candidate, manager recognition task. | Bypassing DNC, consent, complaint, capacity, payment, eligibility, or approval gates; autonomous perks/offers. |
| `lapsed` | Prior trusted completed history and no active booking after configured service-specific inactivity/cadence threshold. | Staff-reviewed winback/rebooking candidate, internal lapsed queue item, suppression/no-action reason. | Autonomous winback, promotion, review request, or rebooking send without all gates. |
| `repeat` | Trusted multi-visit/service history or recurring usage with no higher stage. | Staff context, reviewed rebooking/package/retention candidate, suppression/no action. | Discounts, package changes, payment/deposit exceptions, or campaign sends without approval. |
| `lead` | Inquiry/lead identity without trusted active/completed customer history. | Internal follow-up task, reviewed acknowledgement or missing-info draft, staff-facing lead summary. | Booking promise, availability/price/deposit promise, eligibility decision, promotion, autonomous marketing. |

Transition rules:

- Any DNC/opt-out/legal/privacy/admin suppression sets primary `do_not_contact` for the applicable scope until authorized clearance.
- Any unresolved complaint/concern/dispute/negative sentiment/manager recovery flag sets `complaint_recovery` unless DNC has stronger contact-suppression priority.
- Reservation confirmed/checked-in/active/in-care or unresolved checkout moves operational routing to `active_booking`.
- First confirmed/completed booking with no prior trusted history enters `first_time_customer` or `active_booking` depending on immediate service state.
- Second completed service or trusted recurrence moves from `first_time_customer` to `repeat` if no higher stop applies.
- Configured inactivity/cadence thresholds can move repeat/first-time/VIP contexts to `lapsed`, but missing thresholds must not create lapsed status.
- VIP may be primary only when configured criteria or authorized manager tags exist and no higher stage applies.
- Complaint recovery re-enters ordinary lifecycle only after authorized resolution/clearance with audit refs and renewed consent/suppression checks.
- DNC clears only through authorized consent-management/admin action with scope, actor, timestamp, reason, and audit refs.
- If required facts are missing/conflicting and no hard-stop evidence exists, choose `needs_review` or the safest supported stage and suppress risky outbound effects.

## Review request eligibility, suppression, and cadence

A review request is considered only after trusted service completion or checkout. A trigger is not enough to send.

Possible decisions:

- `eligible_for_review_request_draft`: all gates pass; customer-facing copy remains draft/review-gated unless an approved deterministic send path exists.
- `auto_send_candidate`: allowed only after future policy/template/channel approval explicitly authorizes review-request auto-sends.
- `suppressed`: no customer-facing review prompt; record reason and source refs.
- `manager_or_staff_review_required`: risk, ambiguity, mixed history, or missing facts require human review.

All eligibility gates must pass:

1. Completion gate.
   - Trusted `reservation.checked_out`, `service.completed`, `checkout.completed`, or staff-confirmed completed service exists.
   - Completion timestamp, location, service line, customer, pet, service-history anchor, and source refs are present.
   - Active/in-care, pending checkout, cancelled, no-show, rejected, waitlisted, partially completed, or raw/unverified completion states suppress.
2. Clean-service gate.
   - No open incident, injury, illness, escape/lost-pet issue, bite/fight/aggression event, medication error, facility hazard, safety concern, care/behavior/medical/vaccine/document issue, checkout issue, late pickup issue, or manager/admin/reputation/legal hold.
3. Complaint/payment gate.
   - No active complaint, unresolved concern, negative feedback, service recovery case, refund/credit/waiver request, chargeback, billing dispute, payment exception, disputed cancellation/no-show, or staff sensitivity note.
4. Sentiment gate.
   - Sentiment is positive/neutral and source-backed, or manager-cleared. Missing, stale, contradictory, low-confidence, AI-only, mixed, negative, or risky sentiment suppresses or escalates.
5. Contact-policy gate.
   - Recipient, channel, destination, purpose-specific consent, opt-out/DNC state, quiet-hours policy, timezone, and prior delivery suppression are present and satisfied.
6. Cadence and over-contact gate.
   - No review request for the same customer/pet/location/service/history anchor has already been sent, queued, approved, or suppressed in the active policy window.
   - Overall contact frequency is within configured caps by customer, channel, purpose, service line, pet, and time window.
   - Recent sensitive operational, complaint, incident, payment, or manager contact suppresses or escalates even if numeric limits are not exceeded.
7. Template/brand gate.
   - Approved review-request template/link/copy policy exists for the location/channel if customer-facing delivery is considered.
   - Copy is warm, optional, truthful, non-coercive, and source-grounded.

Hard suppression examples:

- service not checked out/completed, active/in-care stay, or missing completion source;
- incident, injury, illness, aggression, medication error, facility hazard, safety/medical issue;
- unresolved concern, complaint, negative sentiment, public review response need, recovery case;
- refund/payment/chargeback dispute or cancellation/no-show conflict;
- manager, staff, legal, privacy, reputation, DNC, opt-out, consent, quiet-hours, delivery, or over-contact block;
- mixed/ambiguous history, stale service data, identity conflict, or contradictory notes;
- no approved review-request template/link/policy for the location/channel.

Cadence, timing, and idempotency:

- Earliest consideration is after trusted completion and enough post-checkout reconciliation time for same-day issues to surface.
- Default candidate timing, expiry, and cooldowns are configurable by location, channel, and service line; do not hard-code examples as policy.
- Quiet hours must be evaluated in the location/customer timezone. Unknown quiet-hours policy suppresses automation or routes to review.
- Use one review-request attempt per service-history anchor within the policy window, plus broader customer/location/channel/purpose caps.
- Cross-service or multi-pet completions should bundle only under approved policy; otherwise staff chooses one reviewed candidate or suppresses duplicates.
- Retry delivery only for the exact approved payload under adapter policy; do not regenerate copy, switch channels, or bypass suppression.

Suggested idempotency keys:

```text
review_request_eligibility:v1:{location_id}:{customer_id}:{pet_or_group_ref}:{service_line}:{completion_event_id}:{policy_version}
message_draft:v1:{customer_id}:review_request:{service_history_anchor}:{template_version}:{policy_version}
approved_send:v1:{approved_draft_id}:{recipient_ref}:{channel}:{approval_id}
review_request_suppression:v1:{location_id}:{customer_id}:{service_history_anchor}:{reason_code}:{policy_version}
```

Review-request output contract:

```text
review_request_decision:
  decision: eligible_for_review_request_draft | auto_send_candidate | suppressed | manager_or_staff_review_required
  reason_codes[]
  subject_refs: customer_id, pet_ids[], location_id, service_or_reservation_id, completion_event_id
  contact_gate
  risk_gate
  cadence_gate
  message_mode: no_message | draft_only | approval_required | approved_auto_send_candidate
  template_refs[]
  human_review_reason
  idempotency_key
  audit_refs[]
```

Copy constraints:

- no incentives, discounts, refunds, loyalty points, sweepstakes, quid-pro-quo, pressure, guilt, urgency manipulation, review gating, or review filtering;
- no sensitive incident, medical, vaccine, behavior, payment, refund, complaint, or service-recovery details;
- no request to post only positive feedback or to change/remove a review;
- include opt-out/unsubscribe/contact-preference handling when required by policy.

## Rebooking eligibility, suppression, and cadence

The rebooking workflow is recommendation-first and review-gated. It may identify boarding, daycare, grooming, training, package, or winback opportunities, but it must not promise availability, modify reservations, enroll packages, apply discounts, charge/refund, or send outreach unless a separate approved messaging/execution path exists.

Output order:

1. suppression/no-action when contact, risk, or policy blocks outreach;
2. internal staff task when history is useful but customer-facing outreach is not safe;
3. review packet when a human decision is required;
4. draft customer prompt only when facts, timing, consent, and template path are clean;
5. approved send or provider mutation only through a later deterministic approval/outbox path, never from this workflow alone.

Hard suppression: no customer-facing draft/send when any of these apply:

- DNC, unsubscribe, opt-out, legal/privacy suppression, closed-account suppression, or missing consent for the intended channel/purpose;
- active complaint, unresolved negative sentiment, unresolved service recovery, refund/payment dispute, chargeback, public-review escalation, or manager/customer-experience hold;
- open incident, injury/illness/safety event, aggression/behavior restriction, group-play suspension, medication/care ambiguity, medical/vaccine/document issue, or care/behavior review;
- active care/checked-in state for the same pet, unless the output is approved operational checkout/prep support;
- stale, conflicting, imported-only, or unknown history where cadence would be inferred rather than source-backed;
- prior failed delivery or channel mismatch without approved alternate-channel policy;
- over-contact limit reached by customer, pet, channel, purpose, or service line;
- future confirmed booking already exists for the same pet/service/cadence window;
- location policy disabled for the service line, channel, template, season, or segment.

Review-gated conditions that may produce a staff/manager task but not routine outreach:

- repeated cancellations, late cancellations, no-shows, deposit restrictions, forfeiture questions, or cancellation-policy enforcement;
- holiday/peak boarding with limited capacity, waitlists, manager holds, minimum stays, or deposit/prepayment rules;
- daycare eligibility uncertainty, package/membership ambiguity, missed-visit threshold, group-play reinstatement, or care-lane change;
- grooming cadence outside ordinary 2-8 weeks unless supported by groomer recommendation, customer preference, or manager approval;
- complaint recovery marked resolved but not explicitly cleared for re-entry;
- discounts, credits, waivers, loyalty rewards, package changes, retention incentives, or exceptions;
- sensitive wording about behavior, medical care, incidents, payment, no-shows, denials, restrictions, or policy exceptions.

Eligible candidate baseline:

- customer, pet, location, service line, and source identity are high-confidence and non-conflicting;
- relevant history anchor is completed or explicitly policy-approved;
- no hard suppression applies;
- consent, quiet hours, over-contact, delivery, template, and idempotency facts are present for the intended output, or output is internal-only;
- prior prompt history does not show an equivalent prompt already drafted/sent/suppressed in the same window.

Service-line cadence guidance:

| Service line | Candidate signals | Candidate timing windows | Key constraints |
| --- | --- | --- | --- |
| Boarding | Completed stays, annual/seasonal/holiday patterns, prior peak stays, lead-time patterns, future booking absence. | Post-checkout follow-up 3-10 days after clean checkout; annual/seasonal repeat 45-90 days before observed travel window; peak planning 60-120 days before location peak periods; lapsed boarding after configured grace interval. | No availability, room/suite, price, deposit, minimum stay, cancellation, or booking promises. Holiday/peak prompts often require front-desk/manager review. |
| Daycare | Attendance frequency, stable weekday pattern, package/membership status, missed visits, no-shows, eligibility/group-play state. | Habit maintenance 1-3 business days before expected attendance when no future occurrence exists; missed expected drop-off after at least 1 business day; package review 7-21 days before policy threshold; lapsed daycare after configured missed cycles. | Do not auto-confirm recurrence, materialize reservations, consume visits, charge, enroll, renew, discount, or promise capacity. Suppress during group-play suspension or unresolved incidents. |
| Grooming | Last completed grooming anchor, groomer recommendation, customer preference, ordinary 2-8 week interval, future appointment absence. | Due-soon 7-14 days before target; due-now through grace period; overdue after grace period; post-service prebook at checkout or 1-3 days after clean completion; lapsed grooming beyond ordinary band by configurable threshold. | Cancelled/no-show appointments are not positive anchors. Service suitability, matting, skin/product, medical, handling, pricing, and deposit language require review. |
| Training / package / add-on | Trainer-approved progress, package status, homework/re-enrollment candidate, approved add-on history. | Configurable by program milestone, package/term threshold, or trainer-approved follow-up date. | Trainer/staff review required for progress/outcome claims; no package sale, discount, or outcome guarantee without approval. |

Over-contact and cadence constraints:

- Maintain counters by customer, pet, service line, purpose, channel, and policy window.
- Do not duplicate prompts for the same customer/pet/service/history anchor/policy window.
- Separate operational reminders from marketing/rebooking/winback purposes; consent for one purpose does not imply another.
- Respect quiet hours in the location/customer timezone.
- Do not silently switch channels after failure, opt-out, or missing destination.
- Suppress lower-priority retention prompts when operational messages, incident follow-up, complaint recovery, payment issues, or manager tasks are active.
- For multi-service customers, bundle or prioritize through staff review rather than sending several nudges close together.
- Conservative configurable defaults until policy exists: at most one rebooking/retention candidate per customer per rolling 14 days across all service lines; at most one service-line prompt per pet/service/cadence window; at most one lapsed/winback candidate per customer per rolling 60-90 days; post-complaint re-entry requires explicit clearance.

Rebooking output contract:

```text
rebooking_decision:
  decision_id
  location_id
  customer_id
  pet_ids[]
  service_line: boarding | daycare | grooming | training | package_membership | other
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

## Staff-facing customer history summary schema

The customer history summary is internal-only and risk-forward. It answers: “What should authorized staff know before serving this customer and pet today?” It must not become customer-facing copy.

Invocation scope:

```text
customer_history_summary_request:
  scope:
    location_id
    customer_id
    pet_ids[]
    service_context: front_desk | boarding | daycare | grooming | training | manager | checkout | retention_review
    service_date_or_window
    requester_role
    requester_actor_id
    timezone
    policy_refs[]
  output_visibility: staff_internal_only
  review_profile: routine | sensitive | manager_review_required
```

Summary output schema:

```text
customer_history_summary:
  schema_version
  summary_id
  generated_at
  generated_for:
    location_id
    customer_id
    pet_ids[]
    service_context
    service_date_or_window
    requester_role
  status: ready | needs_staff_review | needs_manager_review | failed_safely
  sensitivity:
    visibility: staff_internal_only
    redaction_profile
    contains_sensitive_flags[]
    customer_facing_use: prohibited
  headline:
    one_line_context
    immediate_attention_flags[]
  identities:
    customer:
      display_name
      customer_id
      identity_confidence
      preferred_contact_summary
      suppression_or_contact_caution
    pets[]
  urgent_flags:
    unresolved_concerns[]
    incident_or_safety_flags[]
    medical_medication_feeding_flags[]
    vaccine_document_flags[]
    behavior_or_eligibility_flags[]
    payment_deposit_billing_flags[]
    complaint_recovery_flags[]
    manager_review_required[]
  service_history:
    boarding_summary
    daycare_summary
    grooming_summary
    training_summary
    other_services_summary
    cancellations_no_shows_waitlists_summary
  preferences_and_handling:
    customer_preferences[]
    pet_preferences[]
    handling_notes[]
    feeding_medication_care_note_refs[]
    document_or_vaccine_status_refs[]
  issues_and_notes:
    open_issues[]
    resolved_but_relevant_issues[]
    manager_notes[]
    staff_notes[]
    operational_flags[]
  opportunities:
    rebooking_or_cadence_candidates[]
    service_reminders[]
    vip_or_loyalty_handling[]
    retention_or_recovery_follow_up[]
    package_membership_or_add_on_candidates[]
  recommended_staff_actions[]
  unknowns_and_conflicts:
    missing_facts[]
    stale_facts[]
    conflicting_facts[]
    denied_sources[]
  source_citations[]
  audit:
    invocation_id
    prompt_packet_manifest_ref
    output_validation_result
    policy_refs[]
    model_runtime_ref
```

Required content rules:

- The first screen must promote unresolved concerns and risk flags before ordinary loyalty/service history.
- Identity conflicts, duplicate profiles, imported-only records, or low-confidence matches require staff review; do not merge or choose identities.
- Service history must distinguish boarding, daycare, grooming, training, add-ons/packages, cancellations/no-shows, and upcoming/active context where relevant.
- Preferences must be separated from care/medical/feeding/medication/document facts. Ambiguous care facts must stay “review required” with source refs.
- Issues/incidents/complaints/manager notes must be present even when empty and must preserve unresolved status, suppression, owner, source refs, and review gates.
- Opportunities are internal candidates only. Each must include source refs, policy/freshness status, suppression checks, and required review gate.
- Every operationally meaningful claim needs source citation, trust marker, and freshness marker.
- Customer-facing use is prohibited. Do not paste staff-only notes, incident narratives, payment internals, care details, or manager rationale into customer-message fields.

Human review is required when the summary includes unresolved complaint/negative sentiment, open incident, behavior/eligibility flag, medical/medication/feeding/vaccine ambiguity, payment/refund/discount dispute, DNC/opt-out/legal/privacy suppression, over-contact risk, identity conflict, manager-only/legal-sensitive note, proposed offer/discount, provider mutation, or customer-facing message.

Failure behavior: if validation fails, return `failed_safely` with a minimal review packet containing scope, missing/invalid categories, denied source categories, and recommended owner. Do not display a partial optimistic summary that hides known risk.

## Complaint recovery path

Enter complaint recovery whenever a trusted or reviewable source indicates an unresolved customer concern that could make routine retention outreach inappropriate.

Trigger conditions:

- direct complaint by email, SMS, portal, phone note, front desk note, or customer reply;
- negative public/private review, survey, NPS/CSAT, social mention, or reputation item;
- incident-adjacent concern about injury, illness, behavior, medication, feeding, belongings, safety, timeline, or owner notice;
- staff/manager flag, dissatisfied checkout, service miss, manager callback request, unresolved reservation/customer flag;
- refund, credit, discount, waiver, chargeback, duplicate/incorrect charge, deposit forfeiture, no-show/cancellation, package/membership, or service-quality dispute;
- automation-safety trigger from review-request, rebooking, VIP, package, promotion, or winback workflow.

Complaint recovery input packet:

```text
complaint_recovery_input:
  scope:
    location_id
    customer_id
    pet_ids[]
    reservation_ids[]
    service_lines[]
    timezone
    policy_refs[]
  trigger:
    trigger_type: inbound_complaint | negative_review | incident_concern | staff_flag | payment_dispute | automation_safety
    source_channel: email | sms | portal | phone_task | public_review | survey | staff_note | provider | other
    source_event_refs[]
    received_at
    source_trust
    urgency: routine | time_sensitive | urgent_manager_review | legal_or_safety_escalation
  issue_summary:
    customer_concern_summary
    requested_outcome_if_any
    sentiment_or_tone_evidence
    unresolved_questions[]
    disputed_facts[]
    sensitive_categories[]
  context:
    customer_context_ref
    pet_context_refs[]
    reservation_context_refs[]
    service_history_refs[]
    staff_task_refs[]
    incident_refs[]
    payment_or_refund_refs[]
    prior_message_refs[]
    prior_commitments[]
    active_suppressions[]
  contact_policy:
    preferred_contact
    channel_availability
    consent_by_purpose
    opt_outs
    dnc
    quiet_hours
    over_contact_state
    failed_delivery_state
  audit:
    idempotency_key
    actor_or_system_ref
    prompt_packet_ref
    redaction_policy_ref
```

Manager task requirements:

```text
manager_task:
  title: "Complaint recovery review: {customer/pet/service/date or concise issue label}"
  priority: routine | high | urgent | legal_or_safety
  owner_role: manager | owner_admin | payment_reconciliation | legal_compliance | service_lead
  due_at
  subjects: customer_id, pet_ids[], reservation_ids[], incident_ids[], payment_ids[], message_thread_refs[]
  source_refs[]
  issue_summary:
    concise_customer_concern
    relevant_reservation_pet_service_context
    timeline
    disputed_or_missing_facts[]
    sensitive_categories[]
    requested_outcome_if_any
  recommended_next_steps[]
  draft_response_ref: optional draft id, never send id
  suppression:
    status: suppressed
    reasons[]
    affected_automation_purposes[]
  resolution_tracking:
    status: open | investigating | waiting_on_customer | waiting_on_internal_review | approved_response_pending_send | commitment_pending | resolved_pending_clearance | cleared | closed_no_action | reopened
    owner
    due_at
    follow_up_commitments[]
  approval_gates[]
  audit:
    idempotency_key
    policy_refs[]
    created_by_actor
    created_from_workflow_event
```

Empathetic response drafts:

- Drafts are `DraftOnly` and manager-review-only. They must never auto-send, auto-queue, post publicly, or imply that the customer has already been contacted.
- Allowed posture: acknowledge concern warmly, thank the customer, reference only verified non-sensitive facts, state a manager-reviewed next step when safe, ask one concise clarifying question if needed, and stay calm/non-defensive.
- Prohibited unless exact human-approved wording exists: liability/fault admission, diagnosis, staff/customer/pet blame, refund/credit/discount/waiver/package promise, outcome promise, investigation result, booking/eligibility promise, public response, review removal/change request, quid-pro-quo, pressure, legal/privacy/insurance/chargeback/fraud language.

Suppression while unresolved:

- Suppress review requests, routine rebooking, lapsed/winback, VIP/appreciation, loyalty, package/membership upsell, cross-sell, public-review solicitation/response automation, and non-essential marketing nudges.
- Do not suppress operationally necessary internal work. Staff/manager tasks, manager-reviewed response drafts, legal/compliance follow-up, payment reconciliation, incident investigation, care follow-up, and required customer-service messages may proceed through their own gates.
- Suppression must be visible to all downstream CRM workflows as a hard stop. If scope is ambiguous, suppress the broader safe scope temporarily and ask the manager to narrow it.

Resolution and re-entry:

Normal CRM/review/rebooking lifecycle may resume only when all minimum criteria pass:

1. Complaint case status is `cleared`, or an approved policy permits re-entry from a documented `closed_no_action` or `resolved_pending_clearance` state.
2. Customer-visible follow-up commitments are done, cancelled with manager-approved reason, or superseded by an approved new commitment.
3. No open incident, care/safety, payment/refund, legal/privacy, DNC, opt-out, failed-delivery, or over-contact suppression still blocks the same purpose/scope.
4. Manager or authorized owner/admin cleared the complaint suppression with actor, timestamp, scope, reason, and policy refs.
5. Downstream workflow revalidates consent, DNC, quiet hours, over-contact counters, purpose, template/approval state, and source freshness at future draft/send time.
6. Review-request eligibility applies any additional cooldown/reconsideration policy; absence of such policy means no autonomous review request.

Complaint recovery output contract:

```text
complaint_recovery_result:
  status: needs_manager_review | suppressed | needs_more_information | rejected_by_policy | failed_safely
  summary
  issue_summary_ref
  manager_task_ref
  response_draft_ref: optional
  suppression_ref
  resolution_case_ref
  required_review_gates[]
  risk_flags[]
  verification_notes[]
  source_refs[]
  audit_refs[]
```

## Consent, marketing, do-not-contact, over-contact, and quiet-hours constraints

Required policy facts for any CRM/customer-message candidate:

- message purpose: operational, review request, rebooking reminder, winback, complaint recovery, package/membership, VIP/appreciation, staff follow-up, or promotion;
- channel and destination ref: email, SMS, portal, or phone-task with provider/account refs;
- consent basis/status by channel and purpose;
- opt-out, unsubscribe, DNC, legal/privacy suppression, manager hold, and delivery-failure state;
- location/customer timezone and quiet-hours policy;
- recent contact history by channel and purpose;
- over-contact counters by customer, pet, channel, purpose, service line, and window;
- suppression conflicts: active complaint, incident, care/behavior/medical/vaccine issue, payment/refund dispute, negative sentiment, cancellation/no-show restriction, legal/privacy hold, stale/missing/conflicting facts.

Conservative defaults until policy exists:

- DNC, opt-out, unsubscribe, legal/privacy suppression, or missing consent blocks marketing, review, rebooking, winback, package, VIP, and promotional retention messages. Internal tasks may still be created.
- Quiet-hours policy must be location-timezone aware. Unknown quiet-hours policy blocks autonomous sends and routes to draft/review or scheduling only after approval.
- Unknown over-contact limits suppress automation rather than risking spam.
- Transactional/operational versus marketing treatment is not hard-coded here; preserve purpose-specific consent policy rather than inferring legal permission.
- Failed delivery creates staff/manual retry work or a reviewed replacement send. Do not silently switch channels without policy and audit.
- Suppression is a final outcome with reason, policy version, source refs, and audit context.
- Idempotency keys must include customer, pet, location, service line, purpose, source event/history anchor, policy window/version, and channel where relevant.

## Human approval gates

### Marketing automation gate

Required before any customer-facing review request, rebooking prompt, winback, VIP appreciation, package/membership upsell, promotion, routine retention campaign, or auto-send category is enabled.

Approval must specify:

- location, channel, service line, category, trigger, and allowed lifecycle stages;
- template id/version, allowed variables, localization, review link or call-to-action, and opt-out/disclaimer text;
- required source facts and source freshness;
- suppression conditions, consent basis, quiet-hours rules, over-contact windows, and cadence;
- idempotency scope, immutable payload/audit/outbox fields, retry/dead-letter behavior, and provider-response handling;
- owner role for policy/template changes.

Without this gate, the CRM agent may only draft, suppress, or route to review.

### Complaint response gate

Required for every complaint, recovery, public-review, social-response, service-recovery, refund-adjacent, incident-adjacent, legal/privacy, or sensitive response.

- Customer-message approval and manager approval are always required.
- Public review/social replies require manager/owner approval with exact final text and platform scope.
- Legal threat, privacy/security concern, staff misconduct, discrimination/accommodation issue, severe harm/death, fraud, or chargeback requires owner/admin/legal/compliance routing.
- AI/model output, sentiment score, urgency, or stale text never clears the gate.

### Discounts, offers, and retention incentives gate

Required before any discount, incentive, retention offer, loyalty reward, package adjustment, credit, refund, waiver, deposit exception, forfeiture change, free service, or payment/billing correction is offered, mentioned, applied, or promised.

- Manager/payment authority must approve exact amount/type/scope and wording.
- AI may prepare a recommendation packet only.
- Offer approval does not imply booking/provider/payment execution; those actions need their own audited workflow.

### Sensitive content and provider mutation gates

Human approval is also required for:

- medical, vaccine, medication, allergy, diagnosis, treatment, incident, safety, behavior, aggression, group-play, eligibility, legal/liability, service denial, payment dispute, cancellation/no-show, or policy-exception language;
- booking/provider mutation, reservation confirmation/modification, capacity/overbooking exception, waitlist promotion, package enrollment, or payment action;
- DNC/opt-out/consent changes, suppression-scope narrowing, duplicate merges affecting suppression, legal/privacy holds, retention holds, and policy/template changes;
- lifecycle overrides when evidence is stale, conflicting, imported-only, materially disputed, or manager-sensitive.

Every approval/override must record actor, role/authority, timestamp, policy version, reason, before/after state, scope, source refs, expiration/review date if any, and audit refs.

## Integration notes and open questions

### Integration notes

- CRM outputs should be structured `WorkflowResult`-style artifacts with `structured_output`, `recommended_actions`, `risk_flags`, `verification`, `human_review_reason`, and audit/source refs where the domain model supports it.
- Current message generation should use draft/review schemas from customer messaging. A CRM review or rebooking draft should project to the Customer Messaging Agent schema, not bypass it.
- Phone is a staff call-task channel by default. Email, SMS, and portal are canonical written draft channels. WhatsApp remains out of scope until typed consent/provider/template/audit semantics exist.
- Staff-facing history summaries should be invoked by CRM/rebooking/review/complaint workflows before customer-facing outreach whenever risk, mixed history, or source gaps exist.
- Complaint suppression must be a first-class shared state visible to review-request, rebooking, VIP, winback, package, and promotional flows.
- Data-model work should add typed consent/DNC/suppression, over-contact counters, complaint recovery cases, lifecycle classifications, idempotency records, review/rebooking decision records, and immutable draft/send/audit payload refs.
- Messaging, retention, booking, payment, incident, document, and provider-write execution should remain separate audited workflows. Approval in one workflow is not permission to mutate another system.
- Validators should block customer-facing output when any required stage, consent, suppression, source evidence, template, timing, idempotency, or approval fact is missing.

### Open questions

- What are the approved lifecycle thresholds for first-time versus repeat, lapsed windows by service, VIP criteria, no-show/cancellation thresholds, package/membership semantics, and complaint re-entry cooldowns?
- What is the final consent model by channel and purpose, including opt-in, unsubscribe, DNC scope, quiet hours, over-contact windows, transactional-versus-marketing policy, retention holds, and legal/privacy holds?
- Which review-request links/templates, rebooking templates, winback/VIP/package templates, complaint-recovery templates, localization, disclaimers, and brand copy are approved by location/channel?
- Which roles may approve marketing sends, complaint responses, public replies, discounts/offers, refunds/waivers, DNC changes, lifecycle overrides, and provider/customer/account actions?
- Which provider fields reliably represent no-show, cancellation reason, complaint, sentiment, package balance, completed grooming anchor, daycare recurrence, future booking, and contact permission?
- How should multi-pet and multi-service households be bundled for review/rebooking outreach without over-contact?
- What deterministic auto-send categories, if any, are approved after validators, template versioning, consent checks, suppression checks, idempotency, audit, and provider execution are implemented?
- What retention analytics are allowed without creating unsafe value-tier, protected-class, or sensitive inference risks?

## Conservative implementation rule

If any lifecycle stage, consent, DNC, quiet-hours, over-contact, source evidence, suppression, sentiment, complaint, incident, payment, template, timing, idempotency, or approval fact is missing, stale, conflicting, risky, or outside policy, the CRM/retention agent may create an internal task, staff-facing summary, suppression reason, manager review packet, or draft-only candidate. It must not send, promise, offer, mutate provider/customer/payment state, clear suppression, or return the customer to promotional automation autonomously.
