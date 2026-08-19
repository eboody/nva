# NVA Codebase Quality Remediation Implementation Plan

> **For Hermes:** Use subagent-driven-development and strict RED–GREEN–REFACTOR to implement this plan increment-by-increment. Shared-checkout mutations are serialized; independent read-only audits may run in parallel.

**Goal:** Make NVA consistently organized, semantic, expressive, extensible, and correct without weakening its fail-closed authority model or inventing unavailable provider facts.

**Architecture:** Preserve `domain -> app -> storage/integrations -> runtime/API` ownership while decomposing large boundary aggregators into cohesive modules. Compatibility translations move into explicit boundary codecs. Public APIs become smaller and authority issuance remains opaque. Characterization and architecture tests precede every structural move.

**Tech Stack:** Rust 2024, Cargo, PostgreSQL 17, SpacetimeDB, Next.js/TypeScript, Python, cargo-llvm-cov, Repowise, Clippy, Rustdoc.

---

## Clean-slate platform addendum (2026-08-18)

This addendum supersedes every compatibility-preservation requirement below. NVA is pre-live: no deployed consumer, persisted production data, historical wire shape, migration sequence, public path, alias, fixture, or test has backward-compatibility standing merely because it exists in the current tree.

Classify each such surface as **remove**, **replace with canonical current design**, or **keep for intrinsic current value**. Default to removal when history is its only justification. In particular, the clean-slate target is one current PostgreSQL schema, one current SpacetimeDB schema, one intentional `v1` API/OpenAPI contract, one semantic owner and canonical public path per concept, private raw boundary records that promote fallibly into domain/application values, and no legacy/historical compatibility vocabulary in production code or active documentation.

The implementation order is correctness, semantic fidelity, expressive types and APIs, coherent patterns, concept-owned organization, then maintainability and extensibility. Provider-native DTOs and identifiers belong in integration adapters; generic provenance and honest source uncertainty remain in the domain. Semantic enums, newtypes, module-local errors, builders, and typestate are preferred when they encode real distinctions or legal-operation phases rather than ceremony.

Clean-slate simplification must preserve the intrinsic safety invariants: `Observed<T> -> Candidate<T> -> Accepted<T>`, opaque and scope-bound `Authority<Action, Subject, Scope>`, fallible promotion, redacted diagnostics, exact target/subject/scope checks, and one-shot authority consumption. Caller-created or serialized evidence must remain unable to mint authenticated identity, acceptance/review, completion/contact, queue/outbox, payment/execution, measured labor, realized savings, or value.

The revised completion bar requires executable zero-compatibility-debt and canonical-path gates rather than allowlisting compatibility directories, historical replay, aliases, or old names. Stale compatibility tests are replaced with current behavioral, architecture, or compile-fail contracts through strict RED-GREEN-REFACTOR. Final approval requires a fresh exact-tree independent review against this clean-slate specification; earlier compatibility-preserving verdicts do not authorize the revised tree.

Where later sections say to preserve byte/schema compatibility, freeze historical contracts, retain temporary compatibility re-exports, isolate historical names, or run deployed-base/historical-upgrade verification, this addendum replaces that instruction with the canonical clean-slate design and fresh-schema/current-contract verification.

---

## Non-negotiable invariants

1. `Observed<T> -> Candidate<T> -> Accepted<T>` and `Authority<Action, Subject, Scope>` remain fail closed.
2. Caller-created, serialized, historical, compatibility, and label-only evidence cannot mint identity, review, completion, contact, queue, outbox, payment, execution, measured labor, realized savings, or value.
3. The `domain` crate remains storage-decoupled. Provider DTOs remain quarantined in integration boundaries.
4. PostgreSQL, SpacetimeDB, JSON, and OpenAPI expose only the canonical current contract; historical shapes are removed unless independently justified by a real current external protocol.
5. Every boundary-contract change begins with an explicit test for the intended canonical design and a deliberate schema/API decision.
6. Every production edit begins with a failing characterization, architecture, compile-fail, or behavior test.
7. `.vscode/` remains untouched and untracked.

## Completion bar

The project is complete only when all of the following are true:

- No production Rust source file exceeds 1,500 physical lines; boundary roots are thin routers/re-export surfaces, not subsystem implementations.
- `apps/api/src/http.rs`, `storage/src/operations.rs`, and `domain/src/entities.rs` are decomposed by owned concept with stable, narrow interfaces.
- Repowise reports zero import cycles for current source.
- Historical compatibility names and codecs are isolated under explicit `compatibility`/`codec` modules and do not leak into active domain or API vocabulary.
- Every public re-export has an ownership or compatibility rationale; broad convenience re-exports are removed.
- `cargo llvm-cov` produces workspace LCOV/HTML evidence. Changed production lines are covered; high-risk authority and boundary modules have explicit branch/negative-path tests.
- Repowise dead-code analysis has no confirmed safe-to-delete production finding.
- The exact final staged binary diff passes secret scanning, the complete enterprise release ladder, and independent specification/security/quality review.
- The committed tree is clean except for intentional untracked `.vscode/`.

## Phase 0: Baseline and enforce architecture quality

### Task 1: Add reproducible structural and coverage baselines

**Objective:** Convert the current assessment into executable, versioned evidence before refactoring.

**Files:**
- Create: `scripts/check_architecture_quality.py`
- Create: `scripts/tests/test_architecture_quality.py`
- Create: `docs/quality/codebase-quality-baseline.md`
- Modify: CI/pre-commit workflow owning workspace quality gates

**RED:** Add fixtures proving the checker rejects oversized production modules, newly introduced compatibility leakage, unrestricted broad re-exports, and dependency cycles represented in a checked manifest.

**GREEN:** Implement deterministic checks and generate the baseline without timestamps or machine-specific paths.

**Verify:**
```bash
uv run --with pytest python -m pytest scripts/tests/test_architecture_quality.py -v
uv run python scripts/check_architecture_quality.py --repo-root .
```

### Task 2: Install and prove Rust coverage reporting

**Objective:** Replace test-file naming heuristics with measured line/branch evidence.

**Files:**
- Create: `scripts/check_rust_coverage.sh`
- Create: `docs/quality/rust-coverage-policy.md`
- Modify: CI quality workflow

**RED:** The gate must fail when LCOV is absent, malformed, or omits a changed production Rust file.

**GREEN:** Use `cargo llvm-cov --workspace --all-features --all-targets` with DB-backed suites configured explicitly. Record line and branch evidence without claiming universal behavioral proof.

**Verify:**
```bash
./scripts/check_rust_coverage.sh
```

## Phase 1: Remove confirmed noise and shrink accidental API

### Task 3: Remove confirmed dead production exports

**Objective:** Delete only production symbols proven unused and unrequired by wire/schema compatibility.

**Initial target:** `apps/spacetimedb/src/read_model/staff_queue_item.rs::HygieneOutcomeCardRow`.

**RED:** Add or update compile/contract checks proving no supported public contract requires the export.

**GREEN:** Remove it and rerun SpacetimeDB schema/build tests.

### Task 4: Inventory and narrow public surfaces

**Objective:** Make public APIs intentional and concept-owned.

**Files:**
- Create: `docs/quality/public-api-inventory.md`
- Add compile-pass/compile-fail fixtures under owning crates
- Modify public `lib.rs`/module exports incrementally

**Acceptance:** Every retained re-export is canonical ownership or documented compatibility. No alias exists solely to avoid a semantic module path. Opaque authority constructors remain inaccessible.

## Phase 2: Decompose the HTTP boundary

### Task 5: Characterize the complete API contract

**Objective:** Freeze route paths, request/response JSON, OpenAPI, auth/error mapping, replay, and fail-closed behavior before moving code.

**Files:**
- Extend: `apps/api/tests/`
- Create: route-family characterization tests where absent

### Task 6: Split `apps/api/src/http.rs` by bounded capability

**Objective:** Turn `http.rs` into composition/router glue.

**Target modules:**
```text
apps/api/src/http/
  auth.rs
  health.rs
  inquiry.rs
  checkout.rs
  data_quality.rs
  manager_brief.rs
  site_finance.rs
  workflow.rs
  dto/
  error_mapping.rs
  router.rs
```

Move one route family per TDD increment. Keep public JSON/OpenAPI byte-for-byte stable. Extract shared semantics only when at least two route families genuinely own the same contract.

**Acceptance:** Root `http.rs` is below 500 lines; no route-family module exceeds 1,500 lines; no hidden API/worker contract coupling remains.

## Phase 3: Decompose storage ownership and break cycles

### Task 7: Characterize repositories, SQL admission, and codecs

**Objective:** Freeze PostgreSQL records, exact SQL authority predicates, idempotency, replay, atomic failure, and deployed-base migration behavior.

### Task 8: Split `storage/src/operations.rs` into repositories

**Target modules:**
```text
storage/src/operations/
  approval_outbox.rs
  data_quality.rs
  manager_daily_brief.rs
  site_finance.rs
  workflow.rs
  outcomes.rs
  compatibility/
```

Move one repository family at a time. Replace the six-operator outbox conditional with named predicates or an accepted-binding type proven by negative tests.

### Task 9: Break storage service-line cycles

**Objective:** Remove the `operations -> service_line -> operations` dependency cycle.

Introduce stable records/ports owned by the narrowest common parent. Do not move storage concerns into `domain`.

**Acceptance:** Repowise reports no storage cycle and tests prove unchanged rows/queries.

## Phase 4: Decompose core domain ownership

### Task 10: Characterize entity serialization and validation

**Objective:** Freeze semantic construction, custom deserialization, sensitive diagnostics, and compile-fail invariants before moves.

### Task 11: Split `domain/src/entities.rs` by concept owner

**Target owners:** customer, pet/animal, reservation, document, actor/staff, location, and identifiers. Preserve canonical semantic paths; use temporary compatibility re-exports only with an explicit removal task and test.

**Acceptance:** `entities.rs` becomes a thin module index below 500 lines; no owned concept file exceeds 1,500 lines; domain cycles are eliminated.

### Task 12: Decompose remaining oversized semantic modules

Incrementally split:
- `domain/src/training/mod.rs`
- `domain/src/operations.rs`
- `domain/src/source.rs`
- `app/src/data_quality_hygiene.rs`
- `app/src/tools.rs`
- `app/src/booking_triage.rs`
- `domain/src/workflow.rs`
- `app/src/manager_daily_brief.rs`

Each split follows concept ownership, not arbitrary line-count slicing.

## Phase 5: Isolate compatibility and reduce negative prose

### Task 13: Move historical compatibility into explicit codecs

**Objective:** Make legacy names visible only at wire/storage boundaries.

**Acceptance:** Active domain/app/API code uses current semantic vocabulary. Historical field names appear only in migrations, deployed row types, serde aliases, and compatibility tests.

**Status (2026-08-17): Complete.** The retained surfaces and removal conditions are recorded in `docs/quality/historical-compatibility-inventory.md`; the architecture baseline now permits zero compatibility leakage outside explicit codecs.

### Task 14: Replace denial-heavy prose with structural impossibility

**Objective:** Keep concise boundary warnings while eliminating repeated paragraphs where the type system now makes the forbidden interpretation impossible.

Do not weaken safety documentation. Prefer opaque types, private fields, non-serializable capabilities, accepted-state enums, and named predicates over comments alone.

## Phase 6: Cohesion, extensibility, and ownership proof

### Task 15: Add architecture dependency tests

Prove:
- `domain` has no storage/runtime dependency;
- provider DTOs do not enter domain truth directly;
- API/worker share explicit application contracts rather than hidden co-change coupling;
- compatibility modules do not become general dependencies;
- authority issuers are scarce, opaque, and non-serializable.

### Task 16: Establish bounded-context ownership documentation

**Files:**
- Create: `docs/architecture/bounded-context-map.md`
- Update: canonical README reader path and architecture ADRs

Document where each concept, error, conversion, compatibility codec, repository, and authority issuer belongs.

## Phase 7: Final quality convergence

### Task 17: Re-run health, dead-code, cycle, and coverage audits

**Acceptance:**
- zero current import cycles;
- zero confirmed deletable production exports;
- all completion-bar file-size limits met;
- measured changed-line coverage complete;
- remaining analyzer warnings are either remediated or documented false positives with executable contrary evidence.

Historical co-change scatter is reported as historical debt and cannot be erased by rewriting Git history. Success is a smaller current structural blast radius plus stable future interfaces.

### Task 18: Complete exact-tree release ladder

Run formatting, focused tests, workspace tests/all features/all targets, strict debug/release Clippy, strict Rustdoc/doctests, audit/deny, Python/docs/workspace checks, Staff Web unit/browser/build/standalone/audits, SpacetimeDB build, PostgreSQL 17 fresh/rerun/historical upgrade, coverage, architecture checks, secret scan, staged binary hash, and independent exact-index review.

Commit only after the exact current staged tree receives no specification, security, correctness, or quality blockers.

## Execution order and commits

- Commit this plan first.
- Execute tasks serially in the shared checkout, with read-only audits parallelized when useful.
- Commit after each independently green structural increment.
- Run Repowise `get_risk` before each hotspot and `get_change_risk` after each meaningful commit.
- Any behavior or contract drift stops the refactor and becomes a RED regression before correction.
