# QA: non-coder jargon and glossary audit

Task: `t_41168ab8`
Audit date: 2026-06-19
Workspace: `/home/eran/code/nva`

## Scope and method

This was a representative source-doc audit for operator/owner comprehension, not a full rewrite pass. I read and/or scanned:

- Entity and navigation docs: `README.md`, `domain/README.md`, `app/README.md`, `storage/README.md`, `integrations/gingr/README.md`, and service-line entity summaries under `domain/src/{boarding,daycare,grooming,training,retail,reservation}/README.md`.
- Operator workflow pages: `docs/workflows/operator/*.md`.
- Canonical workflow/spec docs: `docs/workflows/booking-triage-agent.md`, `docs/workflows/daily-care-update-agent.md`, and `docs/workflows/crm-retention-agent.md`.
- Glossary and translation-layer pages: `docs/glossary.md`, `docs/glossary-architecture-terms.md`, `docs/glossary-source-data-terms.md`, `docs/glossary-workflow-state-terms.md`, `docs/glossary-translation-layer-inventory.md`, and `docs/design/glossary-translation-layer.md`.

Automated support: a simple term scan counted likely coder/internal terms such as `domain`, `app`, `storage`, `DTO`, `provenance`, `source ref`, `packet`, `deterministic`, `projection`, `promotion`, `codec`, `payload`, `endpoint`, `webhook`, `typestate`, `schema`, `refs`, and related terms. The count was used only as triage evidence; the findings below are based on page context and non-coder readability.

## Executive summary

The docs are materially better than a raw engineering wiki: operator workflow pages now lead with audience, problem solved, source data, allowed drafts/ranking, human approval, measured outcome, and code/Rustdoc evidence. The glossary index also defines the main repo/Rust terms that would otherwise block non-coders.

The remaining comprehension risk is not absence of a glossary. It is that many entity and crate pages still ask operators to read a dense cluster of internal terms before they get a concrete pet-resort example. The highest-priority fixes are therefore short lead-in/callout edits, not broad terminology removal. Keep precise terms where they protect authority boundaries, but pair them with plain-English labels on first use.

## Terms that are acceptable when glossary-linked

These terms are acceptable in non-coder docs if they are linked near first use and paired with an operational label. They already have glossary coverage:

| Term | Glossary coverage | Use condition |
| --- | --- | --- |
| `domain` | `docs/glossary-architecture-terms.md#domain` | Use as “source-of-truth business vocabulary (`domain`)” before later shorthand. |
| `app` | `docs/glossary-architecture-terms.md#app` | Use as “workflow/review-bundle layer (`app`)”; avoid letting non-coders think it means mobile/web app. |
| `storage` | `docs/glossary-architecture-terms.md#storage` | Use as “durable records/projections, not the decision-maker.” |
| `integrations/gingr`, `integration` | `docs/glossary-architecture-terms.md#integration-integrationsgingr` | Use as “Gingr boundary/source adapter,” and state live connectivity/support limits where relevant. |
| `adapter` | `docs/glossary-architecture-terms.md#adapter` | Use with “translator at an outside-system boundary.” |
| `DTO` | `docs/glossary-architecture-terms.md#dto` | Use with “provider payload shape (DTO), not business truth.” |
| `source ref` / `RecordRef` | `docs/glossary-architecture-terms.md#source-ref-domainsourcerecordref` | Use with “source pointer/evidence handle.” |
| `provenance` | `docs/glossary-architecture-terms.md#provenance-domainsourceprovenance` | Use with “lineage/evidence trail,” and avoid implying correctness. |
| `source-of-record` | `docs/glossary-source-data-terms.md#source-of-record` | Always say source-of-record for which fact/action. |
| `data-quality issue` | `docs/glossary-source-data-terms.md#domaindata_qualityissue-data-quality-issue` | Use with “source data exception needing review/tracked resolution.” |
| `draft` | `docs/glossary-workflow-state-terms.md#draft` | Prefer “staff-review draft” on customer/provider/payment pages. |
| `review gate` | `docs/glossary-workflow-state-terms.md#review-gate` | Use with “required human/system approval before action.” |
| `blocked action` | `docs/glossary-workflow-state-terms.md#blocked-action` | Use with “requires approved human/system action,” not product failure. |
| `outcome capture` | `docs/glossary-workflow-state-terms.md#outcome-capture` | Use with “recording what happened and labor result,” not performing the action. |
| `workflow packet` | `docs/glossary-workflow-state-terms.md#workflow-packet` | Use with “review bundle.” |
| `agent spec` | `docs/glossary-workflow-state-terms.md#agent-spec` | Use with “operating contract for a bounded automation helper.” |
| `tool port` | `docs/glossary-architecture-terms.md#tool-port-apptools` | Use with “approved integration capability interface.” |
| `read model` | `docs/glossary-architecture-terms.md#read-model` | Use with “reporting/review projection, not source-of-record.” |
| `provider record` | `docs/glossary-source-data-terms.md#provider-record` | Use with “provider-native evidence, not canonical customer/pet/reservation truth.” |

## Undefined or under-defined jargon that still blocks comprehension

These terms appear in representative docs but do not yet have enough glossary/index support for owner/operator readers:

| Priority | Term or phrase | Example pages/anchors | Why it blocks comprehension | Suggested replacement or glossary action |
| --- | --- | --- | --- | --- |
| P0 | `contract` / `contracts` | `README.md#documentation-contracts`, `domain/README.md#readme-vs-rustdoc-contract`, service-line READMEs | “Contract” is used as an internal code promise, API guarantee, and workflow boundary. Owners may read it as legal/vendor contract. | On non-coder pages, use “source-backed rule/promise” or “workflow rule.” If kept, add a glossary entry for “contract in this repo.” |
| P0 | `semantic` / `semantic truths` | `README.md#workspace-map`, `domain/README.md`, service-line READMEs | Accurate for maintainers, opaque to operators. | Replace first use with “business meaning/source-of-truth vocabulary,” then keep `semantic` in parentheses only when module ownership matters. |
| P0 | `projection` / `persistence projection` | `README.md#workspace-map`, `storage/README.md`, glossary pages | Operators may not know why a reporting/database shape is not truth. | Use “reporting/database-friendly copy” or “durable view of the record”; consider a glossary sub-entry for “projection.” |
| P0 | `promotion` / `demotion` | `README.md#type-module-map`, `storage/README.md`, `integrations/gingr/README.md`, `integrations/gingr/src/mapping/README.md` | Sounds like staff/job status rather than data conversion/validation. | Use “translate and validate into business meaning” / “convert back to storage/provider shape.” Add glossary coverage because this term recurs at source boundaries. |
| P1 | `codec` / `stable codes` | `README.md#workspace-map`, `storage/README.md`, `storage/src/service_line/README.md` | Pure implementation vocabulary on pages that also try to explain business safety. | Use “database-safe codes and converters” on operator pages; keep `codec` for maintainer sections. |
| P1 | `quarantined` | `glossary-translation-layer-inventory.md`, `integrations/gingr/README.md`, DTO/mapping docs | Useful metaphor but may sound security-only or punitive. | Define as “kept separate until reviewed/validated.” Use “separated raw provider data” in public prose. |
| P1 | `source-agnostic` | `README.md#type-module-map`, `integrations/gingr/README.md`, mapping docs | Internal architecture phrase; operators need to know it means “not tied to Gingr-only wording.” | Replace first use with “provider-neutral / not Gingr-specific.” |
| P1 | `payload` | Workflow/spec docs and Gingr docs | Technical term for raw request/response bodies; can hide whether data is trusted. | Use “raw provider data/body” or “data package” depending on context. Add “payload” to glossary if it remains in public pages. |
| P1 | `endpoint`, `transport`, `webhook` | `README.md#workspace-map`, `integrations/gingr/README.md`, Gingr module READMEs | Integration details appear before operational explanation on source pages. | Use “API request,” “connection layer,” and “event notification” on first use. Keep technical terms in maintainer tables. |
| P1 | `runtime shell` | `README.md#workspace-map`, `domain/README.md`, app/runtime README links | Maintainer phrase; owner may read “shell” as terminal. | Use “the API/worker/CLI program that runs the workflow,” then technical term in parentheses. |
| P1 | `deterministic` | Workflow/operator pages and workflow specs | Important safety term, but operators need “rule-based / same input gives same result.” | Replace first use with “rule-based deterministic check.” Add glossary or callout if used broadly. |
| P2 | `typed`, `scalar`, `enum`, `aggregate`, `invariant`, `builder`, `typestate`, `doctest` | `README.md#rust-quality-conventions`, `domain/README.md`, generated Rustdoc/API item pages | Appropriate in maintainer sections, but not helpful in operator-facing summaries. | Keep in developer-only sections. For operator pages, translate to “validated field,” “allowed value list,” “complete record,” “safety rule,” “guided setup,” “compile-checked example.” |
| P2 | `refs`, `evidence refs`, `policy refs` | Workflow specs and operator pages | Abbreviation is easy for maintainers, but non-coders may miss that these are traceable source pointers. | Prefer “evidence links/source pointers” on first use; keep `_refs` only inside schemas. |
| P2 | `structured output`, `schema`, `canonical`, `MVP`, `BI` | Workflow specs and architecture/docs plans | Acceptable for implementation specs, but should not lead non-coder summaries. | Define locally or move after plain-language workflow purpose. |

## Prioritized fix list

### P0. Add a tiny “plain-English first” callout to crate/entity pages

Pages/anchors:

- `domain/README.md#domain`
- `app/README.md#app`
- `storage/README.md#storage`
- `integrations/gingr/README.md#integrationsgingr`
- `domain/src/boarding/README.md#domainboarding`
- `domain/src/daycare/README.md#domaindaycare`
- `domain/src/grooming/README.md#domaingrooming`
- `domain/src/training/README.md#domaintraining`
- `domain/src/retail/README.md#domainretail`
- `domain/src/reservation/README.md#domainreservation`

Problem: These pages are source-accurate but often start with `domain`, `semantic`, `crate`, `contract`, `module`, `payload`, `storage codes`, or type paths before a non-coder gets the concrete resort job.

Suggested pattern:

```md
Operator translation: this page is about <resort job>. It helps staff/managers <save/review/prevent>. In code, that business meaning lives in `<path>` so workflows can use it without guessing or bypassing review.
```

Example replacement for `domain/src/grooming/README.md` first paragraph:

```md
Operator translation: grooming pages describe how the system helps staff estimate appointments, spot no-show/rebooking work, prepare reminder or follow-up drafts, and keep customer-facing grooming decisions under review. In code, that business meaning lives in `domain::grooming` so provider records, calendar notes, and storage rows do not become policy by accident.
```

### P0. Add glossary coverage for “contract,” “semantic,” “projection,” and “promotion/demotion”

Pages/anchors:

- `docs/glossary.md#architecture-and-layer-terms`
- `docs/glossary-architecture-terms.md`
- `docs/glossary-source-data-terms.md`

Problem: These terms are among the most repeated undefined terms in representative entity docs. They carry important architecture meaning, so replacing all uses would harm precision.

Suggested entries:

- `contract`: “source-backed rule or code promise; not a legal/customer contract.”
- `semantic`: “business meaning after validation; not just text or model interpretation.”
- `projection`: “database/reporting-friendly view of facts; not the authority for live decisions.”
- `promotion/demotion`: “explicit conversion between raw/provider/storage shapes and validated business meaning.”

### P0. Retitle or qualify “draft” glossary pages before public readers hit them

Pages/anchors:

- `docs/glossary-architecture-terms.md#architecture-glossary-draft-for-non-coder-nva-docs`
- `docs/glossary-source-data-terms.md#glossary-draft-source-system-and-data-quality-terms`
- `docs/glossary-workflow-state-terms.md#glossary-draft-workflow-and-operator-state-terms`

Problem: The glossary term pages call themselves “draft.” That is honest for maintainers, but a non-coder can read it as not approved or not safe to rely on. It also overloads the glossary term “draft.”

Suggested replacement:

- “Architecture glossary for non-coder NVA docs”
- “Source-system and data-quality glossary”
- “Workflow and operator-state glossary”

If draft status must remain, put it in metadata or a maintainer note: “Maintainer status: reviewed against current source as of YYYY-MM-DD; update when source contracts change.”

### P1. Move the glossary-link list below a one-sentence workflow explanation on operator pages

Pages/anchors:

- `docs/workflows/operator/manager-daily-brief.md#glossary-links-used-on-this-page`
- `docs/workflows/operator/booking-triage.md#glossary-links-used-on-this-page`
- `docs/workflows/operator/data-quality-hygiene.md#glossary-links-used-on-this-page`
- `docs/workflows/operator/checkout-completion.md#glossary-links-used-on-this-page`
- Same pattern likely applies to `grooming-rebooking-retention.md`, `daily-updates-pawgress-drafts.md`, and `regional-labor-exceptions.md`.

Problem: The operator pages are understandable, but the first substantial section after status is a list of jargon terms. This makes the page feel glossary-first instead of job-first.

Suggested structure:

1. Audience.
2. Status/safety boundary.
3. One-sentence plain-English workflow summary.
4. “Helpful glossary links” collapsible/list section.
5. Problem/time/source/draft/approval/outcome/evidence sections.

Example addition before the glossary list on `booking-triage.md`:

```md
Plain-English summary: this workflow helps front-desk staff sort booking requests into “ready for review,” “missing information,” “vaccine/document pending,” “special review,” or “waitlist/rejection needs approval,” without promising space or changing Gingr by itself.
```

### P1. Replace abbreviation-heavy `refs` language outside schemas

Pages/anchors:

- `docs/workflows/operator/manager-daily-brief.md#3-what-source-data-does-it-need`
- `docs/workflows/operator/booking-triage.md#3-what-source-data-does-it-need`
- `docs/workflows/operator/data-quality-hygiene.md#3-what-source-data-does-it-need`
- `docs/workflows/booking-triage-agent.md#rule-contract`
- `docs/workflows/daily-care-update-agent.md#input-packet`
- `docs/workflows/crm-retention-agent.md`

Problem: `source refs`, `evidence_refs`, `policy refs`, and `audit refs` are clear to implementers but not to owners.

Suggested replacement:

- In prose: “source pointers/evidence links” or “traceable evidence pointers.”
- In schemas: keep exact field names, but add a plain-language parenthetical: `evidence_refs` (traceable source/evidence pointers).

### P1. Make source/integration docs answer “what does this connect to?” before “how is it shaped?”

Pages/anchors:

- `integrations/gingr/README.md#integrationsgingr`
- `integrations/gingr/src/dto/README.md`
- `integrations/gingr/src/endpoint/README.md`
- `integrations/gingr/src/mapping/README.md`

Problem: Gingr pages correctly protect source boundaries, but they quickly switch to `endpoint`, `transport`, `DTO`, `webhook`, `mapping`, `source-agnostic`, and `promotion`.

Suggested lead:

```md
Operator translation: Gingr is the outside pet-resort operating system this repo reads as evidence. These pages explain how we keep raw Gingr data separate, check what it means, and pass only reviewed/validated facts into NVA workflows. They do not mean the repo can change Gingr live unless a separate approved runtime path exists.
```

### P1. Distinguish implementation specs from operator pages in workflow docs

Pages/anchors:

- `docs/workflows/booking-triage-agent.md#booking-triage-agent`
- `docs/workflows/daily-care-update-agent.md#daily-care-update-agent`
- `docs/workflows/crm-retention-agent.md#crm-retention-agent`

Problem: These are long canonical workflow/spec artifacts, not quick owner pages. They include schemas, rule ids, JSON, status enums, source refs, and deterministic rule language. They are acceptable for implementation planning but should not be the first non-coder destination.

Suggested fix:

- Add a top callout: “If you are a resort operator, start with `docs/workflows/operator/<page>.md`; this page is the implementation/spec evidence behind it.”
- Keep the current glossary help line, but move it after the start-here callout.

### P2. Keep developer-only Rust quality terms out of operator summaries

Pages/anchors:

- `README.md#rust-quality-conventions`
- `domain/README.md#maintainer-notes`
- generated Rustdoc/API item pages, especially typestate/builder-heavy workflow structs

Problem: Terms such as `nutype`, `bon`, `statum`, `typestate`, `builder`, `invariant`, `enum`, `scalar`, `doctest`, and `crate` are fine for maintainers, but they should not be required for operator comprehension.

Suggested fix:

- Leave these in maintainer sections.
- Add “Maintainer note” labels where these terms appear.
- If a generated Rustdoc item is linked from an operator page, add/keep a human-readable “Operational meaning” paragraph before the generated signature details.

## Glossary and translation-layer readability check

### What is understandable

- `docs/glossary.md` is a useful index. It groups terms by architecture/layer, source/data, and workflow/state, and tells writers to link terms near first use.
- `docs/glossary-architecture-terms.md`, `docs/glossary-source-data-terms.md`, and `docs/glossary-workflow-state-terms.md` generally use the right entry shape: term, plain-language label, where it appears, code-derived contract, operational meaning, why it matters, what not to infer, boundary/authority, evidence hooks, suggested wording, and related terms.
- `docs/design/glossary-translation-layer.md` correctly teaches maintainers how to preserve source truth while translating for operators.

### What still needs cleanup

- The glossary pages themselves are jargon-dense by design. That is acceptable for a glossary, but public readers need the title/status wording to feel authoritative rather than “draft.”
- `docs/glossary.md` line 3 uses “source-derived contract” before defining “contract.” Suggested replacement: “source-backed meaning from the code/docs.”
- `docs/glossary.md` line 5 says “crate READMEs” and “Rustdoc-facing docs.” Suggested replacement: “technical README and API-reference pages (Rustdoc).”
- `docs/design/glossary-translation-layer.md` is a maintainer guide, not a public glossary. It is understandable for doc writers, but too procedural for operators. Keep it under `docs/design/`; link public readers to `docs/glossary.md` instead.
- `docs/glossary-translation-layer-inventory.md` is valuable but table-heavy and source-location-heavy. Treat it as source inventory for maintainers, not the operator-facing glossary landing page.

## Representative jargon-density triage

The simple scanner is intentionally rough, but it highlighted the pages where a non-coder is most likely to hit terms before examples:

| Page | Jargon hits per 100 words | Interpretation |
| --- | ---: | --- |
| `docs/glossary.md` | 3.91 | Acceptable for index, but title/purpose should avoid undefined “contract/crate” terms. |
| `domain/src/grooming/README.md` | 3.30 | Needs a concrete operator translation before module/type lists. |
| `docs/glossary-architecture-terms.md` | 3.24 | Acceptable for glossary, but remove/qualify “draft” status. |
| `storage/README.md` | 3.22 | Needs “filing cabinet, not decision-maker” lead before projection/codec language. |
| `docs/design/glossary-translation-layer.md` | 3.15 | Good maintainer guide; not an operator page. |
| `domain/src/boarding/README.md` | 3.11 | Needs a concrete operator translation before module/type lists. |
| `domain/src/reservation/README.md` | 3.06 | Strong safety boundary, but still dense with type/module terms. |
| `domain/README.md` | 2.96 | Good navigation; add more plain-English labels before type tables. |
| `integrations/gingr/README.md` | 2.87 | Needs “what this connects to” lead before DTO/endpoint/webhook language. |
| `README.md` | 2.69 | Improved by the glossary map, but still relies on undefined `contract`, `semantic`, `projection`, and `promotion`. |
| `docs/workflows/operator/*.md` | about 1-2 | Generally understandable; improve by moving glossary term lists below workflow summary. |

## Recommended acceptance order for follow-up cards

1. Add glossary entries for `contract`, `semantic`, `projection`, and `promotion/demotion`; update `docs/glossary.md` index.
2. Add one-sentence operator translation callouts to service-line/entity README openings.
3. Retitle public glossary term pages to remove “draft” from reader-facing H1s.
4. Reorder operator workflow pages so job summary comes before glossary-link list.
5. Add start-here callouts from implementation workflow specs to operator workflow pages.

## Caveats

- This audit did not change the source docs other than producing this report.
- The automated term density counts are heuristics. They overcount legitimate glossary/index pages and undercount hard-to-understand plain English. Use the prioritized fix list, not the raw counts, as the action plan.
- Some technical terms should remain visible because they preserve authority boundaries and keep docs from overstating product readiness. The desired fix is usually “define and pair with operator language,” not “delete.”
