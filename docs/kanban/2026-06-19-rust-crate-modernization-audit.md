# Rust Crate Modernization Audit

**Date:** 2026-06-19

**Strategic lens:** NVA Pet Resorts wants this repo to lower labor costs, not merely compile. Crate choices should reduce hand-written plumbing, make domain contracts harder to misuse, and keep automation/auditing surfaces explicit. Prefer high-signal crates where they remove boilerplate without hiding operational meaning.

## Board set

This audit feeds three staged Kanban boards:

1. `nva-builder-modernization` — migrate or justify every hand-written builder.
2. `nva-semantic-types-modernization` — use `nutype`, `derive_more`, `strum`, and related semantic-type crates where current source has repeated string/newtype/enum boilerplate.
3. `nva-rust-ergonomics-modernization` — evaluate higher-value crates for errors, collections, paths, serialization, secrets, iterators, and API ergonomics.

All boards are staged behind a blocked start gate so they do not race the shared checkout until explicitly unblocked or assigned to isolated worktrees.

## Current workspace crate posture

Already present at workspace level:

- `bon = "3.9"`
- `nutype = { version = "0.7", features = ["serde"] }`
- `statum = "*"` — deliberate project policy: latest-tracking contract dependency.
- `strum = { version = "0.27", features = ["derive"] }`
- `thiserror = "2.0"`
- `secrecy`, `url`, `serde_urlencoded`, `uuid`, `chrono`, `tracing`, `tower-http`, etc.

Not currently present but worth evaluating in focused lanes:

- `derive_more` — derive `Display`, `From`, `AsRef`, `Deref`, etc. for transparent value wrappers and code enums when it stays semantically clear.
- `snafu` — richer context/error attachments for app/integration boundaries where source operation context matters more than terse enum errors.
- `serde_with` — consistent serde transforms for stringly IDs, URL/query/date edge cases, default-as-empty, skip-empty, and transparent wrappers.
- `nonempty` — enforce evidence/provenance collections that must never be empty.
- `itertools` — replace bespoke grouping/partition/sorted/dedup plumbing when it improves readability.
- `camino` — UTF-8 paths for CLI/config/artifact surfaces, if path APIs become first-class.
- `smol_str` or `compact_str` — reduce allocation cost for many small labels/codes only if profiling or data volume justifies it.
- `enum-map` / `strum` — enumerate compact domain option sets without bespoke arrays/matches.
- `typed-index-collections` / `slotmap` — only if graph/index-heavy domain state appears; not currently a blanket recommendation.

## 1. Manual builder audit

Found 21 hand-written builder structs:

```text
domain/src/source.rs:591 SnapshotBuilder
domain/src/source.rs:1082 gingr::SnapshotBuilder
domain/src/training/mod.rs:758 ReportBuilder
domain/src/training/mod.rs:927 DocumentationBuilder
integrations/gingr/src/transport.rs:104 RequestPartsBuilder
integrations/gingr/src/endpoint/reference_data.rs:62 GetVetsBuilder
integrations/gingr/src/endpoint/labor_ops.rs:50 TimeclockReportBuilder
integrations/gingr/src/endpoint/report_cards_files.rs:30 ReportCardFilesBuilder
integrations/gingr/src/endpoint/commerce_retail.rs:155 SubscriptionsBuilder
integrations/gingr/src/endpoint/commerce_retail.rs:259 TransactionsBuilder
integrations/gingr/src/endpoint/commerce_retail.rs:362 InvoicesBuilder
integrations/gingr/src/endpoint/owners_animals.rs:39 OwnersBuilder
integrations/gingr/src/endpoint/owners_animals.rs:90 AnimalsBuilder
integrations/gingr/src/endpoint/owners_animals.rs:315 SearchBuilder
integrations/gingr/src/endpoint/reservations.rs:40 TypesBuilder
integrations/gingr/src/endpoint/reservations.rs:103 WidgetDataBuilder
integrations/gingr/src/endpoint/reservations.rs:187 SearchFiltersBuilder
integrations/gingr/src/endpoint/reservations.rs:277 Builder
integrations/gingr/src/endpoint/reservations.rs:374 AnimalBuilder
integrations/gingr/src/endpoint/reservations.rs:450 OwnerBuilder
integrations/gingr/src/endpoint/reservations.rs:545 BackOfHouseBuilder
```

Recommended policy:

- Use `bon` for ordinary optional/required construction.
- Use `statum` or a domain-specific typed constructor for true phased/protocol flows.
- Use `build() -> Result<T, Error>` or `nonempty` when missing evidence/provenance is a domain error.
- Avoid panic-based required-field enforcement such as `expect("... requires ...")` in production builders.
- Keep hand-written builders only when they enforce Gingr endpoint modes, redaction-sensitive lookup constraints, or NVA provenance rules that `bon` cannot express cleanly.

## 2. Semantic type / enum / conversion modernization

Observed high-value candidates:

- 143 transparent `String` tuple newtypes, including:
  - `domain/src/customer.rs`: `Name`, `Email`, `Phone`
  - `domain/src/care.rs`: `FeedingInstruction`, `AllergyName`, `MedicationName`, etc.
  - `domain/src/source.rs`: endpoint/batch/scope/schema/payload/status identifiers
  - `domain/src/training/mod.rs`: report/session/evidence/documentation IDs and notes
  - `app/src/data_quality_hygiene.rs`: `IssueRef`, `ActionId`, `ContextPacketId`, `CorrelationId`, `ActionRationale`
  - `integrations/gingr/src/webhook.rs`: `EntityId`
- 28 manual `as_str`/string-code methods, including:
  - `domain/src/analytics.rs`
  - `domain/src/source.rs`
  - `domain/src/retail/product.rs`
  - `storage/src/operations.rs`
  - `integrations/gingr/src/config.rs`
  - `integrations/gingr/src/response.rs`
  - `integrations/gingr/src/webhook.rs`
  - `app/src/data_quality_hygiene.rs`
- 45 manual `From` impls and 14 manual `TryFrom` impls, concentrated in storage/domain mapping code:
  - `storage/src/operations.rs`
  - `storage/src/service_line/{boarding,daycare,grooming,retail,training}.rs`
  - `domain/src/source.rs`

Recommended policy:

- Use `nutype` for validated string value objects: non-empty, bounded length, normalized email/phone-ish inputs where appropriate.
- Use `derive_more` for transparent wrappers only when the derived behavior is semantically safe and discoverable.
- Use `strum` for enum string codes, labels, `EnumIter`, `VariantNames`, and parsing when the code table is stable.
- Keep explicit `TryFrom` where conversion is lossy, fallible, or documents a provider/storage contract.
- Add tests around each migration so labor-cost domain contracts do not silently change.

## 3. Error/context and ergonomic crate opportunities

Observed current posture:

- Workspace uses `thiserror`, not `snafu`.
- App/integration errors often need operational context: source endpoint, provider record, redaction path, evidence batch, resort/workflow surface.
- Some places use manual sort+dedup style plumbing, e.g. `apps/api/src/http.rs:1874`.
- API/storage serde edges likely carry custom transform pressure as DTO/projection surfaces grow.

Recommended evaluation lanes:

- `snafu`: evaluate for integration/app boundary errors that need context selectors and source chaining; do not churn simple domain errors just to swap crates.
- `serde_with`: inventory serde adapter needs before adding custom ad-hoc serializers.
- `nonempty`: use for evidence/provenance lists where an empty list is invalid by construction.
- `itertools`: adopt where it removes bespoke grouping/sorted/partitioning code without obscuring ownership or performance.
- `camino`: consider for CLI/runbook/artifact path surfaces if UTF-8 path assumptions already exist.
- `smol_str`/`compact_str`: consider only after identifying hot/small label/code fields in provider payloads or large DTO batches.
- `enum-map`: consider for dense enum-indexed metrics/score tables once domain enums stabilize.

## Verification baseline for all boards

Workers should run, at minimum:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
./scripts/check_docs.sh
```

If `cargo doc` still hits known `statum` macro-generated `missing_docs` artifacts, workers must document the exact failure and either apply the established workaround or preserve the caveat rather than weakening public Rustdoc quality.
