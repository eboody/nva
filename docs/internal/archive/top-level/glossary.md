# NVA pet-resort glossary index

Purpose: route non-coder public docs, operator workflow pages, and API-reference/Rustdoc readers to the glossary entries that translate repo/Rust terms into pet-resort operational meaning. The glossary preserves source-backed meaning from the code and docs: it explains what a term means in this repository, where it appears, why operators should care, and what not to infer.

Use this index as nearby help from landing pages, workflow pages, technical READMEs, and API-reference/Rustdoc pages. Do not use it as a replacement for source files, Rustdoc, tests, or design docs.

## Entity Atlas bridge

The glossary now links bidirectionally with the entity-centered docs system. Use [NVA Pet Resorts entity index](design/entity-index.md) as the atlas spine, [Entity atlas relationship map](design/entity-atlas-relationships.md) for entity-to-entity edges, [contract crosswalk schema](entity-atlas/contract-crosswalk/crosswalk-schema.md) for source/workflow/storage/Rustdoc proof rows, and [Glossary/entity-atlas cross-link map](design/glossary-entity-atlas-link-map.md) as the downstream link-map note for board workers.

Stub convention: when a term points at an entity page that is not yet written, link to the entity index row, nearest family page, or contract-crosswalk row rather than inventing a broken page. Mark future targets as `planned/no page yet` in link-map notes.

## Architecture and layer terms

- [`domain`](glossary-architecture-terms.md#domain): source-of-truth business vocabulary for pet-resort facts, policies, review gates, source evidence, and service-line rules.
- [`app`](glossary-architecture-terms.md#app): workflow/review-bundle layer for request packets, deterministic evaluations, draft artifacts, agent prompt packets, and tool-port contracts.
- [`storage`](glossary-architecture-terms.md#storage): persisted projection and conversion boundary for durable records, codes, codecs, and outcome/reporting evidence.
- [`integrations/gingr`](glossary-architecture-terms.md#integration-integrationsgingr): provider boundary for Gingr evidence, endpoint/response/webhook shapes, DTOs, and mapping candidates.
- [`contract`](glossary-architecture-terms.md#contract): source-backed rule or code promise in this repo; not a legal/customer/vendor contract.
- [`semantic`](glossary-architecture-terms.md#semantic): business meaning/source-of-truth vocabulary after validation; not just text wording or AI interpretation.
- [`projection`](glossary-architecture-terms.md#projection): database/reporting-friendly view of facts; not live decision authority.
- [`promotion` / `demotion`](glossary-architecture-terms.md#promotion-demotion): explicit conversion between raw/provider/storage shapes and validated business meaning.
- [`adapter`](glossary-architecture-terms.md#adapter): quarantined translator at an outside-system or boundary shape.
- [`DTO`](glossary-architecture-terms.md#dto): provider payload shape; evidence from a provider before domain mapping validates what can be used.
- [`tool port` / `app::tools`](glossary-architecture-terms.md#tool-port-apptools): approved capability interface for typed reads, checks, drafts, and future side-effect implementations.
- [`read model`](glossary-architecture-terms.md#read-model): reporting or review projection optimized for reading, not source-of-record authority.

## Source evidence and data-quality terms

- [Gingr / `domain::source::System::Gingr`](glossary-source-data-terms.md#domainsourcesystemgingr-gingr): provider operating-system evidence source, not automatic NVA domain truth.
- [Provider record](glossary-source-data-terms.md#provider-record): provider-native evidence record before explicit mapping and promotion.
- [Source-of-record](glossary-source-data-terms.md#source-of-record): the authorized owner for a specific fact or action; ask “source of record for what?”
- [Data-quality issue / `domain::data_quality::Issue`](glossary-source-data-terms.md#domaindata_qualityissue-data-quality-issue): tracked source-data exception that can drive review, BI caveats, cleanup, or workflow blocking.
- [Provenance and source refs as data evidence](glossary-source-data-terms.md#domainsourceprovenance-and-domainsourcerecordref-as-data-evidence): evidence trail and source pointer for operational facts.
- [Source ref / `domain::source::RecordRef`](glossary-architecture-terms.md#source-ref-domainsourcerecordref): stable pointer to upstream evidence.
- [Provenance / `domain::source::Provenance`](glossary-architecture-terms.md#provenance-domainsourceprovenance): chain of custody for a source-backed fact.

## Workflow, review, and operator-state terms

- [Draft](glossary-workflow-state-terms.md#draft): staff-review artifact prepared before any live action.
- [Review gate / `domain::policy::ReviewGate`](glossary-workflow-state-terms.md#review-gate): named human-approval stop for sensitive resort work.
- [Blocked action](glossary-workflow-state-terms.md#blocked-action): action automation must not perform directly.
- [Outcome capture](glossary-workflow-state-terms.md#outcome-capture): recording what staff did and what labor result was observed.
- [Workflow packet](glossary-workflow-state-terms.md#workflow-packet): typed review bundle for one workflow.
- [Agent spec](glossary-workflow-state-terms.md#agent-spec): operating contract for a bounded automation helper.

## Core entity-linked review entries

These entries are maintainer review material for terms that cross several glossary files and Entity Atlas pages. Use them when a non-coder reader needs to preserve the boundary between domain truth, source/provider evidence, workflow packets, storage projections, review gates, and measured outcomes.

- [Core entity-linked glossary review notes](design/core-entity-linked-glossary-draft.md): review-ready entries for entity/domain truth, source/provider fact, DTO, workflow packet, review gate, blocked action, storage projection, outcome/labor metrics, adapter/integration, runtime shell, idempotency, validated values, readiness/typestate, Rustdoc/source links, and [non-coder example translations tied to entities and workflows](design/core-entity-linked-glossary-draft.md#non-coder-example-translations-tied-to-entities-and-workflows).

## Where to link from public docs

Use glossary links near the first non-coder-facing use of a term, not as a detached dictionary at the end of a page.

- Landing/navigation pages should link the crate labels `domain`, `app`, `storage`, and `integrations/gingr` to the architecture entries, then link workflow phrases such as draft, review gate, blocked action, and outcome capture where the safety story appears.
- Operator workflow pages should link the page’s context packet, draft/recommendation packet, human approval gate, blocked actions, outcome record, source refs/provenance, and data-quality issue terms in the section where each concept is introduced.
- Gingr/source pages should link Gingr, provider record, DTO, adapter, source-of-record, source ref, provenance, and data-quality issue terms before asking non-coders to read endpoint or mapping Rustdocs.
- Rustdoc-facing README pages should pair technical crate/module names with glossary links so non-coders get the operational meaning before reading generated API details.
