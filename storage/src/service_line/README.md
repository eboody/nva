# `storage::service_line`

`storage::service_line` is the storage crate's boundary for service-line-owned persistence records and code tables. The domain crate owns the boarding, daycare, grooming, retail, training, and cross-service operating concepts; this module owns the storage-shaped wrappers and snake_case-compatible code values that promote into, and demote from, those domain concepts.

Start at [`mod.rs`](./mod.rs). It exposes [`storage::service_line::boarding`](./boarding.rs), [`daycare`](./daycare.rs), [`grooming`](./grooming.rs), [`retail`](./retail.rs), and [`training`](./training.rs). Those files are intentionally small: each keeps provider/database-facing codes at the storage boundary while preserving explicit conversions into semantic `domain::*` paths.

## Module navigation

- [`boarding.rs`](./boarding.rs) defines [`storage::service_line::boarding::ContractRecord`](./boarding.rs) plus code tables for `domain::operations::ServiceOffering::Boarding`: [`AccommodationCode`](./boarding.rs), [`CareFeatureCode`](./boarding.rs), and [`AddOnCode`](./boarding.rs). The codes convert to and from `domain::operations::lodging_offer::{Accommodation, CareFeature, AddOn}`.
- [`daycare.rs`](./daycare.rs) defines [`storage::service_line::daycare::ContractRecord`](./daycare.rs), [`FormatCode`](./daycare.rs), and [`EligibilityRuleCode`](./daycare.rs). The code values map to `domain::operations::DaycareFormat` and `domain::operations::DaycareEligibilityRule`, which are service-catalog values rather than replacements for the richer `domain::daycare` policy modules.
- [`grooming.rs`](./grooming.rs) defines [`storage::service_line::grooming::ContractRecord`](./grooming.rs), [`ServiceCode`](./grooming.rs), [`StoredCadenceWeeks`](./grooming.rs), and [`StoredCadenceWeeksError`](./grooming.rs). `ServiceCode` maps to `domain::grooming::Service`; `StoredCadenceWeeks` validates the positive-week storage shape used when `domain::grooming::rebooking::Cadence::EveryWeeks` crosses the storage boundary.
- [`retail.rs`](./retail.rs) defines [`storage::service_line::retail::ContractRecord`](./retail.rs), [`PartnerCode`](./retail.rs), and [`ProductCategoryCode`](./retail.rs). These values map to `domain::retail::Partner` and `domain::retail::product::Category`, leaving raw provider retail item payloads to the Gingr integration layer.
- [`training.rs`](./training.rs) defines [`storage::service_line::training::ContractRecord`](./training.rs), [`ProgramRecord`](./training.rs), [`StoredProgramDurationWeeks`](./training.rs), and [`StoredProgramDurationWeeksError`](./training.rs). `ProgramRecord` is the storage form of `domain::training::Program`; the stored duration newtype preserves the positive-week invariant required by `domain::training::program::DurationWeeks`.

## Type/module map

| Concept | Public type/module path | Defined in |
| --- | --- | --- |
| Service-line storage module surface | `storage::service_line` | [`mod.rs`](./mod.rs) |
| Boarding contract wrapper | `storage::service_line::boarding::ContractRecord` | [`boarding.rs`](./boarding.rs) |
| Boarding service-offering codes | `storage::service_line::boarding::AccommodationCode`, `CareFeatureCode`, `AddOnCode` | [`boarding.rs`](./boarding.rs) |
| Daycare contract wrapper | `storage::service_line::daycare::ContractRecord` | [`daycare.rs`](./daycare.rs) |
| Daycare service-offering codes | `storage::service_line::daycare::FormatCode`, `EligibilityRuleCode` | [`daycare.rs`](./daycare.rs) |
| Grooming contract wrapper | `storage::service_line::grooming::ContractRecord` | [`grooming.rs`](./grooming.rs) |
| Grooming service and cadence storage | `storage::service_line::grooming::ServiceCode`, `StoredCadenceWeeks`, `StoredCadenceWeeksError` | [`grooming.rs`](./grooming.rs) |
| Retail contract wrapper | `storage::service_line::retail::ContractRecord` | [`retail.rs`](./retail.rs) |
| Retail partner/category codes | `storage::service_line::retail::PartnerCode`, `ProductCategoryCode` | [`retail.rs`](./retail.rs) |
| Training contract wrapper | `storage::service_line::training::ContractRecord` | [`training.rs`](./training.rs) |
| Training program and duration storage | `storage::service_line::training::ProgramRecord`, `StoredProgramDurationWeeks`, `StoredProgramDurationWeeksError` | [`training.rs`](./training.rs) |
| Cross-service offering record using these codes | `storage::operations::ServiceOfferingRecord`, `ServiceOfferingKindCode` | [`../operations.rs`](../operations.rs) |
| Core service-contract bundle using these wrappers | `storage::operations::CoreServiceContractsRecord` | [`../operations.rs`](../operations.rs) |
| Storage errors for validated service-line values | `storage::operations::Error`, `StorageField` | [`../operations.rs`](../operations.rs) |

## Record shapes and domain mappings

`ContractRecord` appears in each service-line file as a transparent wrapper around that service line's domain contract:

- [`boarding::ContractRecord`](./boarding.rs) wraps `domain::boarding::Contract` from [`domain/src/boarding/mod.rs`](../../../domain/src/boarding/mod.rs).
- [`daycare::ContractRecord`](./daycare.rs) wraps `domain::daycare::Contract` from [`domain/src/daycare/mod.rs`](../../../domain/src/daycare/mod.rs).
- [`grooming::ContractRecord`](./grooming.rs) wraps `domain::grooming::Contract` from [`domain/src/grooming/mod.rs`](../../../domain/src/grooming/mod.rs).
- [`retail::ContractRecord`](./retail.rs) wraps `domain::retail::Contract` from [`domain/src/retail/mod.rs`](../../../domain/src/retail/mod.rs).
- [`training::ContractRecord`](./training.rs) wraps `domain::training::Contract` from [`domain/src/training/mod.rs`](../../../domain/src/training/mod.rs).

Those wrappers are used by [`storage::operations::CoreServiceContractsRecord`](../operations.rs), which stores one `location_id` plus boarding/daycare/grooming/training/retail contracts and converts to `domain::operations::service_core::ServiceContracts`. This is the durable storage shape for a location's core service-line policy bundle.

Service catalog rows use [`storage::operations::ServiceOfferingRecord`](../operations.rs). It stores one `service_kind` plus service-line-specific optional fields and vectors:

- Boarding rows require `boarding_accommodation`; they may also carry `boarding_included_care` and `boarding_add_ons`, all backed by [`boarding.rs`](./boarding.rs) code enums.
- Daycare rows require `daycare_format` and may carry `daycare_eligibility_rules`, backed by [`daycare.rs`](./daycare.rs) code enums.
- Grooming rows require `grooming_service`; `grooming_cadence_weeks` is present only when the domain cadence is `domain::grooming::rebooking::Cadence::EveryWeeks`.
- Training rows require `training_program`, backed by [`training::ProgramRecord`](./training.rs).
- Retail partner-product rows require `retail_partner` and `retail_product_category`, backed by [`retail.rs`](./retail.rs) code enums.

`ServiceOfferingRecord::ensure_empty_cross_variant_fields` in [`../operations.rs`](../operations.rs) rejects fields that belong to a different `ServiceOfferingKindCode`. That check is the storage-side evidence that a grooming row cannot accidentally carry boarding or retail fields just because the JSON record has optional columns for every service line.

## Labor-cost and automation evidence

The labor-cost-reduction role here is source-data normalization and safer automation evidence, not customer-facing workflow execution.

1. Service-line contracts round-trip through [`CoreServiceContractsRecord`](../operations.rs), so application workflows can read one normalized location policy bundle instead of reassembling boarding, daycare, grooming, training, and retail rules from free-text provider settings.
2. Service offerings cross the storage boundary through explicit code maps. That means staff/admin dashboards can depend on `domain::operations::ServiceOffering` variants instead of asking managers to interpret raw service names or database strings each time an exception is triaged.
3. Validated storage newtypes such as [`grooming::StoredCadenceWeeks`](./grooming.rs) and [`training::StoredProgramDurationWeeks`](./training.rs) reject zero-week values before they become domain cadences or program durations. The resulting `storage::operations::Error::InvalidDomainValue` values identify `StorageField::GroomingCadenceWeeks` or `StorageField::TrainingProgramDurationWeeks` in [`../operations.rs`](../operations.rs), which supports targeted data-quality remediation rather than broad manual investigation.
4. Cross-variant shape checks in [`ServiceOfferingRecord`](../operations.rs) turn malformed service-catalog rows into typed storage errors. That is useful automation evidence for exception queues and manager review: a data issue can be reported as a storage-shape mismatch instead of silently producing an incorrect service offer.
5. The storage module does not send customer messages, assign staff, mutate provider systems, place vendor orders, or book reservations. It supplies normalized records and conversion evidence that app and integration layers can use for reviewable automation.

## Cross-crate relationships

- The storage crate exposes this module from [`storage/src/lib.rs`](../lib.rs) as `pub mod service_line`.
- [`storage::operations`](../operations.rs) consumes these service-line records from [`ServiceOfferingRecord`](../operations.rs) and [`CoreServiceContractsRecord`](../operations.rs). Its tests cover service-offering shape checks, service-line promotion/demotion, and core-contract JSON round trips in [`storage/tests/operations_storage_contracts.rs`](../../tests/operations_storage_contracts.rs) and [`storage/tests/core_service_contract_storage.rs`](../../tests/core_service_contract_storage.rs).
- `domain::operations::ServiceOffering` and `domain::operations::service_core::ServiceContracts` live in [`domain/src/operations.rs`](../../../domain/src/operations.rs). They are the domain concepts that this storage module serializes and deserializes.
- Boarding domain policy lives in [`domain/src/boarding/mod.rs`](../../../domain/src/boarding/mod.rs) with a maintainer guide at [`domain/src/boarding/README.md`](../../../domain/src/boarding/README.md). `storage::service_line::boarding` stores the contract wrapper and `domain::operations::lodging_offer` codes; it does not own boarding capacity, deposit, care, handoff, or upsell policy.
- Daycare domain policy lives in [`domain/src/daycare/mod.rs`](../../../domain/src/daycare/mod.rs) with a maintainer guide at [`domain/src/daycare/README.md`](../../../domain/src/daycare/README.md). `storage::service_line::daycare` stores catalog format/eligibility codes; it does not own daycare attendance, coverage, eligibility, assignment, incident, front-desk, or package-opportunity decisions.
- Grooming domain policy lives in [`domain/src/grooming/mod.rs`](../../../domain/src/grooming/mod.rs) with a maintainer guide at [`domain/src/grooming/README.md`](../../../domain/src/grooming/README.md). `storage::service_line::grooming` stores service codes and cadence weeks; grooming estimation, no-show, rebooking, reminders, and history remain in `domain::grooming`.
- Retail domain policy lives in [`domain/src/retail/mod.rs`](../../../domain/src/retail/mod.rs) with a maintainer guide at [`domain/src/retail/README.md`](../../../domain/src/retail/README.md). `storage::service_line::retail` stores partner/category codes; POS, inventory, recommendation, reorder, and vendor semantics remain in `domain::retail`.
- Training domain policy lives in [`domain/src/training/mod.rs`](../../../domain/src/training/mod.rs) with a maintainer guide at [`domain/src/training/README.md`](../../../domain/src/training/README.md). `storage::service_line::training` stores program records and positive duration weeks; assignment, progress, outcome, package, and follow-up policy remain in `domain::training`.
- App workflows such as booking triage, daily updates, checkout completion, CRM retention, and manager daily briefs live under [`app/src`](../../../app/src). They should consume domain decisions and storage-normalized records rather than adding new service-line storage codes locally.
- Gingr provider-boundary code lives under [`integrations/gingr/src`](../../../integrations/gingr/src). Retail has provider DTO and mapping coverage in [`integrations/gingr/src/dto/retail.rs`](../../../integrations/gingr/src/dto/retail.rs) and [`integrations/gingr/src/mapping/retail.rs`](../../../integrations/gingr/src/mapping/retail.rs). Training and grooming currently document `ProviderSurface::NoDocumentedServiceDto` in [`integrations/gingr/src/dto/training.rs`](../../../integrations/gingr/src/dto/training.rs) and [`integrations/gingr/src/dto/grooming.rs`](../../../integrations/gingr/src/dto/grooming.rs), while [`integrations/gingr/src/endpoint/catalog.rs`](../../../integrations/gingr/src/endpoint/catalog.rs) lists `retail`, `training`, and `grooming` in `semantic_mapping_gaps`.

## Maintainer notes

- Add a service-line code here only when the storage boundary needs a stable persisted representation. Do not duplicate richer domain policy enums just to shorten call sites.
- Keep conversions bidirectional and explicit. A new `storage::service_line::*` code should show exactly which `domain::*` concept it maps to, and fallible conversions should report the relevant `storage::operations::StorageField`.
- Keep cross-service record assembly in [`storage::operations`](../operations.rs). This module owns the service-line code tables and contract wrappers; `storage::operations::ServiceOfferingRecord` owns the tagged record shape that prevents cross-variant field leakage.
- Preserve semantic paths in prose and code. `storage::service_line::training::ProgramRecord`, `domain::training::Program`, and `domain::operations::ServiceOffering::Training` are related, but they are not interchangeable names for the same boundary.
