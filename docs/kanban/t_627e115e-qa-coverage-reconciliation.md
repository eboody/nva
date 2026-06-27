# QA synthesis: coverage reconciliation

Task: `t_627e115e`
Date: 2026-06-19
Scope: reconcile representative non-coder QA matrix plus service-line, workflow/contract, Gingr/source, app-packet/review-gate, outcomes/value, runtime/storage, published-site/link, Rustdoc/evidence, and jargon-audit handoffs.

## Executive synthesis

Overall docs coverage is strong enough for owner review of the entity-centered documentation system, with two launch-blocking classes that should be resolved before publishing the public experience as ready for non-coder readers:

1. Published-site / GitHub-main link blocker: the generated local Rustdoc landing page works, but many landing links point to new local docs that are not yet available on GitHub `main` and return 404.
2. P0 non-coder language blocker: core terms used across public/operator pages (`contract`, `semantic`, `projection`, `promotion/demotion`) still need glossary entries or plain-English first-use translations; several page openings should lead with operator meaning before crate/module vocabulary.

Everything else is follow-up or polish: grooming/training per-entry atlas rows, localized stale Rustdoc wording, storage README discoverability for data-quality hygiene outcome records, runtime-shell discoverability, and future outcome/metric ownership for not-yet-durable workflows.

## Unified coverage table

| Entity / question area | Reconciled verdict | Evidence from lanes | Gap, blocker, or follow-up |
| --- | --- | --- | --- |
| Representative matrix / entity-first QA spine | PASS | Seed matrix covered 15 representative entities across source/provenance, core resort entities, service-line contracts, workflow packets, review gates, Gingr/provider boundary, outcomes/labor, and storage/runtime. Downstream lanes used the README/entity-index Q1-Q9 questions. | Follow-up only: keep this matrix as the owner-review checklist; no additional seed work needed. |
| Boarding contract | PASS | Service-line lane found a non-coder can answer all README questions from README -> entity index -> core atlas -> boarding README. Evidence paths cover domain, storage, tests, workflows, authority, allowed automation, and blocked actions. | Polish: add direct backlinks from boarding README to entity-index/core atlas anchors once anchors stabilize. |
| Daycare contract | PASS | Service-line lane found strong plain-English coverage for attendance/package/ratio/group-play/front-desk decisions, with clear authority and blocked-action boundaries. | Polish: add direct backlinks from daycare README to entity-index/core atlas anchors once anchors stabilize. |
| Grooming contract / grooming rebooking cadence | FOLLOW-UP | Service-line and workflow lanes agree grooming is understandable and correctly review-gated. Revenue-opportunity atlas plus grooming README answer the questions, but readers must combine pages. | Follow-up: add a dedicated per-entry Grooming Contract / rebooking row or subsection in `docs/design/entity-atlas-revenue-opportunity-entities.md` with the same source-of-record, human role, allowed action, blocked action, outcome, and evidence fields used for boarding/daycare. Not launch-blocking if owner packet calls out the caveat. |
| Training contract / package-session opportunity | FOLLOW-UP | Service-line lane found training understandable, with package/session value, trainer capacity, payment/source evidence, and blocked outcome claims preserved. Like grooming, the answer is split between revenue atlas and README. | Follow-up: add a dedicated per-entry Training Contract / package-session row or subsection in `docs/design/entity-atlas-revenue-opportunity-entities.md`. Not launch-blocking if called out. |
| Retail / commerce opportunity entities | FOLLOW-UP | Contract-crosswalk and Gingr/source lanes found retail product candidates and commerce evidence are carefully framed as provider evidence and mapping candidates, not live POS/inventory authority. | Follow-up: value claims remain hypothetical until workflow/outcome dispositions exist; no launch blocker for current docs because caveats are explicit. |
| Source system, source fact, source ref, provenance | PASS | Gingr/source and crosswalk lanes agree docs clearly separate provider evidence from NVA-derived domain/app/storage truth. Source refs/provenance are described as evidence handles, not correctness or approval. | Polish: keep asking “source of record for what?” near future BI/read-model pages. |
| Gingr endpoint / DTO / response / webhook / mapping candidate | PASS WITH JARGON FOLLOW-UP | Authority lane found no unsafe source/provider claims. Crosswalk lane found DTOs/webhooks remain quarantined until mapper/source contracts promote them. | P1 language follow-up: integration pages should answer “what does this connect to?” before `endpoint`, `transport`, `DTO`, `webhook`, `mapping`, `source-agnostic`, and `promotion` detail. |
| Customer / pet parent; pet/care/vaccine/temperament/incident facts | PASS | Gingr/source, workflow, and app-packet lanes found source facts are evidence inputs while safety, document, care, behavior, and policy exceptions remain human/system reviewed. | No blocker. Maintain current source-evidence vs approval distinction. |
| Reservation / stay / status / capacity / deposit/payment facts | PASS | Workflow and service-line lanes found booking triage and service-line contracts preserve staff/manager authority for confirmations, denials, waitlists, capacity holds, room/group assignments, provider writes, and payment/deposit actions. | No blocker. Durable booking-triage outcome storage remains a correctly labeled future gap, not a hidden claim. |
| Booking triage packet | PASS | Workflow/contract, app-packet/review-gate, Rustdoc/evidence, and crosswalk lanes agree the packet is a staff-review bundle with deterministic readiness results, evidence refs, review gates, and blocked actions. | Follow-up only: no dedicated durable booking outcome projection; docs already say this. |
| Checkout completion packet | PASS | App-packet/review-gate and workflow lanes found automation may draft/handoff/audit but not move payments, provider state, customer messages, or policy exceptions. Rustdoc/evidence lane sampled source/tests/Rustdoc support. | Follow-up only: durable checkout outcome storage remains not claimed. |
| Daily update / Pawgress draft + message entity | PASS | Workflow and app-packet lanes found strong review-gated customer-message language: draft/preview, included/omitted facts, approval record, blocked send stub. | Follow-up only: durable writing-time outcome storage is not yet identified and should not be overclaimed. |
| Manager Daily Brief packet + outcome record | PASS | Workflow lane selected this as the cross-entity-heavy workflow and found strong evidence from source facts through ranked actions, labor estimates, and storage outcome records. Outcomes/value lane found this is one of the strongest measured-value paths. Crosswalk and Rustdoc lanes found source/test/Rustdoc support. | No blocker. Keep claims scoped to reviewed local labor outcomes, not production ROI. |
| Data-quality hygiene packet / issue / action / outcome | PASS | Workflow, outcomes/value, runtime/storage, and crosswalk lanes agree this is a strong source/provenance cleanup path with durable outcome storage and clear human gates for destructive/source-sensitive changes. | Follow-up: storage README should surface `DataQualityHygieneOutcomeRecord`, outcome codes, and labor-minute scalar alongside manager daily brief records. |
| Grooming rebooking / CRM retention packet | PASS WITH FOLLOW-UP | Workflow and outcomes/value lanes found comprehension is strong and caveats are explicit: app-level outcome record exists, customer contact remains reviewed, and durable grooming-retention storage is not claimed. | Follow-up: keep app-local/fixture value claims until durable outcome projection or metric ownership exists. |
| Regional labor exceptions / portfolio value view | FOLLOW-UP / PLANNED | Workflow and outcomes/value lanes agree the page is intentionally future/planned and uses indirect evidence only. | Follow-up: do not present as launched until dedicated app packet, API endpoint, tests, durable regional outcome record, and metric ownership exist. Not a contradiction because current docs label the gap. |
| Review gate / approval record / blocked action | PASS | App-packet/review-gate lane found all audited docs restrict automation to draft/rank/summarize/validate/route/record-reviewed-outcome and preserve human lanes for sends, provider/PMS mutations, booking/status/schedule changes, payment movement, safety/medical/behavior/policy approvals, and sensitive payload exposure. | No blocker. This is a cross-lane strength and should be highlighted in owner packet. |
| Outcome record / labor minutes / operations analytics | PASS WITH FOLLOW-UP | Outcomes/value lane found no launch-blocking comprehension gap; manager daily brief and data-quality hygiene are strong. Runtime/storage lane confirms durable evidence surfaces and tests. | Follow-up: before pilot/live ROI claims, define production metric ownership: estimated vs actual minutes, rework reduction, payroll variance, adoption, BI/read-model ownership, and regional metric definitions. |
| Storage projection + storage service-line records | PASS WITH LOCALIZED DOC FIXES | Runtime/storage lane found storage docs are coherent and source/Rustdoc/test-backed. Storage projections are described as durable evidence/reporting views, not decision makers. | Follow-up: fix stale storage Rustdoc sentence for `StoredManagerDailyBriefLaborMinutes::try_new`; update storage README for data-quality hygiene outcome records. |
| API / worker / CLI runtime shell | PASS WITH LOCALIZED DOC FIXES | Runtime/storage lane found API, worker, CLI, and app tool surfaces are understandable as local/test/stubbed or read-only shells, not live side-effect authority. | Worker crate-root wording now uses safe local/background shell language instead of overclaiming durable Postgres backing. Optional public runtime-shell evidence links remain future polish. |
| Rustdoc freshness and evidence links | PASS AFTER CLEAN REBUILD | Rustdoc/evidence lane reproduced a stale target-cache E0460 failure, then `cargo clean && scripts/check_docs.sh` passed; `cargo doc --workspace --no-deps` passed; sampled entity links and all 58 public landing Rustdoc hrefs existed. Gingr/source lane also got Markdown link checks passing after fixing generated-Rustdoc links. | Follow-up: if stale target/cache failures recur in CI or shared worktrees, treat as build-cache/gate hygiene rather than content failure and consider isolated target dirs for docs jobs. |
| Published Rustdoc landing local artifact | PASS | Published-site lane generated an isolated Rustdoc artifact, served representative pages, and found 70/70 local relative landing links OK with representative Rustdoc HTTP pages returning 200. | No local artifact blocker. Browser visual/console QA remains unperformed because Chromium is missing in worker environment. |
| Published-site GitHub-main external links | BLOCKER | Published-site lane found many landing links to entity atlas, glossary, and operator workflow docs return 404 on GitHub `main` because those docs exist locally but are not yet published there. | Launch blocker for public site publication. Resolve by merging/publishing those docs before pointing the landing page to `main`, or link to published/local generated artifacts instead. |
| Non-coder jargon / glossary coverage | BLOCKER FOR PUBLIC OWNER READINESS | Jargon lane found docs are far better than raw engineering docs, but repeated terms `contract`, `semantic`, `projection`, and `promotion/demotion` are under-defined; page openings often lead with crate/module/type vocabulary before operator meaning. | Launch comprehension blocker: add glossary entries and plain-English first-use callouts before declaring the public/operator docs non-coder ready. |
| Glossary pages and translation layer | FOLLOW-UP | Jargon lane found glossary structure useful and translation-layer guidance strong. | Follow-up: retitle public glossary pages to remove reader-facing “draft”; keep design/translation-layer inventory as maintainer-facing. |
| Operator workflow page ordering | POLISH | Workflow lane found comprehension strong; jargon lane found many operator pages put glossary-link lists before the concrete workflow summary. | Polish/P1: move glossary-link lists below a one-sentence job summary on operator pages. |
| Contract crosswalk usability | PASS WITH PATCHED CONSISTENCY NOTE | Crosswalk lane found the six-file crosswalk usable and patched a factual inconsistency about SQL migration presence vs runtime DB wiring. | Follow-up: add a short “How to read these six files” crosswalk README if it becomes a primary non-coder entry point. |

## Contradictions and duplicate findings

### True contradictions

No unresolved true contradictions were found between lanes. Apparent disagreements reconcile as environment/scope differences:

- Rustdoc/evidence lane says docs gates pass after clean rebuild; published-site and crosswalk lanes saw failures during shared-target concurrency or missing generated docs. Reconciliation: content/Rustdoc evidence is sound after clean generation, while the gate is fragile when `target/doc` is missing or another worker runs `cargo clean`.
- Runtime/storage lane flags stale wording in worker/storage Rustdoc while Rustdoc/evidence lane says no stale paths or missing generated pages. Reconciliation: generated pages exist and link targets are fresh; two pieces of generated content still contain misleading prose.
- Outcomes/value lane reports no launch-blocking value-comprehension gap while service-line/runtime lanes list follow-ups. Reconciliation: current docs label unsupported durable/production value claims as gaps; follow-ups improve completeness but are not contradictions.
- Published-site lane finds GitHub 404s while local landing/Rustdoc links pass. Reconciliation: local generated artifact is valid; public GitHub-main publication state is not.

### Duplicate / overlapping findings to merge

- Grooming/training revenue atlas completeness appears in service-line, workflow, and outcomes lanes. Track as one follow-up: add per-entry rows in `docs/design/entity-atlas-revenue-opportunity-entities.md`; do not split into separate implementation tasks.
- Durable outcome caveats for booking, checkout, daily update, grooming/retention, and regional labor exceptions repeat across workflow and outcomes lanes. Track as one evidence/claims-policy item: preserve “planned/future/not yet durable” wording until storage/outcome ownership lands.
- Runtime/storage evidence wording issues are local and can be grouped: worker crate-root overclaim, labor-minutes Rustdoc typo, storage README data-quality outcome discoverability, optional runtime-shell public links.
- Jargon fixes cluster into one public-readiness edit pass: glossary entries for P0 terms, operator translation callouts, public glossary title cleanup, and workflow/spec start-here summaries.
- Published-site link failures are one release/publishing task, not dozens of link bugs: align landing links with what is actually published.

## Launch-blocking vs polish

### Launch-blocking before public/non-coder owner-readiness claim

1. Published-site link state: GitHub-main links from the landing page must not 404 for the entity atlas, glossary, and operator workflow docs.
2. P0 comprehension terms: `contract`, `semantic`, `projection`, and `promotion/demotion` need glossary/first-use translations; owner-facing page openings should not require crate/module vocabulary before the resort job is explained.

### Not launch-blocking, but should be visible in owner review

- Grooming and training pass, but need per-entry revenue-atlas rows for parity with boarding/daycare.
- Several workflows correctly lack dedicated durable outcome storage; do not overclaim measured savings for booking, checkout, daily update, grooming/retention, or regional labor exceptions.
- Worker/storage localized Rustdoc/readme wording fixes are needed, but they do not invalidate the representative evidence chain.
- Published-site browser visual/console QA was not possible in the worker environment; HTTP/static checks substituted.
- Build/doc gates can fail in shared-target concurrent worker conditions; clean/isolated target docs generation passes.

### Polish / quality-of-life

- Direct README back-links to stable entity-index/family-atlas anchors.
- Operator workflow page ordering: job summary before glossary-link list.
- Start-here callouts from long implementation specs to the operator workflow pages.
- Crosswalk README: “How to read these files.”
- Optional runtime-shell evidence section on the public landing page.

## Smallest follow-up card set recommended before owner review

Recommended minimal set: two blockers before public readiness, plus one bundled polish/evidence card that owner review can either accept as non-blocking or schedule after review.

1. `BLOCKER: Fix published-site external docs links`
   - Goal: make `docs/public/index.html` only link to entity atlas, glossary, and operator workflow pages that are actually published, or publish/merge the currently local docs before using GitHub-main URLs.
   - Acceptance: local generated Rustdoc landing still passes; scripted external-link check has no GitHub-main 404s for required entity/glossary/operator docs.

2. `BLOCKER: P0 non-coder glossary and operator-language pass`
   - Goal: add/verify glossary coverage and first-use plain-English translations for `contract`, `semantic`, `projection`, and `promotion/demotion`; add concise operator-translation callouts to the main crate/service-line/source pages where readers currently hit module/type vocabulary first.
   - Acceptance: owner/non-coder can read README/entity-index/operator entrypoints without needing to infer these architecture terms; glossary pages no longer present reader-facing “draft” titles as the primary status.

3. `FOLLOW-UP: Bundle non-blocking evidence parity fixes`
   - Goal: in one docs pass, add grooming/training per-entry revenue-atlas rows, fix worker/storage stale Rustdoc wording, add data-quality hygiene outcome records to storage README, and preserve future/durable-outcome caveats for workflows without storage ownership.
   - Acceptance: no changed implementation required; Markdown/doc checks pass; owner packet can mark these as non-blocking if not completed first.

## Go/no-go recommendation for the next owner-review packet

Go for owner review with caveats, not go for public launch. The final owner-review packet can accurately say the representative docs mostly let a non-coder answer the entity questions and safely distinguish evidence, drafts, review gates, outcomes, and authority. It should also clearly label the two public-readiness blockers above so the owner does not confuse local documentation completeness with published-site readiness.

## Verification basis

This synthesis is based on upstream handoffs and the following audit artifacts already present in the workspace:

- `docs/qa-service-line-entity-comprehension-2026-06-19.md`
- `docs/qa-workflow-contract-comprehension-2026-06-19.md`
- `docs/qa-gingr-source-authority-comprehension-2026-06-19.md`
- `docs/kanban/t_68f74d96-runtime-storage-surface-evidence.md`
- `docs/kanban/t_b5f82fe5-published-site-smoke.md`
- `docs/qa-rustdoc-freshness-evidence-links-2026-06-19.md`
- `docs/qa-jargon-glossary-audit-2026-06-19.md`
- `docs/qa-contract-crosswalk-source-evidence-usability-2026-06-19.md`
- Parent task metadata for the representative matrix, app-packet/review-gate audit, and outcomes/value audit.
