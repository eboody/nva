# Workflow result envelope

Purpose: define the result envelope returned by agents and workflow handlers. The envelope is a semantic boundary contract: it tells operators, deterministic policy validators, and adapter layers what the workflow learned, what it already did, what remains only a recommendation or draft, and what review gate is still required.

Status: proposed contract. This document does not authorize production side effects. Until a later implementation card encodes and validates the contract in Rust, use this as the canonical schema proposal for workflow-result consumers.

## Core principle

A workflow result is not an execution receipt unless it explicitly records an already-executed action in the execution/audit record for that action. The result envelope may contain structured extraction, recommended next actions, draft messages, proposed internal tasks, risk flags, and verification evidence, but those are review packets by default.

The safe default is:

- Agents may read, extract, summarize, draft, recommend, flag risk, and propose internal tasks.
- Deterministic policy validators decide whether an action is allowed, whether it needs review, and whether it can be sent to a tool.
- Adapter/tool layers execute only approved commands, then write their own execution/audit records.
- Customer-facing messages remain drafts unless the event and policy explicitly permit outbound sending and required approvals are already satisfied.

## Proposed schema

```rust
pub struct WorkflowResult<TStructuredOutput> {
    pub status: WorkflowResultStatus,
    pub summary: workflow::Summary,
    pub structured_output: Option<TStructuredOutput>,
    pub recommended_actions: Vec<workflow::RecommendedAction>,
    pub draft_messages: Vec<workflow::DraftMessage>,
    pub tasks_to_create: Vec<workflow::TaskDraft>,
    pub risk_flags: Vec<workflow::RiskFlag>,
    pub verification: workflow::VerificationRecord,
    pub human_review_reason: Option<workflow::ReviewReason>,
}
```

Wire-format equivalent:

```json
{
  "status": "needs_human_review",
  "summary": "Short operator-safe summary of the result.",
  "structured_output": {},
  "recommended_actions": [],
  "draft_messages": [],
  "tasks_to_create": [],
  "risk_flags": [],
  "verification": {
    "evidence": [],
    "unchecked_sources": [],
    "redactions": [],
    "confidence": "source_backed"
  },
  "human_review_reason": "Required when status or policy requires review."
}
```

Naming note: this proposal separates `draft_messages` and `tasks_to_create` from general `recommended_actions` because message drafts and task drafts have stricter lifecycle and review semantics. A future Rust migration can keep compatibility by mapping the existing `RecommendedAction::DraftMessage` and `RecommendedAction::InternalTask` variants into the dedicated fields at the boundary, then deprecating the overloaded variants after consumers move.

## Status enum semantics

Use status values to describe the result's disposition, not the action's execution state.

| Status | Meaning | Safe side-effect behavior |
| --- | --- | --- |
| `success` | The workflow completed the allowed analysis or deterministic operation and no policy/human review is required by the result itself. | Does not imply external side effects occurred. Consumers may auto-advance only deterministic, pre-approved, non-sensitive internal paths. Any provider mutation, customer send, payment movement, booking acceptance, eligibility approval, or care/safety decision still needs a separately approved command/audit record. |
| `needs_human_review` | The workflow produced useful output, but policy, uncertainty, sensitivity, or action type requires a human before execution or customer-facing use. | No external send/mutation/payment/care completion may occur from this result. Route to the named review gate with `human_review_reason`, evidence, drafts, and recommendations. |
| `blocked` | The workflow cannot safely produce a complete result because required source data, credentials, policy, adapter availability, or approvals are missing. | No side effects except optional creation of a review/blocker task when policy permits task drafting. Include missing prerequisites in `human_review_reason` or risk flags. |
| `failed` | The workflow attempted to process the event but failed safely. Partial output is absent or explicitly marked incomplete. | No side effects. Preserve failure evidence without secrets; retry only through normal workflow orchestration. |
| `no_action` | The event was valid but did not require any action, draft, task, or review. | No side effects. Verification should explain why the event was intentionally ignored or already satisfied. |

Mapping from current Rust names, if needed during migration:

- `WorkflowStatus::Completed` -> `success`
- `WorkflowStatus::NeedsHumanReview` -> `needs_human_review`
- `WorkflowStatus::NeedsMoreInformation` -> `blocked`
- `WorkflowStatus::FailedSafely` -> `failed`
- `WorkflowStatus::RejectedByPolicy` -> usually `no_action` when policy denies all action, or `needs_human_review` when an operator must review the denial/customer impact.

## Field definitions

### `status`

Required enum. It describes the safe disposition of the workflow result. It must never be used as a proxy for permission to execute side effects.

Invariants:

- `needs_human_review` requires `human_review_reason`.
- `blocked` requires either `human_review_reason` or a blocking `risk_flags` entry that names the missing prerequisite.
- `failed` requires verification evidence sufficient for debugging without exposing secrets.
- `no_action` should have empty `recommended_actions`, `draft_messages`, and `tasks_to_create` unless a policy explicitly allows an informational audit/review task.

### `summary`

Required short operator-safe summary. It should state what was learned and why the workflow chose the status. It should not include unnecessary PII, payment secrets, raw provider payloads, medical details beyond the intended operator audience, or customer-facing wording unless the recipient context is explicitly internal and authorized.

### `structured_output`

Optional typed payload owned by the specific workflow. Examples: vaccine extraction fields, booking readiness packet, daily-update evidence summary. The payload is data or decision support, not authority.

Invariants:

- Boundary/provider strings must be promoted into semantic domain values before policy behavior branches on them.
- Confidence scores are not authority. If a fact changes eligibility, medical/care status, payment status, booking status, capacity, or customer-facing copy, the structured output must preserve source evidence and review state.
- When extraction is incomplete or conflicting, represent unknown/conflict explicitly rather than omitting the field in a way consumers could read as clear.

### `recommended_actions`

List of proposed next actions that have not been executed. Recommended actions are instructions to a human reviewer or deterministic policy runner, not receipts.

Recommended action examples:

- request human review by a specific gate/role;
- suggest reservation status transition with typed target, intent, and reason;
- suggest play eligibility review;
- recommend provider lookup, reconciliation, or policy update;
- recommend suppressing or revising a draft because of sensitivity.

Invariants:

- A recommended action must never claim completion. Use action names such as `SuggestReservationStatus`, `RequestHumanReview`, or `PrepareProviderLookup`, not `ReservationConfirmed` or `MessageSent`.
- Every recommended provider mutation, booking decision, payment movement, eligibility/care decision, customer-facing send, or exception must carry either a review gate or a policy decision reference proving review is not required.
- If an action has already been executed by a tool, it belongs in that tool's execution result/audit event, not in `recommended_actions`.

### `draft_messages`

Customer-facing, staff-facing, manager-facing, or internal messages drafted by the workflow. Drafts are content artifacts, not send commands.

Suggested shape:

```rust
pub struct DraftMessage {
    pub audience: MessageAudience,
    pub channel: Option<message::Channel>,
    pub subject: Option<message::Subject>,
    pub body: message::Body,
    pub source_refs: Vec<workflow::EvidenceRef>,
    pub review_gate: Option<policy::ReviewGate>,
    pub send_policy: DraftSendPolicy,
}

pub enum DraftSendPolicy {
    DraftOnly,
    EligibleForDeterministicSend { policy_ref: policy::Id },
    RequiresApproval { gate: policy::ReviewGate },
}
```

Invariants:

- Default `send_policy` is `DraftOnly`.
- Customer-facing drafts remain drafts unless the workflow event, current policy context, recipient, template/copy, and required approvals explicitly permit outbound sending.
- Sensitive content involving health, medication, allergy, behavior, incident, safety, payment, refund/waiver/discount/forfeit, legal/compliance, booking denial, eligibility refusal, or policy exceptions requires review even when the draft is source-backed.
- Drafts should include source references and uncertainty markers; they should not hide material uncertainty to make copy sound smoother.

### `tasks_to_create`

Draft internal task records proposed by the workflow. They are separate from `recommended_actions` because task creation may be auto-drafted safely but auto-creating live staff work at scale requires approved trigger/kind/priority/assignee policy.

Suggested shape:

```rust
pub struct TaskDraft {
    pub kind: operations::StaffTaskKind,
    pub title: workflow::task::Title,
    pub body: workflow::task::Body,
    pub assignment: operations::StaffTaskAssignment,
    pub priority: operations::StaffTaskPriority,
    pub source: operations::StaffTaskSource,
    pub due_basis: Option<TaskDueBasis>,
    pub evidence_refs: Vec<workflow::EvidenceRef>,
    pub creation_policy: TaskCreationPolicy,
}

pub enum TaskCreationPolicy {
    DraftOnly,
    AutoCreateAllowed { policy_ref: policy::Id },
    RequiresReview { gate: policy::ReviewGate },
}
```

Invariants:

- Task drafts are not live tasks until a deterministic task-creation policy or human approval creates them.
- Completion authority is never implied by task creation. Care, medication, feeding, incident, payment, document, checkout, and customer-message tasks require authorized human/tool evidence before completion.
- Assignment and priority rationale should be source-backed and reviewable.

### `risk_flags`

Typed warnings that the consumer must display or route. Risk flags should be short semantic values, not free-form dumping grounds.

Examples:

- `missing_required_vaccine_proof`
- `unverified_veterinary_source`
- `conflicting_pet_profile_facts`
- `customer_message_requires_review`
- `payment_sensitive_content`
- `capacity_or_ratio_exception`
- `medical_or_medication_ambiguity`
- `provider_payload_unverified`
- `raw_pii_redacted`

Invariants:

- Any flag that changes routing should eventually become a typed enum or policy reason, not remain an opaque string.
- Do not use risk flags as hidden approvals or denials. They are evidence/routing signals only.

### `verification`

Required evidence and source-reference record. Verification explains what was checked, what was not checked, which sources were used, and what redaction/minimization was applied.

Suggested shape:

```rust
pub struct VerificationRecord {
    pub evidence: Vec<EvidenceRef>,
    pub unchecked_sources: Vec<UncheckedSource>,
    pub redactions: Vec<RedactionNote>,
    pub confidence: VerificationConfidence,
}

pub struct EvidenceRef {
    pub source_type: EvidenceSourceType,
    pub source_id: EvidenceSourceId,
    pub observed_at: DateTime<Utc>,
    pub field_refs: Vec<EvidenceFieldRef>,
    pub summary: workflow::VerificationNote,
}

pub enum VerificationConfidence {
    SourceBacked,
    PartiallySourceBacked,
    ConflictingSources,
    MissingRequiredSource,
    UnverifiedExtraction,
}
```

Evidence/source-reference rules:

- Evidence should cite stable source IDs, document IDs, provider record IDs, workflow event IDs, audit event IDs, policy IDs, or field references appropriate for the operator audience.
- Evidence should not leak secrets, raw payment tokens, signed webhook material, raw provider payloads, or unnecessary PII.
- Verification may include sensitive operational facts only when the consuming operator audience is intended and authorized for those facts.
- Customer-facing copy should cite or paraphrase only customer-safe evidence; internal verification can be richer but still minimized.
- When source records conflict, preserve the conflict and route for review instead of choosing the most plausible value.

### `human_review_reason`

Optional only for statuses and policies that truly do not require human review. This field is the human-readable reason that the result is not safe to execute or send without review.

Required when:

- `status` is `needs_human_review`;
- policy context has any `required_reviews` not already satisfied;
- any `draft_messages` have `RequiresApproval` or sensitive customer-facing content;
- any `tasks_to_create` have `RequiresReview` and auto-creation is not approved;
- any recommended action would mutate provider state, send an outbound message, move money, approve/deny booking or eligibility, resolve medical/care ambiguity, override policy, or expose sensitive content;
- `status` is `blocked` because a human decision, credential, source, or approved policy is missing.

## Global invariants

1. Distinguish proposed from executed. `recommended_actions`, `draft_messages`, and `tasks_to_create` are proposed/draft artifacts unless an external execution/audit record says otherwise.
2. Status is not authority. `success` means the handler completed safely; it does not grant provider mutation, message send, payment movement, or care/eligibility authority.
3. Review reasons must be explicit. Any required review must have a human-readable reason and enough evidence to act.
4. Draft messages stay drafts by default. Sending requires an event/policy that explicitly permits outbound sending plus satisfied approvals.
5. Verification is source-referenced and minimized. Record enough evidence to audit the result without leaking secrets or unnecessary PII beyond the intended operator audience.
6. Structured output is typed and semantic. Avoid raw status strings, booleans, arbitrary provider JSON, and unmodeled confidence-as-authority. Promote meaningful distinctions into enums/newtypes before behavior depends on them.
7. Risks route; they do not decide. Risk flags should make policy/human review visible, not silently approve or deny outcomes.
8. Human review is sticky until cleared. A downstream policy runner may clear it only by referencing an approved policy decision or explicit staff/manager approval audit event.

## Example: vaccine extraction

```json
{
  "status": "needs_human_review",
  "summary": "Extracted a rabies vaccine date from an uploaded document, but the veterinary source and expiration date need review before eligibility changes.",
  "structured_output": {
    "pet_id": "pet_123",
    "document_id": "doc_vaccine_upload_456",
    "extracted_vaccines": [
      {
        "name": "rabies",
        "administered_on": "2026-05-20",
        "expires_on": null,
        "source_state": "veterinary_source_unverified",
        "extraction_state": "partial"
      }
    ],
    "eligibility_effect": "not_applied"
  },
  "recommended_actions": [
    {
      "type": "request_human_review",
      "gate": "vaccine_document_review",
      "reason": "Expiration date missing and vet source not verified."
    }
  ],
  "draft_messages": [],
  "tasks_to_create": [
    {
      "kind": "DocumentReview",
      "title": "Review vaccine upload for pet_123",
      "assignment": "Role(FrontDesk)",
      "priority": "High",
      "creation_policy": "RequiresReview(vaccine_document_review)",
      "evidence_refs": ["document:doc_vaccine_upload_456"]
    }
  ],
  "risk_flags": [
    "unverified_veterinary_source",
    "missing_vaccine_expiration"
  ],
  "verification": {
    "evidence": [
      {
        "source_type": "document",
        "source_id": "doc_vaccine_upload_456",
        "field_refs": ["ocr.page_1.line_12", "ocr.page_1.line_14"],
        "summary": "Rabies label and administered date found; expiration field absent."
      }
    ],
    "unchecked_sources": ["veterinary license/source registry", "current location vaccine policy snapshot"],
    "redactions": ["owner address and phone omitted from operator summary"],
    "confidence": "partially_source_backed"
  },
  "human_review_reason": "Vaccine eligibility cannot change until source and expiration are reviewed."
}
```

Notes:

- The workflow extracted useful structure but did not approve the vaccine.
- The task is a draft/review-routed task, not proof of staff completion.
- No customer message is drafted because the uncertainty is internal-review first.

## Example: booking triage

```json
{
  "status": "needs_human_review",
  "summary": "Booking request is close to ready, but missing vaccine proof and a capacity/ratio check block confirmation.",
  "structured_output": {
    "reservation_id": "res_789",
    "requested_service": "Boarding",
    "requested_dates": { "arrival": "2026-07-03", "departure": "2026-07-07" },
    "readiness": "blocked_for_review",
    "resolved_facts": ["customer_profile_present", "pet_profile_present", "deposit_policy_snapshot_present"],
    "missing_or_blocking_facts": ["required_vaccine_proof", "holiday_capacity_review"],
    "suggested_status": {
      "status": "SpecialReview",
      "intent": "RequestMedicalReview",
      "reason": "Required vaccine proof missing for holiday boarding request."
    }
  },
  "recommended_actions": [
    {
      "type": "suggest_reservation_status",
      "target": "reservation",
      "status": "SpecialReview",
      "intent": "RequestMedicalReview",
      "review_gate": "staff_review"
    },
    {
      "type": "request_human_review",
      "gate": "manager_capacity_review",
      "reason": "Holiday period capacity/ratio decision is unresolved."
    }
  ],
  "draft_messages": [
    {
      "audience": "customer",
      "channel": "email",
      "body": "Thanks for the request. We are reviewing the stay dates and still need updated vaccine information before we can confirm availability.",
      "send_policy": "RequiresApproval(staff_review)",
      "source_refs": ["reservation:res_789", "policy:vaccine_policy_current"]
    }
  ],
  "tasks_to_create": [
    {
      "kind": "DocumentReview",
      "title": "Collect required vaccine proof for res_789",
      "assignment": "Role(FrontDesk)",
      "priority": "High",
      "creation_policy": "DraftOnly",
      "evidence_refs": ["reservation:res_789"]
    },
    {
      "kind": "CheckInPrep",
      "title": "Manager capacity review for holiday boarding request res_789",
      "assignment": "Role(Manager)",
      "priority": "High",
      "creation_policy": "RequiresReview(manager_capacity_review)",
      "evidence_refs": ["reservation:res_789", "operating_day:2026-07-03"]
    }
  ],
  "risk_flags": [
    "missing_required_vaccine_proof",
    "capacity_or_ratio_exception",
    "customer_message_requires_review"
  ],
  "verification": {
    "evidence": [
      {
        "source_type": "reservation",
        "source_id": "res_789",
        "field_refs": ["requested_service", "arrival_date", "departure_date", "pet_id", "customer_id"],
        "summary": "Booking request has service, dates, pet, and customer identifiers."
      },
      {
        "source_type": "policy",
        "source_id": "vaccine_policy_current",
        "field_refs": ["required_vaccines"],
        "summary": "Policy requires vaccine proof before confirmation."
      }
    ],
    "unchecked_sources": ["live room availability", "holiday staffing ratio snapshot"],
    "redactions": ["customer contact details excluded from summary"],
    "confidence": "partially_source_backed"
  },
  "human_review_reason": "Reservation confirmation and customer send are blocked by missing vaccine proof and unresolved holiday capacity review."
}
```

Notes:

- `recommended_actions` suggest a status transition and review gates; they do not update Gingr/provider state.
- The customer copy is a draft requiring approval.
- Capacity review is separate from vaccine document collection because different authority may apply.

## Example: daily update

```json
{
  "status": "needs_human_review",
  "summary": "Prepared a source-backed daily update draft from completed care notes; staff message review is required before customer send.",
  "structured_output": {
    "reservation_id": "res_456",
    "pet_id": "pet_123",
    "care_evidence_summary": {
      "feeding": "morning meal recorded complete",
      "play": "individual yard time recorded complete",
      "medication": "no medication tasks scheduled",
      "incidents": "none in checked notes"
    },
    "customer_update_state": "draft_ready_for_staff_review"
  },
  "recommended_actions": [
    {
      "type": "request_human_review",
      "gate": "staff_message_review",
      "reason": "Customer-facing daily update draft requires staff approval before sending."
    }
  ],
  "draft_messages": [
    {
      "audience": "customer",
      "channel": "portal_message",
      "body": "Today went smoothly. Morning meal was completed, and your pet enjoyed individual yard time. No concerns were recorded in the checked care notes.",
      "send_policy": "RequiresApproval(staff_message_review)",
      "source_refs": ["task:feed_001", "task:play_002", "reservation:res_456"]
    }
  ],
  "tasks_to_create": [],
  "risk_flags": ["customer_message_requires_review"],
  "verification": {
    "evidence": [
      {
        "source_type": "staff_task",
        "source_id": "feed_001",
        "field_refs": ["status", "completion_evidence"],
        "summary": "Morning feeding task was completed by authorized staff."
      },
      {
        "source_type": "staff_task",
        "source_id": "play_002",
        "field_refs": ["status", "completion_evidence"],
        "summary": "Individual yard time task was completed by authorized staff."
      }
    ],
    "unchecked_sources": ["media/photo review not requested", "provider portal send state"],
    "redactions": ["internal staff names omitted from customer draft"],
    "confidence": "source_backed"
  },
  "human_review_reason": "Staff must approve the customer-facing daily update before any portal send."
}
```

Notes:

- The workflow succeeded at preparing a draft, but the result status is `needs_human_review` because the policy requires staff approval before customer send.
- The example avoids claiming the message was sent. A later messaging adapter execution record must record actual send status.
- If a future location adopts a deterministic pre-approved daily-update send path, the result may become `success` only when recipient, facts, template/copy, event type, and approval policy are all explicitly satisfied and cited.

## Implementation implications

- Add dedicated `DraftMessage`, `TaskDraft`, `VerificationRecord`, and `EvidenceRef` domain types before workflow consumers branch on these fields.
- Promote routing-relevant `risk_flags` into enums/reasons once policy or queues depend on them.
- Keep `RecommendedAction` for non-message, non-task action proposals or split it into action-specific modules as behavior grows.
- Add constructors or validators that enforce status/review invariants, especially `needs_human_review` requiring `human_review_reason` and draft sends defaulting to `DraftOnly`.
- Keep executed action receipts out of this envelope unless they are references to immutable audit/tool-result records; do not mix recommendations with execution logs.
