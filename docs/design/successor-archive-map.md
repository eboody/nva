---
title: "Docs successor and archive map"
slug: "docs-successor-archive-map"
status: "current-navigation"
audience: ["operator", "docs-writer", "engineer", "reviewer"]
plain_english_definition: "A routing map that labels current docs, supporting proof, background artifacts, audits, Kanban/planning notes, and superseded workflow/spec pages so older documents do not compete with the canonical reader path."
primary_labor_problem: "Reduces reader and reviewer time spent deciding whether an older workflow/spec, audit, or planning artifact is current authority."
source_of_record: "README.md canonical path, entity atlas family pages, operator workflow index, contract crosswalk, and linked source/Rustdoc/test evidence."
---

# Docs successor and archive map

Use this map when an older page looks useful but may compete with the canonical reader path. The current spine is still:

1. [README canonical docs path](../../README.md#canonical-docs-path)
2. [Entity index](entity-index.md)
3. [Entity atlas relationship map](entity-atlas-relationships.md)
4. [Workflow-to-entity navigation map](workflow-to-entity-navigation-map.md)
5. [Operator workflow index](../workflows/operator/README.md)
6. [Contract crosswalk closeout](../entity-atlas/contract-crosswalk/README.md)

A document that is not on that path can still be valuable evidence. It should be read through the label below, not as standalone product authority.

## Labels used in this repo

| Label | Meaning | Reader action |
| --- | --- | --- |
| Current canonical | The page is part of the current entity-first operating model or current operator workflow set. | Trust it first, then follow its proof links. |
| Supporting proof | Source/Rustdoc/test crosswalk, architecture, safety, or runtime evidence that supports current claims. | Use it to verify a specific claim; do not treat it as the primary table of contents. |
| Background/discovery | Useful older or exploratory context that explains why the model exists. | Read for history or rationale, then follow the successor link before making current claims. |
| Internal QA/audit | Review packets, smoke checks, readiness memos, and closeouts. | Use as evidence of caveats, risks, and verification state; not as product/workflow authority. |
| Kanban/planning | Board plans, task packets, duplicate audits, and staged work lists. | Use for work history and rationale only. Do not let it outrank current docs. |
| Archived/superseded | Older workflow/spec/page shape that has a clearer successor route. | Preserve for provenance, but cite the successor route for current behavior. |

## Current canonical docs

| Area | Current destination | Why it is current |
| --- | --- | --- |
| Front door and reader order | [README canonical docs path](../../README.md#canonical-docs-path) | Declares the entity-first operating model, authority layers, and safe reading sequence. |
| Entity atlas spine | [Entity index](entity-index.md), [relationship map](entity-atlas-relationships.md), [audience paths](entity-atlas-audience-paths.md) | Primary model for source evidence, domain entities/contracts, workflow packets, review gates, outcomes, and proof. |
| Workflow entry from operator jobs | [Workflow-to-entity navigation map](workflow-to-entity-navigation-map.md) and [operator workflow index](../workflows/operator/README.md) | Routes familiar resort jobs back to entity families before procedure-like workflow prose. |
| Current operator workflow examples | [Manager Daily Brief](../workflows/operator/manager-daily-brief.md), [Booking Triage](../workflows/operator/booking-triage.md), [Data Quality Hygiene](../workflows/operator/data-quality-hygiene.md), [Checkout Completion](../workflows/operator/checkout-completion.md), [Grooming Rebooking / Retention](../workflows/operator/grooming-rebooking-retention.md), [Daily Updates / Pawgress Drafts](../workflows/operator/daily-updates-pawgress-drafts.md), [Regional Labor Exceptions / Future Portfolio View](../workflows/operator/regional-labor-exceptions.md) | These are the readable workflow pages, but they remain entrypoints into entity and proof pages rather than the documentation spine. |
| Full proof chain example | [Manager Daily Brief walkthrough](../workflows/operator/manager-daily-brief-walkthrough.md) | Shows source evidence -> provenance/data quality -> app packet -> agent-safe output -> human review -> outcome/labor minutes -> storage/runtime/reporting proof. |
| Contract/source proof | [Contract crosswalk closeout](../entity-atlas/contract-crosswalk/README.md) and its package pages | Verifies where entities appear in source, tests, storage, runtime, and Rustdoc evidence. |
| Labor-cost navigation | [Labor-cost reduction crosswalk](labor-cost-reduction-crosswalk.md) | Connects labor drivers to workflows, entity families, review gates, and outcome proof. |

## Supporting proof docs

| Document or family | Label | Current successor / use |
| --- | --- | --- |
| [Source/provenance/data-quality atlas](source-provenance-data-quality-atlas.md) | Current canonical for source-quality questions | Use before allowing source/provider facts into workflow packets. |
| [Review safety boundaries atlas](entity-atlas-review-safety-boundaries.md) and [safety docs](../safety/source-evidence-map.md) | Supporting proof | Use for entity/action review gates and blocked actions; workflow pages should link rather than duplicate broad safety prose. |
| [Runtime/storage/API surfaces atlas](entity-atlas-runtime-storage-api-surfaces.md) | Supporting proof | Confirms storage/reporting/runtime shells are visibility and proof surfaces, not workflow authority. |
| [Pet-resort workflow events](../architecture/pet-resort-workflow-events.md) | Supporting proof | Event-invocation design evidence. Current workflow reading still starts at the workflow-to-entity map. |
| [Pet-resort AI/Hermes runtime architecture](../architecture/pet-resort-ai-runtime.md) and [structured output validation](../architecture/pet-resort-ai-runtime-structured-output.md) | Supporting proof | Runtime and validation architecture drafts; they do not approve production LLM use or side effects. |
| [Pet Resort canonical data model](../architecture/pet-resort-data-model.md) | Supporting proof | Data-model background for entity rows; current operational meaning lives in entity atlas pages. |
| `domain/`, `app/`, `storage/`, `integrations/gingr/`, `apps/*` READMEs and Rustdocs | Supporting proof | Source/Rustdoc evidence after the entity meaning is clear. |

## Older workflow and specification docs

Top-level pages under `docs/workflows/*.md` are detailed specification artifacts. They remain useful for integration planning, schemas, and unresolved gates, but they should not outrank the current operator workflow pages and entity atlas.

| Older workflow/spec | Label | Successor route |
| --- | --- | --- |
| [Booking triage agent](../workflows/booking-triage-agent.md) | Supporting proof / older spec | Read [Booking Triage operator page](../workflows/operator/booking-triage.md), then [workflow-to-entity Booking row](workflow-to-entity-navigation-map.md#workflow---entity-matrix), then use this spec for detailed deterministic-rule evidence. |
| [CRM and retention agent workflow](../workflows/crm-retention-agent.md) | Supporting proof / older spec | Read [Grooming Rebooking / Retention](../workflows/operator/grooming-rebooking-retention.md) and [revenue opportunity atlas](entity-atlas-revenue-opportunity-entities.md) first. |
| [Daily Care Update Agent](../workflows/daily-care-update-agent.md) | Supporting proof / older spec | Read [Daily Updates / Pawgress Drafts](../workflows/operator/daily-updates-pawgress-drafts.md), then core/review-safety entity pages. |
| [Vaccine document agent workflow](../workflows/vaccine-document-agent.md) | Supporting proof / older spec | Read [PetSuites core entities](entity-atlas-petsuites-core-entities.md) and [review safety boundaries](entity-atlas-review-safety-boundaries.md) before treating vaccine/OCR facts as workflow evidence. |
| [Incident/Escalation Agent](../workflows/incident-escalation-agent.md) | Supporting proof / older spec | Read core incident entities and review-safety boundaries first; incident closure, owner messages, care/medical decisions, and provider writes remain gated. |
| [Inquiry intake agent workflow](../workflows/inquiry-intake-agent.md) and inquiry-intake parts | Supporting proof / older spec | Route through [Booking Triage](../workflows/operator/booking-triage.md), source/provenance, and review-safety pages before using intake templates. |
| [Customer messaging agent](../workflows/customer-messaging-agent.md) | Supporting proof / older spec | Route through [workflow packets/agents](entity-atlas-workflow-packets-agents.md), [review safety boundaries](entity-atlas-review-safety-boundaries.md), and the operator workflow whose draft message is being prepared. |
| [Staff operations workflow](../workflows/staff-operations.md) | Supporting proof / older spec | Route through core entities, review safety, Manager Daily Brief, Daily Updates, Checkout Completion, and Data Quality Hygiene depending on the staff job. |
| [Payments and pricing workflow](../workflows/payments-pricing.md) | Supporting proof / older spec | Route through [outcomes/operations/money atlas](entity-atlas-outcomes-operations-money.md) and review-safety payment boundaries before citing payment or discount behavior. |
| [Workflow event idempotency and replay rules](../workflows/workflow-event-idempotency-replay.md) | Supporting proof / design draft | Use as runtime/event evidence only after a current workflow packet and review gate are identified. |
| Part directories under `docs/workflows/*-parts/` | Supporting proof | Schemas, tone guides, fixtures, and policy drafts. Link them from the relevant current workflow/spec only when they support a named entity, packet, or review gate. |

## Background/discovery docs

| Document | Label | Successor route |
| --- | --- | --- |
| [NVA Pet Resorts AI context pack](../../nva-pet-resorts-ai-context.md) | Background/discovery | Use [README](../../README.md), [entity index](entity-index.md), [workflow-to-entity map](workflow-to-entity-navigation-map.md), and [labor-cost crosswalk](labor-cost-reduction-crosswalk.md) for current claims. |
| [Core entity linked glossary draft](core-entity-linked-glossary-draft.md), [glossary link maps](glossary-entity-atlas-link-map.md), and [entity-relevant architecture/Rust term inventory](entity-relevant-architecture-rust-term-inventory.md) | Background/supporting proof | Use the entity atlas pages and glossary index for current reader navigation; these are editing aids. |
| [Operator workflow page inventory](operator-workflow-page-inventory.md), [entity-driven workflow template](entity-driven-workflow-page-template.md), [workflow source/Rustdoc maps](workflow-page-source-rustdoc-map.md) | Background/supporting proof | Use as drafting/evidence aids; current readers should start with the operator index and workflow-to-entity map. |
| Public-doc landing content/technical maps | Background/supporting proof | Use [docs/public/index.html](../public/index.html) and README for current published front-door content. |

## Internal QA and audit docs

| Family | Label | Successor route |
| --- | --- | --- |
| `docs/audits/2026-06-18-*.md` readiness memos | Internal QA/audit | Use for caveats and readiness evidence; current navigation remains README -> entity atlas -> workflows -> contract proof. |
| Modum/semantic closeout and lint audit docs under `docs/audits/` | Internal QA/audit | Preserve as source-quality closeout history; not product authority. |
| `docs/qa-*.md` reports | Internal QA/audit | Use as regression evidence for jargon, source authority, runtime/storage caveats, and final non-coder checks. |
| [Entity atlas coverage reconciliation](entity-atlas-coverage-reconciliation.md) and [non-coder QA findings](entity-atlas-non-coder-qa-findings.md) | Internal QA/audit supporting the current spine | Use to find residual gaps; do not replace the entity index or relationship map. |

## Kanban and planning docs

| Family | Label | Successor route |
| --- | --- | --- |
| [Docs usefulness/pruning Kanban board](../kanban/2026-06-19-docs-usefulness-pruning-board.md) | Kanban/planning | Explains this cleanup board. Current readers should use this successor/archive map plus README. |
| Old factory inventories and duplicate/misroute audits under `docs/kanban/` | Kanban/planning / archived evidence | Preserve as work-history evidence. The current successor route is this map, the entity atlas, and current operator workflows. |
| Task packets under `docs/kanban/t_*.md` | Kanban/planning / internal QA | Use only when auditing why a fix or QA result happened. |
| Board plans under `docs/plans/` | Kanban/planning | Use as editing process history, then verify against current docs and source. |

## Citation rule for older docs

When citing an older spec, audit, QA report, or Kanban artifact in new work, include the current successor route in the same paragraph or nearby link. Example:

> The older [booking triage agent spec](../workflows/booking-triage-agent.md) contains detailed deterministic-rule evidence; current readers should enter through the [Booking Triage operator page](../workflows/operator/booking-triage.md) and [workflow-to-entity map](workflow-to-entity-navigation-map.md#workflow---entity-matrix).

Do not cite older artifacts as evidence for production authority, live provider/customer actions, autonomous sends, payment movement, staffing decisions, or BI/reporting authority unless the current entity/workflow/proof pages and source/test evidence also establish that authority.
