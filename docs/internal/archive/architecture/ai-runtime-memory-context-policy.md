# AI runtime memory and context policy

Purpose: define the default runtime context, memory, privacy, and audit policy for AI/Hermes workers in the pet-resort product. This is an architectural policy input for the AI runtime design, not an approval to send live customer, pet, staff, payment, or medical data to an LLM in production.

Human approval gate: the exact data fields sent to any LLM/runtime, the production invocation path, the provider/model, retention settings, and tool permissions must be approved before production use. Until that gate is approved, treat all LLM calls as prototype/backoffice-only and use synthetic, redacted, or explicitly approved test data.

## Default posture

AI workers are source-grounded assistants that produce typed drafts, review packets, risk flags, and internal task recommendations. They are not systems of record and are not authorities for medical, vaccine, safety, payment, reservation, staffing, customer-message, policy-exception, or provider-write decisions.

The app owns context assembly. A worker should receive a minimal typed prompt packet, fetch only approved records by ID through app-owned repositories, return a `WorkflowResult`-style structured output, and rely on deterministic policy validators plus human approvals before any side effect.

Core rules:

1. Minimize first: send only the fields required for the specific workflow action and recipient role.
2. Reference by ID by default: pass IDs and typed policy/review metadata in the event packet; fetch details only when the workflow allowlist permits them.
3. Prefer semantic summaries over raw records: use statuses, flags, policy refs, and source citations instead of full provider payloads, full notes, full documents, or full message threads.
4. Preserve source and uncertainty: every generated recommendation must cite the records, policy snapshots, and review gates it used; missing/conflicting facts must become `NeedsMoreInformation`, `NeedsHumanReview`, or `RejectedByPolicy`, not model guesses.
5. Keep secrets and raw provider material out of the LLM. Tool credentials, payment secrets, signed payloads, raw card/bank data, webhook signatures, and API keys are never prompt context.
6. Log enough to audit decisions without storing unnecessary sensitive prompt text.

## Runtime context packet

The prompt/event packet passed directly to Hermes or another AI worker should contain only the routing, policy, and task context needed to choose and execute an approved workflow.

Allowed direct packet fields:

- `workflow_name` / agent spec name and version.
- Workflow goal/purpose and output schema name.
- `WorkflowEvent` envelope fields needed for routing:
  - `event_id`.
  - `event_type`.
  - `occurred_at`.
  - `actor` as a typed `ActorRef`, minimized to role and ID; no staff/customer contact details unless explicitly required.
  - `location_id`.
  - `subject` as typed customer/pet/reservation/workflow/external IDs.
  - `policy_context`: allowed actions, automation level, required review gates, and relevant policy snapshot/ref IDs.
- Explicit `AllowedAction` list, for example read entities, extract structured data, draft customer message, create internal task, suggest reservation status, suggest play eligibility, summarize care notes, or flag risk.
- Review-gate instructions and forbidden actions from the selected agent spec.
- Record IDs and time windows to fetch, not whole records, when the worker can retrieve approved data by ID.
- Minimal source excerpts that have already been redacted and approved for the workflow, when a worker cannot fetch by ID.
- Output contract: structured schema, required citations, required redaction, and safe-failure statuses.

Do not put broad database snapshots, complete customer profiles, complete pet histories, raw provider webhook bodies, full message inboxes, raw payment payloads, unredacted vaccine documents, or staff-only notes directly in the initial packet.

## Data fetched by ID at invocation time

Workers should fetch richer context by ID only through app-owned repositories or tools that enforce role, tenant/location, workflow, and field-level policy. Fetches must be scoped to the subject, time window, and allowed action in the event packet.

Typical allowed fetches by workflow:

| Workflow class | Fetch by ID allowed | Minimized payload shape |
| --- | --- | --- |
| Inquiry intake / lead conversion | Customer/lead record, pet names/species, requested service/date, prior approved contact preference, approved policy snippets | Missing-field checklist, service/date preferences, safe reply draft facts; avoid full history unless relevant |
| Booking triage | Reservation request, pet IDs, vaccine status summaries, deposit status, availability/capacity snapshot, policy refs | Eligibility/readiness status, hard stops, policy refs, proposed review gate; no raw document/payment payloads |
| Vaccine document | Document metadata, OCR/extracted vaccine candidates, pet/customer IDs, vaccine policy ref | Document source, vaccine names/dates/candidate confidence, ambiguity flags; raw image/OCR only if explicitly approved for document workflow |
| Daily care update | Stay/reservation ID, approved staff task evidence, approved care-note excerpts, selected photo/media refs if approved | Warm summary facts, risk flags, omitted sensitive details, customer-message draft requiring review |
| Incident escalation | Incident ID, typed incident fields, involved pet/reservation/location IDs, witness/staff refs, evidence refs | Timeline, severity/risk flags, missing fields, manager/customer review packet; no autonomous closure |
| Manager daily brief | Location/day, occupancy/arrival/departure/labor/task snapshots, pet-care watchlist summaries, revenue opportunity statuses | Aggregate/role-appropriate brief with IDs for drill-down; no raw customer histories or payment payloads |
| Payment/pricing/deposit | Reservation/payment/deposit IDs, approved policy snapshot, semantic payment status and references from trusted adapters | Amount/status/due/refundability only from trusted records; no raw card/bank/token/webhook/provider JSON |
| SOP/policy assistant | Approved SOP/policy versions, location scope, staff role/context | Cited policy answer or escalation; no customer/pet facts unless the question is tied to a specific approved case |

Fetch controls:

- Every fetch must record `event_id`, worker/agent identity, actor/initiator, tenant/location, subject IDs, fields/categories returned, reason, policy version, and timestamp.
- Repositories must return redacted/domain DTOs, not raw database rows, unless the workflow has explicit approval for raw content.
- Workers should not be able to enumerate tenants, locations, customers, pets, reservations, staff records, payment records, or documents outside the event scope.
- If additional context is needed, the worker must return `NeedsMoreInformation` or request human review rather than broadening access itself.

## Allowlist: context categories that may be sent to the runtime

These categories are allowed only when the workflow's `AllowedAction`, role, tenant/location, and review gate permit them.

### Always-safe operational routing context

- Workflow/event IDs, event type, timestamps, location ID, subject IDs.
- Agent spec name/version, output schema, allowed actions, forbidden actions, and required review gates.
- Policy IDs/versions and short approved policy instructions.
- Source record IDs and citations.
- Prior workflow status and audit references.

### Low-risk operational summaries

- Reservation status, service kind, arrival/departure dates, readiness state, task status, capacity/labor aggregate summaries.
- Missing-info checklist, hard-stop categories, next required staff/manager action.
- Approved policy snippets and SOP excerpts.
- Semantic payment/deposit status such as paid/unpaid/partial/failed/refund-review-needed when sourced from trusted adapters.

### Conditional sensitive context

The following can be sent only as minimal snippets for a specific approved workflow and must carry review gates:

- Customer contact data: only the contact field needed for a draft/send review or identity match; avoid full profiles.
- Pet medical, medication, allergy, feeding, or veterinary data: only facts required for care/vaccine/incident/review workflow; never use for diagnosis.
- Vaccine document/OCR content: only for document extraction or vaccine-review workflows; keep raw images/OCR behind the document tool unless explicitly approved.
- Behavior/temperament/incident facts: only for play eligibility, care routing, incident escalation, or manager/customer review packets.
- Payment amounts/statuses/provider references: only trusted semantic fields, never secrets or raw provider payloads.
- Staff-only notes/internal policy: only approved excerpts necessary for SOP answer, operations brief, task routing, or manager review.

## Denylist: never sent unless explicitly approved

Do not send these to the LLM/runtime by default:

- API keys, OAuth tokens, webhook signing secrets, database credentials, service-account secrets, session cookies, password reset links, signed URLs that expose private files beyond the approved window, or tool credentials.
- Raw card numbers, CVV, bank account/routing numbers, payment tokens, payment secrets, full raw payment-provider payloads, signed webhook payloads, or chargeback/legal packet internals.
- Full customer databases, broad exports, mailing lists, unscoped message inboxes, and raw analytics/session tracking data.
- Full customer contact profiles when an ID or single contact channel is sufficient.
- Unredacted vaccine files, raw OCR dumps, medical documents, vet records, medication labels, or care instructions unless the approved workflow specifically requires document extraction/review.
- Camera/video/media snapshots unless the media workflow and privacy policy explicitly approve the exact media and retention behavior.
- Staff HR/timeclock/payroll/disciplinary records, private staff notes, internal investigation notes, or legal/compliance privileged material.
- Internal business strategy, pricing formulas, negotiated vendor terms, or unpublished policy drafts unless the SOP/policy workflow explicitly needs an approved excerpt.
- Prior AI prompts/responses as authority. Prior model output can be cited as a draft artifact only; it is not a source of truth.

Any exception must be narrow, recorded as an approval event, and tied to a workflow, model/provider, data category, subject scope, retention period, and reviewer.

## Persistent memory policy

Default for this product: do not use open-ended cross-tenant persistent LLM memory for customer, pet, reservation, medical, vaccine, incident, payment, staff, or message data.

Allowed persistent memory categories, if the product enables memory later:

- Tenant/location configuration that is intentionally durable and approved: location name, timezone, service capabilities, policy record IDs/versions, integration mode, notification routing, and non-sensitive workflow preferences.
- Approved SOP/policy knowledge that is already meant to be reused by staff and has a policy version/source.
- Agent/runtime operational preferences that do not identify customers, pets, staff, or private incidents: output format expectations, escalation routing names/roles, audit schema versions, safe test-account markers.
- De-identified product-learning patterns only when aggregated enough that they cannot reveal a person, pet, reservation, incident, payment, or staff record.

Forbidden persistent memory categories:

- Customer names, emails, phone numbers, addresses, portal IDs, conversation history, preferences, complaints, or private account facts.
- Pet names tied to owners, medical conditions, medications, allergies, vaccine status/documents, behavior notes, incident history, feeding/care plans, or stay history.
- Reservation, payment, deposit, refund, discount, waiver, invoice, chargeback, loyalty, membership, or provider-reference details.
- Staff HR, scheduling, performance, disciplinary, payroll, or private note details.
- Any cross-tenant fact unless it is public product documentation or explicitly approved shared policy.

Tenant/privacy boundaries:

- Memory keys and stores must be tenant-scoped and location-scoped where appropriate.
- A worker serving tenant A must not search, summarize, or retrieve tenant B memory.
- Memory writes must include category, source, approver or policy ref, tenant/location, sensitivity class, retention/review date, and deletion path.
- Memory retrieval must be logged like database fetches.
- If a memory candidate includes sensitive operational facts, store a pointer to the authoritative database record instead of the fact itself, or do not store it.

## Handling specific sensitive domains

### Vaccine and medical information

Allowed:

- Use minimal vaccine names/dates/statuses, source refs, and ambiguity flags for vaccine-review workflows.
- Use medication/feeding/allergy/medical condition facts only for care-task planning, review packets, and staff/manager/vet clarification.
- Summarize uncertainty and missing proof.

Not allowed:

- Diagnose, infer treatment, approve uncertain medical documents, create executable medication instructions from vague notes, or hide concerning facts.
- Send raw medical/vaccine documents or full OCR to a general-purpose workflow.
- Persist medical/vaccine facts in LLM memory.

### Incident and behavior details

Allowed:

- Summarize source-grounded incident timelines, involved subject IDs, severity/risk flags, missing fields, and required reviews.
- Use behavior/temperament facts to recommend human review or safe care-lane decisions.

Not allowed:

- Close incidents, blame parties, suppress concerning facts, publish owner/legal/public responses, or make final playgroup eligibility decisions when policy requires staff/manager review.
- Include graphic/unnecessary details in customer drafts or manager briefs.

### Payments and customer billing

Allowed:

- Use trusted semantic statuses: amount due/paid/refunded/waived, due date, deposit status, provider reference present/absent, reconciliation state, refundability window, and policy basis.
- Draft internal reconciliation tasks or customer reminders for human review.

Not allowed:

- Process, retry, void, refund, waive, discount, forfeit, or mark disputes resolved.
- Send payment-sensitive customer messages unless a separately approved deterministic send path fixes facts, recipient, template, and send condition.
- Send raw card/bank/payment-provider data to an LLM.

### Customer contact data and messages

Allowed:

- Include customer first name/preferred channel or a specific email/phone only when required for a draft/review packet or approved send workflow.
- Include minimal, relevant excerpts from customer messages for intake/missing-info handling.

Not allowed:

- Send whole inboxes, historical message threads, unrelated customer profile fields, or broad contact exports.
- Auto-send sensitive health, incident, payment, refund, policy-exception, legal, complaint, or safety messages without the required review gate.

### Staff-only notes and internal policy

Allowed:

- Include approved SOP/policy excerpts and role-appropriate staff task notes needed for the workflow.
- Use staff-only notes to route internal review packets when policy permits.

Not allowed:

- Expose staff-only notes to customer-facing drafts unless the content has been intentionally transformed into approved customer-safe language.
- Send HR/payroll/disciplinary or privileged/legal material to general workflows.

## Logging, redaction, and audit

Runtime audit must answer: who/what invoked the worker, what data categories were sent or fetched, why the data was allowed, what model/provider/runtime was used, what output was produced, what review gate applied, and what human or deterministic step turned a draft into an action.

Required audit records for each worker invocation:

- Invocation ID, workflow/event ID, tenant/location, actor/initiator, agent/workflow name and version, model/provider/runtime, tool permission set, and timestamp.
- Prompt packet manifest: field/category list, subject IDs, source refs, policy/review refs, token/count estimates if available, and redaction profile. Do not store raw sensitive prompt text by default.
- Data fetch manifest: repository/tool name, record IDs, fields/categories returned, reason, policy version, redaction profile, and timestamp.
- Output manifest: structured schema name, workflow status, recommended action categories, risk flags, human review reason, citations/source IDs, and validation result.
- Side-effect handoff: approval event ID, reviewer/actor, final action/tool path, before/after state refs, provider refs, and rollback/escalation notes when applicable.
- Error/safe-failure records for policy denials, redaction failures, validation failures, provider errors, and model refusals/malformed output.

Redaction expectations:

- Logs should store manifests and hashes/refs instead of raw prompt/completion text for sensitive workflows.
- If raw prompt/output retention is required for debugging, it must be disabled by default, time-limited, access-controlled, tenant-scoped, and separately approved.
- Redact or omit customer contact data, pet medical/vaccine/care details, incident specifics, payment references, provider payloads, staff-only notes, and media refs unless the audit role is authorized to see them.
- Sensitive debug output in code should remain redacted, consistent with the domain's existing redaction expectations for care/temperament notes.

Audit expectations:

- Every AI suggestion, draft, task recommendation, risk flag, policy denial, approval, outbound message, provider action, and state mutation must be traceable to source records, policy snapshot/version, actor, timestamp, and review gate.
- AI output is not source truth. The audit trail must distinguish source records, deterministic policy decisions, human approvals, model drafts, and external tool outcomes.
- Audit records must be immutable/append-only for decisions and side effects; correction records should supersede rather than rewrite history.
- Access to audit logs must respect tenant/location/role boundaries.

## Data sent to LLM/runtime approval checklist

Before enabling a production worker, approve and record:

1. Workflow/event types covered.
2. Agent spec/version and output schema.
3. Runtime/provider/model, retention settings, and region/security posture.
4. Direct prompt-packet fields.
5. Fetch-by-ID repositories/tools and exact field/category allowlist.
6. Denylist enforcement and redaction profile.
7. Persistent memory setting: disabled, or approved categories/tenant boundaries.
8. Required review gates and forbidden actions.
9. Tool permissions and side-effect boundaries.
10. Audit manifests, raw prompt/output retention rule, and reviewer access.
11. Test cases for sensitive domains: vaccine/medical, incident/behavior, payment, customer contact, staff-only notes, and internal policy.
12. Rollback/safe-failure behavior when context is missing, ambiguous, conflicting, over-broad, or denied.

## Implementation implications

- Add a context-builder layer before any Hermes/LLM invocation. It should produce a typed `AgentPromptPacket` plus a prompt-data manifest, not ad hoc prompt strings.
- Add repository DTOs that are already minimized/redacted for AI use instead of passing database rows to prompt templates.
- Treat `AllowedAction`, `AutomationLevel`, and `ReviewGate` as hard gates for context assembly and tool permissions.
- Validate `WorkflowResult` outputs before persistence or side effects. Invalid output should fail safely and create an audit/error record.
- Keep prompt/context policy versioned so every invocation can cite the policy used at the time.
- Use synthetic fixtures and approved test accounts for live testing until the production data-sent-to-runtime gate is approved.
