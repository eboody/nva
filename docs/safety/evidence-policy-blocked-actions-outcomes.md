# Evidence, policy, blocked actions, and outcome capture

Glossary bridge: this page is the safety home for [source refs/provenance](../glossary-source-data-terms.md#domainsourceprovenance-and-domainsourcerecordref-as-data-evidence), [workflow packets](../glossary-workflow-state-terms.md#workflow-packet), [drafts](../glossary-workflow-state-terms.md#draft), [review gates](../glossary-workflow-state-terms.md#review-gate), [blocked actions](../glossary-workflow-state-terms.md#blocked-action), and [outcome capture](../glossary-workflow-state-terms.md#outcome-capture).

Purpose: explain the safety mechanism behind the labor-saving workflows in this repo. The short version is: the app gathers source evidence, applies policy and review gates, lets agents prepare only reviewable drafts or summaries, blocks unsafe actions, and records outcomes so the next run starts from evidence instead of manual reconstruction.

This document is a plain-English explainer. It is not a new permission grant. If a workflow or tool port does not explicitly authorize a live side effect, assume the action remains staff-, manager-, or system-owned.

Primary source map: [Source evidence map for agent safety and human review](source-evidence-map.md).

## The loop in one sentence

A workflow should move through this loop:

1. Gather source evidence.
2. Apply policy instructions, deterministic checks, and review gates.
3. Prepare a staff-visible packet, brief, or draft.
4. Stop on blocked actions that need human review or an approved app-owned write path.
5. Capture the reviewed outcome and source references for future runs.

That is how the system can save labor without turning an agent into an autonomous resort operator.

## What “source evidence” means

Source evidence is the set of facts the app can point to when it explains why a draft, action, or review packet exists. It includes several layers:

- External/provider facts: facts read from systems of record or provider-shaped inputs, such as reservation data, customer/pet profiles, service demand, checkout status, payment/deposit signals, or BI/labor projections. Provider facts are evidence; they are not permission for an agent to write back to the provider. The tool-port boundary is documented in [`app::tools`](../../app/src/tools.rs) and summarized in the app README’s tool-port map ([`app/README.md`](../../app/README.md)).
- Normalized domain facts: provider facts promoted into domain language such as `domain::source`, `domain::policy`, `domain::workflow`, `domain::daily_brief`, reservation status, vaccine requirements, or service-demand facts. The workspace glossary links these boundaries in [`README.md`](../../README.md), especially `domain::source`, `domain::policy`, `domain::workflow`, and manager daily brief truths.
- Provenance and source refs: trace data that says where a fact came from, which source system or endpoint produced it, which record it refers to, when it was extracted, and which raw or batch reference supports it. The source/provenance vocabulary lives in [`domain::source`](../../domain/src/source.rs).
- Prompt packets: app-owned packets that pass workflow identity, input, triggering event, policies, and expected output shape to an agent. The packet can ask for a summary, ranking, classification, or draft, but the packet is not live operational authority. See [`app::agents::AgentPromptPacket`](../../app/src/agents.rs) and the architecture draft on [agent prompt packets](../architecture/agent-prompt-packet.md).
- Prior outcomes: records of what staff or managers did after reviewing a packet or draft, including disposition, actor, source refs, before/actual labor minutes, and estimated minutes saved. Outcome persistence is represented in [`storage::operations`](../../storage/src/operations.rs) and connected to labor measurement in the [labor-cost reduction crosswalk](../design/labor-cost-reduction-crosswalk.md).

A good workflow does not ask an agent to “figure out the truth” from scratch. It gives the agent a packet built from source evidence and asks for a constrained draft or recommendation that the app can validate.

## What policy instructions and review gates do

Policy instructions and review gates tell the workflow what it may do, what it may only draft, and what it must not do.

They include:

- Allowed actions: safe internal work such as summarizing evidence, ranking manager actions, drafting staff packets, drafting customer-message text for review, estimating labor minutes, or recording feedback. These actions reduce staff lookup and writing time but do not mutate live systems.
- Review gates: named human-approval points such as manager approval, medical document review, behavior review, customer message approval, and refund/deposit exception review. These live in [`domain::policy::ReviewGate`](../../domain/src/policy.rs) and are reused by `app::agents`, `app::booking_triage`, and `app::manager_daily_brief`.
- Deterministic checks: app-owned checks that happen before or after any agent draft. Booking triage evaluates rule findings and readiness before staff packet output ([`app::booking_triage`](../../app/src/booking_triage.rs)). Manager daily brief builds source-grounded action packets and requires actions to carry source facts ([`app::manager_daily_brief`](../../app/src/manager_daily_brief.rs)). `WorkflowAgent` implementations in [`app::agents`](../../app/src/agents.rs) validate proposed outputs before downstream code can use them.
- Workflow-specific constraints: each workflow owns its own safe and blocked action vocabulary. Booking triage treats confirmation, provider mutation, customer sends, and payment movement as blocked unless separate gates clear. Manager daily brief treats schedule changes, provider/PMS mutation, customer sends, refunds, discounts, payments, and hidden source-quality issues as blocked.

The app README states the same boundary at the crate level: `app` composes use-case packets and draft/review envelopes from domain truth, while live provider DTOs, storage schemas, HTTP routes, and external mutation belong elsewhere ([`app/README.md`](../../app/README.md)).

## What “blocked actions” mean

A blocked action is work the workflow should not let an agent complete on its own. Blocking is not failure; it is the control point that keeps labor-saving automation safe.

The agent or workflow should stop and ask for the right review when an action would require:

- Staff review: missing information, stale or conflicting source data, unclear customer/pet/reservation identity, or a staff queue decision.
- Manager review: schedule/labor changes, capacity or availability decisions, policy exceptions, operational tradeoffs, or data-quality ambiguity that affects location work.
- Customer-message review: any outbound customer or pet-parent message, especially booking, incident, medical, payment, sensitive care, discount, refund, or reputation communication.
- Payment review: deposit exceptions, refunds, discounts, authorization/capture, ledger movement, or any money-impacting action.
- Provider/PMS review: booking confirmation, check-in/out, capacity release, room/playgroup assignment, customer/pet/profile mutation, source-system cleanup, or any write back to Gingr/PMS/provider records.
- Medical/behavior review: vaccines, medical documents, care instructions, incident/safety issues, aggression/temperament, or any advice that staff should validate.

The important phrasing for docs is: “the workflow prepares the evidence and draft; staff or an approved app-owned gate performs the live action.” Do not say that agents can mutate live providers, send customer/payment actions, waive deposits, or make medical/behavior decisions.

## What “outcome capture” means

Outcome capture records what happened after a draft, review packet, or manager action. It turns a one-time review into reusable evidence.

A useful outcome record captures:

- The workflow action or draft id.
- The final disposition, such as completed, reviewed, deferred, suppressed, source-fact-wrong, or needs follow-up.
- Who recorded the outcome and in what role.
- The source refs that justified the original recommendation.
- The location, operating day, action kind, persona, or other reporting group.
- Estimated manual minutes before the workflow and actual minutes after review, when labor measurement applies.
- Any caveat that should be visible next time, such as wrong source data, missing vaccine evidence, contact suppression, or manager override.

[`storage::operations::ManagerDailyBriefOutcomeRecord`](../../storage/src/operations.rs) is the clearest current storage example: it persists action id, outcome, before/actual minutes, actor, source refs, reporting dimensions, and estimated minutes saved. The [labor-cost reduction crosswalk](../design/labor-cost-reduction-crosswalk.md) says labor-savings claims must be measured with outcome capture, not merely asserted.

Outcome capture is what prevents repeated manual reconstruction. If a manager already reviewed a demand-versus-staffing action and recorded that the source was wrong or the task saved 30 minutes, the next brief should start from that record instead of asking another person to re-check every dashboard.

## Example 1: Booking triage

Booking triage is the front-desk/staff review example. The relevant surfaces are [`app::booking_triage`](../../app/src/booking_triage.rs), [`app::agents`](../../app/src/agents.rs), [`domain::source`](../../domain/src/source.rs), [`domain::policy`](../../domain/src/policy.rs), [`domain::workflow`](../../domain/src/workflow.rs), [`app::tools`](../../app/src/tools.rs), and [`storage::operations`](../../storage/src/operations.rs).

Flow:

1. Inquiry/profile/vaccine/source evidence arrives.
   - A booking inquiry or reservation reference is attached.
   - Pet profile evidence is attached next.
   - Policy and reservation facts are attached after that.
   - Vaccine, deposit, hard-stop, and source freshness evidence remain visible as evidence refs.

2. Deterministic triage runs before agent drafting.
   - `app::booking_triage::Request` uses a typestate sequence so intake, pet profile evidence, and policy evidence are collected in order.
   - Rule evaluations produce a `DeterministicResult` with readiness buckets, review findings, approval gates, failure codes, safe actions, and blocked actions.
   - The result can say the reservation is ready for staff approval, missing information, vaccine pending, special review, waitlisted/availability review, or failed-safe data cleanup.

3. A staff packet or draft is prepared.
   - `StaffEvaluationPacket` gives staff the source-backed rationale and recommended next queue.
   - An agent may summarize evidence or draft customer-safe language for review through an `AgentPromptPacket`.
   - The draft is not a booking confirmation and not a customer send.

4. Blocked/manual actions remain blocked.
   - The workflow must not confirm or reject the booking, promise availability, hold or release capacity, assign rooms/playgroups, mutate provider/PMS records, send customer messages, clear vaccine/care/behavior exceptions, or move deposit/payment money.
   - If the next step requires medical document review, behavior review, customer-message approval, payment/deposit exception review, manager approval, or provider/PMS mutation, the workflow stops at the packet and asks the responsible staff or approved app-owned gate.

5. The outcome is captured.
   - Staff can record whether the packet was approved, deferred, missing information, source-fact-wrong, or routed to another review queue.
   - The record should keep reservation/action identity, source refs, actor, disposition, and any labor or rework signal so future booking triage does not require another manual reconstruction.

Labor-saving effect: staff no longer have to hunt across reservation notes, pet profiles, vaccine documents, deposit status, and policy pages before deciding the next queue. Safety effect: the workflow still stops before live provider changes, customer communication, payment movement, or medical/behavior decisions.

## Example 2: Manager daily brief

Manager Daily Brief is the manager/front-desk prioritization example. The relevant surfaces are [`app::manager_daily_brief`](../../app/src/manager_daily_brief.rs), [`domain::daily_brief`](../../domain/src/daily_brief.rs), [`domain::source`](../../domain/src/source.rs), [`domain::policy`](../../domain/src/policy.rs), [`domain::workflow`](../../domain/src/workflow.rs), [`app::agents`](../../app/src/agents.rs), [`app::tools`](../../app/src/tools.rs), [`storage::operations`](../../storage/src/operations.rs), and the [labor-cost reduction crosswalk](../design/labor-cost-reduction-crosswalk.md).

Flow:

1. Service demand, checkout, retention, and source-quality evidence arrives.
   - Service-demand facts show demand units, projection version, source refs, and data-quality caveats.
   - Checkout exception packets show unresolved departure or handoff work.
   - Retention packets show source-grounded follow-up opportunities and contact/consent evidence.
   - Prior outcomes show which similar actions were completed, deferred, or source-fact-wrong.

2. The app builds a prioritized brief.
   - `app::manager_daily_brief::Request` scopes evidence to a location, operating day, and manager persona.
   - `Packet` and `BriefAction` carry source facts, action kind, priority, rationale, and labor impact estimate.
   - Safe agent work is internal: summarize evidence, rank manager actions, draft internal tasks, record feedback, and estimate labor minutes saved.

3. The manager chooses the action.
   - The manager sees a ranked queue instead of manually reconciling demand, checkout, retention, and source-quality dashboards.
   - The manager may choose to adjust staffing, follow up with a front-desk task, defer, mark source data wrong, or route another review.
   - The agent does not make that operational decision.

4. Blocked/manual actions remain blocked.
   - The brief must not change staff schedules, mutate provider/PMS records, send customer messages, issue refunds/discounts/payments, hide source data-quality issues, or perform member-facing actions.
   - If action requires a schedule edit, provider write, customer outreach, payment movement, or policy exception, the brief stops at recommendation/draft and asks the manager or approved downstream process.

5. Outcome and labor minutes are captured.
   - `OutcomeRecord` and `ManagerDailyBriefOutcomeRecord` capture action id, outcome, actor, before minutes, actual minutes, source refs, reporting group, and estimated minutes saved.
   - The next brief can use those records to avoid rediscovering the same facts and to support measured labor-savings claims.

Labor-saving effect: the manager starts the day with a source-backed action queue instead of scanning multiple dashboards and asking staff to reconstruct context. Safety effect: the system ranks and drafts only; it does not edit schedules, provider records, customer messages, or money.

## Why this saves labor without unsafe autonomous changes

The labor savings come from reducing search, reconciliation, rewriting, prioritization, and repeat explanation work:

- Source evidence removes “where did this fact come from?” backtracking.
- Policy instructions and deterministic checks remove avoidable judgment calls from obvious routing work.
- Prompt packets let agents summarize or draft from app-owned context instead of improvising from raw provider access.
- Blocked actions make unsafe steps explicit instead of burying them in vague automation language.
- Outcome capture turns staff/manager review into durable evidence for the next workflow.

The safety boundary is equally important: the same mechanism that saves labor also names what the agent cannot do. This repo’s current source-backed posture is draft/review/outcome capture, with live customer, payment, provider/PMS, schedule, medical, and behavior actions staying blocked unless a separate deterministic, approved, app-owned path is added and documented.
