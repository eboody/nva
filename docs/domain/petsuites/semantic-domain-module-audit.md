# PetSuites semantic domain-module audit

Audit date: 2026-06-11

## Why this audit exists

The previous PetSuites service-domain board produced useful service maps, implication documents, and a set of typed Rust contracts. However, the intended architecture is stricter than “docs plus some typed contracts.” Each service line should be a semantic domain module with its own owned vocabulary and invariants. Operational workflows should compose those domain types and introduce workflow-owned types only where the workflow has distinct meaning. Workflow orchestration belongs in the application layer, not as agent/workflow execution logic inside the domain crate. Storage records and provider DTOs should be explicit boundary variants that convert into service-domain types.

## Current repo shape

Workspace members:

- `domain`
- `storage`
- `integrations/gingr`
- `apps/api`
- `apps/cli`
- `apps/worker`

Relevant current files:

- `domain/src/operations.rs`: 4,809 lines.
- `storage/src/operations.rs`: 1,100 lines.
- `domain/src/booking_triage.rs`: 628 lines.
- `domain/src/daily_update.rs`: 608 lines.
- `domain/src/workflow.rs`: 341 lines.
- `apps/api/src/http.rs`: 685 lines.

`domain/src/operations.rs` currently contains nested service modules rather than service-owned source modules:

| Nested module | Lines | Span |
| --- | ---: | --- |
| `operations::boarding` | 686 | 1124-1809 |
| `operations::daycare` | 843 | 1810-2652 |
| `operations::grooming` | 642 | 2653-3294 |
| `operations::training` | 910 | 3295-4204 |
| `operations::retail` | 605 | 4205-4809 |

The service-domain maps exist under `docs/domain/petsuites/<service>/service-domain-map.md`, but the source tree does not yet reflect the intended module shape.

## Findings

### 1. Service lines are not first-class source modules

Status: **needs remediation**.

There are service domains, but they are nested in a single `operations.rs` monolith:

- `domain::operations::boarding`
- `domain::operations::daycare`
- `domain::operations::grooming`
- `domain::operations::training`
- `domain::operations::retail`

This is better than raw enums, but it is not the desired architecture. The desired shape is closer to:

```text
domain/src/service/
  mod.rs
  boarding/
    mod.rs
    accommodation.rs
    capacity.rs
    care.rs
    deposit.rs
    handoff.rs
    upsell.rs
    ...
  daycare/
    mod.rs
    eligibility.rs
    group.rs
    coverage.rs
    incident.rs
    recurring.rs
    package.rs
    ...
  grooming/
    mod.rs
    calendar.rs
    appointment.rs
    coat.rs
    no_show.rs
    rebooking.rs
    reminder.rs
    history.rs
    ...
  training/
    mod.rs
    enrollment.rs
    curriculum.rs
    trainer.rs
    progress.rs
    outcome.rs
    package.rs
    follow_up.rs
    ...
  retail/
    mod.rs
    product.rs
    inventory.rs
    pos.rs
    recommendation.rs
    reorder.rs
    vendor.rs
    ...
```

A compatibility re-export can preserve old call sites temporarily, but the canonical public surface should be service-owned, e.g. `domain::service::boarding::capacity::Policy`, not an ever-growing `domain::operations` module.

### 2. Cross-service shared concepts need explicit ownership

Status: **partially present, needs clearer extraction**.

Some cross-service concepts already exist: `policy`, `workflow`, `reservation`, `payment`, `money`, `care`, `temperament`, `vaccine`, `document`, `location`, `pet`, and `customer`. That direction is good.

However, `operations.rs` still owns many cross-service operational types such as daily-brief/occupancy/labor/revenue/task concepts. These need an ownership pass. Some may belong in:

- `domain::service` for service-line concepts.
- `domain::operations` only for true cross-service operations management concepts.
- `domain::staff` or `domain::task` for staff tasking.
- `domain::approval` / `domain::execution` for human-gated action control.
- `domain::workflow` for domain event contracts, not app workflow execution.

### 3. Operational workflows are in the domain crate

Status: **needs remediation**.

`domain/src/booking_triage.rs` and `domain/src/daily_update.rs` contain workflow/agent behavior and preview construction. They use domain values, but they are application/workflow orchestration surfaces, not pure domain concepts.

Desired split:

- Domain crate: domain entities, service modules, policies, decisions, typed evidence, approval gates, executable-action boundaries.
- App crate(s): workflow orchestration such as booking triage, daily care update generation, incident escalation, customer messaging, task creation.
- Workflows should compose service-domain types, e.g. booking triage should ask boarding capacity/deposit/care policies for domain decisions and then assemble a workflow-owned packet/draft/gate result.

There is currently no dedicated `app`/`application` crate. Existing `apps/api` and `apps/worker` are thin binary/application shells. A new workspace member such as `app` or `application` may be appropriate if workflows are shared between API and worker.

### 4. Storage variants exist but are monolithic and not service-owned

Status: **partially present, needs remediation**.

`storage/src/operations.rs` contains records and conversions, including storage variants for `CoreServiceContracts`. This proves boundary serialization exists, but it mirrors the monolithic operations shape.

Desired storage shape:

```text
storage/src/service/
  boarding.rs
  daycare.rs
  grooming.rs
  training.rs
  retail.rs
```

Each storage module should define explicit record/row/document variants for the service-owned domain module and named conversions across the boundary. Storage should not become a second domain model; it should be a boundary representation that promotes into domain types.

### 5. DTO/provider mapping gaps remain

Status: **needs remediation where relevant**.

`integrations/gingr` has endpoint catalogs and some mappings, but provider DTOs for retail/training/grooming/service-specific operations remain incomplete or explicitly documented as mapping gaps. DTOs should be introduced only where they correspond to real provider/API payloads, and they should convert into semantic service-domain types before workflow/app code sees them.

Desired shape:

```text
integrations/gingr/src/dto/
  boarding.rs
  daycare.rs
  grooming.rs
  training.rs
  retail.rs
integrations/gingr/src/mapping/
  boarding.rs
  daycare.rs
  grooming.rs
  training.rs
  retail.rs
```

DTO names should speak Gingr/provider language; mapping functions should explicitly promote them into `domain::service::*` types.

### 6. Tests are useful but reinforce the current monolith

Status: **needs follow-up**.

`domain/tests/petsuites_core_service_contracts.rs` contains good semantic assertions, but it imports `domain::operations::*` paths and therefore locks in the monolithic public shape. These tests should be split or augmented with service-module contract tests that prove the canonical service paths.

## Target architecture standard

A remediation is done only when:

1. Each service has a service-domain module with owned types and policies.
2. Cross-service concepts are extracted into explicit shared modules only when they are truly shared.
3. Workflow/application orchestration is outside the domain crate, while still preserving domain-only policy/decision functions in domain modules.
4. Storage has explicit service boundary record variants and conversions for migrated service-domain types.
5. Provider DTOs/mappings exist where provider data is relevant and do not leak into the domain or app workflow core.
6. Old `domain::operations::*` paths are either removed or retained only as compatibility re-exports with tests proving the canonical `domain::service::*` paths.
7. `cargo fmt --check`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Recommended remediation board

Create a staged, serialized board because this repo is not a git repository and all code cards share one checkout.

Suggested card graph:

1. Audit/canonical architecture plan.
2. Extract boarding service module.
3. Extract daycare service module.
4. Extract grooming service module.
5. Extract training service module.
6. Extract retail service module.
7. Extract cross-service shared modules and prune `operations.rs`.
8. Move booking triage and daily care update orchestration into an application crate/layer; keep domain policies in domain.
9. Add storage service record/conversion modules.
10. Add Gingr DTO/mapping skeletons for service domains where source endpoints exist; document provider gaps where they do not.
11. Split/update tests to assert canonical service-domain paths and workflow composition.
12. Final semantic review and Rust gates.

## Current verdict

The previous board was **useful but not complete against the intended architecture**. It created enough typed material to begin the conversation with NVA, but it did not fully create service-owned domain modules or correctly separate application workflows from domain contracts. A remediation board is warranted.
