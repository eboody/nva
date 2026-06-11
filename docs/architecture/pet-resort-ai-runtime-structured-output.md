# Structured output validation and escalation

Purpose: define the baseline contract for AI/Hermes runtime results in the pet-resort application. This is an architecture guardrail for agents that read domain state, draft messages, recommend booking/document/incident actions, or create internal task packets.

Status: draft architecture convention. This document authorizes no production automation by itself. Production side effects still require deterministic policy checks, approved tool paths, audit records, and the review gates named by the relevant workflow.

## Source anchors

Use these current repo contracts as the baseline until richer runtime types are implemented:

- `domain/src/agents.rs` defines `WorkflowAgent<Input, Output>`, `AgentPromptPacket<T>`, baseline agent specs, forbidden actions, and default review gates.
- `domain/src/workflow.rs` defines the current implemented `WorkflowEvent`, `PolicyContext`, `AllowedAction`, `WorkflowResult<T>`, `WorkflowStatus`, `RecommendedAction`, risk flags, verification notes, and human review reason.
- `docs/architecture/workflow-result-envelope.md` defines the proposed workflow result envelope for downstream consumers. This document defines the validation/retry/escalation runtime around that envelope; where names differ, use the mapping in that envelope during migration.
- `docs/architecture/domain-contract-skeleton.md` states that agents may recommend actions while deterministic Rust validators decide whether an action is allowed and whether human review is required.
- `docs/workflows/payments-pricing-parts/ai-boundaries.md` defines the payment-sensitive truth/authority/execution boundary: AI can read, summarize, draft, and create review packets, but cannot charge, refund, waive, forfeit, confirm, cancel, or send payment-sensitive messages autonomously.
- `docs/workflows/staff-operations-parts/inputs.md` defines the staff-operations posture: AI workflow workers read, extract, draft internal tasks/messages, suggest status/play eligibility, summarize care notes, and flag risk while preserving review gates.

## Non-negotiable rule

Unvalidated free text is never an executable result.

The runtime must not infer hidden state changes, tool calls, sends, approvals, suppressions, task completions, reservation changes, document approvals, or incident closure from prose outside the validated structured payload. If an agent writes “I confirmed the booking,” “message sent,” or “vaccines approved” in free text, but the JSON envelope does not contain a validated, policy-allowed, audited action for that operation, the application must treat the statement as non-authoritative text and apply no side effect.

## Shared result envelope

Every AI/Hermes workflow result should validate against a shared envelope plus a workflow-specific `structured_output` schema.

Baseline envelope fields:

```json
{
  "schema_name": "BookingRecommendationResult",
  "schema_version": "2026-06-11",
  "workflow_name": "booking-triage",
  "run_id": "agent-run-...",
  "event_id": "workflow-event-...",
  "subject": {
    "type": "reservation",
    "id": "res_..."
  },
  "status": "needs_human_review",
  "summary": "Short safe summary for audit/UI.",
  "structured_output": {},
  "recommended_actions": [],
  "risk_flags": [],
  "verification": [],
  "uncertainty": [],
  "missing_inputs": [],
  "approval_requirements": [],
  "human_review_reason": {
    "gate": "manager_approval",
    "reason": "Capacity exception and deposit status conflict."
  },
  "safe_log": {
    "summary": "Booking triage requires manager review.",
    "schema_errors": [],
    "ids": ["workflow-event-...", "res_..."],
    "redacted_excerpt_refs": []
  }
}
```

Recommended alignment with current Rust types:

| Envelope field | Existing anchor | Rule |
| --- | --- | --- |
| `schema_name` | `agent::OutputSchemaName` / `AgentPromptPacket.output_schema_name` | Must match the schema requested in the prompt packet. |
| `schema_version` | future schema registry | Required once any schema can evolve; versions should be date or semver-like, not implicit prompt text. |
| `workflow_name` | `agent::Name` | Must match the runtime workflow that produced the prompt packet. |
| `run_id` | runtime-generated | Required for idempotency, replay, and audit correlation. |
| `event_id` | `WorkflowEventId` | Required; output without event correlation is invalid. |
| `subject` | `WorkflowSubject` | Required; must match the prompt event subject or name an explicitly allowed derived subject. |
| `status` | `WorkflowStatus` | One of completed, needs_human_review, rejected_by_policy, needs_more_information, failed_safely. |
| `summary` | `workflow::Summary` | Short, safe, non-secret, non-PII where possible; never the sole source of actionable data. |
| `structured_output` | `WorkflowResult<T>.structured_output` | Workflow-specific typed result; may be null only when status explains why no result is usable. |
| `recommended_actions` | `WorkflowResult.recommended_actions` | Drafts/recommendations only; deterministic validators and approval gates decide side effects. |
| `risk_flags` | `WorkflowResult.risk_flags` | Required for safety, medical, incident, payment, privacy, or policy ambiguity. |
| `verification` | `WorkflowResult.verification` | Lists checked sources, trusted/missing/conflicting state, and confidence limits. |
| `uncertainty` | runtime convention | Structured uncertainty items; do not hide uncertainty in prose. |
| `missing_inputs` | runtime convention | Structured list of missing fields/sources/blockers. |
| `approval_requirements` | `policy::ReviewGate` | Every customer-facing send, exception, or unsafe/uncertain output must name the gate. |
| `human_review_reason` | `WorkflowResult.human_review_reason` | Required when status is needs_human_review or when any approval gate remains open. |
| `safe_log` | audit/runtime convention | Only safe summaries, schema errors, IDs, and redacted excerpt references. |

### Status semantics

Use `status` to tell the application what it may do next. The names below follow the current implemented Rust `WorkflowStatus` variants. If a consumer adopts the proposed names in `docs/architecture/workflow-result-envelope.md`, map `completed -> success`, `needs_more_information -> blocked`, and `failed_safely -> failed` without changing the safety rules.

- `completed`: the result is well-formed and safe as a recommendation/draft packet. This does not mean side effects are approved.
- `needs_human_review`: output is well-formed enough to persist as a review packet, but a named human gate must approve before side effects.
- `rejected_by_policy`: output is well-formed but conflicts with deterministic policy or requested a forbidden action.
- `needs_more_information`: output is well-formed but cannot answer because required inputs are missing.
- `failed_safely`: the runtime failed, validation failed after retry, or uncertainty was too high to trust any structured result.

## Schema strategy

Use a two-layer JSON Schema strategy:

1. Shared envelope schema: required for every workflow. It validates correlation, workflow identity, status, safe logging shape, uncertainty/missing-input structure, approval requirements, and the prohibition on free-text action surfaces.
2. Workflow-specific schema: embedded under `structured_output` and keyed by `schema_name` + `schema_version`. It validates the domain-specific output, for example booking recommendations, vaccine document extraction/review packets, customer-message drafts, or incident summaries.

Baseline conventions:

- Set `additionalProperties: false` at the envelope and workflow-specific object levels unless a field is explicitly a redacted metadata map.
- Use enums for statuses, review gates, action kinds, source-trust states, recipient roles, severity, and approval states.
- Use semantic IDs (`location_id`, `reservation_id`, `pet_id`, `customer_id`, `document_id`, `incident_id`, `workflow_event_id`) instead of raw provider objects.
- Represent uncertainty as structured entries: `{ "field": "deposit_status", "state": "conflicting", "reason": "reservation says unpaid, payment adapter says paid", "required_resolution": "payment_reconciliation_review" }`.
- Represent missing inputs as structured entries: `{ "field": "rabies_expiration", "required_source": "licensed vet vaccine proof", "blocks": ["eligibility_decision"] }`.
- Represent approval requirements as structured entries: `{ "gate": "manager_approval", "reason": "capacity exception", "before": ["status_update", "customer_send"] }`.
- Keep raw OCR text, raw email bodies, raw provider JSON, payment payloads, tokens, signatures, internal chain-of-thought, and unnecessary PII out of the result envelope. If excerpts are needed, store redacted excerpts with source references.
- Require `source_refs` inside workflow-specific results for facts that affect review gates, messages, status recommendations, or tasks.

## Validation and failure-handling state machine

The application-owned runtime should use this state machine for every AI/Hermes run:

```text
1. Build typed prompt packet
   -> include workflow_name, event_id, subject, allowed_actions, policy_context, output_schema_name, schema_version, and data-minimized inputs.

2. Invoke AI/Hermes worker
   -> receive raw model/runtime output.

3. Parse JSON only
   -> if not parseable JSON: no side effects; record safe validation failure; retry once with parse error context.

4. Validate shared envelope
   -> if envelope invalid: no side effects; record schema errors; retry once with validation error context.

5. Validate workflow-specific structured_output
   -> if schema invalid: no side effects; record schema errors; retry once with validation error context.

6. Check correlation and policy invariants
   -> schema_name, workflow_name, event_id, subject, allowed_actions, and review gates must match the prompt packet and deterministic policy context.
   -> if mismatched or forbidden: no side effects; status becomes rejected_by_policy or failed_safely.

7. Run deterministic domain validators
   -> verify source refs, trusted-source states, idempotency keys, policy snapshots, and review gates.
   -> if unsafe/uncertain/missing approval: persist review packet only and escalate.

8. Persist safe result packet
   -> append audit/workflow event with safe summary, IDs, schema version, validation outcome, and redacted excerpts.

9. Apply side effects only through approved paths
   -> only after validated structure + deterministic policy approval + required human approval + idempotent tool command.
```

Retry behavior:

- Retry exactly once for parse/schema validation failures.
- The retry prompt may include only safe validation error context: JSON pointer, schema keyword, expected type/enum, missing required field names, and redacted excerpts if necessary.
- The retry prompt must not include secrets, raw provider payloads, raw payment data, unredacted medical/customer text, or chain-of-thought requests.
- If the retry output is still malformed, invalid, mismatched, unsafe, or uncertain, mark the run `failed_safely` and escalate to a human/engineering owner.

Side-effect rule:

- Before validation succeeds, the runtime may write only a validation-failure audit record and safe log line. It must not create tasks, send messages, update reservations, change document status, apply payments, close incidents, or call external provider mutators.
- After validation succeeds, recommended actions are still drafts/recommendations until deterministic policy and approval gates permit execution.

## Escalation rules

Escalate to a human when any of these are true:

- Parsing or schema validation fails twice.
- The model output requests, implies, or claims an action outside `PolicyContext.allowed_actions`.
- The output references the wrong event, subject, workflow, schema, customer, pet, reservation, document, or incident.
- Source facts are missing, stale, untrusted, or conflicting for a decision that affects safety, eligibility, payment, capacity, incident handling, or customer-facing communication.
- The output is well-formed but has `needs_human_review`, `needs_more_information`, or non-empty `approval_requirements`.
- The output is confident in prose but the structured fields contain uncertainty, missing inputs, or absent source refs.
- The result would affect medical/care instructions, vaccine eligibility, group-play eligibility, incidents, payment/refund/waiver/discount/deposit state, booking confirmation/cancellation, capacity exceptions, or sensitive customer messages.
- The output contains unredacted secrets, raw payment/provider payloads, unnecessary PII, or policy-prohibited content.

Suggested escalation owners:

| Situation | Escalation owner |
| --- | --- |
| Routine missing intake/document/payment information | Front desk / staff review |
| Vaccine/document ambiguity, medical/care-source uncertainty | Medical document review / manager depending on local policy |
| Capacity, exception, incident, sensitive customer language, behavior/safety concern | Manager |
| Payment conflicts, refunds, waivers, discounts, forfeitures, provider mismatches | Manager and/or payment reconciliation owner |
| Schema mismatch, idempotency ambiguity, provider mapping bug, unable to distinguish trusted from raw input | Engineering/integration owner |
| Possible secret exposure, raw payment payload exposure, legal/regulatory threat | Privacy/security/legal/compliance owner |

## Logging and audit minimization

Logs may contain:

- `run_id`, `event_id`, `workflow_name`, `schema_name`, `schema_version`, subject IDs, and result status.
- Safe one-line summary.
- Schema validation errors: JSON pointer, expected type/enum/required field, actual high-level type, not raw value if sensitive.
- Deterministic policy outcome: allowed, denied, review required, or failed safely.
- Redacted excerpt references, for example `document_excerpt:doc_123:lines_4_6:redacted`, not full raw document text.
- Human escalation gate and reason.

Logs must not contain:

- Raw card/payment data, payment tokens, signed webhook payloads, provider secrets, API keys, or credentials.
- Unredacted owner contact data beyond role-appropriate IDs.
- Raw vaccine/OCR documents, medical notes, care notes, incident narratives, staff notes, customer messages, or email bodies unless explicitly redacted and necessary.
- Internal model chain-of-thought or hidden reasoning.
- Free-text claims that an action occurred unless matched by a validated/audited tool result.

## Example workflow schemas

The examples below are illustrative schema shapes, not final implementation files. Field names should be promoted into module-owned Rust types when behavior depends on them.

### Booking recommendation

Use for booking triage, waitlist/offering packets, and check-in readiness recommendations. It may recommend status or tasks, but it may not confirm, cancel, overbook, waive deposits, or send customer messages by itself.

Required workflow-specific fields:

```json
{
  "recommendation": "offer_pending_review",
  "reservation_id": "res_123",
  "location_id": "loc_1",
  "service_kind": "boarding",
  "requested_dates": {
    "start": "2026-07-02",
    "end": "2026-07-06"
  },
  "source_trust_state": "conflicting",
  "availability_state": {
    "state": "limited",
    "source_refs": ["availability_snapshot:av_456"]
  },
  "eligibility_state": {
    "state": "needs_review",
    "reasons": ["rabies_expiration_missing"],
    "source_refs": ["pet:pet_123", "document:doc_789"]
  },
  "deposit_state": {
    "state": "conflicting",
    "source_refs": ["reservation:res_123", "payment_projection:pay_456"]
  },
  "status_suggestion": {
    "target": "special_review",
    "transition_intent": "needs_manager_review",
    "reason": "Limited capacity and unresolved vaccine/deposit state."
  },
  "staff_tasks": [
    {
      "kind": "document_review",
      "title": "Review missing rabies expiration for Luna",
      "assignment": "front_desk_or_medical_review"
    },
    {
      "kind": "customer_follow_up",
      "title": "Prepare boarding availability follow-up draft",
      "assignment": "front_desk"
    }
  ],
  "draft_message": null,
  "source_refs": ["workflow_event:evt_123", "reservation:res_123", "policy_snapshot:pol_7"],
  "approval_requirements": [
    {
      "gate": "manager_approval",
      "reason": "Capacity exception and payment conflict before offer/confirmation.",
      "before": ["reservation_status_update", "customer_message_send"]
    }
  ]
}
```

Safe handling:

- `completed` is valid only for a recommendation packet that requires no unresolved review before being displayed internally.
- If vaccine, capacity, deposit, or hard-stop inputs are missing/conflicting, use `needs_human_review` or `needs_more_information`.
- Any customer-facing availability or booking message remains a draft until approved by a customer-message or manager gate.

### Document review

Use for vaccine/document extraction and review routing. It may extract dates/names and route uncertainty, but it may not finally approve uncertain medical documents.

Required workflow-specific fields:

```json
{
  "document_id": "doc_789",
  "pet_id": "pet_123",
  "document_type": "vaccine_record",
  "extraction_state": "needs_human_review",
  "extracted_records": [
    {
      "vaccine_name": "rabies",
      "administered_on": "2025-06-01",
      "expires_on": null,
      "confidence": "medium",
      "source_excerpt_ref": "doc_789:page_1:line_12:redacted",
      "uncertainty": [
        {
          "field": "expires_on",
          "state": "missing",
          "reason": "Expiration date not visible in OCR."
        }
      ]
    }
  ],
  "trusted_source_state": "unverified",
  "policy_checks": [
    {
      "policy_ref": "vaccine_policy:loc_1:current",
      "result": "cannot_determine",
      "reason": "Missing expiration date and source-license verification."
    }
  ],
  "recommended_actions": [
    {
      "kind": "internal_task",
      "task_kind": "document_review",
      "title": "Verify rabies expiration for Luna"
    }
  ],
  "source_refs": ["document:doc_789", "pet:pet_123", "policy_snapshot:vax_loc_1"]
}
```

Safe handling:

- If OCR is unclear, source verification is missing, or dates conflict, route to `medical_document_review` or manager review.
- Never convert extracted text directly into final eligibility or care permission without deterministic policy and review when required.
- Store raw OCR outside the AI result envelope; log only redacted excerpt references.

### Messaging draft

Use for customer update drafts, intake follow-ups, payment reminders, and incident-response drafts. It may draft text and review notes, but it may not send unless an approved deterministic send path has fixed recipient, facts, template/send condition, and approval gate.

Required workflow-specific fields:

```json
{
  "draft_id": "draft_456",
  "recipient": {
    "role": "customer",
    "customer_id": "cust_123",
    "contact_channel_ref": "preferred_channel:cust_123"
  },
  "message_kind": "customer_follow_up",
  "sensitivity": "payment_sensitive",
  "draft_body": "Hi Jordan — we are reviewing Luna's upcoming boarding request and need one vaccine detail before our team can finalize next steps. Please upload the current rabies record when convenient.",
  "facts_used": [
    {
      "field": "pet_name",
      "value_ref": "pet:pet_123:name",
      "source_trust_state": "verified"
    },
    {
      "field": "missing_requirement",
      "value_ref": "document_review:doc_789:rabies_expiration_missing",
      "source_trust_state": "needs_review"
    }
  ],
  "claims_not_made": [
    "booking confirmed",
    "payment accepted",
    "vaccine approved",
    "refund promised"
  ],
  "review_notes": [
    "Customer-facing send requires staff approval because the vaccine record is unresolved."
  ],
  "send_authorization": {
    "state": "not_authorized",
    "required_gate": "customer_message_approval",
    "reason": "Draft only; staff must verify facts and approve recipient/body."
  },
  "source_refs": ["workflow_event:evt_456", "pet:pet_123", "document:doc_789"]
}
```

Safe handling:

- `draft_body` is never a send command.
- If the draft mentions medical, incident, behavior, payment, refund, cancellation, policy exception, or sensitive tone, set `sensitivity` and require the relevant approval gate.
- The send worker must ignore any prose such as “send this now” unless `send_authorization.state` is approved by deterministic policy and human approval records.

### Incident summary

Use for incident escalation, manager packets, and owner-message drafts. It may summarize facts and create review tasks, but it may not diagnose, assign fault, hide concerning facts, close incidents, or send owner messages without approval.

Required workflow-specific fields:

```json
{
  "incident_id": "inc_123",
  "pet_id": "pet_123",
  "reservation_id": "res_123",
  "summary_type": "manager_packet",
  "severity_assessment": {
    "level": "requires_manager_review",
    "basis": ["staff_note:note_456", "care_task:task_789"],
    "not_a_diagnosis": true
  },
  "known_facts": [
    {
      "fact": "Staff observed limping after group play.",
      "source_ref": "staff_note:note_456:redacted",
      "trust_state": "staff_reported"
    }
  ],
  "missing_inputs": [
    {
      "field": "manager_disposition",
      "required_source": "manager_review_record",
      "blocks": ["incident_closure", "owner_message_send"]
    },
    {
      "field": "care_follow_up_outcome",
      "required_source": "care_task_completion_evidence",
      "blocks": ["final_summary"]
    }
  ],
  "recommended_actions": [
    {
      "kind": "internal_task",
      "task_kind": "incident_follow_up",
      "title": "Manager review for Luna limping observation"
    },
    {
      "kind": "draft_message",
      "channel": "customer_preferred",
      "body_state": "draft_for_manager_review"
    }
  ],
  "owner_message_draft": {
    "body": "Hi Jordan — our team noticed Luna favoring a paw during play today. A manager is reviewing the care notes now and we will follow up with the next steps.",
    "approval_required": "manager_approval"
  },
  "source_refs": ["incident:inc_123", "pet:pet_123", "staff_note:note_456"]
}
```

Safe handling:

- If injury, illness, aggression, medication, allergy, legal/regulatory threat, or customer-sensitive language appears, escalate to manager and/or medical/legal/privacy owner.
- Do not let AI mark `incident_closed`, suppress owner notification, or send a final owner message without a validated manager approval record.
- Logs should store safe summary and IDs, not raw incident narrative.

## Implementation checklist

Before enabling any workflow in production, require:

- A registered shared envelope schema and workflow-specific schema with version.
- Prompt packet includes `output_schema_name`, `schema_version`, `WorkflowEventId`, `WorkflowSubject`, `PolicyContext`, and allowed actions.
- Parser accepts JSON only; no Markdown/prose fallback.
- Validator rejects unknown fields, wrong schema/workflow/event/subject, missing required fields, invalid enums, and malformed IDs.
- Runtime retries once with safe validation error context.
- Failed retry produces `failed_safely`, no side effects, and human/engineering escalation.
- Deterministic validators check policy, source trust, approval gates, idempotency, and side-effect eligibility after schema validation.
- Side-effect executors accept only validated typed commands, never free text.
- Audit/logging is minimized to safe summary, schema errors, IDs, schema version, policy outcome, approval gate, and redacted excerpts.
- Tests cover malformed JSON, schema mismatch, forbidden action in prose, forbidden action in structured actions, wrong subject/event, missing approval gate, unsafe uncertainty, successful review packet persistence, retry success, and retry failure escalation.
