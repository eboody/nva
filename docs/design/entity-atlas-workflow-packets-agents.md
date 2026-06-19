---
title: "Workflow packets, agents, drafts, and review queues"
slug: "workflow-packets-agents-drafts-review-queues"
family: "workflow-packets-and-agents"
status: "draft"
audience: ["front-desk", "general-manager", "regional-ops", "docs-writer", "engineer"]
plain_english_definition: "The review packets, agent specs, prompt bundles, drafts, outcomes, and source-backed queues that let pet-resort teams reduce manual triage without giving automation live authority."
primary_labor_problem: "Reduces repeated resort-dashboard reconciliation, draft writing, review routing, and outcome measurement across manager daily brief, booking triage, data-quality hygiene, checkout completion, CRM retention/rebooking, and daily updates."
source_of_record: "Typed app/domain workflow contracts plus source evidence; humans and systems of record remain authoritative for live resort actions."
authoritative_human_role: "front desk lead, general manager, care/medical reviewer, approved customer-message sender, or regional operations reviewer depending on the gate"
workflow_links: ["manager-daily-brief", "booking-triage", "data-quality-hygiene", "checkout-completion", "crm-retention-rebooking", "daily-updates"]
source_paths:
  - "app/src/agents.rs"
  - "app/src/manager_daily_brief.rs"
  - "app/src/booking_triage.rs"
  - "app/src/data_quality_hygiene.rs"
  - "app/src/checkout_completion.rs"
  - "app/src/crm_retention.rs"
  - "app/src/daily_update.rs"
  - "domain/src/agent.rs"
  - "domain/src/workflow.rs"
  - "domain/src/daily_brief.rs"
  - "domain/src/operations.rs"
rustdoc_contracts:
  - "app::agents::{AgentPromptPacket, AgentSpec, WorkflowAgent}"
  - "domain::agent::Spec"
  - "domain::workflow::{Event, Result, AllowedAction, RecommendedAction, Status}"
  - "app::manager_daily_brief::{Packet, BriefAction, OutcomeRecord}"
  - "app::booking_triage::{StaffEvaluationPacket, ConfirmationDraft, AuditEventDraft}"
  - "app::data_quality_hygiene::{Packet, Candidate, Action, DraftSubmission, OutcomeRecord}"
  - "app::checkout_completion::{Packet, StaffHandoff, AuditEventDraft}"
  - "app::crm_retention::{Packet, StaffReviewPacket, OutcomeRecord}"
  - "app::daily_update::{MvpPreview, CustomerMessageDraft, SendStub}"
glossary_links:
  - "../glossary-workflow-state-terms.md#workflow-packet"
  - "../glossary-workflow-state-terms.md#draft"
  - "../glossary-workflow-state-terms.md#review-gate"
  - "../glossary-workflow-state-terms.md#blocked-action"
  - "../glossary-workflow-state-terms.md#outcome-capture"
allowed_action_summary: "read source-backed evidence, validate deterministic gates, rank and summarize queues, draft internal/customer text for review, create audit drafts, and record reviewed outcomes"
blocked_action_summary: "no autonomous customer sends, provider/PMS writes, booking/status/schedule/capacity changes, payment/refund/discount movement, policy/safety approvals, source-data hiding, or sensitive-payload exposure"
outcome_fields: ["source refs", "review gates", "review disposition", "blocked actions", "feedback outcome", "actual labor minutes", "audit events", "correlation id"]
---

# Workflow packets, agents, drafts, and review queues

Purpose: this family entry turns the workflow-packet and agent contracts from the app/domain source into pet-resort language. It follows the [entity atlas page template](entity-atlas-page-template.md) and expands the [workflow-packets-and-agents inventory row](entity-atlas-inventory.md#family-workflow-packets-and-agents). Markdown explains the concept; the linked Rust source, module paths, and tests remain the behavioral authority.

## 1. Plain-English pet-resort definition

Workflow packets are the packets of source-backed facts, review gates, recommended safe actions, drafts, blocked actions, and outcome evidence that a resort team reviews before work changes a customer, pet, reservation, source record, schedule, payment, or customer message.

Agents are narrow helpers that may summarize, rank, validate, or draft from those packets. They are not free-form chatbots and they do not own live resort authority. A review queue is the staff-visible list produced from a packet: the system can tell a front-desk lead or manager what needs attention, why it is safe to review, and what remains blocked until a human or system of record acts.

## 2. Purpose: labor-cost or safety problem

This family exists because the labor-saving product needs to prove two things at the same time:

- Pet-resort staff should not manually reconcile the same source facts across PMS screens, care notes, checkout handoffs, customer history, data-quality exceptions, and labor reports.
- Automation must not become the hidden decision maker for customer sends, reservation changes, payment movement, medical/care approvals, schedule changes, provider writes, or data cleanup.

The packet pattern creates a safe middle layer: source facts and policy gates become reviewable drafts, queues, audit drafts, and outcomes. The outcome records then show whether the reviewed action actually saved manager, front-desk, care-team, reconciliation, or regional-ops time.

## 3. Workflows where it appears

| Workflow | Packet, draft, or queue | Upstream facts | Downstream draft, queue, or outcome |
| --- | --- | --- | --- |
| [Manager Daily Brief](../workflows/operator/manager-daily-brief.md) | `app::manager_daily_brief::Packet`, `BriefAction`, `SourceFact`, `OutcomeRecord` | demand facts, checkout packets, retention packets, labor snapshots, data-quality issues, source refs, policy gates | ranked manager actions, internal task drafts, source-quality flags, labor-minute estimates, manager feedback/outcome records |
| [Booking Triage](../workflows/operator/booking-triage.md) | `StaffEvaluationPacket`, deterministic `ReadinessBucket`, `ConfirmationDraft`, `AuditEventDraft` | reservation, pet profile completeness, policy snapshot, deposit and hard-stop evidence, source refs | staff approval queue, missing-info/customer-safe script draft, review-gate request, failed-safe audit draft |
| [Data Quality Hygiene](../workflows/operator/data-quality-hygiene.md) | `Packet`, `Candidate`, `Action`, `DraftSubmission`, `OutcomeRecord` | data-quality issues, duplicate/profile-gap/service-line/source-freshness candidates, source refs, issue refs, sensitivity flags | cleanup/reconciliation queue, internal cleanup task draft, draft validation result, cleanup outcome and labor minutes |
| [Checkout Completion](../workflows/operator/checkout-completion.md) | `Packet`, `StaffHandoff`, `AuditEventDraft` | source reservation status, staff handoff, care summary, belongings return, departure-note review | checkout evidence summary, internal handoff task, retention follow-up draft for review, audit-event drafts |
| [Grooming Rebooking / Retention](../workflows/operator/grooming-rebooking-retention.md) | `crm_retention::Packet`, `StaffReviewPacket`, `RetentionOpportunity`, `OutcomeRecord` | completed stay/service evidence, consent and channel permission, source-grounded reasons, checkout proof | staff retention review queue, customer follow-up draft for approval, outcome evidence such as booked, no response, suppressed, or wrong source |
| [Daily Updates / Pawgress Drafts](../workflows/operator/daily-updates-pawgress-drafts.md) | `MvpPreview`, `AgentPromptPacket<daily_care_update::Input>`, `CustomerMessageDraft`, `SendStub` | staff care notes, workflow event, source pet/customer context, allowed action and policy instructions | customer message draft, approval record, send stub blocked by review gate, audit log, included/omitted fact list |

## 4. Relationships and adjacency

```text
Provider or staff source evidence
  -> source refs / provenance / data-quality issues
  -> domain entities and workflow events
  -> app workflow request
  -> packet with candidates, actions, review gates, safe actions, and blocked actions
  -> optional agent prompt packet for summary/ranking/draft help
  -> reviewed draft, internal task, audit-event draft, or review queue item
  -> human/system-of-record action outside the agent boundary
  -> outcome record with source refs, disposition, blocked-action proof, and labor measurement
```

Cross-workflow relationship notes:

1. Manager daily brief is the aggregator. It can include scoped checkout and retention packets, source data-quality issues, labor snapshots, and domain daily-brief facts, then turn them into ranked manager actions and outcome records.
2. Booking triage, checkout completion, CRM retention, daily updates, and data-quality hygiene all preserve source facts before drafting or ranking. They should feed the manager brief as source-grounded action candidates, not as already-approved live changes.
3. Data-quality hygiene is a guardrail for every other workflow. It routes missing, stale, duplicate, conflicting, ambiguous, or sensitive facts into cleanup queues before those facts become unsafe customer-facing or provider-facing actions.
4. Checkout completion can feed retention/rebooking. A verified departure and staff handoff can become retention follow-up evidence, but customer outreach remains a draft/review queue until approved.
5. Daily updates turn staff care notes into customer-message drafts, approval records, and send stubs. They do not send the message by themselves.
6. Outcome records and labor-minute fields connect the review queue back to measurable labor reduction. A packet can estimate saved time; a reviewed outcome records what actually happened.

## 5. Contracts and source/Rustdoc links

Rendered Rustdoc may not be present in a local checkout, so this page names Rustdoc/module paths and links source/test files.

| Contract type | Link or path | Rustdoc/module path | What the writer should verify |
| --- | --- | --- | --- |
| Agent spec domain contract | [`domain/src/agent.rs`](../../domain/src/agent.rs) | `domain::agent::Spec` | allowed tools, forbidden actions, default review gates, purpose, output schema name |
| App agent boundary | [`app/src/agents.rs`](../../app/src/agents.rs) | `app::agents::{AgentSpec, WorkflowAgent, AgentPromptPacket}` | prompt packet includes source event, typed input, policy instructions, allowed/forbidden actions, review gates, expected output schema |
| Workflow event/result envelope | [`domain/src/workflow.rs`](../../domain/src/workflow.rs) | `domain::workflow::{Event, Result, AllowedAction, RecommendedAction, Status}` | event subjects, policy context, allowed action names, recommended action kinds, review status |
| Manager daily brief packet | [`app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs) | `app::manager_daily_brief::{Request, Packet, BriefAction, SourceFact, OutcomeRecord}` | source facts, scoped checkout/retention packets, review gates, labor estimates, blocked side effects, outcome evidence |
| Booking triage packet | [`app/src/booking_triage.rs`](../../app/src/booking_triage.rs) | `app::booking_triage::{Request, DeterministicResult, StaffEvaluationPacket, ConfirmationDraft, AuditEventDraft}` | readiness bucket, rule evaluations, approval gates, safe draft rules, blocked booking/provider/message/payment actions |
| Data-quality hygiene packet | [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs) | `app::data_quality_hygiene::{Request, Packet, Candidate, Action, DraftSubmission, DraftValidation, OutcomeRecord}` | source/issue refs, sensitivity, draft validation, cleanup outcomes, no hiding or auto-resolving ambiguity |
| Checkout completion packet | [`app/src/checkout_completion.rs`](../../app/src/checkout_completion.rs) | `app::checkout_completion::{Request, Packet, StaffHandoff, CompletionStatus, AuditEventDraft}` | source status versus staff handoff, safe closeout queue, audit drafts, blocked PMS/payment/customer actions |
| CRM retention packet | [`app/src/crm_retention.rs`](../../app/src/crm_retention.rs) | `app::crm_retention::{Request, StaffReviewPacket, Packet, RetentionOpportunity, ContactPermission, OutcomeRecord}` | consent/channel permissions, source-grounded opportunity evidence, follow-up eligibility, outcome capture |
| Daily update draft packet | [`app/src/daily_update.rs`](../../app/src/daily_update.rs) | `app::daily_update::{MvpPreview, CustomerMessageDraft, SendStub}` and `app::daily_update::daily_care_update::{Input, Output, Agent}` | included/omitted facts, internal flags, customer-message draft, approval record, blocked send stub |
| Daily brief domain view | [`domain/src/daily_brief.rs`](../../domain/src/daily_brief.rs) | `domain::daily_brief::{Resort, ResortOperatingDay, Section, Action, Risk}` | manager-facing operating-day facts: occupancy, labor, arrivals/departures, follow-up queues, safety watches, revenue opportunities |
| Operations domain vocabulary | [`domain/src/operations.rs`](../../domain/src/operations.rs) | `domain::operations::{Portfolio, ServiceOffering, TechnologyEcosystem, AiUseCase, OperatingFunction}` | shared source-of-truth vocabulary for service lines, technology ecosystem, pain areas, data-quality issues, operating functions |
| App contract tests | [`app/tests/booking_triage_mvp.rs`](../../app/tests/booking_triage_mvp.rs), [`app/tests/checkout_completion_workflow_contracts.rs`](../../app/tests/checkout_completion_workflow_contracts.rs), [`app/tests/crm_retention_workflow_contracts.rs`](../../app/tests/crm_retention_workflow_contracts.rs), [`app/tests/daily_care_update_mvp.rs`](../../app/tests/daily_care_update_mvp.rs), [`app/tests/data_quality_hygiene_workflow_contracts.rs`](../../app/tests/data_quality_hygiene_workflow_contracts.rs), [`app/tests/manager_daily_brief_workflow_contracts.rs`](../../app/tests/manager_daily_brief_workflow_contracts.rs) | test modules | executable proof of draft-only boundaries and packet behavior |
| API/storage contract tests | [`apps/api/tests/manager_daily_brief_agent_context_contract.rs`](../../apps/api/tests/manager_daily_brief_agent_context_contract.rs), [`apps/api/tests/manager_daily_brief_agent_drafts_contract.rs`](../../apps/api/tests/manager_daily_brief_agent_drafts_contract.rs), [`apps/api/tests/manager_daily_brief_outcome_capture_contract.rs`](../../apps/api/tests/manager_daily_brief_outcome_capture_contract.rs), [`apps/api/tests/data_quality_hygiene_agent_contract.rs`](../../apps/api/tests/data_quality_hygiene_agent_contract.rs), [`storage/tests/manager_daily_brief_outcome_storage.rs`](../../storage/tests/manager_daily_brief_outcome_storage.rs), [`storage/tests/data_quality_hygiene_outcome_storage.rs`](../../storage/tests/data_quality_hygiene_outcome_storage.rs) | API/storage contract modules | context, draft, outcome, and storage projections for measurable loops |

## 6. Authoritative source system or human role

| Fact or decision | Source of record | Human role when source is incomplete or sensitive |
| --- | --- | --- |
| Agent capability and forbidden actions | `domain::agent::Spec` plus `app::agents` baseline specs | product/engineering owner for the contract; manager for local workflow policy |
| Workflow event, allowed action, result status | `domain::workflow` event/result contract | staff or manager reviewer named by the policy context |
| Manager daily brief action priority | source facts, domain daily brief, app manager-brief packet | general manager or regional ops reviewer |
| Booking readiness | provider reservation evidence, pet/care/vaccine/payment/policy facts, booking triage deterministic result | front desk lead, manager, medical/care reviewer, behavior reviewer, payment/accounting reviewer |
| Data-quality cleanup | `domain::data_quality` issue plus hygiene candidate/action packet | front desk lead, manager, regional ops, or sensitive-data reviewer |
| Checkout completion | source reservation status plus staff handoff and care/departure notes | front desk lead or manager for exceptions |
| Retention/rebooking follow-up | completed stay/service/source evidence plus consent/channel permission | front desk lead, grooming/rebooking owner, manager for credits/complaints/policy-sensitive cases |
| Daily update customer message | staff care notes, customer/pet context, approval record | approved message sender or manager; care/medical reviewer for sensitive care facts |
| Labor savings claim | workflow outcome record and storage projection | reviewer who performed the work, manager/regional ops for reporting |

## 7. Allowed actions

Across this family, automation may:

- read source-backed evidence and preserve source refs;
- validate deterministic gates and required fields;
- summarize evidence for staff, managers, and regional operators;
- rank review queues and recommend internal task priority;
- draft internal tasks, customer-safe scripts, customer-message drafts, and follow-up text for review;
- produce audit-event drafts and approval records;
- estimate labor minutes saved when the app packet supports it;
- record reviewed outcomes, feedback, and actual labor minutes after a human or approved system of record acts.

Workflow-local allowed examples:

| Workflow | Safe actions named in source |
| --- | --- |
| Manager daily brief | summarize source evidence; rank manager actions; draft internal task for review; record manager feedback; estimate labor minutes saved |
| Booking triage | evidence summary; internal task draft; manager packet draft; customer-safe script draft; missing-info request draft |
| Data-quality hygiene | summarize source evidence; rank hygiene actions; draft internal cleanup task; preserve ambiguity for review; estimate reconciliation minutes saved |
| Checkout completion | summarize checkout evidence; create internal handoff task; draft retention follow-up for review |
| CRM retention | summarize retention evidence; create internal staff review task; draft customer follow-up for review; record follow-up outcome evidence |
| Daily update | build a customer-message draft, approval record, blocked send stub, and audit log from staff notes and policy instructions |

## 8. Blocked actions and review gates

The [review gate](../glossary-workflow-state-terms.md#review-gate) and [blocked action](../glossary-workflow-state-terms.md#blocked-action) boundary is the point of the packet pattern. Default blocked actions for this family are:

- no customer/member message send without an approved sender and approval record;
- no provider/PMS/Gingr mutation, status write, record hiding, source-data deletion, or source ambiguity auto-resolution;
- no booking confirmation/rejection, checkout completion, room/capacity/playgroup/schedule assignment, or staff schedule change by agent authority;
- no refund, discount, deposit, charge, payment, package/session-balance, or credit movement;
- no vaccine, medical, care, temperament, incident, behavior, safety, or policy exception approval;
- no sensitive/quarantined payload exposure;
- no suppression of data-quality issues just because a draft looks plausible.

Workflow-local examples from source:

| Workflow | Blocked actions named in source |
| --- | --- |
| Manager daily brief | change staff schedule; mutate provider/PMS record; send customer message; move refund/discount/payment; hide source data-quality issue |
| Booking triage | confirm booking; reject request; accept special care; approve behavior exception; mutate provider record; send customer message; move payment |
| Data-quality hygiene | send customer message; mutate provider/PMS record; change staff schedule; move refund/discount/payment; hide or auto-resolve source ambiguity; expose quarantined sensitive payload |
| Checkout completion | suggest checked-out status without the required agreement; send customer message; mutate provider/PMS record; move refund/discount/payment |
| CRM retention | send customer message; mutate provider/PMS record; move refund/discount/payment; auto-apply discount |
| Daily update | send stub stays blocked until human approval; sensitive/internal/omitted facts stay out of customer-facing text |

## 9. Safe-use evidence and outcome fields

A packet or draft is safe to use only when the source/test contract shows enough evidence for its workflow. Writers should look for these fields before describing a workflow as review-ready:

- source refs, provenance, or source event id;
- typed request input and subject entity;
- data-quality issue refs and sensitivity flags when source facts are incomplete or unsafe;
- policy context and required review gates;
- safe action enum and blocked action enum values;
- included and omitted facts for customer-facing drafts;
- approval record, send stub, or audit-event draft;
- outcome disposition and reviewer/actor identity or persona;
- estimated and actual labor minutes;
- correlation id or storage projection id for replay/reporting;
- source refs on the outcome record so labor claims remain traceable.

Outcome examples:

| Workflow | Outcome evidence |
| --- | --- |
| Manager daily brief | `OutcomeRecord` captures action id, actor/persona, feedback outcome, actual labor minutes, blocked actions, source refs, and correlation id; storage/API tests cover capture and projection. |
| Data-quality hygiene | `OutcomeRecord` captures cleanup disposition, source/issue refs, actual saved or wasted minutes, blocked actions, and storage projection fields. |
| CRM retention | `OutcomeRecord` preserves opportunity evidence and follow-up outcome such as booked, interested, no response, not interested, or suppressed by staff. |
| Daily update | approval record, send stub, audit events, included facts, omitted facts, and review disposition show whether a draft is still blocked or ready for approved sending. |
| Booking triage / checkout completion | audit-event drafts and review gates preserve why staff may proceed, must review, or must fail safe; downstream outcome capture should live in the workflow or manager brief when implemented. |

## 10. Examples and non-examples

| Type | Item | Why |
| --- | --- | --- |
| Example | Agent spec | Operators need to know what an automation helper may use and what live actions remain forbidden. |
| Example | Agent prompt packet | It is the bundle of source event, typed input, policy instructions, allowed/forbidden actions, review gates, and expected output schema given to an agent. |
| Example | Manager daily brief packet | It aggregates source facts and downstream checkout/retention/data-quality/labor facts into ranked manager actions and measurable outcomes. |
| Example | Booking triage staff evaluation packet | It turns reservation/pet/policy/payment evidence into staff-review readiness and draft boundaries before confirmation. |
| Example | Daily update send stub | It proves the message remains blocked until the review gate is satisfied. |
| Non-example | Provider id wrapper by itself | It supports source/provenance lookup but is not the review packet or operator outcome. |
| Non-example | Raw customer-message body by itself | It becomes meaningful only inside a draft with channel, approval, omitted facts, and send gate. |
| Non-example | Storage code by itself | It is a persistence projection detail; document it under the outcome/storage relationship unless operators use it directly. |
| Non-example | Agent runtime mode by itself | It is runtime configuration; it does not replace the app/domain packet and review-gate contract. |

## 11. Per-entry atlas notes

### Agent spec and workflow agent

- What it is: the definition of a narrow automation helper, its purpose, allowed tool surfaces, forbidden actions, default review gates, and output schema.
- Why it exists: prevents resort workflows from becoming generic chatbot authority.
- Workflow use: all workflows, especially manager daily brief, booking triage, data-quality hygiene, and daily updates.
- Allowed: choose a baseline spec, package typed input, draft/summarize/rank according to allowed actions.
- Blocked: grant live PMS, customer-send, payment, schedule, policy, medical, or data-deletion authority outside the workflow contract.
- Source/tests: [`domain/src/agent.rs`](../../domain/src/agent.rs), [`app/src/agents.rs`](../../app/src/agents.rs).

### Agent prompt packet

- What it is: the packet handed to an agent with source event, typed workflow input, policy instructions, allowed actions, forbidden actions, review gates, and expected output schema.
- Why it exists: keeps draft generation grounded in explicit evidence and policy.
- Upstream: workflow event/result, source refs, typed app request, domain policy.
- Downstream: draft, summary, ranked recommendation, or output that the app validates before review.
- Safety evidence: source event, policy language, review gates, schema name, forbidden actions.
- Source/tests: [`app/src/agents.rs`](../../app/src/agents.rs), [`domain/src/workflow.rs`](../../domain/src/workflow.rs).

### Workflow event/result

- What it is: the domain envelope for what happened, what subject it concerns, which action is allowed/recommended, and what status or review reason applies.
- Why it exists: creates replayable, reviewable workflow history rather than ad hoc comments.
- Upstream: source/staff/adapter event.
- Downstream: internal task, draft message, status suggestion, or human review request.
- Safety evidence: `PolicyContext`, `AllowedAction`, `RecommendedAction`, `Status`, review/risk/verification notes.
- Source/tests: [`domain/src/workflow.rs`](../../domain/src/workflow.rs), workflow event docs in [`docs/workflows/workflow-event-idempotency-replay.md`](../workflows/workflow-event-idempotency-replay.md).

### Manager daily brief packet

- What it is: a manager-facing operating-day packet with source facts, checkout and retention scopes, labor estimates, ranked actions, blocked actions, and outcome capture.
- Why it exists: reduces manager dashboard reconciliation and creates a measurable labor loop.
- Upstream: analytics service-demand facts, domain daily brief facts, checkout packets, retention packets, data-quality issues, policy gates.
- Downstream: ranked manager actions, internal task drafts, manager feedback/outcome records, labor measurement.
- Review gates: manager approval for staffing, source-quality, policy-sensitive, or suppression decisions; front-desk/care/message/payment reviewers for delegated actions.
- Safety evidence: source refs on facts/actions/outcomes, `BriefAction::required_review_gates`, `Packet::all_actions_are_source_grounded`, `OutcomeRecord::records_feedback_without_external_mutation`.
- Source/tests: [`app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs), [`domain/src/daily_brief.rs`](../../domain/src/daily_brief.rs), [`app/tests/manager_daily_brief_workflow_contracts.rs`](../../app/tests/manager_daily_brief_workflow_contracts.rs), [`apps/api/tests/manager_daily_brief_outcome_capture_contract.rs`](../../apps/api/tests/manager_daily_brief_outcome_capture_contract.rs), [`storage/tests/manager_daily_brief_outcome_storage.rs`](../../storage/tests/manager_daily_brief_outcome_storage.rs).

### Booking triage packet

- What it is: the front-desk/manager packet for deciding whether a booking request is ready, missing facts, waitlisted/offered, requires review, or failed safely.
- Why it exists: reduces manual reservation readiness checks before staff confirmation.
- Upstream: reservation, pet profile completeness, policy snapshot, deposit/hard-stop evidence, vaccine/care/behavior/payment facts.
- Downstream: staff evaluation packet, confirmation/missing-info draft for review, audit-event drafts, review-gate request.
- Review gates: staff, manager, medical document, behavior, care team, payment manager, customer-message, confirmed-booking automation, or rejection approval.
- Safety evidence: deterministic result, readiness bucket, rule evaluations, approval gates, blocked actions, audit drafts.
- Source/tests: [`app/src/booking_triage.rs`](../../app/src/booking_triage.rs), [`app/tests/booking_triage_mvp.rs`](../../app/tests/booking_triage_mvp.rs).

### Data-quality hygiene packet

- What it is: a review queue for duplicate, stale, incomplete, ambiguous, conflicting, or sensitive source facts before other workflows rely on them.
- Why it exists: prevents bad source data from becoming unsafe automation or hidden labor.
- Upstream: `domain::data_quality::Issue`, source refs, issue refs, field paths, freshness, sensitivity, duplicate/profile/service-line candidates.
- Downstream: ranked hygiene actions, internal cleanup task drafts, draft validation results, cleanup outcome records and labor minutes.
- Review gates: manager/front-desk/regional/sensitive-data reviewer depending on candidate sensitivity and action kind.
- Safety evidence: `Packet::all_actions_are_source_grounded`, `Action::required_review_gates`, `DraftValidation`, issue refs, source refs, sensitivity, blocked action list.
- Source/tests: [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs), [`domain/src/data_quality.rs`](../../domain/src/data_quality.rs), [`app/tests/data_quality_hygiene_workflow_contracts.rs`](../../app/tests/data_quality_hygiene_workflow_contracts.rs), [`apps/api/tests/data_quality_hygiene_agent_contract.rs`](../../apps/api/tests/data_quality_hygiene_agent_contract.rs), [`storage/tests/data_quality_hygiene_outcome_storage.rs`](../../storage/tests/data_quality_hygiene_outcome_storage.rs).

### Checkout completion packet

- What it is: a departure/checkout packet comparing source reservation status with staff handoff evidence.
- Why it exists: reduces manual closeout auditing while avoiding unsafe PMS/status/payment changes.
- Upstream: source reservation status, staff handoff, belongings return, care summary, departure-note review.
- Downstream: checkout evidence summary, internal handoff task, retention follow-up draft for review, audit-event drafts.
- Review gates: staff handoff review or manager review when source status and handoff disagree or money/policy/sensitive concerns appear.
- Safety evidence: `CompletionStatus`, required review gates, audit-event drafts, blocked action list.
- Source/tests: [`app/src/checkout_completion.rs`](../../app/src/checkout_completion.rs), [`app/tests/checkout_completion_workflow_contracts.rs`](../../app/tests/checkout_completion_workflow_contracts.rs).

### CRM retention / grooming rebooking packet

- What it is: a source-grounded staff review packet for retention, rebooking, and follow-up opportunities after a stay, daycare visit, grooming visit, or customer interest signal.
- Why it exists: reduces missed follow-up labor without auto-sending messages or applying discounts.
- Upstream: retention opportunity evidence, completed service/stay facts, checkout proof, consent status, channel permission, source refs.
- Downstream: staff review queue, customer follow-up draft for approval, follow-up outcome evidence.
- Review gates: staff/front-desk or grooming owner for routine follow-up; manager for complaints, credits, discounts, or policy-sensitive outreach.
- Safety evidence: contact permission, source refs, required review gates, follow-up eligibility, blocked actions, outcome evidence.
- Source/tests: [`app/src/crm_retention.rs`](../../app/src/crm_retention.rs), [`app/tests/crm_retention_workflow_contracts.rs`](../../app/tests/crm_retention_workflow_contracts.rs), [`docs/workflows/crm-retention-agent.md`](../workflows/crm-retention-agent.md), [`docs/workflows/crm-retention-parts/rebooking-workflow.md`](../workflows/crm-retention-parts/rebooking-workflow.md).

### Daily update / Pawgress draft packet

- What it is: a customer-message draft packet built from staff care notes, included facts, omitted facts, internal flags, approval record, blocked send stub, and audit log.
- Why it exists: saves staff writing time while protecting sensitive/internal/policy facts from customer-facing text.
- Upstream: staff note event, pet/customer context, allowed action, policy instructions, redaction/tone/language settings.
- Downstream: customer-message draft, approval record, send stub blocked by review gate, audit events.
- Review gates: customer-message approval and care/medical/manager review when facts are sensitive, ambiguous, or not customer-safe.
- Safety evidence: included/omitted facts, omission reasons, internal flags, review disposition, approval record, `SendStub::is_blocked_until_human_approval`.
- Source/tests: [`app/src/daily_update.rs`](../../app/src/daily_update.rs), [`app/tests/daily_care_update_mvp.rs`](../../app/tests/daily_care_update_mvp.rs), [`docs/workflows/daily-care-update-agent.md`](../workflows/daily-care-update-agent.md), [`docs/workflows/operator/daily-updates-pawgress-drafts.md`](../workflows/operator/daily-updates-pawgress-drafts.md).

### Domain daily brief and operations vocabulary

- What it is: the domain vocabulary for resort operating-day views, service offerings, technology ecosystem, operating functions, pain areas, and AI use cases.
- Why it exists: keeps manager/regional packet language typed and source-grounded instead of flattening labor, capacity, service-line, and data-quality concepts into strings.
- Upstream: source-derived operations, service-line, analytics, and operating-day facts.
- Downstream: manager daily brief actions, regional exceptions, labor measurement, service-line context, and data-quality queues.
- Review gates: manager/regional review for labor, staffing, service-line, and portfolio-level decisions.
- Safety evidence: typed daily-brief sections/actions/risks and operations service-line/technology/data-quality enums.
- Source/tests: [`domain/src/daily_brief.rs`](../../domain/src/daily_brief.rs), [`domain/src/operations.rs`](../../domain/src/operations.rs), [`docs/design/labor-cost-reduction-crosswalk.md`](labor-cost-reduction-crosswalk.md), [`docs/design/manager-daily-brief-measurable-labor-loop.md`](manager-daily-brief-measurable-labor-loop.md).
