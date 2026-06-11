# Payment provider integration path and status sync

Purpose: focused provider-integration section for synthesis into `docs/workflows/payments-pricing.md`.

Status: specification only. No live payment provider configuration, production webhook registration, refund execution, discounts, fee waivers, or pricing changes are approved by this document.

## Provider path

Provider selection is not yet approved. Treat it as a human approval gate.

Recommended decision path:

1. If the pet resort runs its operational ledger in Gingr and can use Gingr Payments, prefer a Gingr-native payment path for production because reservation, owner, pet, invoice, in-person terminal, customer-portal payment, card-on-file, and reconciliation records stay in the same pet-care system of record.
2. If Gingr Payments cannot expose the needed checkout-link, webhook, refund, sandbox, or reporting controls, evaluate Stripe Checkout as the primary external provider. Stripe has mature hosted Checkout, metadata, idempotency, signed webhooks, refunds, disputes, and test tooling.
3. If the business already standardizes on Square POS/Terminal, evaluate Square as the in-person-first provider. Square webhooks can track payment creation and update events, including status changes, but the integration must confirm checkout-link, reservation metadata, refund, and ledger-reconciliation fit before selection.
4. Do not run multiple payment processors for the same reservation workflow unless the reconciliation model explicitly supports provider-specific payment references and duplicate-risk controls.

Practical comparison:

| Option | Fit | Strengths | Risks / questions |
| --- | --- | --- | --- |
| Gingr Payments | Best if Gingr is the source-of-truth PMS/ledger | Pet-care-specific, integrated customer portal, card-on-file, in-person terminals, receipts, batch/reconciliation reports, fewer duplicate records | Need approved tenant/sandbox access, exact checkout/deposit APIs or portal flow, refund/dispute event surfaces, webhook coverage for payment lifecycle, and export/reporting detail |
| Stripe Checkout | Best external hosted checkout candidate | Hosted sessions/payment links, strong metadata/idempotency model, signed webhooks, rich payment/refund/dispute events, strong test mode | Requires separate reconciliation back to Gingr/PMS, staff must not treat Stripe alone as reservation truth, terminal/in-person fit must be assessed if front-desk payments matter |
| Square | Best if business already uses Square POS/Terminal | Strong in-person/POS ecosystem, payment.updated lifecycle events, sandbox and developer console logs | Webhooks are more payment-object-status oriented; reservation/deposit metadata and hosted checkout ownership need validation; Gingr ledger sync may be manual or custom |

Decision criteria before approval:

- Can the provider create a hosted checkout session/link for a specific reservation deposit or balance without exposing card data to our app?
- Can the provider carry stable metadata fields for internal reservation/customer/payment-intent references?
- Does it emit signed webhooks for success, failure, refund, dispute/chargeback, cancellation/expiration, and async processing?
- Can staff reconcile provider payments to the PMS ledger and reservation invoice without manual ambiguity?
- Can refunds be requested in-system but executed only after human approval?
- Does the provider support sandbox/test mode and local webhook testing before production registration?
- Does the provider support in-person front-desk payments if those must share the same workflow?
- What PCI scope is introduced, and can hosted checkout/tokenized card-on-file keep the app out of raw card handling?

## Checkout link/session ownership

Ownership rule: the application may request a hosted checkout session/link only after an approved pricing/deposit policy or staff-entered invoice amount exists. The provider owns card collection. The app owns the semantic payment intent and audit record.

Allowed creation flow:

1. Source reservation/customer/pricing facts from trusted records, not free-form customer claims.
2. Create an internal `PaymentIntent` / `PaymentRequest` record before calling the provider.
3. Generate a deterministic idempotency key from internal payment-request id plus attempt number or stable lifecycle version.
4. Call the selected provider to create a hosted checkout session or payment link.
5. Store only provider ids, checkout URL, expiration, amount, currency, status, and redacted request/response metadata.
6. Send or display the link through an approved staff-reviewed or deterministic messaging path.

Metadata allowed in provider checkout/session metadata:

- Internal reservation id or opaque reservation public reference.
- Internal customer/owner id or opaque account reference.
- Payment subject: `reservation_deposit`, `reservation_balance`, `customer_account`, or approved equivalent.
- Internal payment request id.
- Location id if needed for reconciliation.
- Policy version or quote version.
- Non-sensitive service category such as boarding/daycare/grooming/training when useful for reconciliation.

Metadata forbidden in provider metadata, URLs, logs, and webhook diagnostic fields:

- Pet medical details, vaccination details, medications, feeding notes, behavior notes, incident details, staff notes, or private owner notes.
- Full customer contact data unless the provider requires it for receipt/customer identity and it is sent through supported provider fields, not metadata.
- Raw card data, bank data, CVV, webhook signing secrets, API keys, discounts or fee waivers not already approved, and free-form AI-generated rationale.

Checkout-session ownership boundaries:

- The app can create a new checkout request, expire/cancel a still-open checkout request if the provider supports it, and record provider status.
- The app must not change prices, add discounts, waive deposits, or alter cancellation/refund policy as a side effect of checkout creation.
- Staff/human approval is required for provider selection, discounts, fee waivers, pricing changes, manual amount overrides, and production webhook registration.

## Webhook events and idempotency

All inbound provider webhooks are untrusted until signature verification succeeds. The receiver must read the raw body, verify the provider signature using the provider-specific endpoint secret, parse into a provider DTO, and map to a semantic internal event only after verification.

Required semantic event coverage:

| Semantic event | Stripe candidate events | Square candidate events | Gingr/open question |
| --- | --- | --- | --- |
| Checkout completed / payment succeeded | `checkout.session.completed`, `payment_intent.succeeded`, `charge.succeeded` | `payment.created`, `payment.updated` with `COMPLETED` status | Confirm Gingr Payments webhook/report event coverage; existing local Gingr webhooks include reservation/customer operational events, not a proven full payment lifecycle catalog |
| Payment failed / declined | `payment_intent.payment_failed`, invoice/payment failure events if billing is used | `payment.updated` with `FAILED` status or card/payment status failure | Confirm Gingr failure event or reconciliation report |
| Checkout canceled / expired | `checkout.session.expired`; canceled payment intent if used | `payment.updated` when an authorized delayed-capture payment is canceled/voided | Confirm Gingr checkout expiration/cancellation visibility |
| Refunded / partially refunded | `charge.refunded`, `refund.created`, `refund.updated` depending integration needs | refund webhooks / refund status updates must be confirmed against selected Square APIs | Confirm Gingr refund event/reporting detail |
| Disputed / chargeback | `charge.dispute.created`, `charge.dispute.closed` | Square dispute webhook support must be validated for selected API surface | Confirm Gingr dispute/chargeback reporting |
| Async pending / processing | `payment_intent.processing` where applicable | `payment.created`/`payment.updated` with pending ACH or delayed status | Confirm Gingr async payment states |

Idempotency requirements:

- Provider event id plus provider account id is the first dedupe key. If a provider lacks a stable event id, use provider payment id plus event type plus provider updated timestamp, and record why this fallback is safe.
- Store every verified webhook in a durable inbound event table before business processing. Return provider success only after durable acceptance or explicit permanent ignore.
- Processing must be idempotent: repeating the same event cannot double-apply a payment, double-send a reminder, double-release a reservation hold, or double-create a refund request.
- Outbound provider calls use deterministic idempotency keys for checkout/session creation, refund requests, cancellation/expiration, and ledger-record updates.
- Ignore stale regressions unless the provider source-of-truth fetch confirms them. Example: do not move `succeeded` back to `pending` based only on an older webhook delivery.
- Unknown verified events are recorded and either ignored with a permanent-accept policy or routed to staff review; they must not silently drive automation.

## Internal payment statuses and transitions

Recommended internal status enum for provider-backed payment requests:

- `draft`: internal request is being assembled; no provider object exists.
- `checkout_created`: provider checkout/session/link exists but has not been sent or displayed.
- `checkout_sent`: customer has been given the hosted link.
- `pending`: provider reports an async or processing state.
- `succeeded`: provider confirms successful payment/capture for the expected amount/currency.
- `failed`: provider reports decline/failure and no successful payment has superseded it.
- `canceled`: staff/system canceled the request before successful payment.
- `expired`: provider checkout/session expired.
- `partially_refunded`: provider confirms a partial refund.
- `refunded`: provider confirms a full refund.
- `disputed`: provider reports a chargeback/dispute.
- `requires_review`: amount mismatch, duplicate risk, ambiguous provider state, unknown event, source mismatch, or policy exception requires staff/manager review.

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
failed        -> checkout_created  (new attempt, new attempt number)
expired       -> checkout_created  (new attempt, if policy still allows hold)
any           -> requires_review   (mismatch, duplicate, unknown, manual exception)
```

Source-of-truth rules:

- Provider is source of truth for provider payment object lifecycle: created, processing, succeeded, failed, refunded, disputed, canceled, expired.
- PMS/Gingr/reservation ledger is source of truth for reservation status, invoice/balance application, booking confirmation, capacity hold, and customer-visible booking state.
- Internal system is source of truth for workflow status, idempotency, review gates, reminders sent, staff notifications, and provider-to-reservation mapping.
- A provider `succeeded` event does not by itself confirm or modify a reservation. It permits the system to record the payment and request/perform an approved ledger/reservation update through typed adapters.
- A PMS/Gingr balance marked paid without a matching provider event should be reconciled as external/manual payment, not backfilled as provider success without evidence.

Reconciliation strategy:

- Nightly and on-demand reconciliation compares internal payment requests, provider payment objects, and PMS/Gingr ledger/invoice records.
- Reconciliation keys: provider payment id/reference, internal payment request id, reservation id, amount, currency, customer/account id, provider account/location id, and timestamps.
- Mismatches route to `requires_review`: amount mismatch, currency mismatch, duplicate provider payments for one request, successful provider payment not applied to ledger, ledger marked paid without provider evidence, refund in provider not reflected in ledger, dispute not reflected in operational risk queue.
- Keep raw provider payloads in restricted storage for audit/debug; expose only redacted semantic summaries to agents and general staff views.

## Failed payment handling

Failure handling must separate provider failure, customer reminder, staff action, and reservation hold policy.

Recommended behavior:

1. On verified failure/decline, mark the attempt `failed` and keep the payment request open only if policy allows another attempt.
2. Record provider decline category in a redacted enum (`card_declined`, `insufficient_funds`, `requires_customer_action`, `provider_unavailable`, `unknown`) rather than raw processor text.
3. Create a new checkout attempt with a new idempotency key only when policy/staff action permits retry.
4. Draft or send reminders according to the approved messaging policy. If deterministic auto-send is not approved, create staff-reviewed drafts only.
5. Notify staff/manager when payment failure threatens reservation confirmation, hold expiration, holiday capacity, repeated decline, amount mismatch, dispute, or unclear state.

Reservation-hold impact:

- Payment failure does not automatically cancel a reservation unless an approved reservation/deposit policy explicitly says so.
- If a required deposit is unpaid, existing domain semantics should treat confirmation readiness as blocked by deposit requirement and route deposit exceptions to human review.
- Holds may be marked `deposit_pending`, `payment_failed`, or `hold_expiring` in internal workflow, but final release/cancellation must follow approved policy and staff review gates.
- For holiday/peak/high-value bookings, failure should escalate earlier to staff because capacity may be scarce, but the workflow must not invent cancellation terms or deadlines.

Reminder cadence inputs needed from policy synthesis:

- Initial due date and checkout link expiration.
- Number/timing of reminders.
- Whether SMS/email/portal messages can be deterministic or staff-reviewed only.
- Hold expiration deadline and manager override rules.

## Refund request and approval workflow

System behavior:

1. Staff or workflow records a refund request with reservation/payment reference, requested amount, reason, requester, supporting note, and policy basis.
2. System validates that the payment exists, amount/currency are plausible, and prior refund/dispute state is known.
3. System sets internal refund status to `requested` or `requires_review`; it does not execute the provider refund automatically.
4. Manager/human approves, rejects, or requests more information.
5. Only after approval may a provider refund call be executed with an idempotency key and approved reason.
6. Webhooks/reconciliation update refund status to `submitted`, `succeeded`, `failed`, `partially_refunded`, or `refunded`.
7. Ledger/PMS updates occur through approved adapters and remain auditable.

Refund approval gates:

- Refund execution always requires human approval.
- Partial refunds, fee waivers, discounts, cancellation-fee waivers, deposit exceptions, service-not-rendered adjustments, and outside-window exceptions require manager approval.
- Disputed payments must escalate before issuing a refund to avoid double-loss or conflicting provider actions.

## Security, audit, and redaction boundaries

Secret storage:

- Store provider API keys and webhook endpoint secrets only in the approved secret manager/environment mechanism for the deployment target.
- Do not commit secrets, endpoint signing keys, dashboard credentials, live checkout URLs with embedded secrets, or real payload fixtures.
- Use separate test/sandbox and production credentials.

Webhook verification:

- Verify provider signatures over raw request bodies before JSON-derived business logic.
- Use provider-specific timestamp/replay protections where available.
- Reject or quarantine unsigned, malformed, or signature-mismatched events.
- Keep unverified payloads out of domain workflows and AI prompts.

Redaction:

- Logs and audit summaries may contain provider name, provider object ids, internal payment request id, status, amount/currency, event type, and redacted customer/reservation references.
- Logs must not contain full card/bank data, CVV, API keys, webhook secrets, raw signatures, full owner contact details unless operationally required and role-restricted, medical/behavior notes, or raw provider payloads in general logs.

Audit trail:

- Record who/what created checkout links, sent reminders, approved refunds, executed provider calls, handled webhook events, changed status, and resolved reconciliation mismatches.
- Store before/after status, source (`provider_webhook`, `provider_fetch`, `staff_action`, `reconciliation_job`, `pms_sync`), idempotency key, provider event id, and review gate outcome.
- AI-generated summaries or drafts must be labeled as drafts/summaries and linked to source records; they are not payment truth.

## Existing repo touchpoints for future implementation

Current implementation/documentation anchors:

- `domain/src/payment/mod.rs`: existing `PaymentReference`, `Deposit`, and `DepositStatus` semantics. Current deposit statuses include `NotRequired`, `Required`, `Paid`, `Refunded`, `Failed`, and `WaivedByManager`.
- `domain/src/tools.rs`: existing `tools::payments::PaymentGateway` trait with `authorize`, `refund`, and `record_deposit`; `AuthorizationRequest`, `AuthorizationResult`, `RefundRequest`, `RefundResult`, `DepositRecordRequest`; idempotency key and typed subject/reason enums.
- `docs/architecture/domain-contract-skeleton.md`: boundary rule that provider IDs/raw payloads belong in adapters or boundary DTOs; domain-facing contracts should use semantic amounts, statuses, drafts, references, reasons, and review gates.
- `integrations/gingr/src/webhook.rs` and `docs/integrations/gingr/sdk-webhooks.md`: local example of webhook signature verification, typestate-like unverified-to-verified separation, durable acknowledgement semantics, and payload quarantine for Gingr operational webhooks.
- `integrations/gingr/`: likely host for Gingr-native payment/PMS adapters if Gingr Payments is selected or if payment status must reconcile to the Gingr ledger.
- Future provider adapter location should be explicit after provider selection, e.g. `integrations/stripe/`, `integrations/square/`, or a Gingr payments module under `integrations/gingr/`.
- Future storage should add restricted inbound webhook/event, payment request, payment attempt, refund request, reconciliation finding, and audit tables/modules rather than placing provider payloads directly in domain entities.

Future implementation follow-ups:

- Add a provider-neutral internal payment request/status model separate from deposit-only `DepositStatus` if full balance/refund/dispute states are required.
- Add provider-specific webhook verifier/adapters after provider approval.
- Add reconciliation job contracts and storage for idempotent inbound event processing.
- Extend `PaymentGateway` or split it into capability traits when checkout-link creation, provider status fetch, refund execution, and ledger recording need different review/secret boundaries.
- Add semantic tests for allowed status transitions, duplicate webhook handling, refund approval gates, and redaction.

## Open questions for final synthesis

- Is Gingr the operational source-of-truth PMS and does the business intend to use Gingr Payments?
- Which provider is approved for production checkout/deposit collection?
- Does the provider need to support in-person terminal payments in the same integration path?
- What exact deposit/payment due dates, reminder cadence, and hold expiration rules are approved?
- Which messages may be auto-sent versus staff-reviewed drafts only?
- What refund approval roles exist, and who can approve partial refunds or outside-window exceptions?
- What provider/PMS reporting exports are available for reconciliation?
- What production environment will hold secrets and receive public webhooks?
