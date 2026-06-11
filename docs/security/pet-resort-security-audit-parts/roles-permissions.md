# Roles and permissions matrix

Purpose: define the canonical draft role and permission model for the pet-resort system security/audit design.

Status: draft security input. This document is not final production approval. The final role matrix, AI tool grants, approval delegation, export authority, payment authority, and retention/legal-hold authority require explicit human approval before production use.

Source anchors:

- `docs/security/pet-resort-security-audit-parts/inputs.md`
- `docs/architecture/agent-permissions-by-workflow.md`
- `docs/workflows/payments-pricing.md`
- `docs/workflows/payments-pricing-parts/ai-boundaries.md`
- `docs/workflows/staff-operations-parts/inputs.md`
- `docs/domain/petsuites/implementation-review.md`
- Current Rust policy/runtime vocabulary: `AutomationLevel`, `ReviewGate`, `AllowedAction`, `WorkflowResult`, and app-owned tool/runtime boundaries.

## Permission vocabulary

Use named capabilities rather than broad role labels.

| Capability | Meaning |
| --- | --- |
| `R` | Read a role-scoped projection. Sensitive fields may be redacted. |
| `C` | Create a new draft, request, upload, internal note, task, or subject record. |
| `U` | Update an existing record within allowed state/field boundaries. |
| `D` | Delete, archive, void, retire, supersede, or suppress. Destructive actions should normally be soft-delete/supersede with audit. |
| `A` | Approve, verify, resolve a gate, or authorize an effect. |
| `S` | Send or queue an outbound customer/provider/system action after required policy checks. |
| `X` | Export, download, bulk view, privileged inspect, or access governed raw evidence. |
| `M` | Administer policy, roles, integrations, tool grants, retention, or security configuration. |
| `Draft` | May draft/recommend only. Requires validation and human/system approval before effect. |
| `Own` | Limited to the customer's own household, pets, reservations, documents, messages, and customer-visible status. |
| `Assigned` | Limited to assigned location/shift/task/reservation/pet or operational queue. |
| `Location` | Limited to the user's authorized location/business unit. |
| `Privileged` | Requires owner/admin, compliance, manager, or approved break-glass purpose. |

Default deny: if a permission is not explicitly granted by role, subject scope, workflow purpose, and current state, the system should return denied/review/blocked/suppressed with audit evidence.

## Canonical roles

| Role | Primary purpose | Default scope |
| --- | --- | --- |
| Customer | Pet parent/customer portal actor. | `Own` household, pets, reservations, uploads, approved messages, and customer-visible statuses. |
| Staff | Front desk, caregiver, or routine operator. | `Assigned` tasks/reservations/pets plus minimal `Location` operational queues. |
| Lead staff | Shift lead / senior operator for routine escalation and routing. | `Location` operational queues, lead-level task routing, ordinary escalation triage. |
| Manager | Business/location manager with approval authority for operational exceptions. | `Location` privileged operational, incident, payment-exception, and customer-message gates. |
| Owner/admin | Business owner or security/admin operator. | Cross-location or tenant-level privileged administration, exports, role/config changes, retention/legal holds. |
| System | Deterministic application actor. | Policy-enforced state transitions, validations, idempotency, queues, audit writes, and approved deterministic sends/mutations. |
| AI workflow worker | Hermes/LLM/agent runtime actor. | Least-privilege typed prompt context for one workflow event; draft/recommend/classify/summarize/extract only unless a later approved tool grant narrows an effect. |

Optional specialist responsibilities, if split later, should be modeled as capabilities or subroles rather than silently broadening the seven canonical roles: payment reconciliation, compliance/privacy/legal, engineering/integration owner, and veterinarian/medical reviewer.

## High-level role matrix

| Domain | Customer | Staff | Lead staff | Manager | Owner/admin | System | AI workflow worker |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Customers/accounts | `R/C/U Own` for profile/contact/preferences through approved portal fields. No merge/export/admin. | `R Assigned/Location`; `C/U` routine intake/contact corrections with audit; no merge/delete/export. | Staff rights plus duplicate/intake routing and ordinary correction review; no privileged export. | `R/C/U/A Location`; approve merges, contact exceptions, do-not-contact conflicts, sensitive corrections. | `R/C/U/D/X/M Privileged`; role/account lifecycle, cross-location export, retention/legal hold. | Validate identity links, enforce contact preferences, dedupe, audit. | Read minimized customer facts for the workflow; draft missing-info/merge-review suggestions only. |
| Pets | `R/C/U Own` pet profile and approved care fields; cannot change staff-only risk/status decisions. | `R Assigned`; `C/U` routine observations/care-profile updates within task; no final medical/behavior eligibility. | Staff rights plus routing/lead review for routine care or behavior flags. | Approve medical/behavior/group-play exceptions and high-risk pet status changes. | Privileged pet record administration, export, retention/legal hold. | Enforce required fields/state transitions, source provenance, audit. | Read scoped pet facts; extract/summarize/recommend; no final eligibility or care-truth mutation. |
| Reservations | `R Own` status; `C` request/inquiry; `U Own` limited request details before locked states; no confirm/cancel exception/override. | `R Assigned/Location`; `C/U Draft` intake, check-in/out prep, factual status suggestions; no booking exception approval. | Staff rights plus queue routing, waitlist/blocked task triage, ordinary operational reassignment where policy permits. | Approve accept/reject/cancel/no-show/overbooking/waitlist release/capacity/holiday/ratio exceptions. | Configure reservation policy, cross-location overrides, privileged exports/admin. | Apply deterministic state-machine checks, holds/releases only under approved policy/approval refs. | Booking triage recommendations, gap flags, draft customer explanations; no live status/provider mutation. |
| Vaccine/document records | `R Own` customer-safe status; `C` upload documents; `U` replace/upload clearer documents; no verification/delete. | `R Assigned` document status and collection needs; `C` upload/import; `U Draft` classification metadata; no final verify/waive. | Staff rights plus review-queue routing and ordinary collection escalation. | `A` verify/reject/exception where policy permits; approve eligibility effect and sensitive customer wording. | Privileged document administration, governed raw evidence export, retention/legal hold, policy config. | Quarantine, hash, OCR pipeline, supersession, eligibility recompute after approved verification. | OCR/extraction candidates, confidence/source refs, review packet drafts only; no final valid/expired/waived decision. |
| Tasks | `R Own` customer-visible task/status only if exposed; may create requests/messages, not staff tasks. | `R/C/U Assigned` operational tasks; complete routine tasks with evidence; escalate blocked/high-risk. | `R/C/U/A Location` lead-level task routing, reassignment, priority triage, ordinary blocked resolution. | Approve manager-review tasks, exceptions, incident/payment/message gates; close escalations. | Administer task templates, automation triggers, global queues, exports. | Create deterministic tasks from approved triggers; enforce dedupe/idempotency/SLA states. | Draft/recommend internal tasks where policy permits; no bulk auto-generation without approval. |
| Notes/care notes/internal notes | `R Own` only customer-approved summaries; `C` customer messages/requests. | `R/C/U Assigned` care notes and internal observations; cannot publish customer summaries or view privileged-only notes unless needed. | Staff rights plus lead handoff notes and ordinary review. | Approve customer-visible summaries, sensitive corrections/voids, and incident-linked notes. | Privileged note export, retention/legal hold, redaction override. | Enforce note state transitions, append corrections/voids, audit redactions. | Summarize/draft notes/customer-safe wording from scoped evidence; no hidden side effects or publication. |
| Incidents | `R Own` only approved customer-visible incident communications/status; `C` customer report/message. | `R/C Assigned` incident observations/evidence; no close/downgrade/customer notification authority. | Staff rights plus initial severity routing and escalation packet; no manager-only closure. | `R/C/U/A/S Location` incident investigation, severity approval, owner notification, resolution/closure/reopen, customer communications. | Privileged cross-location incident export, legal/compliance escalation, retention/legal hold. | Enforce incident workflow states, evidence immutability, audit chain of custody. | Summarize, classify risk, draft manager/customer packets; no severity finalization, blame, diagnosis, closure, or send. |
| Messages | `R Own` messages; `C` inbound/customer requests; `U` customer preferences within policy. | `R Assigned`; `C/U Draft` replies; may send routine approved/staff-authorized messages only if deterministic policy/human approval covers the category. | Staff rights plus queue routing and ordinary template-reviewed sends if approved. | Approve/send sensitive messages: medical, behavior, incident, payment, legal, eligibility/refusal, complaint, exception. | Configure templates, channels, automation, export, suppression policy. | Enforce preferences/opt-outs/suppression, deterministic send paths, provider status reconciliation. | Draft, summarize, risk-flag, and suppression-recommend; no direct send except future approved deterministic path with fixed facts/template/recipient. |
| Payment status | `R Own` customer-safe balance/deposit/status; `C` payment initiation through approved provider flow; no manual status/refund/waiver. | `R Assigned` semantic status only; create failed-payment/collection tasks; no provider command or manual financial status change. | Staff rights plus routing/reconciliation task triage; no refund/waiver/forfeit approval. | Approve payment exceptions: deposit waiver, discount, refund, forfeit, write-off, manual status correction, customer-facing payment language. | Configure provider/policies, privileged payment exports, reconciliation administration, fee/refund authority as approved. | Verify webhooks, dedupe, reconcile provider states, apply approved commands with idempotency/audit. | Read minimized semantic payment status; draft reminders/escalations/reconciliation summaries; no money movement or financial truth mutation. |
| Audit events | `R Own` only customer-visible history if product exposes it. | `R Assigned` safe operational audit summaries needed for task context. | Staff rights plus location operational history for triage. | `R/X Location` privileged approval/evidence views for manager duties. | `R/X/M Privileged` audit policy, exports, legal hold, retention, redaction override; cannot mutate append-only history. | Append-only audit writer, integrity checks, redacted projections, policy decisions. | Read minimal source/result/review refs for dedupe/traceability; write result metadata through app runtime only; no raw privileged audit browsing. |
| AI outputs/configuration | May see only approved customer-facing AI-assisted messages/status where exposed. | May review assigned drafts/recommendations; cannot change model/tool grants. | Staff rights plus route AI review queues and return drafts for revision. | Approve workflow-specific AI drafts/outcomes and safe operational automation candidates where policy permits. | `M` model/provider/prompt/tool grants, automation classes, retention/logging config, break-glass access. | Validate structured output, enforce allowed actions, create review packets, record prompt/output/tool audit. | Produce outputs within schema; request review; cannot approve itself, expand context, change tools, or alter policy/config. |

## Action rights by domain

### Customers/accounts

- Read: customer reads own customer-safe profile/history; staff/lead read assigned or location operational projections; managers/admins can inspect broader records for authorized purpose.
- Create/update: customer can create/update own portal fields; staff can create/update routine intake/contact corrections; manager approves merges, sensitive contact preference conflicts, account closures, or do-not-contact exceptions.
- Delete/export/admin: delete should mean archive/retire with audit. Exports and role/account administration are owner/admin or delegated compliance-only.
- Separation of duties: the actor requesting a duplicate merge or privileged contact override should not be the sole approver when the action affects identity, contactability, or cross-household data.

### Pets

- Read: customers see own pet profile and approved care/status; staff see assigned pets and task-relevant care details; AI sees minimized subject snapshots only.
- Create/update: customer/staff can propose or update factual profile/care details; medical, behavior, group-play, medication/allergy, or eligibility-affecting changes require manager/authorized reviewer approval.
- Delete/export/admin: archiving, merge, governed evidence export, or retention actions are privileged.
- Separation of duties: AI or routine staff may flag behavior/medical risk but cannot final-approve eligibility, group-play reinstatement, or medical ambiguity.

### Reservations

- Read/create/update: customers create requests and see own customer-safe status; staff work assigned reservations and prepare intake/check-in/out evidence; leads route operational queues; managers approve exception-bearing transitions.
- Approve/send: booking acceptance/rejection, cancellation/no-show consequences, waitlist release, overbooking, capacity holds/releases, and customer commitments require approved manager/system authority by policy.
- Delete: cancellations and rejections are lifecycle states, not hard deletion.
- Separation of duties: payment provider success alone must not confirm a reservation unless deterministic policy and reservation authority agree. Staff may collect facts; managers approve exceptions.

### Vaccine/document records

- Read/create/update: customers upload and see customer-safe document/vaccine status; staff collect/import; system quarantines/OCRs; AI suggests extractions; manager/authorized reviewer verifies/rejects/exceptions.
- Delete/export: raw documents, OCR, vet proof, images, signatures, and governed evidence are privileged and retention-controlled; supersede rather than delete unless retention policy permits deletion.
- Separation of duties: the actor/AI extracting facts cannot be the sole verifier. OCR confidence is evidence, not approval. Verification must cite source document/trusted integration and reviewer/system policy.

### Tasks

- Read/create/update: staff work assigned tasks; leads route and reprioritize; managers approve manager-review tasks; system can create tasks from approved deterministic triggers.
- Delete/close: cancel/supersede with reason and audit; bulk task automation requires approval by trigger, kind, priority, assignee, rate limit, and dedupe key.
- Separation of duties: AI may draft/recommend tasks but cannot approve its own task effects or flood queues without approved trigger/rate policy.

### Notes and internal-only notes

- Customer-visible notes are a separate projection from internal notes.
- Staff may write factual operational/care notes within assignment. Managers approve customer-visible summaries, corrections/voids, and sensitive incident-linked notes.
- Internal-only notes are visible to staff/lead/manager only when assigned/operationally necessary. Privileged HR/security/legal notes should require owner/admin/compliance purpose and should not enter ordinary prompt packets.
- Separation of duties: customer-visible publication or void/correction of sensitive notes requires reviewer different from AI drafter and, for high-risk subjects, manager approval.

### Incidents

- Staff may report and attach evidence; lead staff may route/escalate; managers own severity, investigation state, customer communication, resolution, closure, and reopen decisions.
- Owner/admin/compliance owns legal/privacy/security escalation, privileged exports, legal hold, and cross-location review.
- AI may summarize, classify risk, identify missing facts, and draft review packets. It must not assign blame, diagnose, downgrade severity, close incidents, or notify owners/public channels.
- Separation of duties: incident author/drafter should not be sole approver for closure or customer/legal communication when safety, liability, refund/credit, staff conduct, or disputed facts are involved.

### Messages

- Customers can create inbound messages and view their own messages.
- Staff can draft and, if policy explicitly allows, send routine low-risk messages using approved templates and verified facts. Sensitive or ambiguous messages require manager approval.
- Manager approval is required for medical, vaccine eligibility, medication/allergy, behavior, incident, safety, legal/liability, payment/refund/waiver/discount/forfeit, complaint, refusal/denial, booking exception, cancellation/no-show, or policy-exception language.
- System can send only through deterministic preapproved paths that fix recipient, consent/preference, template, facts, timing, suppression rules, and audit evidence.
- AI can draft/recommend/suppress/risk-flag only. AI-authored free text is not a deterministic approved template.

### Payment status

- Customers can initiate payment via approved provider flows and read customer-safe status.
- Staff/lead see semantic payment/deposit status needed for operations and create collection/reconciliation tasks. They do not change financial truth.
- Managers approve exceptions: waivers, discounts, refunds, forfeitures, write-offs, manual status corrections, disputed/failed/duplicate reconciliation outcomes, and payment-sensitive customer messages.
- Owner/admin configures providers/policies and privileged exports; system verifies webhooks and applies approved idempotent commands.
- AI reads minimized semantic status and drafts/recommends. It never captures, refunds, waives, discounts, changes status, stores card data, or treats unverified raw payload/customer claims as truth.

### Audit events

- Audit is append-only. Nobody, including owner/admin, can rewrite historical audit events; corrections are later events.
- Customers may see only a customer-visible projection if product policy exposes it.
- Staff/lead read safe operational audit summaries for assigned work; managers read approval/evidence audit for authorized location; owner/admin/compliance can export/inspect governed audit under purpose/retention controls.
- AI can receive minimal refs needed for dedupe/source traceability, not privileged raw audit browsing.

### AI outputs and configuration

- Staff/lead/manager can review assigned AI outputs according to workflow role and gate.
- AI outputs must be schema-valid, policy-valid, source-linked, confidence/uncertainty-labeled, and audited before becoming drafts/tasks/review packets.
- Owner/admin or delegated security/release authority controls model/provider/prompt/tool grants, automation categories, retention/logging, and rollback.
- AI worker cannot approve itself, change its tools, expand its own context, change policy, bypass validators, or execute high-risk effects.

## Separation-of-duties rules

1. Document/vaccine verification: AI/OCR may extract; staff may collect; manager or explicitly authorized document reviewer verifies/rejects/waives. Extraction confidence cannot self-verify.
2. Incident approval: staff may report; lead may route; manager approves severity-affecting changes, customer communication, resolution/closure/reopen, refunds/credits tied to incident, and legal/privacy escalation. Reporter/drafter should not be sole closer for high-risk incidents.
3. AI draft approval: AI cannot approve or send its own draft. A human or deterministic preapproved system path must approve the exact effect, recipient, facts, and template/category.
4. Payment status changes: provider webhooks are verified/reconciled by system; humans approve exceptions; AI/customer/staff free text cannot change financial truth. Refunds/waivers/discounts/forfeitures/write-offs require manager/owner-approved authority.
5. Workflow override: staff may request/flag; lead may route ordinary blocks; manager approves operational exceptions; owner/admin approves policy/config/security overrides. Every override cites reason, approval id, policy version, before/after, and actor.
6. Internal-only notes: staff/lead/manager see internal notes only for assigned/operational purpose. Privileged HR/security/legal/compliance notes are owner/admin/compliance-scoped and excluded from ordinary AI prompts.
7. Exports and raw evidence: broad exports, raw documents/OCR, raw provider payloads, privileged audit, AI prompt/output/tool traces, and governed media require owner/admin/compliance purpose and audit. Routine staff views use redacted projections.
8. Role/config/tool changes: the actor requesting a privileged role, permission, prompt, tool, integration, webhook, retention, or automation change should not be the only approver/releaser for production.
9. Break-glass: privileged emergency access must be time-limited, purpose-bound, logged, and reviewed after use.
10. Customer communication: final send authority must be distinct from AI drafting for sensitive messages. Suppression/denial is an auditable outcome.

## Row and entity scoping rules

### Customer scope

Customers can access only their own household boundary:

- their customer/account profile and contact preferences;
- their pets;
- their reservations/requests/statuses;
- their uploaded documents and customer-safe vaccine/document status;
- their messages and approved outbound communications;
- payment links and customer-safe payment/deposit status;
- optional customer-visible history if product policy later approves it.

Customers cannot access other households, internal-only notes, staff identities beyond approved communication context, raw audit, raw provider payloads, raw AI traces, manager review queues, or privileged policy/admin settings.

### Staff scope

Staff access is purpose-limited to assigned and operational data:

- assigned shift/location/task/reservation/pet queues;
- data needed to perform intake, care, check-in/out prep, document collection, routine communication, and evidence capture;
- redacted operational summaries rather than broad raw PII/medical/payment/audit/provider/AI traces;
- no manager-only exception approval, privileged export, role/config/admin, or cross-location browsing unless separately granted.

### Lead staff scope

Lead staff inherit staff scope and add location-level operational coordination:

- queue triage, assignment/reassignment, blocked task routing, shift handoff, ordinary escalation packet review;
- no assumption of manager-only authority for payment exceptions, legal/compliance matters, incident closure, high-risk customer sends, or policy/config changes.

### Manager scope

Managers operate within authorized location/business scope:

- exception approval, incident resolution, sensitive messages, payment exception approval, capacity/booking overrides, staff workflow oversight;
- privileged evidence access only when needed for approval/review;
- no unrestricted cross-tenant/security admin unless also owner/admin.

### Owner/admin scope

Owner/admin covers tenant/cross-location privileged functions:

- role and permission administration;
- policy, pricing, payment, integration, prompt/tool, automation, retention, legal-hold, export, security configuration;
- break-glass and privileged audit/evidence views;
- still subject to append-only audit, purpose limits, and separation from routine business approvals where possible.

### System scope

The deterministic application may act only through approved policies:

- validate permissions, schemas, state machines, idempotency, signatures, consent/preferences, suppression rules, and review gates;
- write audit and workflow events;
- apply approved state transitions or external commands only when policy and approval references are present;
- never invent business policy or bypass missing approvals.

### AI workflow worker scope

AI worker grants are per event, workflow, subject, location, tenant, and purpose:

- prompt packets include only required snapshots, redacted evidence, source refs, policy refs, and allowed action vocabulary;
- tool grants should start as `ReadOnly`, `DraftOnly`, and narrowly approved `InternalTaskOnly` where appropriate;
- customer-send, provider-mutation, payment, policy, role/config, raw evidence, broad audit, and cross-customer tools are denied unless a later human-approved production model defines deterministic constraints;
- AI memory is not operational truth and must not expand access beyond the current packet.

## Approval gates preserved

The following are not approved by this draft and must remain explicit gates:

1. Final production role matrix, including whether optional specialist roles are separate roles or delegated capabilities.
2. Final staff vs lead staff vs manager authority for every approval gate and whether approvals can be delegated by location, shift, amount, incident severity, or service line.
3. AI tool permissions by workflow, including any `SafeToAutomate`, `InternalTaskOnly`, `ApprovedCustomerSend`, or `ApprovedProviderMutation` paths.
4. Customer-message automation categories, templates, suppression rules, consent/preference model, and final send authority.
5. Payment provider, reconciliation authority, refund/waiver/discount/forfeit thresholds, and payment-status correction process.
6. Document/vaccine reviewer authority, trusted-integration bypass rules, waiver/exception policy, and raw evidence access.
7. Audit retention, prompt/output/tool trace retention, raw provider payload retention, document/OCR retention, media retention, legal holds, deletion, and export policy.
8. Multi-location/tenant boundaries, cross-location manager/admin access, break-glass policy, and post-access review.

## Rationale

The model separates data access, workflow authorship, approval authority, external effects, exports, and administration because the pet-resort domain mixes ordinary operations with high-risk subjects: medical/vaccine evidence, behavior/safety, incidents, customer-facing commitments, payments, legal/liability language, provider writes, and AI-generated content.

Customers receive a narrow own-household projection. Staff receive assigned/operational data needed to do work. Lead staff coordinate routine operations without inheriting manager-only approval. Managers approve location-level business exceptions and sensitive communications. Owner/admin handles privileged configuration, exports, retention, and security. The system enforces deterministic policy and audit. AI workers remain least-privilege assistants whose outputs are evidence-linked drafts/recommendations, not authority.

This preserves the conservative downstream rule: missing, stale, contradictory, sensitive, unverified, or out-of-scope facts produce denied/review/blocked/suppressed outcomes with audit evidence rather than broadened access, hidden mutations, customer sends, provider actions, or AI-authorized approvals.
