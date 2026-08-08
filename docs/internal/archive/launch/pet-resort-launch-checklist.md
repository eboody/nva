# Pet Resort Launch Checklist

Status: checklist artifact only. This document does not approve internal pilot launch, live customer use, production deployment, auto-send messaging, provider/PMS mutations, or payment collection. It defines evidence and approval gates that a human launch owner must review before any pilot mode widens.

Primary inputs:

- `docs/launch/pet-resort-launch-readiness.md`
- `docs/roadmap/pet-resort-mvp-implementation.md`
- Current `pet-resort-mvp-implementation` board status as observed from the kanban DB on 2026-06-11:
  - `t_8986d4e5` stack/architecture: `blocked` with `review-required` handoff; artifact exists at `docs/roadmap/pet-resort-mvp-stack.md`.
  - `t_ffcc45ad` project skeleton: `todo`.
  - `t_71c866a7` data model/migrations: `todo`.
  - `t_70c9f6b6` staff dashboard: `todo`.
  - `t_b0d73d7e` inquiry intake slice: `todo`.
  - `t_776c8848` booking triage slice: `todo`.
  - `t_8d95b551` vaccine document slice: `todo`.
  - `t_921dba60` daily care update slice: `todo`.
  - `t_b6372868` incident escalation slice: `todo`.
  - `t_3a72309d` final MVP smoke test/review: `todo`.
- `docs/roadmap/pet-resort-mvp-stack.md`
- `docs/security/pet-resort-security-audit.md` and security/audit parts.
- `docs/workflows/staff-operations.md` and parts.
- `docs/workflows/customer-messaging-agent.md` and parts.
- `docs/workflows/inquiry-intake-agent.md`.
- `docs/workflows/booking-triage-agent.md`.
- `docs/workflows/vaccine-document-agent.md`.
- `docs/workflows/daily-care-update-agent.md` and parts.
- `docs/workflows/incident-escalation-agent.md` and parts.
- `docs/workflows/payments-pricing.md` and parts.

## 1. Launch decision posture

Default until proven otherwise:

- Live customer use: NO-GO.
- Production deployment: NO-GO.
- Auto-send customer messaging: NO-GO.
- Payment collection / refunds / waivers / discounts / forfeitures: NO-GO.
- Provider/PMS writes: NO-GO.
- Autonomous booking, vaccine, incident, behavior, eligibility, or care decisions: NO-GO.

A human may later approve a narrower mode only after the relevant checklist rows have evidence:

1. Internal-only practice with synthetic/demo data.
2. Demo-customer practice using staff/test recipients and visible test labels.
3. Limited real-customer pilot with named/consented customers, one location, manager supervision, manual fallback, and disabled live automation except explicitly approved paths.

No agent or checklist row may convert these into approval by implication. Approval must name actor, role, date/time, scope, pilot mode, location, enabled features, disabled features, rollback owner, and expiration/review date.

## 2. Must-have launch blockers

These are blockers for any pilot that uses real customer data, real customer communications, production/staging integrations with live credentials, or money-adjacent/payment records. Several also block internal-only practice when they would make local testing misleading or unsafe.

### 2.1 MVP implementation and smoke evidence

- [ ] MVP implementation board has no unresolved launch-blocking implementation card.
- [ ] Stack/cutline approval gate is resolved or explicitly scoped as internal-only/demo-only.
- [ ] Project skeleton, local dev, tests, and CI exist and can be run from a clean checkout.
- [ ] Core data model/migrations include customers, pets, reservations, documents, vaccines, tasks, notes, incidents, messages, payments/deposit projections, workflow events/results, review packets, approvals, outbox records, object metadata, and append-only audit events.
- [ ] Staff dashboard surfaces exist for the MVP workflows: today/operations, customer/pet/reservation details, task queues, document review, booking review, message drafts, incident review, and audit history.
- [ ] Final MVP smoke test artifact exists and records exact commands, fixtures, pass/fail results, defects, and forbidden side effects.
- [ ] Smoke tests prove the end-to-end path: event -> durable job -> deterministic/AI result -> schema validation -> review packet/task/draft -> audit.
- [ ] No smoke test depends on production customer data, live sends, live provider writes, or live payment actions unless the human launch scope explicitly allows that mode.

Current blocker note: as of this checklist creation, the implementation board is not launch-ready because stack review is blocked for human review and all downstream code/smoke cards are still todo.

### 2.2 Auth, session, and role gating

- [ ] Staff auth exists for the pilot mode; anonymous access to staff/admin surfaces is denied.
- [ ] Sessions use secure server-owned cookies or equivalent backend-verified session tokens; staff identity and role are extracted server-side.
- [ ] Role/location/subject authorization checks run in the backend/API/domain layer, not only in frontend route guards.
- [ ] Staff, lead staff, manager/admin, system, and AI workflow actors are distinguishable in audit records.
- [ ] Every customer, pet, reservation, document, incident, message, payment, review packet, workflow result, and audit view is subject-scoped and location-scoped.
- [ ] Manager/admin-only actions cannot be executed by ordinary staff roles.
- [ ] AI workflow workers have bounded `DraftOnly` / `ReadEntities` / `ExtractStructuredData` / `FlagRisk` / approved internal-task permissions only; they cannot approve themselves, expand context, mutate provider state, send messages, or move money.
- [ ] Break-glass, privileged export, raw evidence access, role/config changes, retention/legal-hold changes, and production tool grants are denied in pilot unless an owner/admin approval path and audit view exist.
- [ ] Failed authorization emits a safe audit event with denied/blocked reason and no raw sensitive payload.

### 2.3 Backup, restore, and evidence preservation

- [ ] Database backup procedure exists for the pilot environment and has been tested with a restore drill.
- [ ] Object/evidence storage backup or replication policy exists for uploaded vaccine documents, incident media, and other evidence objects used in pilot.
- [ ] Restore procedure replays or preserves deletion tombstones, legal holds, document supersession, customer/pet archive state, and audit integrity metadata before restored data is exposed.
- [ ] Raw evidence objects have content hashes, immutable evidence ids, storage refs, privacy class, scan/quarantine state, retention class, legal-hold state, and subject refs.
- [ ] Document and incident evidence is referenced by id in prompts/logs/audit; raw files, raw OCR, raw message bodies, and raw provider payloads are not copied into ordinary logs or broad dashboards.
- [ ] A staff-visible fallback procedure explains how to continue intake, document review, incident logging, message drafting, and manual approvals if the app is unavailable.
- [ ] Evidence preservation is explicit during rollback: pausing agents or disabling sends must not delete documents, drafts, review records, audit history, or unresolved incidents.

### 2.4 Append-only audit events and review history

- [ ] Audit events are append-only: corrections, voids, reversals, redactions, supersessions, deletion/anonymization, and human overrides append new events referencing prior events.
- [ ] Audit rows include actor, role/scope/session, subject refs, action, source, permission decision, policy refs, approval refs, evidence refs, redacted before/after summaries, AI/tool linkage, idempotency/correlation ids, and data classification/redaction metadata.
- [ ] Security-relevant events are audited: login/session, denied access, role/config changes, workflow enqueue/result validation, review packet creation, approval requested/approved/rejected/suppressed, message draft/send attempt/delivery failure/suppression, document upload/extraction/review, incident triage/closure, payment request/reconciliation/refund/waiver, and retention/deletion.
- [ ] Review queues preserve reviewer identity, timestamp, final decision, edits, rationale, source refs, approval scope, expiration/revocation if relevant, and final side-effect reference if executed.
- [ ] Audit views are role-scoped and redacted: ordinary staff see operational history, managers see approval/evidence for their duties, owner/admin/compliance sees privileged audit/export under purpose-bound controls.
- [ ] Smoke tests include at least one denied/blocked action and prove an audit event is recorded without leaking raw payloads.

### 2.5 Secret safety and prompt/log/client-bundle hygiene

- [ ] No production/staging/test secret appears in source control, docs, frontend bundles, CI logs, browser logs, prompt packets, AI outputs, screenshots, audit projections, or ordinary app logs.
- [ ] `.env.example` contains placeholder values only; local `.env` files are ignored.
- [ ] Payment provider secrets, webhook signing secrets, API keys, database passwords, object-storage keys, model provider keys, and PMS/provider credentials live only in an approved secret store/environment boundary.
- [ ] Frontend build artifacts are checked for accidental `SECRET`, `TOKEN`, `API_KEY`, provider keys, database URLs, webhook signing strings, and live endpoint credentials.
- [ ] Prompt packets include policy refs, evidence refs, safe excerpts, redaction profile, allowed/forbidden action vocabulary, and sensitivity rules; they do not include hidden prompts, raw secrets, raw provider JSON, raw card data, raw documents, or broad unrelated histories.
- [ ] Logs use ids, statuses, schema versions, safe summaries, policy decisions, validation outcomes, and redacted refs only.
- [ ] Tool/model/provider config changes are owner/admin-gated and audited before production or live pilot use.

### 2.6 Controlled messaging boundaries

- [ ] Customer-facing messages default to draft/review, never direct send.
- [ ] Any live send path requires an approved category/template/channel/fact-set policy, exact recipient/destination ref, consent/opt-out state, quiet-hours decision, suppression state, idempotency key, approval actor when required, provider response handling, and immutable approved-payload record.
- [ ] Draft, edit, approval request, approve/reject/return, suppress, queue, send attempt, delivered, failed, bounced, unsubscribe/opt-out, and reply events are separate states with audit history.
- [ ] AI outputs cannot approve, queue, send, suppress, or retry their own customer-facing drafts.
- [ ] Provider retries resend only the exact approved payload through the exact approved path; any content, recipient, channel, template, or fact change requires a new draft and new approval.
- [ ] Missing destination, unknown consent, opt-out, quiet-hours conflict, stale/conflicting facts, unsupported channel, duplicate semantic target, provider failure, or unsafe content blocks send and creates a review/suppression path.
- [ ] Sensitive message categories remain manager/specialist-gated: medical/vaccine/document eligibility, medication/allergy, incident/safety/behavior, payment/refund/waiver/discount/forfeiture, booking confirmation/rejection/waitlist/capacity, complaints, legal/privacy/liability, public responses, and policy exceptions.
- [ ] Smoke tests cover at least: draft-only normal message, suppressed/opt-out message, delivery failure/dead-letter, and blocked sensitive message.

### 2.7 AI output validation and prompt-injection handling

- [ ] Every AI workflow result is parsed as structured output and validated against the shared envelope plus workflow-specific schema before persistence or display as a review packet.
- [ ] Validators reject malformed JSON, wrong subject/event, missing source refs for customer-visible or decision-relevant facts, unknown/forbidden actions, unsafe confidence usage, forbidden side-effect claims, prompt-injection compliance, raw sensitive payload leaks, and customer copy that lacks required gates.
- [ ] AI confidence is recorded as extraction/evidence quality only; it never establishes medical, vaccine, payment, booking, incident, eligibility, legal, or customer-message authority.
- [ ] Customer text, staff notes, OCR, provider payloads, email bodies, transcripts, attachments, and browser/customer-portal events are treated as prompt-injection-capable boundary evidence.
- [ ] Prompt packets declare allowed actions, forbidden actions, source trust/freshness, data categories, logging rules, output schema, review gates, and policy refs.
- [ ] Missing, stale, conflicting, unsupported, sensitive, or policy-ambiguous inputs produce `NeedsHumanReview`, `NeedsMoreInformation`, `RejectedByPolicy`, `FailedSafely`, `Suppressed`, or workflow-specific blocked states.
- [ ] Fake/deterministic AI runtime is available for CI and local smoke tests; real AI runtime remains optional, review-mode only, and disabled/fails safely when not configured.
- [ ] Smoke tests include prompt-injection attempts in inquiry text, OCR/document text, customer message text, and provider-like payloads.

### 2.8 Review queues

- [ ] Review queue rows expose source anchors, source state, review gate, required authority, customer-safe summary, internal-only notes, manager-only rationale, actor/timestamp history, and audit refs.
- [ ] Vaccine/document queue supports upload/source metadata, safe preview/redacted preview, extraction suggestions, policy snapshot, confidence/ambiguity/contradictions, reviewer actions, supersession, and audit-visible reviewer decision.
- [ ] Booking/exceptions queue shows deterministic rule results, missing/stale/conflicting facts, capacity/staffing/payment/vaccine/care/behavior hard stops, manager approvals required, and customer-message draft status.
- [ ] Message queue shows draft category, recipient/destination ref, channel, consent/opt-out/quiet-hours/suppression checks, fact citations, forbidden-claim checks, approval status, delivery status, and failure/retry disposition.
- [ ] Incident queue shows severity candidate, source-backed timeline, missing fields, owner-notice state, temporary flags/restrictions, follow-up tasks, manager/legal/privacy/payment/care/behavior gates, and closure blockers.
- [ ] Payment-sensitive queue or payment review state shows semantic amount/status refs, policy snapshot, provider/reconciliation state, failure/dispute/mismatch, refund/waiver/discount request, manager/payment specialist gate, and no raw card/provider secret data.
- [ ] Queues have dedupe/idempotency keys so reruns update existing open work rather than creating duplicate tasks or duplicate sends.
- [ ] Queue age/SLA views are safe summaries; exact SLAs remain policy-approved values, not invented deadlines.

### 2.9 Incident safety

- [ ] Incident intake captures observations and source refs, not diagnosis, blame, legal conclusions, or unsupported reassurance.
- [ ] Medium/high/emergency classification, downgrades, closure, owner notice, owner-message copy, restriction clearance, behavior/eligibility flags, provider writes, and payment/service-recovery decisions require authorized human approval.
- [ ] Emergency/active safety paths are real-world human/operator procedures; AI must not delay or substitute for them.
- [ ] Incident media and raw/internal narratives are stored by reference with sensitivity, redaction, retention, legal-hold, and owner-sharing review metadata.
- [ ] Owner-facing incident drafts require `CustomerMessageApproval`; manager/admin and legal/privacy/medical/behavior/payment gates apply when content is sensitive.
- [ ] Serious incidents cannot close while required intake fields, owner-notice decision, active restrictions, temporary flags, follow-up tasks, medical/care/behavior/legal/payment gates, or manager review remain unresolved.
- [ ] Review-request/reputation workflows are suppressed or manager-reviewed while an incident, complaint, payment dispute, owner notice, or follow-up remains unresolved.
- [ ] Smoke tests include incident draft creation, manager review routing, no owner-send, no final severity, no closure, and audit of forbidden side effects.

### 2.10 Payment policy and money movement

- [ ] Payment provider selection, production checkout links, live webhooks, payment collection, refunds, discounts, fee waivers, credits, forfeitures, write-offs, manual price changes, and payment-status corrections remain human approval gates.
- [ ] In pilot, payments are manual-only unless a human explicitly approves a scoped, tested, provider-backed path with sandbox evidence and live gate owner.
- [ ] Semantic payment records separate policy/quote/payment request/payment attempt/refund request/reconciliation finding from raw provider payloads.
- [ ] Provider webhooks, if enabled in non-production, verify signatures over raw request bodies before parsing; verified events are stored durably and idempotently.
- [ ] Payment success may update semantic payment/reconciliation status; it does not by itself confirm, cancel, release, check in/out, refund, waive, or mutate a reservation.
- [ ] Failed/late/disputed/partial/expired/ambiguous payments create staff/manager/reconciliation review, not autonomous retries, penalties, cancellations, or customer commitments.
- [ ] Customer-facing payment copy remains draft-only unless exact deterministic template/fact/recipient/channel/send policy is approved.
- [ ] No raw card/bank/CVV/token/webhook-secret/API-key data is stored, logged, prompted, or shown in ordinary UI.

### 2.11 Staff training, runbook, escalation, and fallback

- [ ] Pilot runbook exists and is visible to all participating staff before launch.
- [ ] Staff can explain current mode: internal-only, demo-customer, or limited live pilot; which customers/data are in scope; and which actions are disabled.
- [ ] Escalation matrix names owners for front desk, lead staff, manager/admin, payment/reconciliation, document/vaccine review, incident/legal/privacy, engineering/integration, and launch/rollback decisions.
- [ ] Staff are trained to enter evidence standards: observations over diagnosis, source refs over memory, unknowns instead of guesses, internal notes separate from customer-safe copy, and no secrets/payment details in notes.
- [ ] Staff know that AI drafts/recommendations are not truth, not approvals, and not execution proof.
- [ ] Manual fallback steps exist for intake, booking review, document review, care/daily notes, incident handling, message drafting/manual contact, payment collection/reconciliation, and audit/evidence preservation.
- [ ] Stop-the-line triggers are defined: wrong recipient risk, message sent/queued unexpectedly, auth bypass, raw sensitive data leak, missing audit, data loss/restore failure, payment/provider mutation, incident escalation failure, or AI output acting outside allowed actions.
- [ ] Rollback owner can disable agents/workers, hold queues, revoke model/tool/provider credentials, disable outbox sends, pause pilot, and verify no pending live side effects remain.

## 3. Pilot-mode approval gates that must remain explicit

These approvals are separate. Passing one does not imply another.

### 3.1 Internal-only practice approval

Required evidence before approval:

- MVP local/dev environment works with synthetic/demo data.
- Auth/session/role guard works for staff accounts.
- Workflows can produce review packets/tasks/drafts/audit without live sends, live provider writes, or live payment actions.
- Fake/stub side-effect adapters are the default.
- Staff know the system is not live and all customer-facing output is draft/test-only.
- Backup/restore and audit basics are proven for the test environment.

Still disabled:

- Live customer data unless separately approved.
- Live customer messages.
- Production provider/PMS/payment credentials.
- Payment collection.
- Production deployment.

### 3.2 Demo-customer/test-recipient approval

Required evidence before approval:

- Named test recipients/accounts are listed and labeled.
- All demo outputs are test-labeled or kept internal.
- Message sends, if tested, use only approved test channels/recipients and exact approved templates/copy.
- Wrong-recipient and suppression tests pass.
- Audit can distinguish test/demo actors, recipients, and payloads from real customer operations.
- Manual fallback and rollback owner are on call during the demo.

Still disabled unless explicitly approved:

- Unlabeled live customer sends.
- Any real payment collection or provider mutation.
- Customer eligibility/booking/vaccine/incident/payment final decisions.

### 3.3 Limited real-customer pilot approval

Required evidence before approval:

- Human launch owner approves named location, dates, hours/support coverage, staff participants, workflow slices, named/consented customer scope, and expiration/review date.
- Final MVP smoke tests pass in local/staging and known critical defects are resolved or accepted with mitigation.
- Backup/restore drill and rollback/fallback plan are complete.
- Security/audit/retention role and raw evidence policies are approved for pilot scope.
- Live credentials, if any, are scoped to the minimum required read/write permissions and audited.
- Outbound messaging remains manual/draft-review unless exact template/channel/category is approved.
- Payment collection remains manual outside the app unless exact payment path is approved.
- Incident, medical/vaccine, behavior/eligibility, booking confirmation/rejection, and payment exceptions are manager/human gated.
- Staff training is complete and stop-the-line procedure is rehearsed.

Still not approved by this checklist:

- General live customer rollout.
- Production automation.
- Auto-send messaging beyond approved templates.
- Autonomous payment/provider/booking/vaccine/incident/eligibility actions.

## 4. Post-pilot hardening backlog

These are important but may be scheduled after a constrained internal/demo/limited pilot only if the must-have blockers above are satisfied and the pilot scope explicitly accepts the residual risk.

- [ ] OIDC/SSO or passkey production staff auth, if first-party auth is only a pilot bridge.
- [ ] Dual-control and time-boxed break-glass for privileged raw evidence/export/legal-hold actions.
- [ ] Tamper-evident audit hash chain or WORM storage, if legal/compliance requires it beyond append-only database events.
- [ ] Approved jurisdiction-specific retention periods and automated retention/deletion/anonymization jobs across DB, object storage, search, vectors, backups, and restored snapshots.
- [ ] Production-grade malware scanning for uploads if local MVP uses stub scan states.
- [ ] Full observability stack: OpenTelemetry traces, metrics dashboards, alerting, Sentry/error aggregation, and review-queue SLA dashboards.
- [ ] Automated backup monitoring and periodic restore rehearsal schedule.
- [ ] Dedicated payment reconciliation specialist UI and provider-specific adapter test suite after provider approval.
- [ ] Expanded customer consent/quiet-hours/frequency-cap model and approved template catalog.
- [ ] Safe deterministic auto-send candidates for narrow low-risk templates, with sampling/review and kill switch.
- [ ] Deterministic vaccine auto-accept policy only if the business approves source trust, thresholds, monitoring, rollback, and sample review.
- [ ] Provider/PMS write adapters with typed commands, idempotency, reconciliation, rollback, and approval records.
- [ ] Multi-location/tenant segmentation, cross-location admin policy, and location-specific role hierarchy.
- [ ] Customer portal auth/household scoping and customer-visible history/export/redaction policy.
- [ ] Formal security incident response plan for prompt leakage, unauthorized export, webhook-secret compromise, role misconfiguration, wrong-recipient sends, model/tool misbehavior, and sensitive-data exposure.
- [ ] Load/performance testing and broker-backed queue only if Postgres queue metrics justify it.

## 5. Go/no-go checklist summary

A human go/no-go reviewer should record one of these outcomes:

- `NO-GO`: any must-have blocker remains unresolved for the requested mode, or the MVP implementation/smoke evidence is missing.
- `GO_INTERNAL_ONLY`: safe for synthetic/demo local practice only; no live customer data, sends, provider writes, or payments.
- `GO_DEMO_CUSTOMERS`: safe for named staff/test recipients only, with labels, manual review, rollback owner, and no real customer/provider/payment effects.
- `GO_LIMITED_PILOT_WITH_CONSTRAINTS`: safe for named/consented customers in a named scope only; all high-risk actions remain human-gated and rollback/fallback is active.

Required reviewer record:

- Decision and mode.
- Reviewer/approver name, role, timestamp, and scope.
- Evidence artifacts reviewed, including smoke test result and rollback/fallback plan.
- Open defects accepted with mitigations, or statement that none are accepted.
- Explicit enabled features and disabled features.
- Approval expiration/review date.
- Stop-the-line contacts and rollback owner.

Conservative rule: if source facts, role authority, approval state, policy version, retention class, redaction state, AI schema validity, evidence trust, provider verification, payment truth, or tool permission are missing, stale, conflicting, sensitive, unverified, or outside the approved pilot scope, the launch decision must stay `NO-GO` or route to narrower internal/demo-only practice with explicit human approval.
