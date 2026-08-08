# Labor-Cost Agent Platform + Docs-as-Contracts Kanban Plan

> **For Hermes:** Use Kanban orchestration with isolated worktrees for parallel implementation cards and active owner-review gates. The goal is not generic docs polish; the acceptance lens is whether the repo becomes easier for humans and agents to use to lower pet-resort labor costs safely.

**Goal:** Convert the repo assessment and Rustdoc/doctest direction into a parallel Kanban execution graph that strengthens the NVA pet-resort repository as a labor-cost-reduction platform.

**Architecture:** Preserve the existing crate boundaries: `domain` owns semantic truths, `app` owns use-case workflows and review gates, `storage` and `integrations/gingr` own boundary promotion/demotion, and `apps/*` stay thin runtime shells. READMEs remain wiki/navigation; Rustdoc examples become compile-checked API contracts; README/link tests protect the wiki shape.

**Primary acceptance lens:** Every card should either reduce future engineering/agent labor, prove a safer automation contract, or make the next labor-saving workflow easier to build and review. `nva-pet-resorts-ai-context.md` is required context for every card.

---

## Board strategy

Board slug: `nva-labor-cost-doc-contracts`

Parallelism strategy:

- Use existing profiles plus cloned parallel workers: `pet-resort-code`, `petresortcode2`, `petresortcode3`, `pet-resort-docs`, `petresortdocs2`, `pet-resort-reviewer`, and `default`.
- Every code/docs implementation card uses an isolated Kanban worktree branch so independent lanes can run at the same time.
- The final fan-in card merges/reconciles worktree outputs back into the main checkout and runs repo-wide gates.
- Routine `review-required` gates are owner-review work, not user blockers, unless the card touches live/member-facing actions, secrets, destructive operations, or product decisions.

## Immediate parallel root lanes

### 1. Safety hardening: unknown draft side effects fail closed

**Why it matters:** The current readiness review says this is the remaining blocker before any pilot/live posture. It protects labor-saving agents from silently expanding permissions.

**Expected files:**

- `app/src/manager_daily_brief.rs`
- `app/tests/manager_daily_brief_workflow_contracts.rs`
- possibly `apps/api/tests/manager_daily_brief_agent_drafts_contract.rs`

**Acceptance criteria:**

- Unknown non-empty draft requested side effects are rejected as unsupported.
- Known blocked side effects still produce explicit blocked-action evidence.
- Outcome capture and draft validation are aligned fail-closed.
- Focused tests and `cargo test --workspace --no-run` pass.

### 2. Canonical docs-as-contracts policy

**Why it matters:** READMEs should guide humans/agents, while Rustdoc examples should compile. This reduces repeated orientation labor and stale examples.

**Expected files:**

- `README.md`
- `domain/README.md`
- `app/README.md`
- `storage/README.md`
- `integrations/gingr/README.md`
- optionally `docs/architecture/agent-app-infrastructure.md`

**Acceptance criteria:**

- README/wiki policy states that READMEs are navigational and source Rustdoc contains executable examples.
- The root README links the doctest policy and labor-cost lens.
- No duplicate stale code examples are added to READMEs unless intentionally non-executable.
- Markdown local link check passes.

### 3. Rustdoc/doctest infrastructure and lint gate

**Why it matters:** Compile-checked docs are only valuable if the repo has an obvious way to run them and catch broken intra-doc links.

**Expected files:**

- crate roots: `domain/src/lib.rs`, `app/src/lib.rs`, `storage/src/lib.rs`, `integrations/gingr/src/lib.rs`
- `scripts/test.sh`
- maybe a new `scripts/check_docs.sh`
- README verification section

**Acceptance criteria:**

- Crates opt into `rustdoc::broken_intra_doc_links` checking where feasible.
- There is a documented command for Rust doctests and markdown link checks.
- `cargo test --workspace --doc` or the chosen equivalent is wired/documented.
- Repo test script remains green.

### 4. Labor-cost context crosswalk

**Why it matters:** The repo should keep deciding work by labor-cost reduction, not by whatever source system or workflow is loudest.

**Expected files:**

- `docs/design/labor-cost-reduction-crosswalk.md` or similar
- update links from `README.md`/architecture docs as appropriate

**Acceptance criteria:**

- Map major labor-cost drivers from `nva-pet-resorts-ai-context.md` to current/future repo surfaces.
- Separate source-of-record questions, BI/read models, workflow automation, and agent review loops.
- Name the top 3 next labor loops with safety boundaries and measurable outcomes.

### 5. Data-quality hygiene second-workflow spec

**Why it matters:** The readiness review recommends data-quality hygiene as the second workflow because it amplifies every later labor-saving loop.

**Expected files:**

- `docs/design/data-quality-hygiene-labor-loop.md`
- possibly updates to `docs/architecture/agent-app-infrastructure.md`

**Acceptance criteria:**

- Define context endpoint, draft endpoint, outcome endpoint, action kinds, blocked actions, personas, source refs, and labor metric.
- Preserve data-quality ambiguity as manager-visible work, never hidden or auto-resolved.
- The spec is ready for implementation without guessing.

### 6. Domain/service Rustdoc examples

**Why it matters:** The domain model is the repo glossary. Compile-checked examples prove the semantic module paths and construction patterns.

**Expected files:**

- `domain/src/boarding/deposit.rs`
- `domain/src/boarding/capacity.rs` if present/applicable
- `domain/src/daycare/*`
- `domain/src/payment/mod.rs`
- `domain/src/source.rs`
- `domain/src/daily_brief.rs`

**Acceptance criteria:**

- Each touched module has a short `//!` rustdoc explanation and at least one compiling example.
- Examples use semantic module paths, not flattening aliases.
- Examples show labor-relevant review/policy/source truths, not toy strings.
- `cargo test -p domain --doc` passes.

### 7. App workflow/tool Rustdoc examples

**Why it matters:** The app crate is where deterministic workflow contracts become agent-safe prompt/context/draft/outcome loops.

**Expected files:**

- `app/src/manager_daily_brief.rs`
- `app/src/booking_triage.rs`
- `app/src/tools.rs`
- `app/src/agents.rs` if needed

**Acceptance criteria:**

- Rustdoc examples show context packet -> draft validation -> outcome capture or equivalent workflow slices.
- Examples make blocked actions and review gates visible.
- Examples avoid raw provider/system mutation authority.
- `cargo test -p app --doc` passes.

### 8. Storage + Gingr boundary Rustdoc examples

**Why it matters:** Labor-saving automation depends on source-grounded promotion/demotion. Provider/record boundaries must not become the domain model.

**Expected files:**

- `storage/src/operations.rs`
- `storage/src/service_line/*`
- `integrations/gingr/src/mapping/*`
- `integrations/gingr/src/endpoint/*`
- `integrations/gingr/src/webhook.rs` if feasible

**Acceptance criteria:**

- Examples show provider DTO/source refs/promoted candidates/storage records without leaking boundary vocabulary upward.
- Storage examples show explicit promotion/demotion and stable codes.
- Gingr examples stay fixture-safe and secret-free.
- `cargo test -p storage --doc` and `cargo test -p gingr --doc` or equivalent package names pass.

### 9. README/wiki coverage and local-link contract test

**Why it matters:** The wiki is now large enough that broken links and missing module READMEs create real agent/human labor.

**Expected files:**

- `scripts/check_markdown_links.py` or similar
- `scripts/test.sh`
- README verification docs

**Acceptance criteria:**

- Local markdown links are checked while excluding `.git`, `target`, `node_modules`, and generated/vendor trees.
- Required crate/module README coverage is asserted for current workspace crates and major domain modules.
- The script is deterministic and can run locally without network/secrets.

## Gated implementation lanes

### 10. Data-quality hygiene domain/app/storage implementation

**Parents:** safety hardening and data-quality workflow spec.

**Expected files:**

- `domain/src/data_quality.rs` and/or new semantic submodules
- `app/src/data_quality_hygiene.rs` or appropriate app module
- `app/src/lib.rs`
- `storage/src/operations.rs`
- tests under `domain/tests`, `app/tests`, and `storage/tests`

**Acceptance criteria:**

- Adds typed context/actions/outcome capture for internal data-quality hygiene.
- Encodes allowed internal work and blocked external actions.
- Captures estimated and actual minutes saved.
- Preserves source refs and issue provenance.
- Focused tests pass.

### 11. Data-quality hygiene API/runtime shell

**Parents:** data-quality implementation.

**Expected files:**

- `apps/api/src/http.rs`
- `apps/api/tests/*data_quality*` or similar
- `apps/cli/src/main.rs` and/or `apps/worker` only if needed for local demo

**Acceptance criteria:**

- Adds local/sandbox API endpoints for context, draft validation, and outcome capture.
- Does not add live sends, provider writes, schedule changes, or payment/refund/discount movement.
- Contract tests prove blocked action behavior.

### 12. Data-quality hygiene local smoke/runbook

**Parents:** API/runtime shell and docs/test infrastructure.

**Expected files:**

- `docs/ops/data-quality-hygiene-local-smoke.md`
- `scripts/smoke_data_quality_hygiene_local_loop.sh` if useful
- README links

**Acceptance criteria:**

- A local fake-data smoke proves context -> draft -> outcome capture.
- Smoke records estimated vs actual labor minutes saved.
- Runbook states no live/customer/provider side effects.

## Fan-in and review lanes

### 13. Integration fan-in, merge reconciliation, and repo-wide gates

**Parents:** all root implementation/doc lanes plus data-quality smoke.

**Acceptance criteria:**

- Reconcile parallel worktrees cleanly into the main checkout.
- Run `cargo fmt --check`, `cargo test --workspace --no-run`, doctest/docs gates, `./scripts/test.sh`, markdown link checks, and `git diff --check`.
- Resolve concrete conflicts or semantic drift.
- Produce a final summary of what changed and what is still not pilot/live-ready.

### 14. Labor-cost platform readiness memo

**Parents:** crosswalk, data-quality smoke, and final fan-in.

**Expected files:**

- `docs/audits/2026-06-18-labor-cost-platform-readiness.md` or dated equivalent
- README status refresh if needed

**Acceptance criteria:**

- Score repo against the labor-cost-reduction objective.
- Name the next best workflow(s), their measurable labor metric, and safety gates.
- Distinguish local/demo, sandbox, pilot, and live readiness.
- No vague “AI program” claims without source-grounded app contracts.

## Global guardrails for every worker

- Read `nva-pet-resorts-ai-context.md`, root `README.md`, `docs/architecture/agent-app-infrastructure.md`, and the relevant crate README before editing.
- Preserve semantic module paths and boundary layering.
- Do not introduce live/member-facing actions, PMS writes, schedule mutations, customer sends, payment/refund/discount actions, or secret-dependent runtime behavior.
- Keep examples fixture-safe and secret-free.
- Prefer narrow tests and doctests first, then repo-wide gates in fan-in.
- If a card blocks with routine `review-required`, the owner/reviewer should clear it with evidence rather than requiring user intervention.
