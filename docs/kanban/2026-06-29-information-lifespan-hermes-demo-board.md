# Information Lifespan + Hermes Processor Demo Board

Date: 2026-06-29
Board slug: `nva-information-lifespan-hermes-demo`
Repo: `/home/eran/code/nva`

## Purpose

Build a visual, running demo that shows the lifespan of pet-resort information as it moves through the NVA-owned infrastructure:

```text
mocked Gingr/source payload
  -> provider/source model
  -> source snapshot + provenance/import run
  -> NVA-owned normalized/domain/workflow models
  -> database rows/projections
  -> Hermes-in-Docker processing step
  -> calculations/ranking/review gates
  -> API/network response + structured logs
  -> final Manager Daily Report artifact
```

The demo should feel like a visible Rube Goldberg machine: a user clicks **Run information lifespan**, the system processes deterministic mock data, stages light up, real/request-like artifacts appear, DB/log/network panels update, and the final manager report is produced.

## Non-negotiable framing

- Gingr/provider access is mocked and clearly labeled as synthetic read-only source evidence.
- Everything after the mocked source should be as real as practical: Rust models, local DB/migrations, API calls, logs, Docker Compose services, and a Hermes container that processes the mock data into the final report.
- Do not claim production NVA/Gingr access, live customer sends, live PMS writes, payments/refunds/discounts, schedule changes, or medical/safety decisions.
- The demo should show infrastructure through product use, not by leading with an architecture diagram.
- The final artifact is a Manager Daily Report with visible lineage, calculations, review locks, and labor-minute/value proof.

## Target demo experience

1. User opens staff-web demo route.
2. User clicks **Run information lifespan**.
3. Stage cards animate or progress in order:
   - Mock Gingr event received.
   - Provider DTO/source model shown.
   - Source snapshot/provenance stored.
   - NVA models normalized.
   - Database rows/projections written/read.
   - Hermes processor container runs the report-generation step.
   - Calculations/ranking/review gates applied.
   - API/network response returned.
   - Manager Daily Report appears.
4. At each stage, a proof drawer can show one or more of:
   - Rust model/type path.
   - JSON payload.
   - SQL/table row/projection.
   - Structured log line.
   - HTTP request/response.
   - Hermes container invocation/output.
   - Calculation summary.
5. The final report includes counts like `source snapshots`, `normalized facts`, `workflow packets`, `review gates`, `audit events`, and `estimated labor minutes saved`.

## Board decomposition: 3 cards per piece

Each piece has exactly three kinds of card:

- **Assembly card**: put the functional/backend/frontend pieces together.
- **UX/visual card**: make that piece visible, organized, animated, and understandable.
- **Verification card**: prove the piece works and is honest/safe.

Because the repo shares one checkout, code-mutating cards should run serialized unless workers explicitly create isolated worktrees. This board is staged behind a blocked start gate until execution is explicitly started.

## Piece 1 — Trace contract + mocked source evidence

Goal: define the canonical trace envelope and deterministic mocked Gingr/source inputs that drive the entire demo.

Cards:

1. Assembly: trace contract and source mock model scaffold.
2. UX/visual: source-event cards and model/source preview design.
3. Verification: trace-contract and source-boundary tests.

## Piece 2 — Local DB/projection lifecycle

Goal: show actual database-oriented lifecycle proof from source snapshot/import run through workflow/audit/report projections.

Cards:

4. Assembly: migration/seed/query hooks for information-lifespan rows.
5. UX/visual: DB/projection proof drawer and table-row timeline.
6. Verification: DB migration/seed/query smoke and schema truthfulness review.

## Piece 3 — Hermes-in-Docker processor

Goal: build a Dockerized Hermes processor service that consumes the deterministic trace/input and generates or enriches the final manager report artifact.

Cards:

7. Assembly: Hermes processor container, compose wiring, script/API bridge.
8. UX/visual: Hermes-processing stage, live log/output panel, agent boundary labels.
9. Verification: container build/run smoke, redaction, deterministic output tests.

## Piece 4 — Report-generation logic + API/network proof

Goal: expose the running flow through API endpoints/network requests and produce the final Manager Daily Report artifact from the processed trace.

Cards:

10. Assembly: report-generation endpoint and trace replay/run API.
11. UX/visual: network request/response panel and final artifact layout.
12. Verification: API contract, report artifact, and side-effect-lock tests.

## Piece 5 — Rube Goldberg frontend experience

Goal: turn the trace into an animated, understandable product demo rather than a wall of JSON.

Cards:

13. Assembly: interactive stage machine and replay state wiring.
14. UX/visual: animation, layout, responsive polish, visual artifacts, information hierarchy.
15. Verification: browser QA, accessibility, screenshot/readability checks.

## Piece 6 — End-to-end Docker Compose/demo package

Goal: make the whole stack runnable/reviewable as a single local demo package and optionally deployment-ready later.

Cards:

16. Assembly: compose/run scripts and local-demo operator command path.
17. UX/visual: presenter walkthrough, labels, screenshots, fallback assets.
18. Verification: full-stack smoke, final independent review, readiness verdict.

## Final definition of done

The board is complete when a fresh checkout can run one documented command or short sequence that starts the demo stack, clicks or calls the information-lifespan run, and proves:

- mocked Gingr/source evidence is generated;
- source/provenance and NVA-owned models are visible;
- database lifecycle artifacts are written/read or truthfully simulated where the current schema cannot persist them yet;
- Hermes-in-Docker processes the mock data into the report path;
- logs and network requests are visible;
- review/side-effect locks are explicit;
- the Manager Daily Report artifact is generated;
- tests/smokes verify the trace contract, processor determinism, UI rendering, no-live-access claims, and final demo run.

## Suggested verification commands for final board closeout

```sh
git status --short --branch
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
./scripts/check_docs.sh
python scripts/check_markdown_links.py --repo-root .
./scripts/demo_information_lifespan.sh
# plus app-specific npm/build/browser checks once the route exists
```

## Safety constraints to repeat in every implementation card

- Use synthetic/mock Gingr/source inputs only.
- Do not require or print real credentials.
- Do not send live customer communications.
- Do not write to live PMS/provider systems.
- Do not create payments/refunds/discounts/schedule changes.
- Do not make medical/safety decisions; show review gates instead.
- Keep logs and UI free of secrets or real customer data.
