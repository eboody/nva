# `storage`

Operator translation: `storage` is the durable filing cabinet/reporting view for source-backed pet-resort facts. It can save records, stable codes, source references, and reviewed outcomes for audit or reporting, but it does not decide bookings, customer messages, payments, staffing, provider/PMS changes, or policy exceptions.

`storage` is the persistence and [projection](../docs/glossary-architecture-terms.md#projection) boundary for the pet-resort workspace — projection means a database/reporting-friendly view, not live decision authority. It does not own domain truth: [`domain`](../domain/README.md) owns the [semantic](../docs/glossary-architecture-terms.md#semantic) business models, invariants, and operating policies. This crate owns storage-shaped records, stable persisted code values, JSON codecs, and explicit [promotion/demotion](../docs/glossary-architecture-terms.md#promotion-demotion) paths between those records and semantic `domain::*` types.

Start at [`src/lib.rs`](./src/lib.rs). The public surface is intentionally small: [`storage::operations`](./src/operations.rs) contains cross-service records, codecs, and storage errors, while [`storage::service_line`](./src/service_line/mod.rs) contains service-line-specific contract wrappers and code tables. The service-line maintainer guide is [`src/service_line/README.md`](./src/service_line/README.md).

## README vs Rustdoc contract

This README is the storage-boundary wiki: use it to navigate persisted records, stable codes, codecs, and promotion/demotion ownership. Promotion/demotion here means validated conversion between storage/provider shapes and trusted business meaning. Keep it focused on where storage shapes live and why they are separate from domain truth and provider DTOs.

Executable storage examples belong in Rustdoc on [`src/lib.rs`](./src/lib.rs), [`src/operations.rs`](./src/operations.rs), and the service-line modules under [`src/service_line`](./src/service_line/mod.rs). Those examples should compile under `cargo test -p storage --doc` and demonstrate explicit conversion between storage records/codes and semantic `domain::*` values instead of copying unverified snippets into this README.

Non-coder glossary help: [`storage`](../docs/glossary-architecture-terms.md#storage) is the persisted projection and conversion boundary, while a [projection](../docs/glossary-architecture-terms.md#projection) or [read model](../docs/glossary-architecture-terms.md#read-model) is a reporting/review view and [outcome capture](../docs/glossary-workflow-state-terms.md#outcome-capture) is the staff-reviewed evidence loop. [Source refs](../docs/glossary-architecture-terms.md#source-ref-domainsourcerecordref) and [provenance](../docs/glossary-architecture-terms.md#provenance-domainsourceprovenance) explain why stored records can cite evidence without becoming the source of record.

## Module navigation

- [`src/lib.rs`](./src/lib.rs) declares the crate-level boundary: `domain` owns language; `storage` owns persistence records, stable codes, codecs, and conversion between storage records and domain types. It exposes [`storage::operations`](./src/operations.rs) and [`storage::service_line`](./src/service_line/mod.rs), and re-exports `storage::operations::{CodecError, RecordKind, Result}`.
- [`src/operations.rs`](./src/operations.rs) owns the cross-service storage records: `PetResortPortfolioRecord`, `ServiceOfferingRecord`, `CoreServiceContractsRecord`, `TechnologyEcosystemRecord`, `ManagerDailyBriefOutcomeRecord`, `DataQualityHygieneOutcomeRecord`, and the first Data-Quality Hygiene read-model proof records (`DataQualityIssueRecord`, `DataQualitySourceImportRunRecord`, `DataQualitySyncGapRecord`, `SourceQualityBacklogRow`, `ImportFreshnessRow`). It also owns `Error`, `CodecError`, `RecordKind`, `ShapeMismatchReason`, and `StorageField`.
- [`src/service_line/mod.rs`](./src/service_line/mod.rs) exposes the service-line storage modules: [`boarding`](./src/service_line/boarding.rs), [`daycare`](./src/service_line/daycare.rs), [`grooming`](./src/service_line/grooming.rs), [`retail`](./src/service_line/retail.rs), and [`training`](./src/service_line/training.rs).
- [`src/service_line/boarding.rs`](./src/service_line/boarding.rs) stores `domain::boarding::Contract` behind `boarding::ContractRecord` and maps boarding service-offering codes to `domain::operations::lodging_offer::{Accommodation, CareFeature, AddOn}`.
- [`src/service_line/daycare.rs`](./src/service_line/daycare.rs) stores `domain::daycare::Contract` behind `daycare::ContractRecord` and maps daycare catalog codes to `domain::operations::{DaycareFormat, DaycareEligibilityRule}`.
- [`src/service_line/grooming.rs`](./src/service_line/grooming.rs) stores `domain::grooming::Contract` behind `grooming::ContractRecord`, maps `grooming::ServiceCode` to `domain::grooming::Service`, and validates `grooming::StoredCadenceWeeks` before it becomes `domain::grooming::rebooking::CadenceWeeks`.
- [`src/service_line/retail.rs`](./src/service_line/retail.rs) stores `domain::retail::Contract` behind `retail::ContractRecord` and maps `retail::{PartnerCode, ProductCategoryCode}` to `domain::retail::{Partner, product::Category}`.
- [`src/service_line/training.rs`](./src/service_line/training.rs) stores `domain::training::Contract` behind `training::ContractRecord`, maps `training::ProgramRecord` to `domain::training::Program`, and validates `training::StoredProgramDurationWeeks` before it becomes `domain::training::program::DurationWeeks`.

## What this crate owns

`storage` owns the boundary facts required to persist and rehydrate domain concepts safely:

1. Stable code enums such as [`OperatorCode`](./src/operations.rs), [`ServiceOfferingKindCode`](./src/operations.rs), [`CoreOperatingSystemCode`](./src/operations.rs), and service-line code tables under [`src/service_line`](./src/service_line/mod.rs). These are storage codes, not replacement domain vocabularies.
2. Record shapes such as [`PetResortPortfolioRecord`](./src/operations.rs), [`ServiceOfferingRecord`](./src/operations.rs), [`CoreServiceContractsRecord`](./src/operations.rs), [`TechnologyEcosystemRecord`](./src/operations.rs), and [`ManagerDailyBriefOutcomeRecord`](./src/operations.rs). These structs say how data is serialized or projected at the storage boundary.
3. JSON codecs on records with `decode_json` and `encode_json` methods in [`src/operations.rs`](./src/operations.rs). Codec failures are reported through [`CodecError`](./src/operations.rs) and composed into [`Error`](./src/operations.rs).
4. Validated storage scalar wrappers such as [`StoredResortCount`](./src/operations.rs), [`StoredBrandName`](./src/operations.rs), [`StoredManagerDailyBriefLaborMinutes`](./src/operations.rs), [`grooming::StoredCadenceWeeks`](./src/service_line/grooming.rs), and [`training::StoredProgramDurationWeeks`](./src/service_line/training.rs). These keep invalid persisted values from silently becoming domain values.
5. Explicit conversion relationships (`From` and `TryFrom`) between storage records/codes and semantic `domain::*` types. Fallible conversions preserve boundary evidence through [`Error::InvalidDomainValue`](./src/operations.rs) or [`Error::StorageShapeMismatch`](./src/operations.rs).

`storage` does not send customer messages, mutate Gingr, book reservations, assign staff, define domain policy, or decide customer-facing outcomes. Those responsibilities belong to `app`, `integrations/gingr`, and `domain` surfaces linked below.

## Type/module map

| Concept | Public type/module path | Defined in | Conversion or relationship |
| --- | --- | --- | --- |
| Crate surface | `storage` | [`src/lib.rs`](./src/lib.rs) | Exposes `storage::operations` and `storage::service_line`; re-exports `CodecError`, `RecordKind`, and `Result`. |
| Cross-service operations module | `storage::operations` | [`src/operations.rs`](./src/operations.rs) | Owns records, codecs, errors, and conversion implementations. |
| Storage result alias | `storage::operations::Result<T>` | [`src/operations.rs`](./src/operations.rs) | Alias for `std::result::Result<T, storage::operations::Error>`. |
| Storage error | `storage::operations::Error` | [`src/operations.rs`](./src/operations.rs) | Composes `CodecError`, shape mismatch, and invalid-domain-value boundary failures. |
| Codec error | `storage::operations::CodecError` | [`src/operations.rs`](./src/operations.rs) | Wraps `serde_json` encode/decode failures from record codecs. |
| Shape/error classifiers | `storage::operations::{RecordKind, ShapeMismatchReason, StorageField}` | [`src/operations.rs`](./src/operations.rs) | Identify malformed record kinds and specific invalid persisted fields. |
| Source provenance | `storage::operations::StoredSourceRecordRef` | [`src/operations.rs`](./src/operations.rs) | Carries source-system, record, observation, and adapter-version evidence for projected records. |
| Manager daily brief outcome projection | `storage::operations::ManagerDailyBriefOutcomeRecord` | [`src/operations.rs`](./src/operations.rs) | JSON record with labor minutes, outcome, actor, source refs, reporting group, and savings evidence. |
| Manager daily brief codes | `storage::operations::{ManagerDailyBriefOutcomeCode, ManagerDailyBriefPersonaCode, ManagerDailyBriefActionKindCode}` | [`src/operations.rs`](./src/operations.rs) | Stable stored classifications used by `ManagerDailyBriefOutcomeRecord`. |
| Manager daily brief labor scalar | `storage::operations::StoredManagerDailyBriefLaborMinutes` | [`src/operations.rs`](./src/operations.rs) | Validates non-zero before storage projection accepts labor-minute evidence. |
| Portfolio record | `storage::operations::PetResortPortfolioRecord` | [`src/operations.rs`](./src/operations.rs) | Fallibly converts to/from `domain::operations::pet_resort::Portfolio`. |
| Portfolio codes | `storage::operations::{OperatorCode, PortfolioStructureCode, BusinessLineCode, PetResortBrandCode}` | [`src/operations.rs`](./src/operations.rs) | Bidirectional maps to `domain::operations::pet_resort::{Operator, PortfolioStructure, BusinessLine, Brand}`. |
| Portfolio record variants/scalars | `storage::operations::{PetResortBrandRecord, StoredResortCount, StoredResortCountError, StoredBrandName}` | [`src/operations.rs`](./src/operations.rs) | Preserve persisted brand/resort-count shape and fallibly promote into domain brand/name/count values. |
| Service offering record | `storage::operations::ServiceOfferingRecord` | [`src/operations.rs`](./src/operations.rs) | Fallibly converts to/from `domain::operations::ServiceOffering`; rejects cross-variant optional fields. |
| Service offering kind code | `storage::operations::ServiceOfferingKindCode` | [`src/operations.rs`](./src/operations.rs) | Tags the storage variant for boarding, daycare, grooming, training, or retail partner products. |
| Core service contracts record | `storage::operations::CoreServiceContractsRecord` | [`src/operations.rs`](./src/operations.rs) | Converts to/from `domain::operations::service_core::ServiceContracts` using service-line `ContractRecord` wrappers. |
| Technology ecosystem record | `storage::operations::TechnologyEcosystemRecord` | [`src/operations.rs`](./src/operations.rs) | Converts to/from `domain::operations::TechnologyEcosystem`. |
| Technology ecosystem codes | `storage::operations::{CoreOperatingSystemCode, DataAccessPatternCode, AdjacentSystemCode}` | [`src/operations.rs`](./src/operations.rs) | Bidirectional maps to `domain::operations::{service_core::OperatingSystem, DataAccessPattern, AdjacentSystem}`. |
| Service-line storage module | `storage::service_line` | [`src/service_line/mod.rs`](./src/service_line/mod.rs) | Public namespace for service-line records and code tables. |
| Boarding storage records/codes | `storage::service_line::boarding::{ContractRecord, AccommodationCode, CareFeatureCode, AddOnCode}` | [`src/service_line/boarding.rs`](./src/service_line/boarding.rs) | Wraps `domain::boarding::Contract`; maps codes to `domain::operations::lodging_offer` values. |
| Daycare storage records/codes | `storage::service_line::daycare::{ContractRecord, FormatCode, EligibilityRuleCode}` | [`src/service_line/daycare.rs`](./src/service_line/daycare.rs) | Wraps `domain::daycare::Contract`; maps codes to `domain::operations::{DaycareFormat, DaycareEligibilityRule}`. |
| Grooming storage records/codes | `storage::service_line::grooming::{ContractRecord, ServiceCode, StoredCadenceWeeks, StoredCadenceWeeksError}` | [`src/service_line/grooming.rs`](./src/service_line/grooming.rs) | Wraps `domain::grooming::Contract`; maps service/cadence storage to `domain::grooming` values. |
| Retail storage records/codes | `storage::service_line::retail::{ContractRecord, PartnerCode, ProductCategoryCode}` | [`src/service_line/retail.rs`](./src/service_line/retail.rs) | Wraps `domain::retail::Contract`; maps partner/category codes to `domain::retail` values. |
| Training storage records/codes | `storage::service_line::training::{ContractRecord, ProgramRecord, StoredProgramDurationWeeks, StoredProgramDurationWeeksError}` | [`src/service_line/training.rs`](./src/service_line/training.rs) | Wraps `domain::training::Contract`; maps program/duration storage to `domain::training` values. |

## Record, code, codec, and error relationships

The storage crate has four recurring type families:

- `*Record` types are persistence shapes. Examples: [`PetResortPortfolioRecord`](./src/operations.rs), [`ServiceOfferingRecord`](./src/operations.rs), [`CoreServiceContractsRecord`](./src/operations.rs), [`TechnologyEcosystemRecord`](./src/operations.rs), [`ManagerDailyBriefOutcomeRecord`](./src/operations.rs), and service-line `ContractRecord` wrappers under [`src/service_line`](./src/service_line/mod.rs).
- `*Code` types are stable serialized classifications. Examples: [`BusinessLineCode`](./src/operations.rs), [`PetResortBrandCode`](./src/operations.rs), [`ServiceOfferingKindCode`](./src/operations.rs), [`DataAccessPatternCode`](./src/operations.rs), [`boarding::AccommodationCode`](./src/service_line/boarding.rs), and [`retail::ProductCategoryCode`](./src/service_line/retail.rs).
- `Stored*` types are storage-side validated scalars. Examples: [`StoredResortCount`](./src/operations.rs), [`StoredBrandName`](./src/operations.rs), [`StoredManagerDailyBriefLaborMinutes`](./src/operations.rs), [`grooming::StoredCadenceWeeks`](./src/service_line/grooming.rs), and [`training::StoredProgramDurationWeeks`](./src/service_line/training.rs).
- `*Error` types explain boundary failures. [`Error`](./src/operations.rs) is the crate-level operations error; [`CodecError`](./src/operations.rs) wraps JSON codec failures; service-line scalar errors such as [`grooming::StoredCadenceWeeksError`](./src/service_line/grooming.rs) and [`training::StoredProgramDurationWeeksError`](./src/service_line/training.rs) identify local validation failures.

The conversion rule is: storage values are promoted into domain values only through explicit `From`/`TryFrom` implementations, and domain values are demoted back into storage records/codes through the matching reverse implementation when a persisted form exists.

Important examples:

- [`PetResortPortfolioRecord`](./src/operations.rs) fallibly promotes into `domain::operations::pet_resort::Portfolio` and demotes back from that domain type. `StoredResortCount` maps through `domain::operations::ResortCount`; `PetResortBrandRecord::Other` maps through `domain::location::Name`.
- [`ServiceOfferingRecord`](./src/operations.rs) is a tagged storage shape for `domain::operations::ServiceOffering`. Its `service_kind` field chooses the domain variant; `ensure_empty_cross_variant_fields` rejects records that carry fields from a different service line.
- [`CoreServiceContractsRecord`](./src/operations.rs) is the persisted bundle for `domain::operations::service_core::ServiceContracts`. The per-line fields use [`boarding::ContractRecord`](./src/service_line/boarding.rs), [`daycare::ContractRecord`](./src/service_line/daycare.rs), [`grooming::ContractRecord`](./src/service_line/grooming.rs), [`training::ContractRecord`](./src/service_line/training.rs), and [`retail::ContractRecord`](./src/service_line/retail.rs).
- [`TechnologyEcosystemRecord`](./src/operations.rs) maps to `domain::operations::TechnologyEcosystem` through code maps for `service_core::OperatingSystem`, `DataAccessPattern`, and `AdjacentSystem`.
- [`ManagerDailyBriefOutcomeRecord`](./src/operations.rs) is currently a storage projection with JSON codecs, labor-minute validation, `actual_minutes_saved`, and `reporting_group`. It records automation outcome evidence and source references; it is not a domain policy object.

## Provider DTO and projection boundary

`integrations/gingr` is the provider boundary; `storage` is the persisted projection boundary. Keep those roles separate:

- Provider DTOs and endpoint facts live under [`integrations/gingr/src`](../integrations/gingr/src). Retail has DTO and mapping files in [`integrations/gingr/src/dto/retail.rs`](../integrations/gingr/src/dto/retail.rs) and [`integrations/gingr/src/mapping/retail.rs`](../integrations/gingr/src/mapping/retail.rs).
- Provider surface gaps for catalog data are represented under [`integrations/gingr/src/endpoint/catalog.rs`](../integrations/gingr/src/endpoint/catalog.rs), including documented semantic gaps for retail, training, and grooming.
- Storage records should not mirror every provider payload. Promote provider DTOs into semantic domain concepts or explicit storage records only when the repo needs a durable projection, reviewable automation evidence, or stable source-data normalization.

## Labor-cost and automation role

The labor-cost-reduction contribution of `storage` is normalization and review safety:

1. [`ServiceOfferingRecord`](./src/operations.rs) turns service catalog data into a typed `domain::operations::ServiceOffering` variant, so downstream app code does not ask a manager to interpret raw service names or mixed optional fields during exception triage.
2. [`CoreServiceContractsRecord`](./src/operations.rs) stores a location's service-line policy bundle in one normalized shape, reducing handoffs between boarding, daycare, grooming, training, and retail rule lookups.
3. [`ManagerDailyBriefOutcomeRecord`](./src/operations.rs) stores before/actual labor minutes, source references, action kind, owner persona, and reporting group so automation can produce evidence about time saved and data-quality issues.
4. Validated storage scalars and shape mismatch errors turn bad source data into typed failures (`StorageField`, `RecordKind`, `ShapeMismatchReason`) instead of quiet fallthrough or ad hoc manual investigation.
5. Because `storage` keeps provider DTOs and domain policy separate, reviewers can inspect a small conversion surface before trusting automation that consumes persisted records.

## Cross-crate relationships

- [`domain`](../domain/README.md) owns semantic truth. `storage` converts to and from `domain::operations`, `domain::boarding`, `domain::daycare`, `domain::grooming`, `domain::retail`, `domain::training`, and supporting types such as `domain::entities::LocationId` and `domain::location::Name`.
- [`domain/src/operations.rs`](../domain/src/operations.rs) defines `domain::operations::pet_resort::Portfolio`, `domain::operations::ServiceOffering`, `domain::operations::service_core::ServiceContracts`, and `domain::operations::TechnologyEcosystem`, which are the primary domain types promoted from storage records.
- Service-line domain guides live in [`domain/src/boarding/README.md`](../domain/src/boarding/README.md), [`domain/src/daycare/README.md`](../domain/src/daycare/README.md), [`domain/src/grooming/README.md`](../domain/src/grooming/README.md), [`domain/src/retail/README.md`](../domain/src/retail/README.md), and [`domain/src/training/README.md`](../domain/src/training/README.md). Storage service-line wrappers preserve those contracts; they do not replace service-line domain policy.
- [`app/src`](../app/src) contains workflows such as [`booking_triage.rs`](../app/src/booking_triage.rs), [`daily_update.rs`](../app/src/daily_update.rs), [`checkout_completion.rs`](../app/src/checkout_completion.rs), [`crm_retention.rs`](../app/src/crm_retention.rs), and [`manager_daily_brief.rs`](../app/src/manager_daily_brief.rs). App code should consume domain decisions and storage-normalized records instead of defining new local persisted code tables.
- [`integrations/gingr/src`](../integrations/gingr/src) owns provider DTOs, endpoints, mappings, transport, and webhook parsing. When a Gingr payload needs durable normalized form, map provider DTOs through domain/storage conversion surfaces rather than treating provider JSON as domain truth.
- Executable storage contracts live in [`storage/tests`](./tests): [`operations_storage_contracts.rs`](./tests/operations_storage_contracts.rs), [`core_service_contract_storage.rs`](./tests/core_service_contract_storage.rs), [`manager_daily_brief_outcome_storage.rs`](./tests/manager_daily_brief_outcome_storage.rs), [`data_quality_hygiene_outcome_storage.rs`](./tests/data_quality_hygiene_outcome_storage.rs), [`data_quality_read_model_storage.rs`](./tests/data_quality_read_model_storage.rs), and [`mvp_migration_contract.rs`](./tests/mvp_migration_contract.rs).

## Data-Quality Hygiene persistence/read-model proof

The first owned API persistence proof is intentionally narrow and safe:

- [`../migrations/0002_data_quality_read_models.sql`](../migrations/0002_data_quality_read_models.sql) adds `source_import_runs`, `source_quality_issues`, and `sync_gaps`, plus BI-safe views `source_quality_backlog`, `data_quality_hygiene_labor_outcomes`, `audit_lineage`, and `import_freshness`.
- `DataQualityIssueRecord`, `DataQualitySourceImportRunRecord`, and `DataQualitySyncGapRecord` are storage-shaped source/import evidence. They preserve source refs, issue/review/labor dimensions, redaction posture, and workflow lineage without embedding raw provider payloads.
- `SourceQualityBacklogRow` and `ImportFreshnessRow` are read-model DTOs that BI can consume for location, source, issue kind, severity, freshness, sensitivity, workflow-blocking status, resolution state, latest outcome lineage, projection version, and caveats such as `raw_payload_redacted`, `review_pending`, `source_stale`, and `live_side_effects_disabled`.
- Runtime HTTP handlers still use the in-memory `WorkflowRepository` adapter until a later Postgres adapter is explicitly wired. The durable proof is the migration contract plus storage codecs/read-model projection functions; it does not claim production imports, live provider writes, customer sends, payment movement, schedule changes, or medical/safety decisions.

## Maintainer notes

- Add a record here when the project needs a durable persisted shape or projection. Do not add a storage record merely to rename a domain type.
- Add a `*Code` enum when the stored representation needs stable serialized values. Keep richer business decisions in `domain::*` modules.
- Keep fallible boundary validation explicit. If a stored scalar can be invalid, represent it with a `Stored*` newtype and surface failures through a local error or `storage::operations::Error`.
- Keep provider DTOs outside this crate. `storage` should store normalized records and source references, not raw Gingr endpoint payloads unless a future task explicitly introduces such a projection.
- Preserve semantic paths in docs and code. `storage::operations::ServiceOfferingRecord`, `domain::operations::ServiceOffering`, and a Gingr service DTO are related boundary artifacts, not interchangeable names.
