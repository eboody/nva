# Security and audit canonical inputs

Purpose: collect the canonical upstream inputs needed to synthesize the pet-resort security, permissions, auditability, data-safety, and AI-governance model.

Status: draft input packet. This document does not approve production access control, production AI permissions, customer-message automation, retention periods, payment-provider configuration, live reservation writes, or live external-system mutations. Treat unresolved items as approval gates, not implementation permission.

## Source index

Primary repo sources checked:

- `docs/workflows/staff-operations-parts/inputs.md` — staff/persona assumptions, operating-day and task inputs, review gates, staff task model, conservative downstream rule.
- `docs/workflows/payments-pricing.md` — payment/pricing source-of-truth records, payment state machine, payment AI boundaries, escalation routing, provider/webhook/idempotency constraints, open policy/provider questions.
- `docs/workflows/payments-pricing-parts/ai-boundaries.md` — explicit AI allowed/forbidden roles around deposits, payments, refunds, waivers, reminders, and sensitive billing data.
- `docs/domain/petsuites/implementation-review.md` — production-readiness boundaries: durable approvals/audit events, source provenance, live execution boundary, customer-message draft/review posture.
- `domain/src/policy.rs` — current `ReviewGate` and `AutomationLevel` vocabulary plus conservative group-play policy anchors.
- `domain/src/workflow.rs` — current workflow event/result/action envelope anchors.
- `domain/src/entities.rs`, `domain/src/operations.rs`, `domain/src/care.rs`, `domain/src/payment/mod.rs`, `domain/src/reservation/mod.rs`, `domain/src/tools.rs`, `domain/src/agent.rs` — current Rust domain-contract anchors cited by peer handoffs and current repo inspection.

Canonical kanban handoffs used where artifacts were generated in scratch workspaces or not yet copied to this repo:

- `t_d13548b9` (`Define core entities and relationships`) — required entities, lifecycle state inventory, relationship summary, AI-write policy, entity audit events, open human gates.
- `t_9be768e3` (`Define reservation schema and lifecycle`) — reservation required relationships, state machine, capacity assumptions, payment/special-care gates, automation boundaries, integration hooks.
- `t_a9088781` (`Define document and vaccine model`) — document/vaccine model, OCR/AI suggestion policy, verification states, confidence/review policy, eligibility gates, audit evidence.
- `t_0a8f9c6e` (`Define audit model and required audit policy`) — immutable audit schema, actor model, approval policy, event taxonomy, redaction policy, outbound message chain of custody.
- `t_cf266086` (`Collect canonical input constraints for AI/Hermes runtime`) — app-owned queue/runtime boundary, prompt-packet/output-validation posture, security categories, missing approval gates.
- `t_41e8b605` (`Collect canonical staff-operations inputs`) — staff roles, operational process states, task model, source/audit refs, and gates for staff workflows.
- Parent/input handoffs referenced by peers: `t_32dc302c` canonical data-model constraints, `t_81dc80d3` workflow-event inputs, `t_03f4d420` operational task model.

Missing or caveated sources:

- `docs/product/pet-resort-product-map.md` is still missing; use product-map board handoffs and repo README/context until restored.
- Final `docs/architecture/pet-resort-data-model.md` and `docs/architecture/pet-resort-ai-runtime.md` are not yet approved in this repo; use peer handoff metadata and existing Rust contracts as provisional inputs.
- The scratch artifacts named by peer tasks (`docs/data-model/core-entities.md`, `docs/data-model/reservation-schema-lifecycle.md`, `docs/data-model/documents-vaccines.md`, `audit-model-policy.md`) were created in kanban scratch workspaces rather than the current repo path at collection time. Their completion metadata is treated as canonical handoff input for this packet.
- Location-specific live policy is unresolved: prices, deposits, cancellation/no-show windows, refundability, holiday/peak rules, staff ratios, role authority, message templates, retention durations, and provider choices remain approval gates.

## Scope baseline

The current product shape is an internal operations/workflow layer for a pet resort or small resort group. The Rust application owns policy, state, writes, queueing, validation, and audit. Hermes/LLM/AI workers are bounded assistant actors that consume typed, least-privilege prompt packets and return structured drafts, recommendations, summaries, internal-task suggestions, and review packets.

MVP/security assumptions:

- Internal staff/manager workflows are the first surface; customer-facing automation is draft/review by default.
- Source-of-truth state lives in typed application/provider records, not in model memory, free text, screenshots, raw webhooks, or AI summaries.
- AI outputs default to suggestions/drafts unless deterministic app policy explicitly marks a narrow workflow safe to automate.
- Medical, safety, behavior, vaccine, legal/liability, payment, booking-exception, capacity-exception, service-denial/acceptance, and customer-facing sensitive-message outcomes require human approval.
- Every AI suggestion, policy decision, approval, outbound message draft/send, before/after state change, workflow event, tool call, and external-system action must be traceable to actor, subject, source event/evidence, policy snapshot, review gate, and idempotency/audit key.

## Data model constraints for security and audit

### Core subjects and ownership

Canonical entities to protect and audit:

- Customer: account/contact root. Owns contact PII, portal account refs, preferences, message history, payments, reservations, pets, and audit actor references.
- Pet: animal root under customer. Owns species/age/sex/spay-neuter, care profile, medication/allergy/medical data, temperament/group-play evidence, vaccine facts, care notes, and incidents.
- Reservation: service-delivery root. Links customer, pet(s), service, location, room/suite/capacity holds, tasks, care notes, documents, messages, payment/deposit records, incidents, workflow/audit events, and external provider refs.
- Service: offering semantics for boarding, day play/daycare, day boarding/individual play, grooming, DaySpa/bathing, training, retail/package/add-ons.
- Room/Suite or accommodation inventory: location-owned capacity resource with held/reserved/occupied/cleaning/maintenance/out-of-service states.
- Staff: human operator root and audit actor for assignments, approvals, notes, messages, and task evidence.
- Task: workflow/review root for operational work, escalation, and manager gates; always linked to concrete subject(s), source event, assignment, status, priority, evidence, and audit trail.
- Document: evidence/provenance root for uploads/imports, immutable file refs, OCR/redacted artifacts, extraction runs, review decisions, retention/legal holds, and supersession.
- VaccineRecord: pet-owned policy fact supported by document/source evidence and human/approved trusted-integration verification.
- CareNote: pet/reservation-scoped observation and care history, with draft/internal/review/customer-summary states.
- Incident: high-risk case-management subject spanning pets, customers, reservations, staff, documents, messages, tasks, and approvals.
- Message: communication lifecycle and provider-status record linked to customer plus optional reservation/pet/incident/task context.
- Payment/Deposit: financial lifecycle linked to customer/reservation and provider/PMS references, with exception gates.
- AuditEvent: append-only cross-cutting trace of actor/subject/action/evidence/before-after/approval/output/tool state.

Security implication: permissions should be subject-scoped and purpose-scoped. A role that can view operational task titles should not automatically view raw medical documents, raw payment payloads, customer PII, staff/security logs, or model/tool traces.

### Sensitive data categories

Minimum categories for access control, prompt minimization, logging, retention, and redaction:

- Public/business-safe: service names, approved public policy copy, general availability categories without customer/pet identifiers.
- Internal operations: capacity snapshots, staff tasks, occupancy, labor/staffing signals, room/suite assignments, operational risk summaries.
- Customer PII/contact: names, phone/email/address, portal account refs, emergency/vet contacts, customer messages, preferences, account status.
- Pet care/medical/safety: medication, allergies, feeding/care instructions, medical conditions, behavior/temperament, group-play observations, incidents, vaccine documents/OCR/extracted facts.
- Payment/legal/liability: deposits, balances, provider refs, refunds, disputes, waivers, discounts, forfeitures, cancellation/no-show consequences, legal/liability language.
- Provider/integration raw data: raw PMS/Gingr payloads, webhook events, signatures, external ids, provider-specific errors, screenshots/imports/exports.
- AI/tool execution: prompt packets, model responses, structured outputs, validation failures, tool-call requests/results, risk flags, policy-denial reasons.
- Secrets/internal security: API keys, tokens, webhook signing secrets, raw card data, auth material, credential metadata, privileged admin/security logs.

Never store or expose raw secrets, full payment instruments, auth tokens, webhook signatures, credential material, CVV/card/bank data, or unnecessary raw provider payloads in model prompts, ordinary audit projections, customer-facing messages, or broad staff views.

### Lifecycle states that drive permissions/audit

Security-sensitive state sets to preserve in downstream models:

- Reservation: Inquiry, Requested, MissingInfo, VaccinePending, SpecialReview, Waitlisted, Offered, Confirmed, CheckedIn, Active/InCare, CheckedOut, Cancelled, Rejected, and explicit NoShow or Cancelled-with-NoShow reason.
- Task: DraftSuggestion/Open/Assigned/InProgress/Blocked/NeedsManagerReview/Completed/Cancelled/Superseded.
- Document: Received/Quarantined/Rejected/Extracting/ExtractionFailed/AwaitingReview/Verified/Superseded/Archived or deleted under retention policy.
- VaccineRecord: Suggested/Extracted/PendingReview/VerifiedCurrent/VerifiedExpired/Rejected/ExceptionRequested/ExceptionApproved/Superseded.
- CareNote: Draft/RecordedInternal/NeedsReview/ApprovedForCustomerSummary/SentPublished/CorrectedVoided.
- Incident: DraftReported/NeedsManagerReview/Investigating/CustomerCommunicationDrafted/ApprovedForSend/Resolved/Closed/Reopened.
- Message: Draft/NeedsReview/Approved/Queued/Sent/Delivered/Failed/Received/SuppressedCancelled.
- Payment/Deposit: NotRequired/Required/Pending/Authorized/Paid/Failed/Waived/Refunded/PartiallyRefunded/Cancelled plus provider-backed checkout_created/checkout_sent/expired/disputed/requires_review where payment requests are modeled.
- AuditEvent: append-only Recorded; corrections use later correcting/superseding events.

State transitions, not just entity updates, must emit audit events. Denied, blocked, suppressed, failed validation, and dead-letter outcomes are audit-worthy; they are part of the safety story.

## Roles and personas

Canonical personas for security synthesis:

- Customer / owner / pet parent: may submit/update their own profile and documents through approved surfaces, receive approved messages, view customer-safe status, and request changes. Cannot approve staff/manager gates, see internal notes unless explicitly customer-visible, or see other customers/pets/staff data.
- Staff / front desk / caregiver: performs routine intake, check-in/out prep, customer follow-up, document collection, belongings, routine care execution, factual drafts, and task evidence within assigned location/scope. Cannot clear medical/behavior/payment/booking exceptions unless policy grants a specific gate.
- Lead staff / shift lead: accepts shift packets, routes blocked tasks, triages ordinary escalations, and may resolve approved lead-level gates. Cannot clear manager-only exceptions unless the approved role matrix says so.
- Manager / owner / admin: approves capacity/ratio/booking exceptions, medical/behavior/vaccine exceptions where policy permits, sensitive customer language, refunds/waivers/discounts/forfeitures, policy configuration, staff/admin controls, and production operational exceptions.
- Payment reconciliation / provider operator: handles provider-status reconciliation, failed payments, duplicate/partial/unknown transactions, webhook verification issues, and provider/store conflicts; should have payment-specific views without broad medical/care detail unless needed.
- Legal / compliance / privacy: reviews chargeback/regulatory/legal threats, suspected fraud, payment secret exposure, sensitive data incidents, retention/legal hold exceptions, and customer-visible audit boundaries.
- Engineering / integration owner: manages adapters, webhook verification/mapping, idempotency failures, provider schema ambiguity, deployment secrets, and tool/runtime configuration. Should not gain business approval authority by virtue of technical access.
- System / deterministic policy evaluator: app-owned deterministic actor that validates policy, schemas, idempotency, permissions, and guardrails; may apply safe state-machine checks only within approved policy.
- AI workflow worker / Hermes worker / agent: bounded assistant actor. May read least-privilege typed context, summarize, extract, classify, draft, recommend, flag risks, and request internal review. Must not approve its own suggestions or directly execute high-risk external effects.
- External integration / provider: PMS/payment/email/webhook/source-system actor. Its payloads are untrusted until verified, mapped, reconciled, and recorded with source/provenance.

Role-matrix approval gate: the exact authority matrix for staff vs lead vs manager vs owner/admin vs specialist roles is not approved. Downstream synthesis must preserve this as a human approval gate and avoid hard-coding real business authority from provisional role labels.

## Workflow events and lifecycle transitions that mutate state

Canonical workflow/event families that require permission checks and audit:

- Identity/account: customer created/updated/merged/archived, pet created/updated/archived, portal account linked, contact preference changed, do-not-contact set, duplicate merge approved.
- Intake and documents: document uploaded/imported/quarantined/classified/extracted/reviewed/rejected/superseded; vaccine suggestion created/disposed; vaccine record verified/expired/rejected/exception approved; missing-info task created/resolved.
- Reservation lifecycle: inquiry/request created, missing-info/vaccine-pending/special-review set, waitlist/offered/confirmed transitions, capacity hold created/extended/released, room/suite assignment changed, check-in/check-out/active state changes, cancellation/rejection/no-show, external provider write/read/reconciliation.
- Capacity/service assignment: availability snapshot created, room/suite/yard/groomer/trainer/labor/capacity slot held or released, overbooking/waitlist/holiday exception requested/approved/rejected, group-play/day-boarding lane suggested/confirmed/reassigned.
- Care execution: feeding/medication/care tasks created/assigned/started/completed/blocked, medication skipped/exception escalated, care note recorded/corrected/voided, daily update/Pawgress drafted/approved/sent/suppressed, shift handoff accepted, room/yard cleaning turnover controls capacity return.
- Incidents and safety: incident reported, severity changed, manager review requested, evidence attached, investigation updated, customer communication drafted/approved/sent/suppressed, incident resolved/closed/reopened.
- Messaging: inbound message received, draft created/edited, approval requested, approved/rejected/returned, queued, send attempted, delivered/failed/bounced/suppressed/unsubscribed/replied.
- Payments/pricing: quote/policy snapshot evaluated, deposit required/waived/failed/paid, checkout created/sent/expired, provider webhook verified/rejected/deduped, reconciliation result, failed payment task, refund/discount/waiver/forfeit request, manager approval/rejection, provider command executed, ledger sync.
- AI/runtime: workflow event enqueued, prompt packet built, model/tool run started/completed/failed, structured output validated/rejected, policy denial, human-review packet created, dead-letter entry created, retry/replay/deduplication outcome, model/prompt/policy version changed.
- Security/admin/compliance: role/permission change, integration credential/secret configured or rotated, retention/legal hold set/released, export/download, privileged audit view, redaction override, production worker/tool permission change.

Mutation rule: every state mutation must have an actor, source event, subject, before/after representation or redacted diff, policy/version context, permission decision, and audit event. Every high-risk mutation must reference a human approval or explicit deterministic policy allowance.

## AI runtime boundaries

### Invocation and queue boundary

Canonical runtime posture from `t_cf266086`:

- Production should use an app-owned durable inbox/queue as the primary pattern.
- The app verifies/adapts source events, builds typed prompt packets, invokes Hermes/AI asynchronously, validates structured output, persists workflow/audit/task records, and lets deterministic policy plus human review decide side effects.
- Webhooks should be durably accepted and verified before ACK and queued for semantic mapping; long model work must not happen inline.
- Synchronous AI should be limited to read-only previews or low-latency deterministic app checks, not external writes/customer sends.
- CLI/manual Hermes runs are acceptable for prototypes/backoffice/manual jobs, not as the stable production app contract.

### Prompt packet policy

Prompt packets should be least-privilege and purpose-built:

- Include only the entity snapshots, redacted excerpts, policy refs, evidence refs, and task context required for the specific workflow.
- Prefer stable ids, semantic statuses, policy versions, and redacted summaries over raw documents, raw provider payloads, full message bodies, or broad customer histories.
- Include role/purpose, allowed action vocabulary, required output schema, review gates, escalation rules, retention/logging classification, source/evidence refs, and idempotency key.
- Exclude secrets, raw payment instruments, webhook signatures, raw card/bank data, unnecessary PII, unnecessary medical/care detail, and unbounded prior model memory.
- Mark every value as trusted, untrusted, missing, stale, conflicting, or source-pending where relevant. AI must not turn untrusted input into business truth.

### Tool permission policy

Minimum tool-permission strata for downstream synthesis:

- ReadOnly: query approved internal projections and redacted evidence refs.
- DraftOnly: create drafts/recommendations/review packets but no live operational effect.
- InternalTaskOnly: create or recommend internal staff/manager tasks when policy permits and dedupe/idempotency rules pass.
- ApprovedCustomerSend: send only when recipient, facts, template/copy, send condition, and approval are fixed by policy/human approval.
- ApprovedProviderMutation: execute bounded provider/system mutation only with authorized approval, typed command, idempotency key, before/after audit, and rollback/reconciliation path.
- Forbidden/NeverAutomate: raw payments/refunds/waivers/discounts/forfeitures, autonomous medical/safety/eligibility decisions, high-risk customer messages, credential/secret exposure, policy changes, live booking exceptions, and unsupported provider writes.

Current Rust vocabulary anchors: `AutomationLevel::{SafeToAutomate, DraftOnly, InternalTaskOnly, ManagerApprovalRequired, NeverAutomate}` and `ReviewGate::{ManagerApproval, MedicalDocumentReview, BehaviorReview, CustomerMessageApproval, RefundOrDepositException}`. The final security model likely needs finer-grained tool/action permissions than this current vocabulary.

### Structured outputs and validation

AI outputs should be typed and validated before any persistence beyond audit/logging:

- Required envelope: status, summary, structured output, recommended actions, risk flags, verification notes, human review reason, source refs, confidence/uncertainty where appropriate.
- Allowed statuses: Completed, NeedsHumanReview, RejectedByPolicy, NeedsMoreInformation, FailedSafely, plus domain-specific blocked/suppressed/dead-letter outcomes.
- Free text must not drive hidden side effects. Only schema-valid, policy-valid, permission-valid structured actions may become tasks/drafts/review packets.
- Validation failures, unsafe outputs, missing required inputs, policy denials, and tool errors route to human review/dead-letter rather than fallback mutation.
- Model confidence is never authority for payments, vaccine/medical compliance, eligibility, incident conclusions, refunds/waivers, booking exceptions, or customer-facing legal/safety language.

### Memory and context policy

- AI memory is not a source of operational truth.
- Prompt/context reuse must be scoped by tenant/location/customer/pet/reservation and purpose; avoid leaking cross-customer or cross-location data.
- Prior AI summaries can be used as historical artifacts only with source refs and audit trail; they are not authoritative by themselves.
- Retention for prompts, outputs, tool traces, redacted excerpts, documents/OCR, raw provider payloads, DLQ records, and audit logs remains unresolved and must be approved before production.

## Messaging and customer-facing automation boundaries

Default posture: customer-facing messages are draft/review unless a deterministic, preapproved send path fixes recipient, facts, template/copy, policy basis, timing, suppression conditions, and audit evidence.

Allowed AI messaging roles:

- Draft routine factual messages from trusted/current sources for staff review.
- Summarize customer inquiries, missing-info needs, care-update evidence, payment status, or incident timelines for internal review.
- Identify missing or ambiguous facts and route to staff/manager rather than fill gaps.
- Prepare customer-safe wording only when sensitive categories are excluded or explicitly approved.

Required review/approval before customer send:

- Medical, vaccine, medication, allergy, care, behavior, incident, safety, legal/liability, eligibility, service denial/acceptance, refund, waiver, discount, forfeiture, cancellation/no-show, payment-sensitive, disputed, complaint, or exception language.
- Any promise of availability, booking confirmation, check-in/check-out completion, refund timing, discount/waiver, policy exception, or service acceptance/denial.
- Any message using non-approved free-form AI copy rather than approved deterministic template and fixed facts.

Outbound chain of custody to audit:

1. Drafted, with source refs and AI/model/tool version when applicable.
2. Edited, with actor and diff/redacted diff.
3. Approval requested, with review gate and reviewer role.
4. Approved/rejected/returned, with actor, timestamp, reason, and scope.
5. Queued, with provider/channel/recipient and idempotency key.
6. Send attempted, delivered/failed/bounced/suppressed/unsubscribed/replied.

Suppression is a first-class outcome: messages should be suppressed when facts are missing, stale, conflicting, unverified, overly sensitive for the recipient, legally/compliance risky, or outside approved templates.

## Payments, deposits, and financial safety inputs

Payment/pricing truth comes only from approved policy records, reservation/payment stores, trusted provider adapters, manager/staff approvals, and audit/workflow events. Customer claims, staff free text, screenshots, emails, raw provider JSON, unsigned/unverified webhooks, and AI summaries are not payment truth by themselves.

Security model requirements:

- Keep provider ids/raw payloads in adapter/boundary DTOs; domain-facing contracts use semantic amounts/statuses/references/reasons/review gates.
- Verify provider signatures over raw bodies before parsing business events; durably store verified inbound events before processing.
- Use provider event id plus provider account id as dedupe key; process idempotently to prevent double charges, double refunds, double reminders, double hold releases, or duplicate tasks.
- Provider `succeeded` permits recording/reconciliation, not automatic reservation confirmation or ledger mutation unless approved adapter policy says so.
- Refunds, discounts, comps, credits, fee/deposit waivers, forfeitures, write-offs, manual price changes, and payment-processing-fee waivers require human approval before execution or customer commitment.
- Customer-facing deposit/cancellation/no-show/refund/balance/dispute messages remain drafts until approved unless a deterministic send path is separately approved.

## Audit model inputs

AuditEvent minimum shape from `t_0a8f9c6e`:

- Required fields: id, occurred_at, recorded_at, actor, subject, source, taxonomy, before, after, ai_suggestion, approval, outbound_message, evidence, redaction, integrity.
- Immutability: append-only; corrections are later events, not mutation of the original event.
- Actor model: customer, staff, manager, system, AI agent, external integration. Current Rust `ActorRef` lacks a distinct external-integration actor; this is a foundation gap.
- Taxonomy: entity lifecycle, workflow/AI, messaging, external integration, security/admin/compliance.
- Redaction: sensitive before/after values should store field path, change kind, old/new hash or token, and safe summary; raw values only in governed evidence storage when justified.
- Approval invariant: AI agents may request approval but may not approve their own suggestions. Applying an approved change references approval id, suggestion id when present, source workflow event id, and before/after state.

Minimum audit events by action class:

- Create/update/status transition for every core entity and lifecycle state above.
- Any AI prompt/output/tool run, validation result, policy denial, review request, approval/rejection, and external action.
- Any document/vaccine extraction, human verification, exception, or eligibility computation.
- Any payment provider webhook/command, reconciliation event, refund/waiver/discount/forfeit approval/execution.
- Any customer-message draft/edit/approval/send/suppress/delivery failure/reply.
- Any security/admin event: role changes, permission grants, privileged views, export/downloads, secret rotation/configuration, retention/legal-hold changes.

Audit projections should be role-scoped. Broad staff views should see safe summaries and next actions, while privileged compliance/security views may access governed evidence under retention/legal-hold rules.

## Assumptions

- Single-location or small-group internal operations is the initial product shape; multi-location/enterprise separation is not yet modeled in detail but should be anticipated with location-scoped policy and access.
- Gingr/PMS/provider integrations are boundary surfaces; raw events are untrusted until verified, semantically mapped, and reconciled.
- Current role labels are provisional and must not be treated as approved payroll titles or real permissions.
- Existing Rust contracts provide semantic anchors but not a full production authorization system.
- For v1, vaccine/document verification and high-risk medical/behavior/payment/customer-message outcomes remain human-reviewed unless a later approved trusted integration narrows the gate.
- AI can help create internal review work, but task auto-generation at scale needs explicit approval by trigger/kind/priority/assignee.

## Open questions

1. Role matrix: Which exact roles can view, draft, create, assign, approve, execute, cancel, suppress, export, or delete/retire each entity/action class?
2. Role matrix: Are lead staff, payment reconciliation, legal/compliance/privacy, engineering/integration owner, and owner/admin distinct production roles or responsibilities under manager/admin?
3. AI governance: Which workflows, if any, are `SafeToAutomate` without per-run human review, and what deterministic preconditions make them safe?
4. AI governance: What is the final tool-permission matrix by agent type/workflow, including read-only vs draft vs internal-task vs customer-send vs provider-mutation tools?
5. AI governance: What model/provider/prompt/version provenance must be retained, and who can inspect prompts/outputs/tool traces?
6. Retention: What are approved retention and deletion/legal-hold periods for audit logs, prompt packets, AI outputs, documents/OCR, raw provider payloads, payment summaries, messages, incident evidence, media, and DLQ records?
7. Privacy/redaction: Which raw sensitive values may be stored in governed evidence storage, and what redacted projections are available to staff, managers, customers, compliance, and AI?
8. External integrations: Is Gingr the source-of-truth PMS/ledger, what provider APIs/webhooks are approved, and which mutations are allowed in v1?
9. Messaging: Which customer-facing templates/send paths are deterministic and preapproved, if any, and what classes remain always staff/manager reviewed?
10. Documents/vaccines: Can future trusted vet/EHR structured integrations bypass manual review for exact records, or are all v1 medical/vaccine compliance outcomes human-reviewed?
11. Payments: Which provider is approved, which exact payment/refund/waiver/discount roles exist, and what customer-facing policy copy can be used?
12. Auditability: Do audit events need per-subject sequence numbers and replay guarantees, or is append-only timestamp/id ordering sufficient for v1?
13. Customer-visible history: Which audit/message/status events, if any, are customer-visible through a portal?
14. Security operations: What incident-response workflow covers accidental sensitive-data exposure, webhook secret compromise, model prompt leakage, or unauthorized role changes?

## Approval gates to preserve for downstream synthesis

Human approval gates explicitly required by this task:

1. Role matrix.
   - Do not finalize permissions until the human-approved authority matrix exists.
   - Keep provisional roles separate from approved production permissions.
   - Distinguish view, draft, approve, execute, export, configure, and audit privileges.

2. AI governance.
   - Do not approve production AI tool permissions, customer-message automation, provider mutations, task auto-generation, or safe-to-automate classes without explicit policy.
   - Preserve AI-as-assistant posture: draft, summarize, recommend, flag, and request review by default.
   - Require schema validation, deterministic policy checks, audit correlation, idempotency, and human review for high-risk outcomes.

3. Retention policy.
   - Do not guess retention/deletion/legal-hold durations for audit logs, prompts/responses, documents/OCR, raw provider payloads, payment records, incidents, messages, media, or model/tool traces.
   - Require role-scoped redacted projections and governed raw-evidence storage before production.

Additional inherited gates:

- Medical/vaccine/medication/allergy/feeding/behavior/incident/safety ambiguity.
- Vaccine/document approval and licensed-vet/source proof verification.
- Deposit/payment/refund/waiver/forfeit/discount/credit/cancellation/no-show exceptions.
- Booking acceptance/rejection exceptions, overbooking, waitlist release, capacity holds/releases, group-play reinstatement/override, room capacity return.
- Customer-facing messages involving health, safety, legal, payment, incident, eligibility, refusal, or policy exceptions.
- Provider/system mutations such as reservation status updates, check-in/out, payment commands, refunds, message sends, webhook registration, raw payload storage, and capacity release.

## Conservative downstream rule

When source facts or policy are missing, stale, contradictory, sensitive, unverified, or outside the role/tool permission boundary, the security model should produce a denied/review/blocked/suppressed state with audit evidence. It must not silently invent policy, broaden a role, expose raw sensitive data, convert AI confidence into authority, send a customer message, or execute a provider/system mutation.
