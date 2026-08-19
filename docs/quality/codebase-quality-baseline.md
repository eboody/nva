# Codebase architecture-quality baseline

This document records the executable Phase 0 architecture baseline for the NVA Rust workspace. The checked data lives in [`architecture-quality-baseline.json`](architecture-quality-baseline.json), [`public-api-inventory.md`](public-api-inventory.md) explains the ownership decisions, and [`scripts/check_architecture_quality.py`](../../scripts/check_architecture_quality.py) enforces them in CI.

## Policy

The gate is a ratchet, not an assertion that current debt is acceptable.

| Measure | Current checked debt | Completion threshold |
| --- | ---: | ---: |
| Production Rust files over 1,500 physical lines | 6 | 0 |
| Compatibility identifiers outside `compatibility` or `codec` modules | 9 | 0 |
| Exact rationale-owned wildcard public re-exports | 13 | 0 unreviewed declarations |
| Exact rationale-owned renamed public re-export declarations | 13 | 0 unreviewed declarations |
| Import dependency cycles in the checked analyzer manifest | 4 | 0 |

Every numeric debt value is an exact ceiling. Growth fails. A reduction also fails until the checked ceiling is reduced to the new measurement; this prevents later changes from consuming already-won improvement. Public wildcard and renamed re-exports use an exact declaration allowlist instead: additions fail, and removals make the inventory stale until it is ratcheted.

Run the gate with:

```bash
uv run python scripts/check_architecture_quality.py --repo-root .
```

The output contains only stable counts and repository-relative paths. It has no timestamps or machine-specific paths.

## Deterministic measurement

### Production Rust file size

A production Rust file is a `*.rs` file below a `src` directory, excluding `target`, `test`, `tests`, `examples`, `benches`, and files named `*_test.rs` or `*_tests.rs`. The checker counts physical lines with Python `splitlines()`.

Current oversized files:

| Path | Physical lines | Completion maximum |
| --- | ---: | ---: |
| `apps/api/src/http.rs` | 5,133 | 1,500 |
| `storage/src/operations.rs` | 3,941 | 1,500 |
| `domain/src/entities.rs` | 3,123 | 1,500 |
| `app/src/data_quality_hygiene.rs` | 1,959 | 1,500 |
| `domain/src/training/mod.rs` | 1,734 | 1,500 |
| `domain/src/operations.rs` | 1,689 | 1,500 |

The completion threshold applies to every production Rust file, including newly created files. Existing oversized files may not grow above their exact checked count.

### Compatibility leakage

The scanner removes Rust comments and string/character literals, then counts identifiers containing `legacy`, `compatibility`, or `historical`, case-insensitively. Modules with a path component named exactly `compatibility` or `codec` are explicit boundaries and are excluded from leakage debt.

Current active-boundary debt:

| Path | Identifier occurrences |
| --- | ---: |
| `apps/spacetimedb/src/authz.rs` | 3 |
| `apps/spacetimedb/src/reducers.rs` | 3 |
| `domain/src/analytics.rs` | 2 |
| `integrations/gingr/src/endpoint/commerce_retail.rs` | 1 |

The target is zero outside explicit boundary modules. Comments do not affect the metric, so safety explanation can remain complete. Serde strings and provider wire values also do not affect it; the rule targets active Rust vocabulary rather than required wire compatibility.

### Public re-exports and aliases

The scanner records exact `pub use ...::*;` and renamed `pub use ... as ...;` declarations in production Rust. Every retained declaration must match the machine-readable allowlist and carry a non-empty ownership or executable-runtime rationale. A new alias cannot pass merely because another declaration was removed from the same file.

Current broad surfaces:

| Path | Wildcard public re-exports |
| --- | ---: |
| `apps/spacetimedb/src/lib.rs` | 1 |

The SpacetimeDB root export is runtime ABI registration surface. The thirteen renamed re-export declarations are canonical role names at concept-owning boundaries and are enumerated in the public API inventory. The completion condition for this phase is zero unreviewed declarations, not deletion of required current runtime contracts. Later decomposition tasks may remove allowlisted entries only with their owning contract tests.

### Dependency cycles

CI reads cycles from the checked machine-readable manifest rather than calling Repowise. This keeps the gate offline and reproducible. The manifest source identifies the indexed commit used to refresh the cycle inventory.

Current cycle inventory:

1. `scc-0c9271a29584`, Apps API: `apps/api/src/http.rs` and `apps/api/tests/vaccine_review_regression_contract.rs`.
2. `scc-acdafbfe4e2a`, Domain: `domain/src/daily_brief.rs`, `domain/src/operations.rs`, and `domain/src/staff.rs`.
3. `scc-b04a1e3030f6`, Storage: `storage/src/operations.rs`, `storage/src/service_line/grooming.rs`, `storage/src/service_line/mod.rs`, and `storage/src/service_line/training.rs`.
4. `scc-bee8d2b9ae87`, SpacetimeDB: `apps/spacetimedb/src/lib.rs` and `apps/spacetimedb/src/realtime_queue_tests.rs`.

The checked count may not exceed four and must ratchet downward as cycles disappear. Completion requires a refreshed analyzer inventory with zero cycles.

## Refresh protocol

After an architecture change:

1. Run Repowise against the exact current commit and inspect its cycle pages.
2. Replace the manifest cycle inventory and source commit with that analyzer output.
3. Run the checker. If source debt fell, lower each changed path ceiling to the exact reported measurement; remove a path once it reaches the completion threshold.
4. Run the focused Python suite and the containing repository-quality suite.
5. Review the manifest diff. Never raise a ceiling merely to make a new violation pass.

The manifest is evidence, not authority over application behavior. This gate does not modify domain types, API/OpenAPI contracts, PostgreSQL or SpacetimeDB schemas, boundary codecs, replay behavior, or side-effect posture.
