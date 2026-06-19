# NVA Pet Resorts entity atlas

This repository is an **entity-first operating model for labor-cost reduction** across NVA Pet Resorts. It is not a crate-first Rust index and it is not a generic AI demo. Start with the pet-resort thing a non-coder recognizes, then follow the proof trail to workflows, authority, automation boundaries, value measurement, source/Rustdoc evidence, and tests.

A reader should be able to pick any important entity and answer:

- What is it in pet-resort language?
- Why does it exist, and which labor, safety, source-quality, or customer-trust problem does it reduce?
- Which workflows, source-backed rules/promises ([contracts](docs/glossary-architecture-terms.md#contract)), review queues, or source systems feature it?
- Which other entities does it depend on or feed?
- Who or what is authoritative: source system, domain rule/contract, app packet, database/reporting view ([storage projection](docs/glossary-architecture-terms.md#projection)), runtime shell, or human role?
- What may automation summarize, rank, draft, recommend, validate, route, or record?
- What remains blocked or human-reviewed?
- How is value measured: minutes saved, handle time, rework avoided, wrong-source findings, staff disposition, or outcome record?
- Where do source files, Rustdoc pages, doctests/tests, public docs, and local smoke checks prove the claim?

## Public front door

Use the published/non-coder landing source at [docs/public/index.html](docs/public/index.html) when the audience needs the public entrypoint. It mirrors this README's operating model: entity atlas first, workflows and crate/Rustdoc pages as evidence after the business meaning is clear.

## Canonical docs path

If you are new, trust this sequence first. It is the canonical reader path for understanding the labor-saving operating model without starting from crate names:

1. **Start here:** this README explains the strategy, authority layers, automation boundaries, and value-measurement contract.
2. **Choose the business entity:** [NVA Pet Resorts entity index](docs/design/entity-index.md) is the canonical atlas spine for source systems, provenance, customers, pets, reservations, service-line contracts, care/vaccine/incident facts, workflow packets, agents, review gates, blocked actions, outcomes, labor minutes, Gingr/provider boundaries, and storage/runtime shells.
3. **Follow relationships:** [Entity atlas relationship map](docs/design/entity-atlas-relationships.md) shows the proof chain from provider/staff/import evidence to source refs, domain facts, app packets, agent drafts, human review, outcomes, storage, and runtime proof.
4. **Enter from a job-to-be-done:** [Workflow-to-entity navigation map](docs/design/workflow-to-entity-navigation-map.md) routes booking triage, data-quality hygiene, checkout, grooming retention, daily updates/Pawgress, manager brief, and regional exception questions back to the entity families.
5. **Read operator workflows:** [Operator workflow index](docs/workflows/operator/README.md) contains the current workflow pages after the entity relationships and review gates are clear.
6. **Verify proof:** [Contract crosswalk closeout](docs/entity-atlas/contract-crosswalk/README.md) is the source/Rustdoc/test proof layer for claims about source entry, normalization, workflow use, persistence, runtime exposure, evidence gaps, and caveats.

Supporting routes are useful after the canonical path:

- [Entity atlas audience paths](docs/design/entity-atlas-audience-paths.md) gives role-specific routes for operations leaders, resort managers/front-desk, IT/integration, compliance/safety, and product/customer-success readers.
- [Glossary translation layer](docs/design/glossary-translation-layer.md) and [glossary index](docs/glossary.md) translate repo/Rust terms that could otherwise hide the pet-resort meaning.
- [Docs successor and archive map](docs/design/successor-archive-map.md) labels current canonical pages, supporting proof, background/discovery docs, internal QA/audit reports, Kanban/planning artifacts, and superseded workflow/spec pages.
- Safety maps, audits, QA notes, board artifacts, planning docs, and discovery/background pages are supporting evidence or work history. They should not outrank the canonical path above unless a page explicitly says it is the current source of record for a narrow safety, proof, or verification question.
- Archived, superseded, duplicate, or older planning pages should be read as history unless they link forward to the current entity index, relationship map, workflow map, operator workflow pages, contract crosswalk, or successor/archive map.

Front-door principle: the strategy is labor-saving, source-grounded, review-gated operational workflow automation with outcome capture. Gingr and other provider systems provide source evidence; they are not the strategy or automatic business truth. BI, read models, reporting databases, storage projections, and dashboards measure or visualize reviewed work; they do not own workflow authority or bypass human/system-of-record review gates.

## How to read any entity

Every important public entity page, README section, Rustdoc page, or operator guide should preserve this entity reading contract:

| Question | Required answer |
| --- | --- |
| What is it? | Plain-English pet-resort meaning, with source/module names only when they carry authority. |
| Why does it exist? | Labor, safety, customer-trust, source-quality, revenue, or review problem it helps reduce. |
| Where is it used? | Workflows, source-backed contracts, app packets, storage projections, Gingr mappings, tests, Rustdocs, and runtime shells that feature it. |
| What does it relate to? | Source evidence, downstream drafts, review gates, blocked actions, packets, storage records, outcomes, DTOs, and runtime surfaces. |
| Who is authoritative? | Source system, domain contract, app workflow, storage projection, runtime shell, or human/system-of-record role — with projection meaning reporting/review view, not live decision authority. |
| What can automation do? | Draft, rank, summarize, route, validate, record, or recommend only from source-backed app/domain contracts. |
| What is blocked? | Customer/member sends, provider/PMS writes, schedule/capacity changes, payments/refunds/discounts, source hiding, medical/safety approvals, incident decisions, vaccine acceptance, policy exceptions, and labor/staffing mandates unless a specific app-owned approval contract and human/system-of-record action allows them. |
| How is value measured? | Outcome records, labor minutes, handle time, avoided rework, wrong-source findings, completed/deferred/suppressed disposition, or reviewed staff/manager feedback. |
| Where is the proof? | Source files, Rustdoc item pages, doctests/tests, design docs, local smoke checks, and public-doc build checks. |

## Entity navigation map

Start from the family nearest the business question, not from a crate name:

| Entity family | Use it when the reader asks... | Main route |
| --- | --- | --- |
| Source, provenance, and data quality | Where did this fact come from, is it clean enough to use, and what ambiguity must stay visible? | [Source/provenance/data-quality atlas](docs/design/source-provenance-data-quality-atlas.md) |
| Core pet-resort entities | Which customer, pet, reservation, service-line, care, vaccine, incident, message, or staff facts matter? | [PetSuites core entity atlas](docs/design/entity-atlas-petsuites-core-entities.md) |
| Workflow packets, agents, drafts, and review queues | What can an agent draft or recommend, and which packet/review queue owns the work? | [Workflow packets and agents atlas](docs/design/entity-atlas-workflow-packets-agents.md) |
| Review gates and blocked actions | What must be approved by a human or system of record, and what is explicitly forbidden? | [Review safety boundaries atlas](docs/design/entity-atlas-review-safety-boundaries.md) |
| Outcome, labor, operations, money, and safety evidence | How do we prove labor was reduced without unsafe side effects? | [Outcomes/operations/money atlas](docs/design/entity-atlas-outcomes-operations-money.md) |
| Revenue/service-line opportunities | How do grooming, training, retail, package, product, rebooking, and retention opportunities stay review-gated? | [Revenue opportunity atlas](docs/design/entity-atlas-revenue-opportunity-entities.md) |
| Gingr/provider boundary | How do provider facts become source evidence instead of blind business truth? | [Gingr provider boundary atlas](docs/integrations/gingr/provider-boundary-atlas.md) |
| Runtime, storage, API, worker, CLI, and contract-test surfaces | Where are projections, local demos, tests, APIs, workers, and CLI routes exposed safely? | [Runtime/storage/API atlas](docs/design/entity-atlas-runtime-storage-api-surfaces.md) |

## Workflow entrypoints, routed back to entities

Workflow pages are wayfinding for familiar jobs-to-be-done; the entity atlas remains the spine. If you enter from an operator workflow, use [docs/design/workflow-to-entity-navigation-map.md](docs/design/workflow-to-entity-navigation-map.md) first, then the workflow page:

- [Operator workflow index](docs/workflows/operator/README.md)
- [Manager Daily Brief](docs/workflows/operator/manager-daily-brief.md)
- [Booking Triage](docs/workflows/operator/booking-triage.md)
- [Data Quality Hygiene](docs/workflows/operator/data-quality-hygiene.md)
- [Checkout Completion](docs/workflows/operator/checkout-completion.md)
- [Grooming Rebooking / Retention](docs/workflows/operator/grooming-rebooking-retention.md)
- [Daily Updates / Pawgress Drafts](docs/workflows/operator/daily-updates-pawgress-drafts.md)
- [Regional Labor Exceptions / Future Portfolio View](docs/workflows/operator/regional-labor-exceptions.md)

The business acceptance lens and measurement docs are:

- [nva-pet-resorts-ai-context.md](nva-pet-resorts-ai-context.md)
- [Labor-cost reduction crosswalk](docs/design/labor-cost-reduction-crosswalk.md)
- [Manager Daily Brief measurable labor loop](docs/design/manager-daily-brief-measurable-labor-loop.md)
- [Data-quality hygiene local smoke](docs/ops/data-quality-hygiene-local-smoke.md)
- [Agent/app infrastructure contract](docs/architecture/agent-app-infrastructure.md)
- [Builder modernization policy](docs/architecture/builder-modernization-policy.md)
- [Agent/app infrastructure readiness audit](docs/audits/2026-06-18-agent-app-infrastructure-readiness.md)
- [Labor-cost platform readiness audit](docs/audits/2026-06-18-labor-cost-platform-readiness.md)

## Authority, automation, and human-review boundaries

Canonical boundary: agents prepare source-backed review work inside app-owned workflow contracts; humans or approved systems of record keep live operational authority. Read these safety routes before treating any recommendation as action-ready, then use each operator workflow page for its specific blocked-action list:

- [Source evidence map](docs/safety/source-evidence-map.md)
- [Operator safety model](docs/safety/agent-safety-model-for-operators.md)
- [Review boundaries matrix](docs/safety/review-boundaries-matrix.md)
- [Entity/action safety overlays](docs/safety/entity-action-overlays/README.md)
- [Evidence, policy, blocked actions, and outcomes](docs/safety/evidence-policy-blocked-actions-outcomes.md)
- [Labor-cost with human review crosswalk](docs/safety/labor-cost-with-human-review-crosswalk.md)

## Contract crosswalk and proof paths

Use the contract crosswalk when an entity page or workflow claim needs source/Rustdoc/test proof rather than plain-English explanation. Here, [contract](docs/glossary-architecture-terms.md#contract) means a source-backed rule or code promise in this repo, not a legal/customer/vendor agreement:

- [Contract crosswalk closeout](docs/entity-atlas/contract-crosswalk/README.md): package index, coverage summary, evidence gaps, and Entity Atlas/public-docs handoff.
- [Crosswalk schema](docs/entity-atlas/contract-crosswalk/crosswalk-schema.md): required row shape.
- [Surface inventory](docs/entity-atlas/contract-crosswalk/surface-inventory.md): where an entity appears in source, docs, tests, and Rustdoc.
- [Source/provider flows](docs/entity-atlas/contract-crosswalk/source-provider-flows.md): where provider evidence enters and normalizes.
- [Workflow packets](docs/entity-atlas/contract-crosswalk/workflow-packets.md): where entities are used by app workflows.
- [Storage/persistence](docs/entity-atlas/contract-crosswalk/storage-persistence.md): where entities are projected or deliberately not persisted.
- [Runtime exposure](docs/entity-atlas/contract-crosswalk/runtime-exposure.md): API, worker, CLI, web, and script exposure.
- [Relationship adjacency and flow diagrams](docs/entity-atlas/contract-crosswalk/relationship-adjacency.md): bidirectional enter -> normalize -> use -> persist -> expose paths.

## Rustdoc/source evidence after the entity meaning is clear

Crate and module names are evidence paths, not the primary table of contents. Use them after the entity family explains the pet-resort meaning:

- Business vocabulary and invariant-bearing pet-resort facts — the repo's [semantic](docs/glossary-architecture-terms.md#semantic) meaning/source-of-truth vocabulary after validation: [domain/README.md](domain/README.md), [domain/src/lib.rs](domain/src/lib.rs), and service-line/operator summaries for [boarding](domain/src/boarding/README.md), [daycare](domain/src/daycare/README.md), [grooming](domain/src/grooming/README.md), [training](domain/src/training/README.md), [retail](domain/src/retail/README.md), [reservation/checkout](domain/src/reservation/README.md), [money](domain/src/money/README.md), and [payment](domain/src/payment/README.md).
- Cross-service safety/support source surfaces: [care](domain/src/care.rs), [documents](domain/src/document.rs), [vaccines](domain/src/vaccine.rs), [temperament](domain/src/temperament.rs), [incidents](domain/src/incident.rs), and [shared entities/review records](domain/src/entities.rs).
- App-owned workflow packets, deterministic checks, draft validation, agent prompt packets, and tool-port contracts: [app/README.md](app/README.md), [app/src/lib.rs](app/src/lib.rs), [app/src/booking_triage.rs](app/src/booking_triage.rs), [app/src/data_quality_hygiene.rs](app/src/data_quality_hygiene.rs), [app/src/checkout_completion.rs](app/src/checkout_completion.rs), [app/src/crm_retention.rs](app/src/crm_retention.rs), [app/src/daily_update.rs](app/src/daily_update.rs), [app/src/manager_daily_brief.rs](app/src/manager_daily_brief.rs), [app/src/agents.rs](app/src/agents.rs), and [app/src/tools.rs](app/src/tools.rs).
- Storage-shaped proof/projections and stable codecs — durable reporting/review views, not live authority: [storage/README.md](storage/README.md), [storage/src/lib.rs](storage/src/lib.rs), [storage/src/operations.rs](storage/src/operations.rs), and [storage/src/service_line/README.md](storage/src/service_line/README.md).
- Gingr/provider evidence boundaries: [integrations/gingr/README.md](integrations/gingr/README.md), [docs/integrations/gingr/README.md](docs/integrations/gingr/README.md), [docs/integrations/gingr/fixtures/webhooks/README.md](docs/integrations/gingr/fixtures/webhooks/README.md), [integrations/gingr/src/endpoint/README.md](integrations/gingr/src/endpoint/README.md), [integrations/gingr/src/dto/README.md](integrations/gingr/src/dto/README.md), and [integrations/gingr/src/mapping/README.md](integrations/gingr/src/mapping/README.md).
- Runtime shells that expose app/domain contracts without owning business truth: [apps/api/README.md](apps/api/README.md), [apps/worker/README.md](apps/worker/README.md), [apps/cli/README.md](apps/cli/README.md), [apps/api/src/http.rs](apps/api/src/http.rs), [apps/worker/src/runtime.rs](apps/worker/src/runtime.rs), and [apps/cli/src/main.rs](apps/cli/src/main.rs).

The dependency direction remains: source/provider evidence and runtime input adapt into domain/app contracts; storage and runtime projections do not invent business truth or bypass review gates. When docs mention [promotion/demotion](docs/glossary-architecture-terms.md#promotion-demotion), read that as explicit data conversion between raw/provider/storage shapes and validated business meaning.

## Documentation contracts

READMEs are the wiki and navigation layer. They should explain labor saved, entity ownership, source-of-truth boundaries, relationships, human-review boundaries, value measurement, and where to inspect proof. They should not accumulate duplicate Rust snippets that can drift away from compiled APIs.

Executable API examples belong in Rustdoc on source modules and crate roots, where `cargo test --doc` can compile-check them as contracts. When documenting behavior, prefer a README link to the relevant source/Rustdoc/test surface over copying code into Markdown. If a README must include a non-executable sketch, mark it as conceptual and keep it source-grounded.

The docs-as-contracts plan is [docs/plans/2026-06-18-labor-cost-docs-as-contracts-kanban.md](docs/plans/2026-06-18-labor-cost-docs-as-contracts-kanban.md), and the editor checklist is [docs/quality/nva-documentation-style-guide.md](docs/quality/nva-documentation-style-guide.md). The practical rule: lead with labor saved for a specific pet-resort role, use operator English before module/API detail, include a concrete resort example, name source/Rustdoc evidence, state the human approval boundary, and keep executable API details in Rustdoc/source.

## Verification

For docs-only README/navigation changes, run:

```sh
python scripts/check_markdown_links.py --repo-root .
```

For executable docs and wiki/navigation checks, run:

```sh
./scripts/check_docs.sh
```

For the public docs artifact published at the Rustdoc root, keep the non-coder landing page source in [docs/public/index.html](docs/public/index.html) and generate the local artifact with:

```sh
./scripts/build_public_docs.sh
```

For code changes, run:

```sh
cargo fmt --all -- --check
cargo test --workspace --no-run
```

For the canonical local gate, run:

```sh
./scripts/test.sh
```

The external Rustdoc completeness guardrail can also be run directly:

```sh
python scripts/check_rustdoc_completeness.py
```

It executes the strict source-of-truth command:

```sh
RUSTDOCFLAGS='-D missing_docs' cargo doc --workspace --no-deps
```
