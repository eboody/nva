# Message generation schema

Purpose: define the structured output contract for each generated customer-message draft produced by the Customer Messaging Agent. This is a draft/review schema, not a send command. A valid output may create reviewable copy, a no-send/suppression result, or a human-escalation reason; it does not authorize live customer delivery, provider writes, booking changes, payment actions, refunds, waivers, medical/behavior determinations, or policy exceptions.

Source basis:

- `docs/workflows/customer-messaging-parts/inputs.md` is the canonical input packet for this schema.
- Current workflow support is `WorkflowResult<T>` with `structured_output`, `recommended_actions`, `risk_flags`, `verification`, and optional `human_review_reason` in `domain/src/workflow.rs`.
- Current draft-message primitive is `RecommendedAction::DraftMessage { channel, body }`, where `message::Channel` is a non-empty string up to 80 chars and `message::Body` is non-empty text up to 2,000 chars.
- Current canonical contact channels are `ContactChannel::{Email, Sms, Phone, Portal}` in `domain/src/entities.rs`; this schema uses `email`, `sms`, and `portal` for generated written customer-message drafts. `phone` is a staff call-task or call-script channel unless a later artifact explicitly scopes phone scripts as generated customer messages.
- Customer-facing sends remain review-gated by default through `CustomerMessageApproval`; sensitive categories may additionally require `ManagerApproval`, `MedicalDocumentReview`, `BehaviorReview`, or `RefundOrDepositException`.

## Schema object

Each generated customer-message draft uses this top-level JSON-compatible object:

```json
{
  "channel": "email",
  "recipient": {
    "customer_id": "cust_123",
    "display_name": "Jordan Lee",
    "destination_ref": "contact.email.primary",
    "destination_redacted": "jo***@example.com",
    "portal_account_ref": null,
    "consent_state": "allowed",
    "quiet_hours_state": "send_window_open"
  },
  "subject": "Question about Milo's upcoming stay",
  "body": "Hi Jordan — thanks for reaching out about Milo's upcoming stay...",
  "category": "inquiry",
  "requires_approval": true,
  "approval_reason": "CustomerMessageApproval required for all customer-facing drafts in manual-v1; no deterministic send policy was supplied.",
  "facts_used": [
    {
      "fact_id": "fact_001",
      "claim": "Milo is the pet for reservation res_456.",
      "source_ref": "reservation:res_456.pet_id",
      "source_kind": "normalized_record",
      "trust_state": "trusted",
      "used_in": ["body"],
      "sensitivity": "ordinary",
      "freshness": "current_at_prompt_time"
    }
  ],
  "forbidden_claims_checked": [
    {
      "claim_type": "booking_confirmation_or_capacity_promise",
      "status": "absent",
      "evidence": "No phrasing confirms space, approval, or reservation status beyond the cited request context."
    }
  ]
}
```

The object must be embedded as the Customer Messaging Agent's typed `structured_output`. If also projected into current domain primitives, `channel` and `body` may populate `RecommendedAction::DraftMessage`, while the richer fields remain in `structured_output` and audit/review metadata.

## Field contract

### `channel`

Type/shape: required string enum.

Allowed values for generated written drafts:

- `email`
- `sms`
- `portal`

Reserved or out of scope:

- `phone`: only valid if the output is explicitly a staff call script in a later schema; otherwise generate an internal customer-follow-up task, not a customer message.
- `whatsapp`: invalid until a typed consent/provider/template/audit design exists.

Channel-specific rules:

- Email: may include a `subject`; body can be longer and may include paragraph breaks. Must use an email-capable destination reference and redacted email display.
- SMS: `subject` must be `null`; body must be short, plain text, and should include only the minimum facts needed for the customer action. Must use an SMS-capable destination reference, confirmed mobile-phone destination, consent/opt-out state, quiet-hours policy, and suppression/delivery-failure state.
- Portal: `subject` may be present when the portal notification surface supports titles; otherwise `null`. Must use a portal account/customer reference and portal notification/inbox surface approved by product policy.

Validation constraints:

- Non-empty and exactly one allowed value.
- Must be compatible with `recipient.destination_ref` and `recipient.consent_state`.
- Must not be chosen only because it is available on the customer record; availability is not consent.
- Must not silently switch from the preferred or requested channel after a delivery failure; channel replacement requires policy support and audit.

Example values:

```json
"email"
"sms"
"portal"
```

Failure/escalation behavior:

- If channel is missing, unsupported, lacks consent/quiet-hours facts, or conflicts with destination state, output `WorkflowStatus::NeedsMoreInformation` or `NeedsHumanReview` and do not produce a send-ready draft.
- If the only available channel is `phone`, create an internal follow-up/call-task recommendation rather than forcing this schema.
- If channel policy is unresolved, set `requires_approval = true` and explain the missing policy in `approval_reason`.

### `recipient`

Type/shape: required object identifying the intended customer-visible recipient and destination without exposing unnecessary raw PII.

Required shape:

```json
{
  "customer_id": "CustomerId or provider-mapped customer ref",
  "display_name": "Customer display name for salutation/review",
  "destination_ref": "stable reference to email, phone, portal inbox, or provider contact record",
  "destination_redacted": "redacted customer-visible destination for review",
  "portal_account_ref": "portal/provider account ref or null",
  "consent_state": "allowed | denied | unknown | not_modeled | opted_out | suppressed",
  "quiet_hours_state": "send_window_open | send_window_closed | unknown | not_modeled",
  "delivery_suppression_state": "none | prior_failure | bounced | complained | unsubscribed | unknown"
}
```

Required vs optional rules by channel:

- Email requires `customer_id`, `destination_ref`, `destination_redacted`, `consent_state`, `quiet_hours_state`, and `delivery_suppression_state`. `portal_account_ref` may be `null`.
- SMS requires all email fields plus a destination known to be SMS/mobile-capable. `subject` must be `null`.
- Portal requires `customer_id`, `portal_account_ref`, `destination_ref`, `consent_state`, and `delivery_suppression_state`; `destination_redacted` may be a portal username/ref instead of email/phone.

Validation constraints:

- The recipient must map to the customer/reservation/pet context in the prompt packet.
- Raw email addresses, phone numbers, provider payloads, webhook signatures, payment secrets, and high-PII payloads should not appear unless the review UI explicitly needs them; use stable refs and redacted display.
- `consent_state` values other than `allowed` require `requires_approval = true` and normally suppress external sending.
- `quiet_hours_state = send_window_closed` requires hold-for-review/scheduling behavior; do not bypass quiet hours.
- `delivery_suppression_state` other than `none` requires review or suppression.

Example:

```json
{
  "customer_id": "customer:9f4b",
  "display_name": "Jordan Lee",
  "destination_ref": "customer:9f4b.email.primary",
  "destination_redacted": "jo***@example.com",
  "portal_account_ref": null,
  "consent_state": "allowed",
  "quiet_hours_state": "send_window_open",
  "delivery_suppression_state": "none"
}
```

Failure/escalation behavior:

- Missing recipient identity, destination, consent, or suppression facts route to `NeedsMoreInformation` or a staff follow-up task.
- Recipient/customer mismatches route to `FailedSafely` or `NeedsHumanReview` with a privacy risk flag.
- Opt-out, unsubscribe, complaint suppression, bounced address, or unknown consent should produce a no-send/suppression result unless a human policy override exists.

### `subject`

Type/shape: nullable string.

Required vs optional rules by channel:

- Email: required, non-empty.
- SMS: must be `null`.
- Portal: required only when the portal surface supports a title/subject; otherwise `null`.

Validation constraints:

- Email/portal subjects should be concise, customer-safe, and source-grounded.
- Must not include raw payment details, medical specifics, behavior allegations, incident details, legal/privacy labels, internal queue names, staff blame, or unsupported promises.
- Must not imply booking confirmation, availability, approval, refund/waiver, medical determination, vaccine acceptance, group-play eligibility, or payment completion unless those facts are explicitly cited and approved for customer copy.

Example values:

```json
"Question about Milo's upcoming stay"
"Reminder: Luna's grooming appointment"
null
```

Failure/escalation behavior:

- Missing email subject fails validation.
- Subject/body mismatch requires rewrite before review.
- Sensitive or unsupported subject claims route to manager/customer-message approval with the offending span identified in `forbidden_claims_checked` or validator notes.

### `body`

Type/shape: required string containing the proposed customer-visible message body.

Channel-specific rules:

- Email: may include greeting, short paragraphs, and signoff. Keep copy concise and operationally clear.
- SMS: plain text only, no subject, minimal detail, no long policy explanation, no sensitive detail unless explicitly approved. Use a staff callback/escalation route for complex issues.
- Portal: may be similar to email or a shorter portal notification depending on product surface; must not rely on unapproved portal JavaScript/browser events as authoritative operational state.

Validation constraints:

- Non-empty after trimming.
- Must fit the current domain primitive limit when projected to `message::Body`: max 2,000 chars. Product-specific limits may be stricter, especially SMS.
- Must be fully supported by `facts_used`; every customer-visible factual claim must map to one or more `facts_used` entries.
- Must not expose raw provider JSON, raw OCR text, raw email bodies, unredacted PII beyond normal customer salutation/context, payment/card secrets, webhook signatures, API keys, or internal-only notes.
- Must not include forbidden claims unless the corresponding trusted fact, policy, and approval are cited. Forbidden claim classes are listed under `forbidden_claims_checked`.
- Must use warm, pet-parent-friendly, factual language. Avoid pressure, threats, unsupported reassurance, blame, diagnosis, legal conclusions, and invented location policy.

Example:

```json
"Hi Jordan — thanks for reaching out about Milo's upcoming boarding request. We have your request for June 12-14 and our team is reviewing the details. We'll follow up if we need anything else before confirming next steps."
```

Failure/escalation behavior:

- Unsupported factual span: reject or route to rewrite with a rejected-AI-output reason.
- Missing required fact: generate no-send/needs-info output or draft a customer-safe request for that specific missing fact if allowed.
- Sensitive content: require the applicable review gate before customer-visible use.
- Over-length or channel-inappropriate copy: rewrite or downgrade to a staff follow-up task.

### `category`

Type/shape: required string enum naming the message purpose.

Allowed initial categories, aligned to `docs/workflows/customer-messaging-parts/message-categories.md`:

- `inquiry`
- `missing_info`
- `vaccine_reminder`
- `booking_offer_confirmation`
- `deposit_request`
- `waitlist`
- `pre_arrival`
- `daily_update`
- `incident_draft`
- `checkout`
- `review_request`
- `suppression_no_send`

Validation constraints:

- Category must be compatible with the workflow event, subject, service line, allowed actions, and required reviews in the prompt packet.
- Category must not expand automation authority. For example, `booking_offer_confirmation` is a candidate draft/review packet, not a confirmation action.
- Categories involving payment, incident, complaint, medical/document, behavior, capacity, refund/waiver, or policy exceptions require human review by default.
- `review_request` must check eligibility/suppression facts: recent incident, complaint, payment dispute, unresolved issue, negative sentiment, contact history, and approval policy.
- `suppression_no_send` is valid when the safest output is to not produce customer copy; its `body` should be an internal-safe explanation or empty only if the downstream typed variant supports no-message objects. If this exact schema is required, use a short internal no-send rationale and do not project it to `RecommendedAction::DraftMessage`.

Example:

```json
"daily_update"
```

Failure/escalation behavior:

- Unknown category routes to `NeedsHumanReview` or schema validation failure.
- Category/event mismatch routes to `RejectedByPolicy` or rewrite. Example: a `booking.requested` event must not produce a confirmed-booking message unless a verified confirmation/approval input exists.
- If no category fits, output `NeedsMoreInformation` with the missing product/template classification.

### `requires_approval`

Type/shape: required boolean.

Rules:

- Default must be `true` for all customer-facing drafts in manual-v1.
- May be `false` only for a future deterministic send path where the prompt packet supplies all of the following: approved category/template, fixed allowed facts, verified recipient/channel/consent/quiet-hours state, non-sensitive content class, send condition, policy version, idempotency key semantics, audit path, and explicit automation authorization.
- Must be `true` whenever `approval_reason` is non-null.

Validation constraints:

- If any required review gate is present in `PolicyContext.required_reviews`, this must be `true`.
- If any forbidden-claim check is `requires_approval`, `blocked`, or `not_checked`, this must be `true`.
- If any fact has `trust_state` other than `trusted` or `approved`, this must be `true` or the output must suppress the customer message.

Example:

```json
true
```

Failure/escalation behavior:

- A `false` value without a deterministic send policy is invalid and should be rejected by validators.
- Conflicts between model output and policy-required reviews route to `NeedsHumanReview` and `CustomerMessageApproval`.

### `approval_reason`

Type/shape: nullable string. Required non-empty string when `requires_approval = true`; must be `null` only when `requires_approval = false` and deterministic send authority is proven.

Validation constraints:

- Must name the specific approval gate or policy reason, not a vague phrase like `needs review`.
- Should include the blocking sensitive content class or missing authority: customer-facing draft, incident, payment/refund, medical/document, behavior, booking/capacity promise, complaint/legal/privacy, unknown consent, quiet hours, missing source fact, unresolved policy, or unsupported deterministic template.
- Should be customer-safe enough for internal review displays but need not be customer-visible.

Examples:

```json
"CustomerMessageApproval required for all customer-facing drafts in manual-v1."
"ManagerApproval and CustomerMessageApproval required because this draft discusses an incident follow-up."
"RefundOrDepositException required before mentioning any deposit waiver or refund outcome."
```

Failure/escalation behavior:

- Missing `approval_reason` while `requires_approval = true` fails validation.
- Vague approval reasons should be rewritten or augmented by deterministic validators.
- If the model claims no approval is required but the policy context requires review, override to review and record a rejected-output quality issue.

### `facts_used`

Type/shape: required array of fact-citation objects. Must contain at least one entry for any non-empty customer-visible body. May be empty only for `suppression_no_send` outputs that are not projected to a customer draft.

Required fact object shape:

```json
{
  "fact_id": "stable id unique inside this output",
  "claim": "short natural-language statement of the exact fact used",
  "source_ref": "stable domain/policy/evidence/reference id",
  "source_kind": "normalized_record | policy_snapshot | staff_approved_note | manager_decision | provider_verified_mapping | payment_provider_ref | customer_supplied_text | reviewed_document | template | prior_message | derived_fact",
  "trust_state": "trusted | approved | customer_supplied_unverified | provider_unverified | stale | conflicting | missing | redacted",
  "used_in": ["subject", "body", "recipient", "category", "approval_reason"],
  "sensitivity": "ordinary | pii | medical | behavior | incident | payment | legal_privacy | staff_internal | policy_exception",
  "freshness": "current_at_prompt_time | as_of_timestamp | stale | unknown",
  "approved_for_customer_copy": true
}
```

Validation constraints:

- Every factual claim in `subject` and `body` must be covered by one or more `facts_used` entries.
- `claim` must be an exact, narrow fact, not a broad source summary. Good: `Reservation res_456 is a boarding request for June 12-14.` Bad: `The reservation is fine.`
- `source_ref` must point to a normalized record, policy snapshot, reviewed document, approval record, trusted provider mapping, prior message, or approved template. Raw provider/OCR/email/customer text can be referenced only after redaction/review state is explicit.
- `customer_supplied_unverified`, `provider_unverified`, `stale`, `conflicting`, `missing`, or `redacted` facts cannot support confident customer claims. They can support a request for clarification or an internal approval reason.
- `approved_for_customer_copy` must be `true` for facts used directly in customer-visible body text; otherwise the draft must be suppressed or rewritten.
- Derived facts must cite all source refs used and must not infer policy, eligibility, medical, behavior, payment, booking, or capacity conclusions without an approved rule/decision.

Examples:

```json
[
  {
    "fact_id": "fact_pet_name",
    "claim": "The pet's name is Milo.",
    "source_ref": "pet:pet_123.name",
    "source_kind": "normalized_record",
    "trust_state": "trusted",
    "used_in": ["body"],
    "sensitivity": "ordinary",
    "freshness": "current_at_prompt_time",
    "approved_for_customer_copy": true
  },
  {
    "fact_id": "fact_review_gate",
    "claim": "The policy context requires CustomerMessageApproval.",
    "source_ref": "workflow_event:evt_789.policy_context.required_reviews",
    "source_kind": "policy_snapshot",
    "trust_state": "trusted",
    "used_in": ["approval_reason"],
    "sensitivity": "policy_exception",
    "freshness": "current_at_prompt_time",
    "approved_for_customer_copy": false
  }
]
```

Failure/escalation behavior:

- If the model cannot cite a needed fact, it must omit that claim, ask for the missing information, or route to human review.
- If a validator detects an uncited body/subject claim, reject the output as unsupported and create or route to a rejected-AI-output review item.
- If cited facts conflict, are stale, or are untrusted, the draft must not present them as settled; route to `NeedsMoreInformation`, `NeedsHumanReview`, or a customer-safe clarification request if allowed.

### `forbidden_claims_checked`

Type/shape: required array of check objects. Must include every forbidden claim class relevant to the category/channel, plus a shared baseline set for all customer-facing drafts.

Required check object shape:

```json
{
  "claim_type": "booking_confirmation_or_capacity_promise",
  "status": "absent | supported | blocked | requires_approval | not_applicable | not_checked",
  "evidence": "short validator/model explanation with source refs or offending span",
  "offending_spans": ["optional exact text spans when blocked/requires_approval/not_checked"]
}
```

Baseline forbidden claim classes to check for every draft:

- `unsupported_fact_or_hallucination`
- `booking_confirmation_or_capacity_promise`
- `payment_charge_refund_waiver_or_deposit_exception`
- `medical_diagnosis_vaccine_or_document_eligibility_determination`
- `behavior_group_play_or_safety_eligibility_determination`
- `incident_fault_liability_or_legal_claim`
- `policy_exception_or_location_specific_rule`
- `provider_write_status_change_or_system_mutation`
- `discount_price_package_or_upsell_claim_without_policy`
- `privacy_or_unnecessary_pii_disclosure`
- `staff_blame_internal_note_or_confidential_detail`
- `unsupported_reassurance_or_outcome_guarantee`
- `unauthorized_channel_consent_or_quiet_hours_bypass`
- `raw_provider_payment_ocr_or_webhook_secret_exposure`

Category-specific checks:

- `booking_offer_confirmation`: must check booking confirmation, capacity, deposit/payment, document/vaccine eligibility, special-care/capacity constraints, cancellation/waitlist/offer wording.
- `daily_update`: must check medical/behavior/incident/safety content, unsupported care outcomes, staff blame, media/privacy permissions.
- `vaccine_reminder`: must check document/vaccine status, upload/receipt-vs-verification wording, medical advice, eligibility consequences, and reviewer authority.
- `deposit_request`: must check payment status, amount/payment-link handling, threats, refunds/waivers/discounts, card/payment secrets, and reconciliation uncertainty.
- `waitlist` and `pre_arrival`: must check capacity/availability promises, dates/times/timezone, document/payment/special-care gates, and policy-copy support.
- `incident_draft`: must check fault/liability/legal/privacy, medical/behavior conclusions, incident facts, refund/waiver/service recovery promises, staff discipline, and manager approval.
- `checkout`: must check completion, balance/receipt, belongings, care-task/final-report, incident, and rebooking/review-request eligibility claims.
- `review_request`: must check incident/complaint/payment-dispute/unresolved issue suppression and pressure/incentive wording.

Validation constraints:

- `status = absent` means the claim class does not appear in `subject` or `body`.
- `status = supported` means the claim appears and is fully supported by `facts_used`, policy, and approval state.
- `status = requires_approval` means the claim may be supportable but cannot be customer-visible until the named review gate approves it.
- `status = blocked` means the claim must be removed or the draft suppressed.
- `status = not_checked` is invalid for send-ready drafts and forces `requires_approval = true`.
- Checks must identify offending spans for blocked or approval-requiring text when possible.

Examples:

```json
[
  {
    "claim_type": "booking_confirmation_or_capacity_promise",
    "status": "absent",
    "evidence": "Body says the request is being reviewed and does not confirm space, acceptance, or booking status.",
    "offending_spans": []
  },
  {
    "claim_type": "unsupported_fact_or_hallucination",
    "status": "supported",
    "evidence": "All visible facts map to fact_pet_name and fact_requested_dates.",
    "offending_spans": []
  }
]
```

Failure/escalation behavior:

- Any `blocked` check suppresses the draft or routes to rewrite before approval.
- Any `requires_approval` check keeps `requires_approval = true` and must be reflected in `approval_reason`.
- Any `not_checked` baseline class fails validation for external sending and creates a review/engineering issue.

## How the schema proves no invented facts or forbidden claims were used

The schema proves grounding through three linked mechanisms:

1. Claim citation: `facts_used` enumerates each factual claim allowed into `recipient`, `subject`, `body`, `category`, and `approval_reason`, with source refs, trust state, sensitivity, freshness, and customer-copy approval.
2. Span/claim validation: validators must compare the generated `subject` and `body` against `facts_used`. Any visible factual span without a supporting fact is an `unsupported_fact_or_hallucination` failure.
3. Forbidden-claim checklist: `forbidden_claims_checked` requires the model and deterministic validators to explicitly mark high-risk claim classes as absent, supported, approval-required, blocked, not-applicable, or not-checked.

A draft is eligible for human review only when:

- The object passes schema validation.
- Every customer-visible factual claim is cited in `facts_used`.
- Every cited fact used in customer copy is trusted or approved for that use.
- Every relevant forbidden claim class has been checked.
- No check is `blocked` or `not_checked`.
- `requires_approval` and `approval_reason` correctly reflect policy/review state.

A draft is eligible for future deterministic no-human send only when all review gates are absent, all facts are trusted and approved for customer copy, the category/template/send path is explicitly authorized by policy, no forbidden check requires approval, consent/quiet-hours/suppression checks pass, and idempotent audited execution exists. Current manual-v1 posture should therefore treat normal generated outputs as review packets, not sends.

## Failure and escalation matrix

| Failure condition | Required behavior |
| --- | --- |
| Schema parse failure or missing required top-level field | `WorkflowStatus::FailedSafely` or validator rejection; no draft projection to send path. |
| Missing recipient, destination, consent, quiet-hours, or suppression facts | `NeedsMoreInformation` or `NeedsHumanReview`; create staff follow-up if appropriate. |
| Unsupported channel (`whatsapp`, unscoped `phone`) | Reject or convert to internal follow-up/call-task workflow. |
| Customer-visible claim lacks `facts_used` support | Reject as unsupported/hallucinated; route to rejected-AI-output review and rewrite. |
| Fact is stale, conflicting, provider-unverified, or customer-supplied-only | Do not present as settled; ask for clarification or route to review. |
| Sensitive category or required review gate present | `requires_approval = true`; name specific gate in `approval_reason`. |
| Forbidden check is `blocked` | Suppress or rewrite; do not send to customer review as-is except as rejected-output evidence. |
| Forbidden check is `not_checked` | Fail validation for send readiness; route to review/engineering. |
| Body exceeds channel/product limit or is channel-inappropriate | Rewrite, choose approved channel only with policy, or create manual follow-up task. |
| Recipient/customer mismatch or privacy leak | `FailedSafely`, privacy risk flag, manager/engineering escalation. |
| Deterministic send requested without policy proof | Override to `requires_approval = true` and record policy/authority failure. |

## Complete examples

### Email inquiry follow-up draft

```json
{
  "channel": "email",
  "recipient": {
    "customer_id": "customer:c_123",
    "display_name": "Jordan Lee",
    "destination_ref": "customer:c_123.email.primary",
    "destination_redacted": "jo***@example.com",
    "portal_account_ref": null,
    "consent_state": "allowed",
    "quiet_hours_state": "send_window_open",
    "delivery_suppression_state": "none"
  },
  "subject": "Question about Milo's boarding request",
  "body": "Hi Jordan — thanks for reaching out about Milo's boarding request for June 12-14. We have the request in review and our team will follow up if we need any additional details before confirming next steps.",
  "category": "inquiry",
  "requires_approval": true,
  "approval_reason": "CustomerMessageApproval required for customer-facing inquiry follow-up drafts in manual-v1.",
  "facts_used": [
    {
      "fact_id": "fact_customer_name",
      "claim": "The customer's display name is Jordan Lee.",
      "source_ref": "customer:c_123.full_name",
      "source_kind": "normalized_record",
      "trust_state": "trusted",
      "used_in": ["recipient", "body"],
      "sensitivity": "pii",
      "freshness": "current_at_prompt_time",
      "approved_for_customer_copy": true
    },
    {
      "fact_id": "fact_pet_name",
      "claim": "The pet's name is Milo.",
      "source_ref": "pet:p_456.name",
      "source_kind": "normalized_record",
      "trust_state": "trusted",
      "used_in": ["subject", "body"],
      "sensitivity": "ordinary",
      "freshness": "current_at_prompt_time",
      "approved_for_customer_copy": true
    },
    {
      "fact_id": "fact_request_dates",
      "claim": "The boarding request is for June 12-14.",
      "source_ref": "reservation:r_789.start_end",
      "source_kind": "normalized_record",
      "trust_state": "trusted",
      "used_in": ["body"],
      "sensitivity": "ordinary",
      "freshness": "current_at_prompt_time",
      "approved_for_customer_copy": true
    },
    {
      "fact_id": "fact_review_required",
      "claim": "Customer-facing drafts require CustomerMessageApproval in manual-v1.",
      "source_ref": "policy_context:evt_001.required_reviews.CustomerMessageApproval",
      "source_kind": "policy_snapshot",
      "trust_state": "trusted",
      "used_in": ["approval_reason"],
      "sensitivity": "policy_exception",
      "freshness": "current_at_prompt_time",
      "approved_for_customer_copy": false
    }
  ],
  "forbidden_claims_checked": [
    {
      "claim_type": "unsupported_fact_or_hallucination",
      "status": "supported",
      "evidence": "All visible facts are covered by fact_customer_name, fact_pet_name, and fact_request_dates.",
      "offending_spans": []
    },
    {
      "claim_type": "booking_confirmation_or_capacity_promise",
      "status": "absent",
      "evidence": "The draft says the request is in review and does not confirm availability, acceptance, or booking status.",
      "offending_spans": []
    },
    {
      "claim_type": "payment_charge_refund_waiver_or_deposit_exception",
      "status": "absent",
      "evidence": "No payment, charge, refund, waiver, or deposit terms appear.",
      "offending_spans": []
    },
    {
      "claim_type": "medical_diagnosis_vaccine_or_document_eligibility_determination",
      "status": "absent",
      "evidence": "No medical, vaccine, or document eligibility determination appears.",
      "offending_spans": []
    },
    {
      "claim_type": "unauthorized_channel_consent_or_quiet_hours_bypass",
      "status": "absent",
      "evidence": "Recipient consent and quiet-hours facts are present; no bypass wording appears.",
      "offending_spans": []
    }
  ]
}
```

### SMS missing-information draft

```json
{
  "channel": "sms",
  "recipient": {
    "customer_id": "customer:c_234",
    "display_name": "Sam Rivera",
    "destination_ref": "customer:c_234.mobile_phone.primary",
    "destination_redacted": "***-***-0198",
    "portal_account_ref": null,
    "consent_state": "allowed",
    "quiet_hours_state": "send_window_open",
    "delivery_suppression_state": "none"
  },
  "subject": null,
  "body": "Hi Sam, we need one more detail before our team can finish reviewing Bella's grooming appointment request. Please reply here or call the front desk when you have a moment.",
  "category": "missing_info",
  "requires_approval": true,
  "approval_reason": "CustomerMessageApproval required for SMS customer-facing drafts; missing-info detail should be verified before send.",
  "facts_used": [
    {
      "fact_id": "fact_customer_name",
      "claim": "The customer's display name is Sam Rivera.",
      "source_ref": "customer:c_234.full_name",
      "source_kind": "normalized_record",
      "trust_state": "trusted",
      "used_in": ["recipient", "body"],
      "sensitivity": "pii",
      "freshness": "current_at_prompt_time",
      "approved_for_customer_copy": true
    },
    {
      "fact_id": "fact_pet_name",
      "claim": "The pet's name is Bella.",
      "source_ref": "pet:p_678.name",
      "source_kind": "normalized_record",
      "trust_state": "trusted",
      "used_in": ["body"],
      "sensitivity": "ordinary",
      "freshness": "current_at_prompt_time",
      "approved_for_customer_copy": true
    },
    {
      "fact_id": "fact_missing_info",
      "claim": "The grooming appointment request needs one more detail before review can finish.",
      "source_ref": "workflow_event:evt_002.structured_output.missing_fields",
      "source_kind": "normalized_record",
      "trust_state": "trusted",
      "used_in": ["body", "category"],
      "sensitivity": "ordinary",
      "freshness": "current_at_prompt_time",
      "approved_for_customer_copy": true
    }
  ],
  "forbidden_claims_checked": [
    {
      "claim_type": "unsupported_fact_or_hallucination",
      "status": "supported",
      "evidence": "Visible claims are covered by customer, pet, and missing-info facts.",
      "offending_spans": []
    },
    {
      "claim_type": "booking_confirmation_or_capacity_promise",
      "status": "absent",
      "evidence": "No appointment confirmation or capacity promise appears.",
      "offending_spans": []
    },
    {
      "claim_type": "privacy_or_unnecessary_pii_disclosure",
      "status": "absent",
      "evidence": "SMS copy avoids detailed PII and does not expose the missing field value.",
      "offending_spans": []
    },
    {
      "claim_type": "unsupported_reassurance_or_outcome_guarantee",
      "status": "absent",
      "evidence": "The draft does not guarantee approval or outcome after the detail is provided.",
      "offending_spans": []
    }
  ]
}
```

## Implementation notes

- Prefer a dedicated future domain type such as `GeneratedCustomerMessageDraft` rather than overloading `RecommendedAction::DraftMessage`; the current primitive loses recipient, subject, approval, citation, and forbidden-claim proof.
- Persist the generated object with a stable draft ID, workflow event ID, policy version, prompt/input snapshot ref, model/version ref, validator version, and idempotency key.
- Treat send retries as retries of an approved immutable payload. Regenerating copy after approval requires a new draft and new approval.
- Redact failure/dead-letter fields shown to staff/engineering; do not expose exact outbound body or high-PII in infrastructure errors.
- Deterministic validators should run after model generation and before any review/send projection: JSON schema validation, channel/recipient policy validation, claim-to-fact coverage, forbidden-claim coverage, sensitivity/review-gate validation, length/channel formatting, and redaction checks.
