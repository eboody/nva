# Entity/action safety overlay template

Use this template for one entity/action family page under `docs/safety/entity-action-overlays/`. The goal is to help a non-coder pick a concrete resort entity or action and understand what automation may safely do, what stays blocked or reviewed, what source evidence is required, and what audit/outcome record proves value.

Do not turn this into generic policy prose. Every section must name the specific pet-resort entity/action family, the labor or error cost reduced, the source authority, the review role, and the proof record.

Copy the section headings below for each child overlay. Keep implementation paths after operator English.

## 1. Plain-English entity/action definition and labor-cost problem

Required content:

- Name the entity/action family in resort language.
- Name the staff role whose work is reduced: front desk, general manager, regional ops leader, groomer, trainer, kennel/daycare lead, medical/vaccine reviewer, payment/accounting reviewer, IT/security, or product/ops owner.
- Name the repeated manual work or error risk reduced.
- State the safe outcome in one sentence: reviewed draft, ranked queue, source-backed recommendation, deterministic policy check, internal task, or outcome record.

Use this shape:

> This overlay helps `<role>` reduce `<manual work or error cost>` for `<entity/action family>` by showing what source facts automation may read, what it may draft or recommend, what a human must approve, and which outcome/audit record proves safe use.

Avoid generic openings such as “This module provides boundaries.”

## 2. Workflows/contracts featuring it and adjacent entities

Required content:

- List the operator workflows where the entity/action appears.
- List adjacent entities it depends on or feeds: customer, pet, reservation, document, vaccine, care note, incident, message, approval record, policy, source ref, payment, provider record, tool-port draft, internal task, outcome record.
- Name the app/domain/storage/integration contracts that feature it.
- If a workflow page exists, cite it; if it is planned or missing, mark it as a gap instead of inventing behavior.

Recommended table:

| Workflow or contract | Entity/action role | Adjacent entities | Source path or doc evidence |
| --- | --- | --- | --- |
| `<workflow>` | `<how this entity/action appears>` | `<related entities>` | `<relative source/doc paths>` |

## 3. Who/what is authoritative

Required content:

- Separate source evidence from behavioral authority and human approval.
- Name which system, crate, module, test, or human role owns each fact or decision.
- Mark provider facts as provider evidence, not business policy, unless a domain/app contract promotes them.

Recommended table:

| Fact or decision | Authoritative source | What it can prove | What it does not prove |
| --- | --- | --- | --- |
| Source fact | Provider/read model/provenance path | Why the recommendation exists | Human approval or live permission |
| Business invariant | `domain::*` path | The semantic rule or policy vocabulary | Provider write-back permission |
| Workflow packet | `app::*` path | What may be drafted, ranked, validated, or recorded | Live resort execution |
| Outcome/audit record | `storage::*`, app outcome, or API test path | What staff reviewed/did and how value was measured | That the agent took the live action |
| Human approval | Reviewer role and approval record | Permission for the approved downstream step | Permission for unrelated actions |

## 4. Agent may read

Required content:

- List only source-backed facts the agent/app workflow may inspect.
- Include provenance/source refs, policy snapshots, read-model facts, existing reviewed outcomes, and scoped context where relevant.
- Name the path that proves read access or context shape.
- Include any scope limits: location, operating day, customer, pet, reservation, source snapshot, document, payment ref, or workflow packet.

Use verbs such as read, inspect, summarize, extract, compare, and validate.

Do not imply broad database, provider, document, payment, or customer-message access if the cited source only shows a narrow packet or port.

## 5. Agent may draft/recommend/rank/record

Required content:

- List draft/review-only actions allowed by the app/domain contract.
- Tie every allowed verb to a concrete artifact: staff evaluation packet, confirmation draft, daily update draft, retention packet, internal task draft, manager brief action, review request, validation result, outcome record.
- Say whether the action is draft-only, internal-task-only, manager-review-required, or never-automate when `domain::policy::automation::Level` or workflow policy is relevant.

Allowed verb examples:

- draft a customer-message body for approval;
- rank a manager/front-desk queue;
- recommend a review gate;
- flag risk or missing evidence;
- summarize care/source facts;
- validate output against blocked actions;
- record a reviewed disposition, actual minutes, feedback, source refs, issue refs, and correlation id.

## 6. Agent must not do directly

Required content:

- Name exact blocked verbs for this entity/action family.
- Include the default blocked actions when relevant: customer sends, provider/PMS writes, schedule/capacity changes, booking status execution, payment/refund/discount movement, medical/vaccine/behavior approvals, incident/legal/safety decisions, destructive source cleanup, policy changes, staff scheduling/payroll actions, secret-dependent/live external side effects.
- Say what the agent should do instead: route to reviewer, draft packet, create internal task, fail closed, or record blocked outcome.

Recommended table:

| Blocked direct action | Why blocked | Correct safe path |
| --- | --- | --- |
| `<send/mutate/move/approve/change/etc.>` | `<customer trust, pet safety, money, source authority, local policy, or security reason>` | `<reviewer role + packet/outcome>` |

## 7. Required human reviewer role(s) and approval condition

Required content:

- Name the exact role(s), not just “human.”
- State the approval condition each role controls.
- State what that role does not approve.
- Map to `domain::policy::ReviewGate` where applicable.

Common roles:

| Role | Usually approves | Does not approve |
| --- | --- | --- |
| Staff/front-desk lead | Routine intake completeness, source-backed queue work, handoff quality, internal tasks | Manager exceptions, money movement, medical/behavior approval, provider write-back permission |
| Manager/general manager | Capacity, staffing, policy exceptions, incidents/complaints, suppression/escalation, customer-trust decisions | Medical/vaccine validity unless trained; payment processing authority unless assigned; IT integration scope |
| Medical/vaccine qualified staff | Vaccine proof, medical/care-document ambiguity, medication/care readiness | Payment/refund decisions, provider integration permissions, general customer-send approval outside medical wording |
| Behavior/daycare lead | Temperament, group-play, behavior safety, incident-care implications | Payment, provider write-back, broad policy changes |
| Customer-message reviewer/approved sender | Final recipient, channel, timing, body, contact preference/consent, suppression | Provider/PMS mutation, payment movement, medical/behavior approval |
| Payment/accounting | Deposits, refunds, waivers, discounts, balances, receipts, duplicate/amount/provider ambiguity | Medical/behavior/schedule decisions, customer-message text except payment wording |
| IT/security | Integration scope, secrets, logging, rate limits, tool-port failure modes, provider write-back security | Business policy approval, message/customer/payment decisions |
| Product/ops owner | Whether a workflow or port is allowed to exist and under what policy | Individual live-action approval without the proper operational reviewer |

## 8. Required source evidence before a recommendation

Required content:

- List the minimum source evidence for the entity/action recommendation.
- Include source refs/provenance, timestamps, source system/endpoint/record id, policy snapshot, reviewer-facing facts, and uncertainty flags where needed.
- Say what to do when evidence is missing, stale, ambiguous, or conflicting.

Recommended table:

| Recommendation or draft | Required source evidence first | If missing/stale/ambiguous |
| --- | --- | --- |
| `<specific recommendation>` | `<source refs, policy snapshot, packet ids, facts>` | `<route to review, fail closed, mark gap, do not recommend live action>` |

## 9. Outcome/audit record proving safe use and value measurement

Required content:

- Name the record that proves safe use for this entity/action family.
- Separate source evidence, draft creation, human approval, downstream live action elsewhere, and durable outcome record.
- Include labor-value fields without unsupported ROI claims.

Fields to look for or require:

- source refs/provenance and issue refs;
- context packet id, workflow event id, action id, draft id, task id, and correlation id;
- review gates and blocked action reasons;
- requested side effects and validation result;
- approval status, reviewer/actor/persona, timestamp, and decision reason;
- disposition, feedback, actual minutes, before/after minutes, minutes saved or avoided, wrong-source findings, reporting group;
- `live_side_effects_allowed: false`, `outcome_persisted: false`, or equivalent proof when a blocked action was rejected.

Recommended table:

| Proof needed | Example field or record | What it proves | What it does not prove |
| --- | --- | --- | --- |
| Source evidence | `RecordRef`, `Provenance`, source refs | The recommendation was grounded | Approval or completion |
| Draft/recommendation | draft id, packet id, safe action list | Work product was prepared | Customer/provider/payment/schedule action happened |
| Human approval | approval record, reviewer role, gate, status | The sensitive step was reviewed | Authority for unrelated steps |
| Outcome/value | disposition, actual minutes, minutes saved, feedback, correlation id | Reviewed work and measured value | Guaranteed future ROI |

## 10. Source/Rustdoc/test evidence links

Required content:

- Link source paths that back the page. Prefer current source/Rustdoc/test paths over stale architecture prose.
- Cite tests when they prove blocked side effects or outcome persistence.
- Do not invent rendered Rustdoc URLs if generated Rustdoc is not present; cite module/type paths and source files.
- Keep links local and relative.

Starter anchors most overlays should inspect:

- `../../../app/src/agents.rs` for `AgentSpec`, `WorkflowAgent`, `AgentPromptPacket`, and baseline forbidden actions/review gates.
- `../../../domain/src/policy.rs` for `ReviewGate`, denial reasons, and `domain::policy::automation::Level`.
- `../../../domain/src/workflow.rs` for workflow events, allowed actions, recommendations, results, risk flags, and verification notes.
- `../../../domain/src/source.rs` for `RecordRef`, `Provenance`, source systems, source snapshots, and data-quality promotion errors.
- `../../../storage/src/operations.rs` for outcome/labor evidence records and stored source refs.
- `../source-evidence-map.md`, `../review-boundaries-matrix.md`, and `../../design/entity-atlas-review-safety-boundaries.md` for shared safety navigation.

Add specialized anchors for the family, for example booking triage, daily update, service-line modules, payment/money modules, documents/vaccine/care modules, Gingr adapters, API tests, or operator workflow pages.

## 11. Open gaps or owner decisions

Required content:

- List missing source coverage, stale anchors, absent tests, missing operator pages, or decisions that need a product/ops/security owner.
- Use “gap” or “owner decision needed” language rather than turning guesses into facts.
- State the safest current behavior while the gap remains.

Recommended table:

| Gap or owner decision | Why it matters | Safest current behavior | Evidence needed to close |
| --- | --- | --- | --- |
| `<missing source/test/approval>` | `<risk or ambiguity>` | `<draft-only, route to review, fail closed, do not claim>` | `<source/Rustdoc/test/approval record>` |

## Final reviewer checklist

Before a child overlay is ready, a reviewer should be able to answer yes to all of these:

- Does the first section name a concrete labor or error cost for a pet-resort role?
- Can a non-coder route each listed entity/action to the right reviewer?
- Are “agent may read,” “agent may draft/recommend/rank/record,” and “agent must not do directly” separate?
- Are source evidence, human approval, draft creation, and outcome proof clearly different?
- Are blocked actions precise and entity-specific?
- Does every labor-value claim require outcome/audit evidence rather than intent?
- Are source/Rustdoc/test links current and local?
- Are gaps marked as gaps instead of being filled with assumptions?
