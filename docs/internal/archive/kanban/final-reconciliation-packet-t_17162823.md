# Final reconciliation packet and staged action list

Generated: 2026-06-19 01:28 UTC
Task: `t_17162823` — final reconciliation packet and staged action list
Scope: planning packet only. No board archives, unblocks, moves, dispatches, or task recreations were executed by this card.

Inputs:
- `docs/kanban/old-factory-board-inventory-t_5078771f.md`
- `docs/kanban/old-docs-to-entity-successor-map-t_ae0bcb4b.md`
- `docs/kanban/duplicate-misrouted-implementation-cards-t_2c4c4c2f.md`
- `docs/kanban/liveness-and-no-race-execution-protocol-t_07a0e7c3.md`
- read-only SQLite snapshot at 2026-06-19 01:28 UTC across NVA old and successor boards

## Executive decision

The entity-centered board graph is the canonical execution surface. Preserve completed legacy outputs as evidence/source material, but do not run old workflow/service-line/public-landing/Rustdoc closeout chains as standalone work. Old-board closeout should happen recoverably only after each old output is captured in a successor artifact, successor final QA comment, or this packet.

Current read-only snapshot found many live shared-dir workers on `/home/eran/code/nva` across successor boards. Therefore the first operational rule is: do not unpark old-board cards into the shared directory. Let active successor workers finish, unblock only successor fan-in cards whose parents are done, and archive old duplicates only after adding preservation comments.

## 1. Old outputs to preserve and where to link or port them

| Legacy output/workstream | Preserve as | Link/port destination |
| --- | --- | --- |
| Old factory control board `nva-noncoder-docs-board-factory` create-board summaries | Historical board-construction record only | Keep summarized in `docs/kanban/old-factory-board-inventory-t_5078771f.md`, `docs/kanban/old-docs-to-entity-successor-map-t_ae0bcb4b.md`, and this packet. Do not use as execution surface. |
| Entity atlas board outputs (`docs/design/entity-atlas-inventory.md`, `docs/design/entity-atlas-page-template.md`, family pages, relationship map, non-coder QA/coverage artifacts) | Canonical successor spine | Board `nva-entity-atlas-relationship-map`; final fan-in `t_1dfbfe2b`; downstream final QA board synthesis. |
| Workflow-first operator pages (`docs/workflows/operator/*.md`, `docs/design/operator-workflow-page-inventory.md`, `docs/design/workflow-page-source-rustdoc-map.md`) | Workflow examples and source/Rustdoc evidence, not the navigation spine | `docs/design/workflow-to-entity-navigation-map.md`, `docs/design/entity-driven-workflow-page-template.md`, entity-driven workflow board final review `t_f4e4b7b9`, and final QA coverage synthesis `t_627e115e`. |
| Service-line Rustdoc summaries for boarding/daycare/care/reservation/grooming/training/retail/rebooking | Source/Rustdoc evidence and operator-language snippets | Core service entities in `docs/design/entity-atlas-petsuites-core-entities.md`; revenue/service-opportunity entities in `docs/design/entity-atlas-revenue-opportunity-entities.md`; successor final QA Rustdoc/evidence checks. |
| Safety and human-review docs (`docs/safety/*.md`) | Safety policy/evidence corpus | `docs/design/entity-atlas-review-safety-boundaries.md`; board `nva-entity-action-safety-overlays`; final safety QA `t_a8d2413a`; relationship/contract crosswalk where safety edges are modeled. |
| Glossary/translation artifacts (`docs/glossary-translation-layer-inventory.md`, `docs/design/glossary-translation-layer.md`, `docs/glossary-architecture-terms.md`, `docs/glossary-workflow-state-terms.md`, `docs/glossary-source-data-terms.md`) | Entity-linked decoder material | Board `nva-entity-atlas-glossary`; core glossary draft `t_7f80c501`; examples/cross-links/review/closeout cards; entity pages should link glossary entries only after entity meaning is established. |
| Labor-cost doc-contracts board outputs | Source/Rustdoc contract hardening evidence | Board `nva-relationship-contract-crosswalk`; entity family evidence sections; `docs/design/entity-atlas-relationships.md`; do not resurrect the old board. |
| Public landing/navigation outputs (`docs/design/public-docs-landing-content-map.md`, `docs/design/public-docs-landing-technical-inventory.md`, `docs/public/index.html`, build/check scripts) | Technical implementation and audience-path evidence | Board `nva-entity-atlas-readme-landing`; public landing card `t_d7b18f7d`; README fan-in `t_e72c5265`; publish verification `t_2e4a1d0b`. Landing should lead through entity index/audience paths before crates/workflows. |
| Documentation style guide (`docs/quality/nva-documentation-style-guide.md`) | Cross-cutting acceptance/style guide | Keep as-is and cite from future entity docs card bodies. No successor content board needed. |
| Rustdoc filler cleanup outputs (`docs/rustdoc-operational-language-guide.md`, smell inventories, rewrite batches) | Language guide and evidence for weak source docs | Board `nva-rustdoc-filler-entity-cleanup`; verification `t_dbf6b850`; final QA `t_9d9555b9`; family pages decide acceptance by entity contract fidelity, not generic operational English. |
| Published docs QA reports (`docs/qa-published-jargon-link-audit-2026-06-19.md`, smoke/freshness/non-coder review handoffs) | Regression QA corpus | Board `nva-entity-docs-final-qa`; published smoke `t_b5f82fe5`; Rustdoc freshness/evidence `t_e1f45670`; coverage synthesis `t_627e115e`; final owner packet `t_e52ae96a`. |

Preservation rule: a legacy output is successor-ready only if it answers an entity-template question for a named entity/relationship: meaning, source authority, allowed automation, blocked actions, outcome proof, or evidence path. Otherwise port it as source material and rewrite with entity questions.

## 2. Old todo/running cards to leave alone until completion

Leave these untouched until they complete or a separate human-approved stale-worker/reclaim card runs the no-race protocol:

| Board | Card(s) | Why leave alone now | Successor handling after completion |
| --- | --- | --- | --- |
| `nva-rustdoc-filler-cleanup-operational-english` | running `t_740e8d37` final smell search and repo verification | It is already running; interrupting risks losing useful final smell evidence. Snapshot now shows the earlier reviewer pass completed and only final verification is running. | Harvest final smell findings into `nva-rustdoc-filler-entity-cleanup` verification/final review (`t_dbf6b850`, `t_9d9555b9`) if not already covered, then archive/close the old board recoverably. |
| Any old board card whose PID is alive at action time | Determined by rerunning `docs/kanban/liveness-and-no-race-execution-protocol-t_07a0e7c3.md` | Current snapshot shows old workflow verification no longer open, but future state must be verified before action. | Wait for done handoff, preserve outputs, then decide archive/recreate. |

Do not unblock these old chains:
- `nva-service-line-rustdoc-summaries`: blocked `t_f20f1528`, todo `t_e083fce5`, `t_4b6bc5e3`, `t_05ca48da`.
- `nva-public-noncoder-docs-landing-navigation`: blocked `t_ce0dfe0b`, todo `t_045b6715`.

Those are not live and are duplicate old acceptance gates; they should remain parked until successor coverage is proven and archive comments are ready.

## 3. Cards to archive recoverably after outputs are captured

Archive only after adding an archive-preflight comment to each old card naming the successor owner/artifact and confirming current liveness/dirty-file state. Do not delete repo files.

| Archive stage | Candidate cards/boards | Required capture before archive | Successor owner |
| --- | --- | --- | --- |
| Stage A — metaboard safety duplicates | `nva-entity-centered-docs-metaboard` blocked cards `t_67801671`, `t_8445ee01`, `t_64bacab7`, `t_6ad64a49`, `t_02b5ce7d`, `t_2fcc9d53`, `t_e43f0045`, `t_2c9ea1e8`, `t_e0fa3981`, `t_05c5271c` | Comment that the dedicated safety board exists and currently owns the exact overlay lanes; no output exists on these blocked metaboard clones beyond their bodies. | `nva-entity-action-safety-overlays`, especially final QA `t_a8d2413a`. |
| Stage B — old service-line closeout chain | `nva-service-line-rustdoc-summaries`: `t_f20f1528`, `t_e083fce5`, `t_4b6bc5e3`, `t_05ca48da` | Ensure service-line Rustdoc summary handoffs are referenced in entity family pages or final QA comments; capture any unique cargo/doc commands from card bodies if successor QA lacks them. | `nva-entity-docs-final-qa` (`t_e1f45670`, `t_627e115e`, `t_e52ae96a`) and service-line family pages. |
| Stage C — old public landing verification chain | `nva-public-noncoder-docs-landing-navigation`: `t_ce0dfe0b`, `t_045b6715` | Copy unique build/render/public caveats from old bodies to successor landing publish verification if missing. Confirm successor landing has entity-front-door acceptance. | `nva-entity-atlas-readme-landing` (`t_d7b18f7d`, `t_e72c5265`, `t_82b8168d`, `t_2e4a1d0b`). |
| Stage D — old Rustdoc filler board | `nva-rustdoc-filler-cleanup-operational-english` after running `t_740e8d37` completes | Preserve final smell search findings and any residual phrases as evidence for entity-cleanup verification. | `nva-rustdoc-filler-entity-cleanup` (`t_dbf6b850`, `t_9d9555b9`). |
| Stage E — closed legacy boards | `nva-noncoder-docs-board-factory`, `nva-safety-human-review-docs`, `nva-glossary-translation-layer`, `nva-labor-cost-doc-contracts`, `nva-published-docs-qa-noncoder-usability`, `nva-documentation-style-guide-examples` | This packet and previous inventory/mapping artifacts already capture board-level output. Add board-level closeout comments only if a later operator wants explicit archive audit trail. | No new execution surface; successor boards consume evidence as needed. |

## 4. Successor cards that should be unblocked first

There are no blocked cards on `nva-entity-centered-docs-metaboard` that should be unblocked first. Its remaining blocked cards are misrouted safety clones and should be archived after preservation comments, not resumed.

First unblocks/promotions should happen on focused successor boards, in dependency order, and only after rerunning the no-race preflight:

1. `nva-relationship-contract-crosswalk`: unblock `t_4dc31fc7` after running `t_ee3d3e4f` and `t_70294150` complete. This finalizes relationship/contract evidence that other QA packets can cite.
2. `nva-entity-action-safety-overlays`: unblock `t_a8d2413a` after all five running overlay cards complete. This produces the safety overlay QA that replaces the blocked metaboard safety clones.
3. `nva-rustdoc-filler-entity-cleanup`: continue its serialized chain after current `t_f09070fd`: `t_fbcfd3c2` -> `t_dbf6b850` -> `t_9d9555b9`. Do not run old Rustdoc final gates in parallel except to harvest already-completed `t_740e8d37` output.
4. `nva-entity-atlas-glossary`: after running `t_7f80c501`, proceed `t_b9929fcc` -> `t_8c7f913a` -> `t_85440163` -> `t_82196ffe`.
5. `nva-entity-atlas-readme-landing`: after running `t_6f6916e5`, proceed `t_d7b18f7d` and `t_e72c5265`, then `t_82b8168d`, then `t_2e4a1d0b`.
6. `nva-entity-docs-final-qa`: after running `t_68f74d96`, `t_b5f82fe5`, and `t_e1f45670` complete, unblock `t_627e115e`, then `t_e52ae96a`.
7. `nva-entity-atlas-relationship-map`: let current `t_1dfbfe2b` and `t_b5aefa7b` finish; if final fan-in blocks on missing evidence from successor focused boards, comment that dependency rather than starting old boards.

Because all listed successor boards share `/home/eran/code/nva`, use the liveness/no-race protocol before each unblock. Prefer waiting over parallel broad docs/Rustdoc gates.

## 5. Child boards/cards that should be recreated with entity-centered bodies

Do not recreate immediately. Recreate only if, after successor final QA reads the old outputs, a specific useful scope remains without an entity-centered owner.

Potential recreate candidates:

| Legacy source | Recreate only if this gap remains | New location/body requirements |
| --- | --- | --- |
| Old service-line closeout chain | Successor QA lacks explicit evidence that boarding/daycare/care/reservation/grooming/training/retail Rustdoc summaries are linked from entity family pages. | Create on `nva-entity-docs-final-qa` or the relevant entity-family board, not the old service-line board. Body must name exact entity family pages, old source card IDs, required Rustdoc/source links, and entity-template acceptance. |
| Old public landing verification `t_ce0dfe0b`/`t_045b6715` | Successor landing board lacks a concrete build/render/published-site verification card or unique script caveats from old bodies. | Create/augment on `nva-entity-atlas-readme-landing`, preferably as a comment/body amendment to `t_2e4a1d0b`. Acceptance: entity index/audience paths are public front door; links render; no crate-first regression. |
| Old Rustdoc filler final search | Successor entity-cleanup verification lacks the exact residual smell phrases or old retail/generic operational-English findings. | Create on `nva-rustdoc-filler-entity-cleanup` as a narrow evidence-harvest card. Acceptance: phrases are judged against entity purpose/authority/workflow/safety/labor-value contract, not just removed globally. |
| Old workflow verification/publication evidence | Published root or generated docs remain stale after successor landing/workflow pages finish. | Create on `nva-entity-atlas-readme-landing` or `nva-entity-docs-final-qa` as a publication-pipeline diagnostic. Body must prohibit content rewrites outside entity-front-door acceptance unless explicitly approved. |
| Blocked safety metaboard clones | Dedicated safety overlay final QA finds an uncovered entity/action family. | Do not unblock clones. Create one new focused card on `nva-entity-action-safety-overlays` naming the missing entity/action family, source evidence, blocked actions, and non-coder review expectations. |

Recreation template for any new card:
- Title starts with the entity/action or artifact, not the old board name.
- Body names the successor board and exact artifact(s) to edit.
- Body cites old card IDs only as evidence sources.
- Acceptance requires entity-template fields: entity meaning, source authority, allowed automation, blocked actions, outcome proof, source/Rustdoc/test evidence, non-coder route.
- Body includes stop condition: if a live shared-dir writer overlaps target paths, block/wait rather than editing.

## 6. Dependencies between old-board closeout and new entity-centered execution

| Dependency | Direction | Practical consequence |
| --- | --- | --- |
| Old done outputs -> successor final QA | Old output preservation must happen before old archive, but it can be via comments/artifacts rather than re-running cards. | Before archiving old closeout cards, add comments pointing to entity family pages, final QA cards, or this packet. |
| Successor safety overlays -> archive metaboard safety clones | Dedicated safety overlay board must own/finish the entity/action acceptance before clone archive is fully safe. | Leave metaboard safety clones blocked until `t_a8d2413a` is ready/done, unless a human explicitly wants earlier archive with current board reference. |
| Successor landing/README -> archive old public landing verification | Entity-front-door landing must exist or be in active successor verification before old public closeout is archived. | Do not archive `t_ce0dfe0b`/`t_045b6715` until successor landing bodies/comments capture old build/render caveats. |
| Successor final QA -> archive service-line closeout chain | Entity docs final QA must consume service-line Rustdoc evidence before old service-line final gates are removed. | Park old service-line gates; later archive after `t_e1f45670`/`t_627e115e` capture evidence. |
| Old Rustdoc final smell output -> successor entity cleanup | If `t_740e8d37` produces unique findings, successor entity-cleanup verification should absorb them before old board archive. | Do not start a second whole-repo smell gate from old board. Harvest once done. |
| Relationship crosswalk -> entity atlas/final QA | Final relationship/crosswalk package should feed entity atlas fan-in and final QA. | Prioritize `t_4dc31fc7` before final owner packet if relationship evidence is incomplete. |
| Shared-dir liveness -> every unblock/archive/reclaim | The repo has many active shared-dir workers. | Run the liveness/no-race protocol before every mutation; comment before archive/reclaim/unblock; do not batch-unblock. |

## Staged action list for a later operator card

This list is intentionally actionable but not executed here.

### Stage 0 — preflight every time

1. Rerun read-only board snapshot across old and successor boards.
2. Verify every running PID with `ps`.
3. Run `git status --short`, `git diff --name-only`, and `git ls-files --others --exclude-standard` in `/home/eran/code/nva`.
4. Map dirty files to running/done handoffs.
5. If any candidate action overlaps a live writer or broad docs/Rustdoc gate, wait or create an explicit worktree plan.

### Stage 1 — preserve evidence into successor comments

1. Add a comment to `nva-entity-docs-final-qa` synthesis `t_627e115e` pointing to:
   - `docs/kanban/old-factory-board-inventory-t_5078771f.md`
   - `docs/kanban/old-docs-to-entity-successor-map-t_ae0bcb4b.md`
   - `docs/kanban/duplicate-misrouted-implementation-cards-t_2c4c4c2f.md`
   - this packet
2. Add a comment to `nva-entity-atlas-readme-landing` publish verification `t_2e4a1d0b` with old public landing build/render caveats if not already present.
3. Add a comment to `nva-rustdoc-filler-entity-cleanup` verification `t_dbf6b850` with old final smell search output after `t_740e8d37` completes.

### Stage 2 — unblock focused successor fan-ins only

Unblock/promote at most one card at a time using the order in section 4. Never unblock the blocked metaboard safety clones.

### Stage 3 — archive recoverably

After successor comments/fan-ins exist, archive in this order:
1. Metaboard safety clones.
2. Old public landing verification/closeout cards.
3. Old service-line verification/closeout chain.
4. Old Rustdoc filler cleanup board after `t_740e8d37` output is harvested.
5. Any remaining closed legacy boards only if a human wants visual board cleanup; otherwise leaving done boards alone is safe.

Each archive comment must name:
- old output preserved in artifact/comment;
- successor board/card/artifact owner;
- current liveness check result;
- confirmation no dirty files were discarded.

### Stage 4 — recreate only gap cards

After successor final QA identifies concrete gaps, create new entity-centered cards using the recreation template above. Do not recreate by copying old board bodies verbatim.

## Human decision status

No human decision is required to accept this packet. A later mutation card should still follow the no-race protocol and may block if live worker state, dirty-file ownership, or successor coverage is ambiguous at action time.
