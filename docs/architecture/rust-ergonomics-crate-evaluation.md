# Rust Ergonomics Crate Evaluation

**Date:** 2026-06-19

**Scope:** evaluate whether iterator/path/string-storage/enum-indexing ergonomics crates should be added now: `itertools`, `camino`, `smol_str` or `compact_str`, `enum-map`, and adjacent small helpers. The decision rule is conservative: add a dependency only when repeated production call sites become clearer, safer, or measurably cheaper than the standard-library shape.

## Recommendation

Do **not** add any of these crates yet.

The concrete audit found one small code improvement, not a repeated dependency-worthy pattern: `apps/api/src/http.rs` used stable `sort(); dedup();` for a data-quality rejection-reason list where order stability is irrelevant after sorting. That now uses `sort_unstable(); dedup();`, matching the existing app workflow code and avoiding dependency churn.

`itertools::Itertools::unique()` would express the known instance compactly, but it would also preserve first-seen order instead of the deterministic sorted order this API response currently returns. `unique().sorted()` or `sorted().dedup()` would add a new trait dependency for a two-line standard-library idiom already used consistently nearby.

## Concrete inventory

### Iterator uniqueness and sorting

Observed production instances:

- `apps/api/src/http.rs:1874` collected data-quality hygiene rejection reasons, then sorted and deduplicated them.
- `app/src/booking_triage.rs` normalizes approval gates, blocked actions, and audit event drafts with `sort_unstable(); dedup();`.
- `app/src/checkout_completion.rs` normalizes blocked actions and audit event drafts with `sort_unstable(); dedup();`.
- `app/src/data_quality_hygiene.rs` deduplicates draft rejection reasons without sorting; that preserves validator emission order and should not be silently changed.

Decision: keep standard-library `Vec` normalization. The repeated production pattern is small, explicit, and uses domain enum ordering. `itertools` is not justified unless more iterator pipelines start needing multi-step grouping, cartesian products, tuple windows, or ordered uniqueness in several places.

Change made: `apps/api/src/http.rs` now uses `sort_unstable()` before `dedup()` because the rejection reasons are strings emitted from independent validation checks and the result is intentionally canonicalized, not stable-sort-sensitive.

Future trigger for `itertools`: at least three production call sites where the standard-library iterator version is materially harder to audit than the `itertools` equivalent, with tests proving the intended order semantics.

### UTF-8 paths (`camino`)

Observed path surfaces:

- `domain/tests/service_module_architecture.rs` walks workspace test directories using `std::path::Path`, `read_dir`, and `join`.
- `integrations/gingr/src/config.rs` uses URL joining, not filesystem paths.
- Other `.join(" ")` / `.join("&")` hits are string joins, not path joins.
- `integrations/gingr/src/endpoint/mod.rs` passes provider URL path strings into a semantic `endpoint::Path`; this is not a filesystem path.

Decision: do not add `camino`. The only real filesystem path handling is a test scanner that interoperates directly with `std::fs`; converting it to `Utf8Path` would not remove enough `.to_str()` friction or protect a production invariant.

Future trigger for `camino`: production code starts passing repository-relative paths, cache paths, or artifact paths across module boundaries as validated UTF-8 paths, especially if repeated `to_str()` / `to_string_lossy()` conversions appear.

### Small-string storage (`smol_str` / `compact_str`)

Observed string-heavy surfaces:

- App/API DTOs and storage records intentionally use owned `String` because they are serde/http/database boundary payloads.
- Gingr endpoint parameters use `Vec<(String, String)>` because provider query/form field names are visible wire-contract evidence and values are formatted on demand.
- Storage/domain error variants carry `String` reason/value fields so diagnostics preserve raw external or persisted text.
- Some short codes are already better represented semantically through enums/newtypes (`strum` display/parse, `nutype`, provider/domain id wrappers) rather than by swapping the storage representation of raw strings.

Decision: do not add `smol_str` or `compact_str` now. The audit found many strings, but they are mostly boundary-owned payloads, diagnostics, or provider parameters; changing their storage type would churn serde and conversion code without a measured allocation hot spot.

Future trigger: repeated high-volume in-memory keys or codes that are cloned/stored in domain collections after they have already been semantically validated, plus either profiling evidence or a clear memory-pressure rationale. Even then, prefer domain newtypes first; use a compact string only as the representation hidden behind the semantic type.

### Enum-indexed maps (`enum-map`)

Observed enum surfaces:

- The codebase already leans on semantic enums and `strum::VariantArray` for closed sets and code lists.
- Current enum-to-value behavior is mostly `match`-based labels, priorities, and policy decisions where each arm documents domain meaning.
- No repeated dense `Enum -> value` mutable table or parallel arrays were found in production code.

Decision: do not add `enum-map`. The current matches are readable domain policy declarations. Replacing them with tables would risk hiding meaning unless the table itself becomes a true domain artifact.

Future trigger: a closed enum needs a dense, exhaustive, frequently indexed table in production code, and the table is data rather than policy logic. Add tests that iterate every variant and prove the table covers the domain vocabulary.

### Adjacent crates considered but not adopted

- `indexmap`: no observed need to preserve insertion order in maps beyond ordinary JSON/provider unknown-field preservation, where `BTreeMap` / `serde_json::Map` are already adequate.
- `smallvec`: no profiled tiny-vector hot path. Most `Vec` fields are boundary collections or semantic workflow lists where clarity matters more than stack optimization.
- `arrayvec`: no fixed-capacity invariant surfaced that is not already better expressed by a semantic type or builder.

## Adoption rule

Add one of these crates only when all of these are true:

1. At least three concrete production call sites repeat the same ergonomic or performance problem, or one hot path has profiling evidence.
2. The crate improves the semantic call site instead of hiding domain meaning behind a clever helper.
3. The dependency can be scoped to the crate that owns the repeated problem; do not add workspace dependencies by default.
4. Focused tests prove the behavior that the crate is meant to protect: ordering, deduplication, path validity, compact string conversion, or enum coverage.

Until then, prefer standard-library code plus semantic domain types. The codebase benefits more from named invariants than from broad dependency-based terseness.
