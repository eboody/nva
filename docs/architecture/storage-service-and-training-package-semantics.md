# Storage service-line and training package ID semantics

## Decision

`storage::service` is an honest persistence boundary for service-line storage records, not a catch-all module to flatten away.

The module is technical, so it should not expose context-free leaves such as `storage::service::Repository` or `storage::service::Record`. Its current shape keeps the service-line namespace visible (`storage::service::{boarding, daycare, grooming, retail, training}`) and each child module owns only boundary records, stable storage codes, and promotion/demotion conversions for the corresponding domain service line. That is a truthful storage facet parallel to the domain service-line modules.

The follow-up pressure is not to rename `storage::service`; it is to keep splitting each service-line child by real persisted facets only when the child becomes too broad. Examples:

- `storage::service::training::program::Record` if training program storage grows beyond the current `ProgramRecord` and duration value.
- `storage::service::grooming::cadence::Weeks` if cadence storage gains a richer error/conversion surface.
- no `storage::service::common`, `types`, or generic record module unless a real cross-service persistence concept appears.

## Training package identity

The canonical caller-facing package identifier is now `training::package::Id`.

Rationale:

- `package` is the semantic owner visible at the call site.
- `Id` is a good short leaf once `training::package` supplies the context.
- `training::package::id::Id` repeated generic identity and made the internal child facet part of the public API.
- A separate flat `PackageId` would erase the stronger package namespace and create a parallel public surface.

The package ledger and usage APIs accept and return `training::package::Id` directly. No public or private `id` module is kept around the type; if package identity later gains enough behavior to deserve its own child facet, that should be a deliberate design change rather than a linter appeasement.

## Modum classification

- `training::package::id::Id` contradiction: good lint / resolved by choosing `training::package::Id` as the single canonical public surface.
- `storage::service` catch-all pressure: directionally right prompt, but the better solution is documentation plus service-line facet discipline rather than a mechanical rename. The current boundary is acceptable while it contains only service-line storage records/codecs/conversions.
- Remaining broad service-line modules: manual-review/deferred; split only when a concrete persisted facet has enough behavior or failure surface to own a child module.

## Follow-up recommendation

When the next storage change adds more than one persisted concept to a service-line child, split that child around the persisted facet before adding new flat leaves. For training package work, migrate any new docs/tests/examples to `training::package::Id`; do not re-open `training::package::id` publicly unless the identifier gains a substantial dedicated API that callers genuinely need to name as an `id` facet.
