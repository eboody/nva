# Modum semantic lint remediation closeout

Status: integrated and owner-verified on `main`.

Base before remediation: `c88d2c82`.
Final integrated commit: `676fe80` (`fmt: normalize modum fan-in integration`).

## What changed

Six parallel remediation lanes were integrated:

- Gingr boundary typed surfaces: request parts, endpoint/path/status/id/error/provider values now promote provider concepts into typed Rust surfaces earlier.
- Policy/play ownership: policy concepts now live under stronger owners such as `policy::play`, `policy::automation`, and `policy::denial` instead of broad flattened names.
- Boarding/daycare boundaries: service-line contracts gained owned child facets for minimum stay, cancellation, housekeeping, handoff, attendance, incident, and assignment/playgroup surfaces.
- Operations constructors and semantic leaves: public positional constructors were replaced or hidden behind named typed inputs/builders; operations action/risk and grooming/training/retail leaves were tightened.
- Root export cleanup: removed `domain::prelude`, promoted service-line modules to canonical root paths (`domain::{boarding,daycare,grooming,retail,training}`), and cleaned flat Gingr/storage root exports.
- Call-site/import cleanup: call sites now preserve meaningful namespace context for workflow, payment, app tools, and service-line imports.

## Verification

Owner reran these gates after fan-in:

```bash
cargo fmt --check
cargo test --workspace
git diff --check c88d2c82..HEAD
modum check --format json
```

Results:

- `cargo fmt --check`: pass
- `cargo test --workspace`: pass
- `git diff --check c88d2c82..HEAD`: pass
- `modum check --format json`: exit 2 because follow-up diagnostics remain, but the report runs successfully

Final Modum count after integration:

- scanned files: 96
- files with violations: 47
- diagnostics: 227

Top remaining diagnostic families:

- `api_redundant_leaf_context`: 67
- `namespace_flat_pub_use_redundant_leaf_context`: 47
- `api_candidate_semantic_module`: 42
- `namespace_flat_pub_use`: 14
- `namespace_flat_use`: 13
- `api_candidate_semantic_module_unsupported_construct`: 11
- `namespace_redundant_qualified_generic`: 6
- `api_candidate_child_facet_module`: 5

## Accepted / intentionally deferred lint pressure

These remaining diagnostics are not automatically defects. Treat Modum as a discovery tool, not an oracle.

Accepted or deferred examples:

- `transport::Error` in Gingr remains acceptable because `transport` is the SDK transport boundary; a longer leaf would be less truthful.
- `storage::service` remains a separate persistence-boundary decision; do not flatten it just to satisfy a catch-all warning.
- `policy::automation::Rationale` child-facet pressure is a possible follow-up, but the approved public shape is currently `policy::automation::{Level,Rationale,Rule}`.
- `training::package::id::Id` has contradictory parent-surface / redundant-qualified-generic pressure; resolve with a focused training/package design pass, not a mechanical rename.
- Macro/source-inference diagnostics are informational unless the linter can point to a concrete safe rewrite.
- External derive/proc-macro imports such as `use thiserror::Error;` inside local `error.rs` modules should not be treated as caller-facing domain surfaces.

## Doctrine for future code

When adding new Rust domain/API code in this repo:

1. Prefer semantic module ownership over flat public aliases.
2. Preserve minimum necessary qualified paths at call sites; do not hide important domain context through broad preludes.
3. Use typed boundary values for provider IDs, statuses, paths, emails, errors, and other protocol concepts.
4. Avoid public constructors with many positional parameters; use named builders or cohesive typed input structs.
5. Keep `error.rs` as an implementation module when it owns module-local `Error`/`Result`; parent modules may re-export the canonical error surface.
6. Do not follow Modum suggestions mechanically. Classify each lint as good, directionally right with a better semantic solution, informational/manual review, or bad for this repo's doctrine.

## Board closeout

Kanban board: `pet-resort-modum-audit`.

Completed active cards:

- 6 implementation lanes
- 1 fan-in integration lane

The original per-diagnostic audit backlog remains parked as scheduled historical/follow-up work, not active blocking work.
