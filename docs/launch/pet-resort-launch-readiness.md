# Pet Resort Launch Readiness

Status: integrated launch-readiness artifact. This document summarizes the launch checklist, pilot modes, smoke-test scripts, rollback/manual fallback, go/no-go process, approval gates, and current caveats for the Pet Resort MVP.

Review timestamp: 2026-06-11T07:11:34Z
Workspace: `/home/eran/code/pet-resort-agent-foundation`
Final synthesis task: `t_cd827e7b`

This document does not approve pilot launch, live customer use, production deployment, auto-send messaging, provider/PMS mutation, payment collection, or autonomous safety-sensitive decisions. It defines what evidence a human launch owner must review before making one of the allowed decision shapes.

## 1. Source artifacts

Required child artifacts synthesized:

| Artifact | Purpose |
| --- | --- |
| `docs/launch/pet-resort-launch-checklist.md` | Must-have launch blockers, post-pilot hardening, and evidence checklist. |
| `docs/launch/pet-resort-pilot-mode.md` | Internal/demo/limited-real/one-location pilot taxonomy and approval boundaries. |
| `docs/launch/pet-resort-smoke-test-scripts.md` | Eight local/demo and limited-live smoke scripts with pass/fail criteria and forbidden effects. |
| `docs/launch/pet-resort-rollback-fallback.md` | Stop-the-line, disablement, outbound hold, manual fallback, evidence preservation, and recovery procedure. |
| `docs/launch/pet-resort-go-no-go-review.md` | Final go/no-go process, current decision shape, gates, and caveats. |

Additional inputs consulted:

- `docs/roadmap/pet-resort-mvp-implementation.md`
- `docs/roadmap/pet-resort-mvp-stack.md`
- `docs/security/pet-resort-security-audit.md`
- `docs/workflows/staff-operations.md`
- `docs/workflows/customer-messaging-agent.md`
- `docs/workflows/payments-pricing.md`
- `docs/workflows/incident-escalation-agent.md`
- workflow artifacts for inquiry intake, booking triage, vaccine documents, daily care updates, CRM/retention, workflow events, data model, AI runtime, and integrations.

## 2. Current implementation and launch status

Current implementation-board status observed from the Kanban DB on 2026-06-11T07:11:34Z:

| Card | Status | Launch impact |
| --- | --- | --- |
| `t_8986d4e5` — Define MVP technical stack and integration architecture | done | Architecture artifact exists; human stack/cutline acceptance still matters for pilot scope. |
| `t_ffcc45ad` — Create MVP project skeleton, local dev, tests, and CI | running | Runnable project/test evidence is not complete. |
| `t_71c866a7` — Implement core MVP data model and migrations | todo | Blocks data/audit/workflow smoke evidence. |
| `t_70c9f6b6` — Implement staff dashboard MVP surfaces | todo | Blocks staff readiness and review queue verification. |
| `t_b0d73d7e` — Implement inquiry intake MVP slice | todo | Blocks inquiry path smoke evidence. |
| `t_776c8848` — Implement booking triage MVP slice | todo | Blocks booking/capacity/special-care/payment gate evidence. |
| `t_8d95b551` — Implement vaccine document MVP slice | todo | Blocks vaccine/document gate evidence. |
| `t_921dba60` — Implement daily care update MVP slice | todo | Blocks care-update draft/review evidence. |
| `t_b6372868` — Implement incident escalation MVP slice | todo | Blocks incident safety evidence. |
| `t_3a72309d` — Run final MVP end-to-end smoke test and review | todo | Blocks final launch-readiness evidence. |

Current decision shape:

- `NO-GO_WITH_BLOCKERS` for live customers, limited real-customer pilot, production deployment, auto-send messaging, provider/PMS mutation, and payment collection.
- `GO_INTERNAL_ONLY_PREP` may be considered only for documentation review, fixture planning, and implementation preparation. It is not yet approval for staff practice in a runnable app.

Reason: launch-planning artifacts are now present, but runnable MVP implementation, local/demo smoke results, staff dashboard/review queues, auth/role proof, backup/restore drill, and final smoke/review evidence are still incomplete.

## 3. Human approval gates preserved

These gates require explicit human decisions. Passing documentation review or smoke tests does not imply approval.

| Gate | Required before | Minimum approval scope |
| --- | --- | --- |
| Pilot launch | Any pilot mode beyond internal implementation prep | Approver, mode, scope, date/time, rollback owner, expiration/review date. |
| Live customer use | Any real customer, pet, reservation, evidence, or live operational use | Named customers/pets/location/date window/services/capacity/staff owners/consent. |
| Auto-send messaging | Any outbound message delivered without staff approving the exact payload | Exact category/template/channel/facts/recipient/suppression/idempotency/audit path. |
| Payment collection or payment mutation | Checkout links, charges, refunds, waivers, discounts, credits, forfeitures, status corrections, live webhooks | Provider path, policy, sandbox/live evidence, payment owner, reconciliation/audit controls. |
| Production deployment or provider/PMS mutation | Production deploy, provider write, reservation mutation, customer-portal script, capacity hold/release | Environment, commands allowed, credentials, rollback path, engineering/integration owner. |
| Medical/vaccine/behavior/incident threshold | Eligibility effects, vaccine acceptance, incident severity/closure, owner notice, behavior restriction/clearance | Manager/reviewer authority, policy version, evidence refs, scope, expiration/review. |
| Staff readiness | Any staffed pilot/practice that uses the app as an operational tool | Training completion, runbook access, escalation coverage, manual fallback, evidence standards. |

Approval record must name the decision, approvers, timestamp/timezone, scope, evidence reviewed, enabled features, disabled features, accepted risks/defects, expiration/review date, rollback owner, and stop-line contacts.

## 4. Launch checklist summary

The launch checklist separates must-have blockers from post-pilot hardening.

### 4.1 Must-have launch blockers

These must be satisfied for the requested mode or the decision must stay `NO-GO` or narrow to a safer internal/demo posture:

1. MVP implementation and smoke evidence: project skeleton, local dev, tests/CI, data model, staff dashboard, workflow slices, and final smoke run.
2. Auth/session/role gating: backend-enforced staff auth, role/location/subject scoping, manager/admin-only controls, denied-action audit.
3. Backup, restore, and evidence preservation: DB/object backups, restore drill, raw evidence refs/hashes, no cleanup during incidents.
4. Append-only audit and review history: actor/scope/source/policy/approval/evidence/idempotency linkage for all sensitive state changes.
5. Secret safety and prompt/log/client-bundle hygiene: no secrets/raw payment/provider/documents/hidden prompts in source, logs, UI, prompts, screenshots, or bundles.
6. Controlled messaging boundaries: draft/review by default, exact recipient/channel/template/fact approvals, suppression/opt-out/quiet-hours, immutable approved payloads.
7. AI output validation: schema validation, source refs, prompt-injection handling, confidence not authority, forbidden action rejection.
8. Review queues: document/vaccine, booking exceptions, messages, incidents, payment/cancellation, CRM/review suppression, with dedupe/idempotency and audit.
9. Incident safety: observed facts, manager gates, owner-message approval, no autonomous severity/closure/eligibility flags, unresolved incidents block review/marketing asks.
10. Payment policy and money movement: payment provider/checkout/refunds/waivers/discounts/forfeits/reconciliation remain human-gated; semantic/redacted payment records only.
11. Staff training, runbook, escalation, and fallback: staff know mode, disabled features, evidence standards, stop-line triggers, and manual fallback.

### 4.2 Post-pilot hardening backlog

These are important after a constrained pilot but do not replace must-have blockers:

- Production SSO/passkeys or stronger staff auth if the MVP uses a bridge auth path.
- Dual-control break-glass and privileged export/legal-hold controls.
- Tamper-evident audit hash chain or WORM storage if required.
- Approved jurisdiction-specific retention/deletion/anonymization schedules.
- Production malware scanning for uploads if MVP uses stub scan states.
- Full observability, alerting, review-queue SLA dashboards, and automated restore rehearsal schedule.
- Payment reconciliation specialist UI and provider adapter test suite.
- Expanded consent/quiet-hours/frequency caps/template catalog.
- Deterministic low-risk auto-send candidates only after human approval, sampling, monitoring, and kill-switch proof.
- Provider/PMS write adapters with typed commands, idempotency, reconciliation, rollback, and approval records.

## 5. Pilot mode summary

Use the narrowest mode that answers the current learning goal.

| Mode | Current posture | Allowed only after approval | Always disabled unless separately approved |
| --- | --- | --- | --- |
| Internal-only | Not yet full GO; implementation/smoke evidence missing | Synthetic/demo data, local/dev practice, fake/stub adapters, draft/recommendation AI, audit/review rehearsal | Live customer data/recipients, production credentials, provider writes, payment collection, production deployment. |
| Demo customers/test recipients | NO-GO until internal/local smoke passes | Owner/staff/test recipients, visible TEST/DEMO labels, stub/sandbox/test destinations, staff approval rehearsal | Real customers, unlabeled sends, real money movement, provider mutations, final eligibility/booking/payment/incident decisions. |
| Limited real customers | NO-GO | Named/consented customers, one approved location/scope, manager supervision, draft-only AI, manual approvals, manual payment fallback | Auto-send, live checkout/refunds/waivers/discounts, provider writes, vaccine auto-accept, autonomous booking/incident/payment decisions. |
| One-location pilot | NO-GO | One location, defined services/dates/capacity/support, trained staff, manager/reviewer owners, rollback owner, passing smoke evidence | Multi-location, widened scope, unsupported services, automated sends/payments/provider writes, AI authority to approve/widen itself. |

Default recommendation from the pilot-mode artifact: start with `internal_only` after the MVP can actually run locally with safe adapters and smoke evidence. Move to `demo_customers` only after internal smoke passes. Move to `limited_real_customers` or one-location pilot only after a recorded human go/no-go decision.

## 6. Smoke test summary

The smoke-test artifact defines eight scripts. They must be run after the MVP implementation produces runnable software and exact test commands/fixtures.

| Script | Purpose | Required safety proof |
| --- | --- | --- |
| Happy boarding | Inquiry/request -> reservation review -> staff dashboard -> care note/update -> checkout-ready summary | No auto-confirm, provider write, room/capacity mutation, payment request, review send, or live customer send. |
| Missing vaccine | Upload/request -> extraction/review queue -> safe draft -> eligibility remains gated | No OCR/AI auto-accept; raw docs/OCR/storage keys do not leak; eligibility changes require human review. |
| Full dates/no capacity | Capacity conflict -> waitlist/rejection draft -> manager review | No autonomous rejection/waitlist/capacity hold/provider mutation/send; no invented availability/policy. |
| Special-care pet | Medical/feeding/medication/behavior ambiguity -> care/manager review | No final care/medical/behavior truth from free text/AI; no group-play or eligibility update without approval. |
| Daily update | Staff notes -> AI draft -> preview/edit/approval/audit | No live send by default; no invented meals/play/photos/reassurance; internal-only notes suppressed. |
| Incident draft | Incident capture -> severity suggestion -> manager review -> owner-message draft | No autonomous severity finalization, closure, owner send, behavior flag, diagnosis, blame, liability, or refund promise. |
| Cancellation | Cancellation request -> task/audit/refund/payment-sensitive manual gate | No provider cancellation, refund, waiver, discount, forfeit, payment correction, or payment-sensitive send without approval. |
| Review request | Completed safe stay -> CRM/review draft -> suppression/preferences/review gate | No autonomous review request, marketing/rebooking/winback/discount, complaint response, or suppression override. |

Overall smoke pass requires:

1. All eight scripts pass in local/demo mode against current MVP fixtures.
2. Every customer-facing message remains draft/review-gated unless exact deterministic policy is approved.
3. No production provider/PMS/payment/live messaging side effect occurs during local/demo execution.
4. No secrets or raw sensitive content leak to ordinary logs, client bundles, prompt safe logs, screenshots, audit summaries, or customer-visible drafts.
5. Review queues exist for document/vaccine, booking exceptions, care/special handling, incidents, payment/cancellation, messaging approval, and CRM suppression/review.
6. Audit events reconstruct source -> deterministic/AI result -> validation -> review/draft -> approval/blocked effect.
7. The final smoke/review card records exact runnable commands, fixtures, defects, and blockers.

Any live customer/provider/payment effect without explicit approval, secret leak, AI authority over high-risk state, missing audit chain, unavailable rollback controls, broken staff queues, failed happy path, failed suppression/consent, or bypassable incident/vaccine/payment/special-care gate is a launch blocker.

## 7. Rollback and manual fallback summary

Rollback default: fail closed, disable automation first, preserve evidence, continue manually, and resume only after human verification.

### 7.1 Stop levels

| Level | Examples | Required posture |
| --- | --- | --- |
| Watch | Non-customer-affecting defect, isolated local smoke failure, redacted log issue | Keep pilot constrained; assign owner; do not widen. |
| Stop workflow | One unsafe/degraded workflow | Disable affected workflow and route manually. |
| Stop outbound | Wrong-recipient/duplicate/unapproved send risk, suppression/quiet-hours failure | Activate outbound kill switch; manual manager-approved communication only. |
| Stop pilot | Data integrity, audit, auth/role, incident safety, payment/provider mutation, cross-workflow policy risk | Pause pilot effects; continue manual/internal-only operations. |
| Security/legal hold | Data exposure, unauthorized access, credential compromise, raw payment/medical/incident leak, evidence loss | Preserve records; suspend destructive jobs; involve owner/admin/legal/security/compliance. |

### 7.2 Immediate stop-the-line checklist

1. Assign incident commander and record scope, trigger, affected workflows, mode, location, and exposure.
2. Stop AI/workflow processing: set manual-only, disable runtime adapters, stop claimers/workers, hold queues, disable replay/dead-letter reruns.
3. Prevent outbound sends/provider/payment effects: activate global kill switch, hold outbox/provider attempts, disable delivery/write/payment credentials or adapters, do not drain by sending.
4. Preserve data/evidence: snapshot DB/scoped tables, object metadata/evidence refs, deployment/config versions without secrets, safe logs, queue/outbox/audit slices.
5. Notify staff and switch to manual operations with clear disabled functions, manual customer communication boundaries, owners, and next update.

### 7.3 Minimum rollback controls before live pilot

- Global backend-enforced outbound kill switch.
- Workflow-level automation disable flags for every AI/runtime workflow.
- Runtime mode selection with fail-safe disabled/fake behavior.
- Queue/outbox hold/cancel states with audit events and operator visibility.
- Safe dead-letter/retry controls that do not auto-retry approval/policy/ambiguity failures.
- Append-only audit for disable, hold, backup, manual reconciliation, and resume actions.
- Backup/export and restore verification runbook.
- Manual staff dashboard or spreadsheet/paper fallback template.
- Staff incident communication template and responsibility matrix.
- Recovery smoke tests proving no customer sends, provider writes, payment actions, or autonomous approvals occur during rollback.

If any rollback control is missing, live customers, auto-send, payment, and provider effects remain `NO-GO`.

## 8. Go/no-go decision process

Use this process for every launch decision or scope widening.

### Step 1: Name requested mode and scope

Record the requested decision shape:

- `NO-GO`
- `GO_INTERNAL_ONLY`
- `GO_DEMO_CUSTOMERS`
- `GO_LIMITED_PILOT_WITH_CONSTRAINTS`

Also record location, dates, services, capacity, staff roles, customers/test recipients, environment, enabled workflows, disabled workflows, credentials/adapters, and rollback owner.

### Step 2: Check mode-specific entry evidence

- For `GO_INTERNAL_ONLY`: runnable local app, demo data, auth/roles, fake/stub adapters, audit/review packet/draft creation, backup/restore basics, smoke safety cases, rollback controls.
- For `GO_DEMO_CUSTOMERS`: all internal evidence plus named test recipients, visible test labels, stub/sandbox/test destinations, wrong-recipient/suppression tests, demo audit separation, rollback owner on call.
- For `GO_LIMITED_PILOT_WITH_CONSTRAINTS`: final MVP smoke pass plus named/consented customers, one scope, trained staff, approved security/audit/retention defaults, backup/restore drill, rollback/fallback drill, exact enabled/disabled features, and all required approval owners.

### Step 3: Run blocker review across six dimensions

1. Critical bugs: auth/session, data loss, audit gaps, message/payment side effects, broken MVP path.
2. Operational gaps: staff owner, manual fallback, review queues, incident escalation, support coverage.
3. Unsafe AI behavior: hallucinated facts, unvalidated output, prompt leakage, autonomous approvals/sends/charges.
4. Incomplete policies: pilot mode, live customer scope, messaging, payments, incident/medical/vaccine thresholds.
5. Staff readiness: training, runbook, escalation, support coverage, evidence standards.
6. Customer-facing risk: wrong recipient, sensitive wording, unsupported commitments, privacy/payment/medical risk.

Any unresolved blocker either keeps `NO-GO` or narrows the mode.

### Step 4: Record approval or no-go

Required approval record:

```text
Decision: NO-GO | GO_INTERNAL_ONLY | GO_DEMO_CUSTOMERS | GO_LIMITED_PILOT_WITH_CONSTRAINTS
Approver(s): [name, role]
Recorded at: [timestamp/timezone]
Scope: [location, dates, services, capacity, staff, customers/test recipients]
Evidence reviewed: [docs, builds, smoke run ids, backup/restore evidence, rollback drill]
Enabled features: [exact list]
Disabled features: [exact list]
Accepted defects/residual risks: [none, or each defect with mitigation and owner]
Approval expiration/review date: [date]
Rollback owner and stop-line contacts: [names/roles/contact path]
Customer communication owner: [name/role]
Payment/provider owner if applicable: [name/role]
Security/privacy/incident escalation owner: [name/role]
```

Missing fields are not harmless. They should narrow the decision or keep it `NO-GO`.

### Step 5: Monitor and stop on triggers

Stop or narrow the pilot immediately on wrong recipient risk, unapproved send, duplicate send, opt-out/quiet-hours bypass, payment/provider mutation, raw sensitive data leak, missing audit, auth bypass, data loss/restore failure, incident escalation failure, manager queue unavailable, staff authority confusion, customer complaint, legal/privacy concern, or AI output acting outside allowed actions.

## 9. Current caveats and blockers

Current launch blockers:

1. Runnable MVP implementation is incomplete.
2. Project skeleton/local dev/tests/CI are still running/incomplete.
3. Core data model/migrations are not implemented.
4. Staff dashboard and review queues are not implemented/verified.
5. Workflow slices for inquiry intake, booking triage, vaccine documents, daily care updates, and incident escalation are not implemented/verified.
6. Final end-to-end smoke test/review is not run.
7. Auth/session/role gating, subject/location scoping, append-only audit, backup/restore, prompt/log secret safety, and side-effect disablement are policy requirements but not yet proven in a running app.
8. Staff training, runbook, named owners, escalation coverage, and manual fallback rehearsal are not evidenced.
9. Human approval gates for pilot launch, live customers, auto-send messaging, payment, production/provider mutation, and medical/vaccine/behavior/incident thresholds are not granted.

Current non-blocking accomplishments:

- Launch checklist exists and names must-have blockers/post-pilot hardening.
- Pilot mode artifact defines internal/demo/limited-real/one-location modes and widening boundaries.
- Smoke scripts exist for eight critical launch paths and specify forbidden side effects.
- Rollback/fallback plan exists and defines disablement, outbound hold, manual workflow, evidence preservation, responsibility matrix, and recovery verification.
- Go/no-go review process exists and preserves human approval gates.

## 10. Final recommendation

Recommended current decision: `NO-GO_WITH_BLOCKERS` for live launch and limited real-customer pilot.

Recommended safe next step: complete MVP implementation, then run the final end-to-end local/demo smoke/review card. After runnable evidence exists, a human launch owner can use this artifact and the supporting docs to choose the narrowest safe mode:

1. `GO_INTERNAL_ONLY` for synthetic/demo local practice;
2. `GO_DEMO_CUSTOMERS` for owner/staff/test-recipient rehearsal;
3. `GO_LIMITED_PILOT_WITH_CONSTRAINTS` only after explicit live-customer scope approval and passing evidence.

Until then, live customers, production deployment, auto-send messaging, provider/PMS writes, payment collection/mutation, and autonomous safety-sensitive decisions remain `NO-GO`.
