# Public docs landing content map

Purpose: keep `https://nva.eman.network/` aligned with the entity atlas as the public front door instead of a generated Rust crate index. The landing page should let NVA Pet Resorts operators, IT/product evaluators, safety/compliance reviewers, Gingr/integration owners, developers, and non-coders start from a recognizable entity, role, workflow, boundary, or proof need.

This is a content and information-architecture brief for the checked-in landing page at `docs/public/index.html`. The landing page is implemented separately from generated Rustdoc output; Rustdoc remains the evidence layer behind the entity-first narrative.

## Source inputs and positioning constraints

Primary inputs:

- `nva-pet-resorts-ai-context.md`: NVA Pet Resorts is a 170-site, multi-brand pet-resort operating context where the valuable AI work is labor efficiency, customer communication load reduction, source-system/data reconciliation, sales/retention support, and safe workflow integration rather than generic chatbots.
- `docs/design/labor-cost-reduction-crosswalk.md`: the operating thesis is that labor cost falls when managers and front-desk teams stop rediscovering source facts, reconciling dashboards by hand, rewriting repetitive drafts, and reworking ambiguous records. The deterministic app owns facts, workflow, review, audit, policy, outcome capture, and side effects; agents consume context packets and submit reviewable drafts.
- Existing README/Rustdoc surfaces: current repo entry points are maintainer- and developer-oriented. They explain crate/module ownership well, but a public visitor currently has to start from `domain`, `app`, `storage`, and `integrations/gingr` crate names rather than the business problems those crates support.

Public landing page constraints:

- Lead with the entity atlas and the README maxim: entity-first operating model for labor-cost reduction.
- Let a non-coder start from an entity before seeing crate/module names.
- Route immediately to the canonical [entity index](entity-index.md), [audience paths](entity-atlas-audience-paths.md), [workflow-to-entity navigation](workflow-to-entity-navigation-map.md), safety/human-review boundaries, and source/Rustdoc/test evidence.
- Present agents as review-gated helpers behind deterministic app contracts, not as autonomous operators.
- Say what is strongest now: Manager Daily Brief, Data-Quality Hygiene, CRM/retention and daily-update review-gated draft loops, and Gingr/source-system integration posture.
- Distinguish ready/local-demo/sandbox continuation from pilot/live blockers.
- Use Rustdocs as code-derived evidence after the business/entity explanation, not as the first thing a non-coder sees.
- Link the entity-first [contract crosswalk schema](../entity-atlas/contract-crosswalk/crosswalk-schema.md), [surface inventory](../entity-atlas/contract-crosswalk/surface-inventory.md), [source/provider flows](../entity-atlas/contract-crosswalk/source-provider-flows.md), [workflow packets](../entity-atlas/contract-crosswalk/workflow-packets.md), [storage/persistence](../entity-atlas/contract-crosswalk/storage-persistence.md), and [runtime exposure](../entity-atlas/contract-crosswalk/runtime-exposure.md) before sending non-coders into crate indexes.

## Landing-page narrative hierarchy

The landing page should use this order.

### 1. Hero: entity atlas as the front door

Business/entity-first message:

> Start with the pet-resort entity atlas: an entity-first operating model for labor-cost reduction.

Explain the maxim before naming Rust, crates, or modules:

- A non-coder should be able to pick an important thing — source fact, pet, reservation, review gate, workflow packet, outcome record, Gingr boundary, or runtime surface — and understand what it is, why it exists, which workflows feature it, who is authoritative, what automation may draft, what stays human-reviewed, how value is measured, and where proof lives.
- 170-site pet-resort operations create repeated manager/front-desk work across boarding, daycare, grooming, training, retail, customer communication, source-data hygiene, and reporting; named, traceable entities reduce rediscovery and rework.
- The useful AI pattern is not a free-form chatbot. It is source-grounded workflow automation with review gates and measured outcomes.

Primary CTA labels:

- "Browse entity index"
- "Choose a reading path"
- "Audience paths"
- "Workflow-to-entity map"
- "Review safety boundaries"
- "Inspect evidence"

Avoid as first-screen language:

- "Rust workspace"
- "domain/app/storage crate index"
- "generated documentation"
- "modular architecture"
- Workflow-only framing that makes the entity atlas secondary

Those can appear after the entity-first business frame.

### 2. Front-door routes

Explain in non-coder language first:

> Choose the route that matches what you recognize: an entity, an audience decision, a workflow loop, a safety boundary, or a proof/evidence question.

The landing page should route to:

| Entry need | Canonical route |
| --- | --- |
| Pick an important business thing | [NVA Pet Resorts entity index](entity-index.md) |
| Understand how facts connect | [Entity atlas relationship map](entity-atlas-relationships.md) |
| Read by role or decision | [Entity atlas audience paths](entity-atlas-audience-paths.md) |
| Arrive through a labor loop | [Workflow-to-entity navigation map](workflow-to-entity-navigation-map.md) and operator workflow pages |
| Check human-review boundaries | safety/review-boundary pages and review-gate/blocked-action entities |
| Demand source/Rustdoc/test proof | contract crosswalks and evidence anchor map |

### 3. What this repo is

Explain in non-coder language after the atlas route is clear:

> This repo is a safe automation foundation for pet-resort operations. It models operational facts, source evidence, review gates, draft workflows, storage/audit records, and provider integration boundaries so AI agents can help staff without owning business truth.

Then map the technical terms:

| Business explanation | Technical surface |
| --- | --- |
| Pet-resort facts and rules | `domain` crate |
| Workflow packets, draft validation, and agent-safe actions | `app` crate |
| Outcome evidence and normalized stored projections | `storage` crate |
| Gingr/source-system request, DTO, webhook, and mapping boundaries | `integrations/gingr` crate |
| Local HTTP/worker/CLI shells for demos and future runtime surfaces | `apps/api`, `apps/worker`, `apps/cli` |

Keep this section short. The landing page should not become a replacement for the README or Rustdocs.

### 4. Strongest current workflows

Present workflows as labor loops, not modules.

#### Manager Daily Brief

Audience value:

- Gives a GM/AGM a prioritized review queue instead of asking them to manually reconcile demand, staffing, checkout, retention, and data-quality signals each morning.
- Current contract includes source-grounded actions, removed-manual-work categories, review gates, blocked live side effects, labor-minute estimates, and outcome capture.

Current status copy:

> Strongest current loop. The repo can locally demonstrate a source-grounded manager brief, reviewable agent draft, deterministic validation, and reviewed outcome capture for labor minutes.

#### Data-Quality Hygiene

Audience value:

- Turns missing/stale/ambiguous source facts into visible internal cleanup work instead of letting every downstream workflow rediscover the same uncertainty.
- Good second workflow because it improves all later automation and stays internal/review-gated.

Current status copy:

> Internal hygiene loop with local/sandbox-ready contracts. It ranks source issues, preserves ambiguity for review, blocks provider/PMS writes, and captures reconciliation minutes avoided.

#### CRM / retention / grooming rebooking draft loops

Audience value:

- Helps front-desk teams identify safe, consented follow-up opportunities and draft staff-reviewed outreach.
- Useful for retention and grooming cadence, but closer to customer messaging and therefore behind stricter review gates.

Current status copy:

> Draft/review posture exists in app contracts. Live customer sends, offers, discounts, payment movement, and schedule/provider mutation are not allowed without app-owned approval and human review.

#### Daily updates / Pawgress-style drafts

Audience value:

- Converts terse staff care notes into warmer customer-facing draft updates while preserving omitted facts, internal flags, and approval requirements.

Current status copy:

> Draft-only customer communication pattern exists. Sensitive or negative facts stay internal or require review; no unreviewed customer send is authorized.

#### Gingr / source-system integration posture

Audience value:

- Shows how a likely core operating system is treated as source evidence rather than blindly trusted domain truth.
- Typed endpoint builders, DTOs, response wrappers, webhook verification, mapping candidates, redaction, and provider-surface gap documentation reduce ad hoc source inspection.

Current status copy:

> Gingr is modeled as a provider boundary. The repo has typed request/response/mapping surfaces and fixture-safe contracts, but live HTTP support and universal NVA Gingr coverage remain discovery/pilot questions.

### 5. Ready vs planned vs blocked

Use a status band that prevents overclaiming.

| Stage | Landing-page wording | Evidence to link |
| --- | --- | --- |
| Local/demo ready | "The repo can demonstrate source-grounded context -> reviewable draft -> deterministic validation -> reviewed outcome capture with fake/local data." | Manager Daily Brief docs, Data-Quality Hygiene smoke, API tests, Rustdocs |
| Sandbox continuation ready | "The contract pattern is ready for more internal, read-only, fixture-safe, or sandbox workflows." | Agent/app infrastructure guide, labor crosswalk, data-quality hygiene design |
| Pilot not ready | "Pilot work still needs real source/read-model access, identity/approval records, durable audit retention, monitoring, rollback, operator metric definitions, and staff review UI." | Labor-cost platform readiness memo |
| Live/member-facing blocked | "No live customer sends, provider/PMS writes, schedule changes, payment/refund/discount movement, hidden ambiguity resolution, or safety-sensitive decisions are authorized by current repo contracts." | Safety/compliance path and readiness memo |

### 6. Rustdocs as evidence

Only after the business and workflow explanation should the landing page introduce Rustdocs:

> The Rustdocs are code-derived evidence for reviewers and developers. They are not the best starting point for a non-coder, but they prove whether the repo's safety boundaries, workflow packets, source references, storage records, and integration contracts are actually represented in compiled source.

Rustdoc framing:

- Generated Rustdocs should be positioned as evidence, not marketing copy.
- README/docs pages explain the business narrative; Rustdocs prove code-derived contracts.
- Link to representative crate/module pages by workflow and audience, not by forcing users to start with crate names.

## Audience paths

Canonical detailed artifact: [Entity atlas audience paths](entity-atlas-audience-paths.md). The landing page should summarize those role-specific routes and link to the full guide rather than duplicating every entity row. Each path must name the entities to read first, the decision the audience can make, the human-reviewed/blocked boundary, labor-cost value evidence, and source/Rustdoc/test proof.

### 1. Operations leader

Likely question:

> Will this reduce manager/front-desk work across resorts without creating operational risk?

Landing path:

1. Hero labor-cost thesis.
2. Manager Daily Brief workflow.
3. Data-Quality Hygiene workflow.
4. Ready vs planned status.
5. Outcome/labor-minute evidence docs.

Needs to understand:

- What repetitive work is removed: morning dashboard reconciliation, checkout exception audits, retention prioritization, repeated source-data checks, and draft writing.
- What remains human-owned: staffing decisions, customer sends, refunds/discounts/payments, safety/incident decisions, provider/PMS writes.
- How evidence is measured: estimated vs actual minutes, actor/persona, source refs, completed/deferred/suppressed/wrong-source outcomes.

Recommended links:

- `docs/design/labor-cost-reduction-crosswalk.md`
- `docs/design/manager-daily-brief-measurable-labor-loop.md`
- `docs/design/data-quality-hygiene-labor-loop.md`
- `docs/ops/manager-daily-brief-local-smoke.md`
- `docs/ops/data-quality-hygiene-local-smoke.md`
- Rustdoc evidence after explanation: `app::manager_daily_brief`, `app::data_quality_hygiene`, `storage::operations::ManagerDailyBriefOutcomeRecord`

### 2. AI program evaluator

Likely question:

> Is this a reusable, governable way to use enterprise Claude/agents inside operations?

Landing path:

1. Labor-cost thesis.
2. Deterministic app owns facts/policy/review/audit/side effects.
3. Agent/app infrastructure pattern.
4. Workflow examples.
5. Ready vs blocked status.
6. Rustdocs for typed packet/draft/evaluation contracts.

Needs to understand:

- Agents consume typed app context packets; they do not browse raw databases or invent operational facts.
- App validation rejects missing source refs, wrong review gates, stale context, unsupported action kinds, and blocked side effects.
- Outcome capture makes labor-savings claims measurable rather than anecdotal.
- HTTP tool bridge first, possible MCP later, after contracts settle.

Recommended links:

- `docs/architecture/agent-app-infrastructure.md`
- `docs/audits/2026-06-18-agent-app-infrastructure-readiness.md`
- `docs/audits/2026-06-18-labor-cost-platform-readiness.md`
- `README.md#documentation-contracts`
- Rustdoc evidence: `app::agents`, `app::manager_daily_brief`, `app::data_quality_hygiene`, `app::tools`, `pet_resort_api::http`

### 3. Safety/compliance reviewer

Likely question:

> What can the agent not do, and where are review gates/audit records enforced?

Landing path:

1. Safety promise near the top, not buried.
2. Explicit blocked side effects.
3. Review-gated workflows.
4. Audit/replay/outcome capture.
5. Rustdocs for policy/source/audit/workflow contracts.

Needs to understand:

- Generated Rustdocs are code-derived evidence, but they do not imply production readiness.
- Agents are review-gated and draft-only unless an app-owned approval contract exists.
- No live member/customer/provider side effects are currently authorized by the public docs or repo contracts.
- Source ambiguity is preserved and routed to review; it is not hidden by agent prose.

Recommended links:

- `docs/architecture/agent-app-infrastructure.md#trust-boundary-and-threat-model`
- `docs/design/labor-cost-reduction-crosswalk.md#source-of-record-questions-to-keep-explicit`
- `docs/audits/2026-06-18-labor-cost-platform-readiness.md#live-readiness-blocked`
- `docs/security/pet-resort-security-audit.md`
- `docs/architecture/agent-permissions-by-workflow.md`
- Rustdoc evidence: `domain::policy`, `domain::workflow`, `domain::source`, `domain::data_quality`, `domain::audit`, `app::tools`

### 4. Gingr / integration owner

Likely question:

> How does this treat Gingr/source systems without over-assuming authority or mutating live provider records?

Landing path:

1. Gingr/source-system posture summary.
2. Provider facts vs domain truth.
3. Source refs and data-quality issue preservation.
4. Integration docs and provider-surface gaps.
5. Rustdocs for endpoint/DTO/mapping/webhook/transport surfaces.

Needs to understand:

- Public materials show Gingr in the PetSuites portal, but universal use across all 170 resorts remains a source-of-record question.
- Gingr provider ids, statuses, DTOs, and endpoint quirks are evidence, not canonical domain truth.
- The current crate has request builders, response wrappers, DTO and mapping surfaces, webhook verification, mock transport, redaction, and documented gaps.
- Live HTTP support and provider/PMS mutation are not claimed in the current slice.

Recommended links:

- `integrations/gingr/README.md`
- `docs/integrations/gingr/README.md`
- `docs/integrations/gingr/source-inventory.md`
- `docs/integrations/gingr/adapter-boundary-and-labor-source-expansion.md`
- `docs/integrations/gingr/bi-read-model-contract.md`
- Rustdoc evidence: `gingr::endpoint`, `gingr::transport`, `gingr::response`, `gingr::webhook`, `gingr::dto`, `gingr::mapping`, `domain::source`

### 5. Developer

Likely question:

> Where do I inspect or change the contracts without breaking the safety model?

Landing path:

1. Business/labor thesis.
2. Current workflow map.
3. Architecture boundary map.
4. Crate/module map.
5. Verification commands.
6. Rustdocs.

Needs to understand:

- `domain` owns semantic truth.
- `app` owns workflow packets, draft validation, safe/blocked actions, and outcome records before shell/integration code acts.
- `storage` owns persisted projections and explicit conversion from/to domain types.
- `integrations/gingr` owns provider boundaries and mapping into semantic candidates.
- Runtime shells stay thin and do not invent policy.
- README/docs are navigation; Rustdoc/doctests are executable API contracts.

Recommended links:

- `README.md`
- `domain/README.md`
- `app/README.md`
- `storage/README.md`
- `integrations/gingr/README.md`
- `apps/api/README.md`
- `docs/plans/rustdoc-external-source-of-truth.md`
- Rustdoc evidence: crate roots for `domain`, `app`, `storage`, `gingr`, `pet_resort_api`, `pet_resort_worker`, and `pet_resort_cli`

## Public link plan

The implemented landing page should group links by audience/problem, not by crate names.

### Primary navigation groups

1. `Why this exists: reduce pet-resort labor cost`
   - `nva-pet-resorts-ai-context.md`
   - `docs/design/labor-cost-reduction-crosswalk.md`
   - `docs/audits/2026-06-18-labor-cost-platform-readiness.md`

2. `Start with proven workflow loops`
   - `docs/design/manager-daily-brief-measurable-labor-loop.md`
   - `docs/ops/manager-daily-brief-local-smoke.md`
   - `docs/design/data-quality-hygiene-labor-loop.md`
   - `docs/ops/data-quality-hygiene-local-smoke.md`
   - Rustdoc: `app::manager_daily_brief`, `app::data_quality_hygiene`, `storage::operations::*OutcomeRecord`

3. `Understand safety and agent boundaries`
   - `docs/architecture/agent-app-infrastructure.md`
   - `docs/security/pet-resort-security-audit.md`
   - `docs/architecture/agent-permissions-by-workflow.md`
   - Rustdoc: `domain::policy`, `domain::workflow`, `domain::audit`, `app::tools`

4. `Understand source systems and Gingr posture`
   - `integrations/gingr/README.md`
   - `docs/integrations/gingr/README.md`
   - `docs/integrations/gingr/source-inventory.md`
   - `docs/integrations/gingr/bi-read-model-contract.md`
   - Rustdoc: `gingr::endpoint`, `gingr::transport`, `gingr::webhook`, `gingr::mapping`, `domain::source`

5. `Inspect code-derived evidence`
   - Rustdoc crate roots: `domain`, `app`, `storage`, `gingr`, `pet_resort_api`, `pet_resort_worker`, `pet_resort_cli`
   - Representative Rustdoc pages by concept:
     - Source facts: `domain::source`, `domain::analytics`
     - Review gates: `domain::policy`, `domain::workflow`
     - Manager brief: `app::manager_daily_brief`, `domain::daily_brief`, `storage::operations::ManagerDailyBriefOutcomeRecord`
     - Data quality: `domain::data_quality`, `app::data_quality_hygiene`
     - Customer drafts: `app::daily_update`, `app::crm_retention`, `domain::message`
     - Gingr boundary: `gingr::endpoint`, `gingr::dto`, `gingr::mapping`, `gingr::webhook`

6. `Translate repo/Rust terms into pet-resort operations`
   - Glossary index: [`../glossary.md`](../glossary.md)
   - Architecture terms near crate links: [`domain`](../glossary-architecture-terms.md#domain), [`app`](../glossary-architecture-terms.md#app), [`storage`](../glossary-architecture-terms.md#storage), [`integrations/gingr`](../glossary-architecture-terms.md#integration-integrationsgingr), [`DTO`](../glossary-architecture-terms.md#dto), [`tool port`](../glossary-architecture-terms.md#tool-port-apptools), and [`read model`](../glossary-architecture-terms.md#read-model)
   - Source/evidence terms near Gingr and safety sections: [Gingr](../glossary-source-data-terms.md#domainsourcesystemgingr-gingr), [provider record](../glossary-source-data-terms.md#provider-record), [source-of-record](../glossary-source-data-terms.md#source-of-record), [data-quality issue](../glossary-source-data-terms.md#domaindata_qualityissue-data-quality-issue), [source ref](../glossary-architecture-terms.md#source-ref-domainsourcerecordref), and [provenance](../glossary-architecture-terms.md#provenance-domainsourceprovenance)
   - Workflow/safety terms near workflow cards: [draft](../glossary-workflow-state-terms.md#draft), [review gate](../glossary-workflow-state-terms.md#review-gate), [blocked action](../glossary-workflow-state-terms.md#blocked-action), [workflow packet](../glossary-workflow-state-terms.md#workflow-packet), [agent spec](../glossary-workflow-state-terms.md#agent-spec), and [outcome capture](../glossary-workflow-state-terms.md#outcome-capture)

### URL shape guidance for implementation

If the current public site is generated by `cargo doc`, link Rustdoc evidence as generated pages under `https://nva.eman.network/`, for example:

- `https://nva.eman.network/app/manager_daily_brief/index.html`
- `https://nva.eman.network/app/data_quality_hygiene/index.html`
- `https://nva.eman.network/domain/policy/index.html`
- `https://nva.eman.network/domain/source/index.html`
- `https://nva.eman.network/domain/data_quality/index.html`
- `https://nva.eman.network/storage/operations/index.html`
- `https://nva.eman.network/gingr/endpoint/index.html`
- `https://nva.eman.network/gingr/mapping/index.html`

The implementation card should verify exact generated paths after running the docs build, because package and crate names can differ for binary/runtime crates. If a custom landing page sits at `/`, keep generated Rustdoc available as the evidence layer rather than replacing it.

## Copy blocks for implementation

### Hero block

Headline:

> Safe AI workflows for reducing pet-resort labor cost

Subheadline:

> This repo demonstrates how a deterministic pet-resort app can give agents source-grounded context, validate reviewable drafts, block unsafe side effects, and capture labor-minute outcomes for managers and front-desk teams.

Body:

> The target is not a generic chatbot. The target is less time spent rediscovering source facts, reconciling dashboards manually, rewriting repetitive drafts, and reworking ambiguous records across boarding, daycare, grooming, training, customer communication, retention, and source-system hygiene.

CTA labels:

- See the labor-cost crosswalk
- Start with Manager Daily Brief
- Review safety boundaries
- Inspect Rustdocs evidence

### Labor thesis block

> Labor cost falls when managers and front-desk teams start from source-grounded review queues instead of manual scavenger hunts. The app owns source facts, workflow state, policy gates, audit trails, outcome capture, and every external side effect. Agents only summarize, rank, draft, and recommend from app-owned context packets.

### Workflow cards

Manager Daily Brief:

> A GM/AGM starts the day with ranked actions for demand-versus-staffing, checkout exceptions, retention follow-up, and [data-quality issues](../glossary-source-data-terms.md#domaindata_qualityissue-data-quality-issue). Each action carries source evidence, owner persona, [review gate](../glossary-workflow-state-terms.md#review-gate), [blocked actions](../glossary-workflow-state-terms.md#blocked-action), and [outcome capture](../glossary-workflow-state-terms.md#outcome-capture) for estimated/actual labor-minute tracking.

Data-Quality Hygiene:

> Missing or ambiguous source facts become visible cleanup work instead of repeated downstream rework. The workflow ranks internal hygiene tasks, preserves [source refs](../glossary-architecture-terms.md#source-ref-domainsourcerecordref) and [provenance](../glossary-architecture-terms.md#provenance-domainsourceprovenance) for review, rejects customer/provider [blocked actions](../glossary-workflow-state-terms.md#blocked-action), and measures manual reconciliation minutes avoided.

CRM / retention / grooming follow-up:

> The [app](../glossary-architecture-terms.md#app) can identify source-grounded, consent-aware opportunities and let agents [draft](../glossary-workflow-state-terms.md#draft) staff-reviewed follow-up language. It does not authorize live customer sends, discounts, payment movement, scheduling, or provider updates without explicit app-owned approval.

Daily updates / Pawgress-style drafts:

> Staff notes can become warmer customer-facing [draft](../glossary-workflow-state-terms.md#draft) updates with omitted-fact tracking, internal flags, and approval gates. Sensitive or negative facts stay [review-gated](../glossary-workflow-state-terms.md#review-gate).

Gingr/source-system posture:

> [Gingr](../glossary-source-data-terms.md#domainsourcesystemgingr-gingr) is treated as source evidence, not as unchecked [domain](../glossary-architecture-terms.md#domain) truth. Provider ids, endpoint shapes, webhooks, [DTOs](../glossary-architecture-terms.md#dto), mappings, redaction, and documented gaps are kept at the [integration boundary](../glossary-architecture-terms.md#integration-integrationsgingr) before [app](../glossary-architecture-terms.md#app) workflows consume normalized facts.

### Ready vs planned block

> Current state: local/demo and sandbox-continuation foundation. The repo can prove context -> draft -> validation -> reviewed outcome patterns with fake/local data. It should not claim live labor savings or production automation until real NVA source packets, identity/approval records, durable audit, monitoring, rollback, operator metrics, and staff review surfaces exist.

### Safety block

> Safety boundary: generated Rustdocs are code-derived evidence, not production authorization. Agents are review-gated and do not perform live member, customer, provider/PMS, schedule, payment, refund, discount, incident, vaccine, medical, safety, or personnel side effects without deterministic app-owned approval contracts and human review. Source ambiguity is preserved for review; it is never hidden by agent prose.

### Rustdocs evidence block

> The Rustdocs show the compiled contracts behind the narrative: [source refs](../glossary-architecture-terms.md#source-ref-domainsourcerecordref), [data-quality issues](../glossary-source-data-terms.md#domaindata_qualityissue-data-quality-issue), [review gates](../glossary-workflow-state-terms.md#review-gate), [workflow packets](../glossary-workflow-state-terms.md#workflow-packet), agent-safe actions, [blocked actions](../glossary-workflow-state-terms.md#blocked-action), [storage](../glossary-architecture-terms.md#storage) records, and [Gingr](../glossary-source-data-terms.md#domainsourcesystemgingr-gingr) provider boundaries. Start with the business workflow above, then use Rustdocs to verify the code-derived source of truth.

## Suggested page outline for implementation

1. Hero: labor-cost thesis and CTAs.
2. "What problem this solves" with 170-site operations, manager/front-desk load, dashboard/source rechecking, draft writing, and ambiguous records.
3. "How the safety model works" with deterministic app vs agent responsibilities.
4. "Current strongest workflows" cards.
5. "Who are you?" audience paths with links.
6. "Ready vs planned" status table.
7. "Rustdocs as evidence" with curated code-derived links.
8. "Developer map" collapsed/secondary section for crate names and verification commands.

## Terminology guide

Use business term first, technical term second.

| Prefer first | Then map to |
| --- | --- |
| Source facts from operating systems | `domain::source`, Gingr/provider refs, BI/read models |
| Reviewable manager/front-desk work queue | `app::*::Packet`, `BriefAction`, draft submissions |
| Human approval / staff review | `domain::policy::ReviewGate`, review disposition/outcome records |
| Labor-minute evidence | `OutcomeRecord`, storage projections |
| Unsafe live side effects | blocked action enums, app tool ports, provider/PMS/payment/message boundaries |
| Gingr/source-system evidence | `integrations/gingr` endpoint/DTO/mapping/webhook surfaces |

Avoid making non-coders parse these first:

- "domain/app/storage/integrations crates"
- "DTO"
- "typestate"
- "Bon builders"
- "Rustdoc completeness"
- "MCP"

Those terms are acceptable in developer sections after the plain-language explanation.

## Implementation acceptance checklist

The next implementation card should be accepted only if the public landing page:

- leads with labor-cost reduction and review-gated operations, not Rust architecture;
- explains Manager Daily Brief, Data-Quality Hygiene, CRM/retention/daily-update draft loops, and Gingr/source-system posture in non-coder language;
- gives each target audience a path that starts from their question, not from crate names;
- distinguishes local/demo, sandbox continuation, pilot blockers, and live/member-facing blocked status;
- includes explicit safety language for Rustdocs evidence and review-gated agents;
- links to current docs and representative Rustdoc pages without breaking generated Rustdocs;
- keeps crate/module names as evidence/navigation after the business narrative.
