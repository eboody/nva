# Entity-linked glossary entry template

Purpose: define glossary entries as an entity translation layer for NVA pet-resort docs. A glossary entry should translate a repo/Rust term into operating meaning for a resort operator, product reader, IT reviewer, or docs maintainer, while keeping the Entity Atlas, relationship map, workflow pages, safety docs, source, Rustdoc, and tests as the authority.

Use this template for cross-cutting terms that clarify entities, relationships, evidence, review boundaries, or labor measurement. Do not use it as a general architecture dictionary or a place to explain Rust mechanics that do not change operator understanding.

Source basis:

- Parent inventory: [Entity-relevant architecture and Rust term inventory](entity-relevant-architecture-rust-term-inventory.md).
- Entity Atlas contracts: [Entity atlas page template](entity-atlas-page-template.md), [Entity atlas relationship map](entity-atlas-relationships.md), [Entity atlas inventory](entity-atlas-inventory.md), and the family pages under `docs/design/entity-atlas-*.md`.
- Workflow/entity crosswalks: [Workflow-to-entity navigation map](workflow-to-entity-navigation-map.md), [contract crosswalk schema](../entity-atlas/contract-crosswalk/crosswalk-schema.md), [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md), [source/provider flows](../entity-atlas/contract-crosswalk/source-provider-flows.md), and [surface inventory](../entity-atlas/contract-crosswalk/surface-inventory.md).
- Existing glossary design: [Glossary translation layer](glossary-translation-layer.md) and [glossary index](../glossary.md).
- Style and safety: [NVA documentation style guide](../quality/nva-documentation-style-guide.md), [source evidence map](../safety/source-evidence-map.md), [review boundaries matrix](../safety/review-boundaries-matrix.md), and [evidence policy / blocked actions / outcomes](../safety/evidence-policy-blocked-actions-outcomes.md).

## Selection rule

Create or keep a glossary entry only when the term helps a non-coder answer at least one Entity Atlas question:

1. What entity, relationship, workflow packet, review gate, source fact, or outcome does this term help explain?
2. Who or what is authoritative for the related fact, decision, or action?
3. What evidence must travel with the entity or workflow before a recommendation is safe?
4. What may automation draft, rank, summarize, validate, recommend, or record?
5. What stays blocked, human-reviewed, source-system-owned, or explicitly unknown?
6. How does the term affect labor reduction, rework prevention, customer trust, pet safety, or auditable outcomes?

If the term only helps Rust maintainers read implementation mechanics, put the explanation in Rustdoc/source or the nearest maintainer README instead of a non-coder glossary entry.

## Required entry shape

Use this shape for every entity-linked glossary entry. A section may be short, but it should not be omitted unless the term is explicitly marked as a stub awaiting source evidence.

```text
## [Term as used in code/docs]

Term in code/docs:
  Exact spelling, source path, module path, Rustdoc item path, page title, or phrase. Preserve module paths when they teach ownership or authority, such as `domain::source::Provenance` or `app::manager_daily_brief::OutcomeRecord`.

Plain-English operational translation:
  One or two sentences in pet-resort language. Lead with what a front-desk lead, manager, regional ops reviewer, groomer, trainer, daycare/kennel lead, product reader, IT reviewer, or docs writer needs to know before reading implementation detail.

Why this matters to operators / product / IT:
  Name the labor, safety, trust, reporting, or source-quality problem the term prevents or makes auditable. Avoid abstract architecture value unless it maps to a resort workflow or review risk.

Linked entity/entities:
  List the Entity Atlas page(s), entity families, and adjacent concepts the term helps explain. Say whether the term belongs to a core entity, source/provenance entity, workflow packet, review gate, outcome record, storage projection, provider boundary, or runtime shell.

Linked workflows / contracts:
  List workflow pages, app modules, contract-crosswalk rows, policy/review contracts, tool-port contracts, storage records, provider mappings, or API/worker shell contracts where the term appears. Keep workflow names in operator English before code paths.

Authoritative source / Rustdoc / test evidence:
  Cite source files, Rustdoc/module/type paths, tests, README sections, or docs evidence maps that support the wording. If rendered Rustdoc does not exist, write “Rustdoc/module path” and cite the source path instead of inventing a URL.

Automation boundary:
  Split into two bullets:
  - May draft/recommend/validate/record: source-backed actions the app/agent may perform with this term.
  - Blocked / human-reviewed: live customer sends, PMS/provider writes, schedule/capacity changes, payment/refund/discount movement, vaccine/medical/temperament/safety approval, policy exceptions, data hiding/deletion, secret-dependent side effects, or other actions that remain outside glossary authority.

Common confusion / what not to infer:
  Name the dangerous overread. Examples: provider evidence is not domain truth; a review gate is not approval; a draft is not a sent message; a storage record is not policy; provenance is not correctness; a read model is not source-of-record authority; a runtime shell is not business ownership.

Link targets:
  Provide concrete links back to Entity Atlas pages, the relationship map/crosswalk, workflow pages, safety docs, and adjacent glossary entries. These links are part of the entry contract, not optional “see also” filler.

Suggested public wording:
  One to three sentences suitable for a public/operator doc. Keep the code/docs term in parentheses or path form if it preserves authority.

Review status:
  draft | reviewed | published, plus reviewer role or source of review if known.
```

## Optional fields for complex terms

Use these when a term crosses several layers or is likely to be overclaimed.

```text
Lifecycle position:
  Where the term sits in the flow from provider/source evidence -> mapping/data-quality gate -> domain entity -> app workflow packet -> agent draft/recommendation -> human review -> outcome/storage proof -> runtime/API/CLI inspection.

Entity relationship diagram:
  Short text or Mermaid sketch. Mark docs-only/TODO edges explicitly if no source/test contract backs the relationship.

Examples and non-examples:
  Two examples and two non-examples. This prevents helper DTOs, provider ids, storage codes, fixture names, or Rust implementation details from becoming accidental business entities.

Outcome evidence:
  Outcome record, approval record, audit event, source refs, actual minutes saved/wasted, reviewer disposition, or reporting fields that prove the term’s labor/safety claim.

Reviewer checklist:
  Entry-specific questions that catch overclaims before publication.
```

## Frontmatter option

For pages that will be indexed later, use compact frontmatter before the Markdown body.

```yaml
---
title: "Review gate"
slug: "review-gate"
status: "draft|reviewed|published"
audience: ["front-desk", "general-manager", "regional-ops", "product", "IT", "docs-writer"]
term_kind: "workflow-state|source-evidence|architecture-layer|provider-boundary|outcome-evidence|runtime-shell"
related_entities: ["Approval Record", "Workflow Packet", "Blocked Action"]
related_workflows: ["booking-triage", "manager-daily-brief", "daily-updates"]
source_paths:
  - "domain/src/policy.rs"
  - "domain/src/workflow.rs"
  - "app/src/agents.rs"
rustdoc_contracts:
  - "domain::policy::ReviewGate"
  - "domain::workflow::RequestHumanReview"
safety_links:
  - "docs/safety/review-boundaries-matrix.md"
  - "docs/safety/evidence-policy-blocked-actions-outcomes.md"
may_draft_or_recommend: "prepare evidence, route a packet, explain required approval, and record reviewed disposition where the app contract allows it"
blocked_or_human_reviewed: "no live customer/provider/payment/schedule/safety/policy action merely because a gate is present"
---
```

## Link contract

Each entry must link in four directions so it stays an entity translation layer rather than a detached dictionary.

| Link direction | Required target type | Why |
| --- | --- | --- |
| Entity Atlas | entity page, family page, or inventory row | Shows which business concept the term clarifies. |
| Relationship/crosswalk | relationship map, contract crosswalk, source-provider flow, or workflow-packet crosswalk | Shows how the term moves between source facts, domain entities, app packets, review, outcome, storage, and runtime shells. |
| Workflow/contract | operator workflow page, app module, domain policy/source module, storage projection, provider mapping, or tool-port contract | Grounds operational meaning in an implemented or explicitly TODO surface. |
| Safety/evidence | source evidence map, review boundaries matrix, blocked-action/outcome policy, or entity action overlay | Prevents the glossary from implying live authority. |

Recommended default link bundle for high-risk entries:

- [Entity atlas relationship map](entity-atlas-relationships.md)
- [Entity atlas page template](entity-atlas-page-template.md)
- [Workflow-to-entity navigation map](workflow-to-entity-navigation-map.md)
- [Source evidence map](../safety/source-evidence-map.md)
- [Review boundaries matrix](../safety/review-boundaries-matrix.md)
- [Evidence policy / blocked actions / outcomes](../safety/evidence-policy-blocked-actions-outcomes.md)

## Automation and human-review wording

Use this table to keep entries consistent.

| If the term touches... | Automation may draft/recommend/record | Must stay blocked or human-reviewed |
| --- | --- | --- |
| Source/provider evidence | cite source refs, map candidates, flag data-quality issues, summarize evidence | provider/PMS writes, source deletion/hiding, unsupported provider-surface assumptions |
| Customer messaging | draft approved-tone text, cite included/omitted facts, prepare review packet | autonomous customer/member send, sensitive-fact disclosure, local-policy promise |
| Pet safety / medical / temperament | summarize care/vaccine/incident/temperament evidence, flag missing/stale facts | medical/vaccine/temperament/group-play/safety approval |
| Booking / schedule / capacity | rank readiness issues, draft internal tasks, explain hard stops | booking confirmation/rejection, check-in/out, room/capacity/playgroup/schedule mutation |
| Money / retail / payment | summarize checkout/payment evidence, draft review candidates, record approved outcome | refunds, discounts, payment/deposit/credit/session-balance movement, POS/vendor orders |
| Labor / outcome reporting | estimate or record labor minutes only with source/review context | ROI claims without outcome records or reviewed disposition evidence |
| Runtime/API/worker/CLI shell | expose or inspect app/domain/storage proof where implemented | business authority beyond the linked app/domain contract |

## Recommended destination files

Use existing glossary files by term family. Do not create a new top-level glossary file unless the term family is missing or a later publishing task asks for one.

| Term family | Destination | Examples |
| --- | --- | --- |
| Architecture and authority layers | `docs/glossary-architecture-terms.md` | `domain`, `app`, `storage`, `integrations/gingr`, adapter, DTO, tool port, read model |
| Source evidence and data quality | `docs/glossary-source-data-terms.md` | Gingr/source system, provider record, source-of-record, source ref, provenance, data-quality issue, mapping candidate |
| Workflow state and safety boundaries | `docs/glossary-workflow-state-terms.md` | draft, review gate, blocked action, workflow packet, agent spec, outcome capture |
| Entity-specific concepts with broad reuse | nearest Entity Atlas family page plus `docs/glossary.md` index if public-facing | Approval Record, Audit Event, Manager Daily Brief Outcome Record, Data Quality Hygiene Packet |
| Deferred implementation mechanics | source/Rustdoc or closest maintainer README, not non-coder glossary | typestate builders, `nutype`, `bon`, generated `statum` internals, generic Rust errors, HTTP primitive wrappers |
| Provider/runtime details that matter only locally | nearest provider/runtime atlas page | provider ids, raw payload hash, field path, webhook envelope, mock transport, API/worker shell |

Update `docs/glossary.md` only when an entry should be discoverable from public docs. Keep index wording short and link to the full entry; do not duplicate the entry there.

## Reviewer acceptance checklist

Before publishing or approving an entry, verify:

- The first paragraph translates the term into pet-resort operating meaning before implementation detail.
- The term is tied to at least one entity, relationship, workflow packet, source fact, review gate, outcome record, provider boundary, storage projection, or runtime shell.
- The entry names why a pet-resort operator, product reader, or IT reviewer should care.
- The source/Rustdoc/test evidence is specific enough for a maintainer to verify.
- Automation boundaries are split into “may draft/recommend/validate/record” and “blocked/human-reviewed.”
- The entry names at least one common confusion or “what not to infer.”
- Link targets include Entity Atlas, relationship/crosswalk, workflow/contract, and safety/evidence destinations.
- Product gaps are marked as TODO/discovery questions; glossary prose does not invent live integrations, approvals, or labor results.
- If the term is in the parent inventory’s defer/drop list, the entry either stays out of public glossary files or explains why an entity-specific exception is needed.
