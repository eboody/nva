# PetSuites service-domain modularization plan

Status: canonical source-of-truth plan for the serialized remediation cards.

Source audit: `docs/domain/petsuites/semantic-domain-module-audit.md`.

## Goal

Make service-domain ownership visible in Rust module paths. Service modules own their types, policies, decisions, and invariants. Shared modules own only genuinely shared domain concepts. Application workflows compose domain values. Storage and provider crates remain boundary translations.

This card is documentation-only: do not perform broad code moves here.

## Architecture rules

1. `domain` owns semantic domain contracts.
2. `domain::service::<service>` owns service-line vocabulary.
3. `domain::operations` owns only true cross-service resort-operations management concepts.
4. `domain::workflow` owns event/result contracts, not workflow execution.
5. App/API/worker layers own orchestration, preview generation, provider calls, storage calls, and agent execution.
6. `storage` owns persistence-shaped records/codes and explicit promotion/demotion conversions.
7. `integrations/gingr` owns Gingr DTOs/mappings; mapped values must become `domain` types before workflow/app code sees them.
8. Compatibility re-exports are temporary migration scaffolding, never alternate canonical APIs.

## Target domain public surface

Canonical service root:

```text
domain::service
```

Target source shape:

```text
domain/src/service/
  mod.rs
  boarding/{mod.rs,accommodation.rs,capacity.rs,care.rs,deposit.rs,handoff.rs,upsell.rs}
  daycare/{mod.rs,eligibility.rs,group.rs,ratio.rs,attendance.rs,incident.rs,package.rs}
  grooming/{mod.rs,appointment.rs,calendar.rs,coat.rs,no_show.rs,rebooking.rs,reminder.rs,history.rs}
  training/{mod.rs,enrollment.rs,program.rs,curriculum.rs,trainer.rs,progress.rs,outcome.rs,package.rs,follow_up.rs}
  retail/{mod.rs,product.rs,inventory.rs,pos.rs,recommendation.rs,reorder.rs,vendor.rs}
```

Canonical import/call-site posture:

```rust
use domain::service::{boarding, daycare, grooming, retail, training};

let policy: boarding::capacity::Policy = todo!();
let suite: boarding::accommodation::Kind = todo!();
let rules: Vec<daycare::eligibility::Rule> = todo!();
let request: grooming::appointment::Request = todo!();
let program: training::program::Program = todo!();
let category: retail::product::Category = todo!();
```

Prefer paths that preserve the semantic owner:

- `domain::service::boarding::capacity::Decision`
- `domain::service::daycare::eligibility::Rule`
- `domain::service::grooming::rebooking::Cadence`
- `domain::service::training::program::DurationWeeks`
- `domain::service::retail::product::Category`

Avoid preserving old monolith paths as canonical after migration:

- `domain::operations::BoardingAccommodation`
- `domain::operations::DaycareFormat`
- `domain::operations::GroomingService`
- `domain::operations::TrainingProgram`
- `domain::operations::RetailProductCategory`

## What stays in `domain::operations`

`domain::operations` remains for cross-service operations management only. Split the monolith into named submodules such as:

```text
domain::operations::daily_brief
domain::operations::occupancy
domain::operations::labor
domain::operations::revenue
domain::operations::lead
domain::operations::reputation
domain::operations::portfolio
domain::operations::technology
```

Current candidates:

- `ResortDailyBrief`, `DailyBriefSection`, `ResortOperatingDay`, `SnapshotId` -> `operations::daily_brief`.
- aggregate `CapacityMetric`/`CapacityLimit` values -> `operations::occupancy` only when they describe aggregate occupancy, not service-specific capacity rules.
- `LaborSnapshot`, `LaborRisk`, `ScheduledStaffCount` -> `operations::labor` unless later owned by `domain::staff`.
- `RevenueOpportunity` -> `operations::revenue` when cross-service; service-specific upsells move under service modules.
- lead, reputation, portfolio/operator/brand, and technology ecosystem concepts may stay under operations submodules if they are portfolio/operations language rather than service-line language.

## Shared module ownership

### `domain::policy`

Owns cross-service automation/review primitives such as `AutomationLevel` and `ReviewGate`.

Does not own service policy decisions. Service policies live with the service facet:

- `boarding::capacity::Policy`
- `boarding::deposit::Policy`
- `daycare::eligibility::Policy`
- `grooming::no_show::Policy`
- `training::enrollment::Policy`
- `retail::recommendation::Policy`

### `domain::care`

Owns cross-service pet-care facts and care-note vocabulary. Service-specific care rules stay with services:

- boarding medication/feeding/handoff -> `boarding::care` / `boarding::handoff`.
- daycare health/behavior participation -> `daycare::eligibility` / `daycare::group`.
- grooming coat/service-history notes -> `grooming::coat` / `grooming::history`.
- training progress/outcomes -> `training::progress` / `training::outcome`.

### `domain::payment` and `domain::money`

`money` owns amount/currency/value invariants. `payment` owns cross-service payment status, authorization, collection, refund, and ledger-like concepts.

Service commercial rules stay with services: `boarding::deposit`, `daycare::package`, `training::package`, `grooming::no_show`/`grooming::rebooking`, `retail::pos`/`retail::product`.

### `domain::reservation`

Owns reservation identity, source, lifecycle, status, and core scheduling contract.

Service-specific interpretations live under services: boarding stays, daycare attendance, grooming appointments, training enrollment. Retail should not use reservation language unless a real provider/API payload models a booked retail service.

### `domain::staff` and `domain::task`

Move current `operations::StaffTask*` concepts out of the operations monolith.

Recommended roots:

```text
domain/src/staff/mod.rs
domain/src/task/mod.rs
```

Ownership:

- staff role/assignment concepts -> `domain::staff`.
- task identity/kind/status/priority/source/evidence -> `domain::task`.
- service-specific task variants may carry service-domain values.

### `domain::approval` and `domain::execution`

Create as domain contract modules only, not workflow runners.

Recommended roots:

```text
domain::approval::{Gate, Decision, Requirement, Reviewer, Reason}
domain::execution::{Action, Draft, Boundary, Permission, Evidence, Outcome}
```

Durable gate/action/boundary concepts from booking triage move here. Triage packet/draft/orchestration-specific values move to the application layer.

### `domain::workflow`

Owns workflow event/result contracts: `WorkflowEvent`, `WorkflowEventId`, `WorkflowEventType`, `WorkflowSubject`, `PolicyContext`, and general result/action summaries if they remain event contracts.

Does not own booking triage algorithms, daily-update preview generation, customer-message generation, agent prompt execution, provider clients, or storage logic. If `RecommendedAction` becomes an executable command contract, move it toward `domain::execution` and have workflow events reference it.

## Application workflow target

Current domain files `domain/src/booking_triage.rs` and `domain/src/daily_update.rs` mix useful contracts with orchestration/preview behavior.

Create a shared application crate/layer when behavior is used by API and worker:

```text
application/src/lib.rs
application/src/booking_triage.rs
application/src/daily_update.rs
```

Migration rule:

- Domain keeps pure service policies, decisions, evidence types, approval/execution contracts, and event/result contracts.
- Application owns packet assembly, deterministic triage flow, daily-update preview generation, customer-message draft assembly, provider/storage orchestration, and worker/API entrypoints.
- Application imports semantic domain modules: `domain::service::boarding`, `domain::reservation`, `domain::approval`, `domain::execution`, `domain::workflow`.

## Storage target surface

Canonical storage root:

```text
storage::service
```

Target shape:

```text
storage/src/service/{mod.rs,boarding.rs,daycare.rs,grooming.rs,training.rs,retail.rs}
storage/src/operations/{mod.rs,daily_brief.rs,labor.rs,portfolio.rs,technology.rs}
storage/src/{staff.rs,task.rs,approval.rs,execution.rs}
```

Rules:

- Storage names use boundary language: `Record`, `Code`, `Row`, `Document`, `Payload`.
- Storage modules depend on domain modules; domain never depends on storage.
- Storage codes are representations, not parallel domain enums.
- Conversions are explicit, especially when validation or semantic promotion occurs.

Example intended shape:

```rust
storage::service::boarding::AccommodationCode
storage::service::boarding::OfferingRecord
impl TryFrom<storage::service::boarding::OfferingRecord> for domain::service::boarding::Offering
```

## Gingr/provider target surface

Target shape:

```text
integrations/gingr/src/dto/{mod.rs,boarding.rs,daycare.rs,grooming.rs,training.rs,retail.rs}
integrations/gingr/src/mapping/{mod.rs,boarding.rs,daycare.rs,grooming.rs,training.rs,retail.rs}
```

Rules:

- DTOs speak Gingr/API language and exist only for real payloads/endpoints.
- Mappings explicitly promote DTOs into `domain::service::*`, `domain::reservation`, `domain::payment`, `domain::customer`, `domain::pet`, etc.
- Provider mapping gaps stay explicit; do not invent fake DTOs to make the tree look complete.
- App/workflow code consumes mapped domain values, not provider DTOs.

## Compatibility and re-export policy

New code and new tests must use canonical paths.

Temporary old-path compatibility is allowed only as a `pub use` shim with a removal comment:

```rust
// Temporary compatibility; canonical path is
// `domain::service::boarding::accommodation::Kind`.
// Remove after storage/app/test call sites migrate.
pub use crate::service::boarding::accommodation::Kind as BoardingAccommodation;
```

Migration sequence per surface:

1. Add the new owner module and canonical tests.
2. Move/port types into the new owner.
3. Add old-path re-exports only where required to keep the slice small.
4. Update storage/provider/app/tests to canonical paths.
5. Remove old-path imports from tests.
6. Remove compatibility re-exports once no call sites need them.

Do not leave both old and new paths documented as equally valid public APIs.

## Serialized migration slices

1. `domain::service::boarding`: accommodation, capacity, care, deposit, handoff, upsell.
2. `domain::service::daycare`: eligibility, group, ratio, attendance, incident, package.
3. `domain::service::grooming`: appointment, calendar, coat, no-show, rebooking, reminder, history.
4. `domain::service::training`: enrollment, program, curriculum, trainer, progress, outcome, package, follow-up.
5. `domain::service::retail`: product, inventory, POS, recommendation, reorder, vendor.
6. Shared extractions: `domain::staff`, `domain::task`, `domain::approval`, `domain::execution`, and true `domain::operations::*` submodules.
7. Workflow relocation: move booking triage and daily-update orchestration into application layer while retaining domain contracts.
8. Boundary shape: add `storage::service::*` records/conversions and real Gingr `dto::*`/`mapping::*` modules.
9. Test cleanup: service-contract tests prove canonical `domain::service::*` paths; operations tests cover only true cross-service operations.

Each service slice should include at least one canonical-path test before compatibility pruning begins.

## Rust gates

For code-mutating cards, run:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

This plan card is documentation-only, so those gates are not required here.

## Full-board done definition

The modularization is done when:

1. New code imports service lines through `domain::service::*`.
2. Service-specific values are no longer owned by `domain::operations`.
3. Shared modules have explicit semantic ownership and are not catch-alls.
4. Domain workflow modules contain contracts/events, not app orchestration.
5. Application workflow code composes service-domain values from an app layer/crate.
6. Storage and Gingr mappings are explicit boundary translations into canonical domain paths.
7. Compatibility re-exports have been pruned or have specific removal comments and downstream owners.
8. Tests prove canonical paths and no longer anchor the public API to the operations monolith.
9. The Rust gates pass after code-mutating cards.
