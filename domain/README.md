# `domain`

`domain` is the semantic core crate for the pet-resort workflow/agent platform. It owns the typed business language that application workflows, storage adapters, Gingr integrations, and app shells compose: service-line contracts, customer/pet/location/staff identity, reservation lifecycle facts, care/vaccine/incident review surfaces, payment and policy gates, portal/source provenance, workflow events, analytics projections, data-quality findings, audit ids, and agent operating contracts.

Start at [`src/lib.rs`](./src/lib.rs). It exports every domain module explicitly instead of hiding the model behind a prelude. New maintainers should read the modules by concept boundary: stable entities in [`src/entities.rs`](./src/entities.rs), service lines under [`src/boarding`](./src/boarding/README.md), [`src/daycare`](./src/daycare/README.md), [`src/grooming`](./src/grooming/README.md), [`src/training`](./src/training/README.md), [`src/retail`](./src/retail/README.md), and workflow/source/analytics contracts in [`src/workflow.rs`](./src/workflow.rs), [`src/source.rs`](./src/source.rs), [`src/analytics.rs`](./src/analytics.rs), and [`src/data_quality.rs`](./src/data_quality.rs).

The crate is deliberately not an integration client, database schema, HTTP API, or UI. It defines the canonical domain values those layers promote into and demote from. Provider ids, raw payloads, storage codes, and HTTP shapes should remain in boundary crates until they can be validated or normalized into these types.

## README vs Rustdoc contract

This README is the domain wiki: use it to navigate concept ownership, module boundaries, and labor-cost-reduction intent. Keep examples here descriptive and link-oriented rather than duplicating Rust construction snippets.

Executable domain examples belong in Rustdoc on [`src/lib.rs`](./src/lib.rs) and the source modules linked below, where `cargo test -p domain --doc` can compile-check semantic paths, constructors, policy decisions, source refs, and review-gate contracts. When adding or changing API usage examples, update the Rustdoc/source first and link to that surface from this README.

## Module navigation

### Semantic entity core

- [`src/entities.rs`](./src/entities.rs) is the aggregate vocabulary for locations, customers, pets, reservations, documents, vaccine records, care notes, incidents, messages, approval records, and actor refs. It defines canonical ids such as `domain::entities::LocationId`, `CustomerId`, `PetId`, `domain::entities::reservation::Id`, `DocumentId`, `VaccineRecordId`, `IncidentId`, and `MessageId`, plus aggregate records such as `domain::entities::Location`, `Customer`, `Pet`, `Reservation`, `Document`, `VaccineRecord`, `CareNote`, `Incident`, and `Message`.
- [`src/customer.rs`](./src/customer.rs), [`src/pet.rs`](./src/pet.rs), [`src/location.rs`](./src/location.rs), and [`src/portal.rs`](./src/portal.rs) own validated leaf values used by `domain::entities`: `domain::customer::Name`, `Email`, `Phone`; `domain::pet::Name`; `domain::location::Name`, `Timezone`; and `domain::portal::CustomerId`.
- [`src/staff.rs`](./src/staff.rs) owns staff task and completion-evidence vocabulary: `domain::staff::Task`, `domain::staff::Role`, `domain::staff::task::Kind`, `Status`, `Priority`, and `domain::staff::completion_evidence::Kind`. Stable staff/manager identifiers are currently `domain::entities::StaffId` and `ManagerId` in [`src/entities.rs`](./src/entities.rs).

### Service-line contracts

- [`src/boarding`](./src/boarding/README.md) models overnight boarding policy: accommodation, room capacity, care readiness, deposits, cancellations, housekeeping, staff handoff, minimum stay, and exit-bath upsell review. Its umbrella `domain::boarding::Contract` lives in [`src/boarding/mod.rs`](./src/boarding/mod.rs).
- [`src/daycare`](./src/daycare/README.md) models day play/day boarding: service variants, care mode, attendance/package policy, staff-pet ratios, group-assignment rules, group-play eligibility, attendance materialization, coverage decisions, package opportunities, incident disposition, and front-desk queueing. Its umbrella `domain::daycare::Contract` lives in [`src/daycare/mod.rs`](./src/daycare/mod.rs).
- [`src/grooming`](./src/grooming/README.md) models grooming services, appointment/duration/history/no-show/rebooking/reminder workflows, and its `domain::grooming::Contract` in [`src/grooming/mod.rs`](./src/grooming/mod.rs).
- [`src/training`](./src/training/README.md) models training programs, enrollment, curriculum, trainer availability, progress evidence, package/session balances, outcome review, customer-facing boundaries, and `domain::training::Contract` in [`src/training/mod.rs`](./src/training/mod.rs).
- [`src/retail`](./src/retail/README.md) models retail inventory, products, vendors, POS decisions, recommendations, reorders, and its `domain::retail::Contract` in [`src/retail/mod.rs`](./src/retail/mod.rs).
- [`src/operations.rs`](./src/operations.rs) ties service lines to portfolio and operations facts. It defines `domain::operations::ServiceOffering`, lodging/daycare offer vocabulary, technology ecosystem values, AI use cases, data-quality issue categories, operating functions, communication workflows, capacity constraints, optimization opportunities, and `domain::operations::service_core::ServiceContracts` / `ServiceLine`.

### Reservation, care, payment, policy, and review surfaces

- [`src/reservation`](./src/reservation/README.md) owns reusable reservation policy values such as `domain::reservation::MinimumAgeWeeks`, `AgeThreshold`, `AddOnLabel`, and `TransitionReason`. The reservation aggregate and lifecycle enum live in `domain::entities::Reservation` and `domain::entities::reservation::Status` in [`src/entities.rs`](./src/entities.rs).
- [`src/care.rs`](./src/care.rs) owns care-profile leaf values and review semantics: feeding instructions, allergies, medical conditions, medication name/dose/schedule, emergency/vet contacts, and `domain::care::MedicationReviewRequirement`.
- [`src/vaccine.rs`](./src/vaccine.rs) defines `domain::vaccine::Status`, while [`src/policy.rs`](./src/policy.rs) defines vaccine and workflow policy identifiers, `domain::policy::VaccineRequirement`, play-policy and denial vocabulary, automation rules/levels, and `domain::policy::ReviewGate`.
- [`src/document.rs`](./src/document.rs) owns document metadata and storage-reference values: `Classification`, `Source`, verification `Status`, virus-scan and PII-redaction status, `OriginalFile`, and `StorageRef`.
- [`src/incident.rs`](./src/incident.rs) owns cross-service incident category/severity/status/summary values. Daycare-specific incident triage remains under [`src/daycare/incident.rs`](./src/daycare/incident.rs).
- [`src/message.rs`](./src/message.rs) owns message direction/channel/status/body-reference values used by `domain::entities::Message`.
- [`src/money`](./src/money/README.md) owns money scalars and currency values. [`src/payment`](./src/payment/README.md) owns payment references, deposit status, and `domain::payment::Deposit`; service modules consume these values instead of inventing payment-state strings.

### Source, workflow, analytics, audit, and agent contracts

- [`src/source.rs`](./src/source.rs) owns source-system provenance. It models source systems, pull timestamps, endpoints, extraction batches, request scopes, schema versions, raw payload references, observed statuses, source record refs, source-agnostic reservation snapshots, Gingr provider reservation snapshots, assumptions, and data-quality promotion errors.
- [`src/data_quality.rs`](./src/data_quality.rs) owns field paths, issue kind/severity/resolution status, and `domain::data_quality::Issue`, which lets source and analytics layers name missing or inconsistent data without burying problems in strings.
- [`src/analytics.rs`](./src/analytics.rs) owns analytics projections over normalized source data: `domain::analytics::ProjectionVersion`, `domain::analytics::stay::Fact`, and `domain::analytics::service_demand::Fact`.
- [`src/workflow.rs`](./src/workflow.rs) owns workflow event identity and decision outputs: `domain::workflow::Event`, `EventType`, `Subject`, `PolicyContext`, `AllowedAction`, `Result<T>`, `Status`, and `RecommendedAction`, with nested modules for external actions, staff tasks, customer messages, and status updates.
- [`src/audit.rs`](./src/audit.rs) owns `domain::audit::EventId`, the stable audit reference embedded by approval, document, vaccine, incident, and message aggregates in [`src/entities.rs`](./src/entities.rs).
- [`src/agent.rs`](./src/agent.rs) owns agent contract values: `domain::agent::Name`, `Purpose`, `ToolName`, `ForbiddenAction`, `PolicyInstruction`, `OutputSchemaName`, and `Spec`. Application code builds executable prompt packets from these specs; the domain crate only defines the contract.
- [`src/daily_brief.rs`](./src/daily_brief.rs), [`src/lead.rs`](./src/lead.rs), and [`src/reputation.rs`](./src/reputation.rs) define manager-facing operating-day summaries, lead triage, and review/reputation signals that app workflows can turn into daily brief or CRM packets.
- [`src/temperament.rs`](./src/temperament.rs) defines pet-behavior and group-play observation vocabulary used by pet profiles and daycare eligibility.

## Major module table

| Domain area | Module/file | Representative public types |
| --- | --- | --- |
| Crate surface | [`src/lib.rs`](./src/lib.rs) | `pub mod boarding`, `pub mod entities`, `pub mod workflow`, `pub mod source`, `pub mod analytics` |
| Core aggregates | [`src/entities.rs`](./src/entities.rs) | `domain::entities::Location`, `Customer`, `Pet`, `Reservation`, `Document`, `VaccineRecord`, `CareNote`, `Incident`, `Message`, `approval::Record`, `ActorRef` |
| Customer/pet/location leaves | [`src/customer.rs`](./src/customer.rs), [`src/pet.rs`](./src/pet.rs), [`src/location.rs`](./src/location.rs), [`src/portal.rs`](./src/portal.rs) | `domain::customer::Name`, `Email`, `Phone`; `domain::pet::Name`; `domain::location::Name`, `Timezone`; `domain::portal::CustomerId` |
| Staff work | [`src/staff.rs`](./src/staff.rs) | `domain::staff::Task`, `Role`, `task::Kind`, `task::Status`, `completion_evidence::Kind` |
| Boarding | [`src/boarding/mod.rs`](./src/boarding/mod.rs), [`src/boarding/README.md`](./src/boarding/README.md) | `domain::boarding::Contract`, `CapacityPlan`, `ServiceWindow`, `DepositRule`, `domain::boarding::capacity::Policy`, `domain::boarding::care::Readiness` |
| Daycare | [`src/daycare/mod.rs`](./src/daycare/mod.rs), [`src/daycare/README.md`](./src/daycare/README.md) | `domain::daycare::Contract`, `ServiceVariant`, `StaffPetRatio`, `domain::daycare::eligibility::GroupPlayPolicy`, `domain::daycare::front_desk::ReadinessDecision` |
| Grooming | [`src/grooming/mod.rs`](./src/grooming/mod.rs), [`src/grooming/README.md`](./src/grooming/README.md) | `domain::grooming::Contract`, `Service`, `DurationEstimate`, `EstimationPolicy`, `domain::grooming::rebooking::Policy` |
| Training | [`src/training/mod.rs`](./src/training/mod.rs), [`src/training/README.md`](./src/training/README.md) | `domain::training::Contract`, `Program`, `ProgressEvidence`, `ApprovalState`, `OutcomeReviewState`, `SessionBalance` |
| Retail | [`src/retail/mod.rs`](./src/retail/mod.rs), [`src/retail/README.md`](./src/retail/README.md) | `domain::retail::Contract`, `product::Product`, `inventory::Stock`, `pos::Decision`, `recommendation::Decision`, `reorder::Decision` |
| Portfolio operations | [`src/operations.rs`](./src/operations.rs) | `domain::operations::ServiceOffering`, `TechnologyEcosystem`, `AiUseCase`, `DataQualityIssue`, `service_core::ServiceContracts`, `service_core::ServiceLine` |
| Reservations | [`src/reservation/mod.rs`](./src/reservation/mod.rs), [`src/entities.rs`](./src/entities.rs) | `domain::reservation::MinimumAgeWeeks`, `AgeThreshold`, `AddOnLabel`, `TransitionReason`; `domain::entities::Reservation`, `domain::entities::reservation::Status` |
| Care/vaccines/documents/incidents | [`src/care.rs`](./src/care.rs), [`src/vaccine.rs`](./src/vaccine.rs), [`src/document.rs`](./src/document.rs), [`src/incident.rs`](./src/incident.rs) | `domain::care::MedicationReviewRequirement`, `domain::vaccine::Status`, `domain::document::OriginalFile`, `StorageRef`, `domain::incident::Category`, `Severity` |
| Payment and money | [`src/payment/mod.rs`](./src/payment/mod.rs), [`src/money/mod.rs`](./src/money/mod.rs) | `domain::payment::Reference`, `DepositStatus`, `Deposit`; `domain::money::Money`, `MinorUnits`, `Currency` |
| Policy and approval gates | [`src/policy.rs`](./src/policy.rs), [`src/entities.rs`](./src/entities.rs) | `domain::policy::Id`, `VaccineRequirement`, `ReviewGate`, `policy::play::IneligibilityReason`; `domain::entities::approval::Record` |
| Source normalization | [`src/source.rs`](./src/source.rs) | `domain::source::System`, `Provenance`, `RecordRef`, `source::reservation::Snapshot`, `source::reservation::Assumption`, `source::gingr::reservation::Snapshot` |
| Data quality | [`src/data_quality.rs`](./src/data_quality.rs) | `domain::data_quality::FieldPath`, `Kind`, `Severity`, `ResolutionStatus`, `Issue` |
| Analytics projections | [`src/analytics.rs`](./src/analytics.rs) | `domain::analytics::ProjectionVersion`, `analytics::stay::Fact`, `analytics::service_demand::Fact`, `analytics::service_demand::DemandUnits` |
| Workflow decisions | [`src/workflow.rs`](./src/workflow.rs) | `domain::workflow::Event`, `EventType`, `Subject`, `PolicyContext`, `AllowedAction`, `Result<T>`, `Status`, `RecommendedAction` |
| Audit and agents | [`src/audit.rs`](./src/audit.rs), [`src/agent.rs`](./src/agent.rs) | `domain::audit::EventId`; `domain::agent::Spec`, `Name`, `Purpose`, `ToolName`, `PolicyInstruction`, `OutputSchemaName` |
| Daily operations / CRM signals | [`src/daily_brief.rs`](./src/daily_brief.rs), [`src/lead.rs`](./src/lead.rs), [`src/reputation.rs`](./src/reputation.rs) | `domain::daily_brief::ResortOperatingDay`, `LaborRisk`, `Action`; `domain::lead::Triage`; `domain::reputation::Signal` |

## Type/module map

| Concept | Public type/module path | Defined in |
| --- | --- | --- |
| Crate module registry | `domain` crate root | [`src/lib.rs`](./src/lib.rs) |
| Location identity and profile | `domain::entities::LocationId`, `domain::entities::Location`, `domain::entities::Brand`, `domain::entities::LocationPolicyRefs` | [`src/entities.rs`](./src/entities.rs) |
| Customer identity/profile | `domain::entities::CustomerId`, `domain::entities::Customer`, `domain::entities::PortalAccountRef`, `domain::entities::ContactChannel` | [`src/entities.rs`](./src/entities.rs) |
| Customer contact values | `domain::customer::Name`, `domain::customer::Email`, `domain::customer::Phone` | [`src/customer.rs`](./src/customer.rs) |
| Pet identity/profile | `domain::entities::PetId`, `domain::entities::Pet`, `domain::entities::Species`, `Sex`, `SpayNeuterStatus`, `TemperamentProfile`, `CareProfile` | [`src/entities.rs`](./src/entities.rs) |
| Staff identity and roles | `domain::entities::StaffId`, `domain::entities::ManagerId`, `domain::staff::Role` | [`src/entities.rs`](./src/entities.rs), [`src/staff.rs`](./src/staff.rs) |
| Reservation aggregate | `domain::entities::Reservation`, `domain::entities::reservation::Id`, `Status`, `Source`, `domain::entities::ServiceKind`, `AddOn`, `HardStop` | [`src/entities.rs`](./src/entities.rs) |
| Reservation support policy | `domain::reservation::MinimumAgeWeeks`, `AgePolicyReason`, `AgeThreshold`, `AddOnLabel`, `TransitionReason`, `Error`, `Result` | [`src/reservation/mod.rs`](./src/reservation/mod.rs), [`src/reservation/error.rs`](./src/reservation/error.rs) |
| Boarding service line | `domain::boarding::Contract`, `domain::boarding::capacity::Policy`, `domain::boarding::deposit::ConfirmationReadiness`, `domain::boarding::upsell::Recommendation` | [`src/boarding/mod.rs`](./src/boarding/mod.rs), [`src/boarding/capacity.rs`](./src/boarding/capacity.rs), [`src/boarding/deposit.rs`](./src/boarding/deposit.rs), [`src/boarding/upsell.rs`](./src/boarding/upsell.rs) |
| Daycare service line | `domain::daycare::Contract`, `domain::daycare::ServiceVariant`, `domain::daycare::eligibility::GroupPlayPolicy`, `domain::daycare::front_desk::QueueTicket` | [`src/daycare/mod.rs`](./src/daycare/mod.rs), [`src/daycare/eligibility.rs`](./src/daycare/eligibility.rs), [`src/daycare/front_desk.rs`](./src/daycare/front_desk.rs) |
| Grooming service line | `domain::grooming::Contract`, `domain::grooming::Service`, `domain::grooming::DurationEstimate`, `domain::grooming::appointment::Request` | [`src/grooming/mod.rs`](./src/grooming/mod.rs) |
| Training service line | `domain::training::Contract`, `domain::training::Program`, `domain::training::SessionBalance`, `domain::training::availability::Decision`, `domain::training::progress::Report`, `domain::training::outcome::Claim` | [`src/training/mod.rs`](./src/training/mod.rs) |
| Retail service line | `domain::retail::Contract`, `domain::retail::product::Product`, `domain::retail::inventory::Stock`, `domain::retail::pos::Decision` | [`src/retail/mod.rs`](./src/retail/mod.rs), [`src/retail/product.rs`](./src/retail/product.rs), [`src/retail/inventory.rs`](./src/retail/inventory.rs), [`src/retail/pos.rs`](./src/retail/pos.rs) |
| Care profile leaves | `domain::care::FeedingInstruction`, `AllergyName`, `MedicalConditionName`, `MedicationName`, `MedicationDose`, `MedicationSchedule`, `MedicationReviewRequirement` | [`src/care.rs`](./src/care.rs) |
| Document and vaccine review | `domain::entities::Document`, `VaccineRecord`, `domain::document::Status`, `domain::vaccine::Status` | [`src/entities.rs`](./src/entities.rs), [`src/document.rs`](./src/document.rs), [`src/vaccine.rs`](./src/vaccine.rs) |
| Incidents and care notes | `domain::entities::CareNote`, `domain::entities::Incident`, `domain::incident::Category`, `Severity`, `Status` | [`src/entities.rs`](./src/entities.rs), [`src/incident.rs`](./src/incident.rs) |
| Messaging | `domain::entities::Message`, `domain::message::Direction`, `Channel`, `Status`, `BodyRef` | [`src/entities.rs`](./src/entities.rs), [`src/message.rs`](./src/message.rs) |
| Money and deposits | `domain::money::Money`, `MinorUnits`, `Currency`; `domain::payment::Reference`, `DepositStatus`, `Deposit` | [`src/money/mod.rs`](./src/money/mod.rs), [`src/payment/mod.rs`](./src/payment/mod.rs) |
| Policy gates | `domain::policy::Id`, `VaccineName`, `WorkflowName`, `VaccineRequirement`, `ReviewGate`, `policy::automation::Rule`, `policy::automation::Level`, `policy::play::IneligibilityReason` | [`src/policy.rs`](./src/policy.rs) |
| Approval records | `domain::entities::approval::Record`, `Target`, `Lifecycle`, `Status` | [`src/entities.rs`](./src/entities.rs) |
| Source provenance | `domain::source::System`, `Timestamp`, `Endpoint`, `ExtractionBatchId`, `RecordRef`, `Provenance`, `source::record::Id` | [`src/source.rs`](./src/source.rs) |
| Source reservation snapshots | `domain::source::reservation::Snapshot`, `OwnerPetRelationship`, `Status`, `Assumption` | [`src/source.rs`](./src/source.rs) |
| Analytics facts | `domain::analytics::ProjectionVersion`, `analytics::stay::Fact`, `analytics::service_demand::Fact` | [`src/analytics.rs`](./src/analytics.rs) |
| Data-quality issues | `domain::data_quality::FieldPath`, `Kind`, `Severity`, `ResolutionStatus`, `Issue` | [`src/data_quality.rs`](./src/data_quality.rs) |
| Workflow event/result | `domain::workflow::EventId`, `Summary`, `Event`, `EventType`, `Subject`, `PolicyContext`, `AllowedAction`, `Result<T>`, `RecommendedAction` | [`src/workflow.rs`](./src/workflow.rs) |
| Audit reference | `domain::audit::EventId` | [`src/audit.rs`](./src/audit.rs) |
| Agent specification | `domain::agent::Spec`, `Name`, `Purpose`, `ToolName`, `ForbiddenAction`, `PolicyInstruction`, `OutputSchemaName` | [`src/agent.rs`](./src/agent.rs) |

## How the crate reduces labor cost

The labor-cost-reduction goal is encoded as typed exception triage rather than as broad automation claims:

1. Service-line modules turn front-desk and manager judgment into explicit policy decisions. Examples include `domain::boarding::capacity::Decision` in [`src/boarding/capacity.rs`](./src/boarding/capacity.rs), `domain::boarding::deposit::ConfirmationReadiness` in [`src/boarding/deposit.rs`](./src/boarding/deposit.rs), `domain::daycare::eligibility::GroupPlayDecision` in [`src/daycare/eligibility.rs`](./src/daycare/eligibility.rs), and `domain::retail::recommendation::Decision` in [`src/retail/recommendation.rs`](./src/retail/recommendation.rs).
2. Core aggregates in [`src/entities.rs`](./src/entities.rs) keep customer, pet, reservation, document, vaccine, incident, message, and approval data in semantic shapes. Application workflows can route obvious work and reserve human attention for `domain::policy::ReviewGate` or `domain::entities::approval::Record` cases.
3. Source and data-quality modules normalize provider/source data before analytics or workflow use. `domain::source::reservation::Snapshot` in [`src/source.rs`](./src/source.rs) and `domain::data_quality::Issue` in [`src/data_quality.rs`](./src/data_quality.rs) prevent missing customer/pet/location/service facts from silently becoming bad projections or unsafe automation.
4. Analytics projections in [`src/analytics.rs`](./src/analytics.rs) preserve provenance and data-quality status on stay and service-demand facts, so operations reporting can distinguish clean facts from manager-review-required facts.
5. Agent contracts in [`src/agent.rs`](./src/agent.rs) and workflow outcomes in [`src/workflow.rs`](./src/workflow.rs) define allowed tools, forbidden actions, output schemas, recommended actions, and review requirements before app shells execute anything.
6. Storage and integrations can convert at the boundary instead of forcing maintainers to remember which strings or provider fields mean boarding capacity, vaccine readiness, deposit exceptions, or customer-message approval.

## Cross-crate relationships

- `app` composes domain contracts into executable workflow packets and tool-port requests. See [`../app/src/lib.rs`](../app/src/lib.rs), [`../app/src/booking_triage.rs`](../app/src/booking_triage.rs), [`../app/src/checkout_completion.rs`](../app/src/checkout_completion.rs), [`../app/src/manager_daily_brief.rs`](../app/src/manager_daily_brief.rs), [`../app/src/crm_retention.rs`](../app/src/crm_retention.rs), [`../app/src/daily_update.rs`](../app/src/daily_update.rs), [`../app/src/agents.rs`](../app/src/agents.rs), and [`../app/src/tools.rs`](../app/src/tools.rs). These files should orchestrate and translate; they should not redefine domain invariants already owned here.
- `storage` owns persistence-shaped records and code conversions. See [`../storage/src/lib.rs`](../storage/src/lib.rs), [`../storage/src/operations.rs`](../storage/src/operations.rs), and service-line adapters under [`../storage/src/service_line`](../storage/src/service_line/mod.rs): [`boarding.rs`](../storage/src/service_line/boarding.rs), [`daycare.rs`](../storage/src/service_line/daycare.rs), [`grooming.rs`](../storage/src/service_line/grooming.rs), [`retail.rs`](../storage/src/service_line/retail.rs), and [`training.rs`](../storage/src/service_line/training.rs). Storage code values should convert explicitly to/from domain types.
- `integrations/gingr` owns provider-boundary configuration, DTOs, endpoint requests, response parsing, webhooks, and mapping. See [`../integrations/gingr/src/lib.rs`](../integrations/gingr/src/lib.rs), [`../integrations/gingr/src/endpoint/mod.rs`](../integrations/gingr/src/endpoint/mod.rs), [`../integrations/gingr/src/endpoint/reservations.rs`](../integrations/gingr/src/endpoint/reservations.rs), [`../integrations/gingr/src/endpoint/owners_animals.rs`](../integrations/gingr/src/endpoint/owners_animals.rs), [`../integrations/gingr/src/dto/mod.rs`](../integrations/gingr/src/dto/mod.rs), [`../integrations/gingr/src/mapping/mod.rs`](../integrations/gingr/src/mapping/mod.rs), and [`../integrations/gingr/src/webhook.rs`](../integrations/gingr/src/webhook.rs). Gingr field names and transport quirks belong there, not in `domain`.
- `apps/api`, `apps/worker`, and `apps/cli` expose or run app workflows. See [`../apps/api/src/http.rs`](../apps/api/src/http.rs), [`../apps/api/src/main.rs`](../apps/api/src/main.rs), [`../apps/worker/src/runtime.rs`](../apps/worker/src/runtime.rs), and [`../apps/cli/src/main.rs`](../apps/cli/src/main.rs). These shells should depend on domain through app/storage/integration ports instead of becoming alternative domain models.
- Domain module READMEs provide deeper service-line navigation: [`src/boarding/README.md`](./src/boarding/README.md), [`src/daycare/README.md`](./src/daycare/README.md), [`src/reservation/README.md`](./src/reservation/README.md), [`src/payment/README.md`](./src/payment/README.md), [`src/money/README.md`](./src/money/README.md), [`src/grooming/README.md`](./src/grooming/README.md), [`src/training/README.md`](./src/training/README.md), and [`src/retail/README.md`](./src/retail/README.md).
- Repo-level context lives in [`../README.md`](../README.md). Gingr endpoint fixture documentation lives in [`../docs/integrations/gingr/README.md`](../docs/integrations/gingr/README.md) and [`../docs/integrations/gingr/fixtures/webhooks/README.md`](../docs/integrations/gingr/fixtures/webhooks/README.md).

## Maintainer notes

- Preserve semantic module paths in prose and code. Prefer names such as `domain::boarding::capacity::Policy`, `domain::entities::reservation::Status`, `domain::source::reservation::Snapshot`, and `domain::workflow::RecommendedAction` over flattened aliases.
- Add new concepts where the invariant belongs. Service-line policy belongs in the service-line module; stable customer/pet/reservation aggregates belong in [`src/entities.rs`](./src/entities.rs); provider payloads belong in [`../integrations/gingr/src`](../integrations/gingr/src/lib.rs); storage codes belong in [`../storage/src`](../storage/src/lib.rs).
- Keep boundary conversion explicit. If a raw provider/status/storage string becomes operationally meaningful, promote it through a named domain constructor, enum variant, or policy result instead of passing it around as `String`.
- Treat `domain::policy::ReviewGate`, `domain::entities::approval::Record`, data-quality issue types, and workflow result statuses as safety rails for automation. They are how the system saves manager time without skipping review on unsafe actions.
- When adding docs for a module, link the local Rust files and preserve the owner module path so maintainers can navigate from concept to type definition quickly.
