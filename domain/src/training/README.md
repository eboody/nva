# `domain::training`

`domain::training` is the domain crate's model for pet-training programs, enrollment readiness, trainer assignment, curriculum progress, package/session balance, progress reports, outcome documentation, and follow-up planning. It owns training concepts that should not be flattened into reservation notes or provider product names: program duration, curriculum units, milestone status, trainer requirements, package ledger entries, progress evidence, approval states, member-facing boundaries, and follow-up states.

Start at [`mod.rs`](./mod.rs). The module is implemented in one file with nested semantic modules such as `domain::training::program`, `domain::training::enrollment`, `domain::training::curriculum`, `domain::training::availability`, `domain::training::progress`, `domain::training::outcome`, `domain::training::package`, and `domain::training::follow_up`.

## Module navigation

- [`mod.rs`](./mod.rs) defines shared training vocabulary: [`SessionCount`](./mod.rs), [`SessionId`](./mod.rs), [`SessionRef`](./mod.rs), [`ProgressReportId`](./mod.rs), [`EvidenceId`](./mod.rs), [`OutcomeDocumentationId`](./mod.rs), [`ProgressNote`](./mod.rs), [`Program`](./mod.rs), [`ProgressTracking`](./mod.rs), [`Outcome`](./mod.rs), [`FollowUpCadence`](./mod.rs), [`SessionBalance`](./mod.rs), [`Contract`](./mod.rs), [`Error`](./mod.rs), and [`Result`](./mod.rs).
- `domain::training::program` in [`mod.rs`](./mod.rs) defines [`DurationWeeks`](./mod.rs), [`DurationWeeksError`](./mod.rs), and [`Duration`](./mod.rs), including the positive-week invariant for multi-week programs.
- `domain::training::enrollment` in [`mod.rs`](./mod.rs) defines [`enrollment::Id`](./mod.rs) and [`Readiness`](./mod.rs). `Readiness::blocking_gate` converts trainer, behavior/care, and package/payment readiness into shared review gates.
- `domain::training::curriculum` in [`mod.rs`](./mod.rs) defines [`Unit`](./mod.rs), [`Progress`](./mod.rs), and nested [`curriculum::milestone::Id`](./mod.rs) / [`Status`](./mod.rs) values.
- `domain::training::trainer` in [`mod.rs`](./mod.rs) defines [`Availability`](./mod.rs), [`Requirement`](./mod.rs), and [`Qualification`](./mod.rs). `Requirement::requires_named_trainer` keeps named-trainer scheduling pressure explicit.
- `domain::training::availability` in [`mod.rs`](./mod.rs) defines assignment evaluation: [`Request`](./mod.rs), [`CapacityDecision`](./mod.rs), [`Decision`](./mod.rs), [`WaitlistReason`](./mod.rs), [`ReviewReason`](./mod.rs), and [`Policy`](./mod.rs). `Decision::provider_mutation_gate` names when provider-side assignment should wait for approval.
- `domain::training::progress` in [`mod.rs`](./mod.rs) defines [`Report`](./mod.rs) and [`ReportBuilder`](./mod.rs). Reports require at least one [`ProgressEvidence`](./mod.rs) before they can be built.
- `domain::training::outcome` in [`mod.rs`](./mod.rs) defines [`ClaimStatus`](./mod.rs), [`ClaimEvidence`](./mod.rs), [`Claim`](./mod.rs), [`Documentation`](./mod.rs), and [`DocumentationBuilder`](./mod.rs). Achieved/readiness claims require evidence, and outcome documentation requires at least one claim.
- `domain::training::package` in [`mod.rs`](./mod.rs) defines [`Policy`](./mod.rs), [`package::Id`](./mod.rs), [`LedgerEntry`](./mod.rs), [`OpeningLedger`](./mod.rs), [`Ledger`](./mod.rs), [`UsageDecision`](./mod.rs), and [`UsagePolicy`](./mod.rs). Multi-session package usage is modeled as a ledger rather than a loose remaining-session integer.
- `domain::training::follow_up` in [`mod.rs`](./mod.rs) defines [`Trigger`](./mod.rs), [`Purpose`](./mod.rs), [`EvidenceReadiness`](./mod.rs), [`State`](./mod.rs), [`Plan`](./mod.rs), and [`Policy`](./mod.rs), separating follow-up readiness from actual message delivery.

## Type/module map

| Concept | Public type/module path | Defined in |
| --- | --- | --- |
| Training program and duration | `domain::training::Program`, `domain::training::program::Duration`, `DurationWeeks` | [`mod.rs`](./mod.rs) |
| Package/session counts | `domain::training::SessionCount`, `domain::training::SessionBalance` | [`mod.rs`](./mod.rs) |
| Training identifiers | `domain::training::SessionId`, `SessionRef`, `ProgressReportId`, `EvidenceId`, `OutcomeDocumentationId` | [`mod.rs`](./mod.rs) |
| Enrollment readiness | `domain::training::enrollment::Id`, `Readiness` | [`mod.rs`](./mod.rs) |
| Curriculum progress | `domain::training::curriculum::Unit`, `Progress`, `milestone::Id`, `milestone::Status` | [`mod.rs`](./mod.rs) |
| Trainer availability/requirements | `domain::training::trainer::Availability`, `Requirement`, `Qualification` | [`mod.rs`](./mod.rs) |
| Assignment availability decision | `domain::training::availability::Policy`, `Request`, `Decision`, `CapacityDecision`, `WaitlistReason`, `ReviewReason` | [`mod.rs`](./mod.rs) |
| Progress evidence/report | `domain::training::ProgressEvidence`, `ProgressNote`, `ApprovalState`, `progress::Report` | [`mod.rs`](./mod.rs) |
| Outcome documentation | `domain::training::outcome::ClaimEvidence`, `Claim`, `Documentation`, `ClaimStatus` | [`mod.rs`](./mod.rs) |
| Member-facing boundary | `domain::training::MemberFacingBoundary`, `OutcomeReviewState` | [`mod.rs`](./mod.rs) |
| Package ledger and usage | `domain::training::package::Policy`, `LedgerEntry`, `OpeningLedger`, `Ledger`, `UsageDecision`, `UsagePolicy` | [`mod.rs`](./mod.rs) |
| Follow-up planning | `domain::training::follow_up::Policy`, `Trigger`, `Purpose`, `EvidenceReadiness`, `State`, `Plan` | [`mod.rs`](./mod.rs) |
| Location training contract | `domain::training::Contract` | [`mod.rs`](./mod.rs) |
| Training errors/results | `domain::training::Error`, `domain::training::Result` | [`mod.rs`](./mod.rs) |

## Training workflow surface

The labor-cost-reduction surface is exception triage around trainer assignment, reusable package sessions, progress evidence, and parent-facing updates.

1. A location's training contract starts as [`domain::training::Contract`](./mod.rs): program duration, curriculum, progress tracking, outcomes, trainer availability, package policy, and follow-up cadence. `Contract::standard_petsuites` is a fixture-like standard contract for service-contract storage and tests.
2. Assignment readiness combines [`availability::Request`](./mod.rs), [`enrollment::Readiness`](./mod.rs), trainer requirement, and capacity snapshot. [`availability::Policy::evaluate`](./mod.rs) returns assignment drafted, waitlist, or review-required decisions with shared `domain::policy::ReviewGate` values.
3. Progress reporting uses [`progress::Report::builder`](./mod.rs) and [`ProgressEvidence`](./mod.rs). The builder rejects evidence-free reports, which keeps draft progress updates from becoming unsupported parent-facing claims.
4. Outcome documentation uses [`outcome::Claim::from_evidence`](./mod.rs) and [`outcome::Documentation::builder`](./mod.rs). Achieved/readiness outcomes require evidence, and member-facing boundaries remain explicit until approved.
5. Package usage uses [`package::Ledger`](./mod.rs) and [`package::UsagePolicy`](./mod.rs). Session reservations and consumption become ledger entries, reducing manual reconciliation of remaining sessions.
6. Follow-up planning uses [`follow_up::Policy`](./mod.rs) to decide whether a progress update, homework coaching, completion summary, or re-enrollment prompt is not due, evidence-blocked, approval-blocked, or suppressed.

## Cross-crate relationships

- The domain crate exposes this module from [`domain/src/lib.rs`](../lib.rs) as `pub mod training`.
- `domain::operations::ServiceOffering::Training` links service catalog rows to [`domain::training::Program`](./mod.rs) in [`domain/src/operations.rs`](../operations.rs).
- `domain::operations::service_core::ServiceContracts` includes `training: domain::training::Contract` in [`domain/src/operations.rs`](../operations.rs), alongside boarding, daycare, grooming, and retail contracts.
- Shared identity enters through `domain::entities::CustomerId`, `LocationId`, `PetId`, `StaffId`, and `ManagerId` in [`domain/src/entities.rs`](../entities.rs). Keep those IDs from `domain::entities` rather than creating training-local customer/pet/staff primitives.
- Shared approval gates live in [`domain/src/policy.rs`](../policy.rs); training assignment, package usage, progress reports, outcome documentation, and follow-up plans use those gates instead of ad-hoc booleans.
- `storage::service_line::training` persists migrated training contracts and service-offering program codes in [`storage/src/service_line/training.rs`](../../../storage/src/service_line/training.rs). `ContractRecord` wraps [`domain::training::Contract`](./mod.rs); `ProgramRecord` maps to and from [`domain::training::Program`](./mod.rs); `StoredProgramDurationWeeks` converts to and from [`domain::training::program::DurationWeeks`](./mod.rs).
- `storage::operations::ServiceOfferingRecord` stores training service offerings as `training_program` in [`storage/src/operations.rs`](../../../storage/src/operations.rs), with shape checks that keep training fields off boarding/daycare/grooming/retail variants.
- Contract round-trip coverage exists in [`storage/tests/core_service_contract_storage.rs`](../../../storage/tests/core_service_contract_storage.rs), service-offering shape coverage exists in [`storage/tests/operations_storage_contracts.rs`](../../../storage/tests/operations_storage_contracts.rs), and domain behavior coverage for training contracts, availability, package usage, progress reports, outcomes, follow-ups, and service architecture lives in [`domain/tests/petsuites_core_service_contracts.rs`](../../tests/petsuites_core_service_contracts.rs), [`domain/tests/domain_quality_patterns.rs`](../../tests/domain_quality_patterns.rs), and [`domain/tests/service_module_architecture.rs`](../../tests/service_module_architecture.rs).
- Gingr catalog endpoint discovery includes `get_services_by_type` in [`integrations/gingr/src/endpoint/catalog.rs`](../../../integrations/gingr/src/endpoint/catalog.rs). [`integrations/gingr/src/dto/training.rs`](../../../integrations/gingr/src/dto/training.rs) currently records `ProviderSurface::NoDocumentedServiceDto` for that endpoint, and [`semantic_mapping_gaps`](../../../integrations/gingr/src/endpoint/catalog.rs) lists `training`; there is no training DTO mapper yet.

## Maintainer notes

- Keep generic leaves qualified: `domain::training::availability::Decision`, `domain::training::package::UsageDecision`, and `domain::training::follow_up::State` describe different decisions/states.
- Put trainer/capacity assignment behavior in `domain::training::availability`; put evidence-bearing progress updates in `domain::training::progress`; put evidence-backed outcome claims in `domain::training::outcome`; put reusable-session reconciliation in `domain::training::package`.
- Provider catalog/service payloads belong in `integrations/gingr` until promoted into validated `domain::training` values. Storage code should keep explicit program and duration code mappings in `storage::service_line::training` rather than duplicating domain enums.
