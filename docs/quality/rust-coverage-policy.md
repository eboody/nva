# Rust coverage policy

## What this gate proves

`scripts/check_rust_coverage.sh` generates measured workspace coverage with `cargo-llvm-cov`, validates the LCOV structure, and rejects uncovered executable lines in changed production Rust files. It replaces filename-pairing heuristics: an integration test such as `app/tests/manager_daily_brief_workflow_contracts.rs` can cover any instrumented workspace source file regardless of its name.

Coverage is evidence that an instrumented line or branch executed. It is not universal behavioral proof, proof that every semantic outcome was asserted, or permission to weaken negative-path, compile-fail, authority, wire, schema, migration, replay, or no-live-side-effect tests. High-risk boundaries still require explicit behavior and branch tests.

## Canonical command and evidence

From the repository root, with a migrated PostgreSQL 17 test database containing the checked-in safe synthetic fixtures:

```bash
export DATABASE_URL=postgres://pet_resort@127.0.0.1:5432/pet_resort
export TEST_DATABASE_URL="$DATABASE_URL"
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f fixtures/seed/local-demo.sql
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f fixtures/seed/local-demo-data-quality.sql
./scripts/check_rust_coverage.sh
```

Branch instrumentation currently requires nightly Rust. Generation therefore uses the reproducibly pinned `nightly-2026-08-15` toolchain (override only with `RUST_COVERAGE_TOOLCHAIN`) and the locked workspace command:

```bash
cargo llvm-cov --workspace --all-features --all-targets --locked --branch --lcov
```

`--all-targets` deliberately includes unit tests, integration tests, binaries, examples, and other Cargo test targets. The gate refuses to generate if either `DATABASE_URL` or `TEST_DATABASE_URL` is absent, because the storage authority suites otherwise report that their live PostgreSQL contracts were skipped. The caller must apply `migrations/` and the two checked-in safe synthetic fixtures before generation: the Postgres backlog contract deliberately proves that an unrecognized fixture source system fails closed. CI performs those steps against a PostgreSQL 17 service before invoking the gate.

`DATABASE_URL` is used for migration setup, then removed from the instrumented test process. Runtime/API tests intentionally characterize the safe not-configured state and must not be changed merely to collect coverage. `TEST_DATABASE_URL` remains set for the storage PostgreSQL contract suites, which are the DB-backed tests that explicitly consume it.

The six `trybuild` compile-fail harness functions are explicitly skipped only in the nightly coverage invocation. Their value is compiler-diagnostic contract checking rather than runtime source execution, and nightly emits suggestion text that intentionally differs from the stable 1.96 snapshots. This includes the SpacetimeDB historical-row path fixture, whose nightly compiler adds an import-help suggestion absent from the stable snapshot. The normal stable `cargo test --workspace --all-targets --all-features --locked` CI job still runs every compile-fail harness. All other targets, including integration tests whose filenames do not mirror source filenames, remain in the measured run.

The workspace build wrapper relocates instrumented workspace test executables under `target/llvm-cov-target/debug/build/<package>/.../out`, outside the object directory that `cargo-llvm-cov` scans automatically. The gate discovers executables only for members declared in the root workspace manifest, exports them together with the same merged profile, and appends only repository production-source records to LCOV. Absence of the relocated objects or their source records fails generation rather than silently dropping a workspace crate.

Generated evidence is deterministic and machine-local:

- `coverage/lcov.info`: complete line and branch records;
- `coverage/rust-coverage-summary.json`: totals plus changed-production-line results;
- `coverage/html/index.html`: browsable source-level line and branch evidence generated from the same profile data;
- `docs/quality/rust-coverage-baseline.json`: checked-in point-in-time summary for the remediation baseline.

The `coverage/` directory is ignored by Git and uploaded by CI as the `rust-coverage` artifact. The compact checked-in baseline contains no timestamp or machine-specific path.

## Changed-production-line rule

The comparison base is, in order:

1. `--base REF`;
2. `RUST_COVERAGE_BASE_REF`;
3. the merge base of `HEAD` and `origin/main`;
4. `HEAD^`, or `HEAD` in a one-commit repository.

A production Rust file is a changed `*.rs` path under a `src/` directory. For every added or modified production file:

- the file must have an `SF` record in LCOV unless its comment-stripped body is declaration-only: modules, imports/re-exports, compile-time constants, fields-only structs/enums, or traits without default method bodies, which LLVM does not instrument or emit as an `SF` record;
- every changed line that has a `DA` record is executable and must have a hit count greater than zero;
- changed comments, whitespace, attributes, type declarations, and other lines without a `DA` record are reported as non-executable rather than assigned fabricated coverage;
- deleted lines are outside changed-line coverage because no current executable line exists.

The gate does not impose a repository-wide percentage floor during the remediation baseline. It ratchets the stronger policy that changed executable production lines are fully covered while preserving honest measured overall line and branch totals.

## Validation-only mode

Tests and reviewers can validate existing evidence without rerunning the Rust suite:

```bash
./scripts/check_rust_coverage.sh \
  --no-generate \
  --base HEAD^ \
  --lcov coverage/lcov.info \
  --summary coverage/rust-coverage-summary.json
```

Validation fails closed when LCOV is absent, unreadable, structurally malformed, has no complete source records, has no branch records, omits a changed production Rust file, or reports a changed executable line with zero hits.

## Baseline interpretation

The checked-in baseline is a measurement of the current instrumented suite, including explicitly configured DB-backed tests. It is not a promise that unchanged legacy lines have sufficient assertions. Structural remediation tasks must keep changed executable lines green and add targeted negative-path tests wherever authority, compatibility, PostgreSQL, SpacetimeDB, API/OpenAPI, sensitive diagnostics, replay, or side-effect boundaries are touched.
