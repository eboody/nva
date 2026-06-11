# Payments and pricing workflow

Status: canonical workflow artifact for the current specification board. This document is a workflow/specification artifact only. It does not implement product behavior, configure a live payment provider, approve production webhooks, authorize live charges, or approve any real refund, discount, fee waiver, cancellation penalty, price change, or provider selection.

Unresolved business values are intentionally preserved as open questions. Do not infer service prices, deposit amounts, cancellation windows, refundability terms, holiday surcharge values, provider choice, reminder cadence, or customer-facing legal/policy copy from this document.

## Scope

This workflow covers pricing, deposits, payment requests, payment status reconciliation, failed-payment handling, refund/discount/waiver gates, and AI/staff boundaries for pet-resort services across boarding, daycare, grooming, training, retail/package, and add-on workflows.

It defines:

- Which records are source-of-truth for prices, deposits, payment status, policy decisions, approvals, and audit history.
- How approved pricing and deposit policy should be represented without hard-coded resort folklore.
- How payment provider integration should be approached without exposing raw card data or treating provider events as reservation truth.
- What internal payment states and failure/reconciliation behaviors are needed.
- What AI may summarize, draft, flag, and escalate.
- What AI must never decide or execute.
- Which staff role should review each class of payment/pricing situation.

Out of scope unless separately approved:

- Selecting a production payment provider.
- Creating live checkout links or registering production webhooks.
- Executing live charges, retries, voids, refunds, credits, waivers, discounts, or forfeitures.
- Changing customer-facing prices, tax/fee treatment, deposit policy, cancellation policy, holiday/peak policy, or legal copy.
- Mutating live reservation status as a side effect of payment state.

## Source-of-truth records

Payment and pricing decisions must be sourced from typed, auditable records. Free-text notes, customer claims, screenshots, emails, raw provider JSON, and AI summaries are not payment truth by themselves.

Authoritative or trusted after validation:

- Approved location/provider policy records for service prices, packages, add-ons, deposits, taxes/fees, cancellation windows, refundability, due timing, reminder rules, holiday/peak overrides, and approval roles.
- Reservation, service, pet, customer/account, location, invoice, estimate, package, and add-on records from the PMS/reservation ledger or approved internal stores.
- Immutable reservation/service snapshots and policy versions attached to a quote, deposit decision, payment request, or stay.
- Payment/deposit projections returned through trusted repositories/adapters, using semantic amounts, currency, payment references, and reconciliation status.
- Verified payment-provider references and status events after signature verification and provider-specific mapping.
- Staff/manager approvals captured with actor, timestamp, scope, reason, before/after state, and linked payment/reservation/policy reference.
- Durable audit/workflow events for checkout creation, reminder approval/send, webhook receipt, provider status fetch, reconciliation result, refund request/approval/execution, waiver/discount approval, and ledger sync.

Not authoritative by themselves:

- Customer statements such as "I already paid" or "someone waived this".
- Staff free-text notes without an approved payment/policy/approval reference.
- Screenshots, emails, attachments, or chat messages before reconciliation.
- Unsigned, unverified, or unmapped webhook payloads.
- Raw provider JSON before mapping and verification.
- AI drafts, summaries, recommendations, or confidence scores.

When trusted sources conflict, the workflow records the conflict and routes to reconciliation or manager review. It must not choose the most favorable interpretation or silently overwrite one source with another.

## Pricing model

Pricing is location-scoped policy data, not a hard-coded constant. A price quote or balance calculation must reference the policy/source version used to produce it.

Supported pricing dimensions:

- Service line: boarding, daycare, grooming, training, retail/partner products, packages, and approved add-ons.
- Location and provider/PMS context.
- Pet count and species where the policy differentiates service handling.
- Boarding stay dates, nights, accommodation class, same-suite or separate-room handling, capacity/hold status, and add-ons.
- Daycare visit type, recurring attendance/package use, and service variant where approved.
- Grooming menu item, breed/coat estimate, style complexity, rebooking/no-show policy, and groomer/location constraints.
- Training enrollment, package/program, trainer availability, and outcome/progress workflow where relevant.
- Retail SKU/product, package sale line, inventory/POS policy, taxability, and discount/comp gates.
- Medication, special-care, behavioral, late-pickup, extended-checkout, premium bedding, photo/report, grooming-after-boarding, playtime, or other add-on/surcharge candidates only when approved policy defines them.
- Taxes, payment processing fees, convenience fees, rounding, disclosure language, and who absorbs/charges fees only when approved policy defines them.

Supported rate/package structures:

- Flat service price.
- Per-night/day/session/visit/item price.
- Per-pet price.
- Accommodation/menu/package-specific price.
- Tiered or bundled package price.
- Multi-pet rule.
- Add-on or surcharge line item.
- Staff-entered quote requiring approval evidence.
- Unknown/missing policy requiring review.

Rules:

- The workflow may calculate draft totals only from approved line items, taxes/fees, discounts, credits, package balances, deposits, and payments.
- Discounts, comps, credits, package overrides, manual price adjustments, fee waivers, and surcharge waivers require human approval before being shown as final or applied to a ledger.
- Provider string values such as rates, balances, and payment amounts are raw source data until parsed into typed money/currency values and reconciled.
- Missing prices, stale policy versions, mismatched currencies, absent tax/fee policy, or ambiguous package/add-on applicability must produce a review task rather than a guessed quote.

## Deposit/cancellation policy

Deposit requirements and cancellation consequences are approved location/provider policy, not AI inference.

A deposit decision should record:

- Service line, location, customer/account, pet(s), reservation/service dates, quote/reservation id, and policy version.
- Lead time, holiday/peak/local-event classification if available, accommodation/package/add-on context, customer history policy inputs, and high-value booking classification if approved.
- Amount model, due rule, refundability rule, partial-payment rule, reminder rule, expiration/hold rule, evaluated timestamp, and review gate if any input is missing.

Deposit amount model slots:

- Not required.
- Fixed amount.
- Percent of estimated reservation total.
- Per-pet amount.
- Per-night/day/session amount.
- Service/package-specific amount.
- Holiday/peak-specific amount.
- High-value booking amount based on an approved threshold.
- Manager-set amount with approval evidence.
- Unknown/missing policy: manager review required.

Deposit due-rule slots:

- Due at booking/request approval.
- Due before confirmation.
- Due by local deadline.
- Due before check-in/appointment start.
- Due at check-in only when the approved service policy allows it.
- Manager-deferred with approval evidence.

Cancellation/no-show/refundability outcomes must distinguish:

- Cancelled with sufficient notice.
- Late cancellation inside the approved notice window.
- No-show after the approved grace/observation rule.
- Customer/provider/staff facts disputed.
- Cancellation linked to pet health, safety, incident, care, weather, emergency, or other sensitive context.
- Provider correction or duplicate event.

Supported payment consequences are candidates until policy and approval resolve them:

- No payment consequence.
- Deposit refundable candidate.
- Deposit non-refundable/forfeiture candidate.
- Cancellation fee candidate.
- Rebooking deposit required candidate.
- Balance/credit/refund review required.
- Manager review required.

Rules:

- Refundability windows, notice windows, no-show grace periods, cancellation fee rules, forfeiture rules, partial-payment behavior, and hold-expiration rules must come from approved policy.
- A missing, failed, late, disputed, partial, or ambiguous required deposit may block automated confirmation/check-in/check-out readiness or create staff tasks, but automation may not confirm, cancel, reject, hold, release, or modify live reservations based on payment state.
- Refunds, forfeitures, cancellation-fee waivers, waived deposits, manual discounts, account credits, and exception explanations require manager/human approval and audit evidence before execution or customer-facing explanation.
- Customer-facing deposit, cancellation, no-show, refund, balance-due, and dispute messages remain drafts until approved unless a separate deterministic send path has been approved.

## Holiday/peak pricing

Holiday, peak, local-event, minimum-stay, and scarce-capacity behavior is policy-controlled and unresolved unless an approved location policy supplies it.

Holiday/peak policy may affect:

- Service availability and capacity holds.
- Deposit required/not-required decision.
- Deposit amount model.
- Deposit due date and reminder cadence.
- Minimum stay/session/package requirements.
- Cancellation notice windows.
- Refundability, forfeiture, and cancellation-fee rules.
- Late pickup, extended checkout, special-care, or add-on surcharge candidates.
- Escalation urgency for failed payments or unpaid deposits.

Conservative behavior when holiday/peak policy is missing:

- Create manager review for the quote/deposit/cancellation decision.
- Do not invent surcharge values, holiday dates, blackout periods, minimum-stay rules, refund windows, or deadlines.
- Do not auto-release reservation capacity, cancel a hold, promise availability, or send penalty/refund language.

## Payment provider integration path

Provider selection is unresolved and remains a human approval gate.

Recommended decision path:

1. If the resort operates its ledger and reservation workflow in Gingr and can use Gingr Payments with the needed checkout, refund, webhook, sandbox, and reporting controls, prefer a Gingr-native path because the operational ledger, reservation, customer, pet, invoice, portal, terminal, and reconciliation records stay in the same pet-care system of record.
2. If Gingr Payments cannot provide the needed hosted checkout-link/session, webhook, refund, sandbox, or reporting surfaces, evaluate Stripe Checkout as the primary external hosted-checkout candidate.
3. If the business already standardizes on Square POS/Terminal, evaluate Square as an in-person-first provider, but validate reservation metadata, hosted checkout, refund, dispute, and PMS reconciliation fit.
4. Do not run multiple processors for the same reservation workflow unless the reconciliation model explicitly supports provider-specific references and duplicate-risk controls.

Provider approval criteria:

- Hosted checkout/session/link can be created for a specific reservation deposit or balance without exposing card data to the app.
- Stable metadata can carry internal payment request id, reservation reference, customer/account reference, location id, payment subject, service category, and policy/quote version without sensitive pet-care details.
- Signed webhook coverage exists for success, failure, refund/partial refund, dispute/chargeback, cancellation/expiration, and async processing.
- Sandbox/test mode and local webhook testing are available before production registration.
- Provider payments can be reconciled to the PMS ledger and reservation invoice without manual ambiguity.
- Refunds can be requested in-system but executed only after human approval.
- PCI scope remains limited through hosted checkout/tokenized provider surfaces; the app must not handle raw card data.
- In-person terminal needs are either supported by the same path or deliberately handled as a separate reconciled payment source.

Checkout/session ownership:

- The app owns the semantic `PaymentRequest`/`PaymentIntent`, audit record, policy reference, approved amount/currency, idempotency key, and reconciliation state.
- The provider owns card collection, provider payment object lifecycle, checkout URL/session, and provider-specific payment/refund/dispute events.
- The PMS/reservation ledger owns reservation status, invoice/balance application, booking confirmation, capacity hold, and customer-visible booking state.

Allowed checkout creation flow after provider approval:

1. Source reservation/customer/pricing facts from trusted records.
2. Create an internal payment request before provider contact.
3. Generate a deterministic idempotency key from the internal payment request id plus attempt/lifecycle version.
4. Call the selected provider to create hosted checkout/session/link.
5. Store only provider ids, checkout URL, expiration, amount, currency, status, and redacted request/response metadata.
6. Send/display the link only through an approved staff-reviewed or deterministic messaging path.

Forbidden metadata/log fields:

- Pet medical details, vaccination details, medications, feeding notes, behavior notes, incident details, staff notes, private owner notes, raw card/bank data, CVV, webhook signing secrets, API keys, unapproved discounts/waivers, or free-form AI rationale.

Production payment/webhook side effects require separate approval. This document does not approve live provider configuration.

## Payment status state machine

Recommended provider-backed payment request statuses:

- `draft`: internal request is being assembled; no provider object exists.
- `checkout_created`: provider checkout/session/link exists but has not been sent or displayed.
- `checkout_sent`: customer has been given the hosted link through an approved path.
- `pending`: provider reports async or processing state.
- `succeeded`: provider confirms successful payment/capture for expected amount/currency.
- `failed`: provider reports decline/failure and no successful payment has superseded it.
- `canceled`: staff/system canceled the request before successful payment.
- `expired`: provider checkout/session expired.
- `partially_refunded`: provider confirms a partial refund.
- `refunded`: provider confirms a full refund.
- `disputed`: provider reports chargeback/dispute.
- `requires_review`: mismatch, duplicate risk, ambiguous provider state, unknown event, source conflict, policy exception, or approval gap requires review.

Transition sketch:

```text
draft
  -> checkout_created
  -> checkout_sent
  -> pending
  -> succeeded
  -> partially_refunded
  -> refunded

checkout_sent -> expired
checkout_sent -> canceled
checkout_sent -> failed
pending       -> succeeded | failed | canceled | requires_review
succeeded     -> disputed | partially_refunded | refunded | requires_review
failed        -> checkout_created  (new attempt, new attempt number, only if policy/staff permit)
expired       -> checkout_created  (new attempt, only if policy/staff permit)
any           -> requires_review   (mismatch, duplicate, unknown, manual exception)
```

Webhook/idempotency rules:

- Verify provider signatures over raw request bodies before parsing into business events.
- Store every verified inbound event durably before business processing.
- Use provider event id plus provider account id as the primary dedupe key; if unavailable, use a documented provider payment id/event type/updated timestamp fallback.
- Processing must be idempotent: repeated events cannot double-apply payments, double-send reminders, double-release holds, or double-create refund requests.
- Ignore stale regressions unless a provider source-of-truth fetch confirms them.
- Unknown verified events are recorded and routed to permanent ignore or staff review; they must not silently drive automation.

Reconciliation rules:

- Provider is source of truth for provider object lifecycle.
- PMS/reservation ledger is source of truth for reservation status and invoice/balance application.
- Internal workflow store is source of truth for idempotency, review gates, reminders, staff notifications, and provider-to-reservation mapping.
- A provider `succeeded` event permits recording/reconciliation; it does not by itself confirm or modify a reservation.
- A PMS/Gingr balance marked paid without matching provider evidence is reconciled as external/manual payment, not backfilled as provider success without evidence.

## Failed-payment handling

Failed, late, disputed, partial, expired, or ambiguous payments create review/reconciliation work. They do not authorize autonomous payment retries, cancellation, hold release, penalty language, or reservation mutation.

Recommended behavior:

1. On verified failure/decline, mark the attempt `failed` and keep the payment request open only if approved policy permits another attempt.
2. Record provider failure category as a redacted enum such as `card_declined`, `insufficient_funds`, `requires_customer_action`, `provider_unavailable`, or `unknown`; do not expose raw processor text broadly.
3. Create a new checkout attempt with a new idempotency key only when policy and staff action permit retry.
4. Draft reminders or staff tasks according to approved messaging policy. If deterministic sending is not approved, customer reminders remain staff-reviewed drafts.
5. Notify staff/manager when failure threatens confirmation readiness, hold expiration, holiday/peak capacity, repeated decline, amount mismatch, dispute handling, or unclear payment truth.
6. Route processor outage/tool error to reconciliation/retry task without silently changing reservation or payment state.

Reservation-hold impact:

- Required but unpaid deposits may mark confirmation/check-in/check-out readiness as blocked and create collection/review tasks.
- Payment failure does not automatically cancel a reservation unless approved policy explicitly says so and authorized staff/provider action executes it.
- Internal hold states may include `deposit_pending`, `payment_failed`, `hold_expiring`, or `manager_review_required`; final release/cancellation requires policy plus human/provider action.
- Holiday/peak/high-value failures may escalate earlier to staff, but the workflow must not invent cancellation terms or deadlines.

## Refunds/discounts/fee waivers gates

Refunds, discounts, comps, credits, fee waivers, deposit waivers, cancellation-fee waivers, forfeitures, write-offs, goodwill exceptions, manual price changes, package overrides, and payment-processing-fee waivers require human approval before execution or customer commitment.

Refund request workflow:

1. Staff or workflow records refund request with reservation/payment reference, requested amount, reason, requester, supporting note, and policy basis.
2. System validates payment existence, amount/currency plausibility, refund/dispute state, and duplicate-risk state.
3. Internal refund status becomes `requested` or `requires_review`; no provider refund is executed automatically.
4. Manager/human approves, rejects, or requests more information.
5. Only after approval may a provider refund call be executed with idempotency key and approved reason.
6. Webhooks/reconciliation update refund status to `submitted`, `succeeded`, `failed`, `partially_refunded`, or `refunded`.
7. PMS/ledger updates happen through approved adapters and remain auditable.

Gate rules:

- Refund execution always requires human approval.
- Partial refunds, outside-window refunds, service-not-rendered adjustments, dispute-adjacent refunds, fee waivers, deposit waivers, cancellation-fee waivers, forfeitures, comps, discounts, credits, and goodwill exceptions require manager approval.
- Pricing changes and source-policy changes require human approval before use.
- Provider selection and production webhook/payment side effects require human approval before configuration.
- Customer-facing explanation of refund, denial, waiver, penalty, forfeiture, or exception remains draft-only until approved.

## AI allowed roles

The AI may act only as a source-grounded assistant, draft generator, and review-packet builder.

Allowed roles:

- Explain approved pricing, deposit, cancellation, refundability, and payment-timing policy using cited/current policy facts. If a value is missing, stale, location-specific, or ambiguous, state that and escalate.
- Draft payment, balance, deposit, cancellation-policy, refund-window, or failed-payment reminders for human review.
- Identify missing payment/deposit information, missing references, amount/currency mismatches, failed payments, overdue balances, missing refundability/deadline data, stale policy versions, and absent policy snapshots.
- Summarize payment status from trusted records: amount due/paid/refunded/waived, due date, deposit status, payment reference presence, provider reconciliation state, refundability window, and next required action.
- Produce internal staff/manager review packets with source facts, uncertainty, risks, candidate actions, and exact approval needed.
- Create or recommend internal tasks for deposit collection review, payment reconciliation, refund review, waiver review, forfeiture review, checkout balance review, webhook mapping review, or customer reminder review.
- Redact/minimize sensitive billing details before summarizing. Prefer semantic statuses and references over raw provider payloads.

## AI forbidden roles

The AI must not:

- Change prices, package rates, taxes, fees, deposit requirements, refund windows, cancellation rules, holiday/peak rules, minimum-stay rules, or policy snapshots.
- Create discounts, comps, credits, fee waivers, deposit waivers, forfeitures, refunds, voids, write-offs, goodwill exceptions, or manual pricing overrides.
- Capture, retry, void, or refund a payment; store or expose card/payment secrets; or generate signed webhook/payment-provider details.
- Invent payment status, infer payment from a customer claim, or treat unverified notes/screenshots/emails/webhooks as payment truth.
- Promise availability, booking confirmation, check-in/check-out completion, cancellation approval, refund timing, discounts, waivers, or policy exceptions.
- Override cancellation policy, deposit policy, holiday/peak rules, manager/legal/care review gates, or payment-provider reconciliation outcomes.
- Send customer-facing payment-sensitive messages unless an approved deterministic send path has already fixed the facts, recipient, template, and send condition; otherwise it may only draft for review.
- Expose sensitive billing data beyond the recipient's role, including raw provider payloads, card data, tokens, API keys, webhook signatures, or unnecessary PII/care notes.
- Use model confidence as evidence that a payment, refund, discount, waiver, exception, or policy change is valid.

## Staff escalation matrix

| Situation | AI/system may do | Required escalation | Forbidden autonomous outcome |
| --- | --- | --- | --- |
| Routine source-backed price/deposit explanation | Explain approved current policy; cite policy/source id; draft customer copy | Staff review if customer-facing and no deterministic approved send path exists | Invent amounts, quote stale/location-unknown rates, promise availability |
| Missing price/deposit policy or stale policy version | Flag missing source and prepare review packet | Manager or policy owner | Guess rates, deadlines, windows, surcharge values, or legal copy |
| Missing deposit or unpaid balance with clear trusted status | Summarize amount/due date/status; create staff task/draft reminder | Front desk/staff review or approved collection workflow | Charge payment, confirm booking, check in/out, send unapproved message |
| Failed/declined payment or missing provider reference | Flag reconciliation need; summarize trusted evidence | Staff/payment reconciliation review | Retry payment, mark paid, blame customer/card, expose provider internals |
| Amount/currency mismatch or duplicate-risk state | Report mismatch and source records | Manager/payment reconciliation review | Pick one amount, alter invoice/reservation, ask customer to pay inferred amount |
| Refund request | Summarize policy, payment status, refundability window, and request | Manager/human approval before any refund command or commitment | Promise refund, issue refund, compute exception refund without policy |
| Dispute or chargeback-like claim | Summarize timeline/evidence and missing records | Manager; legal/compliance if regulatory/legal signal appears | Admit fault, threaten customer, reverse/forfeit funds, expose sensitive records |
| Discount/comp/credit request | Summarize requested exception and policy facts | Manager approval | Apply discount/credit, offer deal, change package/rate |
| Fee waiver/deposit waiver request | Summarize policy requirement, reason, and operational impact | Manager approval | Waive fee/deposit, mark deposit satisfied, alter policy snapshot |
| Cancellation inside notice window or holiday/peak policy | Summarize cancellation policy, timing, deposit status, review reason | Manager review unless deterministic approved policy fully resolves outcome | Override policy, promise no penalty/refund/forfeiture, cancel autonomously |
| Ambiguous payment state or conflicting sources | List conflicts and trusted/untrusted sources; create reconciliation task | Staff/payment reconciliation; manager if amount/policy impact is material | Treat unverified claim as paid, choose favorable interpretation, send final customer answer |
| Sensitive billing data or secret exposure risk | Redact/minimize; summarize semantic status only | Security/privacy review if secret/raw payload may be exposed | Show card data, tokens, signed webhook details, raw payloads, unnecessary PII |
| Production provider/webhook setup | Prepare implementation checklist and test plan | Human/provider/integration owner approval | Register live webhook, rotate live secrets, create live checkout without approval |

Escalation routing:

- Front desk / staff: routine deposit collection, routine outstanding-balance follow-up, missing information, customer-safe factual reminder review, and provider-reference lookup when no exception is requested.
- Manager: refunds, disputes, discounts, comps, credits, fee/deposit waivers, forfeitures, cancellation-policy exceptions, holiday/peak exceptions, amount/currency mismatches, missing policy snapshots, ambiguous payment truth with customer impact, complaints, and sensitive customer explanations.
- Payment reconciliation specialist / payment-provider operator: failed payments, processor-reference mismatches, duplicate/partial/unknown transactions, webhook verification issues, provider/store conflicts, and reconciliation job findings.
- Legal/compliance/privacy: chargeback/regulatory/legal threats, suspected fraud, payment secret/raw payload exposure, or requests involving sensitive billing data beyond ordinary operations.
- Engineering/integration owner: webhook signature failures, adapter mapping bugs, stale policy data, provider API/schema ambiguity, idempotency failures, and any workflow where verified payment truth cannot be distinguished from raw/untrusted input.

## Open questions

Policy/business questions:

1. Which services require deposits by default: boarding only, boarding plus grooming rebooking deposits, training/package deposits, daycare packages, or location-specific only?
2. What exact price/rate model is approved for each service, package, accommodation/menu item, add-on, and location?
3. What exact deposit amount model is approved for each service/location/season?
4. Which holiday/peak/local-event periods change pricing, deposit, minimum-stay, cancellation, refund, or reminder rules?
5. What booking lead-time thresholds change due dates, quote validity, hold expiration, or manager-review requirements?
6. What customer-history facts can trigger deposit requirements or special review, and who approves those rules?
7. What is the exact due date and reminder cadence for each deposit/payment scenario?
8. Which customer messages may be auto-sent through deterministic paths, and which are always staff-reviewed drafts?
9. When, if ever, does an unpaid deposit expire a reservation hold, and what human approval is required before capacity is released?
10. What are the approved cancellation notice windows and no-show grace rules by service and season?
11. Which deposits are refundable, non-refundable, partially refundable, or manager-reviewed in normal vs holiday/peak periods?
12. How should partial payment affect hold status, confirmation readiness, checkout readiness, and final balance?
13. How should deposits apply to invoices with add-ons, packages, taxes/fees, discounts, credits, refunds, and multi-pet reservations?
14. Which staff roles can approve routine reminders, manager exceptions, refunds, fee waivers, discounts, forfeitures, and disputed payment resolutions?
15. What approved customer-facing policy copy should be used for deposit requests, reminders, cancellation explanations, no-show notices, refund decisions, disputes, and failed payments?

Provider/integration questions:

1. Is Gingr the operational source-of-truth PMS/ledger, and does the business intend to use Gingr Payments?
2. Which provider is approved for production checkout/deposit collection: Gingr Payments, Stripe Checkout, Square, or another provider?
3. Does the provider need to support in-person terminal/front-desk payments in the same reconciliation path?
4. Which provider webhook events are actually available for the selected account and integration surface?
5. What provider/PMS exports or APIs are available for nightly and on-demand reconciliation?
6. What production environment will hold payment secrets and receive public webhooks?
7. What audit retention and role-based data-access policy applies to raw provider payloads and redacted payment summaries?

## Implementation follow-ups

Documentation/policy follow-ups:

- Create approved location policy records for pricing, deposits, cancellation/refundability, holiday/peak periods, taxes/fees, reminders, hold expiration, and exception roles.
- Add customer-facing policy copy templates with staff/manager approval gates.
- Define role names and approval evidence schema for staff, manager, reconciliation, legal/compliance/privacy, and engineering owners.
- Decide provider path before any production payment or webhook configuration.

Domain/model follow-ups:

- Add or refine provider-neutral payment request, payment attempt, refund request, reconciliation finding, policy snapshot, and approval evidence records.
- Keep provider ids/raw payloads in adapter/boundary DTOs; domain-facing contracts should use semantic amounts, statuses, drafts, references, reasons, and review gates.
- Extend deposit-only status modeling if full balance/refund/dispute state is required.
- Add semantic tests for allowed status transitions, duplicate webhook handling, stale event handling, refund approval gates, discount/waiver gates, and redaction.

Integration follow-ups after provider approval:

- Add provider-specific checkout/session creation adapter with idempotency keys and hosted checkout only.
- Add verified webhook receiver using raw-body signature verification, durable inbound event storage, semantic mapping, idempotent processing, and redacted logs.
- Add reconciliation job contracts comparing internal payment requests, provider payment objects, and PMS/Gingr ledger/invoice records.
- Add restricted storage for raw provider payloads and role-limited redacted summaries for staff/AI views.
- Add secret-management deployment plan with separate sandbox/test and production credentials.

Operations follow-ups:

- Define failed-payment task queues and escalation SLAs without inventing customer-facing deadlines.
- Define refund/discount/waiver request queues and approval evidence required before execution.
- Define production runbook for webhook failures, signature mismatches, duplicate events, provider outages, reconciliation mismatches, chargebacks, and accidental sensitive-data exposure.
