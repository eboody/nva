# `domain::grooming`

`domain::grooming` is the domain crate's model for grooming-service policy, estimation, service history, rebooking, reminder planning, and no-show handling. It owns concepts that should not be flattened into calendar notes or provider service names: grooming services, breed/coat duration evidence, groomer-history estimates, review requirements, style/history notes, no-show deposit decisions, rebooking cadence, reminder send boundaries, and the location-level grooming contract.

Start at [`mod.rs`](./mod.rs). The module is implemented in one file with nested semantic modules such as `domain::grooming::breed_coat`, `domain::grooming::history`, `domain::grooming::rebooking`, and `domain::grooming::reminder`; keep those paths visible because several nested modules intentionally expose generic leaves like `Policy`, `Decision`, and `Rule`.

## Module navigation

- [`mod.rs`](./mod.rs) defines the top-level grooming vocabulary: [`domain::grooming::Service`](./mod.rs), [`AppointmentMinutes`](./mod.rs), [`HistoryRequirement`](./mod.rs), [`EstimationRequest`](./mod.rs), [`DurationEstimate`](./mod.rs), [`EstimationPolicy`](./mod.rs), and [`Contract`](./mod.rs).
- `domain::grooming::calendar` in [`mod.rs`](./mod.rs) defines [`calendar::Policy`](./mod.rs), which distinguishes any-qualified-groomer, groomer-specific, and first-available-with-manager-override calendar behavior.
- `domain::grooming::breed_coat` in [`mod.rs`](./mod.rs) defines [`BreedCategory`](./mod.rs), [`CoatCondition`](./mod.rs), and [`TimeEstimate`](./mod.rs). `EstimationPolicy::estimate` uses those estimates before falling back to a positive default.
- `domain::grooming::no_show` in [`mod.rs`](./mod.rs) defines [`Rule`](./mod.rs), [`History`](./mod.rs), [`Decision`](./mod.rs), and [`Policy`](./mod.rs). `Policy::evaluate` turns repeat no-show/late-cancel behavior into deposit or manager-review gates.
- `domain::grooming::history` in [`mod.rs`](./mod.rs) defines service-history evidence: [`ServiceHistoryEntry`](./mod.rs), [`ServiceOutcome`](./mod.rs), [`ApprovalState`](./mod.rs), [`CareReference`](./mod.rs), and [`style_note::StyleNote`](./mod.rs). History entries can carry duration estimates and review/care references used by later estimation.
- `domain::grooming::rebooking` in [`mod.rs`](./mod.rs) defines cadence and recommendation logic: [`CadenceWeeks`](./mod.rs), [`OrdinaryCadenceWeeks`](./mod.rs), [`Cadence`](./mod.rs), [`Status`](./mod.rs), [`Rationale`](./mod.rs), [`Recommendation`](./mod.rs), and [`Policy`](./mod.rs).
- `domain::grooming::reminder` in [`mod.rs`](./mod.rs) defines reminder planning: [`Rule`](./mod.rs), [`Kind`](./mod.rs), [`Consent`](./mod.rs), [`SendBoundary`](./mod.rs), [`Plan`](./mod.rs), and [`Policy`](./mod.rs). `Plan::customer_message_gate` keeps customer-message approval explicit.
- `domain::grooming::appointment` and `domain::grooming::duration_estimate` in [`mod.rs`](./mod.rs) are public vocabulary modules that re-export the appointment request and duration-estimate decision surface without erasing the `domain::grooming` namespace.

## Type/module map

| Concept | Public type/module path | Defined in |
| --- | --- | --- |
| Grooming service kind | `domain::grooming::Service` | [`mod.rs`](./mod.rs) |
| Positive appointment duration | `domain::grooming::AppointmentMinutes`, `AppointmentMinutesError` | [`mod.rs`](./mod.rs) |
| Calendar ownership policy | `domain::grooming::calendar::Policy` | [`mod.rs`](./mod.rs) |
| Breed/coat duration evidence | `domain::grooming::breed_coat::BreedCategory`, `CoatCondition`, `TimeEstimate` | [`mod.rs`](./mod.rs) |
| Estimation request/decision | `domain::grooming::EstimationRequest`, `DurationEstimate`, `EstimationPolicy` | [`mod.rs`](./mod.rs) |
| Estimate provenance and review | `domain::grooming::EstimateBasis`, `EstimateConfidence`, `ReviewRequirement` | [`mod.rs`](./mod.rs) |
| No-show policy | `domain::grooming::no_show::Policy`, `Rule`, `History`, `Decision` | [`mod.rs`](./mod.rs) |
| Grooming service history | `domain::grooming::history::ServiceHistoryEntry`, `ServiceOutcome`, `ApprovalState`, `CareReference` | [`mod.rs`](./mod.rs) |
| Grooming style note | `domain::grooming::history::style_note::StyleNote` | [`mod.rs`](./mod.rs) |
| Rebooking cadence and status | `domain::grooming::rebooking::Cadence`, `CadenceWeeks`, `OrdinaryCadenceWeeks`, `Status`, `Rationale` | [`mod.rs`](./mod.rs) |
| Rebooking recommendation policy | `domain::grooming::rebooking::Policy`, `Recommendation` | [`mod.rs`](./mod.rs) |
| Reminder plan | `domain::grooming::reminder::Policy`, `Rule`, `Kind`, `Consent`, `SendBoundary`, `Plan` | [`mod.rs`](./mod.rs) |
| Location grooming contract | `domain::grooming::Contract` | [`mod.rs`](./mod.rs) |

## Grooming workflow surface

The labor-cost-reduction surface is safer grooming scheduling and follow-up with fewer manager/groomer handoffs.

1. A location's grooming contract starts as [`domain::grooming::Contract`](./mod.rs): calendar ownership, breed/coat estimates, no-show rule, rebooking cadence, reminder rules, and history requirement. `Contract::standard_petsuites` is a fixture-like standard contract for service-contract storage and tests.
2. Duration estimation combines an [`EstimationRequest`](./mod.rs), prior [`history::ServiceHistoryEntry`](./mod.rs) records, and the [`Contract`](./mod.rs). [`EstimationPolicy::estimate`](./mod.rs) prefers same-pet history with duration, otherwise uses breed/coat estimates and marks matted coats for groomer review.
3. Review requirements map to shared [`domain::policy::ReviewGate`](../policy.rs) values through [`ReviewRequirement::calendar_execution_gate`](./mod.rs), so the scheduling surface can ask for manager, groomer, or care review without inventing new approval flags.
4. No-show behavior is isolated in `domain::grooming::no_show`; repeat behavior can become a deposit requirement or manager-review gate rather than an implicit front-desk judgment.
5. Rebooking logic in `domain::grooming::rebooking::Policy` turns completed-service history and cadence into due-later/due-now/overdue/recommendation-needed outcomes.
6. Reminder planning in `domain::grooming::reminder::Policy` separates consent and approval state from the eventual customer-message sender; this module plans send boundaries but does not send messages.

## Cross-crate relationships

- The domain crate exposes this module from [`domain/src/lib.rs`](../lib.rs) as `pub mod grooming`.
- `domain::operations::ServiceOffering::Grooming` links service catalog rows to [`domain::grooming::Service`](./mod.rs) and [`domain::grooming::rebooking::Cadence`](./mod.rs) in [`domain/src/operations.rs`](../operations.rs).
- `domain::operations::service_core::ServiceContracts` includes `grooming: domain::grooming::Contract` in [`domain/src/operations.rs`](../operations.rs), alongside boarding, daycare, training, and retail contracts.
- Shared identity enters through `domain::entities::CustomerId`, `LocationId`, `PetId`, and `StaffId` in [`domain/src/entities.rs`](../entities.rs). Keep those IDs from `domain::entities` rather than creating grooming-local ID primitives.
- Shared approval gates live in [`domain/src/policy.rs`](../policy.rs); grooming uses `ManagerApproval`, `MedicalDocumentReview`, `RefundOrDepositException`, and `CustomerMessageApproval` through typed policy decisions.
- `storage::service_line::grooming` persists migrated grooming contracts and service-offering codes in [`storage/src/service_line/grooming.rs`](../../../storage/src/service_line/grooming.rs). `ContractRecord` wraps [`domain::grooming::Contract`](./mod.rs); `ServiceCode` maps to and from [`domain::grooming::Service`](./mod.rs); `StoredCadenceWeeks` converts to and from [`domain::grooming::rebooking::CadenceWeeks`](./mod.rs).
- `storage::operations::ServiceOfferingRecord` stores grooming service offerings as `grooming_service` and optional `grooming_cadence_weeks` in [`storage/src/operations.rs`](../../../storage/src/operations.rs), with shape checks that keep grooming fields off boarding/daycare/training/retail variants.
- Contract round-trip coverage exists in [`storage/tests/core_service_contract_storage.rs`](../../../storage/tests/core_service_contract_storage.rs), service-offering shape coverage exists in [`storage/tests/operations_storage_contracts.rs`](../../../storage/tests/operations_storage_contracts.rs), and domain behavior coverage for grooming contracts, estimates, no-show policy, rebooking, reminders, and service architecture lives in [`domain/tests/petsuites_core_service_contracts.rs`](../../tests/petsuites_core_service_contracts.rs), [`domain/tests/domain_quality_patterns.rs`](../../tests/domain_quality_patterns.rs), and [`domain/tests/service_module_architecture.rs`](../../tests/service_module_architecture.rs).
- Gingr catalog endpoint discovery includes `get_services_by_type` in [`integrations/gingr/src/endpoint/catalog.rs`](../../../integrations/gingr/src/endpoint/catalog.rs). [`integrations/gingr/src/dto/grooming.rs`](../../../integrations/gingr/src/dto/grooming.rs) currently records `ProviderSurface::NoDocumentedServiceDto` for that endpoint, and [`semantic_mapping_gaps`](../../../integrations/gingr/src/endpoint/catalog.rs) lists `grooming`; there is no grooming DTO mapper yet.

## Maintainer notes

- Keep the generic leaves qualified in prose: `domain::grooming::no_show::Policy`, `domain::grooming::rebooking::Policy`, and `domain::grooming::reminder::Policy` are different policies.
- Put service-history and style-note evidence in `domain::grooming::history`; put cadence decisions in `domain::grooming::rebooking`; put customer-message send boundaries in `domain::grooming::reminder`.
- Provider catalog/service payloads belong in `integrations/gingr` until they are promoted into validated `domain::grooming` values. Storage code should keep explicit code mappings in `storage::service_line::grooming` rather than duplicating domain enums.
