# Old docs factory -> entity-centered successor map

Generated: 2026-06-19
Task: `t_ae0bcb4b` — map old docs work to entity-centered successor structure
Input inventory: `docs/kanban/old-factory-board-inventory-t_5078771f.md`
Scope: mapping artifact only. No board dispatch, archive, unblock, or source/doc mutation beyond this file.

## Successor structure used for this map

The successor structure is entity-centered, not board-centered or workflow-first. Useful legacy work should be preserved by routing it into one of these buckets:

| Successor bucket | Concrete destination board/card/artifact |
| --- | --- |
| Canonical entity atlas spine | Board `nva-entity-atlas-relationship-map`; `docs/design/entity-index.md`; `docs/design/entity-atlas-inventory.md`; family pages under `docs/design/entity-atlas-*.md`; `docs/integrations/gingr/provider-boundary-atlas.md` |
| Relationship/contract crosswalk | Board `nva-entity-atlas-relationship-map`, card `t_2de7d3e8`; `docs/design/entity-atlas-relationships.md`; source/Rustdoc/test links in family pages |
| README/public landing centered on entities | Successor landing should route first to `docs/design/entity-index.md` and audience paths, then workflow pages/Rustdocs as evidence; legacy sources: `docs/public/index.html`, `README.md`, `scripts/build_public_docs.sh` |
| Workflow pages driven by featured entities/contracts | `docs/design/workflow-to-entity-navigation-map.md`; `docs/design/entity-driven-workflow-page-template.md`; existing `docs/workflows/operator/*.md` become examples/entrypoints, not the spine |
| Service-line docs applying entity template | `docs/design/entity-atlas-petsuites-core-entities.md`; `docs/design/entity-atlas-revenue-opportunity-entities.md`; service-line Rustdocs/READMEs become evidence and detail links |
| Glossary subordinate to entity atlas | `docs/design/entity-relevant-architecture-rust-term-inventory.md`; `docs/glossary*.md`; glossary terms should be linked from entity pages and used only after entity meaning is clear |
| Safety/human-review overlays by entity/action | `docs/design/entity-atlas-review-safety-boundaries.md`; `docs/safety/*.md` become entity/action overlay evidence |
| Rustdoc filler cleanup against entity contract | Use entity template questions and family pages as acceptance for remaining Rustdoc cleanup; legacy guide `docs/rustdoc-operational-language-guide.md` remains a language guide, not the organizing model |
| Final non-coder comprehension QA | Entity-atlas QA cards on `nva-entity-atlas-relationship-map` (`t_e119b371`, `t_67d48829`, `t_50abad2d`, final fan-in `t_1dfbfe2b`) plus legacy QA reports as regression evidence |

Classification values used below: keep as-is; fold into entity atlas; subordinate to entity template; rewrite with entity questions; duplicate of successor board; misrouted implementation card; stale/archive candidate; needs human/product decision.

## Executive mapping

| Old board/output | Classification | Preserve / destination | Rationale |
| --- | --- | --- | --- |
| `nva-noncoder-docs-board-factory` control board | stale/archive candidate | Preserve only its create-board summaries in this artifact and the inventory; do not use as an execution surface. | Inventory says all non-archived cards are done and old implementation/review duplicates are archived. The successor execution surface is the entity-centered board graph. |
| `nva-entity-atlas-relationship-map` | keep as-is | Treat as the successor canonical board. Final fan-in destination: `t_1dfbfe2b`; current QA/remediation: `t_50abad2d`, `t_2218d8b3`. | This is already the entity-centered successor structure and includes inventory, page template, family pages, relationship map, coverage QA, and non-coder QA. |
| `workflow-first-operator-pages` | subordinate to entity template | Preserve `docs/workflows/operator/*.md`, `docs/design/operator-workflow-page-inventory.md`, and source/Rustdoc map by linking them from `docs/design/workflow-to-entity-navigation-map.md` and `docs/design/entity-driven-workflow-page-template.md`. | Workflow pages are valuable reader entrypoints, but the successor spine must start from entities/contracts. |
| `nva-service-line-rustdoc-summaries` | fold into entity atlas | Route boarding/daycare/care/reservation into `docs/design/entity-atlas-petsuites-core-entities.md`; grooming/training/retail/rebooking into `docs/design/entity-atlas-revenue-opportunity-entities.md`; use Rustdocs as evidence links. | The service-line work contains useful operator language, but entity-template pages now own the non-coder narrative. |
| `nva-safety-human-review-docs` | subordinate to entity template | Fold into `docs/design/entity-atlas-review-safety-boundaries.md` and per-entity blocked-action sections. Keep `docs/safety/*.md` as background/regression evidence. | Safety prose should be queryable by entity/action: Message send, payment movement, source cleanup, booking status, provider write, care/vaccine/incident decision. |
| `nva-glossary-translation-layer` | subordinate to entity atlas | Preserve `docs/glossary*.md` and `docs/design/glossary-translation-layer.md`, but route from `docs/design/entity-relevant-architecture-rust-term-inventory.md` and entity pages. | Glossary helps non-coders decode repo/Rust terms, but must not replace entity meaning or source contracts. |
| `nva-labor-cost-doc-contracts` | duplicate of successor board | Treat as historical implementation/Rustdoc contract work. Fold durable source/Rustdoc contract improvements into entity pages as evidence. | It predates/overlaps the entity atlas and is not a direct factory child per inventory. It should not spawn more non-entity docs work. |
| `nva-public-noncoder-docs-landing-navigation` | rewrite with entity questions | Recenter landing/README destinations around `docs/design/entity-index.md`, `docs/design/entity-atlas-audience-paths.md`, and entity family pages; keep existing `docs/public/index.html` and build scripts. | Landing work solved non-coder entry paths, but older content/audience paths were labor/workflow-first. Successor landing should lead with entities, relationships, authority, blocked actions, and outcome proof. |
| `nva-documentation-style-guide-examples` | keep as-is | Keep `docs/quality/nva-documentation-style-guide.md` as the style/anti-slop rulebook for all successor entity docs. | The style guide is reusable process guidance, not a competing content spine. |
| `nva-rustdoc-filler-cleanup-operational-english` | rewrite with entity questions | Continue/harvest remaining cleanup only if each target Rustdoc answers the entity contract: entity meaning, source authority, allowed automation, blocked actions, outcomes, evidence. | Operational English is useful, but the successor acceptance should be entity-contract fidelity, not just absence of filler phrases. |
| `nva-published-docs-qa-noncoder-usability` | fold into final non-coder QA | Keep reports and live-site checks as regression evidence for successor final QA. Final gate should ask whether non-coders can start from entities and understand relationships/safety/value. | Legacy QA exposed crate/jargon-first problems and verified public landing freshness; successor QA should consolidate it under entity-first comprehension. |

## Detailed matrix by old board/card/output

### 1. Old factory control board: `nva-noncoder-docs-board-factory`

| Old board/card/output | Classification | Successor destination | Preserve / rewrite instruction |
| --- | --- | --- | --- |
| Board objective: create focused sub-boards for workflow-first, labor-cost-focused non-coder docs | stale/archive candidate | No successor execution board; retain only as history in `docs/kanban/old-factory-board-inventory-t_5078771f.md` and this map. | The factory is complete/control-only. New work should target the entity-atlas board or a clearly entity-scoped successor card. |
| `t_71ea0de8` create board: entity atlas and relationship map | keep as-is | Board `nva-entity-atlas-relationship-map` | This is the canonical successor board. Preserve its dependency graph and use it as the destination for harvested old work. |
| `t_57ba0608` create board: workflow-first operator pages | subordinate to entity template | `docs/design/workflow-to-entity-navigation-map.md`; `docs/design/entity-driven-workflow-page-template.md`; entity atlas workflow packet family page | Keep board output, but route workflow pages through entities. |
| `t_9e0d96b5` create board: service-line operator summaries | fold into entity atlas | Core/revenue/service-line family pages and source/Rustdoc evidence links | Harvest summaries into entity pages; do not treat service-line Rustdocs as standalone non-coder spine. |
| `t_123f6d82` create board: safety and human-review story | subordinate to entity template | `docs/design/entity-atlas-review-safety-boundaries.md`; per-entity blocked-action rows | Convert broad safety story into action/entity overlays. |
| `t_53684f9a` create board: glossary and translation layer | subordinate to entity atlas | `docs/design/entity-relevant-architecture-rust-term-inventory.md`; glossary links from entity pages | Glossary should decode terms after entity context. |
| `t_7b3aa0cb` create board: public landing/navigation | rewrite with entity questions | README/public landing entity-index route | Keep audience-path and build outputs; rewrite entry framing around entity index, relationships, authority, safety, outcomes. |
| `t_30595bd1` create board: documentation style guide/examples | keep as-is | `docs/quality/nva-documentation-style-guide.md` | Reuse as style acceptance for entity docs. |
| `t_c50d846a` create board: Rustdoc filler cleanup | rewrite with entity questions | Entity-contract cleanup acceptance against family pages and Rustdoc evidence | Keep cleanup guide and completed batches; require entity-template questions for remaining/final verification. |
| `t_aef5f94b` create board: published docs QA | fold into final non-coder QA | Entity-atlas final QA and public rendered smoke tests | Preserve QA reports as regression evidence; consolidate final criteria under entity-first comprehension. |
| Accidental implementation/review duplicates archived by factory QA | stale/archive candidate | None unless a downstream duplicate audit identifies a missing entity artifact. | Do not resurrect archived implementation cards without a specific entity-centered destination. |

### 2. Successor/canonical board: `nva-entity-atlas-relationship-map`

| Old board/card/output | Classification | Successor destination | Preserve / rewrite instruction |
| --- | --- | --- | --- |
| Board objective: entity pages, relationships, contracts, authority, safety boundaries, evidence/outcome fields | keep as-is | Board `nva-entity-atlas-relationship-map` | This is the target successor structure. Use as the destination for all useful legacy work. |
| `t_356b5365` entity universe inventory -> `docs/design/entity-atlas-inventory.md` | keep as-is | Canonical entity atlas spine | Preserve as source-of-truth candidate inventory. |
| `t_8a927a47` entity page/template -> `docs/design/entity-atlas-page-template.md` | keep as-is | Entity template | Use this as the required rewrite frame for subordinate legacy docs. |
| Family pages: `t_4100fd14`, `t_a227a3ca`, `t_8cb0eac1`, `t_4cd912e7`, `t_129263d5`, `t_a6322e65`, `t_f7b8c661`, `t_3b166fc3` | keep as-is | Specific family pages under `docs/design/` and `docs/integrations/gingr/` | These pages are the concrete destinations for old service-line, safety, glossary, workflow, runtime, and provider-boundary work. |
| `t_2de7d3e8` relationship diagrams/adjacency -> `docs/design/entity-atlas-relationships.md` | keep as-is | Relationship/contract crosswalk | Use to preserve cross-board relationships from workflow/service/safety work. |
| `t_e119b371` non-coder comprehension/safety QA | keep as-is | Final non-coder comprehension QA | Use as successor QA baseline; legacy published QA should augment it. |
| `t_67d48829` coverage reconciliation -> `docs/design/entity-atlas-coverage-reconciliation.md` | keep as-is | Final coverage QA | Use to decide whether old outputs are already covered or need explicit entity rows. |
| `t_2218d8b3` Message/message-state remediation | keep as-is | Core entity atlas / review safety overlay | Important example of preserving useful work by turning an indirect old concept into an explicit entity. |
| `t_50abad2d` link/source/rendered smoke QA and `t_1dfbfe2b` final reviewer fan-in | keep as-is | Final successor board closeout | Downstream destinations for evidence from this mapping and duplicate/misroute audit. |

### 3. Workflow-first operator pages

| Old board/card/output | Classification | Successor destination | Preserve / rewrite instruction |
| --- | --- | --- | --- |
| Board objective: labor-saving operator workflows for manager brief, booking triage, data hygiene, checkout, grooming retention, daily updates, regional exceptions | subordinate to entity template | `docs/design/workflow-to-entity-navigation-map.md`; workflow packet family page | Workflows remain useful entrypoints but must start with featured entities/contracts. |
| `t_083f1b1a` inventory/page contract -> `docs/design/operator-workflow-page-inventory.md` | fold into entity atlas | Workflow-to-entity navigation map | Preserve page inventory as a list of workflow examples and required entity backlinks. |
| `t_ac651349` source/Rustdoc backing map -> `docs/design/workflow-page-source-rustdoc-map.md` | fold into entity atlas | Relationship/contract crosswalk and family page evidence sections | Convert workflow source links into entity source/Rustdoc/test evidence links. |
| `t_9d040f5f` drafted `docs/workflows/operator/*.md` | subordinate to entity template | `docs/design/workflow-to-entity-navigation-map.md`; family pages | Keep pages; add/verify entity-first lead and backlinks where successor workflow template requires it. |
| `t_89a4cb7f` non-coder operator review | fold into final non-coder QA | Entity-atlas non-coder QA | Reuse pass/caveats as evidence that workflow examples are understandable after entity backlinks. |
| `t_e669cdf5` local/published verification running at inventory time | needs human/product decision | If completed safely, harvest evidence into final QA; if stale, duplicate/misroute audit should propose safe board action. | Mapping cannot determine outcome; next audit should check current state before action. |
| Regional labor exceptions workflow page | rewrite with entity questions | Outcome/labor/operations family page; runtime/storage/API family page | Keep explicitly planned/future until a dedicated app contract exists. |

### 4. Service-line Rustdoc summaries

| Old board/card/output | Classification | Successor destination | Preserve / rewrite instruction |
| --- | --- | --- | --- |
| Board objective: plain-English operator-summary sections in service-line Rustdocs plus README/navigation | fold into entity atlas | Service-line portions of core/revenue family pages; Rustdoc links as evidence | Preserve the operational language; entity pages own the reader path. |
| `t_30cd0e00` inventory of major service-line Rustdocs and README/navigation | fold into entity atlas | `docs/design/entity-atlas-inventory.md`; service-line family pages | Use to verify source/Rustdoc coverage for service-line entities. |
| Boarding/daycare rewrites (`t_b7ba5a2f`, `t_72b676d3`) | subordinate to entity template | `docs/design/entity-atlas-petsuites-core-entities.md` | Keep as evidence/details for Boarding/Daycare contracts. |
| Grooming/training/retail rewrites (`t_df2558df`, `t_8cd647c0`, `t_408344a5`) | subordinate to entity template | `docs/design/entity-atlas-revenue-opportunity-entities.md` | Keep as evidence/details for revenue/service opportunity entities. |
| Reservations/checkout rewrite (`t_5e4d106c`) | subordinate to entity template | Core entity atlas Reservation plus workflow packet family page | Keep as source/Rustdoc evidence for reservation/checkout entities. |
| Care/documents/temperament/incident rewrite (`t_d02ba7a0`) | subordinate to entity template | Core entity atlas care/vaccine/document/incident/temperament sections | Keep as safety-care evidence, not separate narrative spine. |
| README/Rustdoc navigation update (`t_8b4a89f7`) | rewrite with entity questions | Entity index and public landing | Keep concrete links, but prefer entity-first navigation and label Rustdocs as evidence. |
| Blocked/todo cards `t_f20f1528`, `t_e083fce5`, `t_4b6bc5e3`, `t_05ca48da` | duplicate of successor board | Successor QA/final fan-in if still useful | Likely duplicate of entity-atlas QA/rendered gates. Next audit should decide leave-to-finish vs archive/recreate based on live state and race risk. |

### 5. Safety and human-review docs

| Old board/card/output | Classification | Successor destination | Preserve / rewrite instruction |
| --- | --- | --- | --- |
| Board objective: explain agent safety, source evidence, review gates, blocked actions, outcome capture, labor-cost value | subordinate to entity template | `docs/design/entity-atlas-review-safety-boundaries.md`; per-family blocked-action sections | Preserve content by attaching every safety claim to entity/action authority. |
| `docs/safety/source-evidence-map.md` (`t_6fd088d7`) | fold into entity atlas | Source/provenance atlas; review safety atlas | Use as source evidence layer and cross-check for allowed read/draft/never-do classes. |
| `docs/safety/labor-cost-with-human-review-crosswalk.md` (`t_260c6668`) | fold into entity atlas | Outcome/labor atlas plus review safety atlas | Keep as business explainer; ensure entity pages measure value only via outcomes. |
| `docs/safety/agent-safety-model-for-operators.md` (`t_a44f7bf6`) | subordinate to entity template | Review safety family page | Use as plain-language overview after entity/action rows. |
| `docs/safety/evidence-policy-blocked-actions-outcomes.md` (`t_352e65b8`) | fold into entity atlas | Relationship/contract map and review safety atlas | Preserve the evidence -> policy -> blocked action -> outcome flow as the safety edge model. |
| `docs/safety/review-boundaries-matrix.md` (`t_aefa3db5`) | fold into entity atlas | Review safety boundary rows by entity/action | Convert/verify rows against Message, Booking, Payment, Care/Vaccine/Incident, Provider write, Source cleanup. |
| Final safety integration (`t_17329adf`) | keep as-is | Entity safety overlay QA evidence | Preserve correction about current policy source names as source-grounding evidence. |

### 6. Glossary and translation layer

| Old board/card/output | Classification | Successor destination | Preserve / rewrite instruction |
| --- | --- | --- | --- |
| Board objective: translate repo/Rust architecture terms into NVA operational meaning | subordinate to entity atlas | `docs/design/entity-relevant-architecture-rust-term-inventory.md`; entity pages glossary links | Keep glossary as a decoder. It should not be the first reading path. |
| `docs/glossary-translation-layer-inventory.md` (`t_9cb94d96`) | fold into entity atlas | Entity-relevant term inventory | Use to decide which terms belong in entity pages vs global glossary. |
| `docs/design/glossary-translation-layer.md` (`t_262ecf55`, `t_af5acc72`) | subordinate to entity template | Style/template support for entity pages | Keep entry template only where it supports entity-template questions. |
| `docs/glossary-architecture-terms.md` (`t_9eac0d70`) | rewrite with entity questions | Runtime/storage/API family page, provider-boundary atlas, entity term inventory | Define architecture terms only in relation to entities/contracts and source authority. |
| `docs/glossary-workflow-state-terms.md` (`t_8d7f9c79`) | fold into entity atlas | Workflow packet and review safety family pages | Preserve terms such as draft, review gate, blocked action, outcome capture in entity/action context. |
| `docs/glossary-source-data-terms.md` (`t_b53c9ddc`) | fold into entity atlas | Source/provenance/data-quality atlas | Preserve Gingr/source-of-record/provenance terms as source entity vocabulary. |
| Cross-link/public landing work (`t_1052aab8`, `t_d941d5d8`) | rewrite with entity questions | Entity index and public landing | Keep links but prefer entity-index route before standalone glossary route. |

### 7. Labor-cost doc contracts board

| Old board/card/output | Classification | Successor destination | Preserve / rewrite instruction |
| --- | --- | --- | --- |
| Board objective: Rustdoc/source contract completion across app/domain/storage/Gingr/runtime | duplicate of successor board | Entity family page evidence sections; Rustdoc filler/entity-contract cleanup | Treat as historical source/Rustdoc contract hardening. Do not recreate as separate non-coder board. |
| App workflow docs (`t_043082fb`, `t_b9cb7168`) | fold into entity atlas | Workflow packet family page; workflow-to-entity map | Preserve AgentPromptPacket, WorkflowAgent, Request/Result docs as evidence for packet entities. |
| Rustdoc completeness guardrail (`t_a98fb981`) | keep as-is | Verification/tooling evidence for final QA | Keep guardrail as source contract quality control. |
| Operations/analytics/data-quality docs (`t_fa5065d5`) | fold into entity atlas | Outcome/labor/operations and source/data-quality family pages | Preserve as evidence for operations and data-quality entities. |
| Storage/API/worker/CLI docs (`t_34cc7489`) | fold into entity atlas | Runtime/storage/API family page | Preserve as evidence for projection/runtime shell entities. |
| Gingr source-boundary docs (`t_bc94cec5`) | fold into entity atlas | Gingr provider-boundary atlas | Preserve provider-boundary wording and source authority limits. |
| Service-line Rustdocs (`t_c9ca4e4a`, `t_eb932bc2`) | fold into entity atlas | Core and revenue entity family pages | Preserve service-line contract details as evidence. |
| Domain core/policy/source docs (`t_8e05e65b`) | fold into entity atlas | Source/provenance, review safety, workflow, outcome family pages | Preserve core source/policy/entity contract language. |
| Integrated fan-in commit `t_1b7007ed` | keep as-is | Source/Rustdoc evidence baseline | Keep as stable evidence that contract docs were pushed; successor docs should reference source/Rustdoc, not re-run this board. |

### 8. Public non-coder docs landing/navigation

| Old board/card/output | Classification | Successor destination | Preserve / rewrite instruction |
| --- | --- | --- | --- |
| Board objective: public non-coder landing, audience paths, navigation, rendered verification, publish caveats | rewrite with entity questions | Entity-index-centered README/public landing | Keep public docs machinery but make entity index/audience paths the primary route. |
| `docs/design/public-docs-landing-content-map.md` (`t_790720cb`) | rewrite with entity questions | `docs/design/entity-atlas-audience-paths.md`; entity index | Preserve audience analysis; rewrite route order so each path names entities first. |
| `docs/design/public-docs-landing-technical-inventory.md` (`t_9a810082`) | keep as-is | Public landing technical evidence | Preserve build/deploy/file-surface knowledge. |
| `docs/public/index.html` + build wrapper (`t_6358bc2b`) | rewrite with entity questions | Public landing should link entity index, audience paths, relationship map, workflow-to-entity map | Keep implementation, adjust content only if successor landing is not entity-first. |
| Landing content review (`t_65342085`) | fold into final non-coder QA | Entity landing QA | Preserve safety/audience caveats. |
| Blocked/todo verification/closeout (`t_ce0dfe0b`, `t_045b6715`) | duplicate of successor board | Successor final rendered/public QA | Likely overlaps with entity-atlas rendered smoke/final fan-in. Next duplicate audit should check live state before archive/recreate. |

### 9. Documentation style guide/examples

| Old board/card/output | Classification | Successor destination | Preserve / rewrite instruction |
| --- | --- | --- | --- |
| Board objective: practical non-coder docs style guide, examples, prompt wiring, validation rewrite, review | keep as-is | `docs/quality/nva-documentation-style-guide.md` | Keep as cross-cutting style guide for all entity docs. |
| Source audit (`t_af46bff0`) | fold into entity atlas | Entity-doc QA anti-slop checklist | Reuse anti-slop requirements to review entity pages. |
| Draft guide (`t_d0d8e922`) and examples (`t_c7ce1077`) | keep as-is | Style guide | Preserve; cite in future entity docs cards. |
| Prompt wiring (`t_b35793b0`) | keep as-is | Future Kanban docs-card prompts | Reuse but add entity-template requirement in future prompts. |
| Validation rewrite sample (`t_d73383b1`) | subordinate to entity template | Example only | Useful as style example, not successor content spine. |
| Final review/closeout (`t_c2b91c1b`) and docs-gate fix (`t_0fea2331`) | keep as-is | Verification/tooling support | Preserve gates and caveats for future entity docs validation. |

### 10. Rustdoc filler cleanup / operational English

| Old board/card/output | Classification | Successor destination | Preserve / rewrite instruction |
| --- | --- | --- | --- |
| Board objective: replace templated/generated Rustdoc filler with operational English | rewrite with entity questions | Entity-contract cleanup acceptance; family pages as truth map | Keep completed cleanup, but judge remaining work by entity contract fidelity. |
| Smell inventory (`t_28a6ff73`) | fold into entity atlas | Rustdoc cleanup QA + entity evidence coverage | Use to target weak source/Rustdoc evidence for entity pages. |
| Rewrite examples/guide (`t_376c14eb`) | keep as-is | Rustdoc language guide | Keep as style support. Add entity-template checks when used. |
| Batches 1-4 (`t_e2acf76c`, `t_ec1787f9`, `t_a4ea1211`, `t_3e5c6e23`) | fold into entity atlas | Corresponding service/core/Gingr/runtime family pages as evidence links | Preserve source improvements as evidence; no separate narrative required. |
| Running reviewer pass `t_44c1c1b6` and final smell search `t_740e8d37` | needs human/product decision | If live/complete, harvest into entity-contract QA; if stale, next audit should propose safe board action. | Mapping cannot safely infer current run result from inventory alone. Avoid racing shared checkout. |

### 11. Published docs QA / non-coder usability

| Old board/card/output | Classification | Successor destination | Preserve / rewrite instruction |
| --- | --- | --- | --- |
| Board objective: published/local docs QA, non-coder entry paths, jargon/link checks, freshness, final ops-leader usability | fold into final non-coder QA | Entity-atlas final QA, public landing smoke, link/jargon regression checks | Keep reports and methods. Reframe acceptance around entity-first comprehension. |
| Smoke/freshness/jargon/representative page QA (`t_2b722944`, `t_01ca4843`, `t_9ed98528`, `t_1de55523`, `t_3a676d2b`) | fold into final non-coder QA | `docs/design/entity-atlas-non-coder-qa-findings.md`; rendered public smoke evidence | Preserve as regression corpus: no broken links, live freshness, jargon/crate-first risks. |
| Public landing fix/publish (`t_bfbd7aed`, `t_fff2103c`) | rewrite with entity questions | Entity-centered public landing | Preserve implementation/publish evidence; update route if landing is still not entity-index-first. |
| Final ops-leader usability review (`t_e0fb9a2e`) | fold into final non-coder QA | Entity-atlas final fan-in | Preserve pass-with-caveats and specific non-coder caveats. |
| Representative page next-step framing (`t_9cd93304`) | subordinate to entity template | Family pages and workflow-to-entity map | Preserve page-level framing, but ensure it points back to entities/contracts. |

## Destination summary by successor bucket

| Successor bucket | Legacy inputs to preserve | Concrete successor destination |
| --- | --- | --- |
| Canonical entity atlas spine | Entity inventory/template/family pages; source/Rustdoc contract work; service-line summaries; glossary source maps | `nva-entity-atlas-relationship-map`; `docs/design/entity-index.md`; `docs/design/entity-atlas-inventory.md`; family atlas pages |
| Relationship/contract crosswalk | Workflow source/Rustdoc map; safety evidence-policy-outcome docs; labor contract source docs | `docs/design/entity-atlas-relationships.md`; source/Rustdoc/test evidence sections in family pages |
| README/public landing centered on entities | Public landing content/technical inventory; published QA; glossary links | README and `docs/public/index.html` should lead to `docs/design/entity-index.md`, `docs/design/entity-atlas-audience-paths.md`, and relationship map before crate/workflow links |
| Workflow pages driven by featured entities/contracts | Operator workflow page set and non-coder review | `docs/design/workflow-to-entity-navigation-map.md`; `docs/design/entity-driven-workflow-page-template.md`; `docs/workflows/operator/*.md` as examples |
| Service-line docs applying entity template | Boarding/daycare/care/reservation/grooming/training/retail Rustdoc summaries | Core and revenue family pages; service Rustdocs/READMEs as evidence |
| Glossary subordinate to entity atlas | Glossary inventory, architecture terms, workflow-state terms, source-data terms | `docs/design/entity-relevant-architecture-rust-term-inventory.md`; `docs/glossary*.md` as linked decoder pages |
| Safety/human-review overlays by entity/action | Safety source map, review-boundaries matrix, blocked-action explainers | `docs/design/entity-atlas-review-safety-boundaries.md`; per-entity blocked/human-reviewed rows |
| Rustdoc filler cleanup against entity contract | Rustdoc smell inventory, operational language guide, four rewrite batches | Entity family pages as acceptance; Rustdoc/source paths as evidence links |
| Final non-coder comprehension QA | Published QA reports, workflow review, entity QA, coverage reconciliation, rendered smoke | `docs/design/entity-atlas-non-coder-qa-findings.md`; `docs/design/entity-atlas-coverage-reconciliation.md`; `t_50abad2d`; `t_1dfbfe2b` |

## Items requiring follow-up triage by the duplicate/misroute audit

These are not owner/product blockers for this mapping artifact; they are inputs for child task `t_2c4c4c2f`.

| Item | Why triage is needed | Suggested safe default for next audit |
| --- | --- | --- |
| Service-line blocked/todo closeout cards `t_f20f1528`, `t_e083fce5`, `t_4b6bc5e3`, `t_05ca48da` | They may duplicate entity-atlas QA/rendered gates, but could still hold useful verification evidence. | Check current board state; if no unique entity evidence remains, archive or harvest into successor QA. |
| Public landing verification/closeout `t_ce0dfe0b`, `t_045b6715` | Likely overlaps with successor entity landing/QA. | Check whether entity-index landing route exists; leave-to-finish only if it verifies current entity-centered landing. |
| Rustdoc filler cleanup reviewer/final cards `t_44c1c1b6`, `t_740e8d37` | Running/todo in inventory; could race with shared checkout. | Let live worker finish if healthy, then harvest into entity-contract cleanup QA; avoid duplicate source edits. |
| Workflow verification `t_e669cdf5` | Running in inventory. | Harvest verification into final entity workflow QA if completed; otherwise avoid broad parallel verification. |
| `nva-labor-cost-doc-contracts` board | Adjacent legacy board, not direct factory child; overlaps with source/Rustdoc evidence work. | Do not dispatch new work there; treat as archive/history unless a specific missing entity evidence gap appears. |

## Final mapping rule

A legacy output is preserved when it can answer one of the entity-template questions for a named entity or relationship. If it only improves generic labor-cost prose, workflow prose, glossary wording, safety explanation, or Rustdoc style without naming the entity, source authority, allowed automation, blocked action, outcome proof, and evidence path, it should be rewritten with entity questions before being treated as successor-ready.
