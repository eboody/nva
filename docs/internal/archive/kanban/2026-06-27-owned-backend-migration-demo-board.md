# Owned Backend Migration + Labor-Cost Demo Kanban Board

Status: execution board for turning the current NVA pet-resort proof into a stronger job-contact / executive demo.

Strategic objective: demonstrate that NVA could build an owned operations backend above Gingr/source systems, migrate away piece-by-piece, and use that backend to build labor-cost-reduction tools.

Safety boundary: all work remains synthetic/local/demo unless separately approved. No live NVA/Gingr access is claimed. No live customer messages, PMS/provider writes, payment/refund actions, schedule/capacity changes, merge/delete operations, or medical/safety decisions.

## Board shape

Shared repo: `/home/eran/code/nva`

Because cards share one checkout by default, implementation cards should be serialized unless a worker explicitly creates/uses an isolated worktree. Routine review-required gates may be cleared by owner review with focused verification; escalate only for credentials, destructive operations, live/member-facing actions, or true product-owner decisions.

## Executive acceptance bar

A job contact or senior operator should be able to understand these claims in under five minutes:

1. Gingr can remain a source system initially, but NVA should own the operating model.
2. The backend has the expected pieces: source adapters, provenance, owned domain contracts, versioned API, Postgres projections, read models, review gates, audit/outbox posture, metrics, staff UI, and AI/draft safety boundaries.
3. Replacement is incremental: read-only source evidence → owned workflow authority → BI/read-model replacement → controlled side-effect adapters → workflow-by-workflow migration.
4. Labor-cost tools are built on the owned backend, not one-off demos.
5. The pilot ask is narrow and safe: approved read-only docs/exports/sample rows/BI query inventory for one or two workflows.

## Cards

### 1. Strategy spine: piece-meal Gingr replacement narrative

Assignee: `pet-resort-docs`

Objective: make the canonical docs and demo copy frame the project as an owned backend migration path, not merely an AI dashboard.

Expected artifacts:
- Update or create a concise architecture/product note under `docs/presentation/` or `docs/architecture/`.
- Update `docs/presentation/nva-demo-executive-brief.md` if needed.
- Name the migration phases: source evidence, owned workflow authority, BI/read-model replacement, controlled outbox/writeback, workflow-by-workflow replacement.

Acceptance criteria:
- The phrase “Gingr is source evidence, not product authority” is preserved or improved.
- The no-access boundary is framed as safe judgment, not weakness.
- The doc clearly states what access unlocks next without asking for live writes.

Verification:
- `./scripts/check_docs.sh`
- `python scripts/check_markdown_links.py --repo-root .` if practical; otherwise document any pre-existing unrelated failures.

### 2. Demo system map: show all pieces one would expect in the backend

Assignee: `pet-resort-code`
Parent: card 1

Objective: add a visible section to `apps/staff-web/app/page.tsx` showing the owned-backend system pieces and their current proof status.

Expected UI pieces:
- Source adapters / provider evidence / provenance.
- Owned domain model.
- Versioned `/v0` operations API.
- Postgres migrations and projections.
- Review gates / safety policies.
- Audit trail / outbox posture.
- BI/read models / metrics.
- Staff tools / AI draft boundary.

Acceptance criteria:
- The section distinguishes `implemented in demo`, `contract-proven`, and `future live integration` rather than pretending production readiness.
- It visually supports the piece-meal replacement thesis.
- The public page still loads and the live technical proof still works.

Verification:
- `npm --prefix apps/staff-web run lint` if configured.
- `npm --prefix apps/staff-web run build` if practical.
- Browser/snapshot or curl smoke against local or deployed app.

### 3. Labor tools portfolio: show multiple tools on the same backend

Assignee: `pet-resort-code`
Parent: card 2

Objective: make the demo show at least three labor-cost tool slices that all rely on the owned backend.

Required tool cards:
- Data-quality hygiene: source reconciliation labor.
- Intake/booking triage: front-desk parsing/draft labor.
- Manager daily brief: manager context-gathering labor.

Optional fourth card if cheap:
- Checkout/retention/completion: follow-up/revenue leakage labor.

Acceptance criteria:
- The UI explicitly says these are not separate apps; they are tools built on one owned operations backend.
- Each tool names the backend pieces it uses: source refs, review packets, read models, outcome metrics, outbox/review safety.
- Current API-backed data-quality proof remains live; non-live tool cards are accurately labeled as domain/contract-backed or prototype.

Verification:
- Staff-web lint/build.
- Browser snapshot confirms all three tools are visible and wording is honest.

### 4. API/read-model proof expansion decision + minimal implementation

Assignee: `pet-resort-code`
Parent: card 3

Objective: decide whether to add live API endpoints/read models for intake triage and manager daily brief now, or keep them as contract-backed prototype cards; implement the smallest safe proof if it materially improves the demo.

Decision rubric:
- Add endpoints only if they can reuse existing app/domain contracts and avoid broad schema churn.
- Do not invent fake production authority.
- Prefer one high-signal read-only endpoint over multiple shallow mock endpoints.

Acceptance criteria:
- Either a new live proof endpoint exists and is surfaced in the demo, or a short decision note explains why the current API-backed data-quality slice is sufficient for this pass.
- No unsafe side effects are introduced.
- OpenAPI/docs stay accurate if endpoints are added.

Verification:
- Relevant Rust tests, especially API contracts.
- `./scripts/demo_owned_operations_api.sh`

### 5. CEO/pilot close: ROI scaler and read-only pilot ask

Assignee: `pet-resort-docs`
Parent: card 4

Objective: add CEO-facing closeout copy and/or UI that translates the backend into a cautious 170-location pilot story.

Expected content:
- Conservative ROI/labor-savings scaler labeled illustrative only.
- “What I would ask for next”: read-only exports/sample rows/field dictionaries/BI query inventory.
- Scope exclusions: no live writes, no customer messaging, no payment/schedule/medical/safety decisions.
- 2–3 week validation/pilot framing if appropriate.

Acceptance criteria:
- The close makes the next step obvious to a job contact or senior operator.
- It does not overclaim real NVA data, real savings, or production readiness.

Verification:
- Docs check.
- Browser snapshot or screenshot confirms the closeout is visible.

### 6. Final integration, deploy, and presentation QA

Assignee: `pet-resort-reviewer`
Parent: card 5

Objective: independently verify the demo is sendable/presentable after the migration-platform pass.

Review checklist:
- Page tells the owned-backend replacement story in the first minute.
- Live proof still shows API calls, status, latency, DB rows, and JSON artifacts.
- System-piece map is honest and useful.
- Labor tools portfolio clearly supports labor-cost reduction.
- Pilot ask is safe and concrete.
- No claims of live NVA/Gingr access, production deployment, or real customer data.

Verification commands:
- `git diff --check`
- `./scripts/check_docs.sh`
- `./scripts/demo_owned_operations_api.sh`
- Staff-web lint/build if practical.
- Deploy through existing Coolify path only if app changes need public refresh, then verify https://nva-demo.eman.network in browser.

Definition of done:
- Board summary includes changed files, verification evidence, live URL status, remaining caveats, and recommended 3-minute talk track.
