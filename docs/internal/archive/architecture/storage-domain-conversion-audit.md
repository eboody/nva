# Storage/domain conversion boilerplate audit

## Scope

This pass audited the storage/domain conversion surface that was reported as 45 manual `From` impls and 14 manual `TryFrom` impls before the cleanup. After replacing the five transparent service-line contract wrapper pairs with `derive_more`, the remaining manual conversion surface is 35 `From` impls and 14 `TryFrom` impls.

Current remaining manual conversion count by file:

| File | `From` | `TryFrom` | Decision |
| --- | ---: | ---: | --- |
| `storage/src/operations.rs` | 13 | 8 | Keep explicit: these conversions define persisted record shapes, discriminated service-offering shape checks, storage error mapping, portfolio/brand value validation, and stable code mappings. |
| `storage/src/service_line/boarding.rs` | 6 | 0 | Keep explicit: each match is a durable storage code to domain enum contract for boarding accommodations, care features, and add-ons. |
| `storage/src/service_line/daycare.rs` | 4 | 0 | Keep explicit: daycare format and eligibility rule codes are serialized storage vocabulary, not transparent wrappers. |
| `storage/src/service_line/grooming.rs` | 2 | 2 | Keep explicit: service-code matches are durable storage vocabulary; cadence `TryFrom` preserves positive-week validation and `StorageField::GroomingCadenceWeeks` error context. |
| `storage/src/service_line/training.rs` | 0 | 4 | Keep explicit: duration and stay-and-study program conversions preserve fallibility and `StorageField::TrainingProgramDurationWeeks` context. |
| `storage/src/service_line/retail.rs` | 4 | 0 | Keep explicit: retail partner/category codes are serialized merchandising vocabulary. |
| `domain/src/source.rs` | 6 | 0 | Keep explicit: provider-to-normalized provenance promotion crosses a trust boundary and documents the exact source contract being accepted. |

## Patch applied

The only conversions that were obviously transparent and infallible were the five service-line `ContractRecord` tuple wrappers:

- `boarding::ContractRecord(domain::boarding::Contract)`
- `daycare::ContractRecord(domain::daycare::Contract)`
- `grooming::ContractRecord(domain::grooming::Contract)`
- `training::ContractRecord(domain::training::Contract)`
- `retail::ContractRecord(domain::retail::Contract)`

Those now derive `derive_more::From` and `derive_more::Into`. They are `#[serde(transparent)]` mirrors of the domain contract values, and the conversion performs no validation, storage-code mapping, authorization, normalization, or error-context translation. This is the narrow case where `derive_more` reduces boilerplate without hiding a storage contract.

The workspace dependency was added with only the needed conversion features for this storage crate use: `from` and `into` (the existing display feature is used by neighboring storage-code contract work).

## Why the rest stays explicit

The remaining manual conversions are intentionally not derive targets:

1. `TryFrom` impls document fallible storage boundaries. They translate persisted rows into domain builders, validate positive numeric wrappers, reject impossible flattened service-offering shapes, and preserve module-local `storage::operations::Error` context.
2. Stable storage-code enum mappings are persistence contracts. A match arm per variant is reviewable when a new storage code or domain variant is added; hiding those mappings behind generic derive behavior would make serialized compatibility less visible.
3. Provider/source provenance promotion in `domain/src/source.rs` crosses from provider-native Gingr evidence into normalized source evidence. The explicit `expect("... already validated")` messages document which constructor established the invariant before promotion.
4. Service-offering mappings intentionally spell out discriminator and cross-variant field rules. The storage row is flattened while the domain value is a semantic enum; the conversion is structural, not transparent.

## Verification expectation

The code change should be verified with:

```bash
cargo test -p storage --all-targets
cargo test -p domain --all-targets
cargo clippy -p storage --all-targets -- -D warnings
```
