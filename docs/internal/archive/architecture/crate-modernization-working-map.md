# Crate modernization working map

**Date:** 2026-06-19

This is the pass-01 working map for crate modernization. It is intentionally decision-focused: a crate change is useful only when it lowers pet-resort labor cost by making source evidence, workflow packets, review gates, outcome proof, Rustdocs, or repeated implementation patterns clearer, safer, or cheaper to maintain.

## Current dependency posture

Observed workspace dependencies:

- Present at workspace level: `bon`, `derive_more`, `nutype`, `nonempty`, `statum`, `strum`, `thiserror`, Serde/Chrono/UUID/support crates.
- Used in source today:
  - `bon`: broad domain/app/storage/Gingr construction APIs.
  - `nutype`: validated string value objects across domain/app surfaces.
  - `nonempty`: training progress/outcome evidence/claim builders where empty evidence would be false proof.
  - `statum`: `app::booking_triage` request typestate for intake -> pet profile -> policy evidence -> staff-ready review.
  - `derive_more`: narrow provider/storage display/conversion surfaces.
- Not present in manifests or production source today: `serde_with`, `snafu`, `itertools`, `camino`, `smol_str`, `compact_str`, `enum-map`, `typed-index-collections`, `slotmap`, `reqwest`, and `serde_urlencoded`.
- Dependency drift cleaned across the modernization passes: removed `statum.workspace = true` from `domain/Cargo.toml`; no `domain` source or tests use `statum`, and the intentional latest-tracking `statum` dependency remains scoped to `app` where booking triage uses it. The final integration sweep also removed stale package-level `async-trait`, `reqwest`, `serde_urlencoded`, `serde`, and `serde_json` entries from crates that no longer import them, plus unused workspace `reqwest` / `serde_urlencoded` entries.

## Working decisions and successor-pass targets

| # | Surface | Crate posture | Labor-cost proof-chain reason | Next action |
| ---: | --- | --- | --- | --- |
| 1 | `app::booking_triage` typestate request | Keep `statum` in `app`; do not move to domain or workspace consumers without a real state-machine API. | Booking triage staff packets must attach pet profile and policy evidence before deterministic review; compile-time phases reduce review rework and prevent agent drafts from outrunning source evidence. | Preserve latest-tracking policy; any future `statum` use must name the operational phases and legal transitions in Rustdoc/tests. |
| 2 | `domain/Cargo.toml` | Removed unused `statum` dependency. | Avoids teaching successor agents that domain core has a state-machine dependency when no domain proof chain uses it; keeps crate choices auditable. | No further action unless domain gains a real phase/transition API. |
| 3 | Existing `bon` builder coverage in domain/app/storage/Gingr | Keep `bon` where construction is ordinary field assembly and semantic meaning is preserved. | Reduces hand-written builder boilerplate for packets, records, service offerings, and provider filters without hiding review-gate or source-evidence decisions. | Successor code passes should compare new manual builders against `docs/architecture/builder-modernization-policy.md` before adding boilerplate. |
| 4 | `domain::source::{SnapshotBuilder, gingr::SnapshotBuilder}` | Keep manual `build() -> source::Result<_>`. | Source snapshots require provenance and owner/pet relationship evidence; typed failure is better than generated builder shape because source ambiguity must remain reviewable. | Do not migrate to `bon` unless missing-evidence behavior and source errors stay explicit. |
| 5 | Gingr endpoint builders with provider `params[...]` arrays/redaction | Keep manual where provider grammar, date modes, repeated arrays, or redaction classification are semantic; use `bon` only where the builder is ordinary optional/required parameter assembly with runtime `Result` validation preserved. | Provider request shape is source evidence: if parameter names or sensitive lookup redaction become opaque, staff/debuggers spend labor rediscovering why a source call was made. Pass 02 migrated `reservations_by_animal` and `reservations_by_owner` from hand-written state holders to `bon` method builders while preserving `Foo::builder().field(...).build() -> Result<_>` and missing-parameter errors. | Keep `BackOfHouse`/range/manual builders where repeated provider params or date/form grammar matter; any future `bon` migration must pass exact endpoint contract/redaction tests. |
| 6 | `domain::training::progress::ReportBuilder` and `training::outcome::DocumentationBuilder` | Keep manual builder plus `nonempty` evidence/claims. | Training progress/outcome proof with zero evidence is a false labor-savings or care-quality claim. Non-empty wrappers make proof total and avoid review-time archaeology. | Reuse the pattern for future evidence/claim packets; do not add `nonempty` for ordinary optional lists. |
| 7 | `app::data_quality_hygiene` IDs/rationale strings | High-value `nutype` or validated-Deserialize pass. | Data-quality hygiene is the next labor loop: blank issue refs/action ids/context ids would produce cleanup tasks that cannot be traced to source ambiguity or outcome proof. | Migrate five simple manual string wrappers or hand-write `Deserialize` through `try_new`; test blank serde rejection and packet fixtures. |
| 8 | `domain::source` provenance/provider wrappers | Selective validated deserialization, not blanket macro churn. | Source refs, batch ids, scopes, schema versions, payload hashes, raw refs, and observed/provider statuses are durable evidence receipts; invalid persisted values waste reviewer/source-owner time. | Highest priority is constructor-bypass serde risk; migrate only simple trim/non-empty wrappers after preserving `source::Error` semantics and provenance tests. |
| 9 | Sensitive care/customer/message/rationale text | Keep validated `nutype` where present, but avoid `Display`/leaky `Debug`. | These values can contain medical, care, behavior, customer, or staff-sensitive facts; accidental display/debug leaks create review and trust risk, not labor savings. | Add or preserve redacted-debug tests when touching care/message/rationale/body values; no broad `derive_more::Display`. |
| 10 | Storage service-line transparent mirrors | Keep narrow `derive_more` for transparent, non-sensitive conversions already audited. | Storage projections measure reviewed outcomes and service-line proof; mechanical conversion is useful only where it cannot bypass domain validation or approval gates. | Do not derive `From`/`Into` for promotion across trust boundaries; keep fallible conversions where storage rows become domain facts. |
| 11 | Gingr provider numeric/date/filter wrappers | Narrow `derive_more::Display` remains justified for non-sensitive wire parameters. | Exact provider parameter display helps fixture-safe request proof and reduces ad hoc API inspection labor. | Keep display snapshot/parameter tests; no display for sensitive lookup text or raw payloads. |
| 12 | `serde_with` | Not justified today; see [`serde-with-crate-evaluation.md`](serde-with-crate-evaluation.md). | Current Serde attributes (`default`, `alias`, `flatten`, `transparent`) are locally readable boundary contracts; provider query grammar and storage validation should stay explicit. Pass 06 added exact Gingr DTO wire-shape assertions instead of adding a dependency. | Re-open only after at least three production DTO fields repeat the same mechanical non-default transform, such as provider date strings or string-or-number ids. |
| 13 | `snafu` | Not justified as a global dependency; future boundary pilot only. | Domain and simple app errors are already semantic `thiserror` values; swapping crates would not reduce front-desk/manager labor. Boundary context could help later for HTTP transport, webhook ingest, or storage artifact decode failures. | Pilot only when a real source chain needs endpoint/record/evidence/workflow context at failure sites; keep comparable inner errors. |
| 14 | `itertools` | Not justified today. | Existing sort/dedup patterns are short, deterministic, and domain-order aware; `unique()` would obscure order semantics in review queues. | Add only after three production iterator/grouping sites are materially clearer with `itertools` and tests lock ordering. |
| 15 | `camino` | Not justified today. | Path handling is not a production pet-resort workflow concept yet; most path-like strings are provider URL paths or test scanners. | Consider only for production artifact/cache/CLI path contracts that repeatedly assume UTF-8 and cross module boundaries. |
| 16 | `smol_str` / `compact_str` | Not justified today. | The many strings are mostly serde/http/storage/provider boundary payloads; changing representation would churn contracts without measured labor or memory benefit. | Prefer semantic newtypes first; hide compact representation only behind a validated domain type after profiling or high-volume clone pressure. |
| 17 | `enum-map` | Not justified today. | Current enum `match` arms document domain policy/review meaning; tables would hide why a queue, gate, or outcome gets a label/priority. | Add only for dense exhaustive enum-indexed data tables, with tests iterating every variant. |
| 18 | `typed-index-collections` / `slotmap` | Not justified today. | No graph/index-heavy production state exists; entity relationships are documentation/proof chains and domain ids, not arena handles. | Re-open only if relationship graph algorithms or stable arena identities become runtime workflow state. |

## Dependency-drift rule for successor passes

Before adding or retaining a crate in a package manifest, run a source-use check for that package and answer:

1. Which pet-resort entity/workflow/review/outcome proof chain does this dependency make safer or cheaper?
2. Is the dependency scoped to the crate that owns the repeated problem?
3. Does it preserve semantic module paths and trust boundaries?
4. What focused tests prove the intended behavior (validated serde, redaction, non-empty proof, transition legality, parameter emission, ordering, or conversion safety)?

If those answers are not concrete, document “not justified” here or in the focused architecture page instead of adding the crate.
