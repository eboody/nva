# AI boundaries for payments, pricing, deposits, and billing status

Purpose: define what an AI assistant may and may not do when pricing, deposits, payment reminders, cancellation/payment policy, and billing-status workflows are involved. This section is intended as a synthesis input for `docs/workflows/payments-pricing.md`; it is not the canonical workflow file.

Source basis:

- `docs/domain/petsuites/boarding/implications/08-payment-deposit-handling.md` states that exact dollar amounts, deadlines, refund windows, processor behavior, and local exception rules are location/provider policy data, not hard-coded constants; it also states that the agent may read, summarize, detect, draft, and create internal tasks, but must not charge, refund, waive, forfeit, confirm, cancel, or send payment-sensitive messages autonomously.
- `docs/architecture/domain-contract-skeleton.md` states that payment provider IDs and raw payloads belong in adapters/boundary DTOs, that domain-facing contracts should use semantic amounts/statuses/drafts/references/reasons/review gates, and that agents may recommend actions while deterministic validators decide whether actions are allowed and require human review.
- `docs/integrations/gingr/sdk-readiness-review.md` classifies customer/member-facing messages, writes to Gingr, payment/transaction/invoice handling beyond internal review summaries, and raw PII/payment payload forwarding as human-reviewed execution or privacy-sensitive surfaces.

## Core rule

The AI is a source-grounded assistant and review packet generator, not a financial authority, payment processor, reservation authority, or policy exception approver.

For every payment/pricing workflow, the AI must preserve three boundaries:

1. Truth boundary: billing truth comes only from approved policy records, reservation/payment stores, payment-provider references returned through trusted adapters, manager approvals, and audit records. Customer claims, free-text staff notes, unverified webhook payloads, screenshots, emails, or model inference are not payment truth until reconciled by staff or a trusted system.
2. Authority boundary: deterministic policy validators and authorized staff/manager approvals decide whether an action is allowed. The AI may not use confidence, urgency, sentiment, or customer pressure as authority.
3. Execution boundary: payment movement, price changes, waivers, discounts, refunds, forfeitures, reservation confirmations/cancellations, and customer-facing sends require approved tool paths with audit records. The AI may draft or summarize these actions but may not execute them on its own.

## Allowed AI roles

The AI may perform these roles when using source-backed values and role-appropriate data minimization:

- Explain approved pricing, deposit, cancellation, refundability, and payment-timing policy to staff or customers using cited/current policy facts. If a value is missing, stale, location-specific, or ambiguous, say so and escalate instead of filling the gap.
- Draft payment, balance, deposit, cancellation-policy, or refund-window reminders for human review. Deterministic send paths may use AI-authored copy only when the final template, recipient, facts, and send condition have been approved by policy.
- Identify missing payment/deposit information, missing payment references, amount/currency mismatches, failed payments, overdue balances, missing refundability/deadline data, and absent policy snapshots.
- Summarize payment status from trusted records: amount due/paid/refunded/waived, due date, deposit status, payment reference presence, provider reconciliation state, refundability window, and required next action.
- Produce internal staff or manager review packets that include source facts, uncertainty, risks, candidate actions, and the exact approval needed.
- Create or recommend internal tasks such as deposit collection review, payment reconciliation, refund review, waiver review, forfeit review, checkout balance review, or customer reminder review.
- Redact or minimize sensitive billing details before summarizing. Prefer semantic statuses and references over raw provider payloads.

## Forbidden AI roles

The AI must not:

- Change prices, package rates, taxes, fees, deposit requirements, refund windows, cancellation rules, or policy snapshots.
- Create discounts, comps, credits, fee waivers, deposit waivers, forfeitures, refunds, voids, write-offs, or goodwill exceptions.
- Capture, retry, void, or refund a payment; store or expose card/payment secrets; or generate signed webhook/payment-provider details.
- Invent payment status, infer that payment happened from a customer claim, or treat unverified notes/screenshots/emails/webhooks as payment truth.
- Promise availability, confirmation, check-in/check-out completion, cancellation approval, refund timing, or policy exceptions.
- Override cancellation policy, deposit policy, holiday/peak rules, minimum-stay rules, or manager/legal review gates.
- Send customer-facing payment-sensitive messages unless an approved deterministic send path has already fixed the facts, recipient, template, and send condition; otherwise it may only draft for review.
- Expose sensitive billing data beyond the recipient's role. Customer-facing summaries should use minimal operational language; internal staff summaries may include more context; provider payloads/secrets remain behind adapter boundaries.
- Use model confidence as evidence that a payment, refund, discount, waiver, or policy exception is valid.

## Trusted sources and non-trusted inputs

Trusted billing-status sources:

- Approved location/provider policy records for prices, deposits, cancellation windows, refundability, fees, and due timing.
- Reservation records and immutable policy snapshots attached to the reservation/stay.
- Payment/deposit projections returned through trusted payment repositories/adapters, including semantic `DepositStatus`, `PaymentReference`, amount, currency, and reconciliation status.
- Payment-provider references and statuses returned through approved adapters after verification.
- Manager/staff approvals captured in audit/workflow records with actor, timestamp, scope, and reason.
- Audit events and workflow events that preserve prior decisions and tool outcomes.

Not payment truth by themselves:

- Customer statements such as "I already paid" or "someone waived this".
- Staff free-text notes that do not reference a trusted payment/policy/approval record.
- Screenshots, emails, attachments, or chat messages that have not been reconciled.
- Unsigned or unverified webhook payloads.
- Raw provider JSON before mapping/verification.
- Prior AI summaries, drafts, or recommendations.

When trusted sources conflict, the AI must report the conflict and escalate; it must not choose the most favorable interpretation.

## Billing-status summary rules

A safe AI billing summary should include only:

- Reservation/customer/pet/location identifiers appropriate for the recipient role.
- Current trusted status: paid, unpaid, partially paid, failed, refunded, waived, disputed, ambiguous, or reconciliation required.
- Amount/currency and due date only when present in trusted records.
- Policy basis: deposit required/not required, due at booking/check-in/check-out/by local deadline, refundability window, cancellation-policy state.
- Evidence: payment reference present/absent, approval present/absent, policy snapshot present/absent, provider reconciliation status.
- Next required human or deterministic action.

It should not include raw card data, payment tokens, signed webhook material, unredacted provider payloads, unnecessary PII, or speculation about why payment failed.

## Reminder drafting rules

Payment/deposit reminders are allowed only as drafts unless a deterministic send path has been separately approved.

Every draft must:

- Use source-backed amount, due date, deposit/refundability status, and cancellation-policy language.
- Include uncertainty explicitly when a fact is missing or ambiguous.
- Avoid threats, legal claims, refund promises, availability promises, or manager-exception promises.
- Avoid offering discounts, waivers, or altered payment terms.
- Identify the review gate: staff review for routine factual reminders; manager review for exceptions, disputes, refunds, waivers, forfeitures, complaints, sensitive tone, or ambiguous records.

## Escalation matrix

| Situation | AI may do | Required escalation | AI must not do |
| --- | --- | --- | --- |
| Routine source-backed price/deposit explanation | Explain current approved policy; cite source/policy version or say the source is missing | Staff review if customer-facing and not on an approved template/send path | Invent amounts, quote stale/location-unknown rates, promise availability |
| Missing deposit or unpaid balance with clear trusted status | Summarize amount/due date/status; draft staff task or reminder | Staff/front-desk review or deterministic approved collection workflow | Charge payment, confirm booking/check-in/check-out, send unapproved payment message |
| Failed/declined payment or missing provider reference | Flag reconciliation need; summarize trusted evidence | Staff/payment reconciliation review | Retry payment, mark paid, blame a customer/card, expose provider internals |
| Amount or currency mismatch | Report mismatch and source records; prepare reconciliation packet | Manager/payment reconciliation review | Pick one amount, alter invoice/reservation, ask customer to pay an inferred amount |
| Refund request | Summarize policy, payment status, refundability window, and customer request | Manager approval before any refund command or customer commitment | Promise refund, calculate exception refund without policy, issue refund |
| Dispute or chargeback-like claim | Summarize timeline/evidence and missing records | Manager review; legal/compliance review if regulatory/legal signal appears | Admit fault, threaten customer, reverse/forfeit funds, expose sensitive records |
| Discount/comp/credit request | Summarize requested exception and eligible policy facts | Manager approval | Apply discount/credit, offer a deal, change package/rate |
| Fee waiver/deposit waiver request | Summarize policy requirement, requested reason, and operational impact | Manager approval | Waive fee/deposit, mark deposit satisfied, alter policy snapshot |
| Cancellation inside notice window or holiday/peak policy | Summarize cancellation policy, timing, deposit status, and review reason | Manager review unless deterministic policy explicitly resolves outcome | Override policy, promise no penalty/refund/forfeiture, cancel reservation autonomously |
| Ambiguous payment state or conflicting sources | List conflicts and trusted/untrusted sources; create reconciliation task | Staff/payment reconciliation; manager if amount/policy impact is material | Treat unverified claim as paid, choose favorable interpretation, send final customer answer |
| Sensitive billing data or secret exposure risk | Redact/minimize; summarize semantic status only | Security/privacy review if secret/raw payload may have been exposed | Show card data, tokens, signed webhook details, raw provider payloads, or unnecessary PII |
| Customer-facing payment reminder | Draft copy and evidence packet | Staff review; manager review for exceptions/disputes/sensitive tone; deterministic send only if pre-approved | Send autonomously, include unverified facts, threaten/promise beyond policy |
| Billing-status summary for internal ops | Provide role-appropriate summary and next action | Staff review for routine follow-up; manager for exceptions/ambiguity | Include unnecessary raw payloads/secrets or make policy decisions |

## Escalation routing

- Front desk / staff: routine deposit collection, routine outstanding-balance follow-up, missing information, customer-safe factual reminder review, and provider-reference lookup when no exception is requested.
- Manager: refunds, disputes, discounts, comps, credits, fee/deposit waivers, forfeitures, cancellation-policy exceptions, holiday/peak exceptions, amount/currency mismatches, missing policy snapshots, ambiguous payment truth with customer impact, complaints, or sensitive customer explanations.
- Payment reconciliation specialist or payment-provider operator: failed payments, processor-reference mismatches, duplicate/partial/unknown transactions, webhook verification issues, and provider/store conflicts.
- Legal/compliance/privacy: chargeback/regulatory/legal threats, suspected fraud, accidental exposure of payment secrets/raw provider payloads, or requests involving sensitive billing data beyond ordinary operations.
- Engineering/integration owner: webhook signature failures, adapter mapping bugs, stale policy data, provider API/schema ambiguity, or any workflow where the system cannot distinguish verified payment truth from raw/untrusted input.

## Implementation implications for `payments-pricing.md`

The final workflow should model AI output as typed drafts, review packets, and internal tasks, not executable billing decisions. It should require:

- A source citation or policy/reference id for every quoted price, amount, due date, refundability claim, and billing status.
- A `trusted_source_state` distinction between verified, missing, ambiguous, conflicting, and untrusted.
- A `review_gate` for every customer-facing message, refund/waiver/discount/forfeit path, and ambiguous payment state.
- A redaction/data-minimization rule for payment-sensitive details.
- An audit event whenever staff/manager approval turns an AI draft/recommendation into an external message, reservation mutation, payment command, refund, waiver, discount, or policy exception.
