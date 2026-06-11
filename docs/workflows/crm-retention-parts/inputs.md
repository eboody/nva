# CRM and retention inputs

Purpose: collect the canonical source inputs and conservative assumptions downstream CRM/retention workflow cards should use. This is an input packet, not live operating policy. It does not authorize customer-facing automation, marketing sends, discounts, retention offers, complaint responses, provider writes, or lifecycle mutations without the review gates named below.

Status: draft source inventory and constraints note for downstream definitions. Missing or location-specific facts must remain configurable or human-approved.

## Source inventory

Primary repo sources checked:

- `docs/workflows/customer-messaging-parts/inputs.md` - customer-message posture, channel constraints, brand-voice constraints, draft/send separation, consent/quiet-hours gaps, message categories, and review gates.
- `docs/security/pet-resort-security-audit-parts/inputs.md` - permission/audit states, actors, prompt-packet constraints, tool-permission strata, outbound chain of custody, suppression as first-class outcome, and required review categories.
- `docs/workflows/booking-triage-parts/inputs.md` - reservation lifecycle, customer/pet/profile inputs, capacity/availability/payment/holiday signals, no-show/cancellation implications, and booking/customer-message approval boundaries.
- `docs/workflows/staff-operations-parts/inputs.md` - operating-day, checkout/customer-follow-up, Pawgress/customer-summary, staff task, care/incident, manager-review, and approval-gate inputs.
- `docs/domain/petsuites/grooming/implications/04-rebooking-cadence-every-2-8-weeks.md` - grooming rebooking cadence, service-history anchor rules, no-show/deposit restrictions, consent gate, and review states.
- `docs/domain/petsuites/daycare/implications/06-daily-recurring-attendance.md` - daycare recurrence, attendance history, package/membership opportunities, missed-visit/no-show review, and customer-message blocks.
- `docs/domain/petsuites/boarding/service-domain-map.md` and boarding implication docs - boarding stay lifecycle, holiday/peak demand, check-in/out, care/behavior/medication notes, Pawgress/reporting, add-ons, and approval boundaries.
- `docs/domain/petsuites/training/implications/05-parent-follow-up.md` - training follow-up, progress/outcome evidence, homework/re-enrollment/package opportunities, and trainer/manager message gates.
- `docs/domain/petsuites/daycare/implications/04-incident-tracking.md` - incident owner-notice constraints, safety/medical/behavior escalation, and no autonomous incident messaging.
- `docs/workflows/payments-pricing.md` and `docs/workflows/payments-pricing-parts/ai-boundaries.md` - payment/deposit/refund/discount/waiver truth, escalation, and redaction boundaries.
- `domain/src/entities.rs`, `domain/src/operations.rs`, `domain/src/workflow.rs`, `domain/src/policy.rs`, and `domain/src/agents.rs` - current semantic anchors for customer/pet/reservation/service/status/contact/audit/workflow/review-gate surfaces.

Missing/caveated sources:

- No approved CRM/retention operating policy or message-template catalog was found before this card. Downstream rules may define candidate workflows, but final outbound copy, cadence, and automation must remain policy-configurable.
- No dedicated consent/marketing/opt-out/quiet-hours/over-contact aggregate was found beyond preferred contact and repeated policy references. Channel availability is not legal contact permission.
- No dedicated do-not-contact model was found, but security/audit inputs require `do-not-contact set` as an identity/account event and customer-message suppression state. Treat DNC as a hard-stop policy fact until a typed model exists.
- Public PetSuites/NVA details are product/domain context, not pilot-approved local policy. Exact prices, discounts, deposit rules, blackout windows, package offers, review links, no-show thresholds, and brand copy remain location-policy data.
- Sentiment/complaint detection sources are not final. Free-text reviews, replies, calls, emails, and staff notes should be evidence refs with trust/source/freshness markings, not unreviewed business truth.

## Customer journey baseline

Downstream lifecycle cards should model the customer journey around explicit statuses, source evidence, and suppression gates rather than a single marketing funnel.

Provisional journey states and inputs:

1. Lead / inquiry.
   - Entry signals: `inquiry.received`, `lead_created`, website/portal/phone/SMS/email source, owner/customer identity candidate, requested service/date/pet refs, missing-info needs.
   - Safe output: internal follow-up task or reviewed acknowledgement/follow-up draft.
   - Hard limits: no booking promise, price/deposit promise, availability promise, eligibility decision, or marketing automation without approved policy/template/facts.

2. First-time customer.
   - Entry signals: first completed or active reservation/stay/service for a customer/pet/location, or newly confirmed booking after lead/inquiry.
   - Needed context: identity confidence, pet/profile completeness, vaccine/document status, service history count, communication preference and consent facts, active review gates.
   - Safe output: staff-facing prep summary, missing-info task, or reviewed post-service follow-up candidate.

3. Active booking / in-care.
   - Entry signals: reservation status `Confirmed`, `CheckedIn`, `Active/InCare`, operating-day arrivals/departures, care/watchlist/tasks, daily update/Pawgress evidence.
   - Safe output: staff summary, daily-update draft from approved evidence, task recommendations.
   - Suppress: review requests and promotional retention outreach while in-care, unresolved payment/care/incident facts exist, or customer-visible facts are not approved.

4. Repeat customer.
   - Entry signals: multiple completed reservations/services, recurring daycare pattern, grooming history, training/package history, or repeated add-on usage.
   - Safe output: staff-facing context and reviewed rebooking/retention draft candidates based on trusted history and policy.
   - Missing-data behavior: if history is incomplete, stale, conflicting, or imported only as raw provider text, create a history-review task rather than infer cadence or value.

5. Lapsed customer.
   - Entry signals: no completed/active booking for a configurable location/service-specific interval after ordinary cadence would suggest follow-up; grooming ordinary cadence is 2-8 weeks when supported by completed grooming history; daycare/boarding/training lapsing windows are unresolved policy inputs.
   - Safe output: staff-reviewed winback/rebooking candidate only if consent, DNC, complaint, incident, payment, and over-contact checks pass.
   - Unknowns: lapsed thresholds by service line, seasonality, customer segment, and channel are policy-configurable.

6. VIP / high-value / loyal customer.
   - Entry signals: configurable history such as frequent stays/daycare cadence, recurring packages/memberships, high lifetime service volume, holiday boarding pattern, or staff/manager tag.
   - Safe output: staff-facing prioritization/context and reviewed appreciation or concierge follow-up candidate.
   - Hard limits: VIP status must not bypass DNC/consent/complaint suppression, capacity/eligibility/payment policy, or approval gates for discounts/offers.

7. Complaint recovery / unresolved concern.
   - Entry signals: inbound complaint, negative sentiment, incident/customer concern, refund dispute, manager flag, public review needing response, unresolved service/care/payment issue, or staff-created complaint task.
   - Safe output: manager task with summarized issue, source refs, related reservation/pet/service context, proposed next steps, and an empathetic response draft only for manager review.
   - Hard stop: suppress marketing, review requests, routine rebooking prompts, VIP outreach, and promotional retention automation until the complaint is explicitly resolved/cleared.

8. Do-not-contact / suppression.
   - Entry signals: opt-out/unsubscribe, DNC flag, legal/compliance/privacy suppression, channel failure/suppression policy, or manager/admin suppression.
   - Safe output: internal-only notes/tasks. No customer-facing draft/send except legally/operationally required messages under separately approved policy.
   - Precedence: DNC overrides lead, repeat, lapsed, VIP, review request, rebooking, and marketing retention flows.

## Reservation and service-history signals

Minimum signal packet for CRM/rebooking/review-request workflows:

- Identity/scope: location, customer, pets, portal/provider refs, identity confidence, source system, timezone, policy refs/versions.
- Contact/preferences: preferred channel, available destination refs, consent/opt-out/DNC/quiet-hours status, prior delivery failure/suppression, last contact by channel/purpose.
- Reservation history: statuses and timestamps for inquiry/requested/missing-info/vaccine-pending/special-review/waitlisted/offered/confirmed/checked-in/active/checked-out/cancelled/rejected/no-show.
- Stay/service history: completed boarding/daycare/grooming/training/DaySpa services, add-ons, packages/memberships, grooming service-history anchor, daycare recurrence/attendance pattern, training progress/completion where offered.
- Cadence/seasonality: holiday/peak boarding patterns, recurring daycare days/frequency, grooming interval/cadence source, customer-requested timing preferences, package/renewal boundaries.
- Exceptions and risk: cancellations, no-shows, missed daycare visits, deposit/payment restrictions, late pickup/checkout, provider conflicts, source gaps.
- Care/safety flags: incident refs/status, behavior restrictions, group-play suspension/reinstatement status, medical/medication/allergy/care review flags, manager notes, unresolved concerns.
- Customer sentiment/reputation: positive/neutral/negative evidence, public/private review refs, complaint status, response commitments, follow-up due dates.
- Audit/review state: prior approvals, human-review packets, suppression reasons, manager task owner/status/due date, source freshness/trust/conflict markers.

Do not use raw provider payloads, unredacted customer free text, raw email/SMS bodies, or unsupported AI memory as direct CRM truth. Normalize to semantic records or evidence refs first.

## Messaging policy and channel constraints

CRM/retention workflows inherit the customer-messaging constraints:

- Default posture: AI may draft, summarize, classify, suppress, or create internal tasks; customer-facing sends are review-gated unless a later deterministic send path fixes recipient, channel, facts, template/copy, timing, suppression conditions, consent, idempotency, audit, and opt-out handling.
- Email, SMS, and portal are current canonical text channels; phone is a staff/call-task channel by default; WhatsApp is out of scope until typed consent/provider/template/audit semantics exist.
- `Customer.preferred_contact` is a preference, not legal authorization. A send candidate must carry consent/opt-out/DNC/quiet-hours facts and prior delivery/suppression state.
- Failed delivery creates staff/manual retry work or a reviewed replacement send. Do not silently switch channels without policy and audit.
- Suppression is a valid final outcome and must record reason, policy version, source refs, and audit context.
- Review/rebooking/retention idempotency should key on customer/pet/location/service/purpose/source event/history anchor/policy window, not generated message text.

## Brand voice constraints

Use the functional brand voice already collected by customer messaging:

- Warm, pet-parent-friendly, operationally clear, truthful, and source-grounded.
- Plain language tied to approved facts: pet/customer names, service, date/time, approved policy, reviewed care/service evidence, and next action.
- Reassuring but not promissory: no unsupported availability, booking confirmation, refund timing, discount, policy exception, diagnosis, outcome guarantee, group-play eligibility, or payment claim.
- Non-coercive for review requests and rebooking prompts; no pressure, guilt, quid-pro-quo, or incentives unless an approved policy/gate exists.
- For recovery drafts, acknowledge concern empathetically and commit only to manager-reviewed next steps. Do not admit liability, diagnose, blame staff/customer/pet, promise refunds/credits, or hide concerning facts.
- For staff-facing summaries, be factual, compact, and risk-forward. Preserve unresolved flags and cite source refs/freshness.

Final customer-facing copy, localization, legal disclaimers, review-link text, and brand-specific templates remain unknown and configurable.

## Consent, opt-out, DNC, quiet-hours, and over-contact rules

Required policy facts for any CRM/customer-message candidate:

- Message purpose: operational, review request/reputation, rebooking reminder, winback, complaint recovery, package/membership, VIP/appreciation, or staff follow-up.
- Channel and destination ref: email/SMS/portal/phone-task with provider/account refs.
- Consent basis and status by channel/purpose; opt-out/unsubscribe/DNC status; manager/legal suppression; quiet-hours/location timezone policy; age of consent fact.
- Recent contact history by channel/purpose and over-contact counters/windows.
- Suppression conflicts: active complaint, unresolved incident/concern, payment/refund dispute, negative sentiment, cancellation/no-show restriction, care/medical/behavior review, legal/privacy hold, stale/missing facts.

Conservative assumptions until policy exists:

- DNC, opt-out, unsubscribe, legal/privacy suppression, or missing consent blocks marketing/review/rebooking/winback messages. Internal staff tasks may still be created.
- Quiet-hours policy must be location-timezone aware. If unknown, do not queue autonomous sends; create a draft/review item or schedule only after policy approval.
- Over-contact limits are unknown. Downstream cards should define configurable counters by customer, pet, channel, purpose, and time window; default unknown limit should suppress automation rather than risk spam.
- Transactional/operational messages may have different legal treatment from marketing, but that distinction is not approved here. Preserve purpose-specific consent policy instead of hard-coding a legal rule.

## Human approval gates

The following must stay gated unless later policy narrows them with explicit deterministic send/action rules:

- Marketing automation: any review request, winback, VIP appreciation, package/membership upsell, promotion, routine rebooking send, or retention campaign must have consent/DNC/quiet-hours/over-contact checks and a human-approved or deterministic template path.
- Complaint responses: no auto-send. Always create/route manager review with source refs, issue summary, proposed response draft, commitments, owner, due date, and suppression state.
- Discounts, credits, refunds, waivers, retention offers, loyalty credits, package adjustments, deposit exceptions, or forfeiture changes: manager/payment approval required; AI may only draft recommendation/review packets.
- Sensitive content: medical, vaccine, medication, allergy, behavior, incident, safety, legal/liability, eligibility, service denial/acceptance, payment dispute, cancellation/no-show, or policy-exception language requires staff/manager approval before customer-facing use.
- Booking/provider mutations: no booking confirmation, reservation modification, provider status write, calendar booking, package enrollment, payment action, or capacity/overbooking exception from CRM automation alone.
- Do-not-contact changes, duplicate merges, retention/legal holds, role/permission changes, and policy/template changes require authorized human/admin action and audit.

## Downstream input object recommendation

Downstream CRM cards should consume an input packet with at least:

```text
crm_retention_input:
  scope: location_id, customer_id, pet_ids[], timezone, policy_refs
  lifecycle_context: current_candidate_stages[], stage_evidence[], suppression_flags[]
  contact_policy: channels[], preferred_contact, consent_by_purpose, opt_outs, dnc, quiet_hours, over_contact_state
  reservation_history: recent_and_relevant_reservations[], cancellations[], no_shows[], waitlists[], checkout_refs[]
  service_history: boarding[], daycare_attendance, grooming_history, training_history, packages_memberships
  risk_context: incidents[], complaints[], manager_flags[], payment_disputes[], medical_behavior_care_flags[]
  sentiment_reputation: review_refs[], sentiment_evidence[], unresolved_negative_signals[]
  opportunities: rebooking_candidates[], review_request_candidate, vip_candidate, lapsed_candidate, package_or_offer_candidates[]
  approval_context: required_gates[], prior_approvals[], manager_task_refs[], audit_refs, source_gaps[]
```

Every field should be marked trusted/untrusted/missing/stale/conflicting/source-pending where relevant. Missing or conflicting facts route to suppression, staff review, manager review, or no action; they must not be silently filled by AI.

## Unknowns to preserve for downstream configuration

- Formal lifecycle thresholds: first-time versus repeat, lapsed windows by service, VIP criteria, no-show/cancellation thresholds, package/membership status semantics, and complaint re-entry criteria.
- Approved template catalog: review request copy, rebooking prompts, winback/VIP copy, complaint-recovery drafts, legal disclaimers, review links, and localization.
- Consent model: channel/purpose opt-in, unsubscribe handling, DNC scope, quiet hours, over-contact windows, transactional-versus-marketing policy, and retention/legal holds.
- Human authority matrix: which staff/lead/manager/admin roles can approve marketing sends, complaint responses, retention offers, discounts, refunds, DNC changes, or lifecycle overrides.
- Data integration mapping: which provider fields reliably represent no-show, cancellation reason, complaint, review sentiment, package balance, completed grooming anchor, recurrence plan, and customer contact permission.
- Automation rollout: which narrow deterministic sends, if any, are approved after policy, template, consent, suppression, idempotency, audit, and provider execution are implemented.

Conservative downstream rule: if a CRM/retention workflow lacks approved consent, suppression, source evidence, template, timing, or human approval for a customer-facing effect, it may produce an internal task, staff-facing summary, suppression reason, or review packet only.
