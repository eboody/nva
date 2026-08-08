# Pet Resort Pilot Mode and Approval Boundaries

Status: launch-readiness definition artifact. This document does not approve pilot launch, live customer use, auto-send messaging, payment collection, production deployment, provider writes, medical/vaccine/behavior thresholds, or incident/customer-message policy. It defines the safest pilot posture and the human approval gates that must remain in force.

Source inputs reviewed:

- `docs/launch/pet-resort-launch-readiness.md`
- `docs/roadmap/pet-resort-mvp-implementation.md`
- `docs/workflows/staff-operations.md`
- `docs/workflows/customer-messaging-agent.md`
- `docs/workflows/payments-pricing.md`
- `docs/workflows/incident-escalation-agent.md`
- `docs/security/pet-resort-security-audit.md`

Source input not yet available at authoring time:

- `docs/launch/pet-resort-launch-checklist.md`

## Default safest pilot posture

Default recommendation: start in `internal_only` mode.

The safest launch posture is internal-only practice with synthetic/demo data, local/dev or non-production infrastructure, no live customer recipients, no real reservations, no payment collection, no provider/PMS mutations, and all AI results treated as drafts/recommendations. Demo-customer mode may follow only after the launch checklist and smoke scripts pass. Limited real-customer or one-location live pilot use requires explicit human approval after documented readiness evidence.

Default disabled state until separately approved:

1. Live customer/member-facing sends.
2. Automated or AI-approved outbound messages.
3. Payment collection, checkout-link delivery, refunds, waivers, discounts, credits, forfeitures, manual payment-status corrections, and production payment-provider webhooks.
4. Production deployment and production provider/PMS mutations.
5. Autonomous booking confirmation, rejection, waitlist/capacity exception, special-care acceptance, check-in, checkout, capacity release, or provider status mutation.
6. Vaccine/document auto-acceptance, medical-document uncertainty thresholds, and eligibility effects without approved human review.
7. Incident owner notices, medium/high/emergency final classification, incident closure, behavior/eligibility flags, group-play reinstatement or restriction clearance without manager approval.
8. Role/permission, retention, AI tool-grant, message-template, and payment-policy changes not explicitly approved by owner/admin/security.
9. Raw sensitive data in prompts/logs/client bundles: payment instruments/secrets, API keys, webhook signatures, raw provider payloads, raw incident narratives, unredacted documents/OCR, broad staff notes, and internal-only manager rationale.

## Approval gates that remain mandatory

These gates are not satisfied by this document. Each gate requires an explicit human decision, recorded scope, owner, date, and audit/reference before the corresponding effect is enabled.

| Gate | Required before | Minimum approver |
| --- | --- | --- |
| Pilot launch | Any pilot mode beyond local/internal authoring and smoke-test preparation | Owner/manager/product lead |
| Live customer use | Any named real customer, real pet, real reservation, or operational use against live business records | Owner/manager plus location owner |
| Auto-send messaging | Any outbound message delivered without staff pressing an approved send action for the exact copy/recipient/channel | Owner/manager plus legal/compliance where sensitive categories apply |
| Payment collection | Any live checkout link, charge, provider webhook, refund, waiver, discount, credit, forfeiture, or payment-status correction | Owner/manager plus payment/admin owner |
| Production/provider mutation | Production deploy, PMS write, reservation status mutation, capacity hold/release, provider message/payment command, or customer-portal script behavior | Owner/admin plus engineering/integration owner |
| Medical/vaccine/behavior/incident threshold | Auto-acceptance, eligibility effects, incident severity finalization, behavior restrictions, group-play decisions, or owner incident notices | Manager and relevant document/medical/behavior/incident reviewer |

## Pilot mode taxonomy

### 1. Internal-only mode

Purpose: staff practice, product validation, and smoke testing without customer-facing or business-state side effects.

Allowed:

- Synthetic/demo customers, pets, reservations, documents, notes, incidents, messages, and payments.
- Staff-dashboard navigation and practice workflows.
- AI-generated summaries, extractions, triage recommendations, message drafts, and review packets clearly labeled as draft/demo.
- Local/dev smoke tests and fixture-based workflow validation.
- Internal tasks and audit events in demo/local stores.

Must stay disabled:

- Real customer recipients or real pet/customer records.
- Live customer messages, production outbox, SMS/email/portal sends, provider/PMS writes, and customer-portal scripts.
- Live payment provider calls, checkout links, payment webhooks, refunds, waivers, discounts, credits, or payment-status corrections.
- Production deployment and production secrets/tool grants.
- Any claim that a reservation, vaccine, incident, eligibility, payment, or care state is real-world approved.

Entry criteria:

- Demo dataset exists and is marked non-production.
- Local/dev app can run with safe secrets and no production send/payment/provider adapters enabled.
- Role-scoped staff users exist for practice.
- Audit events record draft/review/suppressed outcomes.
- AI output validators reject unsupported side effects.

Exit criteria to demo-customer mode:

- Launch checklist is available and all internal/demo must-haves are either passing or explicitly deferred as non-blocking.
- Smoke scripts pass for the internal happy path and at least the required safety cases: missing vaccine, no capacity, special-care pet, daily update, incident draft, cancellation, and review request.
- Rollback/fallback plan exists and staff can explain how to stop sends, pause agents, and continue manually.

Escalation triggers:

- Any real recipient, live provider, live payment, or production secret appears in internal-only data or logs.
- Any AI result is treated as approval rather than draft/recommendation.
- Any fixture outcome mutates a live record or creates a provider/outbox/payment side effect.

### 2. Demo-customer mode

Purpose: exercise the staff/customer-facing review experience using only owner, staff, and test recipients.

Allowed:

- Owner/staff/test-recipient accounts and test pet/reservation records.
- Explicitly labeled test messages, test drafts, test approval queues, and test payment placeholders.
- Staff approval workflow rehearsal for message drafts and review packets.
- Manual verification that recipients, consent, quiet-hours, suppression, and audit fields are represented.

Must stay disabled:

- Real reservations, live payment collection, production provider mutations, and real customer commitments.
- Any message lacking a prominent test/demo label in subject/body or review UI.
- Automated delivery without a human approving the exact test payload, even to test recipients, unless the exact deterministic test template is approved for demo mode only.
- Payment links that can collect real money.

Entry criteria:

- Internal-only exit criteria pass.
- Test recipients are named in writing and controlled by owner/staff.
- Every outbound test path has a visible `TEST / DEMO - no real reservation or payment` marker.
- Outbox/provider settings route only to test recipients or sandbox adapters.
- Staff know how to confirm that no real customer/pet/reservation is selected.

Exit criteria to limited real-customer mode:

- All demo sends land only with approved test recipients and preserve audit/outbox records.
- Staff can distinguish draft, approved test send, suppressed, failed, and manual-call-task states.
- Demo payment flows prove manual fallback or sandbox-only behavior without money movement.
- Manager confirms staff training for approval queues, sensitive content, rollback, and escalation.

Escalation triggers:

- A demo message reaches a non-test recipient.
- A demo run omits test labeling or implies a real booking/payment/service promise.
- Sandbox/test payment behavior produces or risks real money movement.
- Staff cannot trace who approved a demo send or which exact payload was approved.

### 3. Limited real-customer mode

Purpose: run a tightly constrained live pilot with named, consented customers while preserving manual supervision.

Allowed only after explicit live-customer approval:

- Named customers and pets who have consented to the pilot scope.
- One approved location.
- A small approved date window and capacity cap.
- Manager-supervised staff review queues.
- Draft-only AI outputs for customer messages, booking triage, vaccine/document extraction, incidents, payments, and daily updates.
- Manual staff approvals before any customer-visible send or business-state effect.
- Manual payment/invoice handling outside the app if needed.

Must stay disabled unless separately approved:

- Auto-send messaging, including low-risk reminders, unless the exact template/category/channel/fact set is approved for deterministic delivery.
- Automated payment collection, live checkout creation/delivery, refunds, waivers, discounts, credits, forfeitures, and provider payment webhooks.
- Automated booking confirmation/rejection, waitlist/capacity exceptions, provider reservation writes, and capacity release.
- Vaccine auto-acceptance or eligibility effects without human document review.
- Incident owner sends, serious severity finalization, closure, restriction clearance, and behavior/group-play eligibility changes without manager approval.
- Use of non-consented customers as recipients, examples, or training/evaluation data beyond redacted aggregate operational metrics.

Entry criteria:

- Owner/manager approves the specific customer list, pet list, location, date range, services, staff owners, support coverage, and rollback plan.
- Customers receive and acknowledge the pilot terms, including manual staff supervision and how to contact the resort outside the app.
- Staff can run manual fallback for booking, care, messaging, incident, and payment operations.
- Smoke tests pass in demo mode, including forbidden-side-effect checks.
- Security/audit minimums are active: role-gated access, append-only audit events, redacted logs, safe prompt packets, and approval history.

Exit criteria to one-location pilot or wider live scope:

- No wrong-recipient sends, unauthorized provider/payment mutations, unresolved audit gaps, or severe safety/privacy incidents during the limited period.
- All customer messages were either manual/staff-approved sends or correctly suppressed.
- Every payment-sensitive case used manual fallback or approved human handling with audit evidence.
- Managers can review all approvals, edits, suppressed drafts, incidents, and fallback events.
- Staff complete post-pilot review and document defects, near misses, customer feedback, and training gaps.

Escalation triggers:

- Wrong recipient, unapproved send, duplicate send, privacy/PII leak, raw sensitive data in prompt/log, or opt-out/quiet-hours bypass.
- Any money movement or payment-status/customer-payment claim outside approved manual handling.
- Incident, medical, vaccine, behavior, eligibility, or safety ambiguity that blocks safe service.
- Provider/PMS state diverges from staff-visible state or cannot be reconciled.
- Customer complaint, legal/privacy concern, or staff uncertainty about authority.

### 4. One-location pilot mode

Purpose: constrain a live operational pilot to one location with bounded roles, services, dates, capacity, and support.

Allowed only after limited-real-customer evidence or an explicit human decision:

- One named location.
- Approved staff roles and named managers for approval queues.
- Approved service offerings only, such as a subset of boarding/daycare/grooming/training/add-ons chosen by the business.
- Approved pilot date range, blackout dates, capacity cap, and support hours.
- Approved live-customer cohort rules and manual fallback procedures.

Boundaries to define before entry:

| Boundary | Required decision |
| --- | --- |
| Location | Exact location id/name and timezone. |
| Staff roles | Which staff, leads, managers, and owner/admins may view, draft, approve, send, export, or administer. |
| Services | Which service lines and add-ons are in scope; all others suppressed or routed to manual handling. |
| Dates | Start/end dates, daily operating hours, quiet-hours rules, blackout/holiday handling, and stop-the-line owner. |
| Capacity | Max pilot reservations/pets/messages/day, overbooking prohibition, waitlist handling, and manual override owner. |
| Support | Named support owner, engineering/integration contact, manager escalation, after-hours fallback, and customer manual contact path. |
| Evidence | Required audit events, approval records, smoke-test results, incident logs, and post-pilot review notes. |

Must stay disabled unless separately approved:

- Multi-location operation.
- Complex capacity optimization or auto-release.
- Unsupported service lines and policy exceptions.
- Automated sends/payments/provider writes beyond a separately approved deterministic path.
- Any AI authority to approve its own draft or widen its own tool/context access.

Entry criteria:

- Limited real-customer criteria are met or explicitly waived by owner/manager with rationale.
- Location-specific role matrix, message policy, payment policy, incident escalation owner, and manual fallback are approved.
- Launch checklist, smoke scripts, rollback/fallback, and go/no-go review all support the one-location scope.
- Staff training has been completed for every role participating in the pilot.

Exit criteria to broader rollout:

- Stability over the full approved date range with no unresolved high-severity issues.
- All hard-stop events were escalated and resolved or accepted by management.
- Manual fallback was tested or used successfully.
- Post-pilot defect list is triaged with owners and no open critical blockers.
- Owner/manager approves a new widened scope document rather than relying on this one.

Escalation triggers:

- Capacity, staffing, service offering, or date scope exceeds the approved boundary.
- Support coverage is unavailable.
- Manager approval queue backs up enough to risk delayed safety/payment/customer decisions.
- Any incident or customer complaint suggests the pilot scope is too wide.

### 5. Manual approval for outbound messages

This is not a separate launch phase; it is a mandatory control across every mode.

Default rule: all AI-generated customer-facing text is draft-only unless an exact template, category, channel, trigger, fact set, consent/quiet-hours/suppression policy, idempotency/audit path, and workflow are separately approved for deterministic send.

Requirements for manual sends:

1. Human reviewer approves the exact recipient, destination, channel, subject/body, facts, template/category, and timing.
2. Every customer-visible claim has source evidence and has passed forbidden-claim checks.
3. Sensitive topics route to the correct reviewer: manager, medical/document reviewer, behavior reviewer, refund/deposit/payment reviewer, legal/privacy/compliance, or engineering/integration owner.
4. The approved payload is immutable for delivery. Edits, regeneration, channel switches, recipient changes, or fact changes require a new approval.
5. Audit records capture draft creation, edits, approval/rejection, send/suppression, provider response, delivery failure, and customer replies where applicable.

Must stay disabled:

- Model-confidence-based sends.
- Sends from unreviewed OCR, raw documents, raw provider data, internal notes, or stale/conflicting facts.
- Silent channel fallback after bounce, opt-out, quiet-hours conflict, missing destination, provider failure, or unsafe content.
- Autonomous incident, payment, vaccine/eligibility, booking rejection/confirmation, legal/privacy, complaint, or review-request sends.

### 6. Manual payment fallback

This is also a mandatory control across every mode.

Default rule: use manual invoice/payment handling outside the app until payment collection is explicitly approved. The app may show semantic payment tasks, draft reminders, and reconciliation review packets, but must not collect money or mutate payment truth.

Allowed before payment-collection approval:

- Internal payment readiness tasks and manager review packets.
- Staff-visible payment-sensitive blockers using safe semantic status only.
- Draft payment reminders for human review.
- Manual invoice/payment path outside the app, recorded by staff according to approved business process.
- Sandbox-only provider tests in internal/demo mode.

Must stay disabled:

- Live checkout/session/link creation or delivery.
- Live provider webhooks in production payment workflows.
- Automated retries, reminders that imply due dates/penalties, refunds, waivers, discounts, credits, forfeitures, write-offs, and manual status correction.
- Raw card data, CVV, bank data, provider secrets, webhook signatures, or raw provider payloads in the app, prompts, logs, or customer messages.
- Reservation confirmation/cancellation/capacity release solely because of a payment event.

Entry criteria for enabling automated collection later:

- Provider selected and approved.
- Hosted checkout/tokenized flow keeps raw card data out of the app.
- Signed webhook verification, idempotency, reconciliation, audit, and sandbox tests pass.
- Approved payment policy exists for deposits, due dates, cancellation/no-show/refundability, reminders, expiration, disputes, and manager exceptions.
- Approved customer-message template/path exists for payment links and reminders.

## Widening decision tree

Use the narrowest mode that satisfies the current learning goal.

```text
Need staff practice only?
  -> internal_only
Need recipient/channel/outbox rehearsal?
  -> demo_customers after internal smoke passes
Need real customer operational feedback?
  -> limited_real_customers after explicit live-customer approval
Need routine live operation at one site?
  -> one_location_pilot after limited evidence + go/no-go review
Need broader rollout, automation, auto-send, or payment collection?
  -> new approval scope; this document is insufficient
```

Do not widen mode automatically. Widening requires a documented decision that names: prior mode evidence, target mode, location, customers/services/dates/capacity, disabled features, approval owners, rollback owner, and review date.

## Stop-the-line and rollback triggers

Immediately pause the pilot or fall back to the narrower prior mode when any of these occur:

- Unauthorized live customer send, wrong recipient, duplicate send, opt-out/quiet-hours bypass, missing approval, or provider/outbox inconsistency.
- Unauthorized money movement, payment link delivery, refund/waiver/discount/credit/forfeit/write-off, or unreconciled provider payment state.
- Production/provider mutation outside approved scope.
- Raw sensitive data in prompts, logs, broad dashboards, exports, client bundles, or customer-visible fields.
- Missing append-only audit for a sensitive action, approval, provider event, AI output, or side effect.
- Incident, medical, medication, feeding, vaccine, behavior, eligibility, or safety issue that cannot be resolved within the approved review path.
- Manager approval queue unavailable, staff role confusion, or support coverage gap.
- Customer complaint, legal/privacy/compliance concern, security incident, or staff report that the pilot controls are unclear.

Rollback posture:

1. Disable or pause affected agents, queues, outbox, payment, provider-write, and automation flags.
2. Preserve records and audit evidence; do not delete or rewrite history.
3. Continue operations through manual staff process.
4. Notify internal pilot owners with safe summaries and exact affected scope.
5. Resume only after owner/manager review confirms root cause, remediation, smoke verification, and narrowed scope.

## Mode-by-mode minimum control matrix

| Control | Internal-only | Demo customers | Limited real customers | One-location pilot |
| --- | --- | --- | --- | --- |
| Data | Synthetic/demo only | Owner/staff/test recipients only | Named/consented customers only | Approved one-location cohort |
| Infrastructure | Local/dev or isolated non-prod | Non-prod/test adapters preferred | Approved environment with live use explicitly scoped | Approved one-location environment |
| Customer messages | No real sends | Test-labeled sends only after human approval | Draft/manual approval only | Draft/manual approval unless exact deterministic path approved |
| Payments | None; sandbox only | Manual/sandbox only | Manual fallback outside app | Manual fallback unless collection approved |
| Provider/PMS writes | Disabled | Disabled unless test/sandbox | Disabled unless explicit approved live path | Disabled except approved scoped paths |
| AI outputs | Draft/recommendation only | Draft/test only | Draft/recommendation with manager supervision | Draft/recommendation with role-scoped policy |
| Incidents/safety | Demo only | Demo/test only | Manager-reviewed; owner notices draft-only | Manager-reviewed; stop-line on serious gaps |
| Audit | Required for practice | Required for tests | Required for all sensitive actions | Required for all actions/effects |
| Exit review | Smoke evidence | Demo evidence | Post-pilot review | Go/no-go for broadened rollout |

## Final recommendation

Proceed only with `internal_only` as the default launch-readiness posture until the launch checklist, smoke scripts, rollback/fallback plan, and MVP smoke-test status are available and reviewed.

After internal-only evidence exists, the next safest step is `demo_customers` with owner/staff/test recipients and explicit test labels. Do not move to `limited_real_customers` or `one_location_pilot` without a recorded human go/no-go decision that preserves the gates for pilot launch, live customers, auto-send messaging, payment collection, production/provider mutations, and medical/vaccine/behavior/incident thresholds.
