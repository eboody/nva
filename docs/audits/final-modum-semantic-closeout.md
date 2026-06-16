# Final Modum semantic closeout

Status: fan-in worktree `pet-resort-modum-final-lints/99-fan-in` on branch `kb/final-modum-fan-in-20260616`.

This fan-in integrates the four accepted final cleanup branches and keeps one canonical semantic-module surface for each touched concept. The integration preserved behavior and wire/provider contracts, then applied a small final caller-surface cleanup for overqualified reservation/entity call sites.

## Integrated branches

| Parent task | Branch | Commit | Integrated scope |
| --- | --- | --- | --- |
| `t_140d6529` | `kb/final-modum-entities-20260616` | `4f4a40d` | Refactored broad domain entity families into semantic child modules; parent handoff reported `domain/src/entities.rs` Modum diagnostics `4 -> 0`. |
| `t_6efdf27b` | `kb/final-modum-id-surfaces-20260616` | `ae5df78` | Canonicalized remaining Id surfaces; parent handoff reported targeted `api_redundant_leaf_context` and `namespace_flat_pub_use` cleared. |
| `t_692f6108` | `kb/final-modum-unsupported-closeout-20260616` | `a90c65a` | Documented accepted unsupported/internal diagnostics and reconciled the overlapping `training::package::Id` surface. |
| `t_744504a8` | `kb/final-modum-gingr-endpoints-20260616` | `817072b` | Refactored Gingr endpoint request modules while preserving provider endpoint vocabulary and read/write contracts. |

## Fan-in reconciliation

- The `training::package::Id` overlap resolved to the Id-surface branch's canonical `training::package::Id` shape while retaining the unsupported/internal closeout documentation.
- Gingr endpoint request modules remain semantically owned by provider endpoint families; no verb-bucket compatibility shims were introduced.
- Final fan-in cleanup removed two overqualified call-site paths in application code by importing the meaningful `reservation` / `entities` namespaces.
- The four remaining `namespace_flat_pub_use` diagnostics are intentional parent-surface re-exports for nutype-backed IDs where Modum alternates between asking for parent-surface exports and warning that the re-export hides the child `Id` namespace.

## Modum before/after inventory

Baseline is commit `1a036f3` before the final four cleanup branches. Final is this fan-in after integration and final caller-surface cleanup.

| Run | Scanned files | Files with diagnostics | Diagnostics |
| --- | ---: | ---: | ---: |
| Baseline `1a036f3` | 95 | 21 | 30 |
| Final fan-in | 95 | 17 | 21 |

| Code | Baseline | Final | Closeout decision |
| --- | ---: | ---: | --- |
| `api_candidate_semantic_module` | 8 | 0 | Cleared by entity/Gingr semantic-module refactors. |
| `api_redundant_leaf_context` | 1 | 0 | Cleared by canonical Id-surface work. |
| `namespace_overqualified_callsite_path` | 0 | 0 | Cleared in fan-in after parent integrations exposed two new call-site opportunities. |
| `api_candidate_semantic_module_unsupported_construct` | 11 | 11 | Accepted Modum source-analysis limitation around macro-generated families; manually reviewed. |
| `namespace_flat_pub_use` | 5 | 4 | Remaining four are accepted parent-surface ID re-exports for application/provider IDs and daycare assignment playgroups. |
| `namespace_family_unsupported_construct` | 3 | 3 | Accepted source-analysis limitation for storage macro/conversion surfaces. |
| `internal_organizational_submodule_flatten` | 2 | 2 | Accepted internal `error.rs` organization; public parent surfaces remain `payment::Error` / `reservation::Error`. |
| `api_candidate_child_facet_module` | 0 | 1 | Accepted for now: `training::package::Id` is the canonical package ID surface after the Id cleanup; splitting an `id` child facet would reintroduce narrower namespace churn without a stronger domain owner. |

## Remaining accepted diagnostics

### Accepted source-analysis limitations

`api_candidate_semantic_module_unsupported_construct` remains on macro-heavy source scopes where Modum cannot infer semantic families without macro expansion:

- `domain/src/boarding/mod.rs`
- `domain/src/care.rs`
- `domain/src/daycare/front_desk.rs`
- `domain/src/daycare/mod.rs`
- `domain/src/grooming/mod.rs`
- `domain/src/training/mod.rs`
- `integrations/gingr/src/endpoint/mod.rs`
- `integrations/gingr/src/endpoint/reference_data.rs`
- `integrations/gingr/src/lib.rs`
- `storage/src/lib.rs`
- `storage/src/operations.rs`

`namespace_family_unsupported_construct` remains accepted for storage boundary macro/conversion surfaces:

- `storage/src/lib.rs`
- `storage/src/service_line/grooming.rs`
- `storage/src/service_line/training.rs`

### Accepted intentional organization

`internal_organizational_submodule_flatten` remains accepted for:

- `domain/src/payment/mod.rs`
- `domain/src/reservation/mod.rs`

Both modules keep private `error` submodules while re-exporting parent semantic surfaces, preserving the domain-error convention without changing caller-facing meaning.

### Accepted parent-surface tradeoffs

`namespace_flat_pub_use` remains accepted for:

- `app/src/tools.rs` payment authorization provider ID
- `app/src/tools.rs` payment refund provider ID
- `app/src/tools.rs` Hermes kanban task ID
- `domain/src/daycare/assignment.rs` playgroup ID

These are deliberate parent-surface re-exports for nutype-backed `Id` leaves. They satisfy the readable parent API requested by the missing-parent-surface diagnostics while preserving the underlying child modules for local validation ownership.

### Accepted future design pressure

`api_candidate_child_facet_module` remains on `domain/src/training/mod.rs` for `training::package::Id`. The final agreed surface is `training::package::Id`; introducing `training::package::id::Id` would make call sites more nested without clarifying package ledger/package usage behavior, so this remains accepted design pressure rather than fan-in cleanup.

## Verification

- `cargo fmt --check` passed.
- `cargo test --workspace` passed.
- `git diff --check HEAD` passed.
- `modum check --format json` completed with the 21 accepted diagnostics listed above; the command exits non-zero while diagnostics remain.
