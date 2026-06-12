# Semantic-module follow-up audit

Workspace: `/home/eran/code/pet-resort-agent-foundation-worktrees/pet-resort-semantic-modules-pass/00-audit`
Branch: `kb/semantic-modules-audit-20260612144914`
Command rerun: `modum check --format json`
Scope: audit only; no Rust remediation applied.

## Executive summary

Modum still reports **227 diagnostics** across **47 files**. The remaining pressure is not one mechanical rename pass. It clusters into five design lanes:

1. Domain service-line parent surfaces (`grooming`, `training`, `retail`, `operations`, `staff`, `daily_brief`, `lead`, `reputation`) still expose both flat prefixed names and nested semantic modules.
2. App tool-port contracts in `app/src/tools.rs` are still broad flat families under a single `tools` module, especially availability, payments, messaging, document intake/OCR, task draft, and schedule draft.
3. Storage and integration boundaries need an explicit boundary/public-surface decision rather than blindly shortening provider or persistence names.
4. Test import shape is mostly good: tests tend to import semantic modules and call through them, so production/public re-export shape is the real issue.
5. Several diagnostics remain accepted/deferred linter limitations: macros, `thiserror::Error` imports in local error modules, Gingr transport error shape, and the contradictory `training::package::id::Id` pressure already called out by prior work.

## Current diagnostic shape

| lint code | count | audit reading |
|---|---:|---|
| `api_redundant_leaf_context` | 67 | Mostly good lint: prefixed leaves remain after semantic child modules exist. |
| `namespace_flat_pub_use_redundant_leaf_context` | 47 | Mostly good lint: parent `pub use` aliases are keeping duplicate canonical surfaces alive. |
| `api_candidate_semantic_module` | 42 | Directionally right: these are design lanes, not automatic module extraction instructions. |
| `namespace_flat_pub_use` | 14 | Good lint when a parent prelude/root API hides meaningful domain namespaces. |
| `namespace_flat_use` | 13 | Mixed: useful production import-shape cleanup, but lower priority than public surfaces. |
| `api_candidate_semantic_module_unsupported_construct` | 11 | Linter-noise/manual-review where macros hide families from source analysis. |
| `namespace_redundant_qualified_generic` | 6 | Mostly tied to the accepted/deferred `training::package::id::Id` contradiction. |
| other single-digit diagnostics | 27 | Mostly local design prompts; classified below by lane. |

Top files by pressure: `domain/src/operations.rs` (42), `domain/src/retail/mod.rs` (33), `domain/src/training/mod.rs` (16), `domain/src/grooming/mod.rs` (14), `app/src/tools.rs` (12), `domain/src/staff.rs` (11), `storage/src/operations.rs` (10), `integrations/gingr/src/webhook.rs` (9), `domain/src/daily_brief.rs` (7), and `integrations/gingr/src/endpoint/reservations.rs` (7).

## Implement-now lanes

### 1. Remove duplicate flat public aliases in service-line modules

Files: `domain/src/retail/mod.rs`, `domain/src/grooming/mod.rs`, `domain/src/training/mod.rs`, `domain/src/staff.rs`, `domain/src/daily_brief.rs`, `domain/src/lead.rs`, `domain/src/reputation.rs`.

Classification: good lint / implementation lane.

The service-line modules already contain meaningful child facets but keep parent aliases such as:

- `retail::ProductCategory`, `retail::RecommendationRule`, `retail::ReorderPolicy` while `retail::product::Category`, `retail::recommendation::Rule`, and `retail::reorder::Policy` exist.
- `grooming::RebookingPolicy`, `grooming::ReminderPlan`, `grooming::CalendarPolicy` while `grooming::rebooking::*`, `grooming::reminder::*`, and `grooming::calendar::*` are the semantic facets.
- `training::EnrollmentId`, `training::CurriculumUnit`, `training::TrainerRequirement` while `training::enrollment::*`, `training::curriculum::*`, and `training::trainer::*` carry the domain context.
- `staff::StaffTask*` values that should be either `staff::Task*` or, better, a `staff::task::{Kind, Status, Priority, Assignment, Source}` facet.

Recommended next card: choose each module's canonical parent/child public surface and delete or narrow the flat parent aliases in one service-line pass. This is low-design-risk because the nested facets already exist and tests can migrate call sites to module-qualified paths.

### 2. Narrow `domain::operations` as a re-export/prelude bucket

Files: `domain/src/operations.rs`, plus callers that import operations-flattened values.

Classification: directionally right / implement after public-surface choice.

`operations` is now carrying multiple unrelated domain vocabularies by flat re-export: `daily_brief`, `lead`, `reputation`, and `staff`. The lints on lines 7, 14, 17, and 20 are not just import style; they reveal that `operations` has become a prelude-like umbrella. Prefer callers naming the true owner (`daily_brief::Section`, `lead::Intent`, `reputation::ReviewId`, `staff::Role`) over adding more `operations::*` aliases.

Recommended next card: make `operations` stop re-exporting unrelated sibling-domain surfaces, or explicitly quarantine those re-exports as compatibility-only if downstream consumers require them.

### 3. App tool-port family modules

Files: `app/src/tools.rs`, `app/src/lib.rs`.

Classification: good lint / implementation lane with moderate churn.

`app::tools` is a boundary crate, but its public API has obvious semantic families:

- `availability::{Request, Result, Decision, DenialReason, SuccessReason, ServiceNotes}`
- `portal::lookup::{Request, Result, Match, Criteria}` or `portal::Lookup` plus a child `lookup` facet
- `payments::{authorization, refund, deposit}` and possibly `payments::Gateway` / `payments::Subject`
- `messaging::{draft_message, message_body}` rather than mixed `Message*` and `DraftMessage*` siblings
- `documents::{intake, ocr}` or `document::{Intake, Source, Classification, Ref}` depending on whether the tool boundary owns the family
- `task::{DraftRequest, DraftResult, Id}` and `schedule::{DraftRequest, DraftResult, Cadence, Name, Id}`

`app/src/lib.rs` prelude currently re-exports flattened availability names; that should not become the canonical surface. Keep app callers through `tools::availability::*` or `tools::availability::Request` once the family is split.

Recommended next card: split `app::tools` into small semantic child modules and update the prelude to either disappear or re-export module namespaces, not flattened leaves.

### 4. Local production import-shape cleanup after public surfaces settle

Files: `domain/src/daily_brief.rs`, `domain/src/lead.rs`, `domain/src/reputation.rs`, `domain/src/staff.rs`, `domain/src/retail/product.rs`, `domain/src/retail/recommendation.rs`, plus app/storage boundary imports.

Classification: implement-now only after lanes 1-3.

These diagnostics are real, but doing them first would hide the bigger issue. Once canonical surfaces are chosen, migrate imports from leaf-flattened forms like `OperationalObservation`, `ReservationId`, `InventoryPolicy`, and `SaleQuantity` to module-qualified forms where the namespace carries meaning (`operations::OperationalObservation`, `entities::ReservationId`, `inventory::Policy` or `inventory::InventoryPolicy` depending on the final retail surface).

## Accepted/deferred

### `training::package::id::Id` contradiction

Files: `domain/src/training/mod.rs` around the `package::id` module.

Classification: accepted/deferred / needs design if revisited.

Modum simultaneously flags `id` as catch-all, asks for `training::package::Id`, and also flags `id::Id` as redundant. This is not a safe linter-driven change. Resolve only as part of an explicit training package identity design: either `training::package::Id` as the public surface with an internal `id` implementation module, or a stronger package identity facet. Do not chase both diagnostics mechanically.

### Macro-generated scalar families

Files: `domain/src/training/mod.rs`, `domain/src/grooming/mod.rs`, `storage/src/operations.rs`, `storage/src/lib.rs`.

Classification: accepted/deferred / linter-noise until manually inspected.

Unsupported-construct diagnostics mostly come from `positive_scalar!`/macro-shaped families. The macros are not automatically wrong; the right question is whether each generated scalar/error pair lives under the true semantic owner and exposes one canonical path. Treat these as manual review, not automatic split/rename work.

### Local `error.rs` imports and external derive names

Files include local error modules and `thiserror::Error` imports.

Classification: accepted linter-noise.

The doctrine explicitly accepts local `error.rs` modules with `Error`/`Result` surfaces and external derive names. Do not flatten or rename these just to satisfy namespace diagnostics.

### Gingr transport error shape

Files: `integrations/gingr/src/transport.rs` and local error types.

Classification: accepted/deferred.

Provider transport is boundary code. It should be typed and explicit, but it may preserve provider/transport vocabulary. Keep the existing defer unless a Gingr boundary cleanup card explicitly tackles request parts, path parameters, and transport errors together.

## Needs-human-design

### Domain `entities` canonical surface

File: `domain/src/entities.rs`.

Classification: directionally right / needs human design.

`entities` still contains families that probably deserve real modules: `reservation::{Id, Status, Source}`, `care_note::{Id, Subject, Kind, Visibility, Body}`, `approval::{Id, Record, Target, Lifecycle, Status}`, and `audit::{Event, Subject, Action, MetadataKey, MetadataValue}`. However, the repo also has top-level `reservation`, `care`, and `audit` modules. Before moving anything, decide whether `entities` remains a compatibility/core aggregate or whether these families move to their top-level owners.

Recommended next card: domain entity surface design; decide canonical paths for reservation/care-note/approval/audit and then migrate tests/callers.

### Storage boundary shape

Files: `storage/src/lib.rs`, `storage/src/operations.rs`, `storage/src/service/*`.

Classification: directionally right / needs human design.

`storage::service` is a weak bucket, but the file comments say this crate owns persistence-shaped records/codes and domain promotion/demotion. The better shape is not simply deleting `service`; choose whether storage mirrors domain service-line modules (`storage::{boarding, daycare, grooming, retail, training}`), owns a `storage::service_line` family, or keeps `service` as an intentionally documented compatibility boundary.

`storage/src/operations.rs` also re-exports many service-line storage codes as flat prefixed names (`BoardingAccommodationCode`, `GroomingServiceCode`, `TrainingProgramRecord`). Those are useful for record searchability but create a parallel public surface. Pick one canonical boundary style before renaming.

Recommended next card: storage-boundary design and implementation; include promotion/demotion call sites and storage tests.

### Gingr endpoint and webhook canonical surfaces

Files: `integrations/gingr/src/webhook.rs`, `integrations/gingr/src/endpoint/reservations.rs`, endpoint module re-exports.

Classification: directionally right / needs boundary design.

`webhook::WebhookEventType`, `WebhookEntityId`, `WebhookEnvelope`, `VerifiedWebhook`, etc. repeat `webhook`; a semantic provider boundary would read better as `webhook::{EventType, EntityId, Envelope, Verified, Ack}`. `endpoint::reservations` similarly repeats `Reservations*` and has builder families that could become `reservations::{Builder, ByAnimal, ByOwner}` or parent-level endpoint surfaces.

Because this is external API boundary code, preserve provider terminology where it prevents ambiguity. Do a focused Gingr API-surface pass rather than mixing it with domain service-line cleanup.

### `policy` / play eligibility ownership

File: `domain/src/policy.rs`.

Classification: needs human design / prior known pressure.

`policy::PlayEligibilityPolicy`, `PolicyDenialReason`, and related aliases remain a weak-bucket smell. The better owner is likely `play::eligibility::{Policy, Decision, DenialReason}` or an `eligibility_policy::play` family. This should be decided with the domain model, not shortened to `policy::PlayEligibility` by lint.

## Linter-noise / lower-priority items

- `api_candidate_semantic_module_unsupported_construct` and `namespace_family_unsupported_construct` are analysis limits where macro/source constructs blocked inference.
- `namespace_redundant_qualified_generic` on `id::Id` is part of the `training::package::id::Id` contradiction.
- `api_weak_module_generic_leaf` in local error surfaces should be checked manually; weak technical modules are not automatically wrong in boundary crates.
- `api_boolean_flag_cluster` in `app/src/daily_update.rs` is a real semantic-code concern but not specifically a module-surface blocker; handle with the daily-update contract, not this module pass.
- `api_repeated_parameter_cluster` in `app/src/booking_triage.rs` is a builder/options concern, not a semantic-module cleanup.

## Test import shape

The main domain tests already use the preferred pattern: import semantic modules (`domain::{agent, care, customer, daily_brief, entities, grooming, lead, location, money, operations, ...}`) and call through those modules at assertion sites. That is a good model for production migration. The tests will still need updates after canonical public surfaces change, but they are not the source of the remaining pressure.

## Suggested follow-up cards

1. `domain service-line canonical surfaces`: remove/narrow duplicate flat aliases in `retail`, `grooming`, `training`, `staff`, `daily_brief`, `lead`, and `reputation`; update call sites/tests.
2. `operations prelude cleanup`: stop using `domain::operations` as an umbrella re-export for sibling domains; preserve only concepts truly owned by operations.
3. `app tools semantic modules`: split `app::tools` into availability, portal lookup, payments authorization/refund/deposit, messaging/draft_message, documents/ocr, task, and schedule surfaces.
4. `storage boundary design`: decide `storage::service` versus service-line root modules versus `service_line`; then migrate records/codes and storage tests.
5. `Gingr boundary surface pass`: normalize webhook and endpoint reservation names while preserving provider API vocabulary.
6. `entities canonical domain owners`: decide whether reservation/care-note/approval/audit stay under `entities` or move to top-level domain owners.

## Verification notes

- Reran `modum check --format json`; diagnostics remain 227 and are classified above.
- Ran `cargo fmt --check`.
- Ran `cargo test --workspace`.
- Ran `git diff --check HEAD`.
- No Rust source changed in this audit card.
