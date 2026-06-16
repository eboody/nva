# PetSuites service-domain modularization review

Review date: 2026-06-12
Task: `t_6c5ff004`
Reviewed checkout: `/home/eran/code/pet-resort-agent-foundation`
Baseline remediation commit visible in checkout: `5b5188b test(domain): assert canonical service architecture`

## Verdict

GO for semantic architecture closeout.

NO-GO for production launch on this review alone.

The remediation materially satisfies the architectural standard from `docs/domain/petsuites/semantic-domain-module-audit.md`: service-line source modules now exist under `domain::service`, application workflows live in the `app` crate, storage has service-owned boundary records, DTO/provider mapping is quarantined under `integrations/gingr`, and canonical tests assert `domain::service::*` paths. The remaining gaps are compatibility/depth gaps rather than blockers for closing the modularization remediation: `domain::operations` still owns cross-service portfolio/contracts terms and keeps temporary service-line shims, some service modules are shallow or monolithic internally, and provider DTO coverage is still intentionally partial.

## Gates run

All required Rust gates passed in the shared checkout:

```text
cargo fmt --check
CARGO_BUILD_JOBS=1 cargo test --workspace --quiet
CARGO_BUILD_JOBS=1 cargo clippy --workspace --all-targets -- -D warnings
```

`cargo test --workspace --quiet` reported all non-empty harnesses passing, including the canonical service architecture, app workflow composition, storage contract, Gingr endpoint/mapping, API, worker, and MVP tests.

## Architecture review

### 1. Service domains are first-class source modules

Status: pass for remediation closeout.

Evidence:

- `domain/src/lib.rs` exposes `pub mod service;`.
- `domain/src/service/mod.rs` owns the service-line namespace:
  - `boarding`
  - `daycare`
  - `grooming`
  - `training`
  - `retail`
- `domain/src/service/boarding/` and `domain/src/service/daycare/` are split into policy/facet modules such as `capacity`, `deposit`, `coverage`, `eligibility`, `attendance`, and `front_desk`.
- `domain/src/service/retail/` is split into `product`, `inventory`, `pos`, `recommendation`, `reorder`, and `vendor`.
- `grooming` and `training` now have first-class owned modules and owned types, though they remain internally monolithic and should be split in a later polish pass.

The shape is no longer “docs plus a monolithic `operations.rs`.” `domain::service::<line>` is the canonical semantic home for service-line contracts and policy values.

### 2. Service modules own domain vocabulary and invariants

Status: pass for remediation closeout, with production-depth gaps.

Evidence:

- Boarding owns values such as `RoomInventory`, `StayNights`, `NoticeHours`, `HourOfDay`, `CapacityPlan`, `ServiceWindow`, `MinimumStay`, `CancellationPolicy`, `DepositRule`, `PaymentTiming`, `HousekeepingCadence`, `HandoffRequirement`, and `Upsell`.
- Daycare owns `PackageVisits`, `StaffCount`, `PetCount`, `ServiceVariant`, `CareMode`, `AttendancePolicy`, `PackagePolicy`, `StaffPetRatio`, `GroupAssignmentRule`, `IncidentPolicy`, and `EligibilityRequirement`.
- Retail owns product, POS, inventory, recommendation, reorder, vendor, errors, and contract types under `domain::service::retail::*`.
- Grooming/training own their service contract vocabulary under `domain::service::{grooming,training}`.

The modules use semantic enums/newtypes/builders instead of raw string/boolean helper soup. Some boarding child modules are currently marker/facet files with most implementation still in `boarding/mod.rs`; grooming and training should be decomposed further before the codebase is considered ergonomically mature.

### 3. Workflows are app-layer orchestration

Status: pass.

Evidence:

- `app/src/lib.rs` describes the crate as shared application/workflow orchestration.
- `app/src/booking_triage.rs` and `app/src/daily_update.rs` now contain the large workflow surfaces that were previously called out as mislocated in the domain crate.
- `app/tests/workflow_service_composition_contracts.rs` verifies that booking triage composes `domain::service::boarding::deposit::Policy` decisions without owning that policy.
- `domain/src/lib.rs` no longer exposes `booking_triage` or `daily_update` modules.

This preserves the intended split: domain modules own policy/decision concepts; app workflows assemble review packets, evidence, agent/tool contracts, and previews.

### 4. Storage/DTO boundaries convert into service-domain values

Status: pass for storage; partial for provider DTO coverage.

Storage evidence:

- `storage/src/service_line/mod.rs` declares service-line storage modules for `boarding`, `daycare`, `grooming`, `retail`, and `training`.
- `storage/src/operations.rs` now stores `CoreServiceContractsRecord` fields as `crate::service_line::<line>::ContractRecord` values.
- `storage/tests/core_service_contract_storage.rs` round-trips `CoreServiceContractsRecord` through JSON and rejects invalid validated scalars.
- `storage/tests/operations_storage_contracts.rs` exercises service-domain paths such as `domain::service::grooming::*`, `domain::service::training::*`, and `domain::service::retail::*`.

DTO/provider evidence:

- `integrations/gingr/src/dto/{grooming,retail,training}.rs` and `integrations/gingr/src/mapping/retail.rs` keep provider DTO vocabulary outside the domain crate.
- `integrations/gingr/src/mapping/retail.rs` explicitly promotes provider item fields into `domain::service::retail` values and reports provider-field errors.
- `integrations/gingr/src/dto/mod.rs` uses `ProviderSurface::NoDocumentedServiceDto` for endpoints without documented DTOs.

Provider mapping remains intentionally thin: retail has meaningful promotion logic; grooming/training DTO files are skeletons/documented surfaces. That is acceptable for architecture closeout but remains a production integration gap.

### 5. No vague common/types/util dump was introduced

Status: pass.

Repository search found no new `common`, `util`, `utils`, `helpers`, or `types` junk-drawer modules in Rust source. The service work uses semantic module names (`service`, `boarding`, `daycare`, `retail`, `storage::service_line`, `mapping`) rather than vague catch-alls.

### 6. Canonical tests use `domain::service::*` paths

Status: pass.

Evidence:

- `domain/tests/service_module_architecture.rs` imports `domain::{entities, money, policy, service}` and constructs contracts through `service::boarding`, `service::daycare`, `service::grooming`, `service::training`, and `service::retail`.
- The same test file contains a scanner that fails if non-compatibility tests import service lines through `domain::operations::<line>` shims.
- Search for `domain::operations::{boarding,daycare,grooming,training,retail}` and `operations::{boarding,daycare,grooming,training,retail}` found only compatibility comments in `domain/src/operations.rs`.

This is the right contract: canonical tests protect the semantic path, while temporary compatibility shims are explicitly marked as removable.

## Remaining production gaps

These are not blockers for the modularization closeout, but they should remain visible before calling the broader platform production-ready:

1. Remove or narrow `domain::operations` compatibility shims after all downstream callers have migrated. Today the shims are commented as temporary and are only retained for compatibility.
2. Continue decomposing large service modules. `domain::service::grooming` and `domain::service::training` are first-class homes but remain monolithic internally; several boarding facet files are still placeholders around types in `boarding/mod.rs`.
3. Move `CoreServiceContracts` out of `domain::operations` if it becomes a durable service aggregate rather than a compatibility-era cross-service contract.
4. Expand provider DTO/mapping coverage only where real Gingr payloads justify it. Retail has concrete promotion logic; grooming/training are documented skeletons rather than complete integrations.
5. Add live/provider-backed integration tests before production use. The current gates prove compile-time architecture, serialization, and semantic contracts, not end-to-end Gingr behavior against a live/sandbox account.
6. Add persistence migrations and operational rollout checks for the service-owned storage records before treating the storage boundary as production deployed.
7. Keep workflow executable side effects approval-gated. The app crate has orchestration types and tests, but production launch still needs actor authorization, audit durability, idempotency/replay protections, and live smoke coverage.

## Final closeout decision

Close the semantic modularization remediation as GO.

The codebase now reflects the intended domain shape strongly enough to proceed with downstream product/API work against `domain::service::*` and `app::*` orchestration. Do not market the system as production-operational until the production gaps above are addressed and verified with live or sandbox integration evidence.
