# Gingr Source-of-Record and BI Read-Model Plan

Date: 2026-06-16

## Context

Tyler reported that Gingr is being used as a de facto database, and that the existing team already has code that pulls data from Gingr and constructs various types for BI consumption. That code is messy, but it means this is not hypothetical: Gingr-derived operational data is already being transformed into analytical/business-facing shapes.

The repository-wide business objective is labor-cost reduction. NVA needs high-resolution operational insight so it can creatively build schedule optimization, staffing tools, automations, reports, SOP/chat assistants for employees, process controls, and workflow tooling that reduce avoidable labor spend without degrading care quality or customer experience.

BI is not being asked to define the labor-cost strategy. BI is relevant because it currently owns a database, and Gingr is one source that feeds or informs it. Discovery questions for BI should therefore focus on factual data shape, source provenance, schema, joins, refresh behavior, quality issues, and existing transformation assumptions.

Gingr is only one source in the larger operating picture. NVA should expect additional sources such as labor/timeclock data, staff schedules, payroll or wage-cost exports, capacity/room inventory, POS/payment data, customer communications, task/work-order systems, SOP/process artifacts, and manager-entered operational notes.

For NVA, this means Gingr should not be treated as the whole product, a simple integration, or a clean source of truth. It should be treated as one messy upstream operational source whose facts must be extracted, preserved, normalized, quality-checked, joined with other source facts, and projected into downstream models.

## Architectural posture

NVA should not copy Gingr mess into the domain core. The desired flow is:

```text
Gingr endpoints / reports / exports / webhooks
→ raw Gingr DTOs
→ versioned Gingr source snapshots with provenance
→ semantic domain promotion + data-quality issues
→ BI/read-model projections and workflow typestate validators
```

The existing codebase already has the right broad seams:

```text
integrations/gingr  # provider DTOs, endpoints, mappings
storage             # persistence boundary
app                 # use-case orchestration
analytics/read model # proposed new projection boundary
```

The key addition is a deliberate source-data lane: explicit source snapshots, source provenance, data-quality issues, and BI/read-model projections.

## Non-goals

- Do not assume Gingr is clean enough to be the domain model.
- Do not let BI report columns pollute core domain entities.
- Do not invent fake provider DTO coverage for unknown Gingr surfaces.
- Do not build a production warehouse before we understand their existing DB and BI code.
- Do not store real credentials, connection strings, or production data in the repo.

## Layer model

### 1. Raw Gingr facts

Provider-shaped DTOs and endpoint/report/webhook responses live under the Gingr integration crate. These may preserve Gingr naming and ugliness because they represent the external system honestly.

Examples:

```rust
gingr::dto::Reservation
gingr::dto::Owner
gingr::dto::Animal
gingr::dto::Invoice
gingr::dto::Payment
gingr::dto::ReportCard
```

Rules:

- Preserve raw provider IDs/statuses where needed.
- Preserve original source metadata where available.
- Keep raw DTOs out of the domain core.
- Convert through named mapping/promotion functions rather than casual `.into()` chains when validation or trust-boundary promotion occurs.

### 2. Gingr source snapshots

Snapshots are stable, versioned source facts that NVA can persist, diff, replay, and use for both BI and workflow classification.

Candidate module:

```text
integrations/gingr/src/snapshot/
```

Candidate concepts:

```rust
gingr::snapshot::ReservationSnapshot
gingr::snapshot::CustomerSnapshot
gingr::snapshot::PetSnapshot
gingr::snapshot::StaySnapshot
gingr::snapshot::InvoiceSnapshot
gingr::snapshot::PaymentSnapshot
```

Every snapshot should carry provenance such as:

```rust
source::System::Gingr
source::EndpointName
source::ExtractionBatchId
source::ProviderRecordId
source::PulledAt
source::SchemaVersion
```

Snapshots are not yet final domain truth. They are source-derived facts with provenance.

### 3. Semantic domain promotion

Snapshots promote into domain concepts through explicit validators/conversions:

```text
gingr::snapshot::ReservationSnapshot
→ domain::reservation / service-line types
→ data_quality::Issue when facts are missing, conflicting, or unmapped
```

Promotion should be tested. If new information from Tyler or the BI team changes what a field means, update the relevant semantic type/conversion and let compile/test pressure show downstream effects.

### 4. Data quality as a first-class domain

Because Gingr is being used as a database but is not a clean analytical store, data-quality problems should be modeled, not hidden in logs.

Candidate module:

```text
domain/src/data_quality.rs
```

Candidate concepts:

```rust
data_quality::Issue
data_quality::Severity
data_quality::Provenance
data_quality::ResolutionStatus
data_quality::IssueKind
```

Example issue kinds:

```text
MissingRequiredField
UnknownProviderStatus
ConflictingTimestamps
DuplicateProviderRecord
UnmappedServiceType
AmbiguousOwnerPetRelationship
PaymentStateConflict
CheckoutEvidenceMissing
```

Data-quality issues should be available to:

- BI projections;
- audit trails;
- workflow validators;
- manager-review gates;
- future repair tooling.

### 5. BI/read-model projections

BI/read-model shapes are not domain entities. They are denormalized, stable analytical outputs built from snapshots/domain promotion.

Candidate crate or module:

```text
analytics/
```

or, for a smaller first increment:

```text
app/src/read_model/
```

Preferred eventual shape is a separate workspace crate if projections grow.

Candidate read models:

```rust
analytics::stay::Fact
analytics::customer::Dimension
analytics::pet::Dimension
analytics::revenue::Fact
analytics::service_utilization::Fact
analytics::capacity::Snapshot
analytics::retention::Opportunity
```

These should be generated from semantic source/domain facts, not manually assembled ad hoc from raw Gingr DTOs.

### 6. Workflow typestate inputs

The same source snapshots and data-quality signals should feed Statum workflow validators.

Example:

```text
Gingr reservation status says checked out
+ checkout timestamp exists
+ payment state is settled
+ no blocking incident/data-quality issue
→ CheckoutMachine<ReadyForCompletion>
```

But:

```text
Gingr reservation status says checked out
+ payment unresolved or incident note exists
→ CheckoutMachine<NeedsManagerReview> or CheckoutMachine<Blocked>
```

This keeps workflow typestate grounded in validated runtime facts instead of raw provider status strings.

## Database posture

We should expect that the organization may already have a database or warehouse populated from Gingr. We do not yet know:

- database engine;
- schema shape;
- ingestion cadence;
- ownership;
- field quality;
- whether it is authoritative, experimental, or BI-only;
- whether it stores raw Gingr payloads, normalized tables, derived report rows, or all of the above.

Until that is known, NVA should not hard-code assumptions around their existing DB.

Recommended interim approach:

1. Define NVA's source snapshot/read-model contracts first.
2. Create a local development fixture database only if it helps exercise extraction/projection tests.
3. Keep the local database disposable and ignored by git.
4. Prefer schema files/migrations/sample fixtures in the repo; never commit real Gingr/customer data.
5. When the real BI DB becomes available, add an adapter/import lane that maps its existing tables into NVA source snapshots and data-quality reports.

A local lab DB can be SQLite initially if the first need is deterministic tests and projection shape. If we need to mimic a production warehouse or validate SQL semantics closer to their stack, promote to local Postgres later.

Candidate ignored local path:

```text
.var/nva-source-lab.sqlite3
```

Candidate durable repo artifacts:

```text
analytics/migrations/
analytics/tests/fixtures/
docs/integrations/gingr/source-inventory.md
docs/integrations/gingr/bi-read-model-contract.md
```

## Kanban execution plan

Use a dedicated board because this is a multi-lane workstream that should survive restarts and may need worker handoffs.

Board slug:

```text
nva-gingr-source-bi
```

Suggested card graph:

1. **Document/source inventory** — inspect current Gingr integration code/docs and identify known endpoints, DTOs, mappings, and gaps.
2. **Architecture contract** — write the source snapshot + provenance + data-quality + BI projection target architecture as repo docs.
3. **First code skeleton** — add source/provenance/data-quality/read-model contract types with tests, centered on reservation/stay.
4. **Local lab DB decision/provision** — decide SQLite vs Postgres for immediate projection tests; create ignored local DB only if useful.
5. **Projection spike** — build one deterministic projection from a fixture Gingr reservation snapshot to a BI stay fact, preserving provenance and data-quality warnings.
6. **Workflow connection design** — define how snapshots/data-quality issues feed future Statum validators for reservation/checkout workflows.
7. **Review/fan-in** — run semantic review and Rust gates; commit/push results.

## First thin slice

The first implementation slice should be reservation/stay because it connects most downstream needs:

```text
customer
pet
location
service line
dates
status
payment/deposit
checkout/completion
revenue
BI reporting
workflow state
```

A useful first test name:

```rust
gingr_reservation_snapshot_preserves_provider_provenance_and_projects_to_stay_fact
```

A useful data-quality test name:

```rust
missing_checkout_timestamp_marks_snapshot_as_data_quality_issue
```

## Open questions for Tyler / BI team

- What database engine do they use for the current Gingr-derived DB?
- Is it raw-payload storage, normalized tables, BI fact/dimension tables, or a mixture?
- Which Gingr endpoints/reports feed it?
- What code constructs the current BI types, and can we inspect it?
- Which fields are considered reliable vs messy?
- How do they handle historical backfills and incremental pulls?
- How do they handle deleted/merged/duplicated Gingr records?
- What BI outputs are most important: revenue, utilization, retention, capacity, customer lifecycle, service-line performance?
- What IDs are stable enough to join across exports/endpoints?
- Are there known Gingr statuses or report columns with overloaded meanings?

## Success criteria

This lane is successful when NVA has:

- explicit source snapshots for at least one Gingr object family;
- first-class provenance and data-quality concepts;
- a deterministic BI/read-model projection test;
- a clear local lab DB posture;
- a path for importing their existing BI DB later without contaminating the domain core;
- a clear bridge from source snapshots into future Statum workflow validators.
