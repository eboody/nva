# Codebase quality convergence audit

Captured at `2026-08-19T02:22:55Z` for the dirty remediation worktree. This audit separates live source evidence from the commit-only Repowise index so that stale analyzer findings are not mistaken for executable proof.

## Live architecture inventory

`python scripts/check_architecture_quality.py --repo-root .` reports:

- 204 production Rust files in the checked architecture inventory
- zero files over the 1,500-line completion maximum
- zero compatibility leakage findings
- zero broad re-export findings
- zero public re-export alias findings
- zero checked entity-owner cycles
- zero recorded dependency cycles

The checker excludes test-only Rust modules under production source trees. Its live ratchet inventory includes one accepted broad re-export and 12 accepted public re-export aliases, with zero findings above the checked baseline. The largest current production file remains `apps/api/src/http/manager_brief.rs` at 1,355 lines.

The one verified source-level cycle was `domain/daily_brief -> operations -> staff -> daily_brief`. `staff::Role` remains concept-owned by staff but now lives in `domain/src/staff/role.rs`; `staff.rs` re-exports it and `operations::labor` imports the owner submodule directly. This removes the file-level back edge while preserving the canonical `domain::staff::Role` path. `domain/tests/domain_structural_convergence.rs` characterizes that ownership and public-path contract.

## Repowise import-cycle interpretation

Repowise is indexed at commit `285df2585e2a` and does not include this worktree's uncommitted source. Its overview still reports four cycles. They are retained here as analyzer evidence, not accepted as live source truth:

- API: a production module and an integration-test contract are grouped because the test imports the product. Current source contains no reverse product-to-test import.
- SpacetimeDB: `#[cfg(test)] mod realtime_queue_tests` and test imports of read-model modules form a test-build SCC, not a production runtime cycle.
- Storage: the reported service-line back edge is a README link to `operations.rs`; service-line Rust imports `crate::projection`, not `crate::operations`.
- Domain: the prior `daily_brief`, `operations`, and `staff` cycle was genuine and is the cycle removed by the staff role-owner split above.

The checked live baseline therefore records zero dependency cycles. A future clean commit and Repowise reindex should be used to retire the stale SCC pages rather than adding false-positive cycles back to the baseline.

## Dead-code inventory

Repowise reports 16 total findings, including a stale high-confidence finding for the former `HygieneOutcomeCardRow`. The clean-slate schema now exposes only `HygieneOutcomeCardRow` as the canonical current row, and its public-path contract compiles that one surface without parallel historical/current row types.

The lower-confidence unreachable-file results are also entry-point false positives rather than deletion proof. Examples include quality scripts invoked from `scripts/check_docs.sh`, `scripts/test.sh`, `package.json`, and tests, plus `apps/hermes-processor/processor.py`, which is the Dockerfile entrypoint. No dead code was deleted from analyzer output alone.

## Measured Rust coverage

The nightly branch-coverage run executes all workspace targets with PostgreSQL contract tests enabled. Investigation found that the workspace build wrapper relocates package test executables under `target/llvm-cov-target/debug/build/<package>/.../out`; stock `cargo-llvm-cov` did not scan those objects and silently omitted entire crates. `scripts/check_rust_coverage.sh` exports those executables against the same merged profile, restricts appended evidence to workspace production sources, rejects missing relocated evidence, and documents the six nightly-only trybuild skips.

The same canonical invocation now also starts a pinned `clockworklabs/spacetime:v2.6.0` loopback server, requires the exact 2.6.0 CLI plus Clang, publishes the production module through the coverage harness, executes reducer authorization/rejection/zero-write/replay and projection paths, validates the resulting SpaceTimeDB WASM LCOV, and merges it with native evidence. Missing, empty, malformed, optional, or non-executed WASM evidence fails the gate.

The final merged LCOV inventory reports:

- 20,835 / 24,776 lines covered (84.09%)
- 857 / 1,196 branches covered (71.66%)
- 29 / 29 changed executable lines covered against the exact independently reviewed staged-tree baseline
- zero changed executable lines uncovered

There is no suppression or zero-hit allowance. Serializable `ConsentEvidence` still fails closed and cannot authorize contact. The previously unreachable successful `ResponsePacket` construction arm is measured only through a crate-private test issuer of opaque `AcceptedConsent`; the test exercises the same validation and construction body while proving live send remains unavailable. Native and WASM business semantics remain shared.

## Repowise health snapshot

At indexed commit `285df2585e2a`, Repowise reports:

- average health 7.56 overall and 4.75 for code-only NLOC weighting
- 753 analyzed files, with 560 healthy, 154 warning, and 39 alert
- 193 files below the 8.0 target
- no ingested coverage data
- highest-leverage untested hotspot: `apps/api/src/http.rs`

Several size and test-pairing details in that snapshot predate the dirty worktree's module splits. Use it to prioritize the next clean-commit remediation, but use the live architecture and LCOV gates above for acceptance.

## Verification status

- canonical native plus loopback SpaceTimeDB WASM coverage generation and strict changed-line gate
- architecture checker and its focused Python suite
- Rust coverage gate unit tests
- executable CI release contract for pinned WASM prerequisites and canonical invocation
- domain consent/lead and authority compile-fail contracts
- SpaceTimeDB all-features/all-targets tests and `wasm32-unknown-unknown` build
- workspace all-features/all-targets tests with database URLs unset
- strict debug and release Clippy with warnings denied
- Rustdoc with warnings denied and workspace doctests
- workspace, architecture, Markdown-link, public-landing, and Python compilation checks
- `cargo fmt --all -- --check`
- `git diff --check`

The corrected tree still requires a fresh independent exact-tree review. This audit records executable evidence; it does not approve the correction or authorize merge/release.

The strict `python scripts/check_rustdoc_completeness.py` gate passes after the bounded correction documented the public `staff::role` module in `domain/src/staff.rs`; that documentation-only correction changed neither behavior nor API shape.

Repowise scores the committed remediation range at the 97.6th risk percentile with high review priority and an Elevated classification. That range contains 602 files across 24 subsystems and does not include the final uncommitted convergence edits; the large range and missing ingested per-test map reinforce the independent review requirement.
