# Modum unsupported/internal lint closeout

Status: worktree `pet-resort-modum-final-lints/04-modum-unsupported-closeout`.

This pass intentionally separates low-risk source cleanup from Modum analysis limits. Macro-generated scalar/value families were not mechanically churned just to make source-only lint inference succeed.

## Fixed in this pass

- `domain::training::package::PackageId` became the direct semantic leaf `domain::training::package::Id`.
  - This removes the repeated `package::PackageId` surface and the flattened `package_id::Id as PackageId` re-export.
  - Callers now use `training::package::Id`, while package behavior remains under `training::package`.

## Accepted internal organization

- `domain/src/payment/mod.rs` keeps `mod error; pub use error::{Error, Result};`.
  - The parent public surface is still `payment::Error` / `payment::Result`.
  - Keeping `error.rs` preserves local error ownership and matches the domain-error convention; flattening two error variants into `mod.rs` would not improve caller meaning.
- `domain/src/reservation/mod.rs` keeps `mod error; pub use error::{Error, Result};` for the same reason.
  - The error variants are module-local construction failures for reservation values.
  - The visible API is already the semantic parent surface.

## Accepted macro/source-analysis limitations

The following `api_candidate_semantic_module_unsupported_construct` diagnostics remain accepted as Modum source-analysis limitations after manual inspection:

| File | Construct | Closeout decision |
| --- | --- | --- |
| `domain/src/boarding/mod.rs` | `positive_scalar!` macro-generated positive scalar/error families | Keep macro; repeated validation/error shape is intentional and local. |
| `domain/src/care.rs` | `redacted_debug!` impl macro plus `nutype` values | Keep macro; debug redaction is behavior, not a namespace-family design prompt. |
| `domain/src/daycare/front_desk.rs` | `positive_scalar!` invocation inherited from parent module | Keep macro; `QueuePosition` scalar is a local domain value. |
| `domain/src/daycare/mod.rs` | `positive_scalar!` macro-generated visit/count families | Keep macro; no obvious safer semantic module split. |
| `domain/src/grooming/mod.rs` | `positive_scalar!` macro-generated appointment/cadence counts | Keep macro; prior passes already moved real owner surfaces under semantic modules. |
| `domain/src/training/mod.rs` | `positive_scalar!` macro-generated session/count values | Keep macro; the one low-risk package id cleanup was applied separately. |
| `integrations/gingr/src/endpoint/mod.rs` | endpoint root with macro-bearing descendants | Keep provider endpoint surface; avoid verb-bucket churn. |
| `integrations/gingr/src/endpoint/reference_data.rs` | `simple_reference_endpoint!` provider endpoint declarations | Keep macro; provider endpoint names mirror Gingr API vocabulary. |
| `integrations/gingr/src/lib.rs` | public module root with macro-bearing endpoint module | Keep root surface; diagnostic is transitive source-analysis noise. |
| `storage/src/lib.rs` | public storage root with macro-bearing operations module | Keep storage root; persistence boundary owns codec/result exports. |
| `storage/src/operations.rs` | `bidirectional_code_map!` conversion macro | Keep macro; conversion mapping is storage/domain boundary plumbing, not an API family to split. |

The following `namespace_family_unsupported_construct` diagnostics remain accepted after manual inspection:

| File | Construct | Closeout decision |
| --- | --- | --- |
| `storage/src/lib.rs` | `pub use operations::{CodecError, RecordKind, Result};` with macro-bearing operations scope | Keep canonical storage root exports; the macro only blocks Modum's family inference. |
| `storage/src/service_line/grooming.rs` | `use crate::operations::{self, StorageField};` | Keep import; `StorageField` is the shared persistence-field trait used by storage service-line records. |
| `storage/src/service_line/training.rs` | `use crate::operations::{self, StorageField};` | Keep import for the same storage-boundary reason. |

## Remaining non-target/noise diagnostics worth future work

Current Modum still reports legitimate follow-up pressure outside this closeout's narrow scope:

- `namespace_flat_pub_use` remains in `app/src/tools.rs` for authorization/refund/task IDs and in `domain/src/daycare/assignment.rs` for `PlaygroupId`. These should be handled as a focused caller-surface pass, not mixed into macro unsupported closeout.
- `api_candidate_semantic_module` remains in broad domain entity families and several Gingr provider endpoint files. The Gingr items are provider-sensitive and should not become mechanical `get`/`list` verb modules unless the provider operation model improves.
- `internal_organizational_submodule_flatten` remains for payment/reservation by design as documented above.

## Latest Modum inventory from this pass

After the `training::package::Id` cleanup, `modum check --format json` reports 29 diagnostics across 21 files:

| Code | Count | Closeout status |
| --- | ---: | --- |
| `api_candidate_semantic_module_unsupported_construct` | 11 | Accepted Modum source-analysis limitation; manual inspection complete. |
| `api_candidate_semantic_module` | 8 | Future semantic/provider-boundary design work. |
| `namespace_flat_pub_use` | 4 | Future caller-surface pass. |
| `namespace_family_unsupported_construct` | 3 | Accepted Modum source-analysis limitation; manual inspection complete. |
| `internal_organizational_submodule_flatten` | 2 | Accepted internal `error.rs` organization. |
| `api_candidate_child_facet_module` | 1 | Future semantic facet follow-through. |
