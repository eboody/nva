# Builder modernization policy

Source inventory: [`docs/kanban/2026-06-19-rust-crate-modernization-audit.md`](../kanban/2026-06-19-rust-crate-modernization-audit.md).
Detailed classification: [`builder-modernization-classification.md`](builder-modernization-classification.md).

This policy is for pet-resort construction APIs that feed source evidence, provider requests, staff review queues, storage projections, and customer-facing draft paths. The goal is not to remove every manual builder. The goal is to make invalid operational states difficult to construct, keep provider/source ambiguity reviewable, and avoid hand-written boilerplate where a crate can preserve the same meaning.

## Decision rules

Use this order when adding or changing construction APIs. For the current `statum` audit and the documented "no new phase API yet" finding, see [`statum-phase-api-evaluation.md`](statum-phase-api-evaluation.md).

1. Use a named constructor or small `bon` builder for plain shape assembly.
   - Use `bon` when fields are ordinary optional or required inputs and the generated API keeps the call site readable.
   - Do not use `bon` to hide provider grammar, redaction rules, review boundaries, or source-provenance decisions.
   - A one-field value often wants `Type::new(value)` or a required `bon` field, not a hand-written builder.
2. Use a `Result`-returning constructor when runtime evidence can be incomplete.
   - Missing provider IDs, source provenance, relationship evidence, date windows, location scope, and sensitive lookup inputs should return a module-local typed error.
   - Production builders must not enforce required fields with `expect` or panic. Static fixtures may still use `expect` after they have already constructed valid typed values.
3. Use `nonempty` or a domain wrapper around `nonempty` when an empty collection would be a lie.
   - Evidence sets, outcome claims, provenance bundles, or reviewer-support lists that must contain at least one item should become `NonEmpty<T>` behind a named domain type such as `EvidenceSet` or `Claims`.
   - Keep collection access total where the invariant allows it, for example `first_evidence()` or `first_claim()`.
4. Use `statum` only for true state-machine or phase construction.
   - Reach for `statum` when phases or allowed transitions are the domain truth and should be compile-time-visible.
   - Do not add `statum` for a two-mode request if named constructors already make invalid states unrepresentable.
5. Keep a manual builder when the manual shape is semantic.
   - Manual is correct when the builder is an accumulator for provider `params[...]` grammar, redaction-sensitive request assembly, checked-in-versus-date-range mode selection, or source-normalization provenance.
   - Manual is not correct when it only stores `Option<T>` fields and calls `expect` in `build()`.

## Current resolution of the original 21 builder sites

| Original site | Resolution | Current policy |
| --- | --- | --- |
| `domain::source::reservation::SnapshotBuilder` | Kept manual, `build() -> source::Result<Snapshot>` | Source snapshots require provenance and owner/pet relationship evidence; missing evidence is a typed source error. |
| `domain::source::gingr::reservation::SnapshotBuilder` | Kept manual, `build() -> source::Result<Snapshot>` | Gingr-specific source snapshots remain fallible before promotion into source-agnostic evidence. |
| `domain::training::progress::ReportBuilder` | Kept manual with `EvidenceSet` backed by `nonempty` | Progress reports require IDs plus non-empty evidence and return training errors instead of panics. |
| `domain::training::outcome::DocumentationBuilder` | Kept manual with `Claims` backed by `nonempty` | Outcome documentation requires IDs plus at least one evidence-backed claim and defaults review state to draft. |
| `integrations::gingr::transport::RequestPartsBuilder` | Intentionally manual, still transport-local | Request method/path assembly also owns parameter redaction classification; keep this manual unless it becomes fallible with endpoint errors. |
| `gingr::endpoint::reference_data::GetVetsBuilder` | Replaced by `bon::Builder` on `GetVets` | Optional flag assembly with no hidden domain or provider invariant. |
| `gingr::endpoint::labor_ops::TimeclockReportBuilder` | Kept manual, `build() -> endpoint::Result<TimeclockReport>` | Date range and location remain runtime provider requirements with `MissingRequiredParameter` errors. |
| `gingr::endpoint::report_cards_files::ReportCardFilesBuilder` | Replaced by `bon::Builder` on `ReportCardFiles` | Optional query filters only. |
| `gingr::endpoint::commerce_retail::get::SubscriptionsBuilder` | Replaced by `bon::Builder` on `Subscriptions` | Optional filters with validation owned by field wrappers such as bill day and pagination. |
| `gingr::endpoint::commerce_retail::list::TransactionsBuilder` | Kept manual, `build() -> endpoint::Result<Transactions>` | Legacy transaction dates are required and must stay before the invoice cutover. |
| `gingr::endpoint::commerce_retail::list::InvoicesBuilder` | Kept manual, `build() -> endpoint::Result<Invoices>` | Invoice date filters are optional but must stay on or after the cutover when present. |
| `gingr::endpoint::owners_animals::OwnersBuilder` | Replaced by `bon::Builder` on `Owners` | Provider `params[...]` where-clause grammar remains quarantined in `ProviderWhereClause`; builder accepts an auditable clause collection rather than hand-coded setters. |
| `gingr::endpoint::owners_animals::AnimalsBuilder` | Replaced by `bon::Builder` on `Animals` | Same provider where-clause rule as owners, including provider expression keys. |
| `gingr::endpoint::owners_animals::custom_field::SearchBuilder` | Replaced by `bon::Builder` on `Search` | Form, field name, and lookup text are compile-time required builder fields; sensitive lookup redaction stays explicit. |
| `gingr::endpoint::reservations::reservation::TypesBuilder` | Replaced by `bon::Builder` on `Types` | Optional reservation-type filters only. |
| `gingr::endpoint::reservations::reservation::WidgetDataBuilder` | Replaced by `bon::Builder` on `WidgetData` | Single required timestamp with provider `Date` validation already owned by the field type. |
| `gingr::endpoint::reservations::reservation::SearchFiltersBuilder` | Kept manual | Repeated provider array parameters and status flags stay explicit; `bon` is allowed later only if array emission remains tested. |
| `gingr::endpoint::reservations::Builder` | Kept manual mode constructor | `Reservations::checked_in()` and `Reservations::for_range(range)` make the two request modes explicit without adding `statum` yet. |
| `gingr::endpoint::reservations::by::AnimalBuilder` | Kept manual, `build() -> endpoint::Result<Animal>` | Animal ID is required and missing values return provider-construction errors. |
| `gingr::endpoint::reservations::by::OwnerBuilder` | Kept manual, `build() -> endpoint::Result<Owner>` | Owner ID is required and the location-scope caveat stays attached to the type. |
| `gingr::endpoint::reservations::BackOfHouseBuilder` | Kept manual, `build() -> endpoint::Result<BackOfHouse>` | Location is required; repeated type IDs remain optional until provider semantics prove they are required. |

## Review checklist for future builder changes

Before accepting a new builder or migration, answer these questions in the code review:

- What invalid pet-resort or provider state is this constructor preventing?
- If a required value is missing at runtime, does the caller receive a typed module-local error rather than a panic?
- If a collection is semantically required, is emptiness impossible after construction via `nonempty` or a named wrapper?
- If the API is manual, what meaning would be lost by replacing it with `bon`?
- If `statum` is proposed, what phases or transitions become clearer than named constructors or an enum?
- Are provider IDs, provider search grammar, redaction-sensitive values, and source evidence still quarantined at the boundary?
- Do tests cover the chosen constructor style: missing-field errors, non-empty evidence, provider array parameter emission, redaction, and mode selection?

## Verification expectation

Builder-policy or builder-code changes should pass the same gate used by the modernization board:

```sh
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
./scripts/check_docs.sh
```

If `statum`, `bon`, or `nonempty` are added to a new crate, also verify the package manifest keeps the dependency workspace-scoped unless there is a documented reason not to.
