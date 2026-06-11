# Review request workflow

Purpose: define the conservative post-checkout review-request workflow for the CRM/retention agent. This is a workflow-design artifact, not approval for autonomous marketing sends, public-review management, complaint handling, customer-facing copy, incentives, provider writes, refunds, discounts, or policy exceptions.

Status: draft workflow definition. It builds on `docs/workflows/crm-retention-parts/inputs.md`, `docs/workflows/customer-messaging-parts/send-draft-approval-policy.md`, `docs/workflows/customer-messaging-parts/inputs.md`, staff-operations and incident-escalation inputs, and current workflow/domain constraints. Missing location policy, consent policy, review-link/template catalog, cadence thresholds, and over-contact windows remain configurable and human-approved.

## Operating principle

A review request is considered only after a stay, visit, appointment, class/session, or other service has checked out or completed cleanly. The safest default is suppression or staff/manager review, not a customer-facing prompt. The workflow may classify eligibility, record suppression reasons, prepare a review packet, or draft a message for approval. It must not send automatically unless a later deterministic policy approves the exact category, template, channel, recipient facts, consent gates, suppression gates, timing, idempotency, audit, and opt-out handling.

## Trigger and scope

Primary trigger candidates:

- `reservation.checked_out`, `service.completed`, `checkout.completed`, or equivalent trusted provider/domain event.
- Staff-confirmed completion of grooming, daycare package/session, boarding stay, training session/program milestone, or DaySpa service.
- Explicit manager/staff request to evaluate review-request eligibility for a completed service.

A trigger is not enough to send. The workflow must first build an eligibility packet, apply hard suppressions, evaluate sentiment/risk, verify contact policy, apply cadence/deduping, and decide one of:

- `eligible_for_review_request_draft`: all gates pass, but customer-facing copy is still draft/review-gated unless an approved deterministic send path exists.
- `auto_send_candidate`: allowed only after a future policy/template/channel gate explicitly authorizes review-request auto-sends for this location and channel.
- `suppressed`: no customer-facing review prompt; record reason and source refs.
- `manager_or_staff_review_required`: risk, ambiguity, mixed signals, or missing facts require human review before any outreach.

## Eligibility rules

All required eligibility conditions must be true:

1. Completion gate
   - The relevant reservation/service/stay is in a trusted terminal positive state such as `CheckedOut`, `Completed`, or policy-equivalent.
   - Completion timestamp, location, service line, customer, pet, and source refs are present.
   - The service is not active/in-care, pending checkout, cancelled, no-show, rejected, waitlisted, partially completed with unresolved follow-up, or based only on raw/unverified provider text.

2. Clean-service gate
   - No open incident, injury, illness, escape/lost-pet issue, bite/fight/aggression event, medication error, facility hazard, or safety concern is linked to the stay/service/customer/pet/time window.
   - No unresolved care, behavior, medical, vaccine/document, eligibility, checkout, late pickup, lost-item, staffing, or service-quality concern remains open.
   - No manager/admin hold, reputation hold, legal/privacy hold, DNC hold, or workflow review gate is active.

3. Complaint/payment gate
   - No active complaint, unresolved concern, negative public/private feedback, service recovery case, refund/credit/waiver request, chargeback, billing dispute, payment exception, or disputed cancellation/no-show record is open.
   - No staff note indicates the customer may be dissatisfied, waiting for follow-up, or sensitive to outreach.

4. Sentiment gate
   - Sentiment is positive or neutral and has no negative signals. Acceptable sources include staff checkout disposition, customer reply classification, approved service summary, resolved happy-path survey, or manager-cleared reputation state.
   - If sentiment is missing, stale, contradictory, mixed, low-confidence, imported only from raw provider notes, or contains negative keywords/signals, do not auto-send. Suppress or route to staff/manager review.
   - AI sentiment alone cannot clear a review request when any structured risk, complaint, incident, payment, or manager flag exists.

5. Contact-policy gate
   - Recipient identity, channel, destination ref, consent basis, purpose-specific opt-in/permission, opt-out/unsubscribe/DNC state, quiet-hours policy, timezone, and prior delivery suppression are present and satisfied.
   - `Customer.preferred_contact` is a preference only; it is not sufficient consent.
   - DNC, opt-out, unsubscribe, legal/privacy suppression, missing consent, failed delivery suppression, or channel mismatch blocks customer-facing review prompts.

6. Cadence and over-contact gate
   - No review request for the same customer/pet/location/service/history anchor has already been sent, queued, approved, or suppressed inside the active policy window.
   - Overall customer contact frequency remains below configured over-contact limits by customer, channel, purpose, service line, pet, and time window.
   - Recent sensitive operational sends, complaint recovery contact, incident follow-up, payment collection, or manager outreach should suppress or require staff review even if the numeric limit has not been reached.

7. Template/brand gate
   - A customer-facing review request uses only approved, brand-safe, non-coercive template constraints. Final copy, review-link text, localization, disclaimers, and channel-specific rendering remain configurable policy artifacts.
   - No incentive, discount, refund, loyalty credit, quid-pro-quo, pressure, guilt, urgency manipulation, review gating, review filtering, or request to post only positive feedback is allowed.

## Suppression and escalation rules

Apply suppressions before cadence or copy generation. A suppressed result is a valid terminal workflow output and should include source refs, reason codes, policy version, and next owner where applicable.

| Suppression or risk signal | Workflow outcome | Route/owner |
| --- | --- | --- |
| Service not checked out/completed, completion source missing, active/in-care stay | Suppress review request | No outreach; wait for trusted completion or create data-quality task. |
| Incident, injury, illness, escape/lost pet, bite/fight/aggression, medication error, facility hazard, safety/medical issue | Suppress; create/attach manager/incident review packet | Manager/incident lead; no review prompt until explicitly cleared under policy. |
| Unresolved customer concern, active complaint, negative sentiment, public review needing response, service recovery case | Suppress; route to complaint/reputation path | Manager/reputation owner. |
| Refund dispute, chargeback, billing dispute, waiver/credit request, deposit/payment conflict | Suppress; route to payment/manager review | Payment reconciliation and manager. |
| Manager flag, staff hold, legal/privacy hold, reputation hold, DNC hold | Suppress | Assigned human owner clears or preserves hold. |
| Missing/unknown consent, opt-out, unsubscribe, DNC, channel suppression, quiet-hours unknown, over-contact unknown/exceeded | Suppress automation; optionally create staff review/configuration task | Front desk/admin/ops depending on missing policy. |
| Mixed or ambiguous history: good checkout but recent complaint, repeated failed delivery, contradictory notes, stale service data, identity conflict | Manager_or_staff_review_required | Staff validates facts; manager decides if outreach is appropriate. |
| No approved review-request template/link/policy for the location/channel | Draft-only or suppress; no send | Product/ops/template owner. |

Risky cases must route to staff/manager rather than automated review prompts. The route should preserve exact evidence refs and avoid smoothing over negative facts.

## Timing, cadence, quiet hours, and deduping

Timing is policy-configurable by location, channel, and service line. Until approved, do not queue autonomous sends. Suggested conservative design constraints:

- Earliest consideration: after trusted checkout/service completion has been recorded and routine post-checkout reconciliation has had time to surface same-day concerns.
- Default candidate window: a configurable delay after completion, for example next permissible daytime window after checkout, not during quiet hours. Do not hard-code this example as policy.
- Expiry: if the review request is not sent/drafted within a configured freshness window after completion, suppress rather than sending a stale prompt.
- Quiet hours: evaluate in the location/customer timezone. If the calculated send time falls inside quiet hours or quiet-hours policy is unknown, hold for staff review or schedule only under approved policy.
- Cadence limits: one review-request attempt per service-history anchor within the policy window; broader caps by customer/location/channel/purpose also apply.
- Cross-service dedupe: if a customer has multiple pets/services checked out close together, combine only under an approved template/policy; otherwise choose one human-reviewed candidate or suppress duplicates.
- No retry-by-regeneration: delivery retries may resend only the exact approved payload under adapter retry policy. They must not generate new copy, switch channels, or bypass suppression.

Suggested idempotency keys:

- Eligibility evaluation: `review_request_eligibility:v1:{location_id}:{customer_id}:{pet_or_group_ref}:{service_line}:{completion_event_id}:{policy_version}`.
- Draft: `message_draft:v1:{customer_id}:{review_request}:{service_history_anchor}:{template_version}:{policy_version}`.
- Approved send, if enabled later: `approved_send:v1:{approved_draft_id}:{recipient_ref}:{channel}:{approval_id}`.
- Suppression: `review_request_suppression:v1:{location_id}:{customer_id}:{service_history_anchor}:{reason_code}:{policy_version}`.

## Data fields needed

Minimum input packet:

```text
review_request_input:
  scope:
    location_id
    timezone
    policy_refs[]
    review_request_policy_version
  subject_refs:
    customer_id
    pet_ids[]
    reservation_id_or_service_id
    service_line
    service_history_anchor
    completion_event_id
  completion_state:
    status
    checked_out_or_completed_at
    source_system
    source_refs[]
    source_trust_state
  contact_policy:
    candidate_recipient_ref
    channel
    destination_ref
    preferred_contact
    consent_by_channel_and_purpose
    opt_outs
    dnc_state
    quiet_hours_policy
    over_contact_state
    prior_delivery_suppression
  risk_context:
    incidents[]
    unresolved_concerns[]
    active_complaints[]
    refund_or_payment_disputes[]
    safety_medical_behavior_flags[]
    manager_flags[]
    legal_privacy_holds[]
  sentiment_reputation:
    sentiment_label
    sentiment_confidence
    sentiment_source_refs[]
    negative_signal_refs[]
    public_review_or_response_refs[]
    service_recovery_state
  cadence_history:
    prior_review_requests[]
    prior_contacts_by_channel_and_purpose[]
    last_marketing_or_review_contact_at
    over_contact_counters[]
    dedupe_keys[]
  template_context:
    approved_template_id
    approved_template_version
    review_link_ref
    allowed_variables[]
    copy_policy_refs[]
  audit_context:
    actor_or_workflow_id
    required_review_gates[]
    prior_approvals[]
    suppression_reason
    source_gaps[]
```

Every source-derived field should be marked `trusted`, `untrusted`, `missing`, `stale`, `conflicting`, or `source_pending`. Missing or conflicting eligibility facts must not be filled by the model.

## Output contract

Each run should emit a structured result with at least:

- `decision`: `eligible_for_review_request_draft`, `auto_send_candidate`, `suppressed`, or `manager_or_staff_review_required`.
- `reason_codes`: explicit eligibility, suppression, or escalation reason codes.
- `subject_refs`: customer, pet(s), location, service/reservation, completion event, and source refs.
- `contact_gate`: consent, opt-out/DNC, quiet-hours, channel, destination, and over-contact evaluation.
- `risk_gate`: incident/complaint/payment/safety/medical/manager-flag/sentiment evaluation.
- `cadence_gate`: prior review-request and over-contact evaluation plus idempotency key.
- `message_mode`: `no_message`, `draft_only`, `approval_required`, or `approved_auto_send_candidate`.
- `template_refs`: approved template/link refs when available; never free-form final copy if policy is unknown.
- `human_review_reason`: required whenever facts are missing, risky, ambiguous, sensitive, or policy-dependent.
- `audit`: policy version, source freshness, reviewer/approval refs if any, suppression refs, and immutable payload refs for approved sends.

## Copy constraints, not final copy

Until a local policy and template catalog exists, the workflow may only describe constraints or create internal draft placeholders. Any future review-request copy must:

- Be warm, concise, pet-parent-friendly, truthful, and source-grounded.
- Refer only to approved facts such as pet name, completed service, location, and completion timing when policy allows those variables.
- Ask for feedback in a neutral, optional way; never imply obligation.
- Avoid incentives, discounts, refunds, loyalty points, sweepstakes, quid-pro-quo, or preferential treatment unless separately approved by manager/legal/marketing policy.
- Avoid review gating or filtering language: no "if you had a great experience..." segmentation that suppresses negative reviewers from public links while steering them elsewhere.
- Avoid pressure, guilt, urgency, repeated asks, employee-rating coercion, or statements that staff/pets depend on a positive review.
- Avoid sensitive details: incidents, medical/vaccine/medication facts, behavior/aggression, payment/refund, complaint, or service-recovery language.
- Include opt-out/unsubscribe or contact-preference handling if the channel/purpose policy requires it.

## Human approval gates

Review-request outreach is marketing/reputation automation and stays gated unless explicitly narrowed by deterministic policy.

Required approval gates:

1. Marketing automation gate
   - Product/operations leadership approves whether review requests may auto-send for each location, channel, service line, template, review-link destination, trigger, timing window, consent basis, suppression set, idempotency scope, audit contract, and retry behavior.
   - Without this gate, the workflow may only draft, suppress, or route to staff review.

2. Template/link gate
   - Approved template id/version, allowed variables, review-link ref, localization, disclaimer/opt-out text, and brand/legal constraints must be fixed before customer-facing use.
   - Any copy change, review-link change, incentive language, segmentation change, or review-platform policy change requires renewed approval.

3. Suppression override gate
   - AI cannot override incidents, complaints, negative sentiment, payment disputes, DNC/opt-out, safety/medical issues, or manager flags.
   - A human may clear a hold only through an audited policy path that records actor, reason, source refs, and what outreach is allowed. Clearing an incident/complaint for operations does not automatically approve a review prompt.

4. Ambiguous/mixed-history gate
   - Staff or manager review is required when customer history is mixed, sentiment is uncertain, recent outreach was sensitive, or source systems disagree.
   - Manager/reputation owner reviews public-review response contexts and any service-recovery-adjacent outreach.

5. Execution gate
   - Even when a send is approved, execution requires immutable approved payload, recipient/channel/destination, quiet-hours schedule, idempotency key, provider response handling, retry/dead-letter policy, and audit record.

## Implementation test cases

Happy-path candidate should pass only when:

- Completed checkout/service event is trusted.
- No incidents, complaints, concerns, disputes, safety/medical/behavior flags, manager holds, DNC/opt-out, quiet-hours conflict, over-contact violation, or negative signals exist.
- Sentiment is positive/neutral or manager-cleared neutral.
- Approved template/channel/policy exists or output is draft-only for approval.

Suppression fixtures should cover at least:

- Active incident after checkout.
- Negative checkout note or complaint email.
- Refund/chargeback dispute.
- Missing consent or explicit opt-out/DNC.
- Quiet-hours policy missing or current send time blocked.
- Prior review request already sent for the same service-history anchor.
- Recent complaint recovery or payment collection contact.
- Mixed signals: positive staff note plus unresolved manager flag.
- Multiple services/pets completing close together.
- Delivery failure on preferred channel with no approved alternate-channel policy.

## Conservative implementation rule

If the workflow cannot prove clean completion, no negative/risky signals, valid contact permission, quiet-hours compliance, cadence compliance, approved template/policy, and required human approvals, it must not prompt the customer for a review. It should suppress with reason, create a staff/manager review packet, or produce a draft-only candidate for approval according to the highest-risk signal present.
