---
title: "Review gates, blocked actions, and human approval boundaries"
slug: "review-safety-boundaries"
family: "review-gates-and-blocked-actions"
status: "draft"
audience: ["front-desk", "general-manager", "regional-ops", "compliance", "docs-writer"]
plain_english_definition: "The safety vocabulary that says what automation may read, draft, route, recommend, or record, and what must stop for staff, manager, or system-of-record approval."
primary_labor_problem: "Reduces repeated manual checking while preventing source-backed drafts from being mistaken for executed resort actions."
source_of_record: "domain policy/workflow contracts, app workflow packets, API contract tests, and staff or manager approval evidence"
authoritative_human_role: "general manager, front desk lead, trained reviewer, or approved sender depending on the gate"
workflow_links: ["manager-daily-brief", "booking-triage", "data-quality-hygiene", "daily-updates", "retention", "checkout-completion"]
source_paths:
  - "domain/src/policy.rs"
  - "domain/src/workflow.rs"
  - "domain/src/message.rs"
  - "domain/src/agent.rs"
  - "app/src/agents.rs"
  - "app/src/booking_triage.rs"
  - "app/src/manager_daily_brief.rs"
  - "app/src/data_quality_hygiene.rs"
  - "apps/api/tests/manager_daily_brief_agent_context_contract.rs"
  - "apps/api/tests/manager_daily_brief_outcome_capture_contract.rs"
  - "apps/api/tests/data_quality_hygiene_agent_contract.rs"
rustdoc_contracts:
  - "domain::policy::{ReviewGate, automation::Level, automation::Rule}"
  - "domain::workflow::{PolicyContext, AllowedAction, Result, Status, RecommendedAction}"
  - "domain::message::{Direction, Channel, Status, BodyRef}"
  - "domain::agent::Spec"
  - "app::agents::{AgentSpec, WorkflowAgent, AgentPromptPacket, baseline_agent_specs}"
  - "workflow-local SafeAgentAction and BlockedAction enums"
glossary_links:
  - "../glossary-workflow-state-terms.md#review-gate"
  - "../glossary-workflow-state-terms.md#blocked-action"
  - "../glossary-workflow-state-terms.md#draft"
  - "../glossary-workflow-state-terms.md#outcome-capture"
allowed_action_summary: "read source-backed evidence, prepare drafts, rank/recommend internal work, request named review gates, validate output, and record reviewed outcome evidence"
blocked_action_summary: "no autonomous customer sends, provider/PMS writes, schedule/capacity changes, payment/refund/discount movement, source hiding, medical/safety/policy approvals, or live side effects without an approved deterministic path"
outcome_fields: ["context packet id", "correlation id", "source refs", "issue refs", "review gates", "approval status", "reviewer/actor", "requested side effects", "blocked action reasons", "outcome disposition", "actual minutes", "live_side_effects_allowed"]
---

# Review gates, blocked actions, and human approval boundaries

This family page is an atlas entry for the safety entities that appear together across NVA pet-resort workflows: review gates, automation policy, allowed actions, blocked actions, message drafts, approval records, agent specs, prompt packets, and outcome evidence. It is written for non-coders who need to understand whether a workflow is read-only, draft-only, manager-reviewed, or never allowed to execute directly.

Markdown is only the orientation layer. The behavioral authority remains the linked source files, app/API tests, and later approved production policy.

## 1. Plain-English pet-resort definition

Review safety boundaries are the visible stop signs and permission labels around automation. They let the system reduce staff labor by reading source facts, preparing drafts, ranking work, and recording outcomes, while keeping live resort actions with the right staff member, manager, approved sender, or source-of-record system.

In plain language: a workflow packet can say “here is the source-backed recommendation and here is the draft,” but it must also say “do not send, change, approve, hide, refund, schedule, or write back unless this named gate is cleared.”

## 2. Purpose: labor-cost or safety problem

This page helps front desk leads, managers, compliance reviewers, and docs writers avoid two expensive mistakes:

- underusing automation by forcing staff to reassemble every fact manually; and
- overtrusting automation by treating a source-backed draft as if it already changed Gingr/PMS, contacted a pet parent, moved money, or approved a pet-safety decision.

The safe outcome is a reviewable queue: evidence is collected, the draft or recommendation is prepared, the blocked action is named, and the human/system approval record or outcome evidence proves what actually happened.

## 3. Workflows where these entities appear

| Workflow | How the boundary appears | Safe workflow result |
| --- | --- | --- |
| Manager Daily Brief | Actions carry source facts, safe agent actions, required review gates, blocked actions, and outcome capture. | Ranked manager/front-desk queue plus labor evidence; no schedule, payment, provider, or customer-message side effect. |
| Booking Triage | Deterministic readiness packets can request approval and draft explanations while blocking provider mutation, confirmation, sends, and payment movement. | Staff-ready booking packet, confirmation draft, or review request; not an automatic booking change. |
| Data Quality Hygiene | Context and draft endpoints expose internal cleanup actions and reject side effects or ambiguity hiding. | Source-grounded cleanup task and outcome record; no autonomous provider repair. |
| Daily Updates / Pawgress | Customer-message drafts need review, omissions, internal flags, and send approval. | Customer-safe draft or send stub for review; not a sent message. |
| Retention / Grooming Rebooking | Follow-up opportunities can draft outreach and request `CustomerMessageApproval`. | Reviewable follow-up draft or suppression/outcome evidence; no auto-discount, payment, booking, or send. |
| Checkout Completion | Completion packets can suggest/audit checkout status and retention follow-up while blocking provider mutation. | Staff handoff/audit draft and review queue; not a live checkout write. |

## 4. Relationships and adjacency

```text
upstream source facts and policy snapshots
  -> app workflow context / agent prompt packet
  -> PolicyContext: allowed actions + automation level + required review gates
  -> draft, recommendation, review queue item, or internal task
  -> blocked-action list and validation result
  -> human approval / rejection / staff action
  -> outcome record, audit event, source refs, labor evidence
  -> downstream customer/staff message only through an approved send/action path
```

Key relationships:

- Source facts and provenance explain why a recommendation exists; they are evidence, not approval.
- Review gates say which human or deterministic policy stop remains before the sensitive step.
- Allowed actions say what the workflow may safely prepare or record.
- Blocked actions say what must not be executed directly by the agent/app workflow.
- Message drafts are downstream artifacts; they must preserve recipient/channel/status/approval context before any send path.
- Outcome records and audit events prove what staff accepted, rejected, saved, or blocked.

## 5. Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| Domain policy | [`domain/src/policy.rs`](../../domain/src/policy.rs) | `ReviewGate` variants, `automation::Level`, `automation::Rule`, denial reasons, and play-safety gates. |
| Domain workflow | [`domain/src/workflow.rs`](../../domain/src/workflow.rs) | `PolicyContext`, `AllowedAction`, workflow `Result`, `Status`, `RecommendedAction::RequestHumanReview`, risk flags, and verification notes. |
| Domain message | [`domain/src/message.rs`](../../domain/src/message.rs) | direction, channel, lifecycle status, and body reference for customer/staff/internal messages. |
| Domain agent spec values | [`domain/src/agent.rs`](../../domain/src/agent.rs) | `Spec` fields: name, purpose, allowed tools, forbidden actions, default review gates. |
| App agent contract | [`app/src/agents.rs`](../../app/src/agents.rs) | `AgentSpec`, `WorkflowAgent`, `AgentPromptPacket`, baseline allowed tools/forbidden actions/review gates. |
| Source-evidence map | [`docs/safety/source-evidence-map.md`](../safety/source-evidence-map.md) | citation cautions and safety claims for non-coder docs. |
| Permission matrix | [`docs/architecture/agent-permissions-by-workflow.md`](../architecture/agent-permissions-by-workflow.md) | draft read/write envelope, customer-message levels, never-direct changes, and open approval gates. |
| Manager brief API tests | [`apps/api/tests/manager_daily_brief_agent_context_contract.rs`](../../apps/api/tests/manager_daily_brief_agent_context_contract.rs), [`apps/api/tests/manager_daily_brief_outcome_capture_contract.rs`](../../apps/api/tests/manager_daily_brief_outcome_capture_contract.rs) | context is source-grounded; outcomes persist labor evidence; blocked/unknown side effects are rejected. |
| Data hygiene API tests | [`apps/api/tests/data_quality_hygiene_agent_contract.rs`](../../apps/api/tests/data_quality_hygiene_agent_contract.rs) | drafts reject blocked side effects and ambiguity hiding; outcomes record labor evidence without provider writes. |
| Workflow glossary | [`docs/glossary-workflow-state-terms.md`](../glossary-workflow-state-terms.md) | non-coder wording for draft, review gate, blocked action, workflow packet, agent spec, and outcome capture. |

If rendered Rustdoc is not present, cite source paths plus module/type paths rather than inventing URLs.

## 6. Authoritative source system or human role

| Fact or decision | Source of record | Human/system role when incomplete or sensitive |
| --- | --- | --- |
| Review gate vocabulary | `domain::policy::ReviewGate` | Product/security/compliance approval before widening production authority. |
| Automation level | `domain::policy::automation::Level` and workflow policy config | Manager/security review for changes from draft-only or internal-task-only to any live path. |
| Workflow allowed actions | `domain::workflow::PolicyContext.allowed_actions` and workflow-local safe action enums | App validator rejects disallowed output; staff reviews the resulting packet. |
| Blocked actions | Workflow-local `BlockedAction` enums and API validation | Staff/system-of-record executes separately if appropriate; the agent does not. |
| Message state | `domain::message::{Direction, Channel, Status, BodyRef}` plus app messaging tools | Approved sender or deterministic approved send service. |
| Approval decision | Approval record, workflow/audit event, or domain-specific approval state | Manager, medical/document reviewer, behavior reviewer, front desk lead, or approved sender. |
| Outcome/labor evidence | App/API outcome record and storage projection | Reviewer/actor who performed the work and recorded actual minutes/disposition. |

## 7. Allowed actions

Automation and app workflows may safely do the following when the linked contract allows it and the packet is scoped to the right location, subject, source facts, and policy snapshot:

- read source-backed customer, pet, reservation, document, care, labor, or message evidence;
- extract structured data and preserve uncertainty;
- draft customer messages, staff tasks, audit events, and status suggestions for review;
- summarize source evidence and care notes;
- rank manager/front-desk work queues;
- flag risk, missing facts, stale data, duplicates, and policy stops;
- request a named review gate;
- validate agent output against allowed actions and blocked actions;
- record staff disposition, actual minutes, source refs, issue refs, and audit/correlation IDs after review.

Use “prepare,” “draft,” “suggest,” “route,” “rank,” “flag,” and “record reviewed outcome.” Avoid “send,” “approve,” “apply,” “write,” “change,” “confirm,” or “resolve” unless the source contract and approval evidence actually show that live action occurred.

## 8. Blocked actions and review gates

Default blocked actions unless a linked contract explicitly approves a deterministic path:

- customer/member/public sends;
- Gingr/PMS/provider writes, source-data deletion, source hiding, or material audit editing;
- booking confirmation/rejection/cancellation, check-in/out, waitlist release, room/capacity/schedule changes, or care-task completion;
- payment capture/retry/void/refund, discount, deposit waiver/forfeit, credit, write-off, rate/tax/fee changes;
- vaccine, medical, behavior, group-play, incident, legal, safety, or policy approvals;
- staff schedule, payroll, timeclock, personnel, or final labor decisions;
- secret-dependent or live external side effects.

The current named review gates in `domain::policy::ReviewGate` are:

| Gate | Plain-English meaning | Typical owner |
| --- | --- | --- |
| `ManagerApproval` | Manager must approve an operations, capacity, staffing, policy, incident, or exception-sensitive step. | General manager or authorized lead. |
| `MedicalDocumentReview` | A trained reviewer must verify vaccine/medical document evidence before it changes readiness. | Trained document/vaccine reviewer. |
| `BehaviorReview` | Pet behavior/group-play eligibility needs review before care or play decisions. | Trained staff, daycare lead, or manager. |
| `CustomerMessageApproval` | A person or approved deterministic send path must approve final recipient/channel/body before contact. | Front desk lead, manager, or approved sender. |
| `RefundOrDepositException` | Money movement or deposit/refund exception must be reviewed. | Manager or authorized billing/payment role. |

## 9. Safe-use evidence and outcome fields

A safe recommendation should carry enough evidence for a reviewer to prove both what the system did and what it did not do. Look for these fields in context packets, draft submissions, approval records, audit rows, and outcome capture:

| Evidence field | Why it matters |
| --- | --- |
| `context_packet_id` and `correlation_id` | Connects a draft/outcome back to the exact packet or workflow run. |
| `source_refs`, provenance, issue refs, source snapshot IDs | Proves which upstream facts supported the recommendation. |
| `review_gates` / `required_reviews` | Shows the named human approval stop. |
| `allowed_actions` / `safe_agent_actions` | Shows what the workflow was allowed to prepare. |
| `blocked_actions` and validation reasons | Shows what was not allowed and why a request was rejected. |
| `requested_side_effects` | Lets validators reject attempted sends/writes/payments/schedule changes. |
| `live_side_effects_allowed: false` | Explicit proof that the response is not a live action path. |
| approval status, reviewer/actor/persona, timestamp | Shows who accepted/rejected/recorded the action. |
| outcome disposition, feedback, actual minutes, reporting group | Converts a recommendation into measurable reviewed work. |

API tests currently prove important negative boundaries: manager-brief outcome capture rejects blocked side effects such as `send_customer_message`, `mutate_provider_or_pms_record`, `change_staff_schedule`, `move_refund_discount_or_payment`, and `hide_source_data_quality_issue`; it also fails closed on unknown side effects. Data-quality hygiene draft validation rejects blocked side effects and attempted ambiguity hiding, and its outcome capture records labor evidence while keeping `live_side_effects_allowed` false.

## 10. Family entries

### Review gate

A review gate is a named human-approval stop. It does not mean the action is approved; it means the packet has reached a point where the right human or approved system must decide before a sensitive step can continue.

- Source/Rustdoc: `domain::policy::ReviewGate`; `domain::workflow::PolicyContext.required_reviews`; `domain::workflow::RecommendedAction::RequestHumanReview`.
- Allowed: carry the gate in context, draft/recommend around it, route to the review queue, and record the eventual disposition.
- Blocked: do not treat the gate text itself as approval; do not bypass local policy or production permission review.
- Evidence: gate name, reason/rationale, source refs, reviewer identity/role, status, timestamp, and audit event.

### Policy rule / automation level

A policy rule says how much authority a workflow has: `SafeToAutomate`, `DraftOnly`, `InternalTaskOnly`, `ManagerApprovalRequired`, or `NeverAutomate`.

- Source/Rustdoc: `domain::policy::automation::{Rule, Level, Rationale}` and `domain::policy::WorkflowName`.
- Allowed: classify the workflow, explain the rationale, and drive validation/redaction.
- Blocked: do not upgrade a draft-only or never-automate workflow in docs unless source and approval evidence changed.
- Evidence: workflow name, level, rationale, approval/change record, and test coverage for the new boundary.

### Allowed action / safe agent action

Allowed actions and workflow-local safe agent actions are the positive verbs automation may perform, such as read entities, extract structured data, draft a customer message, create an internal task, suggest status, summarize care notes, flag risk, rank manager actions, or record feedback.

- Source/Rustdoc: `domain::workflow::AllowedAction`; workflow-local `SafeAgentAction` enums in app modules.
- Allowed: prepare reviewable artifacts and outcome evidence.
- Blocked: do not infer that a “suggest” or “draft” action executed the underlying operational change.
- Evidence: allowed action list, validator result, output schema name, draft/task ID, and downstream outcome.

### Blocked action

A blocked action is an explicit no-go item for automation. It often names the exact live action a staff member or source-of-record system may need to perform later.

- Source/Rustdoc: workflow-local `BlockedAction` enums; `domain::policy::denial::Reason`; API validation tests.
- Allowed: surface the blocked action to reviewers, reject side-effect requests, and record why work stopped.
- Blocked: do not hide source issues, mutate provider records, send customer messages, change schedules, move money, or approve safety/policy exceptions.
- Evidence: blocked action code, validation status, reasons, `outcome_persisted: false` on rejection, and `live_side_effects_allowed: false` where applicable.

### Draft / message

A draft is staff-review work product. A message is the customer/staff/internal communication entity with direction, channel, status, and body reference; it is not just generated prose. Use the core family’s explicit [Message and message state](entity-atlas-petsuites-core-entities.md#message-and-message-state) entry for operator-facing definitions, relationships, contracts, and safe-use evidence; use this safety page to keep final sends, queueing, suppression, and sensitive disclosures behind review gates.

- Source/Rustdoc: `domain::entities::Message`; `domain::message::{Direction, Channel, Status, BodyRef}`; app daily-update/CRM/tool messaging draft contracts; `app::tools::messaging::*`.
- Allowed: draft body text, recipient/channel suggestion, included/omitted facts, suppression or blocked-send reason, and fact-check checklist for review.
- Blocked: do not send, queue, suppress, alter consent/contact policy, disclose sensitive care/incident/payment facts, or edit message history without an approved path and approval evidence.
- Evidence: draft ID/body ref, recipient/channel policy, contact preference/consent, status, approval gate/record, reviewer, send stub or blocked reason, included/omitted facts, source/provider refs, follow-up disposition, and send/suppression audit event.

### Approval record / human approval

An approval record is durable evidence that a human or approved process accepted, rejected, or escalated a target such as a document, message, or action. It is the bridge between a review gate and an allowed downstream execution path.

- Source/Rustdoc: `domain::entities::approval::{Record, Target, Lifecycle, Status}` where present; app/API approval tests; domain-specific approval states.
- Allowed: record reviewer decision, lifecycle/status, target, reason, source refs, and audit event.
- Blocked: do not assume approval from confidence, urgency, model output, or the presence of a draft.
- Evidence: approver role/id, target, status, timestamp, gate, decision reason, source refs, and audit lineage.

### Agent spec / prompt packet

An agent spec is the job description and rule sheet for a bounded automation helper. An agent prompt packet is the evidence bundle sent to an agent; it carries workflow identity, source event, typed input, policies, and output schema.

- Source/Rustdoc: `domain::agent::Spec`; `app::agents::{AgentSpec, WorkflowAgent, AgentPromptPacket, baseline_agent_specs}`.
- Allowed: expose narrow read/draft tools, include forbidden actions, include default review gates, and validate structured output.
- Blocked: do not treat an agent spec or prompt as production approval, live write access, or customer-message send authority.
- Evidence: workflow name, allowed tools, forbidden actions, default gates, event, input, policies, output schema, validation result.

### Outcome capture / audit evidence

Outcome capture records what staff did with a recommendation and whether labor was actually saved. Audit evidence records why the recommendation existed and what was blocked, accepted, or rejected.

- Source/Rustdoc: `domain::workflow::Result`; workflow outcome records; storage operation projections; API outcome capture tests.
- Allowed: record disposition, actual minutes, actor/persona, source refs, issue refs, feedback, and correlation ID.
- Blocked: do not claim realized labor savings without an outcome record; do not persist outcomes for unknown action IDs or blocked side-effect requests.
- Evidence: outcome record ID/action ID, actual minutes, estimated-vs-actual comparison, actor, timestamp, correlation ID, and blocked side-effect list.

## 11. Examples and non-examples

| Type | Item | Why |
| --- | --- | --- |
| Example | “Manager Daily Brief may rank a checkout exception and record manager feedback.” | It names a safe queue/result and keeps execution with staff. |
| Example | “Customer-message draft requires `CustomerMessageApproval` before sending.” | It separates drafting from downstream send authority. |
| Example | “Data-quality hygiene rejected a draft because it requested `send_customer_message` and hid ambiguity.” | It proves blocked actions and ambiguity are visible, not silently executed. |
| Non-example | “The agent approved the refund because it was confident.” | Confidence is not approval evidence and refunds are review-gated. |
| Non-example | “The prompt packet updated Gingr.” | Prompt packets are draft/evidence bundles, not provider writes. |
| Non-example | “Source refs prove the customer should be contacted.” | Source refs prove traceability, not message approval or contact consent. |
| Non-example | “Outcome record exists, so the blocked action happened.” | Outcome records can prove reviewed labor and also prove live side effects were not allowed. |

## 12. Glossary cross-links

Use these public glossary terms when copying this page into workflow docs:

- [draft](../glossary-workflow-state-terms.md#draft): staff-review artifact, not a sent/applied action.
- [review gate](../glossary-workflow-state-terms.md#review-gate): named human approval stop.
- [blocked action](../glossary-workflow-state-terms.md#blocked-action): action automation must not perform directly.
- [workflow packet](../glossary-workflow-state-terms.md#workflow-packet): reviewable bundle of source-grounded facts, actions, drafts, and evidence.
- [outcome capture](../glossary-workflow-state-terms.md#outcome-capture): recorded reviewed result, not proof from intent alone.
- [source-of-record](../glossary-source-data-terms.md#source-of-record), [provider record](../glossary-source-data-terms.md#provider-record), [provenance](../glossary-source-data-terms.md#domainsourceprovenance-and-domainsourcerecordref-as-data-evidence), and [data-quality issue](../glossary-source-data-terms.md#domaindata_qualityissue-data-quality-issue): evidence and source-trust terms that must stay separate from approval.

## Writer checklist

Before publishing or reusing this family page, verify:

- Does every customer, provider, payment, schedule, staff, medical, behavior, incident, or policy-sensitive action have a named blocked action or review gate?
- Does the page say “draft/recommend/route/record” instead of implying execution?
- Does every labor-savings claim point to outcome fields rather than intention?
- Are source paths current? The source-evidence map notes that older architecture drafts use stale names such as `domain/src/agents.rs`, `domain/src/tools.rs`, and `AutomationLevel`; current citations should use `app/src/agents.rs`, `app/src/tools.rs`, `domain/src/agent.rs`, and `domain::policy::automation::Level` unless the code changes.
- Is there evidence that a blocked action was not executed unsafely, such as validation rejection, `live_side_effects_allowed: false`, `outcome_persisted: false`, no requested side effects, or an approval/audit record?
