# Workflow state glossary entries

Purpose: public/non-coder glossary entries for workflow/operator state terms that recur across the NVA pet-resort agent platform. These entries translate repo/Rust vocabulary into operational meaning while preserving the code-derived contract: what the source promises, where it appears, why resort operators should care, and what not to infer.

Source basis for these entries:

- Entry shape and contract-preservation rules: `docs/design/glossary-translation-layer.md`.
- Term inventory and translation risks: `docs/glossary-translation-layer-inventory.md`.
- App workflow wiki: `app/README.md`.
- Domain policy/workflow contracts: `domain/src/policy.rs`, `domain/src/workflow.rs`.
- Shared agent contracts: `app/src/agents.rs`, `domain/src/agent.rs`.
- Workflow examples and blocked/outcome contracts: `app/src/booking_triage.rs`, `app/src/checkout_completion.rs`, `app/src/crm_retention.rs`, `app/src/daily_update.rs`, `app/src/manager_daily_brief.rs`.
- Safety citation map: `docs/safety/source-evidence-map.md`.

These glossary entries are not new product capability claims. If later source adds live provider writes, customer sends, production approval flows, or new storage projections, update the entries from source rather than widening the wording here.

## Draft

Term:
  `draft`, including app-layer types such as `booking_triage::ConfirmationDraft`, `daily_update::CustomerMessageDraft`, `daily_update::SendStub`, `checkout_completion::AuditEventDraft`, `crm_retention::OutcomeRecord`, `manager_daily_brief::OutcomeRecord`, `tools::draft_update`, and `tools::messaging::draft`.

Plain-language label:
  Staff-review artifact prepared before any live action.

Audience:
  Operators, resort leaders, product stakeholders, compliance reviewers, and agent-doc writers.

Where it appears:
  `app/README.md` describes app-owned draft artifacts and says executable examples should show draft validation without implying live provider/customer side effects. `app/src/agents.rs` describes `AgentPromptPacket<T>` as a draft/evidence boundary. Workflow modules such as `app/src/booking_triage.rs`, `app/src/daily_update.rs`, `app/src/checkout_completion.rs`, `app/src/crm_retention.rs`, and `app/src/manager_daily_brief.rs` define concrete draft, packet, send-stub, audit-draft, and outcome-record shapes.

Code-derived contract:
  A draft is an app-layer artifact prepared from source-grounded workflow context. It can carry proposed message text, a confirmation/update proposal, an audit event proposal, internal task wording, feedback/outcome data, or an evidence bundle for staff review. The app layer may validate the draft and reject requested blocked side effects before downstream code uses it.

Pet-resort operational meaning:
  For a resort team, a draft is the system doing preparation work: assembling facts, writing safe candidate language, ranking work, or formatting the next staff action so a human does not start from a blank screen. It is the pre-action paperwork for a booking, pet-parent message, checkout handoff, follow-up, daily care update, or manager brief.

Why an operator should care:
  Drafts reduce front-desk and manager labor while keeping the action boundary visible. Staff can review prepared text or evidence, check source facts, and decide whether the work should move forward instead of trusting an agent to act directly.

What not to infer:
  Do not infer that a draft has been sent, approved, applied to Gingr/PMS, persisted as a final record, or accepted as a customer/provider change. A draft confirmation does not confirm a booking; a customer-message draft does not contact the pet parent; a draft update does not mutate the provider record.

Boundary and authority:
  `app` owns draft artifacts and draft validation. `domain` supplies policy, workflow, source, and business vocabulary. Runtime shells and adapters may present or store drafts, but the draft itself is not live-action authority.

Evidence and review hooks:
  Reviewers should cite `app/README.md` for the draft family, `app/src/agents.rs` for the prompt-packet draft/evidence boundary, workflow module Rustdocs/tests for concrete draft validation, and `docs/safety/source-evidence-map.md` for the non-coder safety claim that drafts do not authorize live side effects.

Suggested public wording:
  In this repo, a draft is a staff-review artifact: the system can prepare a message, task, evidence bundle, or update proposal, but it has not sent or applied anything. Drafts reduce manual prep work while preserving human review before customer, provider, payment, schedule, or care-impacting action.

Related terms:
  `review gate`, `blocked action`, `workflow packet`, `agent spec`, `domain::workflow::Result<T>`, `app::agents::AgentPromptPacket<T>`.

## Review gate

Term:
  `review gate`, especially `domain::policy::ReviewGate` and `domain::workflow::PolicyContext::required_reviews`.

Plain-language label:
  Required human-approval stop for sensitive resort work.

Audience:
  Operators, resort leaders, compliance reviewers, product stakeholders, and maintainers.

Where it appears:
  `domain/src/policy.rs` defines `ReviewGate` variants: `ManagerApproval`, `MedicalDocumentReview`, `BehaviorReview`, `CustomerMessageApproval`, and `RefundOrDepositException`. `domain/src/workflow.rs` carries gates in `PolicyContext` and can request human review through `RecommendedAction::RequestHumanReview(policy::ReviewGate)`. `app/src/agents.rs` includes default review gates in baseline agent specs. `app/README.md` says customer sends, manager approval, medical document review, payment review, and draft-only automation are expressed through policy/workflow contracts.

Code-derived contract:
  A review gate is a named policy value that records a human-review category required before automation may proceed with sensitive work. Workflow contexts, recommended actions, agent specs, and app packets can carry these gates so downstream code and human readers can see what authority remains outside model control.

Pet-resort operational meaning:
  A review gate says, “prepare or route the work, but stop before the sensitive step until the right human or system-of-record process reviews it.” Examples include manager approval for operations exceptions, medical-document review for vaccine proof uncertainty, behavior review for group-play safety, approval before customer messaging, and refund/deposit exception review.

Why an operator should care:
  Review gates protect pet safety, customer trust, payment controls, local policy, and staff accountability while still allowing automation to reduce triage and drafting labor.

What not to infer:
  Do not infer that the gate is the approval, a UI implementation, or permission for an agent to proceed after merely mentioning the gate. It is also not a substitute for local resort policy or a production permission model by itself.

Boundary and authority:
  `domain::policy` owns the gate vocabulary. `domain::workflow` and `app` carry gates in reviewable packets/specs. Shells, tool adapters, and human workflows must honor the gates before any side effect happens.

Evidence and review hooks:
  Cite `domain/src/policy.rs` for the gate variants, `domain/src/workflow.rs` for required reviews and `RequestHumanReview`, `app/src/agents.rs` for baseline specs with default gates, and `docs/safety/source-evidence-map.md` for public-doc citation guidance.

Suggested public wording:
  `domain::policy::ReviewGate` is the repo’s named human-approval stop. It lets workflows draft, summarize, or route work while keeping manager approval, medical review, behavior review, customer-message approval, and refund/deposit exceptions outside automatic execution.

Related terms:
  `draft`, `blocked action`, `agent spec`, `domain::policy::automation::Level`, `domain::workflow::PolicyContext`, `domain::workflow::RecommendedAction`.

## Blocked action

Term:
  `blocked action`, including workflow-local enums such as `booking_triage::BlockedAction`, `checkout_completion::BlockedAction`, `crm_retention::BlockedAction`, and `manager_daily_brief::BlockedAction`.

Plain-language label:
  Action automation must not perform directly.

Audience:
  Operators, resort leaders, compliance/security reviewers, product stakeholders, and agent-doc writers.

Where it appears:
  `app/README.md` states that app modules may define local safe/blocked actions while important sends, reviews, payments, and draft-only automation remain expressed through policy/workflow contracts. `app/src/manager_daily_brief.rs` defines blocked actions such as changing staff schedules, mutating provider/PMS records, sending customer messages, moving refund/discount/payment, and hiding source data-quality issues. `app/src/booking_triage.rs`, `app/src/checkout_completion.rs`, and `app/src/crm_retention.rs` define similar workflow-local no-go actions. `app/src/agents.rs` baseline specs also list forbidden actions.

Code-derived contract:
  A blocked action names a side effect or authority-sensitive move that the workflow must not perform directly. Some modules provide code/rejection helpers, such as manager daily brief rejecting requested blocked side effects with `blocked_side_effect:<code>`. The blocked action travels with packets/outcomes so validators and reviewers can see the no-go boundary.

Pet-resort operational meaning:
  A blocked action is not a dead end; it is a handoff rule. The work may still happen through the right staff member, manager, source-of-record system, or approved provider flow. The agent/app packet is only allowed to prepare, rank, draft, or request review.

Why an operator should care:
  Blocked actions keep high-risk operational authority out of model output: booking confirmation, provider/PMS mutation, customer messaging, schedule changes, payment/refund/deposit movement, and hiding source data-quality issues remain controlled.

What not to infer:
  Do not infer that the business action is impossible, that the product is broken, or that a human cannot perform it. Also do not infer that a blocked action can be bypassed because a draft looks correct or a review gate is listed.

Boundary and authority:
  `app` owns workflow-local blocked-action enums and validation around requested side effects. `domain::policy` and `domain::workflow` own shared review vocabulary. The actual authority to perform the business action belongs to staff, managers, approved shell/runtime code, or the provider/system of record.

Evidence and review hooks:
  Cite `app/src/manager_daily_brief.rs` for explicit blocked action variants and requested-side-effect rejection, `app/src/booking_triage.rs` for booking/provider/message/payment blocks, `app/src/agents.rs` for baseline forbidden actions, and `docs/safety/source-evidence-map.md` for non-coder wording.

Suggested public wording:
  A blocked action is an action the agent/app workflow must not take directly. It marks work that may require staff, manager approval, or a source-of-record system—such as confirming a booking, changing a schedule, sending a customer message, moving money, or editing a provider record.

Related terms:
  `review gate`, `draft`, `agent spec`, `SafeAgentAction`, `domain::workflow::AllowedAction`, `domain::workflow::RecommendedAction`.

## Outcome capture

Term:
  `outcome capture`, including app-layer outcome records such as `manager_daily_brief::OutcomeRecord`, `crm_retention::OutcomeRecord`, and storage projections such as `storage::operations::ManagerDailyBriefOutcomeRecord`.

Plain-language label:
  Recording what staff did and what labor result was observed.

Audience:
  Resort leaders, operations/product stakeholders, compliance reviewers, and maintainers.

Where it appears:
  `README.md` names outcome capture as a labor-cost surface. `app/README.md` lists outcome records in the app draft/outcome family. `app/src/manager_daily_brief.rs` defines `OutcomeRecord` with action id, actor, outcome, before/actual minutes, blocked actions, source refs, correlation id, and related manager-brief dimensions. `storage/src/operations.rs` defines durable manager-daily-brief outcome projection fields. `docs/safety/source-evidence-map.md` says labor-minute savings should be cited only with outcome records.

Code-derived contract:
  Outcome capture records feedback about a reviewed workflow action: who recorded it, what outcome was selected, which source/action it ties to, and the before/actual labor-minute evidence used to estimate savings. It preserves blocked-action boundaries and source refs while creating durable evidence for reporting or audit.

Pet-resort operational meaning:
  Outcome capture closes the loop after a draft, recommendation, or manager action: did staff complete it, skip it, escalate it, or find it inapplicable, and how much manual work was actually avoided? It turns a recommendation into reviewable operational evidence.

Why an operator should care:
  Outcome capture separates estimated value from observed value. It helps leaders learn which automations reduce dashboard reconciliation and front-desk/manager handoffs, while preserving who reviewed the work and what source facts supported it.

What not to infer:
  Do not infer that outcome capture performs the underlying action, proves ROI by itself, or retroactively authorizes a side effect. Labor savings are evidence-captured measurements or estimates, not guaranteed results without reviewed records.

Boundary and authority:
  `app` owns workflow outcome-record shapes and validation around feedback. `storage` owns durable projections/records. Neither layer grants authority to mutate provider systems, send messages, move payments, or override review gates.

Evidence and review hooks:
  Cite `app/src/manager_daily_brief.rs`, `storage/src/operations.rs`, `docs/design/manager-daily-brief-measurable-labor-loop.md`, `docs/design/labor-cost-reduction-crosswalk.md`, and `docs/safety/source-evidence-map.md`. For API behavior, cite `apps/api/tests/manager_daily_brief_outcome_capture_contract.rs` when discussing tested routes.

Suggested public wording:
  Outcome capture records what happened after a reviewed workflow recommendation—who handled it, which action it ties to, and what labor minutes were actually saved. It measures reviewed work; it does not perform the work or authorize live changes.

Related terms:
  `draft`, `workflow packet`, `blocked action`, `manager_daily_brief::OutcomeRecord`, `storage::operations::ManagerDailyBriefOutcomeRecord`, `domain::source::RecordRef`.

## Workflow packet

Term:
  `workflow packet`, including app packet types such as `booking_triage::StaffEvaluationPacket`, `checkout_completion::Packet`, `crm_retention::Packet`, `daily_update::MvpPreview`, `manager_daily_brief::Packet`, and `app::agents::AgentPromptPacket<T>`.

Plain-language label:
  Review bundle for one workflow.

Audience:
  Operators, IT/architecture readers, product stakeholders, compliance reviewers, and agent-doc writers.

Where it appears:
  `app/README.md` describes request, evaluation/result, packet, draft, audit/action families. It names packet types as shell-facing outputs and explains that app packets compose domain facts without owning domain truth. `app/src/agents.rs` defines `AgentPromptPacket<T>` as a packet exchanged with an automation agent. Workflow modules define concrete packets for booking triage, checkout completion, CRM retention, daily updates, and manager daily briefs.

Code-derived contract:
  A workflow packet is a typed app-layer bundle for a specific use case. It gathers source-grounded inputs, deterministic conclusions, recommended actions, draft/evidence, required reviews, safe/blocked action metadata, and validation context so a shell, reviewer, or agent runner can handle the workflow without reinterpreting raw provider data.

Pet-resort operational meaning:
  A packet is the organized review bundle a staff member, manager, shell, or agent runtime can work from. Instead of asking a person to reconcile scattered dashboards, records, notes, and policy reminders, the packet gathers the relevant evidence and action boundary for one workflow.

Why an operator should care:
  Workflow packets reduce handoff friction and make decisions auditable. They let teams see why a booking needs review, why a manager action is ranked, what source facts support a recommendation, and which actions remain blocked.

What not to infer:
  Do not infer that a packet is an executed transaction, queued job, customer message, provider write, BI truth, or final approval. The packet is the reviewable shape; later runtime/storage/provider behavior requires separate source-backed code and authority.

Boundary and authority:
  `app` owns workflow packets and composes `domain` values into shell-facing outputs. `domain` owns the semantic truth inside the packet. `storage`, provider integrations, and runtime shells may persist, present, or execute around packets but do not become the source of domain meaning.

Evidence and review hooks:
  Cite `app/README.md` for packet families and type/module map, `app/src/agents.rs` for `AgentPromptPacket<T>`, workflow module Rustdocs/tests for concrete packets, and `domain/src/workflow.rs` for shared workflow event/result vocabulary.

Suggested public wording:
  A workflow packet is a review bundle for one resort workflow. It gathers source-backed facts, policy context, draft output, recommended actions, and blocked/review boundaries so staff or a shell can review the work without treating the packet as permission to act live.

Related terms:
  `draft`, `review gate`, `blocked action`, `outcome capture`, `agent spec`, `domain::workflow::Event`, `domain::workflow::Result<T>`.

## Agent spec

Term:
  `agent spec`, especially `app::agents::AgentSpec` / `domain::agent::Spec` and `app::agents::baseline_agent_specs()`.

Plain-language label:
  Written operating contract for a bounded automation agent.

Audience:
  Operators, IT/security reviewers, product stakeholders, maintainers, and agent-doc writers.

Where it appears:
  `app/src/agents.rs` aliases `AgentSpec` to `domain::agent::Spec`, documents it as the stable contract an agent runner receives before building a prompt packet, and returns baseline specs for pet-resort workflows. `domain/src/agent.rs` defines the underlying spec fields: name, purpose, allowed tools, forbidden actions, and default review gates. `app/README.md` maps baseline specs to the shared agent contract and says agent specs preserve `domain::policy::ReviewGate` safety rails.

Code-derived contract:
  An agent spec names the workflow identity and business purpose, lists the narrow read/draft tools an agent runner may expose, names actions it must never take directly, and carries deterministic review gates. It constrains prompt-packet construction and output validation before any generated output can become a draft or evidence bundle.

Pet-resort operational meaning:
  The spec is the job description and rule sheet for an automation helper. It says, for example, that a manager-daily-brief agent can read reservations/labor/care notes and create tasks, but must not invent occupancy, change schedules, or send customer messages without approval.

Why an operator should care:
  Agent specs make automation reviewable before prompt text enters the picture. A leader, IT reviewer, or compliance reviewer can inspect each agent by workflow, allowed tools, forbidden actions, and review gates rather than trusting broad “AI assistant” authority.

What not to infer:
  Do not infer that an agent spec is a deployed bot, a prompt alone, production permission approval, live provider write access, or customer-message send authority. It is a constraint on an agent runner and packet/output validation path.

Boundary and authority:
  `domain::agent` owns the spec value type. `app::agents` owns the app-facing alias, baseline spec list, `WorkflowAgent` trait, and prompt-packet contract. Runtime shells consume specs, but live authority remains gated by app validation, policy review, and approved tool implementations.

Evidence and review hooks:
  Cite `app/src/agents.rs` for the spec alias, baseline specs, `WorkflowAgent`, and `AgentPromptPacket<T>`. Cite `domain/src/agent.rs` for spec fields. Cite `docs/safety/source-evidence-map.md` for the safety claim that specs are narrow read/draft contracts with forbidden actions and review gates.

Suggested public wording:
  An agent spec is the operating contract for a bounded automation helper. It names the workflow, purpose, allowed read/draft tools, forbidden actions, and required review gates, but it does not deploy a bot or grant permission to change provider records, move money, diagnose pets, change schedules, or send customer messages.

Related terms:
  `workflow packet`, `review gate`, `blocked action`, `draft`, `app::agents::WorkflowAgent`, `app::agents::AgentPromptPacket<T>`, `domain::policy::ReviewGate`.

## Cross-term review checklist

- Does each entry keep the repo/Rust term visible instead of replacing it with a looser business phrase?
- Does each entry separate preparation (`draft`, `workflow packet`, `agent spec`) from authority (`review gate`, `blocked action`, source-of-record staff/system action)?
- Does outcome wording distinguish recorded/reviewed labor evidence from guaranteed savings or live execution?
- Does the public wording avoid implying customer sends, provider/PMS mutation, payment/refund movement, schedule changes, diagnosis, medical approval, or policy override?
- Are source citations specific enough that a maintainer can verify the claim without relying on this glossary as the authority?
