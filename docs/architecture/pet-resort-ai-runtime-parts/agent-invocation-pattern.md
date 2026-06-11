# Agent invocation pattern

Purpose: define how the pet-resort application should invoke Hermes/AI work from product workflows. This is a synthesis input for `docs/architecture/pet-resort-ai-runtime.md`, not the final approval artifact.

Status: recommended architecture for downstream synthesis. Production automation still needs human approval before live customer-facing sends, reservation mutations, payment/provider commands, incident closure, or long-running worker deployment.

## Recommendation in one sentence

Use an application-owned durable workflow queue/inbox as the default production boundary: the app verifies or adapts the trigger, writes an idempotent `WorkflowEvent`/run record, builds a typed agent prompt packet, invokes Hermes/AI asynchronously through an `AgentRuntime` adapter, validates the structured `WorkflowResult<T>`, persists the result and audit trail, and lets deterministic policy plus human review decide whether any side effect may execute.

## Why this is the default

The domain foundation already encodes the intended boundary:

- `domain::tools::AgentRuntime::run_structured(event, input) -> WorkflowResult<T>` is the semantic runtime port. It hides the concrete Hermes transport from product code.
- `domain::agents::WorkflowAgent` builds typed `AgentPromptPacket<T>` values and validates `WorkflowResult<T>` outputs.
- `WorkflowEvent` carries event id, actor, location, subject, and `PolicyContext`.
- `WorkflowResult<T>` carries status, structured output, recommended actions, risk flags, verification notes, and optional human-review reason.
- Tool traits are draft/read oriented. Agents recommend, extract, summarize, draft, and flag risk; deterministic Rust validators and approved staff/manager actions own writes.

A queue-first runtime preserves those contracts better than calling a model directly from request handlers. It also provides location isolation, retry control, dead-letter handling, auditability, and a single place to enforce review gates.

## Invocation patterns

### 1. Sync API request/response

Use for: low-latency previews, deterministic reads, lightweight extraction helpers, staff-facing draft previews, or local validation that can safely time out without losing work.

Do not use for: customer-facing sends, reservation confirmations/cancellations, payments/refunds/waivers, document approvals, incident decisions, long OCR/media work, or anything that needs durable retry.

Trigger source:
- Staff UI action such as "draft reply", "summarize notes", or "preview missing info".
- Internal API call after deterministic authorization and data loading.

Queue ownership:
- No durable AI queue for the request itself unless the result must be kept.
- The app still owns request authorization, source-data loading, prompt-packet construction, validation, and audit append if a result is shown or reused.

Idempotency key:
- `sync-preview:{location_id}:{actor_id}:{workflow}:{source_record_id}:{source_version}:{policy_snapshot_id}:{input_hash}`.
- The key should dedupe cached previews only; it must not imply permission to execute an action.

Timeout/retry/dead-letter:
- Tight timeout, normally 5-15 seconds.
- At most one short retry for transport errors if the UI still waits.
- No automatic dead-letter; on timeout return "draft unavailable" and optionally create a background job or staff task if the workflow requires follow-up.

Persistence and audit:
- Persist only if the preview influences a workflow, creates a draft, or is shown as a sourced recommendation.
- Audit actor, source records, policy snapshot, model/runtime id, prompt packet reference, redacted output reference, and any review gate shown.

Human-task boundary:
- Create a task instead of a sync AI call when the answer requires missing source data, manager authority, medical/vaccine judgment, policy exception, incident escalation, or cross-system reconciliation.

### 2. Background job queue

Use for: the default production pattern for workflow automation, especially when source events may retry, outputs must be audited, or human review may be created.

Trigger source:
- Verified webhook event.
- Staff/customer workflow event from the app.
- Scheduled reconciliation watcher that enqueues concrete work items.
- Batch import, document upload, incident creation, booking request, daily-note event, or messaging event.

Queue ownership:
- The application owns the durable queue/inbox and records the canonical work item before Hermes/AI is invoked.
- Queue records should include `workflow_event_id`, location, subject, source pointer, source version, policy snapshot, requested agent, priority, status, attempts, next-run time, lock owner, and redacted prompt/result artifact refs.
- Hermes/AI workers are consumers behind the `AgentRuntime` port, not the source of product truth.

Idempotency key:
- `workflow:{workflow_name}:{location_id}:{subject_type}:{subject_id}:{source_event_id_or_provider_event_id}:{source_version}:{policy_snapshot_id}`.
- For source systems without stable event ids, use a canonical hash of provider, external id, event type, occurred-at bucket, and source revision.
- Recommended actions get their own keys: `action:{workflow_run_id}:{action_type}:{target_id}:{action_version}`.

Timeout/retry/dead-letter:
- Per-attempt timeout based on workflow class: 30-90 seconds for text; 2-10 minutes for OCR/batch/media; longer work should split into stages.
- Exponential backoff with jitter for transient runtime/tool failures.
- Retry only safe/idempotent phases automatically. Do not retry customer sends, provider writes, or reservation mutations unless they have separate approved idempotency keys and adapter-level confirmation semantics.
- Dead-letter after a small bounded attempt count or permanent validation failure. Dead-letter records create an engineering/staff task with redacted evidence.

Persistence and audit:
- Persist inbound event, queue lifecycle, prompt packet ref, runtime metadata, structured result, validation outcome, recommended actions, policy decision, human review status, final side-effect result, and before/after domain state where applicable.
- AI output is evidence/recommendation, not the final domain fact. The accepted staff/policy decision is a separate audit event.

Human-task boundary:
- Create a human task whenever deterministic validators return `NeedsHumanReview`, sources conflict, required policy is missing, an adapter cannot verify truth, or the recommended side effect is outside approved automation level.

### 3. Hermes CLI invocation

Use for: prototypes, local/manual backoffice jobs, one-shot migrations of non-production artifacts, research/synthesis, and operator-triggered review packets.

Do not use for: the stable application-to-runtime contract in production; request handlers should not shell out to `hermes chat` directly.

Trigger source:
- Human/operator command.
- Controlled admin script.
- Developer or backoffice batch with explicit input and output paths.

Queue ownership:
- The invoking script owns input snapshotting, output files, and run log.
- If the result affects product state, the script must write a normal app workflow event or task instead of bypassing the app queue.

Idempotency key:
- `cli:{job_name}:{input_snapshot_hash}:{policy_snapshot_id}:{operator_id}`.

Timeout/retry/dead-letter:
- Bounded command timeout per job.
- Manual retry only unless the script is explicitly made idempotent.
- Failures produce an operator-visible log or internal task; no silent retries for product-impacting work.

Persistence and audit:
- Store command, input artifact refs, output artifact refs, operator actor, model/runtime info, and any imported result id.
- Never use CLI output as direct authority for writes; import it through the same validation/review path as other AI results.

Human-task boundary:
- If the CLI job discovers missing production policy, ambiguous data, or approval-needed actions, it creates app-owned tasks rather than executing them.

### 4. Webhook-to-Hermes / Tools API

Use for: service-triggered integration where the app or trusted middleware can accept webhooks and then enqueue AI work or call a Hermes Tools API endpoint with a typed packet.

Preferred shape:
1. Receive webhook in the app or integration adapter.
2. Verify signature/raw body and map provider payload to a semantic event.
3. Durably store the inbound event and idempotency key.
4. Enqueue a background workflow job or call a Tools API route that itself performs durable queue acceptance.
5. Return provider success only after durable acceptance or explicit permanent ignore.

Do not use for: long model work inline in the provider webhook request/response path.

Trigger source:
- Gingr/PMS webhook, payment provider webhook, messaging provider event, document-storage event, review platform event, or internal app webhook.

Queue ownership:
- The application or integration service owns webhook verification and durable inbound-event storage.
- Hermes/Tools API may own execution of the AI run only after an app-owned or Tools-owned durable acceptance record exists.

Idempotency key:
- `webhook:{provider}:{event_id}` when provider event ids are stable.
- Fallback: `webhook:{provider}:{event_type}:{external_subject_id}:{occurred_at}:{payload_revision_hash}`.
- AI run idempotency derives from the resulting semantic `WorkflowEvent`, not raw payload bytes alone.

Timeout/retry/dead-letter:
- Webhook receiver timeout should be short, usually under provider limits.
- Provider delivery retries must be absorbed idempotently by inbound-event storage.
- Downstream AI retries follow background queue rules.
- Unknown verified events are recorded and ignored or routed to staff/engineering review; unverified events are rejected or quarantined.

Persistence and audit:
- Store verification result, provider event id, redacted raw payload reference if allowed, semantic event mapping, durable acceptance, downstream workflow run id, and final disposition.

Human-task boundary:
- Create tasks for unknown event types, signature failures requiring operator action, mapping ambiguity, stale/regressive events with customer impact, payment/reservation conflicts, or provider data that cannot be semantically trusted.

### 5. Cron watcher

Use for: scheduled briefs, stale-work detection, reconciliation monitors, missing-information sweeps, overdue follow-ups, drift checks, and operational reporting.

Do not use for: direct customer-facing sends or external writes. Cron should enqueue work or create review tasks.

Trigger source:
- Time-based schedule by location, region, workflow, or system monitor.

Queue ownership:
- The watcher owns scan state/cursor and creates app-owned workflow events, queue jobs, or human tasks.
- The workflow queue owns AI execution after enqueue.

Idempotency key:
- `cron:{watcher_name}:{location_id}:{window_start}:{window_end}:{subject_id_or_query_hash}:{policy_snapshot_id}`.
- For stale-item monitors, use the existing task/reservation/document id plus the observed version to avoid duplicate tasks.

Timeout/retry/dead-letter:
- Watcher scan should be bounded and checkpointed.
- Failed scans retry on the next schedule with cursor safety.
- Repeated failures create engineering/ops tasks.
- AI work produced by the watcher follows background queue rules.

Persistence and audit:
- Store watcher run id, scan window, query/cursor, found candidates, created/skipped job ids, and reason for each skip or task creation.

Human-task boundary:
- Watchers should create human tasks instead of AI calls when the issue is clearly operational: overdue staff action, missing credential/integration, policy not configured, repeated adapter failure, or manager approval required.

### 6. Long-running worker service

Use for: steady production processing of the app-owned queue once operations/security approve deployment. Also appropriate for OCR/document pipelines, batch summarization, and multi-stage work that needs leases, health checks, and autoscaling.

Do not use for: bypassing product review gates or giving agents direct authority over source-of-truth writes.

Trigger source:
- Durable queue subscriptions, topic streams, document/OCR pipelines, or app-created workflow jobs.

Queue ownership:
- The app owns workflow queues and product-state transitions.
- Worker service owns leases, execution environment, runtime adapter, structured output validation, and result handoff back to the app.
- Separate queues may exist by class: intake, booking, documents, messaging drafts, incidents, staff ops, and reconciliation.

Idempotency key:
- Same as background job queue, with worker lease id and attempt id separated from business idempotency key.
- External side effects use separate approved action idempotency keys.

Timeout/retry/dead-letter:
- Lease visibility timeout must exceed expected run time and be heartbeated for long jobs.
- Per-stage timeouts; split OCR/media/batch into resumable stages.
- Backoff and circuit breakers per provider/runtime.
- Dead-letter includes error category: transient runtime, validation failure, policy denial, missing source, adapter failure, or human-review-required.

Persistence and audit:
- Store worker identity/version, runtime/model/provider, tool versions, lease/attempt metadata, prompt/result artifact refs, validation logs, and emitted action/task ids.
- Redact/minimize PII and medical/payment details in logs; keep source pointers for authorized drill-down.

Human-task boundary:
- The worker creates human tasks for review-required outputs and stops. It does not wait interactively for humans unless the workflow has a separate approval callback/event.

## Production default flow

1. Trigger enters the app or integration adapter.
2. App authenticates/authorizes the trigger and normalizes it to a semantic `WorkflowEvent`.
3. App writes inbound event, idempotency key, source pointers, policy snapshot, and initial queue record.
4. Worker claims the queue record with a lease.
5. Worker loads trusted source data through typed stores/tools and builds an `AgentPromptPacket<T>`.
6. Worker invokes `AgentRuntime::run_structured` through the selected Hermes transport.
7. Worker validates schema, allowed actions, review gates, risk flags, source citations, and output semantics.
8. App persists `WorkflowResult<T>` as AI output and separately persists deterministic policy decisions, human tasks, drafts, or approved side effects.
9. Any side effect executes only through typed adapters with its own idempotency key and audit event.
10. Queue item becomes completed, needs-human-review, failed-safely, or dead-lettered.

## Human task instead of AI call

Create a human task before invoking AI when:

- Required source records are missing, inaccessible, stale, or conflicting.
- The task is an authority decision, not an information-processing task: approve booking exception, waive/refund/forfeit money, confirm/cancel reservation, approve ambiguous vaccine/medical proof, close incident, alter staff schedule, or publish/send sensitive customer communication.
- The trigger requests a policy exception or the relevant location/provider policy is not configured.
- The adapter cannot verify identity, signature, payment truth, document provenance, or reservation state.
- The expected output cannot be validated against a typed schema.
- Privacy or data-minimization rules would require exposing more PII/medical/payment data than the agent needs.
- The same subject is already in an active non-terminal task and the new source version does not materially change the required action.

Create a human task after AI runs when:

- `WorkflowResult.status` is `NeedsHumanReview`, `NeedsMoreInformation`, `RejectedByPolicy`, or `FailedSafely` with operational follow-up.
- `human_review_reason` is present.
- Recommended actions include `RequestHumanReview`.
- Risk flags include safety, medical, payment, incident, legal, privacy, reputation, or customer-message sensitivity.
- The output passes schema validation but conflicts with deterministic policy or trusted source state.

## Persistence and audit minimum

Every production AI run should produce durable records for:

- Trigger: provider/source, source event id, source version, actor, location, subject, occurred-at, received-at.
- Idempotency: business key, action keys, dedupe result, prior-run linkage.
- Policy: policy snapshot/ref, allowed actions, automation level, required reviews.
- Prompt: prompt packet schema/version, source refs, redacted prompt artifact ref, runtime/model/provider ref.
- Result: structured output ref, summary, status, recommended actions, risk flags, verification notes, human-review reason.
- Validation: schema validation result, semantic validation result, policy validation result, redaction/data-minimization result.
- Review: human task id, reviewer actor, approval/rejection/change request, timestamps, comments/evidence refs.
- Side effects: adapter call id, idempotency key, before/after state, provider refs, confirmation status, error category.
- Lifecycle: queue status, attempt count, timeout/retry/dead-letter state, worker id/version.

Do not store raw secrets, card data, signed webhook material, unnecessary raw provider payloads, or unredacted medical/payment/message bodies in generic logs. Store secured artifact refs when authorized retention is required.

## Allowed patterns by workflow class

Legend: default = recommended default; allowed = permitted with constraints; avoid = not a production default; forbidden = should not be used for this class except as a non-production prototype.

| Workflow class | Default pattern | Other allowed patterns | Avoid/forbid | Notes |
| --- | --- | --- | --- | --- |
| Intake | Background job queue from app/webhook events | Sync preview for staff draft; webhook-to-queue; cron for stale leads; long-running worker at scale; CLI for prototypes | Direct inline webhook model work; autonomous customer send without approved template/review | AI may extract lead/customer/pet/service/date facts, draft follow-up, and create missing-info tasks. Human/staff review required for customer-facing sends unless a deterministic approved send path exists. |
| Booking | Background job queue after deterministic availability/policy reads | Sync staff preview for triage explanation; webhook-to-queue for booking requests; cron for stale holds; long-running worker at scale; CLI for manual analysis | Sync API for final confirmation; cron direct booking mutation; AI direct reservation writes | Deterministic policy owns availability, hard stops, vaccine/deposit requirements, and status transitions. AI may draft explanations and recommend status/task actions. Manager/human approval required for exceptions, waivers, overbooking, cancellation/refund-sensitive cases. |
| Document | Background job queue or long-running worker service | Webhook-to-queue from upload/storage; sync preview only for tiny staff-visible extraction; cron for missing/expiring docs; CLI for backfills | Sync API for OCR-heavy work; AI final approval of ambiguous medical proof | AI may OCR/extract vaccine names/dates and flag ambiguity. Deterministic validators and trained staff own final approval, especially uncertain source, mismatched pet, expired dates, medical notes, or low-confidence extraction. |
| Messaging | Background job queue for draft generation and review packets | Sync preview for staff compose assist; webhook-to-queue for inbound messages; cron for eligible reminders; long-running worker for volume; CLI for content audits | Autonomous customer sends without explicit approved policy/template/facts/recipient; inline provider webhook model work | Default output is `DraftMessage` plus evidence. Send execution is a separate deterministic/human-approved action with recipient, channel, facts, template/copy, and idempotency key. Sensitive payment/incident/medical messages require review. |
| Incident | Background job queue with manager task creation | Webhook-to-queue from incident form; sync staff preview for summarizing already-entered facts; cron for open-incident follow-up; long-running worker for large operators; CLI for retrospectives | AI closing incidents; autonomous owner messages; direct severity/legal conclusions | AI may summarize facts, missing fields, possible severity signals, and draft manager/owner review packets. Manager approval required for severity, owner communication, closure, compensation, medical/legal escalation, or policy findings. |
| Staff Ops | Cron watcher plus background queue for daily briefs/tasks | Long-running worker for steady ops; sync preview for manager dashboard Q&A; webhook-to-queue from staff/task events; CLI for manual reports | AI changing schedules, labor assignments, policies, or customer records directly | AI may summarize occupancy/labor/care watchlists, recommend tasks, and flag risks. Human/manager approval required for schedule changes, discipline/performance actions, policy changes, or customer-impacting commitments. |

## Pattern choice by workflow stage

| Stage | Recommended pattern | Reason |
| --- | --- | --- |
| Source event acceptance | App API/webhook receiver | Signature/auth/policy context must be deterministic and auditable. |
| AI analysis/drafting/extraction | Background queue or long-running worker | Keeps model latency and retries out of customer/provider request paths. |
| Staff-facing quick preview | Sync API request/response | Acceptable when no side effect occurs and timeout can safely return no draft. |
| Scheduled detection | Cron watcher enqueueing queue jobs/tasks | Cron should find candidates, not mutate customer/provider state. |
| Manual/backoffice research | Hermes CLI invocation | Useful for prototypes and synthesis, but import through app validation for product impact. |
| External side effects | Typed app adapters after approval/policy validation | Separate action idempotency and audit must exist; never let raw AI output call providers directly. |

## Approval gate for production automation

The recommended pattern is not itself approval to automate production actions. Before production rollout, humans must approve:

- Which Hermes transport implements `AgentRuntime` for each environment.
- Queue ownership, retention, PII/medical/payment redaction, and artifact storage policy.
- Per-workflow automation levels and allowed actions.
- Customer-message send policy, including templates/copy ownership and review bypass rules if any.
- Reservation/payment/document/incident side-effect adapter permissions.
- Dead-letter routing and on-call/operations ownership.
- Worker deployment model, secrets, network access, health checks, and rollback plan.

Until those are approved, production AI outputs remain drafts, recommendations, review packets, and internal tasks only.
