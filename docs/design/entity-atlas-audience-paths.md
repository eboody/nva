---
title: "Entity atlas audience paths"
slug: "entity-atlas-audience-paths"
status: "draft"
audience: ["operations-leader", "resort-manager", "it-integration", "compliance-safety", "product-customer-success", "docs-writer"]
plain_english_definition: "Audience-specific reading paths through the entity atlas so non-coders can start from their decision, read the right entities first, and trace labor value, review boundaries, authority, and code-derived evidence."
primary_labor_problem: "Reduces evaluation and handoff time by making each audience's source facts, workflow packets, review gates, blocked actions, outcome records, and proof links visible without replacing the entity index."
---

# Entity atlas audience paths

Purpose: this page gives non-coder readers an audience-first route through the entity atlas. It does not replace the [entity inventory](entity-atlas-inventory.md), the [relationship map](entity-atlas-relationships.md), or the family pages. It tells each audience which entities to read first, what decision the path supports, what automation may draft or recommend, what remains human-reviewed, how labor-cost value is measured, and where source/Rustdoc/test evidence lives.

Repository maxim preserved: this repo is an entity-first operating model for lowering labor cost across NVA Pet Resorts. Each path starts with the business decision and then points into the entity atlas for proof.

## Shared reading rule for every audience

Use this order whenever a path names an entity:

1. Start with the plain-language family page or operator workflow page.
2. Confirm how the entity relates to source facts, workflow packets, review gates, blocked actions, outcome records, storage records, and runtime shells in the [relationship map](entity-atlas-relationships.md).
3. Use the inventory row for source paths and Rustdoc/module/type names.
4. Treat Rustdoc/source/tests as evidence after the business explanation, not as the first thing a non-coder must decode.

Automation boundary common to every path: agents may summarize, rank, draft, recommend, validate, and record review evidence only inside app-owned workflow contracts. Customer sends, provider/PMS writes, schedule/capacity changes, payment/refund/discount movement, hidden source cleanup, medical/safety approvals, incident decisions, vaccine acceptance, policy exceptions, and labor/staffing mandates remain blocked unless a deterministic app-owned approval contract and human review explicitly authorize them.

## Path 1: Operations leaders

Primary question: will this reduce manager/front-desk labor across resorts without creating operational risk?

Read these entities first:

1. [Operations service offering / portfolio](entity-atlas-outcomes-operations-money.md) to understand the NVA operating context by service line, site, technology ecosystem, and operating function.
2. [Manager daily brief packet and outcome record](entity-atlas-outcomes-operations-money.md) because it is the clearest labor-loop example: ranked actions, source evidence, removed manual work, estimates, and actual outcome capture.
3. [Data quality issue and data-quality hygiene packet](source-provenance-data-quality-atlas.md) because source cleanup reduces repeated rework across every downstream workflow.
4. [Review gates, blocked actions, and approval records](entity-atlas-review-safety-boundaries.md) to verify that labor reduction is not being bought by unsafe automation.
5. [Storage operations boundary and outcome storage records](entity-atlas-runtime-storage-api-surfaces.md) to see how evidence becomes reportable proof.

Decisions this path supports:

- Which labor loops are strongest for local/demo or sandbox continuation.
- Which work should be measured first: morning dashboard reconciliation, source-data cleanup, retention prioritization, checkout exceptions, or draft-writing time.
- Which outcomes need real NVA pilot metrics before any live labor-savings claim is credible.

Automation may draft or recommend:

- Ranked manager actions, internal cleanup queues, source-grounded summaries, labor-minute estimates, and draft disposition/outcome records.

Human-reviewed or blocked:

- Staffing mandates, payroll/timeclock edits, schedule/capacity changes, customer/provider side effects, discounts/refunds/payments, safety exceptions, policy overrides, and any regional rollup that would be treated as an operational directive rather than review evidence.

Labor-cost value measurement:

- Estimated vs actual labor minutes, removed-manual-work category, actor/persona, reporting group, reviewed disposition, source refs, wrong-source findings, completed/deferred/suppressed outcomes, and storage outcome records.

Evidence to inspect after the atlas page:

- Design docs: [labor-cost reduction crosswalk](labor-cost-reduction-crosswalk.md), [manager daily brief measurable labor loop](manager-daily-brief-measurable-labor-loop.md), [data-quality hygiene labor loop](data-quality-hygiene-labor-loop.md).
- Operator/local proof: [manager brief local smoke](../ops/manager-daily-brief-local-smoke.md), [data-quality hygiene local smoke](../ops/data-quality-hygiene-local-smoke.md).
- Source/Rustdoc/test surfaces: `app::manager_daily_brief`, `app::data_quality_hygiene`, `domain::daily_brief`, `domain::operations`, `domain::analytics`, `storage::operations`, and the relevant app/storage/API tests named from those docs.

## Path 2: Resort managers and front-desk leaders

Primary question: what can my team use as a safer daily queue, and what still needs a manager or trained staff member?

Read these entities first:

1. [Reservation and reservation status](entity-atlas-petsuites-core-entities.md) to understand booking/stay lifecycle facts before triage or checkout.
2. [Pet, customer, care profile, vaccine, document, temperament, and incident entities](entity-atlas-petsuites-core-entities.md) to understand the safety/customer facts that drive review.
3. [Booking triage packet](entity-atlas-workflow-packets-agents.md), [checkout completion packet](entity-atlas-workflow-packets-agents.md), [daily update / Pawgress draft packet](entity-atlas-workflow-packets-agents.md), and [CRM retention / grooming rebooking packet](entity-atlas-workflow-packets-agents.md) to see the review queues staff touch.
4. [Boarding, daycare, grooming, training, and retail contract entities](entity-atlas-petsuites-core-entities.md) plus [revenue opportunity entities](entity-atlas-revenue-opportunity-entities.md) to connect service-line policy to daily work.
5. [Review gates and blocked actions](entity-atlas-review-safety-boundaries.md) to see when the queue must stop for a manager, care reviewer, approved sender, payment reviewer, or integration owner.

Decisions this path supports:

- Which queue item can be handled by front desk, manager, care/medical reviewer, approved message sender, or another role.
- Whether a booking, checkout, customer update, grooming follow-up, or data cleanup item has enough source evidence for review.
- Which missing facts should become data-quality hygiene work instead of hidden assumptions.

Automation may draft or recommend:

- Booking readiness summaries, checkout exception summaries, customer-message drafts, daily update drafts, retention/rebooking opportunities, internal staff tasks, omitted-fact lists, and audit-event drafts.

Human-reviewed or blocked:

- Booking confirmation/rejection, check-in/out/status changes, customer sends, medical/vaccine/temperament/incident approvals, daycare group-play approval, room/capacity/yard/schedule assignment, payment/refund/discount/deposit movement, and policy exceptions.

Labor-cost value measurement:

- Minutes saved from fewer source lookups, fewer rewritten drafts, faster handoffs, fewer ambiguous checkout/booking loops, completed/deferred/suppressed staff dispositions, wrong-source cleanup, and actual minutes recorded by workflow outcome records.

Evidence to inspect after the atlas page:

- Operator pages: [operator workflow index](../workflows/operator/README.md), [manager daily brief](../workflows/operator/manager-daily-brief.md), [booking triage](../workflows/operator/booking-triage.md), [checkout completion](../workflows/operator/checkout-completion.md), [grooming rebooking / retention](../workflows/operator/grooming-rebooking-retention.md), [daily updates / Pawgress drafts](../workflows/operator/daily-updates-pawgress-drafts.md), [data-quality hygiene](../workflows/operator/data-quality-hygiene.md).
- Source/Rustdoc/test surfaces: `domain::entities`, `domain::reservation`, `domain::care`, `domain::vaccine`, `domain::document`, `domain::temperament`, `domain::incident`, service-line modules under `domain::*`, and workflow modules under `app::*`.

## Path 3: IT and integration readers

Primary question: how does the repo treat source systems such as Gingr without pretending provider records are canonical truth or mutating live systems?

Read these entities first:

1. [Source system, source refs, provenance, raw payload refs, and payload hashes](source-provenance-data-quality-atlas.md) to understand how source evidence is traced.
2. [Gingr provider boundary, endpoint requests, responses, DTOs, mapping candidates, webhook verified events, transport seam, and provider-surface gap markers](../integrations/gingr/provider-boundary-atlas.md) to understand the adapter boundary.
3. [Data-quality issue, field path, and mapping error/provider field](source-provenance-data-quality-atlas.md) to understand how missing, stale, duplicate, invalid, sensitive, or ambiguous facts are preserved.
4. [Runtime, storage, API, worker, CLI, and contract-test surfaces](entity-atlas-runtime-storage-api-surfaces.md) to see how contracts are exposed locally without changing provider systems.
5. [Workflow packets and tool ports](entity-atlas-workflow-packets-agents.md) to see what app workflows request from stores, portals, messaging, payment, documents, media, and agent runtimes.

Decisions this path supports:

- Which provider fields/endpoints are currently modeled, fixture-safe, or gaps.
- Which source facts may feed a workflow packet and which must remain hygiene work.
- Which runtime shells are evidence/demo surfaces versus live provider integration.
- Which read-model, credential, identity, approval, audit retention, monitoring, rollback, and staff-review surfaces are pilot prerequisites.

Automation may draft or recommend:

- Source-normalization summaries, mapping candidates, provider gap notes, data-quality cleanup candidates, redacted evidence summaries, and integration-review tasks.

Human-reviewed or blocked:

- Live provider/PMS writes, record hiding/deletion, source ambiguity auto-resolution, credential exposure, unsupported endpoint assumptions, sensitive raw-payload exposure, and any production read/write claims not backed by a pilot contract.

Labor-cost value measurement:

- Reconciliation minutes avoided, repeated source lookup reduction, wrong-source findings, stale/duplicate/conflicting record counts, resolved hygiene outcomes, and fewer engineering/operator handoffs due to typed endpoint/mapping contracts.

Evidence to inspect after the atlas page:

- Integration docs: [Gingr README](../../integrations/gingr/README.md), [Gingr docs index](../integrations/gingr/README.md), [source inventory](../integrations/gingr/source-inventory.md), [BI read-model contract](../integrations/gingr/bi-read-model-contract.md), [adapter boundary and labor-source expansion](../integrations/gingr/adapter-boundary-and-labor-source-expansion.md).
- Source/Rustdoc/test surfaces: `gingr::endpoint`, `gingr::transport`, `gingr::response`, `gingr::webhook`, `gingr::dto`, `gingr::mapping`, `domain::source`, `domain::data_quality`, `storage::operations`, API/worker runtime modules, and integration/webhook/storage tests.

## Path 4: Compliance and safety reviewers

Primary question: what can agents not do, who reviews each risky action, and where is evidence preserved?

Read these entities first:

1. [Review gate, blocked action, policy rule, automation level, approval record, and audit event](entity-atlas-review-safety-boundaries.md) because these are the safety vocabulary.
2. [Source/provenance and data-quality entities](source-provenance-data-quality-atlas.md) because safety depends on visible source uncertainty.
3. [Pet, care, medication, vaccine, document, temperament, incident, and message entities](entity-atlas-petsuites-core-entities.md) because these hold customer, care, safety, and communication risk.
4. [Workflow packets, agent specs, prompt packets, safe agent actions, and tool ports](entity-atlas-workflow-packets-agents.md) to see how agent behavior is constrained.
5. [Outcome, audit, storage, and runtime surfaces](entity-atlas-outcomes-operations-money.md) plus [runtime/storage/API surfaces](entity-atlas-runtime-storage-api-surfaces.md) to see how reviewed decisions become evidence.

Decisions this path supports:

- Whether a workflow is draft-only, internal-task-only, approval-gated, disabled, or ready only for local/sandbox continuation.
- Which human role owns approval for customer messaging, provider/PMS mutation, safety/medical facts, money movement, source cleanup, data exposure, and labor claims.
- Whether an entity page preserves ambiguity and blocked actions in plain language.

Automation may draft or recommend:

- Evidence summaries, review packets, omitted-fact lists, internal task drafts, audit-event drafts, and validation failures for missing source refs or wrong review gates.

Human-reviewed or blocked:

- All live customer/member communication; provider/PMS writes; schedule/capacity/staffing changes; payment/refund/discount/deposit movement; source hiding/deletion; sensitive PII/payment payload exposure; vaccine, medical, medication, temperament, group-play, incident, or policy-exception approval.

Labor-cost value measurement:

- Only reviewed outcomes count. Compliance/safety should look for actual labor minutes, reviewed disposition, approval record, audit event, source refs/provenance, omitted facts, wrong-source findings, and blocked-action reasons before accepting a labor-savings claim.

Evidence to inspect after the atlas page:

- Safety docs: [source evidence map](../safety/source-evidence-map.md), [operator safety model](../safety/agent-safety-model-for-operators.md), [review boundaries matrix](../safety/review-boundaries-matrix.md), [evidence/policy/blocked actions/outcomes](../safety/evidence-policy-blocked-actions-outcomes.md), [labor-cost with human review](../safety/labor-cost-with-human-review-crosswalk.md).
- Source/Rustdoc/test surfaces: `domain::policy`, `domain::workflow`, `domain::source`, `domain::data_quality`, `domain::audit`, `domain::entities::approval`, `domain::message`, `domain::document`, `domain::vaccine`, `domain::incident`, `app::agents`, `app::tools`, and workflow-local `BlockedAction`/`SafeAgentAction` contracts.

## Path 5: Product and customer-success readers

Primary question: which buyer/user story can we explain without overpromising live automation, and what value proof should customer-success ask for?

Read these entities first:

1. [Operations service offering / portfolio](entity-atlas-outcomes-operations-money.md) to frame multi-site NVA value by service line, operating function, and technology ecosystem.
2. [Workflow packets, agents, drafts, and review queues](entity-atlas-workflow-packets-agents.md) to explain product capabilities as reviewable queues and drafts rather than autonomous chatbot actions.
3. [Customer, pet, reservation, message, care, and document entities](entity-atlas-petsuites-core-entities.md) to understand customer-facing stories and their safety limits.
4. [Revenue opportunity entities](entity-atlas-revenue-opportunity-entities.md) for grooming rebooking, training, retail, retention, and product/inventory follow-up opportunities.
5. [Outcomes, labor, analytics, money, and safety evidence](entity-atlas-outcomes-operations-money.md) to connect value stories to measurable outcome records.
6. [Review gates and blocked actions](entity-atlas-review-safety-boundaries.md) to keep demos and customer conversations honest.

Decisions this path supports:

- Which product stories are strongest now: Manager Daily Brief, Data-Quality Hygiene, retention/grooming rebooking drafts, daily update drafts, and source-system hygiene.
- Which proof a customer-success/pilot plan should capture: actual minutes, reviewed dispositions, wrong-source findings, conversion or suppression outcomes, and staff role feedback.
- Which capabilities are sandbox/demo posture versus pilot prerequisites or live blockers.

Automation may draft or recommend:

- Demo narratives, reviewable customer-update drafts, retention/rebooking draft opportunities, product/retail review items, manager brief actions, data-quality cleanup items, and outcome summaries.

Human-reviewed or blocked:

- Customer sends, marketing/retention outreach, offers/discounts, appointment booking, payment movement, service promises, safety/medical conclusions, and any claim of realized labor savings without reviewed outcome evidence.

Labor-cost value measurement:

- Minutes saved per role, avoided rework, reduced handle time, fewer source lookups, completed/deferred/suppressed/wrong-source outcomes, customer-message review disposition, retention/rebooking conversion signals, and data-quality cleanup outcomes.

Evidence to inspect after the atlas page:

- Public/readiness docs: [NVA pet-resorts AI context](../../nva-pet-resorts-ai-context.md), [labor-cost reduction crosswalk](labor-cost-reduction-crosswalk.md), [labor-cost platform readiness audit](../audits/2026-06-18-labor-cost-platform-readiness.md), [public docs landing content map](public-docs-landing-content-map.md).
- Source/Rustdoc/test surfaces: `app::manager_daily_brief`, `app::data_quality_hygiene`, `app::crm_retention`, `app::daily_update`, `domain::message`, `domain::retail`, `domain::grooming`, `domain::training`, `domain::operations`, `domain::analytics`, and `storage::operations` outcome records.

## Cross-audience quick matrix

| Audience | First entity family | Decision they can make | Must stay human-reviewed | Labor value evidence |
| --- | --- | --- | --- | --- |
| Operations leaders | Outcomes/labor/operations + manager brief | Prioritize safest labor loops and pilot metrics | Staffing, money, schedule, customer/provider side effects, safety exceptions | Actual minutes, reporting group, completed/deferred/suppressed/wrong-source outcomes |
| Resort managers | Reservation/customer/pet + workflow packets | Route daily queue items to the right staff role | Booking/status changes, customer sends, care/safety approvals, payments, policy exceptions | Handoff time, draft-writing time, fewer source lookups, reviewed dispositions |
| IT/integration | Source/provenance + Gingr/provider boundary | Judge source-system coverage, gaps, and pilot prerequisites | Provider/PMS writes, credential exposure, hidden source cleanup, unsupported endpoint assumptions | Reconciliation minutes, hygiene outcomes, wrong-source counts, fewer handoffs |
| Compliance/safety | Review gates/blocked actions + audit/source evidence | Accept or reject workflow safety posture | All live customer/provider/money/safety side effects without approval contracts | Review evidence, audit records, source refs, blocked-action reasons, actual minutes only after review |
| Product/customer success | Workflow packets + revenue/outcome entities | Tell honest product stories and define success proof | Customer outreach/offers/service promises and live labor-saving claims | Role minutes, avoided rework, conversion/suppression outcomes, customer-message dispositions |

## Use in public landing and root README

The public landing page should expose these as a "choose your path" block above Rustdoc links. The root README should link this page immediately after the entity navigation map so non-coders can start by role before dropping into the full inventory.

Do not duplicate every entity row on the landing page or README. Use this page as the audience bridge, then link to the family atlas pages, inventory, and relationship map for depth.
