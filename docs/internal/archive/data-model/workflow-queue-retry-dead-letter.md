# Workflow queue, retry, and dead-letter model

Purpose: define the durable queue model for processing `WorkflowEvent` packets into safe workflow results, review tasks, drafts, and approved provider writes.

Status: draft data-model contract. This document does not authorize live customer messages, reservation mutations, payment/refund/waiver actions, medical/behavior/eligibility decisions, or provider writes. It defines how those jobs must be queued, retried, failed, reviewed, and re-run without bypassing approval gates.

## Source anchors

- `domain/src/workflow.rs` defines `WorkflowEvent`, `WorkflowEventType`, `PolicyContext`, `AllowedAction`, `WorkflowResult`, `WorkflowStatus`, and `RecommendedAction`.
- `docs/workflows/staff-operations-parts/inputs.md` establishes the product posture: AI/workflows may read, summarize, detect gaps, draft internal tasks/messages, and recommend review gates; manual approval is required for medical/safety/payment/eligibility/capacity-exception/customer-message authority.
- Parent handoff `t_81dc80d3` says every workflow event has a primary subject and `LocationId`, external provider payloads are boundary data until semantically mapped, customer-message sends are draft/approval-gated by default, and provider writes must remain bounded and audited.

## Design goals

1. Every inbound workflow event is durable, idempotently enqueued, and traceable to source evidence.
2. Retry handles transient infrastructure/provider failures only. It never weakens policy, review, or approval requirements.
3. Dead-lettered jobs are visible to humans with safe explanations, redacted error details, and explicit next actions.
4. Customer-message/provider-write side effects are idempotent, reviewable, and executed only from approved action records.
5. Operator re-run controls create new attempts under current policy rather than silently replaying stale unsafe decisions.

## Queue table sketch

Table name: `workflow_queue`.

| Field | Type sketch | Required | Purpose / constraints |
| --- | --- | --- | --- |
| `queue_id` | UUID / `WorkflowQueueId` | yes | Stable queue row ID. Not the same as `WorkflowEventId`; one event may fan out into multiple workflow jobs. |
| `event_id` | UUID / `WorkflowEventId` | yes | Domain event being processed. Foreign key to durable workflow-event store. |
| `location_id` | UUID / `LocationId` | yes | Required location policy scope; workers must not process a row without it. |
| `subject_kind` | enum | yes | `Customer`, `Pet`, `Reservation`, or `External`. Mirrors `WorkflowSubject` for operator filtering and idempotency scoping. |
| `subject_id` | string/UUID | yes | Semantic ID or external boundary ID. For external subjects include provider namespace in `provider`. |
| `provider` | nullable string | no | Source provider for external/webhook rows or provider-write jobs. |
| `event_type` | enum/string | yes | Snapshot of `WorkflowEventType` for indexing and operator display. Canonical behavior still reads the event record. |
| `workflow_kind` | enum | yes | Semantic processor name, e.g. `BookingTriage`, `DocumentReview`, `DailyUpdateDraft`, `IncidentFollowUp`, `ApprovedProviderWrite`, `ApprovedCustomerSend`. |
| `status` | enum | yes | `Queued`, `Claimed`, `Succeeded`, `WaitingForHuman`, `RetryScheduled`, `DeadLettered`, `Cancelled`, `Superseded`. See state machine below. |
| `priority` | integer | yes | Higher value claimed first within location/queue partition. Must not override safety gates. |
| `available_at` | timestamp | yes | Earliest claim time. Retry backoff updates this. |
| `created_at` | timestamp | yes | Row creation time. |
| `updated_at` | timestamp | yes | Last row update time. |
| `claimed_at` | nullable timestamp | no | Current claim start time. |
| `claim_expires_at` | nullable timestamp | no | Lease timeout for crashed workers. Claim expiry permits re-claim, not double side effects. |
| `claimed_by` | nullable string | no | Worker identity/process. Display only; not authority. |
| `attempt_count` | integer | yes | Number of started attempts. Increment atomically when a worker claims the row. |
| `max_attempts` | integer | yes | Default 5 for transient processing; lower for non-idempotent-adjacent jobs unless outbox approval exists. |
| `next_retry_at` | nullable timestamp | no | Same as `available_at` while retry scheduled; separated for operator clarity. |
| `first_attempted_at` | nullable timestamp | no | First claim time. |
| `last_attempted_at` | nullable timestamp | no | Most recent claim time. |
| `last_error_class` | nullable enum/string | no | Redacted category: `TransientProvider`, `RateLimited`, `Timeout`, `Validation`, `PolicyDenied`, `ApprovalMissing`, `Conflict`, `Bug`, `Unknown`. |
| `last_error_summary` | nullable string(500) | no | Human-safe summary with tokens, secrets, raw provider payloads, card data, and sensitive customer text removed. |
| `last_error_redaction` | nullable JSON | no | Redaction metadata such as `{"secrets_removed": true, "raw_payload_stored": false, "sensitive_fields": ["access_token"]}`. No secret values. |
| `failure_visibility` | enum | yes | `InternalOnly`, `StaffVisible`, `ManagerVisible`, `EngineeringVisible`. Controls UI exposure. |
| `human_failure_title` | nullable string(160) | no | Operator-facing title when `WaitingForHuman` or `DeadLettered`. |
| `human_failure_detail` | nullable string(2000) | no | Safe explanation of what failed, operational impact, and next safe actions. |
| `required_review_gates` | JSON array | yes | Snapshot of `PolicyContext.required_reviews` or action-specific gates. Must be preserved across retry/re-run. |
| `approval_record_id` | nullable UUID | no | Required for approval-gated execution jobs. Re-run must validate the approval is still valid and scoped to this action. |
| `approved_action_id` | nullable UUID | no | Stable ID for an approved customer send/provider write/outbox action. Must be present before side effects. |
| `idempotency_key` | string | yes | Unique semantic key. See idempotency section. |
| `idempotency_scope` | enum | yes | `WorkflowResult`, `InternalTask`, `DraftMessage`, `CustomerSend`, `ProviderWrite`. |
| `supersedes_queue_id` | nullable UUID | no | Links an operator-created replacement/re-run to the failed row. |
| `superseded_by_queue_id` | nullable UUID | no | Filled when a newer row makes this row obsolete. |
| `payload_ref` | string | yes | Pointer to redacted canonical payload/evidence store, not raw unbounded JSON in the queue row. |
| `result_ref` | nullable string | no | Pointer to structured output, draft, task, or outbox result. |
| `audit_event_ids` | JSON array | yes | Audit events generated for enqueue, claim, result, failure, dead-letter, cancel, and re-run. |
| `policy_version` | string | yes | Policy version evaluated for the current attempt. Re-run should usually re-evaluate under current policy and record a new version. |
| `schema_version` | integer | yes | Queue-row schema version. |

Recommended unique constraints:

- `unique(idempotency_scope, idempotency_key)` for active non-superseded rows.
- `unique(approved_action_id)` where `approved_action_id is not null` for side-effect execution rows.
- `unique(provider, provider_event_id, workflow_kind)` when a provider/webhook event has a stable event ID.
- For draft messages/internal tasks, use semantic dedupe keys instead of raw text hashes so reworded retries do not create duplicates.

## Supporting tables

### `workflow_attempts`

Append-only attempt log. The queue row stores summary fields; this table stores per-attempt details.

Fields:

- `attempt_id`, `queue_id`, `attempt_number`, `worker_id`, `started_at`, `ended_at`, `outcome`.
- `input_policy_version`, `input_payload_ref`, `result_ref`.
- `error_class`, `error_summary`, `error_redaction`, `retry_decision`.
- `claim_token_hash`, not raw claim token, if using opaque claim tokens.
- `audit_event_id`.

### `workflow_outbox`

Required for customer-message sends and provider writes.

Fields:

- `outbox_id`, `approved_action_id`, `queue_id`, `action_kind`, `destination_kind`, `destination_id`.
- `approval_record_id`, `approval_actor`, `approved_at`, `approval_policy_version`.
- `payload_ref` containing the exact approved message/write payload.
- `status`: `Ready`, `Sending`, `Sent`, `ProviderAccepted`, `FailedTransient`, `FailedPermanent`, `DeadLettered`, `Cancelled`.
- `provider_request_id`, `provider_response_ref`, `provider_object_id`, `provider_idempotency_key`.
- `attempt_count`, `last_error_class`, `last_error_summary`.

Workers may retry outbox delivery, but they must never regenerate or modify the approved payload during send retry. Any content/policy change creates a new draft and a new approval record.

## Queue state machine

```text
Queued
  -> Claimed
Claimed
  -> Succeeded
  -> WaitingForHuman
  -> RetryScheduled
  -> DeadLettered
  -> Superseded
  -> Cancelled
RetryScheduled
  -> Claimed          when available_at <= now and lease can be acquired
WaitingForHuman
  -> Queued           when human supplies missing information or resolves review, without approving side effects unless an approval record is created
  -> Superseded       when operator creates a replacement/re-run row
  -> Cancelled        when event is obsolete or unsafe to continue
DeadLettered
  -> Queued           only by explicit operator re-run, with new audit event and current-policy validation
  -> Superseded       when replacement row is created
  -> Cancelled        when no action is needed
Succeeded, Cancelled, Superseded are terminal for that row.
```

State meanings:

- `Queued`: durable and eligible for future claim at `available_at`.
- `Claimed`: one worker has a lease. Lease expiry handles crashes; side-effect idempotency still prevents duplicate sends/writes.
- `RetryScheduled`: failed with a retryable error and is not claimable until `available_at`.
- `WaitingForHuman`: processing reached a policy, evidence, ambiguity, or approval gate. This is not an error retry state.
- `DeadLettered`: retry budget exhausted, permanent validation/policy failure, or repeated claim/worker bug. Human action is required before any continuation.
- `Superseded`: a newer event/re-run/replacement row owns the work.
- `Cancelled`: an operator or deterministic business rule decided no further processing is safe or needed.

## Retry policy

### Retryable failures

Retry automatically only for failures that do not change business meaning or approval requirements:

- transient network/provider timeout;
- HTTP 429/rate limit with `Retry-After` respected;
- HTTP 5xx/provider unavailable;
- database serialization conflict/deadlock;
- worker crash/lease expiry before committing a result;
- temporary dependency unavailable.

### Non-retryable or human-gated outcomes

Do not auto-retry these as infrastructure failures:

- `ApprovalMissing` or approval expired/out of scope;
- `PolicyDenied` or action prohibited by `PolicyContext`;
- missing required vaccine/profile/payment/care evidence;
- medical, behavior, eligibility, capacity-exception, refund/waiver/discount, or sensitive customer-message ambiguity;
- schema validation showing the source payload cannot be semantically mapped;
- provider 4xx that means request invalid, forbidden, or conflict requiring human reconciliation.

These move to `WaitingForHuman` if staff/manager can resolve them, or `DeadLettered` if they require engineering/data repair.

### Backoff

Default transient backoff:

| Attempt after failure | Delay before next claim |
| --- | --- |
| 1 | 1 minute |
| 2 | 5 minutes |
| 3 | 15 minutes |
| 4 | 1 hour |
| 5 | dead-letter unless an operator extends `max_attempts` with reason |

Rules:

- Add jitter of +/- 20% to avoid thundering herds.
- Respect provider `Retry-After` when longer than computed delay.
- Cap automatic retry delay at 1 hour for operational visibility; longer pauses should be explicit operator holds.
- Retry count increments on claim/start, not just on failure, so crash loops are visible.
- Re-run after dead-letter creates a new attempt/audit trail and normally resets automatic retry budget only for a new/superseding row. The old row remains dead-lettered or superseded for history.

## Dead-letter rules

Move to `DeadLettered` when:

- automatic retry budget is exhausted;
- the same worker/class bug repeats without progress;
- the event payload is unmappable or corrupt and cannot become a safe workflow packet;
- an approved outbox action repeatedly fails with permanent provider errors;
- operator marks the job unsafe to continue;
- policy evaluation proves the requested operation is prohibited and there is no human-resolvable review path.

Dead-letter rows must include:

- safe title and detail for staff/manager/engineering view;
- redacted `last_error_class`, `last_error_summary`, and redaction metadata;
- operational impact: e.g. "daily update draft not generated", "provider booking write not sent", "customer message not delivered";
- safe operator actions;
- unsafe operator actions that are blocked;
- links/refs to event, subject, approval/draft/outbox, attempts, and audit trail.

## Error capture and redaction

Store enough to debug without leaking secrets or sensitive raw data into queue dashboards.

Allowed in `last_error_summary`:

- provider/status category, not full headers;
- semantic validation issue, e.g. "missing pet_id for reservation-scoped event";
- safe subject labels/IDs already visible to the operator role;
- short operational impact.

Never store in queue row summaries:

- API keys, bearer tokens, cookies, signatures, webhook secrets;
- full payment card/bank data;
- raw provider JSON or unredacted webhooks;
- customer free-text messages containing medical/behavior/legal allegations unless separately redacted;
- exact approved outbound message body in error fields.

Raw artifacts, if needed for engineering, belong in access-controlled evidence storage via `payload_ref`/`provider_response_ref`, with redaction status recorded and audited.

## Human-visible failure state

Operators should see a queue/failure panel with these fields:

- status badge: `Waiting for staff`, `Waiting for manager`, `Retrying`, `Failed - needs review`, `Dead-lettered`, `Cancelled`, `Superseded`;
- affected location, subject, event type, and workflow kind;
- last safe error summary and last attempted time;
- attempt count and next retry time if any;
- required review gates and missing evidence;
- operational impact;
- safe actions available to their role;
- audit/history timeline.

Example display:

```text
Failed - needs manager review
Reservation R-1234 / Pet: Maple / Booking triage
Impact: booking confirmation was not drafted because vaccine evidence is missing and the requested holiday capacity exception requires manager review.
Safe actions: open reservation, request vaccine proof, assign manager review, cancel this workflow if obsolete.
Blocked actions: confirm booking, send customer confirmation, or write provider reservation status without required approvals.
```

## Operator re-run controls

Allowed controls:

- `Retry now`: only for retryable transient failures. Keeps the same row, increments attempt, preserves gates and approved payload refs.
- `Re-run under current policy`: creates a replacement queue row with `supersedes_queue_id`, re-reads canonical event/evidence, re-evaluates current policy, and records new audit events.
- `Mark superseded`: link to a newer event/job that already handled the operational need.
- `Cancel`: terminally stop the row with reason.
- `Assign review`: create/assign internal staff/manager/engineering task for missing evidence or approval.
- `Extend retry budget`: engineering/admin-only, requires reason, max extension, and audit event.

Blocked controls:

- no "force success";
- no customer-message send from a failed draft row without a separate approval record;
- no provider write retry if the approval is missing, expired, or scoped to different payload/subject/action;
- no re-run that downgrades `required_review_gates` or changes `PolicyContext` to bypass a gate;
- no editing an approved outbox payload in place.

## Safe worker claiming and locking

Workers claim rows with a short lease and a compare-and-swap update:

```sql
update workflow_queue
set status = 'Claimed',
    claimed_by = :worker_id,
    claimed_at = now(),
    claim_expires_at = now() + interval '5 minutes',
    attempt_count = attempt_count + 1,
    last_attempted_at = now(),
    updated_at = now()
where queue_id = :queue_id
  and status in ('Queued', 'RetryScheduled')
  and available_at <= now()
  and (claim_expires_at is null or claim_expires_at < now())
returning *;
```

Operational expectations:

- Prefer `select ... for update skip locked` or equivalent queue partition locking for batch claim.
- Claim by location/priority/available time to preserve locality and avoid starvation.
- Workers heartbeat/extend lease only while actively processing.
- On lease expiry, another worker may claim the row; all side effects must still use idempotency/outbox constraints.
- The worker must write one terminal/next state and append an attempt record in a transaction.
- Worker crashes before committing a result leave the row reclaimable after lease expiry.
- Worker crashes after committing an outbox send but before updating queue state are reconciled by provider idempotency key/provider object ID lookup before any duplicate send/write.

## Idempotency with retry and re-run

Use different idempotency scopes for different effects.

### Workflow result / pure computation

Key shape:

```text
workflow-result:{event_id}:{workflow_kind}:{policy_version-or-current-policy-hash}
```

Retries may recompute but must update the same queue/result row rather than creating duplicate tasks/drafts. Re-run under current policy intentionally uses a new policy hash and supersedes the old row.

### Internal task creation

Key shape:

```text
internal-task:{event_id}:{workflow_kind}:{subject}:{task_intent}:{evidence_version}
```

If a retry sees the task already exists, it links it in `result_ref` and succeeds. It must not create duplicate staff tasks with slightly different wording.

### Draft customer message

Key shape:

```text
draft-message:{event_id}:{subject}:{message_intent}:{audience}:{evidence_version}
```

Retries may regenerate a draft only while no human has approved or edited it. Once reviewed/approved, content is immutable for the approved action; changes require a new draft and approval.

### Customer send / provider write

Key shape:

```text
side-effect:{approved_action_id}
```

Rules:

- Side-effect jobs require `approved_action_id` and `approval_record_id` before entering `Queued`.
- Retry reuses the exact approved payload and provider idempotency key.
- Re-run checks whether the provider already accepted the side effect before sending again.
- If provider status is unknown after timeout, reconcile by idempotency key/provider object lookup before another attempt.
- A failed approval-gated side effect may be retried only if the approval remains valid for the same actor, subject, payload, destination, and action kind.
- If approval expired or policy changed materially, the job moves to `WaitingForHuman`; it does not auto-send.

## Approval gates are never bypassed

Retry/re-run must preserve this invariant:

```text
Executable side effect = allowed action + valid policy + valid approval gate clearance + idempotent outbox record.
```

Consequences:

- A `WorkflowResult::NeedsHumanReview` is a successful safe workflow outcome, not a failed job to auto-retry.
- `RecommendedAction::DraftMessage` does not imply permission to send.
- `RecommendedAction::UpdateStatus` is a suggestion until an approved provider-write action exists.
- Provider/webhook event names are not enough to authorize writes; semantic mapping and policy evaluation must pass first.
- Re-run under current policy may add gates. It must not remove gates unless policy version explicitly says the gate is no longer required and the audit trail records why.

## Examples

### 1. Transient daily-update draft failure

A `DailyUpdateNeeded` event for reservation `R-100` queues `DailyUpdateDraft`.

- Attempt 1: worker times out reading care notes. Row moves to `RetryScheduled`, `attempt_count = 1`, `available_at = now + 1 minute`.
- Attempt 2: succeeds. Result is a draft message and `RequestHumanReview(CustomerMessageApproval)` if policy requires review.
- No customer message is sent by the draft workflow. A separate approved outbox action is required.

### 2. Missing vaccine evidence blocks booking triage

A `BookingRequested` event queues `BookingTriage`.

- Worker validates profile and finds required vaccine evidence missing.
- Row moves to `WaitingForHuman`, not `RetryScheduled`.
- Operator sees: "Missing vaccine evidence; safe actions: request proof, assign document review, cancel if obsolete. Blocked: confirm booking or write provider status."
- When proof is uploaded, a new event/re-run creates or requeues work under current policy. Previous attempts remain audited.

### 3. Approved customer message send times out

A staff member approves draft message `D-55`, creating approved action `A-55` and outbox row `O-55`.

- Send attempt times out after provider request with idempotency key `side-effect:A-55`.
- Retry first reconciles provider state by idempotency key.
- If provider accepted it, mark `Sent/ProviderAccepted` and succeed.
- If provider did not accept and approval is still valid for the exact payload/destination, retry the same payload.
- Worker cannot alter the message text during retry.

### 4. Provider reservation write fails policy after re-run

An approved reservation status update was queued, but before delivery the reservation receives a new incident event requiring manager review.

- Retry/re-run re-evaluates current policy.
- Approval is now stale because the subject risk state changed.
- Row moves to `WaitingForHuman` with `ApprovalMissing`/`approval stale` summary.
- Operator may assign manager review or cancel/supersede; they cannot retry the provider write as-is.

### 5. Dead-letter corrupt webhook event

A webhook event has a provider ID but cannot be mapped to customer, pet, reservation, or safe external subject after repeated validation attempts.

- First validation failure may create an engineering-visible `WaitingForHuman`/data-repair task if the provider source is expected.
- If the payload remains unmappable or malformed, row moves to `DeadLettered` with redacted summary.
- Operator can mark superseded by a corrected event, re-run after adapter repair, or cancel. The UI must not offer business-action buttons from this dead-letter row.

## Acceptance checklist

- Queue rows are durable, location-scoped, subject-scoped, and idempotency-keyed.
- Retry policy distinguishes transient failures from human/policy gates.
- Dead-letter status is visible and actionable without leaking raw secrets/sensitive payloads.
- Operator controls are explicit and audited.
- Worker claim/lease rules avoid double processing while accepting crash recovery.
- Customer-message/provider-write side effects require approved immutable outbox records.
- Retry/re-run cannot bypass human approval gates or mutate approved payloads in place.
