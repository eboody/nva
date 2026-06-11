# Complaint recovery workflow

Purpose: define the CRM/retention complaint-recovery workflow as a safe internal case-management and manager-review path. This workflow may summarize issues, create or route manager tasks, draft empathetic response copy for review, track resolution, and suppress promotional automation. It does not authorize autonomous customer sends, public-review replies, refunds, discounts, credits, waivers, retention offers, booking/provider mutations, policy exceptions, or legal/liability statements.

Status: draft workflow definition for downstream CRM/retention implementation. Location-specific policy, template copy, manager authority, refund/discount rules, public-review handling, and re-entry thresholds remain configurable or human-approved.

## Source anchors

Use this workflow with the constraints in:

- `docs/workflows/crm-retention-parts/inputs.md` - complaint recovery lifecycle state, suppression posture, brand voice, approval gates, and CRM input packet.
- `docs/workflows/customer-messaging-parts/inputs.md` - customer-message draft/review separation, channel constraints, source-grounding, and outbound chain of custody.
- `docs/security/pet-resort-security-audit-parts/inputs.md` - task/message/incident/payment lifecycle states, permission/audit gates, prompt minimization, tool permission strata, and suppression as first-class outcome.
- `docs/workflows/payments-pricing.md` and `docs/workflows/payments-pricing-parts/ai-boundaries.md` - refund, waiver, discount, deposit, and payment-dispute boundaries.
- `docs/workflows/staff-operations-parts/inputs.md` - staff/manager task surfaces, daily operations handoff, care/incident/customer-follow-up review posture.
- `docs/domain/petsuites/daycare/implications/04-incident-tracking.md` and adjacent PetSuites service docs - incident, care, behavior, Pawgress/customer-update, and service-specific review gates.
- Current Rust anchors: `workflow::{WorkflowEvent, WorkflowResult, RecommendedAction, RiskFlag, ReviewReason}`, `policy::{AutomationLevel, ReviewGate}`, `operations::{StaffTask, StaffTaskKind, StaffTaskStatus}`, `entities::{CustomerId, PetId, ReservationId, AuditSubject}`.

## Trigger conditions

Enter complaint recovery whenever a trusted or reviewable source indicates an unresolved customer concern that could make ordinary retention outreach inappropriate.

Primary triggers:

1. Direct inbound complaint.
   - Email, SMS, portal message, phone transcript/call note, front-desk note, or customer reply expressing dissatisfaction, concern, fear, anger, disputed service, missed expectation, staff conduct concern, care concern, safety concern, payment dispute, refund request, cancellation/no-show dispute, or request for a manager.
2. Negative public or private review signal.
   - Public review, survey, NPS/CSAT response, social mention, reputation platform item, or private feedback with negative sentiment or specific concern.
3. Incident-adjacent customer concern.
   - Customer replies to an incident notice, asks about injury/illness/behavior/medication/feeding/belongings, disputes an incident timeline, or staff marks an incident as needing customer follow-up.
4. Staff/manager flag.
   - Staff creates a complaint task, marks a checkout/follow-up as dissatisfied, flags a service miss, records a manager callback request, or marks a reservation/customer as unresolved.
5. Payment/service dispute.
   - Refund/credit/discount/waiver request, chargeback hint, duplicate/incorrect charge claim, deposit forfeiture dispute, no-show/cancellation dispute, package/membership credit dispute, or disputed service quality.
6. Automation-safety trigger.
   - Existing CRM, review-request, rebooking, VIP, winback, package, or promotion workflow detects negative sentiment, unresolved incident, open complaint, payment dispute, legal/privacy hold, or manager suppression flag.

Do not require perfect classification before suppressing promotional automation. If evidence plausibly indicates an unresolved concern, create or route review work and set a conservative suppression reason. The manager can later clear or narrow it.

## Intake data packet

Complaint recovery should consume a compact, source-referenced packet. Raw free text, unredacted provider payloads, full payment details, webhook signatures, and unnecessary medical/care detail should remain in boundary/evidence storage and be represented by redacted excerpts or evidence refs.

Minimum fields:

```text
complaint_recovery_input:
  scope:
    location_id
    customer_id
    pet_ids[]
    reservation_ids[]
    service_lines[]              # boarding, daycare, grooming, training, DaySpa, retail/package, payment-only, unknown
    timezone
    policy_refs[]
  trigger:
    trigger_type                 # inbound_complaint | negative_review | incident_concern | staff_flag | payment_dispute | automation_safety
    source_channel               # email | sms | portal | phone_task | public_review | survey | staff_note | provider | other
    source_event_refs[]
    received_at
    source_trust                 # trusted | untrusted | missing | stale | conflicting | source_pending
    urgency                      # routine | time_sensitive | urgent_manager_review | legal_or_safety_escalation
  issue_summary:
    customer_concern_summary
    requested_outcome_if_any
    sentiment_or_tone_evidence
    unresolved_questions[]
    disputed_facts[]
    sensitive_categories[]       # care | medical | behavior | incident | safety | payment | legal | staff_conduct | privacy | eligibility
  context:
    customer_context_ref
    pet_context_refs[]
    reservation_context_refs[]
    service_history_refs[]
    staff_task_refs[]
    incident_refs[]
    payment_or_refund_refs[]     # redacted/payment-safe only
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

All intake values that could affect manager decisions must be marked trusted, untrusted, missing, stale, conflicting, or source-pending. AI summaries are not operational truth by themselves; they must cite source refs and uncertainty.

## Classification and urgency

The workflow may classify the concern for routing and prioritization, but classification is advisory unless deterministic policy or a manager confirms it.

Suggested categories:

- Service dissatisfaction: grooming result, boarding/daycare/training experience, missed add-on, room/suite/accommodation expectation, check-in/out issue, staff interaction.
- Care or safety concern: injury, illness, medication, feeding, allergy, behavior, group-play, incident, missing belongings, escaped/lost item, pet stress/distress.
- Payment or policy dispute: refund, credit, discount, deposit, cancellation/no-show, package/membership balance, billing error, chargeback threat.
- Communication breakdown: no update, confusing instructions, unanswered message, failed delivery, perceived rude or dismissive response, unclear policy explanation.
- Public reputation item: public review or social post requiring manager/public-response review.
- Legal/privacy/high-risk: legal threat, regulatory complaint, data/privacy concern, staff misconduct allegation, severe injury/death claim, discrimination/accommodation claim, fraud/chargeback.

Routing urgency:

- `routine`: dissatisfaction or question with no safety/payment/legal time pressure.
- `time_sensitive`: customer expects callback soon, active reservation affected, public review is visible, or follow-up commitment due within one business day.
- `urgent_manager_review`: safety/care/incident/payment escalation, angry customer in active conversation, live reservation at risk, or review response deadline.
- `legal_or_safety_escalation`: legal threat, severe harm, privacy/security issue, chargeback/fraud, staff misconduct, or any claim that could require owner/admin/legal review.

If urgency is uncertain, route to manager review and suppress promotional automation while unresolved.

## Manager task payload

Complaint recovery must create or route a manager task unless an equivalent open manager task already exists for the same customer/pet/reservation/issue idempotency key.

Task kind: `ComplaintRecoveryReview` or the nearest available manager-review task kind until a typed enum exists.

Recommended payload:

```text
manager_task:
  title: "Complaint recovery review: {customer/pet/service/date or concise issue label}"
  priority: routine | high | urgent | legal_or_safety
  owner_role: manager | owner_admin | payment_reconciliation | legal_compliance | service_lead
  owner_user_id: optional explicit assignee
  due_at: policy-derived timestamp
  subjects:
    customer_id
    pet_ids[]
    reservation_ids[]
    incident_ids[]
    payment_ids[]
    message_thread_refs[]
  source_refs[]
  issue_summary:
    concise_customer_concern
    relevant_reservation_pet_service_context
    timeline
    disputed_or_missing_facts[]
    sensitive_categories[]
    requested_outcome_if_any
  recommended_next_steps[]:
    verify_source_facts
    call_or_reply_to_customer
    inspect_reservation/care/payment/incident records
    assign service-lead follow-up
    request payment/refund approval
    request legal/privacy review
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

Idempotency should key on location, customer, issue/source refs, reservation/pet/service context, and active resolution state. Do not create duplicate manager tasks on retries; append new source refs or comments to the active task when policy permits.

## Empathetic response draft

The workflow may generate an empathetic response draft only for manager review. It must never auto-send, auto-queue for send, post publicly, or imply that the customer has already been contacted.

Automation level: `DraftOnly` plus `CustomerMessageApproval` and usually `ManagerApproval`. If the draft includes payment/refund/discount/waiver/credit/deposit language, add `RefundOrDepositException`. If it includes care/medical/behavior/incident/safety facts, add the appropriate manager/medical/behavior/incident review gate. Legal/privacy/staff-misconduct issues require owner/admin/legal/compliance routing before customer-facing text.

Allowed draft posture:

- Warmly acknowledge the concern and the customer's trust in the pet resort.
- Thank the customer for raising the issue.
- Reference only verified, non-sensitive facts: pet name, service type, date/stay, and that a manager/team member will review or follow up.
- State a manager-reviewed next step only if that next step is already approved or safe: e.g., "I’m going to have our manager review this and follow up with you.".
- Ask one concise clarifying question only if needed and safe.
- Preserve source uncertainty internally rather than presenting speculation to the customer.
- Keep the tone calm, accountable, and non-defensive.

Prohibited draft claims/offers unless an authorized human supplies exact approved wording:

- No admission of liability, negligence, wrongdoing, fault, legal responsibility, policy violation, or staff blame.
- No diagnosis, medical conclusion, behavior/temperament conclusion, safety clearance, or causation claim.
- No promise of refund, credit, discount, free service, package adjustment, waived fee, deposit return, or payment timeline.
- No promise of outcome, investigation result, disciplinary action, future eligibility, special accommodation, space availability, booking confirmation, or exception.
- No public-review response, review-site post, social reply, or reputation-management action without manager/public-response approval.
- No request to remove/change a review, quid-pro-quo, incentive for positive review, pressure, guilt, or defensive argument.
- No hidden or minimized concerning facts; sensitive facts should be routed to review instead of softened into misleading copy.
- No statement that a manager has reviewed, approved, called, refunded, credited, investigated, or resolved anything unless that approved event exists.
- No legal, privacy, insurance, chargeback, fraud, or regulatory language without legal/compliance/owner approval.

Draft output shape:

```text
complaint_response_draft:
  draft_id
  channel_candidate: email | sms | portal | phone_script | public_response_draft
  automation_level: DraftOnly
  send_allowed: false
  required_review_gates[]
  source_facts_used[]
  omitted_sensitive_or_unverified_facts[]
  body
  reviewer_notes[]
  prohibited_claims_checked[]
  approval_expiration_or_revalidation_rule
```

## Suppression while unresolved

Unresolved complaint recovery must suppress promotional and reputation lifecycle automation for the relevant customer/pet/reservation/location scope.

Suppress at least:

- Review requests and reputation prompts.
- Routine rebooking reminders and lapsed/winback campaigns.
- VIP/appreciation, loyalty, package/membership upsell, cross-sell, and promotional retention campaigns.
- Automated public-review solicitation or response flows.
- Non-essential marketing sends and customer nudges.

Do not suppress operationally necessary internal work. Staff/manager tasks, manager-reviewed customer response drafts, legal/compliance follow-up, payment reconciliation tasks, incident investigation tasks, care follow-up tasks, and required operational/customer-service messages may still proceed through their own approval gates.

Suppression record:

```text
complaint_suppression:
  scope: customer_id, pet_ids[], reservation_ids[], location_id
  status: active | partially_cleared | cleared | expired_by_policy | superseded | reopened
  reasons[]
  source_refs[]
  affected_purposes[]
  started_at
  owner
  due_at_or_review_by
  cleared_by_actor: optional
  cleared_at: optional
  clearance_reason: optional
  audit_refs[]
```

Suppression must be visible to downstream CRM workflows as a hard stop, not a soft ranking signal. If the scope is ambiguous, suppress the broader safe scope temporarily and ask the manager to narrow it.

## Resolution tracking

Complaint recovery is a case lifecycle, not a one-off draft. Track status, owner, due date, commitments, evidence, and clearance explicitly.

Recommended statuses:

1. `open`: concern detected and manager task created/routed.
2. `investigating`: owner is reviewing records, staff notes, payment/provider data, incident evidence, or service context.
3. `waiting_on_customer`: manager-approved response or staff call requested clarification/customer input.
4. `waiting_on_internal_review`: service lead, payment, incident, legal/privacy, owner/admin, or staff follow-up is pending.
5. `approved_response_pending_send`: exact response text is approved but not yet sent or logged.
6. `commitment_pending`: customer response sent and an internal commitment remains open, such as callback, refund review, service correction, document follow-up, or staff/service-lead action.
7. `resolved_pending_clearance`: owner believes issue is resolved, but suppression has not yet been cleared by an authorized actor or required waiting window.
8. `cleared`: authorized actor cleared suppression and documented why normal lifecycle may resume.
9. `closed_no_action`: manager closed as duplicate/no action with reason; suppression clearance still requires explicit policy if ordinary lifecycle resumes.
10. `reopened`: new evidence or customer reply reopens the concern and reinstates suppression.

Commitment fields:

```text
follow_up_commitment:
  commitment_id
  description
  owner_role
  owner_user_id
  due_at
  status: pending | done | cancelled | superseded | blocked
  customer_visible: true | false
  source_or_approval_ref
  completion_evidence_ref
```

Every status transition should emit an audit event with actor, before/after status, reason, source refs, policy refs, and affected suppressions.

## Human approval gates

Required gates:

- Complaint response: `CustomerMessageApproval` and `ManagerApproval`. No auto-send.
- Refund, discount, waiver, credit, deposit exception, package adjustment, free service, retention offer, or billing correction: `RefundOrDepositException` plus manager/payment authority. AI may prepare a recommendation packet only.
- Public review or social response: manager/owner approval with exact final text and platform/channel scope.
- Incident, safety, medical, medication, allergy, behavior, group-play, illness/injury, or care-quality statements: manager review plus medical/behavior/incident review as applicable.
- Legal threat, privacy/security concern, staff misconduct allegation, discrimination/accommodation issue, severe harm/death, fraud, or chargeback: owner/admin/legal/compliance routing before external response.
- DNC/opt-out/contact-permission changes: authorized staff/admin action with audit; complaint recovery may recommend but not perform the change.
- Re-entry into normal promotional/review/rebooking lifecycle: authorized clearance or deterministic policy clearance based on documented resolution and no active conflicting signals.

AI/model output, sentiment score, customer urgency, or stale source text never clears an approval gate.

## Re-entry into normal lifecycle

Normal CRM/review/rebooking/marketing lifecycle may resume only after explicit clearance criteria pass.

Minimum re-entry criteria:

1. Complaint case status is `cleared`, or a manager-approved policy says this exact case type can re-enter after documented `closed_no_action`/`resolved_pending_clearance` state.
2. All customer-visible follow-up commitments are done, cancelled with manager-approved reason, or superseded by an approved new commitment.
3. No open incident, care/safety, payment/refund, legal/privacy, DNC, opt-out, failed-delivery, or over-contact suppression still blocks the same purpose/scope.
4. Manager or authorized owner/admin has cleared the complaint suppression with actor, timestamp, scope, reason, and policy refs.
5. Downstream workflow revalidates consent, DNC, quiet hours, over-contact counters, message purpose, template/approval state, and source freshness at the time of the future send/draft.
6. Review-request eligibility uses an additional cooldown/reconsideration window if policy defines one; absence of a policy should default to no autonomous review request.

Re-entry should not replay missed promotions or review requests by default. It should allow future lifecycle evaluation from the clearance time with ordinary idempotency and over-contact checks.

## Output contract

The workflow result should be structured and policy-valid before persistence:

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

Safe outcomes:

- Create or update an internal manager task.
- Create a manager-review-only response draft.
- Set or maintain suppression for marketing/review/rebooking/promotional automation.
- Record a case/resolution status and follow-up commitment.
- Return `needs_more_information` or `failed_safely` when source facts are missing, conflicting, unsafe, or untrusted.

Forbidden outcomes:

- Auto-send a complaint response or public-review reply.
- Queue customer send without exact human-approved payload and send action.
- Issue or promise a refund/discount/credit/waiver/retention offer.
- Clear suppression or re-enter normal lifecycle based only on AI confidence or sentiment.
- Mutate reservation, payment, incident, DNC, opt-out, provider, or public-review state without authorized policy/human approval.

Conservative downstream rule: unresolved concerns stay visible as manager-owned work and active suppression. Complaint recovery may assist with evidence summaries, routing, empathetic drafts, and resolution tracking, but only authorized humans or deterministic approved policy can contact the customer, approve offers/refunds, clear suppressions, or return the customer to promotional lifecycle automation.
