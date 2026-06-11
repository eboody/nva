# AI governance policy

Purpose: define the governance rules for AI-assisted pet-resort workflows, including risk classes, approval gates, structured-output requirements, logging/audit expectations, and safe fallback behavior.

Status: draft policy requiring human approval before production use. This document defines the proposed governance model, but it does not approve production AI tool permissions, customer-message automation, provider mutations, payment-affecting operations, safety-affecting operations, or final role authority. The final AI governance policy is a human approval gate.

## Non-negotiable principles

1. No unsupervised high-risk actions.
   - AI may propose, draft, classify, extract, summarize, recommend, and route work.
   - AI must not autonomously execute payment-affecting, safety-affecting, legal/liability, medical/vaccine, incident, eligibility, live booking exception, external-provider mutation, or sensitive customer-facing operations.
   - A high-risk operation can proceed only through deterministic application policy, typed approved tool paths, required human approval, idempotency controls, and audit records.

2. Structured outputs are required.
   - Every AI workflow result must use the approved shared result envelope plus a workflow-specific schema.
   - Free text is never executable authority. Prose such as "booking confirmed", "message sent", "vaccines approved", or "refund issued" has no effect unless matched by a schema-valid, policy-valid, permission-valid, audited action through an approved tool path.
   - AI output must include status, summary, structured output, recommended actions, risk flags, verification, source references, uncertainty, missing inputs, approval requirements, and safe-log content where applicable.

3. Uncertainty, confidence, and escalation are explicit.
   - AI must represent uncertainty in structured fields, not hidden in prose.
   - Confidence may help prioritize review; it is never authority for payments, refunds, waivers, vaccine/medical compliance, care safety, incidents, booking eligibility, service denial/acceptance, or customer-facing sensitive language.
   - Missing, stale, conflicting, untrusted, low-confidence, or policy-ambiguous inputs must produce `NeedsHumanReview`, `NeedsMoreInformation`, `RejectedByPolicy`, `FailedSafely`, `Suppressed`, or a domain-specific blocked/dead-letter state.

4. Logs and audit trails must be safe.
   - Ordinary logs may include workflow ids, run ids, event ids, subject ids, schema names/versions, validation outcomes, safe summaries, policy decisions, approval gates, and redacted excerpt references.
   - Logs must not include unnecessary sensitive payloads, secrets, hidden prompts, model chain-of-thought, raw documents/OCR, raw customer messages, raw incident narratives, raw provider payloads, webhook signatures, payment/card data, tokens, or credential material.
   - Raw sensitive evidence, where retained at all, belongs in governed evidence storage with retention/legal-hold controls, not broad operational logs.

5. No hidden state changes.
   - AI suggestions do not mutate business state by themselves.
   - Tools that create tasks, send messages, update reservations, write provider records, change payment state, approve documents, or close incidents must record the actor, source event, reason, approval requirement, policy/version context, before/after or redacted diff, idempotency key, and audit event.
   - AI may not approve its own suggestions.

6. Human override remains available and auditable.
   - Authorized humans may approve, reject, edit, suppress, override, or reverse AI recommendations within the approved role matrix.
   - Overrides must record actor, timestamp, reason, scope, affected subject, policy context, and before/after or redacted diff.
   - Override workflows must not erase the original AI result or prior decision; corrections and supersessions are appended as later events.

## Governance boundaries

The pet-resort application owns state, permissions, policy evaluation, validation, audit, queues, retries, idempotency, and side-effect execution. AI/Hermes workers are bounded assistant actors.

Required runtime boundary:

1. The application verifies and adapts source events.
2. The application builds least-privilege typed prompt packets from trusted state, redacted evidence, policy refs, allowed actions, forbidden actions, output schema, approval gates, and logging rules.
3. AI returns only the declared structured result.
4. The application parses and validates JSON only.
5. Deterministic validators check correlation, subject scope, source refs, permissions, policy, review gates, idempotency, and safety.
6. Safe results are persisted as workflow/audit records or review packets.
7. Side effects occur only through approved deterministic tool paths after all required approvals exist.

CLI/manual Hermes runs may support prototypes, backoffice investigations, or staff-drafted packets. They are not the stable production contract for live customer, payment, safety, or provider effects.

## Risk classification

### Class 0: suggestion or draft, internal only

Examples:
- Internal summaries of a reservation, task, document, message, or incident packet.
- Draft staff notes, task descriptions, handoff summaries, checklist suggestions, or internal review packets.
- Non-sensitive knowledge-base suggestions using approved public/business-safe policy content.

Default AI role:
- Draft, summarize, classify, extract low-risk labels, and recommend next internal steps.

Allowed automation:
- May be displayed internally after schema validation and policy checks.
- May create draft records or draft review packets only if the workflow has an approved internal-draft path.

Review/approval:
- Staff review required before relying on the content for operational decisions.
- Manager review required if the draft touches safety, medical/care, behavior, incidents, payment, booking exceptions, legal/liability, customer-facing commitments, or uncertain policy.

Logging:
- Safe summary, run/event/subject ids, schema/version, validation status, source refs, risk flags, and redacted excerpt refs.

### Class 1: low-risk internal triage

Examples:
- Routing routine missing-information tasks.
- Suggesting priority/assignment for ordinary internal operations tasks.
- Flagging stale records, duplicate packets, missing provider refs, or missing policy snapshots.
- Queueing a staff review packet where no customer/provider state changes occur.

Default AI role:
- Classify, recommend, and prepare internal task packets.

Allowed automation:
- Internal task creation can be allowed only when the trigger, task type, assignee class, priority rules, dedupe/idempotency key, and safe content are preapproved by deterministic policy.
- AI may not mark tasks completed or resolve exceptions unless an approved deterministic workflow permits that exact low-risk transition.

Review/approval:
- Staff or lead review for routine internal triage.
- Manager/engineering review for ambiguous permissions, high-volume automation, unexpected task spikes, policy conflicts, or integration mapping problems.

Logging:
- Actor as AI/system, triggering event, reason, task kind, subject ids, dedupe key, policy snapshot, and safe task summary.

### Class 2: customer-facing message draft

Examples:
- Missing-information request draft.
- Care update or Pawgress-style draft.
- Payment/deposit reminder draft.
- Incident follow-up draft.
- Booking availability, waitlist, cancellation, or policy-explanation draft.

Default AI role:
- Draft only; do not send.
- Identify facts used, claims not made, sensitivity, uncertainty, and review gate.

Allowed automation:
- Customer send is not allowed by default.
- A deterministic send path may be approved only for a narrow template where recipient, facts, source trust, timing, suppression conditions, allowed copy, and audit behavior are fixed by policy.
- AI-authored free-form text remains approval-gated unless specifically approved as part of that deterministic path.

Review/approval:
- Staff approval for routine factual, non-sensitive drafts when facts are current/trusted and template/copy is approved.
- Manager approval for medical, vaccine, medication, allergy, care, behavior, incident, safety, legal/liability, eligibility, service denial/acceptance, booking exception, capacity exception, refund, waiver, discount, forfeit, cancellation/no-show, payment-sensitive, complaint, or disputed content.
- Legal/compliance/privacy review for legal/regulatory threats, sensitive data exposure, fraud, chargebacks, or privacy concerns.

Logging:
- Draft id, recipient ref, source refs, reviewer gate, edit/approval/rejection history, queued/sent/suppressed/provider delivery status, and redacted diff where needed.
- Do not log raw message bodies broadly; store customer-message content in the governed message/audit store with role-scoped projections.

### Class 3: document and vaccine extraction

Examples:
- OCR/vaccine document classification.
- Extracting candidate vaccine names, dates, issuer clues, pet/owner matches, and missing fields.
- Suggesting document-review tasks.
- Producing a review packet for vaccine eligibility.

Default AI role:
- Extract candidate facts with evidence refs, confidence, uncertainty, and mismatch flags.
- Recommend review tasks.

Allowed automation:
- AI may not final-approve vaccine records, medical documents, medication instructions, allergy/care facts, or eligibility outcomes in v1.
- Candidate extracted facts remain `Suggested`, `Extracted`, or `PendingReview` until deterministic policy plus authorized human or explicitly approved trusted integration verifies them.

Review/approval:
- Medical document review is required for all v1 vaccine/document verification outcomes.
- Manager review required for medical/care policy exceptions, source ambiguity with operational impact, mismatched pet/customer identity, or eligibility hard-stop overrides.

Logging:
- Document id, extraction run id, schema/version, candidate fields, confidence band, uncertainty, source excerpt refs, validation outcome, and reviewer decision.
- Avoid raw OCR/document leakage in logs and prompt retries; use redacted excerpts or evidence refs.

### Class 4: incident classification

Examples:
- Classifying severity of a staff-reported injury, illness, aggression, escape risk, medication error, complaint, or safety concern.
- Summarizing known facts and missing investigation evidence.
- Drafting manager packets or owner-message drafts.

Default AI role:
- Triage and summarize for manager review.
- Flag risk, missing facts, conflicting evidence, and sensitive customer language.

Allowed automation:
- AI may not diagnose, assign fault, close incidents, suppress required notifications, decide customer compensation, alter care instructions, or send incident messages.
- AI may recommend incident tasks or a manager-review packet if schema/policy/idempotency checks pass.

Review/approval:
- Manager review required for severity, disposition, owner communication, resolution, closure, reopening, and safety-affecting actions.
- Legal/compliance/privacy review required for injury severity, regulatory/legal threats, suspected negligence/fraud, privacy incident, staff conduct concern, or raw evidence exposure.

Logging:
- Safe incident summary, incident id, source refs, severity recommendation, risk flags, review gate, approval/rejection/override reason, and redacted evidence refs.
- Do not log raw incident narratives or media broadly.

### Class 5: booking and eligibility recommendations

Examples:
- Reservation triage.
- Waitlist/offering recommendations.
- Check-in readiness recommendations.
- Group-play/daycare/day-boarding eligibility suggestions.
- Capacity, room/suite, service, or staffing recommendations.

Default AI role:
- Recommend status or next tasks with source refs and uncertainty.
- Draft customer follow-up text for review.

Allowed automation:
- AI may not confirm, reject, cancel, check in, check out, overbook, release waitlist, release/return capacity, waive prerequisites, approve group-play/medical/behavior exceptions, or promise availability by itself.
- Deterministic policy may apply narrow low-risk state transitions only when facts, source trust, capacity, policy snapshots, and approvals are complete and preapproved.

Review/approval:
- Staff review for routine missing-info or readiness follow-up.
- Medical document review for vaccine/medical uncertainty.
- Manager review for capacity exceptions, eligibility hard stops, overbooking, holiday/peak exceptions, special handling, behavior/safety concerns, service denial/acceptance, group-play reinstatement/override, waitlist release with customer impact, or live booking exceptions.

Logging:
- Reservation/pet/customer/location ids, recommendation, trusted/missing/conflicting source states, policy snapshot, approval requirements, and any eventual state change before/after or redacted diff.

### Class 6: payment-affecting operations

Examples:
- Deposit required/waived/failed/paid decisions.
- Checkout creation or send.
- Payment reminder and billing-status workflows.
- Refunds, discounts, credits, comps, forfeitures, fee/deposit waivers, write-offs, disputes, chargebacks, reconciliation, and provider commands.

Default AI role:
- Read trusted semantic payment projections, summarize, flag conflicts, draft review packets/messages, and recommend reconciliation tasks.

Allowed automation:
- AI must not charge, retry, capture, void, refund, waive, discount, forfeit, mark paid, alter invoices, change prices/policies, or send payment-sensitive messages autonomously.
- Provider commands require authorized approval, typed command, idempotency key, trusted adapter, before/after audit, provider result audit, and reconciliation path.

Review/approval:
- Staff or payment-reconciliation review for routine failed/missing reference follow-up with no policy exception.
- Manager approval for refunds, discounts, comps, credits, waivers, forfeitures, write-offs, cancellation/no-show exceptions, disputed amounts, amount/currency mismatches with customer impact, and sensitive payment messages.
- Legal/compliance/privacy review for chargebacks, fraud, raw payment/provider exposure, regulatory/legal threats, or suspected secret exposure.

Logging:
- Semantic payment status, amount/currency only when from trusted records, provider reference ids, reconciliation state, approval id, command id, idempotency key, provider result, and safe summaries.
- Never log card data, CVV, bank data, payment tokens, webhook signatures, secrets, or unnecessary raw provider payloads.

### Class 7: safety-affecting operations

Examples:
- Medication, allergy, feeding, medical-condition, special-care, behavior, aggression, incident, emergency, and group-play safety workflows.
- Any workflow that can affect animal or human safety, health, isolation/grouping, care tasks, or incident response.

Default AI role:
- Summarize source evidence, flag risks, identify missing/conflicting facts, and draft review packets.

Allowed automation:
- AI must not issue medical instructions, override care instructions, approve safety exceptions, clear group-play eligibility, diagnose, assign medical causation, suppress safety tasks, or close safety incidents.
- Operational care-task suggestions may be drafted only from trusted approved care profiles and still require the relevant staff/manager workflow controls.

Review/approval:
- Staff review for routine execution of approved care tasks.
- Manager or designated medical/document reviewer for care-profile changes, medication/allergy ambiguity, vaccine/medical exceptions, behavior restrictions, group-play eligibility, incident disposition, and safety policy overrides.
- Legal/compliance/privacy review where injury severity, liability, privacy, or regulatory obligations may be implicated.

Logging:
- Safe summary, subject ids, source refs, care/safety risk flags, approval gate, reviewer decision, and redacted diff for any care-profile/task/status change.
- Avoid broad logging of raw medical/care notes or incident evidence.

## Tool permission model

AI workers receive the least tool permission needed for the workflow. The approved role/tool matrix must be finalized separately before production.

Minimum strata:

| Permission stratum | What it allows | Governance rule |
| --- | --- | --- |
| ReadOnly | Query approved projections and redacted evidence refs | No mutation, no customer send, no provider command. |
| DraftOnly | Create drafts, recommendations, extraction packets, review packets | Drafts are non-authoritative and approval-gated. |
| InternalTaskOnly | Create internal tasks for approved low-risk triggers | Requires deterministic trigger, task kind, assignee class, dedupe/idempotency, and safe content. |
| ApprovedCustomerSend | Send fixed approved customer messages | Requires approved recipient, template/copy, facts, timing, suppression checks, approval record, and audit. |
| ApprovedProviderMutation | Execute bounded provider/system mutation | Requires authorized approval, typed command, idempotency, before/after audit, provider result, and reconciliation/rollback path. |
| Forbidden/NeverAutomate | Blocks unsafe actions | Includes autonomous payment, refund, waiver, discount, forfeit, medical/safety/eligibility approval, high-risk customer message, credential/secret exposure, policy change, live booking exception, unsupported provider write. |

A workflow may expose lower strata without higher strata. For example, a document worker can be ReadOnly + DraftOnly without InternalTaskOnly; a payment review worker can summarize and draft a reconciliation task without any provider-mutation tool.

## Required structured result envelope

Every AI workflow result must include at minimum:

- `schema_name` and `schema_version`.
- `workflow_name` and `workflow_version` or equivalent policy/runtime version refs.
- `run_id`, `event_id`, `correlation_id`, and `idempotency_key` where applicable.
- `subject` with type and id.
- `status`: one of `Completed`, `NeedsHumanReview`, `RejectedByPolicy`, `NeedsMoreInformation`, `FailedSafely`, or a policy-approved domain-specific suppressed/blocked/dead-letter state.
- `summary`: short safe summary, not raw sensitive content.
- `structured_output`: workflow-specific typed output.
- `recommended_actions`: drafts/recommendations only, each mapped to allowed actions.
- `risk_flags`: safety, medical, incident, payment, privacy, policy, source-trust, or prompt-injection concerns.
- `verification`: checked sources, trust/freshness/conflict state, policy refs, and skipped checks.
- `uncertainty`: structured field-level uncertainty and confidence limits.
- `missing_inputs`: required facts/sources that are absent or stale and what they block.
- `approval_requirements`: gate, reason, required approver role/class, and actions blocked until approval.
- `human_review_reason`: required whenever any gate remains open or status is not a clean internal-only recommendation.
- `source_refs` or workflow-specific evidence refs for every fact used in recommendations, messages, status suggestions, or task drafts.
- `safe_log`: safe one-line summary, ids, schema errors, and redacted excerpt refs only.

## Validation, fallback, and escalation behavior

### Schema or parse failure

Required behavior:
1. Parse JSON only.
2. Validate the shared envelope.
3. Validate the workflow-specific schema.
4. Retry once with safe validation error context only.
5. If retry fails, mark `FailedSafely`, record a safe validation-failure audit event, and escalate.

No side effects are allowed before validation succeeds except safe validation-failure logging/audit.

Escalation owner:
- Engineering/integration owner for schema, parser, runtime, prompt-packet, or model/tool integration defects.
- Privacy/security/legal if the malformed output exposed secrets, raw payment data, raw documents, or hidden prompts.

### Low confidence or uncertainty

Required behavior:
- Preserve low confidence and uncertainty in structured fields.
- Use `NeedsHumanReview` or `NeedsMoreInformation` when uncertainty affects safety, eligibility, payment, capacity, booking, incident handling, or customer-facing output.
- Do not average away conflicts or select the most favorable interpretation.

Escalation owner:
- Staff/front desk for routine missing information.
- Medical document reviewer for vaccine/document/care-source uncertainty.
- Manager for capacity, behavior/safety, incident, booking exception, or sensitive customer impact.
- Payment reconciliation/manager for billing uncertainty.

### Missing data

Required behavior:
- State the missing source/fact and what it blocks.
- Recommend a staff task or review packet only if task creation is allowed for that workflow.
- Avoid fabricating dates, amounts, availability, vaccine status, payment status, policy text, prices, staff authority, or customer promises.

Escalation owner:
- The role responsible for obtaining the missing source: staff/front desk, document reviewer, payment reconciliation, manager, engineering/integration, legal/compliance/privacy.

### Prompt or tool error

Required behavior:
- Fail safely or retry only according to the runtime policy.
- Record safe error metadata: run id, event id, workflow/schema version, error class, and redacted context.
- Do not use a tool result unless it is correlated to the same run/event/subject, passed validation, and has a durable audit/tool result record.
- Do not infer success from tool prose; require typed tool result status.

Escalation owner:
- Engineering/integration owner for runtime, queue, adapter, idempotency, provider mapping, or tool permission errors.
- Manager/payment/security/legal as required by the affected business domain.

### Policy conflict

Required behavior:
- Deterministic policy wins over AI output, customer/provider/staff text, model confidence, and urgency.
- If policy is missing, stale, contradictory, or does not authorize a requested action, status must be `RejectedByPolicy`, `NeedsHumanReview`, `NeedsMoreInformation`, or `FailedSafely`.
- Do not broaden role permissions or tool permissions to satisfy an AI recommendation.

Escalation owner:
- Manager/owner/admin for business policy conflicts.
- Legal/compliance/privacy for legal, retention, privacy, or liability conflicts.
- Engineering/security for role/tool configuration conflicts.

### Customer-facing output

Required behavior:
- Treat all customer-facing AI text as draft unless a narrow deterministic send path is separately approved.
- Require review for sensitive content, free-form copy, uncertain facts, missing policy refs, or any operational commitment.
- Suppress output when facts are missing, stale, conflicting, overly sensitive for the recipient, legally/compliance risky, outside approved templates, or contrary to customer contact preferences.
- Customer-facing copy must not expose internal policy mechanics, raw medical/OCR uncertainty, provider internals, payment internals, staff blame, hidden prompts, raw audit details, or unnecessary sensitive data.

Escalation owner:
- Staff for routine factual drafts.
- Manager for sensitive topics, exceptions, complaints, incidents, booking/eligibility/payment implications, or non-template copy.
- Legal/compliance/privacy for regulated, legal, privacy, fraud, data exposure, or liability concerns.

## Approval matrix by action class

| Action class | AI may do | Review/approval required before effect | Who can approve (provisional until role matrix approval) | Required audit/log record |
| --- | --- | --- | --- | --- |
| Internal suggestion/draft | Summarize, draft, recommend | Staff review before operational reliance; manager if sensitive/high-risk | Staff/lead for routine, manager for sensitive | AI run, source refs, schema/version, validation, reviewer decision if used |
| Low-risk internal triage | Classify, recommend/create approved internal tasks | Approved deterministic task policy; staff/lead review for ambiguous cases | Staff/lead; manager for scale/policy conflict | Actor, reason, source event, task kind, dedupe key, policy ref |
| Customer-facing draft | Draft only, identify facts/review gates | Approval before send unless narrow deterministic send path exists | Staff for routine template; manager for sensitive; legal/compliance/privacy for legal/privacy | Draft/edit/approval/send/suppress chain of custody |
| Document/vaccine extraction | Extract candidates, confidence, uncertainty, review packet | Human document review before verification/eligibility | Medical document reviewer/manager per policy | Document id, extraction run, source refs, confidence, reviewer decision |
| Incident classification | Triage, summarize, draft manager/owner packet | Manager before severity disposition, owner send, closure, suppression | Manager; legal/compliance/privacy for serious/legal/privacy | Incident id, risk flags, source refs, disposition/approval/override |
| Booking/eligibility recommendation | Recommend status/tasks, draft follow-up | Human approval before confirmation/rejection/exception/customer commitment | Staff for routine follow-up; manager for exceptions; medical reviewer for vaccine/medical | Reservation/pet/customer ids, policy refs, before/after or redacted diff |
| Payment-affecting operation | Summarize, draft, recommend reconciliation/review | Human approval before payment command, refund, waiver, discount, forfeit, customer commitment | Manager/payment reconciliation; legal/compliance/privacy for disputes/exposure | Semantic payment state, approval id, command id, idempotency, provider result |
| Safety-affecting operation | Summarize evidence, flag risk, draft review packet | Human approval before care/safety/behavior/group-play/incident effect | Staff for approved care task execution; manager/medical reviewer for changes/exceptions | Care/safety subject ids, risk flags, source refs, reviewer decision, redacted diff |
| Provider/system mutation | Recommend typed command only where allowed | Approval and approved tool path before execution | Manager/owner/admin/payment/engineering as domain requires | Actor, approval, command, idempotency, before/after, provider result, reconciliation |
| Policy/tool/role change | Flag need for change only | Human admin/security approval; no AI self-approval | Owner/admin/security/engineering | Security/admin audit event, approver, reason, changed scope |

## Audit requirements

Every AI run must create or contribute to an append-only audit/workflow record with:

- AI/runtime actor identity, model/provider/config ref without secrets, prompt packet or packet hash, workflow/schema/policy versions, and run id.
- Triggering event, subject, source refs, causation ids, correlation id, and idempotency key.
- Data classification and redaction state.
- Output validation result, status, safe summary, risk flags, uncertainty, missing inputs, and approval requirements.
- Any tool-call request/result, including typed command, allowed-action mapping, approval reference, before/after or redacted diff, provider/system result, and retry/dead-letter state.
- Human review request, reviewer, approval/rejection/return decision, edits, reason, scope, and timestamp.
- Human override, suppression, rollback/reconciliation, or correction events as separate appended records.

Audit projections must be role-scoped. Broad staff views should show safe operational summaries and next actions; privileged compliance/security views may inspect governed evidence under approved retention/legal-hold rules.

## Safe logging rules

Allowed in ordinary logs:
- `run_id`, `event_id`, `workflow_name`, `schema_name`, `schema_version`, subject ids, correlation id, idempotency key.
- Result status, validation outcome, policy outcome, approval gate, safe one-line summary.
- Schema errors as JSON pointer, expected type/enum/required field, and high-level actual type; avoid raw sensitive values.
- Redacted excerpt refs and governed evidence refs.

Forbidden in ordinary logs:
- Secrets, API keys, tokens, credential metadata, webhook signatures, raw card/bank data, CVV, payment tokens, signed provider payloads.
- Hidden prompts, chain-of-thought, internal model reasoning, or prompt-injection payloads beyond safe summaries.
- Raw documents/OCR, unredacted medical/care notes, incident narratives, staff notes, customer messages, email bodies, provider JSON, screenshots, media, or unnecessary PII.
- Free-text claims that a side effect happened unless matched by a validated tool result and audit event.

## Human override policy

Human override must remain possible for authorized users and must be auditable.

Rules:
- The UI/workflow must let authorized humans approve, reject, edit, suppress, request more information, override policy within their authority, or escalate.
- Overrides must be explicit, not implicit UI side effects.
- Overrides must capture reason, scope, actor, timestamp, source refs, policy/version context, approval gate, and any before/after or redacted diff.
- An override cannot delete or rewrite the AI recommendation, original evidence, or prior decision; it appends a correction/supersession event.
- AI may summarize an override history but may not create or approve its own override.

## Production enablement checklist

Before enabling any AI workflow in production:

- Final AI governance policy is human-approved.
- Final role/authority matrix is human-approved.
- Retention/legal-hold policy for prompts, outputs, tool traces, documents/OCR, raw provider payloads, messages, incidents, media, and audit logs is approved.
- Prompt packet schema and workflow output schema are registered and versioned.
- Workflow has least-privilege prompt construction and redaction rules.
- Workflow declares allowed actions, forbidden actions, approval gates, escalation owners, tool permission strata, and safe logging rules.
- Parser and validators reject malformed JSON, schema mismatch, wrong subject/event, unknown fields, invalid enums, missing source refs, forbidden actions, and unapproved side-effect claims.
- Runtime retries once only with safe validation error context and escalates after failure.
- Deterministic policy validators check source trust, freshness, idempotency, permissions, review gates, and side-effect eligibility.
- Tool executors accept typed commands only and record actor, reason, approval, before/after, idempotency, result, and reconciliation state.
- Tests cover malformed output, wrong subject/event, prompt injection in user/provider text, forbidden free-text action claims, forbidden structured actions, missing approval gates, unsafe uncertainty, low-confidence extraction, customer-message suppression, tool failure, retry success, retry failure, and human override audit.

## Open approval gates

The following remain explicit human approval gates and must not be treated as production permission:

1. Final AI governance policy.
2. Role matrix: exact roles that can view, draft, create, assign, approve, execute, cancel, suppress, export, configure, inspect logs, inspect prompts/outputs/tool traces, and override each entity/action class.
3. Safe-to-automate classes, if any, including deterministic preconditions and test evidence.
4. Tool permission matrix by agent type/workflow.
5. Customer-message automation paths and approved templates/copy.
6. Provider/system mutations allowed in v1.
7. Retention/deletion/legal-hold policy for AI, audit, document, message, payment, provider, incident, and media records.
8. Raw evidence storage and redacted projection policy.
9. Model/provider/prompt/version provenance retention and inspection authority.

Conservative rule: when policy, source facts, role authority, confidence, schema validity, approval status, or tool permission is missing, stale, conflicting, untrusted, or outside scope, the system must deny, suppress, escalate, block, or fail safely with audit evidence. It must not silently invent policy, broaden a role, expose raw sensitive data, convert AI confidence into authority, send a customer message, or execute a provider/system mutation.
