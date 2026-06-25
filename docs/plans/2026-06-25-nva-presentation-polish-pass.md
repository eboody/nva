# NVA Presentation Polish Pass Implementation Plan

> **For Hermes:** Use Kanban execution with serialized shared-checkout work. Do not broaden into new product features unless a card explicitly finds a presentation blocker that cannot be fixed with docs/demo/visual polish.

**Goal:** Turn the current NVA owned-operations API demo into a crisp, credible job-conversation package that is understandable, rehearsable, visually memorable, and honest about the no-live-access boundary.

**Architecture:** This is a presentation-readiness layer on top of the existing local proof. The source of truth remains the owned operations API/read-model/demo code already committed; this pass sharpens the path a job contact sees: executive story, exact demo, skeptical review, visual artifact, and final checklist. Code changes are allowed only when demo rehearsal finds a small reliability or output-clarity issue.

**Tech Stack:** Rust workspace, Markdown docs, local shell smoke scripts, checked OpenAPI artifact, optional standalone HTML/SVG diagram.

---

## Global constraints and safety boundaries

- Repo: `/home/eran/code/nva`, branch `main`.
- Current proof commit: `48746e72944b5296544e674e31631e5664616691`.
- This pass should improve presentation clarity, not invent live access.
- No live NVA, Gingr, provider/PMS, customer, payment, schedule, or medical/safety actions.
- No real customer/provider data.
- No production deployment claims.
- Preserve the thesis: **do not clone Gingr; build the owned operations API NVA needs, with Gingr as a legacy/source adapter during migration.**
- Every card must explicitly distinguish:
  - runs now locally;
  - locally smoke-tested;
  - scaffolded/planned;
  - requires live/read-only access later.

---

## Pass 1: Executive story pass

### Objective

Make the first 60-120 seconds compelling to a job contact who may not read code. The story should prove product judgment, technical ability, safety discipline, and the ability to build useful things despite missing access.

### Target artifacts

- `README.md`
- `docs/presentation/job-presentation-walkthrough.md`
- `docs/presentation/owned-operations-api-replacement-talk-track.md`
- Optional concise handout: `docs/presentation/nva-demo-executive-brief.md`

### Required narrative spine

1. **Situation:** NVA/Pet Resorts operations likely depend on Gingr and downstream BI extraction because the source system is not shaped around NVA's desired operations.
2. **Constraint:** no live NVA/Gingr access, no production data, no permissions.
3. **Choice:** do not fake production or mirror Gingr; build a safe local owned API proof.
4. **Proof:** local API, checked OpenAPI, read-model/storage migration, audit/logging/metrics posture, data-quality hygiene workflow, smoke scripts.
5. **Business value:** reduce manual BI scraping, expose review-gated workflows, capture labor-outcome proof, create a migration path away from legacy vendor authority.
6. **Next access ask:** read-only docs/export/sample data would validate source mappings and turn local proof into an integration pilot.

### Concrete tasks

#### Task 1.1: Rewrite the README top path

**Files:**
- Modify: `README.md`

**Steps:**
1. Read the first 120 lines of `README.md`.
2. Add or tighten a top-level "Presentation path" / "Why this exists" section.
3. Keep the phrase "no live access" explicit but positive.
4. Link to the talk track, OpenAPI artifact, and demo commands.
5. Avoid defensive repository-size/LOC arguments.

**Acceptance:** A reader can understand the project in under one minute without knowing the prior Kanban history.

#### Task 1.2: Create a one-page executive brief

**Files:**
- Create: `docs/presentation/nva-demo-executive-brief.md`

**Sections:**
- One-sentence pitch.
- What I built without access.
- Why Gingr is source evidence, not product authority.
- What runs locally.
- What the demo proves.
- What real access unlocks next.
- Suggested 30-second and 2-minute verbal pitch.

**Acceptance:** The brief can be pasted into an email or used as speaking notes.

#### Task 1.3: Tighten the talk track

**Files:**
- Modify: `docs/presentation/owned-operations-api-replacement-talk-track.md`
- Modify if needed: `docs/presentation/job-presentation-walkthrough.md`

**Steps:**
1. Remove redundant jargon.
2. Add a plain-English opening.
3. Add likely objections and short answers:
   - "Is this real without data?"
   - "Why not just use Gingr?"
   - "What would you need next?"
   - "Is this safe?"
4. Ensure no claim says production-ready.

**Acceptance:** The talk track feels like a conversation, not a dense architecture memo.

### Verification

- `./scripts/check_docs.sh`
- `python scripts/check_markdown_links.py --repo-root .`
- Read the final brief aloud for flow and overclaim risk.

---

## Pass 2: Demo rehearsal pass

### Objective

Make the live demo reliable, short, and easy to narrate. The user should be able to run exactly the commands in the docs and explain expected output.

### Target artifacts

- `docs/presentation/job-presentation-walkthrough.md`
- `docs/presentation/owned-operations-api-replacement-talk-track.md`
- Optional script: `scripts/demo_owned_operations_api.sh`
- Existing smoke scripts:
  - `scripts/smoke_data_quality_hygiene_local_loop.sh`
  - `scripts/smoke_data_quality_hygiene_disabled_worker_outbox.sh`

### Demo shape

The demo should have three lanes:

1. **Contract lane:** show checked OpenAPI / public DTO boundary.
2. **Workflow lane:** run the safe data-quality hygiene smoke loop.
3. **Operations lane:** show disabled worker/outbox and metrics/readiness posture.

### Concrete tasks

#### Task 2.1: Rehearse documented commands exactly

**Files:**
- Modify docs only unless a command is broken.

**Steps:**
1. Extract every shell command currently recommended for the presentation path.
2. Run each from a fresh shell in `/home/eran/code/nva`.
3. Record expected short output snippets.
4. Remove or caveat commands that are too slow/noisy for a live conversation.

**Acceptance:** Every documented command either passes or is clearly labeled optional/slow.

#### Task 2.2: Add a single demo wrapper if useful

**Files:**
- Optional create: `scripts/demo_owned_operations_api.sh`
- Modify docs to reference it.

**Script requirements:**
- `set -euo pipefail`.
- Prints short headers.
- Runs only safe local commands.
- Does not need credentials.
- Does not perform live writes/sends/provider actions.
- Keeps output readable.

**Acceptance:** A presenter can run one command for the core demo if desired.

#### Task 2.3: Add expected-output anchors

**Files:**
- Modify: `docs/presentation/job-presentation-walkthrough.md`
- Modify: `docs/presentation/owned-operations-api-replacement-talk-track.md`

**Steps:**
1. For each demo step, include "what to say" and "what output proves".
2. Include fallback if a full workspace gate is too slow.
3. Keep smoke script output snippets current.

**Acceptance:** The doc prevents fumbling.

### Verification

- `./scripts/demo_owned_operations_api.sh` if created.
- Existing smoke scripts.
- `./scripts/check_docs.sh`
- Markdown link check.

---

## Pass 3: Skeptical hiring-manager / external reviewer pass

### Objective

Read the repo as an outsider looking for vapor, overclaiming, hidden fragility, or confusing presentation flow. Patch small clarity issues immediately; create follow-up notes only for substantial future work.

### Target artifacts

- `docs/presentation/nva-demo-executive-brief.md`
- `docs/presentation/owned-operations-api-replacement-talk-track.md`
- `docs/presentation/job-presentation-walkthrough.md`
- Optional audit artifact: `docs/presentation/nva-demo-skeptical-review.md`

### Skeptical review rubric

Score each item as Green / Yellow / Red:

1. **Reality:** Does the package prove runnable local behavior, or just describe it?
2. **Honesty:** Are no-live-access and non-production caveats visible?
3. **Business value:** Is labor-cost / BI / workflow value clear without reading Rust?
4. **Safety:** Are live/customer/provider side effects blocked and understandable?
5. **Demo usability:** Can someone run or watch the demo without confusion?
6. **Technical credibility:** Are tests/OpenAPI/storage/read-models concrete enough?
7. **Next-step clarity:** Is the ask for read-only access/sample data specific?

### Concrete tasks

#### Task 3.1: Produce skeptical review artifact

**Files:**
- Create: `docs/presentation/nva-demo-skeptical-review.md`

**Sections:**
- Verdict.
- Green/yellow/red table.
- Top 5 possible objections and answers.
- Overclaim scan findings.
- Small fixes applied.
- Deferred improvements.

**Acceptance:** The review can be shown to the user as a readiness check.

#### Task 3.2: Patch clarity/overclaim issues

**Files:**
- Modify the docs implicated by the review.

**Rules:**
- Patch wording, navigation, missing caveats, and confusing ordering directly.
- Do not add major new product scope.
- If a real technical blocker is found, stop and create a correction card rather than hiding it.

**Acceptance:** No Red items remain for presentation; Yellow items have explicit caveats or next-step asks.

### Verification

- `./scripts/check_docs.sh`
- Markdown link check.
- Optional independent review handoff from `pet-resort-reviewer`.

---

## Pass 4: Visual artifact pass

### Objective

Create one memorable diagram/one-pager that explains the strategic shift: **Gingr-centered extraction today → NVA-owned operations API and BI/read-model layer tomorrow**.

### Target artifacts

Preferred durable artifacts:

- `docs/presentation/assets/owned-operations-api-replacement.html`
- `docs/presentation/owned-operations-api-visual-guide.md`

Optional if the repo convention prefers simple Markdown:

- Mermaid diagram embedded in `owned-operations-api-visual-guide.md`

### Diagram content

The visual should show:

#### Today / current pain

- Gingr / provider PMS as operational source.
- BI extracts into separate DB/reporting workaround.
- Operators/staff lack product-owned workflow gates.
- Metrics/labor outcomes are downstream/inferred.

#### Proposed owned layer

- Source adapter / provenance boundary.
- NVA-owned operations API.
- Review-gated workflow packets.
- Audit/logging/metrics/events.
- BI/read-model projections.
- Safe local demo slice: Data-Quality Hygiene.
- Future source validation with read-only access.

#### Safety labels

- No live customer sends.
- No provider/PMS writes.
- No payment/schedule/medical decisions.
- Local proof until real access validates mappings.

### Visual style

Use a single-file dark themed HTML/SVG diagram consistent with the `architecture-diagram` skill:

- External/source boxes in slate.
- Backend/API boxes in emerald.
- Database/read-model boxes in violet.
- Safety/review gates in rose.
- Metrics/event stream in orange/amber.
- Clear arrows showing source evidence → owned API → workflow/read-model outcomes.

### Concrete tasks

#### Task 4.1: Draft visual guide

**Files:**
- Create: `docs/presentation/owned-operations-api-visual-guide.md`

**Sections:**
- What the diagram shows.
- How to narrate it in 60 seconds.
- Legend.
- Link to HTML diagram.
- Caveats and next access ask.

#### Task 4.2: Create standalone HTML/SVG diagram

**Files:**
- Create: `docs/presentation/assets/owned-operations-api-replacement.html`

**Requirements:**
- Single self-contained HTML file.
- Inline CSS/SVG.
- No JavaScript.
- Clear title and subtitle.
- Summary cards under the diagram.
- Legend outside boundary boxes.

**Acceptance:** Opening the file in a browser should communicate the thesis without reading code.

#### Task 4.3: Link visual from presentation path

**Files:**
- Modify: `README.md`
- Modify: `docs/presentation/job-presentation-walkthrough.md`
- Modify: `docs/presentation/nva-demo-executive-brief.md`

**Acceptance:** A job contact can find the visual from the main presentation path.

### Verification

- Markdown link check.
- Inspect HTML for broken internal references and obvious rendering mistakes.
- Optional browser screenshot if tooling is available; otherwise static HTML sanity check.

---

## Pass 5: Final presentation checklist and closeout

### Objective

Leave the repo in a clean, pushed, presentation-ready state with a short checklist the user can follow before the job conversation.

### Target artifacts

- `docs/presentation/nva-presentation-checklist.md`
- Commit pushed to `origin/main`
- Board archived when complete

### Checklist contents

- 30-second pitch.
- 2-minute pitch.
- Exact demo command(s).
- What to open first.
- What not to claim.
- What real access to ask for.
- If-demo-fails fallback path.
- Final evidence: commit, gates, smoke commands.

### Final verification commands

Run at closeout:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
./scripts/check_docs.sh
python scripts/check_markdown_links.py --repo-root .
./scripts/smoke_data_quality_hygiene_local_loop.sh
./scripts/smoke_data_quality_hygiene_disabled_worker_outbox.sh
# If created:
./scripts/demo_owned_operations_api.sh
```

Then:

```bash
git status --short
git add <intentional files>
git commit -m "Polish NVA presentation demo package"
git push origin main
```

### Acceptance

- Board all done.
- Repo clean and pushed.
- User can present with one clear story, one exact demo path, one visual, and one honest next-access ask.
