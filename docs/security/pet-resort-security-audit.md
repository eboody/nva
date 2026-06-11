# Pet resort security, permissions, audit, and AI governance model

Status: draft canonical integration artifact for MVP planning and review. This document synthesizes the security/audit workstream into implementation-ready defaults, but it does not approve production role authority, production AI tool grants, customer-message automation, provider mutations, or retention periods. The role matrix, AI governance policy, and retention policy remain explicit human approval gates before production use.

Primary inputs:

- `docs/security/pet-resort-security-audit-parts/inputs.md`
- `docs/security/pet-resort-security-audit-parts/roles-permissions.md`
- `docs/security/pet-resort-security-audit-parts/sensitive-data.md`
- `docs/security/pet-resort-security-audit-parts/audit-requirements.md`
- `docs/security/pet-resort-security-audit-parts/ai-governance.md`
- `docs/security/pet-resort-security-audit-parts/data-retention.md`

## 1. Security model

### 1.1 Security posture

The pet-resort application owns business state, policy evaluation, permissions, validation, durable queues, idempotency, side-effect execution, and append-only audit. Hermes/LLM/AI workers are bounded assistant actors that receive least-privilege prompt packets for a single workflow and return structured drafts, recommendations, classifications, summaries, extraction candidates, risk flags, or review packets.

Default security posture:

1. Default deny. If a capability is not explicitly granted by role, subject scope, workflow purpose, state, policy version, and approval state, the system must deny, suppress, block, redact, or route to review with audit evidence.
2. Source-of-truth state lives in typed application/provider records, not in AI output, screenshots, OCR, raw webhook JSON, free text, staff notes, customer claims, or model memory.
3. Free text is evidence, not authority. It may suggest facts, but cannot by itself establish payment status, vaccine compliance, medical/care truth, booking acceptance, refunds/waivers, incident conclusions, or customer-facing commitments.
4. State transitions, approvals, denied actions, suppressed actions, validation failures, dead-letter outcomes, and external-system attempts are security-relevant events and must be audited.
5. High-risk actions require deterministic policy, role permission, source evidence, explicit approval when applicable, idempotency, before/after or redacted diff, and audit linkage.
6. AI confidence is never authority for medical, vaccine, payment, booking, incident, eligibility, legal/liability, safety, or customer-message outcomes.
7. Raw sensitive data belongs in governed evidence stores with retention/legal-hold controls, not ordinary logs, broad staff views, or unbounded model prompts.

### 1.2 Protected subjects

Canonical subjects requiring permission checks and audit:

- Customer/account: contact PII, portal refs, preferences, messages, payment links/status, reservations, pets, and deletion state.
- Pet: care profile, medication/allergy/medical facts, behavior, temperament, vaccine facts, documents, notes, incidents, and eligibility state.
- Reservation: lifecycle state, service, location, dates, capacity/room/suite assignment, tasks, documents, messages, payment/deposit refs, incidents, and external provider refs.
- Service and capacity resources: service offering, room/suite, yard, groomer/trainer/labor slot, capacity hold/release, occupancy/cleaning/out-of-service state.
- Staff/user: staff actor, role/capability grants, assignments, approvals, notes, security/admin actions, and privileged access.
- Task/review packet: workflow work item, escalation, source event, assignment, priority, evidence, review gate, and completion/cancellation state.
- Document and vaccine record: raw file, OCR/extraction, source evidence, reviewer decision, verified/superseded/rejected/exception state, retention/legal hold.
- Care note and internal note: factual observations, customer-safe summaries, internal-only rationale, corrections, voids, and visibility labels.
- Incident: report, severity, evidence, investigation, customer communication, manager/legal review, resolution, closure, reopen, and follow-up.
- Message: inbound/outbound body, draft/review/send state, channel, recipient, provider status, preference/consent, suppression and delivery events.
- Payment/deposit: semantic amount/status, due dates, provider refs, checkout/payment events, exceptions, approvals, reconciliation, and disputes.
- AI workflow result: prompt manifest/hash, output envelope, schema validation, risk flags, recommended actions, tool calls, review state, and safe log summary.
- Audit event: append-only record of actor, subject, action, source, policy, permission decision, approval, evidence, before/after, AI/tool linkage, and integrity refs.

### 1.3 Trust boundaries

| Boundary | Trusted side | Untrusted or constrained side | Required controls |
| --- | --- | --- | --- |
| Customer portal and inbound customer messages | Authenticated customer identity, approved portal fields, own-household scope | Uploaded files, free-text claims, message bodies, customer-supplied dates/amounts/status claims | AuthN/AuthZ, subject scoping, file quarantine, validation, rate limits, preference/consent checks, audit |
| Staff UI and operational queues | Authenticated staff role, assignment/location scope, approved workflow actions | Free-text notes, manual corrections, staff claims, accidental overreach | Capability checks, separation of duties, required review gates, redacted views, append-only corrections, audit |
| Manager/admin/security controls | Approved privileged roles and purpose-bound access | Broad evidence/audit/export/config power | Least privilege, break-glass limits, dual-control where possible, export logging, role/config audit, post-access review |
| Provider/PMS/payment/webhook adapters | Verified signatures, mapped semantic provider events, approved typed commands | Raw webhooks, provider JSON, screenshots, unsigned events, external ids, provider errors | Verify before parse where possible, durable inbound event store, dedupe by provider/account event id, idempotency, reconciliation, raw-payload minimization |
| Document/object storage | Governed evidence objects, hashes, scan/quarantine state, retention metadata | Raw uploads, OCR text, images, signed URLs, malware, prompt injection | Private storage, malware/quarantine, immutable hashes, evidence refs instead of raw URLs, role-scoped access, retention/legal hold |
| AI/Hermes runtime | Application-built prompt packet, declared schema, allowed actions, validators | Model output, prompt-injected user/provider/OCR text, model memory, tool prose | Least-privilege packet, structured JSON-only outputs, schema/policy/permission validation, no hidden side effects, safe logging, human review gates |
| Logs, analytics, search, exports, and backups | Redacted projections, safe summaries, hashes, retention classes | Raw PII, medical/payment/incident data, prompt/tool traces, internal notes | Redaction, access control, purpose-scoped exports, retention jobs, restore tombstone replay, audit of privileged access |
| Deterministic system actor | App-owned policy evaluator, state machine, queue, audit writer | Missing/stale/contradictory policy or source data | Fail closed, no invented policy, emit denied/blocked/audit events, require approval refs for high-risk mutations |

## 2. Role and permission matrix

Capability vocabulary:

- `R`: read a role-scoped projection.
- `C`: create a record, request, draft, task, upload, or evidence item.
- `U`: update within allowed field/state boundaries.
- `D`: delete, archive, void, retire, supersede, or suppress. Destructive actions should normally be soft-delete/supersede with audit.
- `A`: approve, verify, resolve a gate, or authorize an effect.
- `S`: send, queue, or execute a customer/provider/system action after required checks.
- `X`: export, download, bulk view, privileged inspect, or access governed raw evidence.
- `M`: administer policy, roles, integrations, tool grants, retention, or security configuration.
- `Own`: limited to the customer's household.
- `Assigned`: limited to assigned shift/task/reservation/pet or operational queue.
- `Location`: limited to authorized location/business unit.
- `Privileged`: purpose-bound owner/admin, compliance, manager, or break-glass access.
- `Draft`: may draft/recommend only; no authority or side effect.

Human approval gate: this matrix is the draft implementation target. Production must not treat it as approved authority until the business approves exact roles, delegated capabilities, thresholds, and optional specialist roles.

| Domain | Customer | Staff | Lead staff | Manager | Owner/admin | System | AI workflow worker |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Customers/accounts | `R/C/U Own` for profile/contact/preferences through approved fields; no merge/export/admin. | `R Assigned/Location`; `C/U` routine intake/contact corrections; no merge/delete/export. | Staff rights plus duplicate/intake routing and routine correction review. | `R/C/U/A Location`; approve merges, do-not-contact conflicts, sensitive corrections. | `R/C/U/D/X/M Privileged`; account lifecycle, role/admin, exports, retention/legal holds. | Validate identity links, enforce preferences, dedupe, audit. | Read minimized customer facts; draft missing-info or merge-review suggestions only. |
| Pets | `R/C/U Own` profile and approved care fields; no staff-only risk/status decisions. | `R Assigned`; `C/U` routine observations/care profile within task; no final medical/behavior eligibility. | Staff rights plus routing/lead review for routine care or behavior flags. | Approve medical, behavior, group-play, and high-risk pet status exceptions. | Privileged pet admin, export, retention/legal hold. | Enforce required fields/state transitions, source provenance, audit. | Read scoped pet facts; extract/summarize/recommend; no final care-truth or eligibility mutation. |
| Reservations | `R Own`; `C` inquiry/request; `U Own` limited request details before locked states. | `R Assigned/Location`; `C/U Draft` intake/check-in prep; no exception approval. | Staff rights plus queue routing/waitlist/blocked-task triage where policy permits. | Approve accept/reject/cancel/no-show/overbooking/waitlist/capacity/holiday/ratio exceptions. | Configure reservation policy, cross-location overrides, privileged exports/admin. | Apply deterministic state checks and holds/releases only under approved policy/approval refs. | Booking triage, gap flags, follow-up drafts; no live status/provider mutation. |
| Vaccine/document records | `R Own` customer-safe status; `C/U` uploads/replacements; no verification/delete. | `R Assigned`; `C` upload/import; `U Draft` classification metadata; no final verify/waive. | Staff rights plus review-queue routing. | `A` verify/reject/exception where policy permits; approve eligibility effects and sensitive wording. | Raw evidence export, retention/legal hold, document policy config. | Quarantine, hash, OCR pipeline, supersession, eligibility recompute after approved verification. | OCR/extraction candidates with evidence/confidence; no final valid/expired/waived decision. |
| Tasks/review packets | `R Own` customer-visible status only if exposed; may create requests/messages. | `R/C/U Assigned`; complete routine tasks with evidence; escalate blocked/high-risk tasks. | `R/C/U/A Location` lead-level routing, reassignment, ordinary blocked resolution. | Approve manager-review tasks, exceptions, incident/payment/message gates. | Administer templates, triggers, queues, exports. | Create deterministic tasks from approved triggers; enforce dedupe/idempotency/SLA states. | Draft/recommend internal tasks only where trigger/task/assignee/rate policy is approved. |
| Notes/care/internal notes | `R Own` approved customer summaries only; `C` customer requests. | `R/C/U Assigned` factual care notes; no customer publication or privileged notes unless needed. | Staff rights plus lead handoff notes and ordinary review. | Approve customer summaries, sensitive corrections/voids, incident-linked notes. | Privileged note export, retention/legal hold, redaction override. | Enforce note states, corrections/voids, visibility separation, audit redaction. | Summarize/draft customer-safe wording from scoped evidence; no publication or hidden side effect. |
| Incidents | `R Own` only approved communications/status; `C` customer report/message. | `R/C Assigned` observations/evidence; no close/downgrade/customer notice authority. | Staff rights plus initial severity routing/escalation packet. | `R/C/U/A/S Location` investigation, severity, owner notice, closure/reopen, customer communications. | Cross-location incident export, legal/compliance escalation, retention/legal hold. | Enforce workflow states, evidence immutability, chain of custody. | Summarize/classify risk/draft packets; no severity finalization, blame, diagnosis, closure, or send. |
| Messages | `R Own`; `C` inbound messages; `U` preferences within policy. | `R Assigned`; `C/U Draft`; may send only routine approved/template messages if policy permits. | Staff rights plus queue routing and ordinary template-reviewed sends if approved. | Approve/send sensitive medical, behavior, incident, payment, legal, eligibility, complaint, or exception messages. | Configure templates/channels/automation/suppression/export. | Enforce preferences/opt-outs/suppression and deterministic send paths. | Draft, summarize, risk-flag, recommend suppression; no direct send except later approved deterministic path. |
| Payment status | `R Own` customer-safe status; `C` payment initiation through provider flow; no manual status/refund/waiver. | `R Assigned` semantic status; create failed-payment/collection tasks; no provider command. | Staff rights plus reconciliation task triage. | Approve waivers, discounts, refunds, forfeits, write-offs, manual corrections, payment-sensitive language. | Configure providers/policies, privileged exports, reconciliation admin. | Verify webhooks, dedupe, reconcile, apply approved idempotent commands. | Read minimized semantic status; draft reminders/escalations; no money movement or financial-truth mutation. |
| Audit events | `R Own` customer-visible history only if product exposes it. | `R Assigned` safe operational audit summaries. | Staff rights plus location operational history for triage. | `R/X Location` approval/evidence views for duties. | `R/X/M Privileged` audit policy, exports, legal hold, retention/redaction; cannot rewrite history. | Append-only audit writer, integrity checks, redacted projections. | Receive minimal refs needed for traceability; write result metadata only through app runtime. |
| AI outputs/config | May see only approved customer-facing AI-assisted content where exposed. | Review assigned drafts/recommendations; no model/tool grants. | Route AI review queues and return drafts. | Approve workflow-specific AI drafts/outcomes and safe automation candidates where policy permits. | `M` model/provider/prompt/tool grants, automation classes, retention/logging, break-glass. | Validate output, enforce allowed actions, create review packets, record audit. | Produce schema-bound outputs; cannot approve itself, expand context, change tools, or alter policy/config. |

Separation-of-duties rules:

1. AI/OCR may extract vaccine/document facts; it cannot be the verifier. Human document review or a later approved trusted integration must verify, reject, waive, or apply eligibility effects.
2. Staff may report incidents; managers own severity-affecting changes, customer communications, resolution, closure, reopening, and legal/privacy escalation. The reporter/drafter should not be the sole closer for high-risk incidents.
3. AI cannot approve, send, execute, or suppress its own customer-facing draft. A human or deterministic approved send path must approve the exact effect, recipient, facts, template/category, and timing.
4. Provider webhook success does not equal business approval. System reconciliation may update semantic provider state, but refunds, waivers, discounts, forfeits, manual price/status changes, and customer commitments require authorized approval.
5. Exports, raw evidence access, privileged audit, AI prompt/output/tool trace inspection, role/config/tool changes, retention/legal holds, and break-glass access are privileged, purpose-bound, time-limited where possible, and audited.
6. Missing, stale, contradictory, sensitive, unverified, or out-of-scope facts must produce denied/review/blocked/suppressed outcomes, not broadened access or hidden mutation.

## 3. Sensitive data category matrix and handling rules

| Category | Sensitivity | Default access | Edit/approval | Export/logging/prompt handling |
| --- | --- | --- | --- | --- |
| Public/business-safe policy and service metadata | Low | Tenant/location scoped business users; customer-facing where approved | Owner/admin or approved policy owner for canonical copy | Safe for docs and approved templates; still avoid cross-tenant leakage. |
| Internal operations/capacity/tasks | Confidential | Staff/lead/manager by assignment/location; customer only customer-safe status | Staff/lead update routine tasks; manager approves exceptions | Logs may use ids/statuses/safe summaries; prompt only workflow-relevant queues/refs. |
| Contact information | Confidential; emergency/vet contacts can be Restricted | Customer own; staff when needed for intake/messaging/emergency; manager/admin for conflicts/merge | Customer/staff routine updates; manager/admin for duplicate merge, do-not-contact, emergency conflicts | Mask in broad views; include minimal recipient/channel/preference in prompts; audit exports. |
| Pet medical/care information | Restricted | Assigned care/document/incident/manager workflows; customer only approved own-pet display | Customers/staff submit facts; medical ambiguity, medication exceptions, eligibility effects need review | Prefer flags/evidence refs; redact from dashboards/logs; prompts require source/trust/freshness/review gates. |
| Vaccine documents/OCR | Restricted | Customer upload/status; document reviewers/managers raw evidence by assignment; staff usually status only | AI/OCR suggests; human/trusted integration verifies/rejects/waives; raw deletion retention-gated | Private evidence storage; no raw URLs/OCR in logs; prompts only for document workflow; use refs elsewhere. |
| Incidents/evidence | Restricted | Assigned staff for reporting; manager/legal/compliance for investigation; customer approved summaries only | Staff report; manager approves severity, notice, closure, reopen, compensation ties | Separate factual timeline/internal rationale/customer copy; no broad raw media/narratives; privileged exports only. |
| Payments/payment status | Restricted; raw instruments/secrets are Secret | Customer semantic own status; staff semantic status; managers/payment roles provider refs as needed | Provider statuses reconciled by system; money movement/exceptions require human approval | Never store/log/prompt raw card/bank/CVV/tokens/webhook secrets; raw provider payloads short-lived/governed. |
| Staff notes | Confidential or Restricted | Staff/lead/manager by need-to-know; customer only approved summaries | Staff record factual notes; sensitive corrections/customer publication need manager/review | Label visibility; separate factual/interpretive; redact from broad views and customer exports. |
| Internal-only notes | Restricted or Secret if security content | Manager/owner/admin/legal/compliance/system by purpose | Authorized roles only; downgrade to customer-safe copy requires separate approved artifact | Never co-mingle with customer-visible fields; exceptional AI access only with explicit prompt rules and redacted output. |
| AI drafts/recommendations | Confidential; Restricted when derived from sensitive sources | Workflow reviewers by subject/assignment; customer only approved final published content | Human/system may accept/edit/reject/suppress under policy; AI cannot approve itself | Store structured envelope/safe summary; raw prompt/completion/tool traces governed and retention-gated. |
| Provider/integration raw data | Restricted; secrets are Secret | System/engineering/payment/compliance by purpose | Verified, mapped, deduped before business use; provider commands require typed approval path | Verify signatures, avoid broad raw retention, log semantic refs/error classes, never expose secrets. |
| Secrets/internal security | Secret | System/owner/admin/security only via secret manager; not ordinary app data | Rotation/config requires privileged audited action | Never enter prompts, logs, audit projections, exports, staff views, or screenshots. |

Safe-handling rules:

1. Every field and artifact should have data category, subject refs, retention class, redaction profile, and access policy.
2. Prompt packets must declare sensitivity, sensitive fields, customer-text rules, logging rules, allowed output classes, forbidden copying rules, and source trust/freshness.
3. Use role-scoped projections. Broad staff dashboards should show safe summaries, flags, review gates, and next actions, not raw medical/payment/incident/provider/AI traces.
4. Customer-visible projections must exclude internal-only notes, staff commentary, manager rationale, raw audit, raw provider payloads, AI reasoning/tool traces, and unrelated staff/customer data.
5. Export/download requires actor, purpose, subject scope, approval if privileged, content classification, redaction profile, created_at, and audit event id.
6. Ordinary logs should contain ids, statuses, schema/version, policy outcomes, safe summaries, hashes, and redacted evidence refs, not raw documents, OCR, message bodies, incident narratives, payment data, provider JSON, hidden prompts, or secrets.
7. Customer/provider/staff/OCR text is prompt-injection-capable event content; it must never override system/developer policy or allowed action vocabulary.

## 4. Audit model

### 4.1 Audit invariants

Audit is append-only by default. Corrections, redactions, reversals, voids, supersessions, deletion/anonymization actions, and human overrides are later events that reference prior events. No actor, including owner/admin or system, can rewrite historical audit chronology.

Every security-relevant event must capture:

- `audit_event_id`
- `occurred_at` and `recorded_at`
- `actor`: customer, staff, lead staff, manager, owner/admin, system, AI workflow worker, external integration/provider, or privileged specialist if later approved
- `actor_role`, `actor_scope`, and `auth/session/service identity` where applicable
- `subject_refs`: customer, pet, reservation, document, vaccine record, incident, message, payment, task, AI run, provider event, role/config, retention/legal hold
- `action` and taxonomy
- `source`: UI/API/webhook/queue/job/tool/manual import plus source event id
- `permission_decision`: allowed, denied, blocked, suppressed, review_required, failed_validation, failed_safely, dead_lettered
- `policy_refs`: role matrix, workflow version, retention policy, AI governance, pricing/payment policy, message template, state-machine version
- `before` and `after` safe summary or redacted field diff; raw values only by governed evidence refs
- `evidence_refs`: document ids/hashes, provider event ids, message ids, source excerpts, screenshots/media refs, reviewer notes, prompt/output refs as allowed
- `approval`: required gate, approval id, approver, decision, reason, timestamp, scope, expiration if applicable
- `ai_linkage`: runtime_call_id, model/provider/config ref without secrets, prompt manifest/hash, schema/version, output status, validation result, risk flags, suggestion id
- `tool/provider_linkage`: typed command, allowed-action mapping, idempotency key, provider result, reconciliation state, retry/dead-letter state
- `redaction`: data categories, redaction profile, masked fields, raw evidence storage class
- `integrity`: tenant/location, sequence or ordering key, hash/signature if used, correlation/causation ids

### 4.2 Required audit events

Audit these events at minimum:

| Event family | Required events |
| --- | --- |
| Identity/account | Customer created/updated/merged/archived/deleted/anonymized; pet created/updated/archived/deleted/merged; portal linked/unlinked; contact preference/opt-out/do-not-contact changes; duplicate merge approval. |
| Documents/vaccines | Upload/import/quarantine/scan/classify/extract/extraction failure; suggested facts; review requested; verify/reject/waive/exception; supersede/archive/delete/export; eligibility recompute; raw document/OCR access. |
| Reservations/capacity | Inquiry/request/missing-info/vaccine-pending/special-review/waitlist/offered/confirmed/check-in/active/check-out/cancel/reject/no-show; capacity hold/extend/release; room/suite assignment; overbooking/waitlist/holiday/ratio exception request/approval/rejection. |
| Care and notes | Care task create/assign/start/complete/block; medication skipped/exception; care note created/corrected/voided; internal-only note access; customer-safe summary drafted/approved/published/suppressed. |
| Incidents | Report, evidence attach/access/export, severity change, manager review requested, investigation update, customer draft, approval/rejection/send/suppress, legal/privacy escalation, resolution/closure/reopen, refund/credit linkage. |
| Messages | Inbound received, draft created/edited, approval requested, approved/rejected/returned, queued, send attempted, delivered/failed/bounced/suppressed, unsubscribed/opt-out, reply received, provider status reconciliation. |
| Payments | Quote/policy snapshot evaluated; deposit required/waived/failed/paid; checkout created/sent/expired; webhook verified/rejected/deduped; reconciliation result; failed-payment task; refund/discount/waiver/forfeit/write-off/manual correction request/approval/rejection/execution; provider command result. |
| AI/runtime | Workflow event enqueued; prompt packet built; AI/model/tool run started/completed/failed; structured output parsed/validated/rejected; policy denial; review packet created; prompt injection flagged; retry/replay/dedupe; dead-letter; model/prompt/schema/policy/tool version change. |
| Human approvals/overrides | Approval requested, approved, rejected, returned for more info, edited, suppressed, overridden, expired, revoked, or applied; final effect references approval id and suggestion/source id. |
| Security/admin/compliance | Role/permission grant/revoke, privileged view, raw evidence access, export/download, redaction override, legal hold set/released, retention policy changed, secret/integration configured/rotated, production tool grant changed, break-glass open/close/review. |
| Retention/deletion | Candidate selected, skipped due to hold, raw object deleted, PII anonymized, tombstone created, embedding/vector row deleted, backup restore tombstone replayed, retention job failure/orphan report. |

### 4.3 Audit visibility

- Customer: only customer-visible own-household history if product policy exposes it; never raw audit, internal notes, provider payloads, AI traces, or other subjects.
- Staff: assigned/task-scoped safe operational summaries and next-action history, not broad raw evidence.
- Lead staff: location operational history needed for routing/triage, still redacted.
- Manager: privileged approval/evidence views for authorized location and duties.
- Owner/admin/compliance/security: governed privileged audit, export, retention, legal hold, and redaction controls under purpose and audit.
- AI worker: minimal source/result/review refs required for dedupe and traceability; no broad privileged audit browsing.

### 4.4 Retention and AI/human approval linkage

Audit events should follow the immutable audit retention tier: proposed minimum seven years, potentially longer for security/admin, incident, legal-hold, and redacted integrity events, subject to human/legal approval.

Every applied high-risk effect must link:

1. Source event/evidence.
2. AI suggestion or deterministic policy result, if any.
3. Validation and permission decision.
4. Human approval id or explicit deterministic policy allowance.
5. Typed command/state transition.
6. Before/after or redacted diff.
7. Result/reconciliation event.

AI may request approval; it may not approve its own suggestion. Human override must append a new event and leave original evidence, AI result, and prior decision intact.

## 5. AI governance rules

Human approval gate: final AI governance policy and workflow-by-workflow tool permission matrix must be approved before production. Until approved, AI is draft/recommend/review-assist by default.

### 5.1 Non-negotiable AI rules

1. No unsupervised high-risk actions. AI must not autonomously execute payment-affecting, safety-affecting, medical/vaccine, incident, eligibility, legal/liability, live booking exception, provider mutation, or sensitive customer-facing operations.
2. Structured outputs are required. Every result must be JSON validated against a shared envelope and workflow-specific schema. Free text is never executable authority.
3. Uncertainty and escalation are explicit. Missing, stale, conflicting, untrusted, low-confidence, or policy-ambiguous inputs must produce `NeedsHumanReview`, `NeedsMoreInformation`, `RejectedByPolicy`, `FailedSafely`, `Suppressed`, or a domain-specific blocked/dead-letter state.
4. Logs are safe. Ordinary logs may include ids, schema/version, validation outcome, safe summary, policy decision, approval gate, and redacted refs only.
5. No hidden state changes. AI suggestions do not mutate business state. All tools and side effects must record actor, reason, approval/policy basis, idempotency, before/after, and audit event.
6. Human override remains available and auditable. Authorized users can approve, reject, edit, suppress, override, or escalate within their role matrix; overrides append events and do not erase originals.
7. AI memory is not operational truth and must not expand access beyond the current prompt packet.

### 5.2 Runtime boundary

Production workflow:

1. Application verifies/adapts source event and persists inbound event where applicable.
2. Application builds least-privilege prompt packet with subject scope, trusted/redacted evidence, policy refs, allowed/forbidden actions, output schema, sensitivity rules, escalation owners, and idempotency key.
3. AI returns schema-bound structured output only.
4. Application parses JSON and validates shared envelope plus workflow schema.
5. Deterministic validators check subject, source refs, freshness, trust, permissions, policy, review gates, idempotency, and allowed actions.
6. Safe results are persisted as workflow/audit records, drafts, recommendations, or review packets.
7. Side effects occur only through approved deterministic tool paths after all required approvals exist.

CLI/manual AI runs may support prototypes or backoffice drafting. They are not the production contract for live customer, payment, safety, booking, or provider effects.

### 5.3 Risk classes and approval matrix

| Class | Examples | AI may do | Effect requires | Audit/log record |
| --- | --- | --- | --- | --- |
| 0 Internal suggestion/draft | Internal summaries, draft notes, checklist suggestions | Summarize, draft, recommend | Staff review before reliance; manager if sensitive/high-risk | AI run, source refs, schema/version, validation, reviewer decision if used |
| 1 Low-risk internal triage | Missing-info tasks, routing, stale record flags | Classify/recommend; create internal tasks only if preapproved | Approved deterministic task policy; staff/lead review for ambiguous cases | Actor, reason, source event, task kind, dedupe key, policy ref |
| 2 Customer-facing draft | Missing-info, care update, payment reminder, incident follow-up | Draft only, identify facts/review gates | Approval before send unless narrow deterministic path approved | Draft/edit/approval/send/suppress chain of custody |
| 3 Document/vaccine extraction | OCR, vaccine candidates, review packet | Extract candidates, confidence, uncertainty | Human/trusted integration verification before compliance/eligibility | Document id, extraction run, source refs, confidence, reviewer decision |
| 4 Incident classification | Severity triage, missing evidence, owner draft | Summarize, risk-flag, draft manager packet | Manager before severity/disposition/owner send/closure; legal/privacy as needed | Incident id, risk flags, source refs, disposition/approval/override |
| 5 Booking/eligibility recommendation | Readiness, waitlist, group-play/capacity suggestions | Recommend status/tasks, draft follow-up | Human approval before confirmation/rejection/exception/customer commitment | Reservation/pet/customer ids, policy refs, before/after diff if applied |
| 6 Payment-affecting operation | Deposits, checkout, refunds, waivers, disputes | Summarize, draft, recommend reconciliation/review | Human approval before provider command, refund, waiver, discount, forfeit, customer commitment | Semantic payment state, approval id, command id, idempotency, provider result |
| 7 Safety-affecting operation | Medication, allergy, behavior, incident, group-play | Summarize evidence, flag risk, draft packet | Staff for approved care execution; manager/medical reviewer for changes/exceptions | Care/safety ids, risk flags, source refs, reviewer decision, redacted diff |
| Provider/system mutation | Reservation/provider/payment/message command | Recommend typed command only where allowed | Authorized approval and approved tool path before execution | Actor, approval, command, idempotency, before/after, provider result |
| Policy/tool/role change | Prompt/tool/role/config request | Flag need only | Owner/admin/security/engineering approval | Security/admin audit event, approver, reason, changed scope |

### 5.4 Required structured result envelope

Every AI workflow result must include:

- `schema_name`, `schema_version`, `workflow_name`, `workflow_version`
- `run_id`, `event_id`, `correlation_id`, `idempotency_key`
- `subject` with type and id
- `status`: `Completed`, `NeedsHumanReview`, `RejectedByPolicy`, `NeedsMoreInformation`, `FailedSafely`, or approved domain blocked/suppressed/dead-letter status
- `summary`: short safe summary
- `structured_output`: workflow-specific typed output
- `recommended_actions`: drafts/recommendations only, each mapped to allowed action vocabulary
- `risk_flags`: safety, medical, incident, payment, privacy, policy, source-trust, prompt-injection, or security concerns
- `verification`: checked sources, trust/freshness/conflict state, policy refs, skipped checks
- `uncertainty`: field-level uncertainty and confidence limits
- `missing_inputs`: facts/sources absent or stale and what they block
- `approval_requirements`: gate, reason, approver role/class, blocked actions
- `human_review_reason` when any gate remains open or status is not clean low-risk internal output
- `source_refs` for every fact used in recommendations, drafts, messages, or task packets
- `safe_log`: safe one-line summary, ids, schema errors, redacted refs only

Validators must reject malformed JSON, wrong subject/event, unknown/forbidden actions, missing source refs, unsafe uncertainty, forbidden side-effect claims, tool results without correlation/audit, prompt-injection compliance, and customer-facing copy that leaks internal, medical, payment, incident, provider, or prompt details.

## 6. Data retention and deletion matrix

Human approval gate: final retention periods, legal-hold process, customer deletion behavior, raw AI/provider/document retention, backup retention, and jurisdiction-specific policy require human/legal/compliance approval. Proposed periods below are planning defaults, not production legal policy.

| Record type | Proposed tier/default | Deletion/anonymization behavior | Holds/exceptions | Implementation notes |
| --- | --- | --- | --- | --- |
| Vaccine documents: raw uploads/images/PDFs/OCR source | Active relationship while current/needed; superseded/rejected raw docs 1-3 years unless tied to Business/Safety tier | Delete raw superseded/rejected files after approved window; keep document kind, hash, reviewer decision, dates, audit refs; anonymize owner PII where allowed | Active reservation, incident, injury, safety review, service denial dispute, legal threat, chargeback, insurance inquiry | Governed private storage, immutable hash, scan/quarantine status, retention class, legal-hold flag; no raw URLs in logs |
| Extracted vaccine facts/policy comparisons | Active pet plus Business tier 3-7 years after related stay/closure if facts supported decisions | Supersede/expire rather than erase decision facts; pseudonymize subject refs when allowed | Vaccine/eligibility dispute, service acceptance/denial dispute, incident/safety, reservation/payment dispute | Store source refs, reviewer refs, policy snapshot, verification state; AI extraction remains candidate until approved |
| Incidents and incident follow-up | Safety/incident/liability tier; proposed 7 years after closure or longer by policy | No hard-delete during open investigation/hold; after expiry anonymize identifiers and keep de-identified aggregate safety data where allowed | Injury, bite/aggression, medication/care error, insurance/legal claim, unresolved complaint, chargeback tied to incident | Raw evidence/media in governed evidence store; separate factual timeline, internal notes, legal material, customer copy |
| Incident follow-up tasks/internal review packets | Same as linked incident; routine non-incident tasks 1-3 years | Delete routine working notes after retention; keep completion status, actor, timestamp, evidence refs, audit | Incident/legal/staff/admin holds | Incident-linked tasks inherit incident retention and access rules |
| Messages: inbound, drafts, approved sends, delivery, replies, suppression | Sent/received material communications Business tier 3-7 years; unapproved drafts 30-90 days after superseded | Approved sends immutable; drafts superseded/rejected and body deleted/redacted quickly; retain provider ids/status and safe summaries | Incident, payment, legal, complaint, cancellation/no-show, consent/opt-out dispute, safety issue | Separate body from metadata for redaction; audit draft/edit/approval/send/suppress chain |
| Payments/payment status references | Business records tier; proposed 7 years after transaction/accounting period or provider/tax rule | Retain semantic amounts/statuses/currency/provider refs/approval refs; delete raw provider payloads after mapping unless approved; anonymize nonessential PII | Chargeback, dispute, fraud, refund/waiver/discount disagreement, provider reconciliation, tax/legal hold | Never store raw card/bank/CVV/tokens; payment archives encrypted/access-controlled |
| Audit logs | Immutable audit tier; proposed minimum 7 years, possibly longer for incident/security/admin/legal | Append-only; corrections/redactions/pseudonymization are later events; no silent row deletion | Legal hold, security incident, admin misuse, payment/incident/provider dispute | Store safe summaries, hashes, field paths, evidence refs; raw values in governed evidence only |
| Deleted customers and pets | Tombstone/immutable audit tier; tombstone indefinitely or audit-retention period | Archive/soft-delete first; disable portal/contact/automation; remove or pseudonymize PII when allowed; keep tombstone id, deletion timestamp, actor, reason, legal-hold state, merge/supersession refs, hashes/audit refs | Active reservation, unpaid balance, open incident, unresolved message/payment dispute, legal hold, required accounting/audit | Backups/restores must reapply tombstones; customer deletion is not immediate cascade deletion of dependent records |
| Prompt packets/runtime context | Ephemeral; raw prompt text disabled by default for sensitive production workflows, or 7-30 days max if approved for debugging | Keep manifest/hash, workflow/version, subject refs, policy refs, redaction profile, model config; delete raw text quickly | Security/model incident, legal request, dispute over AI-assisted decision | Raw prompts may contain PII/medical/payment/incident data; governed evidence access only if retained |
| Structured AI outputs/recommendations/risk flags (AI-runtime artifacts) | Business/audit tier when persisted; malformed/rejected raw output 30-90 days redacted | Persist structured envelope/status/sources/risk flags/validation/human review reason; supersede rather than mutate | Incident/payment/vaccine/customer-message/safety/legal/AI incident | AI output is evidence of process, not source truth; raw completions separate and short-lived if retained |
| Validation errors, policy denials, dead-letter diagnostics | Short operational; proposed 90 days after remediation | Keep safe error class/schema/workflow ids/retry state; delete raw payload/prompt fragments quickly | Security/model/provider/legal dispute | Aggregate counters may remain longer; diagnostics must honor customer deletion/holds |
| AI run metadata | Audit/business; 7 years for workflow decisions, 1-3 years for non-customer operational runs | Keep model/provider/runtime, tool permissions, timestamps, costs, idempotency, source refs; exclude secrets/raw prompts | Legal/security/AI incident holds | Needed for reproducibility/accountability; archive with audit store |
| Drafts: message/task/reservation/payment/vaccine suggestions not approved | Ephemeral/short; 30-90 days after superseded unless approved/acted on | Delete unapproved body; keep safe summary, author/AI refs, review outcome, audit event; approved drafts inherit linked record retention | Dispute, incident/payment/legal complaint, rejected/approved message challenge | Drafts often contain mistakes/unapproved claims; avoid indefinite retention |
| Embeddings/vector indexes/memory | Disabled by default for customer/pet/reservation/medical/vaccine/incident/payment/message/staff data | If approved later, delete embeddings derived from deleted subjects; retain only approved SOP/policy/de-identified memory with deletion path | Legal hold may preserve source but not broad embeddings unless approved | If deletion cannot be guaranteed, do not embed personal/sensitive operational data |
| Tool-call requests/results/provider raw payloads | Raw tool I/O 0-30 days; semantic mapped results follow linked record retention | Store allowed action, tool/version, subject refs, status, redacted summary, policy decision, idempotency; delete raw traces quickly | Security incident, provider/payment/incident/legal hold, AI/tool incident | Never store secrets/tokens/card data/webhook signatures in traces; provider payload retention separately approved |

Retention implementation requirements:

- Every retained record needs `retention_class`, `retention_policy_ref`, `retain_until` or review date, `legal_hold_status`, `deletion_state`, `subject_refs`, `source_refs`, `redaction_profile`, and deletion/anonymization actor/job metadata.
- Retention jobs must check legal holds, active workflows, active reservations, payment/incident/message exceptions, and provider sync state before deletion.
- Deletion/anonymization jobs must delete raw objects, prompt/tool traces, embeddings/vector rows, secondary indexes, and draft bodies where allowed, then emit audit events.
- Backups are not a retention loophole. Restores must reapply deletion requests, tombstones, legal holds, and anonymization before data is exposed.
- Legal/dispute/safety holds must be scoped to specific subjects/evidence, approved by authorized roles, review-dated, and released only through audit.

## 7. Explicit human approval gates

These gates must be resolved and recorded before production enforcement:

1. Role matrix gate
   - Exact production roles, optional specialist roles, delegated capabilities, approval thresholds, location/tenant boundaries, break-glass rules, export authority, and separation-of-duties rules.
   - Staff vs lead staff vs manager authority for document review, incident disposition, customer messages, booking exceptions, payment exceptions, and task automation.
   - Customer-visible history/audit scope and privileged raw evidence access.

2. AI governance gate
   - Final AI governance policy and workflow-by-workflow safe-to-automate classes, if any.
   - Tool permission matrix by agent type/workflow: ReadOnly, DraftOnly, InternalTaskOnly, ApprovedCustomerSend, ApprovedProviderMutation, Forbidden/NeverAutomate.
   - Customer-message automation paths, templates/copy, suppression rules, consent/preference model, final send authority, and tests.
   - Provider/system mutations allowed in v1, deterministic preconditions, approval refs, idempotency/reconciliation, rollback, and audit.
   - Model/provider/prompt/version provenance retention and who can inspect prompts/outputs/tool traces.

3. Retention policy gate
   - Approved jurisdiction/location/provider-specific retention durations for documents/OCR, vaccine facts, incidents, messages, payments, audit logs, deleted customer/pet tombstones, raw AI artifacts, provider payloads, media, logs, backups, and archives.
   - Legal-hold roles/reasons/workflow/release process.
   - Customer/pet deletion behavior, exemptions, tombstone contents, backup restore rules, vector/embedding deletion guarantees, and customer-visible disclosures.
   - Whether raw prompts, raw completions, raw provider payloads, raw OCR, and raw tool traces may be retained at all in production.

Until these gates are approved, implementation must use conservative defaults: least privilege, draft/review for AI, no unsupervised high-risk actions, redacted projections, short raw-runtime retention, and deny/review/suppress when uncertain.

## 8. Open questions

1. Which exact production roles exist beyond the seven canonical roles: payment reconciliation, compliance/privacy/legal, engineering/integration owner, medical/vaccine reviewer, or location admin?
2. What location/tenant model is required for MVP, and can managers/admins view cross-location subjects by default?
3. Which staff/lead/manager actions have thresholds by amount, incident severity, service line, shift, location, or policy version?
4. Which customer-facing message templates may be deterministic auto-send, and which categories are always manager/legal reviewed?
5. Which workflows, if any, are safe for `InternalTaskOnly` auto-creation, and what dedupe/rate-limit/assignee policy prevents queue floods?
6. Which provider/PMS/payment systems are approved sources of truth, and what live mutations are allowed in v1?
7. Can trusted veterinary/EHR integrations ever verify vaccine facts without manual review, or is all v1 verification human-reviewed?
8. What are approved retention periods and deletion rights by jurisdiction, insurance carrier, payment provider, accounting/tax policy, and customer contract?
9. Are raw AI prompts/completions/tool traces retained in production, or are manifests/hashes/source refs the only retained runtime audit evidence?
10. Are embeddings/vector indexes allowed for any personal operational data, and can deletion be proven across vectors, caches, search indexes, backups, and restored snapshots?
11. Do audit events require per-subject sequence numbers, WORM/tamper-evident storage, or cryptographic hash chains for MVP?
12. What security incident response workflow covers sensitive-data exposure, unauthorized export, prompt leakage, webhook-secret compromise, role misconfiguration, and model/tool misbehavior?
13. What customer-visible audit/history/export will the portal provide, and how will internal notes/AI traces/provider payloads be excluded?
14. What release process approves production prompt/tool/model/policy changes, and who reviews post-deploy drift or failures?

## 9. Implementation notes for downstream MVP planning

1. Model permissions as named capabilities and scopes, not hard-coded job titles. Store role grants, subject scopes, location/tenant scopes, policy refs, and approval requirements separately.
2. Put the deterministic policy evaluator between UI/API/AI/provider input and state mutation. No workflow should mutate state directly from free text or AI output.
3. Use append-only domain events/audit events for lifecycle transitions. Represent cancellation, rejection, no-show, void, correction, supersession, deletion, anonymization, and suppression as states/events, not silent deletes.
4. Add a `DataClassification` or equivalent metadata layer to records/evidence/prompt packets with sensitivity, retention class, redaction profile, and allowed audience.
5. Build role-scoped projections early: customer-safe, staff operational, manager review, owner/admin/compliance, AI prompt packet, and audit export are separate views.
6. Store raw documents, incident media, provider payloads, prompt/tool traces, and message bodies as governed evidence refs. Keep ordinary audit rows and logs redacted.
7. Implement AI result validation as a first-class adapter: shared envelope, workflow schema, allowed action mapping, source/ref validation, policy validation, permission validation, idempotency validation, and safe failure states.
8. Treat customer-message chain of custody as a workflow: draft, edit, approval request, approve/reject/return, queue, send attempt, delivery failure, suppression, unsubscribe, reply.
9. Implement approval records as durable subjects with approver role, scope, reason, policy version, expiration/revocation, and target action. Effects reference approval ids.
10. Implement payment flows with semantic payment domain records, verified provider events, idempotency keys, reconciliation state, and explicit manager/payment approval for exceptions. Never store raw card data.
11. Implement document/vaccine flows with quarantine, immutable file hash, extraction runs, candidate facts, human/trusted-integration verification, supersession, and eligibility recompute only after approval.
12. Implement incident flows as case management with evidence refs, manager review, legal/privacy escalation, customer-message approval, closure/reopen events, and retention/legal-hold hooks.
13. Add retention metadata and deletion/anonymization jobs from the start; retrofitting retention after raw prompts/files/provider payloads accumulate will be risky.
14. Tests should cover forbidden role actions, cross-customer access, prompt injection in OCR/messages/provider data, malformed AI output, forbidden free-text side-effect claims, missing approvals, low-confidence extraction, customer-message suppression, payment/provider idempotency, legal holds, deletion tombstone restore, raw-log redaction, and human override audit.
15. Release readiness should distinguish CI/implementation readiness from publish blockers: role matrix approval, AI governance approval, retention/legal approval, customer-message policy, provider/payment policy, security incident response, and audit/export policy are publish blockers.

## 10. Conservative rule

When source facts, role authority, approval status, policy version, retention class, redaction state, AI confidence, schema validity, source trust, provider verification, or tool permission are missing, stale, contradictory, sensitive, unverified, or outside scope, the system must deny, redact, suppress, route to human review, request more information, or fail safely with audit evidence. It must not silently invent policy, broaden roles, expose raw sensitive data, treat AI output as authority, send customer messages, move money, approve medical/vaccine/safety/incident outcomes, or execute provider/system mutations.
