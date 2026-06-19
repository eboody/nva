# Agent safety model for operators

Glossary bridge: use [workflow packet](../glossary-workflow-state-terms.md#workflow-packet), [draft](../glossary-workflow-state-terms.md#draft), [review gate](../glossary-workflow-state-terms.md#review-gate), [blocked action](../glossary-workflow-state-terms.md#blocked-action), [agent spec](../glossary-workflow-state-terms.md#agent-spec), and [source refs/provenance](../glossary-source-data-terms.md#domainsourceprovenance-and-domainsourcerecordref-as-data-evidence) as nearby explanations for non-coders.

Purpose: explain, in plain English, what the current agent model can read, what it can draft, and what it must never do by itself. This is an operator-facing safety narrative, not a developer tutorial and not a production permission approval.

Short version: agents in this repository prepare source-backed review work. They do not run the resort. The app builds a narrow workflow bundle, attaches source evidence, applies deterministic checks, asks an agent only for draft or summary work, validates the result, and keeps live customer, provider, payment, reservation, medical, schedule, cleanup, and approval actions behind human or system-of-record gates.

Primary evidence: `docs/safety/source-evidence-map.md`, especially its evidence table and open evidence gaps. The source map points to `app/src/agents.rs`, `app/src/booking_triage.rs`, `app/src/manager_daily_brief.rs`, `app/src/tools.rs`, `domain/src/policy.rs`, `domain/src/workflow.rs`, `domain/src/source.rs`, `storage/src/operations.rs`, and the labor-loop docs under `docs/design/`.

## What agents may read

Agents may read review packets and source-backed facts that the app prepares for a specific workflow. They are not expected to roam through provider systems or decide what facts matter on their own.

Allowed read inputs include:

1. Source-grounded context packets.
   - Example: `app::agents::AgentPromptPacket` carries workflow identity, triggering source event, typed app input, policy instructions, and expected output schema. The source map cites `app/src/agents.rs` and `docs/architecture/agent-prompt-packet.md` for this boundary.
   - Operator translation: the agent gets a prepared case file, not general operating authority.

2. Normalized domain and store facts.
   - Example: booking triage reads intake, pet profile evidence, policy/reservation facts, and deterministic rule results before any recommendation is drafted. The source map cites `app/src/booking_triage.rs`, `domain/src/policy.rs`, and `domain/src/source.rs`.
   - Operator translation: the workflow starts with facts already shaped by the app and domain model, not free-form model judgment.

3. Provider-derived evidence through adapters and tool ports.
   - Example: source provenance can identify the upstream system, endpoint or record identity, extraction batch, timestamp, request scope, schema version, payload hash, and raw payload reference. The source map cites `domain/src/source.rs`.
   - Example: `app::tools` defines narrow ports for customer stores, reservation systems, payment helpers, messaging drafts, portal lookups, documents/OCR, media, and Hermes automation hooks. The source map cites `app/src/tools.rs` and `app/src/tools/error.rs`.
   - Operator translation: provider facts can be cited as evidence, but the agent does not become the provider system.

4. Prior outcomes and audit-friendly records.
   - Example: manager daily brief outcome records persist action id, outcome, before/actual minutes, actor, source refs, correlation id, location/day/action/persona, and estimated minutes saved. The source map cites `storage/src/operations.rs`, `app/src/manager_daily_brief.rs`, and `docs/design/manager-daily-brief-measurable-labor-loop.md`.
   - Operator translation: the system can remember what staff did with a recommendation and whether it saved time, without treating the recommendation as an executed action.

5. Policy instructions and review gates.
   - Example: `domain::policy::ReviewGate` names human approval gates such as manager approval, medical document review, behavior review, customer message approval, and refund/deposit exception. The source map cites `domain/src/policy.rs`, `app/src/agents.rs`, `app/src/booking_triage.rs`, and `app/src/manager_daily_brief.rs`.
   - Operator translation: the case file says where the agent must stop.

## What agents may draft

Agents may prepare staff-review artifacts that reduce manual gathering, summarizing, and rewriting. A draft is not the live action. It is a proposed artifact for staff, manager, compliance, or another app-owned validator to review.

Allowed draft outputs include:

1. Customer-safe message drafts.
   - Example: messaging tools are customer-message draft contracts and never send without staff approval. The source map cites `app/src/tools.rs`, `domain/src/message.rs`, `domain/src/policy.rs`, and `docs/architecture/agent-permissions-by-workflow.md`.
   - Operator translation: the agent can help write a response, but staff or an approved deterministic send path must own sending.

2. Manager brief recommendations.
   - Example: Manager Daily Brief ranks service-demand, checkout-exception, retention, and data-quality actions with source facts, review gates, and labor-minute estimates. The source map cites `app/src/manager_daily_brief.rs`, `domain/src/daily_brief.rs`, and `docs/design/manager-daily-brief-measurable-labor-loop.md`.
   - Operator translation: the agent helps build the morning action queue; it does not change staffing, schedules, PMS records, payments, or customer messages.

3. Staff evaluation packets for booking triage.
   - Example: booking triage exposes deterministic results, readiness buckets, approval gates, safe agent actions, blocked actions, and a `StaffEvaluationPacket`. The source map cites `app/src/booking_triage.rs` and the `README.md` glossary row for `app::booking_triage`.
   - Operator translation: the packet tells staff why a booking request looks ready, missing information, blocked, or approval-required. It does not confirm the booking.

4. Exception summaries.
   - Example: data-quality hygiene turns source ambiguity into ranked cleanup work and does not autonomously repair source systems. The source map cites `docs/design/data-quality-hygiene-labor-loop.md`, `domain/src/data_quality.rs`, `domain/src/source.rs`, and `storage/src/operations.rs`.
   - Operator translation: the agent may summarize what looks inconsistent or stale. A human or approved system process owns correction.

5. Provider-portal instructions and internal checklists.
   - Example: tool-port modules separate portal lookup, document/OCR intake, media snapshots, reservation availability, draft updates, payment helpers, messaging drafts, and Hermes task/schedule drafts. The source map cites `app/src/tools.rs` and `app/src/tools/error.rs`.
   - Operator translation: the agent can prepare instructions or checklists for someone to execute in the proper system of record.

6. Proposed next actions.
   - Example: `domain::workflow` preserves workflow events, policy context, allowed actions, review reasons, recommended actions, risk flags, and verification notes. The source map cites `domain/src/workflow.rs`.
   - Operator translation: the agent can say “review this vaccine document,” “check this checkout exception,” or “approve this retention follow-up draft,” but the responsible staff or manager still decides.

## What agents may never do directly

The following actions must not be described as direct agent powers in operator docs unless a later source explicitly adds and approves a deterministic live-action path. Current source evidence keeps these blocked or review-gated:

- Live customer sends.
- Payment capture, refund, void, deposit movement, discount, waiver, or other money movement.
- Medical, vaccine, behavior, care, or safety decision overrides.
- Provider/PMS/customer-store data mutation.
- Booking confirmation, rejection, waitlist movement, check-in, checkout, capacity release, or reservation changes.
- Staff schedule changes, payroll changes, or labor-system mutation.
- Destructive cleanup, hiding source data-quality issues, or marking ambiguity resolved without review.
- Policy changes, SOP changes, or automation-authority upgrades.
- Manager approval, staff approval, medical review, behavior review, refund/deposit exception approval, or customer-message approval on the agent's own authority.

Concrete examples from the source map:

- Baseline agent specs forbid direct high-risk actions such as confirming bookings, changing schedules, waiving deposits, diagnosing pets, or sending customer messages without approval. Evidence: `app/src/agents.rs` and `docs/architecture/agent-permissions-by-workflow.md`.
- Booking triage blocks provider mutation, booking confirmation, customer sends, and payment movement unless separate app/human gates clear. Evidence: `app/src/booking_triage.rs`, `app/src/tools.rs`, and `domain/src/policy.rs`.
- Manager Daily Brief blocks schedule changes, provider/PMS mutation, customer sends, refunds, discounts, payments, and hiding source data-quality issues. Evidence: `app/src/manager_daily_brief.rs`, `docs/design/manager-daily-brief-measurable-labor-loop.md`, and `docs/design/labor-cost-reduction-crosswalk.md`.
- Tool ports keep external capabilities behind app-owned interfaces. A port is not proof that a live implementation exists or that the agent may skip review. Evidence: `app/src/tools.rs`, `app/src/tools/error.rs`, and `app/README.md`.

## Why this is not autonomous live operations

This model is not “AI operates the resort.” It is “the app prepares review work, the agent drafts within that boundary, and people or approved systems perform the live operation.”

The safety model has five control points:

1. Deterministic checks happen before and after drafting.
   - Booking triage runs deterministic policy/evidence evaluation before any agent drafting. Its typestate flow orders intake, pet profile evidence, policy/reservation facts, and staff-ready deterministic review. Evidence: `app/src/booking_triage.rs`.
   - `WorkflowAgent` implementations build prompt packets and validate structured output before downstream code can use it. Evidence: `app/src/agents.rs` and `app/README.md`.

2. Review gates name who must approve.
   - Review gates include manager approval, medical document review, behavior review, customer message approval, and refund/deposit exception. Evidence: `domain/src/policy.rs`.
   - Operator translation: “human review” is not vague. The gate should name the kind of approval required.

3. Blocked actions stay visible.
   - Blocking is part of the workflow packet, not a hidden warning. Booking triage, checkout, retention, daily updates, manager brief, and data-quality loops all preserve blocked live actions in their app contracts or design docs. Evidence: `app/README.md`, `app/src/booking_triage.rs`, `app/src/manager_daily_brief.rs`, and `docs/design/labor-cost-reduction-crosswalk.md`.
   - Operator translation: if evidence is incomplete or action is sensitive, the system turns that into review work instead of letting the agent improvise.

4. Tool-port boundaries prevent broad access.
   - `app::tools` defines narrow capability interfaces. Workflow logic can produce draft/update requests and blocked-action lists without directly sending messages, mutating Gingr or another provider, moving payments, or editing schedules. Evidence: `app/README.md` and `app/src/tools.rs`.
   - Operator translation: adapters and ports are controlled doors, not an all-access key.

5. Source evidence and outcome capture make work auditable.
   - Source refs and provenance keep facts tied to their source record. Evidence: `domain/src/source.rs`.
   - Outcome records capture what staff/manager did and how much labor was saved. Evidence: `storage/src/operations.rs`, `app/src/manager_daily_brief.rs`, and `docs/design/manager-daily-brief-measurable-labor-loop.md`.
   - Operator translation: the system can explain why a recommendation existed and whether it helped, without pretending the agent executed the result.

## Why this lowers labor cost safely

The labor-cost thesis is not that the agent replaces accountable operators. The thesis is that managers and front-desk teams waste time rediscovering facts, reconciling dashboards, assembling handoffs, writing repetitive drafts, and chasing ambiguous records. The app/agent boundary removes that repeated prep work while leaving decisions and live side effects with the right owner.

Safe labor reduction comes from:

1. Less dashboard reconciliation.
   - Manager Daily Brief turns service demand, checkout exceptions, retention opportunities, and data-quality issues into a prioritized review queue. Evidence: `app/src/manager_daily_brief.rs` and `docs/design/manager-daily-brief-measurable-labor-loop.md`.
   - Example: instead of a manager manually comparing demand and staffing dashboards, the brief can point to a demand-versus-staffing review action with source facts and estimated minutes saved.

2. Fewer repeated handoffs.
   - Booking triage, checkout completion, CRM retention, daily update, and manager brief packets put the rationale, evidence, review gates, and blocked actions in one place. Evidence: `app/README.md`.
   - Example: front desk receives a staff evaluation packet for a booking request instead of separately checking intake, pet profile, vaccine/policy status, reservation facts, customer-message copy, and payment exceptions.

3. Faster triage.
   - Booking triage readiness buckets and deterministic results help staff see whether work is ready, missing information, blocked, or approval-required. Evidence: `app/src/booking_triage.rs`.
   - Manager brief action kinds prioritize demand/staffing, checkout exceptions, retention follow-up drafts, and data-quality investigations. Evidence: `app/src/manager_daily_brief.rs`.

4. Reusable evidence packets.
   - `AgentPromptPacket`, workflow packets, source refs, provenance, and storage outcome records make evidence reusable across review, audit, and later outcome measurement. Evidence: `app/src/agents.rs`, `domain/src/source.rs`, and `storage/src/operations.rs`.

5. Manager time focused on decisions rather than data gathering.
   - The labor crosswalk states that labor cost falls when teams stop rediscovering source facts, reconciling dashboards by hand, rewriting repetitive drafts, and reworking ambiguous records, while the deterministic app still owns facts, policy, workflow state, review gates, audit, outcome capture, and every external side effect. Evidence: `docs/design/labor-cost-reduction-crosswalk.md`.
   - Operator translation: managers still make the call; the system makes the call easier and faster to review.

## Known gaps / needs owner decision

Do not hide these caveats in operator or product-facing material:

1. Some architecture docs contain stale source anchors.
   - The source map says architecture drafts mention `domain/src/agents.rs` and `domain/src/tools.rs`, but current source paths are `app/src/agents.rs`, `domain/src/agent.rs`, and `app/src/tools.rs`.
   - Owner decision needed: update or avoid stale anchors before using the architecture docs as external evidence.

2. Draft architecture docs are not production permission approval.
   - The source map says `docs/architecture/agent-permissions-by-workflow.md` and related architecture drafts should be cited as conservative implementation inputs, not final production approval.
   - Owner decision needed: final security/product approval evidence before claiming production live-action permissions.

3. No source-backed production runtime was identified for live customer sends, provider writes, refunds, deposits, schedule changes, or reservation changes.
   - The source map says current docs and code define draft/review/payment/provider boundaries but did not find a production runtime that executes those live side effects.
   - Owner decision needed: if any live-action path is desired, add a deterministic approved write contract, source evidence, review/audit behavior, and explicit operational approval before claiming it.

4. Labor savings are recordable, not automatically proven.
   - Manager Daily Brief can estimate and record minutes saved, but product claims need actual outcome records. Evidence: `storage/src/operations.rs` and `docs/design/manager-daily-brief-measurable-labor-loop.md`.
   - Owner decision needed: define which outcome metrics NVA will accept as labor-cost evidence.

## Operator checklist

Before describing an agent workflow as safe, confirm all of the following are true:

- The workflow starts from source-grounded context, normalized facts, or a named tool-port result.
- The output is a draft, summary, packet, checklist, recommendation, or proposed next action.
- Every sensitive area has a named review gate or blocked action.
- The docs do not imply autonomous customer, provider, reservation, payment, schedule, medical, cleanup, policy, or approval authority.
- Source evidence can be cited.
- Outcomes can be captured if the workflow is used to support labor-savings claims.

If any item is missing, describe the workflow as a draft/review concept or evidence gap, not as autonomous live operations.
