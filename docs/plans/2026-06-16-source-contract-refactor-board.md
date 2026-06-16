# Source Contract Refactor Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Keep the shared checkout serialized; do not dispatch parallel mutating tasks against `/home/eran/code/nva`.

**Goal:** Refactor the current Gingr-shaped reservation/source prototype into source-agnostic domain contracts while preserving Gingr as one quarantined provider adapter namespace.

**Architecture:** The domain core owns source-agnostic provenance, source-record identity, reservation source snapshots, typed data-quality issues, and analytics projections. Gingr-specific endpoint names, provider IDs, statuses, request scopes, and parser/schema versions move under a Gingr adapter namespace that promotes into the core through explicit TDD-backed conversions. BI discovery answers remain the decision mechanism: unknown grain, IDs, refresh behavior, or reliability become typed assumptions/issues, not hard-coded Gingr truths.

**Tech Stack:** Rust workspace, `domain` crate, existing `bon`, `serde`, `thiserror`, `chrono`, integration tests under `domain/tests/`.

---

## Current shape inspected

Files inspected:

- `domain/src/source.rs`
- `domain/src/data_quality.rs`
- `domain/src/analytics.rs`
- `domain/tests/reservation_source_contracts.rs`
- canonical BI docs:
  - `docs/discovery/bi-data-shape-questions.md`
  - `docs/discovery/bi-question-decision-rubric.md`

Current semantic pressure:

- `source::System` contains only `Gingr`, so the root source model is nominally generic but effectively provider-specific.
- `source::gingr::Provenance` is the only provenance type; `data_quality::Issue` and `analytics::stay::Fact` therefore depend on `source::gingr::Provenance`.
- `source::gingr::reservation::Snapshot` owns the reservation snapshot shape; projection entrypoint is `analytics::stay::Fact::project_from_gingr_reservation(...)`.
- The test names already express useful domain truths, but the public API still teaches callers that Gingr is the core model.

Target pressure release: make the core path read as source-agnostic at call sites (`source::Provenance`, `source::record::Id`, `source::reservation::Snapshot`, `analytics::stay::Fact::project_from_reservation_snapshot`) while Gingr-specific details stay visibly quarantined (`source::gingr::Endpoint`, `source::gingr::ProviderRecordId`, `source::gingr::reservation::Snapshot::promote(...)` or equivalent adapter conversion).

## Exact semantic module layout

Implement inside the existing `domain` crate first. Do not create new crates for this refactor.

```text
domain/src/source.rs                  # root source contracts and module declarations while small
domain/src/source/provenance.rs       # move here only if source.rs becomes hard to read
domain/src/source/record.rs           # source-agnostic record identity and related-record roles
domain/src/source/reservation.rs      # source-agnostic reservation source snapshot contract
domain/src/source/gingr.rs            # Gingr adapter/source namespace while small
domain/src/source/gingr/reservation.rs# split here only after the root file is already green
```

Because `source.rs` is currently one file, the first implementation may keep nested modules in `domain/src/source.rs` to minimize churn. The desired public paths still must be:

```rust
source::System
source::Provenance
source::Timestamp
source::PayloadHash
source::RawPayloadRef
source::record::Id
source::record::RelatedId
source::record::Role
source::reservation::Snapshot
source::reservation::OwnerPetRelationship
source::reservation::Status
source::reservation::Assumption
source::gingr::Endpoint
source::gingr::ProviderRecordId
source::gingr::ExtractionBatchId
source::gingr::RequestScope
source::gingr::ProviderSchemaVersion
source::gingr::ProviderStatus
source::gingr::reservation::Snapshot
```

Canonical ownership rules:

- Root `source::*` owns facts any upstream source can provide.
- `source::record::*` owns source record identity, related source-record roles, and join assumptions.
- `source::reservation::*` owns the source-agnostic reservation/stay snapshot grain used by data quality and analytics.
- `source::gingr::*` owns Gingr names and raw/provider-shaped values only.
- `data_quality::*` may refer to `source::Provenance` and `source::Field`, but must not refer to `source::gingr::*`.
- `analytics::*` may refer to `source::reservation::Snapshot` and source-agnostic IDs/statuses, but must not require Gingr types.

## Target type names and responsibilities

### Source root

```rust
source::System
```

Keep enum-based, not trait-based:

```rust
pub enum System {
    Gingr,
    BusinessIntelligence,
    LaborScheduling,
    Timeclock,
    Payroll,
    CapacityInventory,
    PointOfSale,
    ManualImport,
}
```

Only add variants with test coverage showing why the source family matters. It is acceptable for the first refactor to add `BusinessIntelligence` and leave the rest as plan-only names if tests do not need them yet.

```rust
source::Provenance
```

Source-agnostic provenance fields:

- `system: source::System`
- `record_id: source::record::Id`
- `related_record_ids: Vec<source::record::RelatedId>`
- `extraction_batch: source::ExtractionBatchId`
- `pulled_at: source::Timestamp`
- `request_scope: source::RequestScope`
- `schema_version: source::SchemaVersion`
- `payload_hash: source::PayloadHash`
- `raw_payload_ref: source::RawPayloadRef`

Use source-agnostic names in the root. If Gingr needs the word `provider`, keep that in `source::gingr::ProviderRecordId` and convert/promote into `source::record::Id`.

```rust
source::ExtractionBatchId
source::RequestScope
source::SchemaVersion
```

These should be source-agnostic root types. Existing `source::gingr::ExtractionBatchId`, `RequestScope`, and `ProviderSchemaVersion` become Gingr adapter wrappers only if they protect Gingr-specific vocabulary; otherwise migrate call sites directly to root types.

### Source record identity

```rust
source::record::Id
source::record::Role
source::record::RelatedId
```

`Role` should encode join meaning without saying Gingr:

```rust
pub enum Role {
    Customer,
    Pet,
    Location,
    ReservationType,
    Invoice,
    Payment,
    Service,
    Staff,
    Unknown,
}
```

Use `Unknown` only with a data-quality issue or assumption attached. Do not silently map unknown roles into `Service` or `ReservationType` because that would hide BI uncertainty.

### Reservation source snapshot

```rust
source::reservation::Snapshot
source::reservation::OwnerPetRelationship
source::reservation::Status
source::reservation::Assumption
```

Fields for the source-agnostic first slice:

- `provenance: source::Provenance`
- `customer_record_id: Option<source::record::Id>`
- `pet_record_id: Option<source::record::Id>`
- `location_record_id: Option<source::record::Id>`
- `service_type_record_id: Option<source::record::Id>`
- `status: Option<source::reservation::Status>`
- `relationship: source::reservation::OwnerPetRelationship`
- `assumptions: Vec<source::reservation::Assumption>`

Status should not be a raw provider status string in the core. Start with conservative variants:

```rust
pub enum Status {
    Requested,
    Confirmed,
    CheckedIn,
    CheckedOut,
    Cancelled,
    Unknown { observed: source::ObservedStatus },
}
```

If observed raw status text is needed, wrap it in `source::ObservedStatus`, not `String`.

Assumptions are typed placeholders for BI-discovery uncertainty:

```rust
pub enum Assumption {
    GrainTreatedAsReservation,
    CustomerRecordIdTreatedAsStableJoinKey,
    PetRecordIdTreatedAsStableJoinKey,
    ProviderStatusMappingIsProvisional,
    RawPayloadRetentionUnknown,
    RefreshMutationPolicyUnknown,
}
```

Only add assumptions that correspond to the BI question/rubric buckets.

### Gingr namespace

`source::gingr::*` should keep external vocabulary and promotion helpers:

```rust
source::gingr::Endpoint
source::gingr::ProviderRecordId
source::gingr::ProviderStatus
source::gingr::ProviderSchemaVersion
source::gingr::reservation::Snapshot
```

The adapter snapshot may have Gingr-shaped fields. It must expose an explicit promotion operation, not implicit `.into()`:

```rust
impl source::gingr::reservation::Snapshot {
    pub fn promote(self) -> source::Result<source::reservation::Snapshot> { ... }
}
```

If promotion cannot be total because status mappings or join assumptions are unknown, return the source-agnostic snapshot with typed assumptions/issues rather than panicking or pretending certainty.

### Data quality

`data_quality::Issue` must become source-agnostic:

```rust
data_quality::Issue {
    kind: data_quality::Kind,
    severity: data_quality::Severity,
    provenance: source::Provenance,
    detected_at: source::Timestamp,
    resolution_status: data_quality::ResolutionStatus,
    visible_to_bi: bool,
    workflow_blocking: bool,
}
```

Rename `data_quality::SourceField` variants away from provider language:

```rust
pub enum SourceField {
    CustomerRecordId,
    PetRecordId,
    LocationRecordId,
    ServiceTypeRecordId,
    ReservationStatus,
}
```

Add uncertainty kinds as needed:

```rust
Kind::AssumptionInForce { assumption: source::reservation::Assumption }
Kind::UnknownSourceStatus { observed: source::ObservedStatus }
Kind::UnstableJoinKey { field: SourceField }
Kind::UnknownRefreshMutationPolicy
```

### Analytics projection

`analytics::stay::Fact` should store source-agnostic identity and provenance:

```rust
provenance: source::Provenance
reservation_record_id: source::record::Id
customer_record_id: source::record::Id
pet_record_id: source::record::Id
location_record_id: source::record::Id
service_type_record_id: source::record::Id
```

Projection entrypoint:

```rust
analytics::stay::Fact::project_from_reservation_snapshot(
    id: analytics::stay::Id,
    snapshot: &source::reservation::Snapshot,
    projection_version: analytics::ProjectionVersion,
) -> Result<Self, Vec<data_quality::Issue>>
```

Keep a temporary compatibility wrapper only if needed for a small migration step:

```rust
project_from_gingr_reservation(...)
```

If kept, mark it as transitional in a comment and make it call Gingr promotion plus the source-agnostic projection. Remove it by the end of this plan unless it is still needed by unchanged tests.

## Migration sequence

### Task 1: Pin the desired source-agnostic API with failing tests

**Objective:** Make the public contract fail for the right reason before touching production code.

**Files:**

- Modify: `domain/tests/reservation_source_contracts.rs`

**Step 1: Write failing tests**

Add or rewrite tests to name the desired contract:

- `source_agnostic_reservation_snapshot_preserves_provenance_without_gingr_paths`
- `gingr_reservation_snapshot_promotes_to_source_agnostic_snapshot_with_assumptions`
- `data_quality_issues_do_not_depend_on_gingr_provenance`
- `complete_source_reservation_snapshot_projects_to_stay_fact`

Expected compile failures before implementation:

- `source::Provenance` missing
- `source::record::Id` missing
- `source::reservation::Snapshot` missing
- `analytics::stay::Fact::project_from_reservation_snapshot` missing
- `data_quality::SourceField::CustomerRecordId` missing

**Step 2: Verify RED**

Run:

```bash
cargo test -p domain --test reservation_source_contracts
```

Expected: FAIL for missing source-agnostic API, not unrelated syntax errors.

### Task 2: Introduce root provenance and source record identity

**Objective:** Add the source-agnostic provenance/record layer without changing reservation projection behavior yet.

**Files:**

- Modify: `domain/src/source.rs`
- Test: `domain/tests/reservation_source_contracts.rs`

**Implementation notes:**

- Add `source::Provenance` with a `bon::Builder` to preserve existing construction ergonomics.
- Add `source::record::{Id, Role, RelatedId}`.
- Promote `ExtractionBatchId`, `RequestScope`, and `SchemaVersion` to root source types unless a failing test proves a Gingr-specific wrapper is necessary.
- Keep `source::gingr::Provenance` only as a transitional adapter if needed; do not make it the data-quality or analytics type.

**Verification:**

```bash
cargo test -p domain --test reservation_source_contracts
```

Expected: root provenance tests pass or move to the next missing reservation snapshot API failure.

### Task 3: Move data-quality issues to source-agnostic provenance

**Objective:** Break the dependency from `data_quality::Issue` to `source::gingr::Provenance`.

**Files:**

- Modify: `domain/src/data_quality.rs`
- Modify: `domain/src/source.rs`
- Test: `domain/tests/reservation_source_contracts.rs`

**Implementation notes:**

- Change `Issue::new(..., provenance: source::Provenance, ...)`.
- Change `Issue::provenance(&self) -> &source::Provenance`.
- Keep `Issue::source_system()` delegating to root provenance.
- Rename `OwnerProviderId`/`AnimalProviderId` to `CustomerRecordId`/`PetRecordId`; update tests first and then implementation.
- Add assumption/uncertainty issue kinds only as tests demand.

**Verification:**

```bash
cargo test -p domain --test reservation_source_contracts
```

Expected: source-agnostic issue tests pass. Existing tests may fail at reservation snapshot call sites until Task 4.

### Task 4: Introduce source-agnostic reservation snapshot

**Objective:** Make reservation source facts independent of Gingr-specific IDs/statuses.

**Files:**

- Modify: `domain/src/source.rs`
- Test: `domain/tests/reservation_source_contracts.rs`

**Implementation notes:**

- Add `source::reservation::Snapshot` and builder.
- Move `OwnerPetRelationship` to `source::reservation`.
- Add `source::reservation::Status`, `Assumption`, and optional `ObservedStatus` if needed.
- Implement `Snapshot::data_quality_issues(...) -> Vec<data_quality::Issue>` against root provenance and source-agnostic fields.
- Missing customer/pet/location/service/status must emit typed issues or assumptions.

**Verification:**

```bash
cargo test -p domain --test reservation_source_contracts
```

Expected: source reservation snapshot and data-quality tests pass. Projection may still fail until Task 5.

### Task 5: Quarantine Gingr snapshot and add explicit promotion

**Objective:** Preserve Gingr-specific source language in one adapter namespace while exposing a source-agnostic core snapshot.

**Files:**

- Modify: `domain/src/source.rs`
- Test: `domain/tests/reservation_source_contracts.rs`

**Implementation notes:**

- Keep Gingr external types under `source::gingr::*`.
- Add or adapt `source::gingr::reservation::Snapshot` so it can promote into `source::reservation::Snapshot`.
- Map `ProviderRecordId` into `source::record::Id` by explicit constructor/conversion at the adapter boundary.
- Map `ProviderStatus` into `source::reservation::Status`; unknown statuses should become `Status::Unknown { observed }` plus a data-quality issue or assumption.
- Preserve endpoint/provider schema version/request scope as provenance metadata without leaking Gingr names into the root type names.

**Verification:**

```bash
cargo test -p domain --test reservation_source_contracts
```

Expected: Gingr promotion test passes and call sites make the provider boundary obvious.

### Task 6: Refactor analytics stay projection to source-agnostic snapshots

**Objective:** Make `analytics::stay::Fact` project from source-agnostic source facts.

**Files:**

- Modify: `domain/src/analytics.rs`
- Test: `domain/tests/reservation_source_contracts.rs`

**Implementation notes:**

- Change stored provenance to `source::Provenance`.
- Change provider-specific fields to `source::record::Id`.
- Add `project_from_reservation_snapshot(...)`.
- Remove or thin `project_from_gingr_reservation(...)` so the core projection is no longer Gingr-shaped.
- Keep `DataQualityStatus::Complete` only for snapshots with no blocking issues and no blocking assumptions.

**Verification:**

```bash
cargo test -p domain --test reservation_source_contracts
```

Expected: projection test passes through the source-agnostic entrypoint.

### Task 7: Clean up names and guard against alias/re-export soup

**Objective:** Ensure final public paths preserve meaning without parallel synonyms.

**Files:**

- Modify: `domain/src/source.rs`
- Modify: `domain/src/data_quality.rs`
- Modify: `domain/src/analytics.rs`
- Modify: `domain/tests/reservation_source_contracts.rs`

**Implementation notes:**

- Search for `source::gingr::Provenance` outside `source::gingr` tests/adapters and remove it.
- Search for `ProviderRecordId` in `data_quality.rs` and `analytics.rs`; there should be none.
- Avoid aliases like `pub use gingr::ProviderRecordId as RecordId`; that masks the boundary.
- Keep one canonical core path and one visible adapter path.

**Verification:**

```bash
rg 'source::gingr::Provenance|ProviderRecordId' domain/src/data_quality.rs domain/src/analytics.rs domain/tests/reservation_source_contracts.rs
cargo test -p domain --test reservation_source_contracts
cargo test -p domain
```

Expected: no Gingr-specific types in `data_quality` or analytics core; domain tests pass.

### Task 8: Run workspace quality gates and commit

**Objective:** Prove the refactor is formatted, test-backed, and scoped.

**Files:**

- No new feature files unless earlier tasks split `source.rs` after tests are green.

**Commands:**

```bash
cargo fmt --check
cargo test -p domain
git diff --check
git status --short
```

Expected:

- formatting passes;
- domain tests pass;
- no whitespace errors;
- only intended source/test files changed.

Commit after review:

```bash
git add domain/src/source.rs domain/src/data_quality.rs domain/src/analytics.rs domain/tests/reservation_source_contracts.rs
git commit -m "refactor: make source reservation contracts source agnostic"
```

## Test contracts to preserve

These are the executable glossary entries the implementation should end with:

```rust
source_agnostic_reservation_snapshot_preserves_provenance_without_gingr_paths
gingr_reservation_snapshot_promotes_to_source_agnostic_snapshot_with_assumptions
missing_and_ambiguous_source_facts_emit_source_agnostic_data_quality_issues
unknown_provider_status_is_typed_uncertainty_not_a_trusted_workflow_status
complete_source_reservation_snapshot_projects_to_stay_fact
analytics_stay_fact_does_not_expose_gingr_provider_identity_types
```

Test contract details:

- A source-agnostic snapshot can be built without referencing `source::gingr::*`.
- Gingr promotion preserves endpoint, pulled-at timestamp, raw payload ref, payload hash, and source system.
- Missing customer/pet/location/service fields emit `data_quality::Kind::MissingRequiredField` with source-agnostic fields.
- Ambiguous owner-pet relationship remains a blocking data-quality issue.
- Unknown provider status is visible to BI and workflow-blocking until mapped or acknowledged.
- A complete source reservation snapshot projects into `analytics::stay::Fact` without Gingr provider types in the fact API.

## Task-selection note tied to the BI rubric

This refactor is the correct next task under `docs/discovery/bi-question-decision-rubric.md` because the highest-risk unanswered buckets are:

1. **Grain and identity:** we do not yet know which BI rows are reservation, pet stay, service-day, or another grain, nor which IDs are stable enough to join. The refactor introduces source-agnostic record IDs, related-record roles, and typed assumptions instead of letting Gingr provider IDs become domain truth.
2. **Refresh and mutation behavior:** edits, backfills, merges, deletes, and duplicate behavior are not known. First-class provenance, batch IDs, pulled-at timestamps, payload hashes, and raw refs are prerequisites for detecting mutation rather than treating rows as immutable facts.
3. **Reliability and semantic ambiguity:** statuses and relationships may be overloaded or messy. Unknown provider statuses and ambiguous owner-pet relationships must become typed data-quality issues, not workflow statuses.
4. **Provenance and raw retention:** BI may or may not retain raw payloads/import batches. The core contract can represent both retained raw refs and explicit `RawPayloadRetentionUnknown` assumptions.

This deliberately does not implement labor optimization yet. The rubric says labor-cost modeling waits until schedule/timeclock/payroll/capacity signals exist. The current safe move is contract refinement for source facts that future labor-cost projections can trust.

## What NOT to do yet

- Do not create a new `analytics` crate or source-contract crate. The current `domain` crate is enough for this contract slice.
- Do not introduce a `SourceAdapter` trait. There is only one real adapter in code today; enum/value contracts are clearer until a second source is implemented.
- Do not add Statum machines or workflow validators. The rubric says workflow validators wait until source facts and reliability are proven.
- Do not model payroll/timeclock/labor optimization from guesses. Add those when BI/source artifacts show available scheduling, timeclock, payroll, capacity, or wage-cost data.
- Do not add live database connectors, credentials, or real Gingr/BI data.
- Do not hide Gingr behind aliases or broad re-exports such as `pub use gingr::ProviderRecordId as RecordId`.
- Do not preserve `project_from_gingr_reservation` as the canonical analytics API.
- Do not treat unknown statuses, missing IDs, unclear grain, or unknown raw retention as non-errors just because tests need a fact to project.
- Do not use `.into()` for trust-boundary promotion where validation, normalization, or assumption recording occurs.

## Acceptance criteria

- `data_quality.rs` and `analytics.rs` no longer depend on `source::gingr::Provenance` or `source::gingr::ProviderRecordId`.
- The canonical projection path is `analytics::stay::Fact::project_from_reservation_snapshot`.
- Gingr-specific types remain visibly under `source::gingr::*`.
- Unknown BI/source facts are represented as `source::reservation::Assumption` or `data_quality::Issue`.
- Tests demonstrate both source-agnostic construction and Gingr adapter promotion.
- Gates pass:

```bash
cargo fmt --check
cargo test -p domain
git diff --check
```
