# Pet Resort rollback and manual fallback plan

Status: launch-readiness artifact for MVP pilot planning.

This plan defines how staff and operators stop unsafe automation, preserve data, continue operations manually, prevent unintended customer sends/provider effects, and decide whether to pause, fall back, or resume the pilot. It does not approve pilot launch, production deployment, live customer messaging, provider/PMS writes, payment collection, refunds, waivers, discounts, medical/vaccine decisions, incident closure, or any autonomous AI behavior.

## Source inputs and assumptions

Primary inputs used:

- `docs/launch/pet-resort-launch-readiness.md`
- `docs/roadmap/pet-resort-mvp-implementation.md`
- `docs/roadmap/pet-resort-mvp-stack.md`
- `docs/workflows/staff-operations.md`
- `docs/workflows/customer-messaging-agent.md`
- `docs/workflows/incident-escalation-agent.md`
- `docs/workflows/payments-pricing.md`
- `docs/security/pet-resort-security-audit.md`
- `docs/security/pet-resort-security-audit-parts/data-retention.md`
- `docs/data-model/workflow-queue-retry-dead-letter.md`
- `docs/architecture/workflow-result-envelope.md`

MVP implementation assumptions:

1. The MVP is an internal staff/manager workflow app with a Next.js staff UI, Rust API/workers, PostgreSQL system-of-record/queue/outbox/audit tables, S3-compatible private evidence storage, and an `AgentRuntime` adapter.
2. Local/CI default adapters are fake, deterministic, or disabled. Any real Hermes/OpenAI-compatible runtime is review-mode only unless separately approved.
3. Live customer sends, live provider/PMS writes, production deployment, payment provider configuration, and money movement remain explicit human approval gates.
4. Customer-facing messages are draft/review artifacts by default. A draft is never a send.
5. Workflow results and AI confidence are not authority. The application owns policy evaluation, validation, idempotency, audit, and side-effect execution.
6. During rollback, the safe default is fail-closed: disable automation first, preserve evidence, continue manually, and resume only after human verification.

## Severity levels for rollback

Use the smallest safe rollback scope, but do not under-scope customer-message, payment, incident, medical, vaccine, or data-loss risks.

| Level | Trigger examples | Required posture |
| --- | --- | --- |
| Watch | Non-customer-affecting defect, isolated local smoke failure, redacted log issue, slow queue with no side effects. | Keep pilot constrained; assign owner; increase monitoring; do not widen pilot. |
| Stop workflow | One workflow is unsafe or degraded, such as daily-update drafts producing unsupported wording, vaccine extraction failures, or booking triage stale data. | Disable the affected workflow and route its work manually. Keep other workflows only if unaffected and verified. |
| Stop outbound | Any risk of wrong recipient, duplicate send, auto-send without authority, opt-out/quiet-hours/suppression failure, or sensitive wording exposure. | Activate outbound kill switch and provider hold; no customer sends except manual manager-approved communication outside the app. |
| Stop pilot | Data integrity risk, audit gap, auth/role defect, missing evidence, incident safety ambiguity, payment/provider mutation risk, or cross-workflow policy failure. | Pause live/demo pilot effects; internal-only manual operations continue; engineering/manager incident process begins. |
| Security/legal hold | Suspected data exposure, unauthorized access, credential compromise, raw payment/medical/incident leak, evidence loss, or legal/insurance issue. | Preserve all records, suspend destructive jobs, involve owner/admin/legal/compliance, and do not delete or redact except through approved incident handling. |

## Immediate stop-the-line checklist

Complete this checklist before diagnosing root cause. The goal is to prevent additional harm and preserve evidence without relying on autonomous AI behavior.

### 1. Assign command and record the incident

- Name the incident commander: manager/admin for operational incidents; engineering owner for infrastructure/runtime incidents; owner/admin or security/compliance for security incidents.
- Record incident start time, reporter, trigger, affected workflows, location/pilot mode, and known customer/provider/payment exposure.
- Create or update an append-only incident/audit record with safe summary and evidence refs.
- Freeze the change window for affected workflow flags, templates, runtime config, provider credentials, and policy records except for stop/disable actions.

### 2. Stop AI/workflow processing

- Set global automation mode to `manual_only` or equivalent fail-closed setting.
- Disable or pause all `AgentRuntime` adapters for affected workflows:
  - inquiry intake extraction/drafting;
  - booking triage recommendation/draft generation;
  - vaccine/document OCR or extraction;
  - daily care update drafting;
  - incident summarization/drafting;
  - CRM/review request drafting;
  - payment reminder drafting.
- Switch runtime adapter to `DisabledAgentRuntime` or fake deterministic mode only for internal diagnostics.
- Revoke or disable model/provider credentials from running workers if a model/tool leak, unsafe output, or runaway invocation is suspected.
- Stop worker leases or job claimers for affected queues. Existing claimed jobs should fail safely or release without committing side-effect candidates.
- Put workflow queue rows into an explicit hold state such as `WaitingForHuman`, `Cancelled`, or operator hold; do not auto-retry policy, approval, or ambiguity failures as infrastructure failures.
- Disable scheduled draft generation, batch jobs, replay jobs, and dead-letter re-runs for affected workflows.

### 3. Prevent outbound sends and provider/payment effects

- Activate the global outbound kill switch.
- Disable customer-message send workers and outbox dispatchers.
- Place `workflow_outbox` rows and provider action attempts into `hold`, `cancelled`, or `manual_review` state unless they have already completed.
- Disable or remove provider delivery credentials from the running environment where feasible:
  - email/SMS/portal send adapters;
  - PMS/provider write adapters;
  - payment checkout/refund/waiver/discount adapters;
  - webhook command endpoints that can mutate business state.
- Preserve inbound webhooks in raw/verifiable boundary storage where configured, but stop business mutation from them unless a deterministic, pre-approved read/reconciliation path is verified safe.
- Do not drain queues by sending. Draining means moving to hold/cancel/review with audit, not attempting delivery.

### 4. Preserve data and evidence

- Stop destructive retention, cleanup, compaction, anonymization, raw prompt deletion, vector deletion, and bulk migration jobs for affected subjects until hold status is assessed.
- Snapshot/export current state before repair:
  - PostgreSQL database or scoped tables for customers, pets, reservations, documents, vaccines, tasks, incidents, messages, payments, workflow events/jobs/results/outbox, approvals, and audit events;
  - object-storage metadata and affected evidence objects;
  - deployment config, feature flag state, model/provider config refs without secrets, template/policy versions, worker versions, and migration version;
  - safe logs with request/job/audit ids and redacted error classes.
- Preserve append-only audit chronology. Corrections, reversals, redactions, and cleanup must be new events, never edits to historical audit rows.
- Preserve raw evidence by governed refs. Do not paste raw documents, OCR, message bodies, incident narratives, payment payloads, secrets, or provider JSON into broad chat/docs/log summaries.

### 5. Notify staff and switch to manual operations

- Announce pilot hold/manual mode in the internal channel or status page.
- Tell staff which workflows are disabled, what work continues manually, who approves exceptions, and where to record evidence.
- Route customer communications to manual manager-approved channels only.
- Remind staff that customer-safe wording must avoid unsupported promises, medical/vaccine/behavior conclusions, payment/refund commitments, incident blame/liability language, and policy exceptions.
- Start a manual work log for every affected customer/pet/reservation/message/payment/incident so later reconciliation can append audit events.

## Disable agents and AI workflows

Rollback must not depend on AI deciding to stop itself. Controls should exist at multiple layers.

### Required kill surfaces

| Surface | Stop action | Verification |
| --- | --- | --- |
| Global automation policy | Set environment/config/policy row to `manual_only` or `automation_disabled`. | API health/status and admin UI show manual-only; new workflow events become review/manual tasks only. |
| Runtime adapter | Switch affected workflows to `DisabledAgentRuntime`; revoke real model routing. | Test event returns `blocked`/`failed_safely`/`needs_human_review` without model call. |
| Queue workers | Stop/scale workers to zero or disable workflow-kind claimers. | No new claimed jobs for affected workflow kinds; existing leases expire or fail safely with audit. |
| Draft generation | Disable scheduled/batch draft jobs and event-triggered draft creation. | New staff notes/incidents/inquiries do not create AI drafts; manual task appears instead. |
| Tool/model grants | Revoke runtime model keys/tool permissions for affected workflows. | Credential lookup/model call fails closed; no secrets exposed in logs. |
| Replay/dead-letter | Disable automatic replay and operator bulk re-run. | Dead-letter/retry rows remain held; re-run requires explicit human action and current-policy validation. |
| Provider/action adapters | Disable provider write/send/payment adapters. | Adapter health indicates disabled; attempts are blocked before provider contact. |

### Workflow-specific fallback behavior

| Workflow | Disable behavior | Manual fallback |
| --- | --- | --- |
| Inquiry intake | Stop extraction and draft replies. | Staff enters inquiry facts in dashboard/spreadsheet, verifies contact consent, and uses approved manual templates or phone follow-up. |
| Booking triage | Stop AI recommendations and confirmation drafts. | Manager/staff manually checks availability, vaccines, care/behavior flags, payment/deposit status, and records decision evidence. No automated confirm/reject. |
| Vaccine/document | Stop OCR/extraction auto-suggestions if unsafe; preserve uploads. | Human document reviewer verifies vaccine proof from the stored file or asks owner for clearer proof. No AI auto-accept. |
| Daily care updates | Stop daily-update draft generation. | Staff records care notes and, if needed, manager/front desk manually writes customer-safe update from approved facts. |
| Incidents | Stop AI summaries, severity suggestions, owner-message drafts, and profile-flag suggestions. | Staff/lead/manager follows incident intake, immediate safety process, evidence capture, and manager-approved owner communication. |
| Customer messaging | Stop draft generation and all send/outbox dispatch. | Manual communication only by approved staff/manager, using customer-safe wording and recording exact copy/outcome in the manual log. |
| CRM/review requests | Stop rebooking/review/retention drafts and sends. | No marketing/review asks during incident unless manager explicitly approves after safety/customer-service review. |
| Payments/pricing | Stop payment reminders, checkout link creation, refunds/waivers/discount suggestions, and provider payment commands. | Manager/payment owner reconciles from approved provider/PMS records and records decisions manually. |

## Prevent sends and side effects

Outbound safety is the highest-priority rollback domain because duplicate/wrong/sensitive messages and provider/payment mutations can affect real customers quickly.

### Global outbound kill switch requirements

The system should provide a single operator-visible setting that blocks all outbound execution regardless of workflow-specific configuration. It must apply to:

- email, SMS, portal, and future messaging channels;
- customer-message outbox workers;
- provider/PMS write commands;
- payment checkout-link send/display, refund, waiver, discount, forfeiture, or manual correction commands;
- automated review request/rebooking/campaign sends;
- incident owner notices and daily care updates.

A kill switch is effective only if it blocks at execution time. UI route hiding or frontend-only disablement is not enough.

### Queue and outbox hold procedure

1. Identify affected rows by workflow kind, location, created/updated time, approval state, and subject refs.
2. Mark unsent outbox rows as `HeldByRollback`, `Cancelled`, or `RequiresManualReview` with an audit event.
3. Preserve exact approved payload refs for any already-approved sends. Do not edit payloads in place.
4. For rows in `Sending` or unknown provider status, query provider status only through a read-only/reconciliation path if approved and safe.
5. For duplicate-risk rows, suppress additional sends until idempotency and provider receipt status are reconciled.
6. Any replacement communication requires a new manual draft/approval and a new outbox/communication record; never mutate and resend a prior payload silently.

### Suppression verification

Before declaring outbound stopped, verify all of the following:

- New workflow event cannot enqueue a send-capable outbox row.
- Existing send-capable outbox rows remain held and are not claimed by workers.
- Provider credentials or adapter settings prevent delivery attempts from the app.
- A test recipient message in the affected channel is blocked before provider contact or routed to draft/manual review only.
- Staff UI clearly labels customer-message actions as disabled/manual-only.
- Audit has stop action, affected scope, actor, timestamp, and verification evidence.

## Continue manual workflow

Manual fallback should keep care and customer-service work moving while separating operational facts from later system reconciliation.

### Staff dashboard/manual records fallback

If the app is usable but AI/workers are disabled:

- Use the staff dashboard for read-only lookup, manual task entry, notes, document review, incident intake, and approval records only when those surfaces are verified safe.
- Do not use buttons that send messages, mutate provider/PMS state, execute payment actions, auto-complete care decisions, or clear review gates.
- Add manual notes with:
  - actor and role;
  - time and location;
  - customer/pet/reservation refs;
  - source of fact;
  - action taken;
  - customer-safe summary if needed;
  - internal-only rationale separately labeled;
  - later reconciliation needed.
- Capture approvals explicitly. A manager saying something in chat is not enough unless it is entered or later reconciled as an approval/audit event.

### Spreadsheet/paper fallback

If the app or database is degraded, use a paper/spreadsheet fallback with controlled fields. Store it in a restricted location, not broad chat.

Minimum columns:

| Field | Purpose |
| --- | --- |
| fallback_row_id | Stable row id for later reconciliation. |
| recorded_at / recorded_by | Actor and timestamp. |
| location / operating_day / shift | Operational scope. |
| customer_ref / pet_ref / reservation_ref | Existing system refs if known; avoid unnecessary PII. |
| workflow_type | Inquiry, booking, vaccine, daily care, incident, message, payment, staff task. |
| source_evidence | Document/message/provider/staff observation ref; attach file path only in governed storage. |
| status | Open, blocked, waiting manager, manually completed, needs reconciliation. |
| action_taken | Factual action, not AI rationale. |
| approval_required / approval_actor | Gate and approver if applicable. |
| customer_contacted | Yes/no/channel/time/manual actor; exact copy stored only in restricted comms log. |
| reconciliation_needed | What must be entered back into app/audit later. |

Paper/spreadsheet rules:

- Use the minimum necessary PII.
- Do not store raw payment instruments, secrets, full incident media, raw provider JSON, or broad medical notes in the spreadsheet.
- Mark documents/media by governed evidence refs or physical custody location.
- Keep customer-safe messages separate from internal notes.
- After recovery, reconcile into the application with audit events and retain/archive the fallback sheet according to incident/evidence policy.

### Manual customer communication boundaries

During rollback, staff may communicate manually only within their approved authority. Use conservative wording:

Allowed:

- Acknowledge receipt.
- State that the team is reviewing or will follow up.
- Confirm routine facts already verified from source records.
- Ask for missing information.
- Provide operational status without blaming systems or exposing internal details.

Forbidden without manager/legal/payment/medical approval:

- Booking confirmations/rejections, waitlist promises, capacity/room/group commitments, or policy exceptions.
- Vaccine/medical eligibility conclusions, medication instructions, diagnosis, treatment advice, or contagiousness statements.
- Behavior/aggression/group-play eligibility decisions, restriction clearance, or reinstatement promises.
- Incident fault/liability statements, staff blame, legal/privacy details, or owner notice language for serious incidents.
- Refunds, credits, waivers, discounts, forfeitures, payment status corrections, or cancellation/no-show penalty explanations.
- Apologies or service-recovery promises that imply liability or compensation.
- Review requests or marketing asks while an unresolved incident/complaint/payment dispute exists.

Safe generic customer wording examples:

- "Thanks for your message. Our team is reviewing the details and will follow up with next steps."
- "We are handling this manually today to make sure the information is reviewed by our team before anything is sent or changed."
- "We received the document and a team member will review it. If we need anything clearer or additional, we will let you know."
- "A manager is reviewing this before we provide an update. We will follow up through the usual contact path."

## Preserve data, backups, and audit posture

Rollback must preserve the ability to prove what happened, what was stopped, what was communicated, and what was later corrected.

### No data-loss cleanup during incident

Until the incident commander confirms scope and hold state:

- Do not delete queue rows, audit rows, workflow results, outbox rows, drafts, provider refs, manual fallback logs, prompt manifests, object metadata, or evidence files.
- Do not run retention/anonymization cleanup for affected subjects.
- Do not compact audit history or overwrite current state to hide a failed attempt.
- Do not hard-delete malformed AI outputs or unsafe drafts if they are evidence of a model/tool incident; store/redact according to incident policy.
- Do not remove customer/pet/payment/incident records under deletion request until legal/retention/incident holds are checked.

### Backup/export procedure

For any `Stop pilot`, `Stop outbound`, or `Security/legal hold` event:

1. Capture database backup or scoped export with checksums where tooling supports it.
2. Export current deployment/config/feature flag/policy/template versions without secrets.
3. Export queue/outbox/audit slices for the incident time window and affected subjects.
4. Preserve evidence objects and object metadata hashes for affected documents/media/messages.
5. Record backup/export actor, reason, scope, storage location, checksum/hash, and access policy as audit events.
6. Verify restore/readback on a non-production target when the incident is data-integrity related.

### Restore and repair rules

- Restore to an isolated environment first unless production data is completely unavailable and a human approves emergency restore.
- Reapply deletion tombstones, legal holds, suppressions, opt-outs, and manual corrections before exposing restored data to users/workers.
- Reconcile manual fallback logs into append-only events. Do not backdate or overwrite history; record actual manual action time and reconciliation time separately.
- Re-run workflow events only after current policy validation and explicit human approval when customer/provider/payment effects could result.
- Any repaired customer-facing message or provider action must be a new approved action, not a silent replay of a stale AI result.

## Staff notification and responsibility matrix

### Internal incident announcement template

Use an internal channel/status page message like:

```text
Pet-resort pilot status: MANUAL MODE / [workflow or global] hold
Start: [time/timezone]
Scope: [workflow/location/pilot cohort]
What is disabled: [AI drafts, outbound sends, provider writes, payments, etc.]
What continues: [manual care tasks, dashboard read-only/manual notes, paper fallback]
Customer communication: manual only through [role/channel], no AI-generated sends
Incident commander: [name/role]
Engineering owner: [name/role]
Next update: [time]
```

Do not include secrets, raw customer data, raw incident details, payment data, or unredacted provider payloads in broad announcements.

### Responsibility matrix

| Responsibility | Primary owner | Backup owner | Notes |
| --- | --- | --- | --- |
| Declare stop level and scope | Incident commander / manager | Owner/admin | Err on stop-outbound or stop-pilot for customer/payment/safety risk. |
| Disable automation/queues/runtime | Engineering owner | Owner/admin | Use documented feature flags/config; verify no new claims/model calls. |
| Activate outbound kill switch | Engineering owner + manager | Owner/admin | Verify at execution layer and provider/adapter layer. |
| Preserve backups/evidence | Engineering owner / security owner | Owner/admin | Export with hashes, access scope, and audit. |
| Manual staff workflow | Lead staff / manager | Front desk lead | Use dashboard/manual sheets; record evidence and approvals. |
| Customer communication | Manager/admin | Approved front desk lead | Manual, customer-safe, source-backed; no unsupported promises. |
| Incident/legal/privacy escalation | Manager/admin/security/legal as applicable | Owner/admin | Preserve legal holds and restricted evidence access. |
| Payment/reconciliation decisions | Manager/payment owner | Owner/admin | No refunds/waivers/discounts/provider commands without approval. |
| Recovery verification | Incident commander + engineering owner | Owner/admin | Use decision tree and verification checklist before resume. |
| Post-incident review | Incident commander | Owner/admin | Create corrective actions and update runbooks/tests. |

## Recovery decision tree

### Step 1: Is there active customer, animal, payment, privacy, or legal risk?

- Yes: keep pilot paused or outbound stopped. Continue manual operations. Escalate to manager/security/legal/payment owner as applicable.
- No: proceed to technical and operational scope check.

### Step 2: Is data integrity/audit complete enough to trust current state?

- No: keep app in manual/read-only mode. Restore/reconcile in isolated environment, preserve evidence, and append repair events.
- Yes: proceed to workflow-specific recovery.

### Step 3: Is the affected workflow isolated?

- Yes: keep only that workflow disabled and route it manually. Other verified-safe workflows may remain internal/demo-only if outbound/provider/payment kills remain in place as needed.
- No or unknown: keep global manual-only/stop-pilot until root cause and blast radius are proven.

### Step 4: Can internal-only operation resume?

Internal-only resume may be allowed when:

- auth/session/role checks are verified;
- audit append works;
- database and object storage are healthy;
- outbound kill switch remains active;
- provider/payment write adapters remain disabled;
- AI runtime is disabled/fake or returns failed-safely;
- staff know manual fallback rules.

If these hold, resume `internal_only` or `demo_customer` practice only. This is not approval for live customers.

### Step 5: Can limited pilot behavior resume?

Limited pilot resume requires all internal-only criteria plus:

- incident scope closed or explicitly accepted by human owner;
- affected workflow tests pass in local/staging/demo mode;
- queue/outbox/dead-letter rows reconciled or held;
- duplicate-send and wrong-recipient risk checked;
- manual fallback logs reconciled or scheduled with owner;
- manager signs off on staff readiness and communication boundaries;
- any live customer/contact/payment/provider effects remain within previously approved pilot scope.

If any criterion fails, remain paused or internal-only.

### Step 6: Can automation be re-enabled?

Automation re-enable is separate from pilot resume. It requires:

- human approval for the exact workflow, location, pilot cohort, templates/categories, and allowed action class;
- passing smoke test for the workflow with forbidden side effects checked;
- feature flag change plan and rollback path;
- monitoring owner and next review time;
- audit event linking approval, policy version, config version, test evidence, and re-enable actor.

Auto-send, payment collection, provider writes, vaccine auto-accept, incident closure, and eligibility-affecting decisions remain disabled unless explicitly approved by the relevant gate.

## Recovery verification checklist

Before closing the rollback incident or resuming any automated workflow, verify and record evidence for each item.

### Stop/hold verification

- Global automation status is manual-only or scoped workflow disable is active.
- Affected workers are stopped, scaled down, or unable to claim affected workflow kinds.
- AI runtime calls are disabled/fake/failed-safe for affected workflows.
- Outbound kill switch blocks send/provider/payment execution at backend/adapter layer.
- No send-capable or provider-write outbox rows are in a claimable ready state unless explicitly approved for recovery testing with safe recipients.
- Retry/dead-letter replay is held for affected workflows.

### Data/evidence verification

- Database/export snapshot exists for affected scope and has checksum or readback confirmation.
- Object/evidence refs and hashes for affected documents/media/messages are preserved.
- Audit events exist for declaration, disable actions, queue/outbox holds, backups/exports, manual actions, and recovery tests.
- Manual fallback logs are secured and assigned for reconciliation.
- No retention/deletion/anonymization/cleanup job touched affected subjects during hold unless explicitly approved and audited.

### Manual workflow verification

- Staff know the current mode and disabled functions.
- Manual task/message/document/incident/payment logs are available and restricted to appropriate roles.
- Customer communication owner and approval path are named.
- Customer-safe wording boundaries have been restated to staff.
- Critical care/incident/checkout/payment blockers have named human owners.

### Resume verification

- Root cause or safe containment is documented.
- Smoke test for affected workflow passes in local/staging/demo mode.
- Forbidden side effects are explicitly checked: no customer sends, no provider writes, no payment actions, no auto-approval of medical/vaccine/incident/eligibility decisions.
- Queue/outbox/idempotency state is reconciled so duplicate sends or duplicated tasks will not occur.
- Feature flag/config diff is reviewed by human owner.
- Monitoring window and rollback owner are named.
- Post-incident review is scheduled.

## Post-incident review

Within one business day for stop-pilot/security incidents, or before widening any pilot scope for lower incidents, complete a review with:

- timeline of detection, stop actions, manual operations, recovery, and customer/provider/payment exposure if any;
- root cause and contributing controls that failed or were missing;
- affected customers/pets/reservations/messages/payments/incidents by refs only in broad reports;
- audit/evidence completeness assessment;
- customer communication review and whether any follow-up is needed;
- data retention/legal hold decision;
- corrective actions assigned to owner and due date;
- test/runbook updates needed before resume or expansion.

Post-incident review must not be written by or approved solely by the affected AI workflow. AI may summarize source-backed timeline evidence only after the incident commander explicitly permits use of a safe, scoped prompt packet.

## Minimum implementation requirements before live pilot

These controls should exist before limited real-customer pilot approval:

1. Global backend-enforced outbound kill switch.
2. Workflow-level automation disable flags for every AI/runtime workflow.
3. Runtime mode selection with `DisabledAgentRuntime` fail-safe behavior.
4. Queue/outbox hold/cancel states with audit events and operator visibility.
5. Safe dead-letter/retry controls that do not auto-retry approval/policy/ambiguity failures.
6. Append-only audit for disable, hold, backup, manual reconciliation, and resume actions.
7. Backup/export and restore verification runbook.
8. Manual staff dashboard or spreadsheet/paper fallback template.
9. Staff incident communication template and responsibility matrix.
10. Recovery smoke tests that prove no customer sends, provider writes, payment actions, or autonomous approval decisions occur during rollback.

## Safe default conclusion

If any rollback control is missing, the launch posture remains no-go for live customers and no-go for auto-send/payment/provider effects. Internal-only or demo-customer practice may continue only when the outbound kill switch, manual fallback path, data preservation, audit capture, and staff communication process are verified.
