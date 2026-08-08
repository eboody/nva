# Intelligible Manager Brief Demo Kanban Board

Status: staged Kanban board for turning `docs/plans/2026-06-28-intelligible-manager-brief-demo-plan.md` into a job-contact-ready demo. The board is intentionally parked behind a start gate because all implementation cards share `/home/eran/code/nva` and should be serialized unless explicit isolated worktrees are created.

Board slug:

```text
nva-intelligible-manager-brief-demo
```

Created from commit context:

```text
95f58c3 docs: plan intelligible manager brief demo
```

## Board policy

- Primary audience: a job contact who needs to understand the workflow in 3–5 minutes.
- Presentation frame: access-constrained, safety-first, synthetic/local product proof.
- Product story: messy pet-resort morning signals → tracked source facts/caveats → review-gated Manager Daily Brief → outcome/labor proof.
- Keep architecture, owned-backend, Gingr migration, DTO/read-model proof secondary until the viewer understands the manager workflow.
- No claims of live NVA/Gingr access.
- No live customer sends, PMS/provider writes, payment/refund actions, schedule/capacity mutation, or medical/safety decisions.
- Safe next ask: read-only sample exports, field dictionaries, and BI query inventory for one workflow.

## Serialization

The active gateway may claim ready work automatically, so the board is parked behind:

```text
t_010bee65 — START GATE: begin intelligible manager brief demo implementation
```

Expected parked state:

```text
blocked=1
ready=0
running=0
todo=11
```

When the user explicitly says to begin, complete the start gate, then dispatch exactly one board-scoped worker at a time unless isolated worktrees are created.

## Task graph

```text
t_010bee65 START GATE
  ↓
t_2ad2daa7 docs: acceptance contract for intelligible manager brief demo
  ↓
t_df224168 ui: rename demo steps into human workflow beats
  ↓
t_be284994 ui: show the messy morning before the manager brief
  ↓
t_3e82ad4b ui: mark manager brief actions as ready or blocked
  ↓
t_69913635 ui: make the no-access safety boundary explicit
  ↓
t_ef0e12b3 ui: add proof drawer behind the visual workflow
  ↓
t_584e6bd7 ui: add safe real-access next ask section
  ↓
t_e2f18143 docs: add 3-5 minute manager brief demo script
  ↓
t_3007beee test: guard product-first intelligibility in staff-web smoke tests
  ↓
t_7f656a02 qa: full local verification and fresh screenshot for presentation
  ↓
t_12a243fd review: final board closeout and presentation readiness note
```

## Cards

### `t_010bee65` — START GATE: begin intelligible manager brief demo implementation

Assignee: `pet-resort-reviewer`

Purpose: keep the board parked until explicit user instruction to begin. This avoids accidental shared-checkout mutation from the running gateway.

### `t_2ad2daa7` — docs: acceptance contract for intelligible manager brief demo

Assignee: `pet-resort-docs`

Creates `docs/presentation/intelligible-manager-brief-demo-acceptance.md` with above-the-fold, interaction, safety, proof, and failure-condition requirements. Verifies markdown links.

### `t_df224168` — ui: rename demo steps into human workflow beats

Assignee: `pet-resort-code`

Changes the four steps to `messy morning`, `facts tracked`, `manager brief`, and `review recorded`; adds active-step explanatory copy; updates smoke assertions.

### `t_be284994` — ui: show the messy morning before the manager brief

Assignee: `pet-resort-code`

Adds the “Before the brief” / morning-chaos strip with lobby rush, arrivals, rabies proof, coverage, and quiet-room pain signals; styles it; updates tests and build.

### `t_3e82ad4b` — ui: mark manager brief actions as ready or blocked

Assignee: `pet-resort-code`

Adds status badges so at least one action is `review ready` and at least one is `blocked`; verifies readability and smoke tests.

### `t_69913635` — ui: make the no-access safety boundary explicit

Assignee: `pet-resort-code`

Adds the compact honesty card: built without live access, no live NVA/Gingr data, no customer sends, no PMS writes, no payment/schedule/medical decisions, read-only validation next.

### `t_ef0e12b3` — ui: add proof drawer behind the visual workflow

Assignee: `pet-resort-code`

Adds secondary proof drawer with staff-web smoke tests, local API side-effect-disabled proof, source refs/caveats, outcome proof, synthetic fixture boundary, and `./scripts/demo_owned_operations_api.sh`.

### `t_584e6bd7` — ui: add safe real-access next ask section

Assignee: `pet-resort-code`

Adds the close: “What real access would unlock” with read-only sample exports, field dictionaries, and BI query inventory; excludes writes/sends/payments/schedules/medical-safety actions.

### `t_e2f18143` — docs: add 3-5 minute manager brief demo script

Assignee: `pet-resort-docs`

Creates `docs/presentation/intelligible-manager-brief-demo-script.md` and links it from `docs/presentation/nva-demo-executive-brief.md`.

### `t_3007beee` — test: guard product-first intelligibility in staff-web smoke tests

Assignee: `pet-resort-code`

Adds regression checks preventing architecture-first phrases from returning and asserting the before/after/safety/next-ask anchors.

### `t_7f656a02` — qa: full local verification and fresh screenshot for presentation

Assignee: `pet-resort-reviewer`

Runs full gates, browser QA, and captures `/tmp/nva-intelligible-manager-brief-demo.png`. Makes small readability fixes only if necessary.

Required gates:

```sh
npm --workspace @pet-resort/staff-web run test
npm --workspace @pet-resort/staff-web run build
./scripts/demo_owned_operations_api.sh
python scripts/check_markdown_links.py --repo-root .
```

### `t_12a243fd` — review: final board closeout and presentation readiness note

Assignee: `pet-resort-reviewer`

Verifies final repo/board state and produces the readiness note: what is presentable, what remains synthetic/no-live-access, screenshot path, commits, gates, and safe next ask.

## Final readiness standard

The board is done only when:

- a non-technical viewer can explain the scenario in one sentence;
- the first 30 seconds show workflow, not architecture;
- the demo has a start/middle/end;
- at least one action is review-ready;
- at least one action is blocked;
- no live-access claims are made;
- no unsafe side effects are implied;
- proof points to local/runtime evidence;
- next ask is read-only and narrow;
- staff-web test and build pass;
- local API demo passes with side effects disabled;
- markdown links pass;
- fresh screenshot is captured.
