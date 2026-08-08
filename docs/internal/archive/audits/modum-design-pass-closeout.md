# Modum design-pass fan-in closeout

Status: integrated in worktree `pet-resort-modum-design-pass/09-fan-in`.

Base before this pass: `ee0e039` (`kb/modum-namespace-fan-in-202606121645`).
Integrated branch: `kb/modum-design-fan-in-20260612`.
Parent design-pass commits integrated:

- `d8dbd4a` — app tool semantic surfaces
- `c21e690` — grooming semantic owner surfaces
- `494fc89` — training semantic owner surfaces
- `8d558ac` — Gingr provider-boundary surfaces

## Verification

Owner gates run after integration:

```bash
cargo fmt --check
cargo test --workspace
git diff --check HEAD
modum check --format json
```

Results:

- `cargo fmt --check`: pass
- `cargo test --workspace`: pass
- `git diff --check HEAD`: pass
- `modum check --format json`: report completed; exit status remains non-zero because 51 policy diagnostics remain

## Modum before / after delta

The before report was generated from detached base worktree `ee0e039`; the after report was generated from the integrated branch head.

| Metric | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Scanned files | 96 | 96 | 0 |
| Files with violations | 33 | 30 | -3 |
| Diagnostics | 92 | 51 | -41 |

Diagnostic-family delta:

| Code | Before | After | Delta | Classification |
| --- | ---: | ---: | ---: | --- |
| `api_redundant_leaf_context` | 35 | 1 | -34 | fixed except `temperament::TemperamentRating`, which is a true design follow-up |
| `api_candidate_semantic_module` | 26 | 20 | -6 | fixed where parent commits introduced honest semantic modules; remaining families are true design follow-ups or vendor-boundary caveats |
| `api_candidate_semantic_module_unsupported_construct` | 11 | 11 | 0 | Modum limitation: macro/source-analysis skip, manual review only |
| `api_candidate_child_facet_module` | 7 | 8 | +1 | true design follow-up where new semantic modules exposed smaller id/body/rationale facets; not an alias regression |
| `internal_organizational_submodule_flatten` | 3 | 3 | 0 | rejected for now where module-local `error` organization preserves local error ownership |
| `namespace_family_unsupported_construct` | 3 | 3 | 0 | Modum limitation: macro/source-analysis skip |
| `api_catch_all_module` | 1 | 1 | 0 | vendor/storage boundary follow-up; do not mechanically flatten `storage::service` |
| `api_missing_parent_surface_export` | 1 | 1 | 0 | true design follow-up: `daycare::assignment::PlaygroupId` parent surface |
| `api_repeated_parameter_cluster` | 1 | 1 | 0 | true design follow-up: `RuleEvaluation` constructors want a cohesive options/builder surface |
| `api_weak_module_generic_leaf` | 1 | 0 | -1 | fixed by Gingr provider-boundary pass |
| `api_boolean_flag_cluster` | 1 | 1 | 0 | true design follow-up: daily-care send/review mode should be typed |
| `api_raw_id_surface` | 1 | 0 | -1 | fixed at Gingr retail DTO boundary |
| `namespace_qualified_child_facet_follow_through` | 1 | 1 | 0 | true design follow-up, dependent on `daily_brief::snapshot_id` facet |

Net result: the fan-in removes 41 of 92 diagnostics while preserving the anti-alias policy. The largest win is eliminating almost all redundant leaf-context diagnostics in grooming, training, and Gingr webhook/reservation surfaces without adding compatibility aliases.

## Top remaining lints and classification

### Fixed by this pass

- Grooming repeated public leaves were moved behind owner modules such as `grooming::calendar`, `grooming::rebooking`, `grooming::reminder`, and `grooming::breed_coat`.
- Training repeated public leaves were moved behind owner modules such as `training::enrollment`, `training::curriculum`, `training::trainer`, and `training::program`.
- Gingr reservation/webhook repeated leaves were shortened inside provider-boundary modules, e.g. `webhook::SignatureKey`, `webhook::EventType`, `webhook::Verified`, and reservations owner/animal surfaces.
- Gingr `transport::Error` weak-module warning and retail raw-id warning are resolved at the provider boundary.
- App tools payment/auth/refund/message surfaces were split into semantic modules instead of re-exporting flat alias-like names.

### True design follow-up

These are legitimate design pressure but should be handled in focused follow-up commits, not in this fan-in:

- `app/src/booking_triage.rs`: `RuleEvaluation::{unknown, needs_human_approval, hard_block}` repeat a positional cluster. Prefer a typed input/options surface or builder that encodes rule evidence and approval semantics.
- `app/src/daily_update.rs`: `DailyCareUpdateOutput` still carries `should_send` / `requires_review` booleans. Prefer a semantic send/review mode enum or outcome surface.
- `domain/src/daycare/assignment.rs`: parent surface is missing `daycare::assignment::PlaygroupId`; add this only if the parent path is the canonical caller-facing assignment surface.
- `domain/src/temperament.rs`: `TemperamentRating` should likely become `temperament::Rating`, but this touches wider temperament call sites and deserves a small semantic rename pass.
- `domain/src/daily_brief.rs` / `domain/src/staff.rs`: `SnapshotId` and follow-through to `daily_brief::snapshot_id` are related and should be done together.
- `domain/src/policy.rs`, `domain/src/staff.rs`, `domain/src/workflow.rs`, and `domain/src/training/mod.rs`: child-facet lints (`rationale`, `completion_evidence`, `reason`, `package::id`) are plausible future facets once their owning modules are intentionally shaped.
- `domain/src/entities.rs` and `domain/src/operations.rs`: remaining broad `Reservation`, `CareNote`, `Approval`, `Audit`, `Operational`, `PetResort`, `Boarding`, `Core`, and `Task` families are larger domain-shaping work.

### Vendor-boundary / storage-boundary caveats

- `integrations/gingr/src/endpoint/commerce_retail.rs`: `Get*` and `List*` families are provider endpoint vocabulary. A mechanical `get::{...}` or `list::{...}` module would likely be less semantic; if changed, use provider-operation ownership, not verb buckets.
- `integrations/gingr/src/endpoint/owners_animals.rs`: `CustomField*` is provider vocabulary. A `custom_field` module may be acceptable, but only if it keeps Gingr's filter/search contract visible.
- `integrations/gingr/src/endpoint/reservations.rs`: `Reservation*` endpoint builders remain provider-boundary surfaces. Do not introduce alias-like parent re-exports just to silence Modum.
- `storage/src/lib.rs`: `service` remains a persistence boundary. Splitting it may be worthwhile later, but not as a flat re-export workaround.

### Modum limitation / manual-review only

- All 11 `api_candidate_semantic_module_unsupported_construct` diagnostics are source-analysis limitations caused by macros or `macro_rules!` in domain, Gingr endpoint, and storage scopes.
- All 3 `namespace_family_unsupported_construct` diagnostics are source-analysis limitations around macro-bearing imports/re-exports.

### Rejected because it would reintroduce alias smell

- Do not restore flat compatibility names for the grooming, training, Gingr webhook/reservation, or app-tools surfaces removed in this pass.
- Do not add broad parent re-exports simply because Modum asks for a shorter path. A parent surface is acceptable only when it is the canonical semantic API; otherwise the child module path is the truth.
- Do not create verb-bucket modules like `get` / `list` for Gingr endpoint surfaces unless the resulting path carries provider-operation meaning better than the current names.
- Do not flatten `error` modules mechanically. Local `error.rs` modules remain acceptable implementation organization when the parent exports the semantic `Error` / `Result` surface.

## Remaining diagnostic inventory

After this fan-in, Modum reports 51 diagnostics across 30 files. Largest remaining groups:

- 20 `api_candidate_semantic_module`
- 11 `api_candidate_semantic_module_unsupported_construct`
- 8 `api_candidate_child_facet_module`
- 3 `internal_organizational_submodule_flatten`
- 3 `namespace_family_unsupported_construct`
- 1 each of `api_catch_all_module`, `api_missing_parent_surface_export`, `api_redundant_leaf_context`, `api_repeated_parameter_cluster`, `api_boolean_flag_cluster`, and `namespace_qualified_child_facet_follow_through`

Top files by remaining count:

- `app/src/tools.rs`: 7
- `domain/src/entities.rs`: 4
- `domain/src/operations.rs`: 4
- `storage/src/lib.rs`: 3
- `domain/src/staff.rs`: 3
- `app/src/booking_triage.rs`: 2
- `app/src/daily_update.rs`: 2
- `domain/src/daily_brief.rs`: 2
- `domain/src/training/mod.rs`: 2
- `integrations/gingr/src/endpoint/commerce_retail.rs`: 2

## Design caveats for reviewer

This integration intentionally prefers canonical semantic paths over compatibility aliases. It keeps remaining lints visible when the safe fix is real domain design work, a provider/storage boundary decision, or a Modum limitation. The review question should be whether the accepted parent commits preserve semantic ownership and whether any remaining parent-surface re-export is truly canonical, not whether the Modum count reaches zero.
