# Glossary/entity-atlas cross-link map

Purpose: record stable link targets between glossary terms, Entity Atlas family pages, workflow/operator pages, safety docs, and Rustdoc/source evidence. This is a downstream-consumable map for documentation workers; it does not create product behavior or broaden automation authority.

## Link conventions

- Prefer reviewed glossary entries in `docs/glossary-architecture-terms.md`, `docs/glossary-source-data-terms.md`, and `docs/glossary-workflow-state-terms.md` for public/operator wording.
- Prefer Entity Atlas family pages for entity-centered explanations, relationship context, review boundaries, and known gaps.
- Prefer source files, module paths, generated Rustdoc, and tests for evidence. If generated Rustdoc is absent, cite the source path or module path instead of inventing a rendered URL.
- When a future entity page does not exist yet, link to the entity index row, nearest family page, or contract-crosswalk row and mark the target as planned/no page yet.

## Cross-link targets

| Term family | Glossary target | Entity Atlas / evidence target | Notes |
| --- | --- | --- | --- |
| Domain entity / domain truth | [`domain`](../glossary-architecture-terms.md#domain) | [PetSuites core entity atlas](entity-atlas-petsuites-core-entities.md), [entity index](entity-index.md) | Domain terms are normalized business vocabulary, not provider rows or database tables. |
| Source/provider fact | [Gingr/source](../glossary-source-data-terms.md#domainsourcesystemgingr-gingr), [provider record](../glossary-source-data-terms.md#provider-record), [source-of-record](../glossary-source-data-terms.md#source-of-record) | [Source, provenance, and data-quality atlas](source-provenance-data-quality-atlas.md), [Gingr provider boundary atlas](../integrations/gingr/provider-boundary-atlas.md) | Provider evidence must stay distinct from domain truth and live action authority. |
| DTO / provider payload shape | [`DTO`](../glossary-architecture-terms.md#dto) | [Gingr provider boundary atlas](../integrations/gingr/provider-boundary-atlas.md), [source/provider flows](../entity-atlas/contract-crosswalk/source-provider-flows.md) | DTOs capture provider-shaped evidence before mapping/review. |
| Workflow packet / review bundle | [workflow packet](../glossary-workflow-state-terms.md#workflow-packet) | [Workflow packets and agents atlas](entity-atlas-workflow-packets-agents.md), [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md) | Packets are review bundles, not executed provider/customer side effects. |
| Review gate and blocked action | [review gate](../glossary-workflow-state-terms.md#review-gate), [blocked action](../glossary-workflow-state-terms.md#blocked-action) | [Review gates, blocked actions, and human approval boundaries](entity-atlas-review-safety-boundaries.md), [review boundaries matrix](../safety/review-boundaries-matrix.md) | Keep human/system-of-record approval explicit. |
| Outcome / labor evidence | [outcome capture](../glossary-workflow-state-terms.md#outcome-capture) | [Outcomes, operations, labor, analytics, money, and safety evidence atlas](entity-atlas-outcomes-operations-money.md) | Outcome records document reviewed results; they do not perform the action. |
| Runtime/API/worker/storage surface | [`storage`](../glossary-architecture-terms.md#storage), [`app`](../glossary-architecture-terms.md#app) | [Runtime, storage, API, worker, CLI, and test surfaces atlas](entity-atlas-runtime-storage-api-surfaces.md), [runtime exposure crosswalk](../entity-atlas/contract-crosswalk/runtime-exposure.md) | Runtime shells and storage projections are evidence/adapter surfaces, not alternate sources of domain truth. |
| Rustdoc/source link | [core entity-linked glossary draft](core-entity-linked-glossary-draft.md#rustdoc-source-link) | [entity atlas evidence-link anchors](entity-atlas-evidence-link-anchors.md), [Rustdoc completeness gate](../../scripts/check_rustdoc_completeness.py) | Generated Rustdoc is local evidence; source files remain authoritative when generated docs are absent. |
