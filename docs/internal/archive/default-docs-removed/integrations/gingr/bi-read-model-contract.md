# Gingr BI read-model contract

Contract date: 2026-06-16

Scope: documentation-only contract for how NVA should move Gingr-derived source
facts into durable snapshots, semantic promotion, data-quality records,
analytics/read models, and future Statum workflow validators. This contract is
based on the Gingr source-of-record plan and the static source inventory; it does
not create database artifacts, migrations, fixtures, or production data copies.

## Contract stance

Gingr is an upstream operational source, not NVA's domain model and not a clean
BI warehouse. The allowed flow is:

```text
Gingr API endpoints / reports / exports / webhooks
-> raw Gingr DTOs and response wrappers
-> versioned Gingr source snapshots with provenance
-> source-agnostic reservation/stay snapshots with typed assumptions/issues
-> analytics stay facts and other read-model projections
-> future workflow-validator inputs
```

Each arrow is a trust-boundary crossing. Code may preserve provider vocabulary
inside the raw and snapshot layers, but domain, read-model, and workflow code must
consume named semantic contracts instead of ad hoc raw DTO fields.

The canonical post-refactor boundary note is
`docs/integrations/gingr/adapter-boundary-and-labor-source-expansion.md`. It
summarizes the exact chain that downstream work should preserve:

```text
Gingr DTO -> Gingr snapshot -> source-agnostic reservation snapshot -> analytics stay fact -> future workflow validators
```

## Boundary summary

- Raw Gingr DTOs own provider-shaped request/response records, unknown fields,
  and redaction markers. They must not own domain invariants, BI facts, or
  workflow states.
- Gingr snapshots own stable Gingr-derived source facts, provider vocabulary,
  provenance, payload identity, and extraction context. They must not own final
  business truth, source-agnostic joins, or denormalized dashboard rows.
- Source-agnostic reservation snapshots own promoted source record identities,
  related-record roles, reservation/stay grain, conservative statuses, owner/pet
  relationship shape, and typed assumptions. They must not own raw payload
  storage, Gingr-only provider semantics, or report-specific columns.
- Promotion owns the explicit crossing from Gingr provider vocabulary into the
  source-agnostic contract. It must not hide uncertainty; unresolved BI-question
  buckets become typed assumptions or data-quality issues.
- Data-quality issues own durable source-data problems with severity,
  provenance, and resolution status. They must not be hidden logs or one-off
  mapper errors only.
- Analytics/read models own denormalized facts and dimensions for reporting and
  local lab queries. They must not own provider DTO semantics or mutation
  workflows.
- Statum validator inputs own validated runtime evidence and blockers for
  workflow typestates. They must not interpret provider status strings directly.

## Raw Gingr DTO contract

Raw Gingr DTOs represent what Gingr returned, including partial schemas and messy
provider naming. Current examples include `response::OwnerRecord`,
`response::AnimalRecord`, `response::ReservationRecord`,
`response::ReferenceRecord`, `dto::retail::Item`, and verified webhook envelopes.

Raw DTOs must:

- preserve provider IDs and raw statuses when they are present;
- preserve unknown fields behind explicit raw/unknown-field containers;
- mark owner, animal, custom-field, form, and payment-sensitive payloads for
  quarantine or restricted access;
- keep request-builder assumptions visible, including date-window limits,
  location filters, API-user location scope, pagination, and historical cutover
  behavior;
- avoid implementing domain invariants directly.

Raw DTOs may:

- use provider names where that is the honest source vocabulary;
- be incomplete when public Gingr schemas are incomplete;
- expose narrow mapper inputs such as contact candidates, pet-name candidates,
  retail-product candidates, or verified webhook metadata.

Raw DTOs must not:

- be serialized as the BI model;
- be imported directly by dashboard/reporting code;
- skip redaction/quarantine for high-PII or payment-sensitive fields;
- invent typed coverage for unknown Gingr surfaces such as undocumented grooming
  or training details.

## Gingr snapshot contract

A Gingr snapshot is a stable, versioned source fact that can be persisted,
diffed, replayed, promoted, and projected. Snapshots are source-derived evidence,
not final domain truth.

Recommended first family: reservation/stay snapshots, because reservation facts
join customer, pet, location, service, lifecycle, payment, revenue, BI, and
workflow needs.

Candidate snapshot concepts:

- `source::gingr::reservation::Snapshot` for the current reservation/stay slice.
- Future Gingr customer, pet, stay, invoice, payment, and webhook-event snapshots
  only when an inspected artifact proves the grain and promotion target. Do not
  create empty modules for those families just because they are likely to matter.

Every snapshot must carry provenance:

- `source_system`: always Gingr for this contract.
- `endpoint_name`: API endpoint, export/report name, or webhook event lane.
- `provider_record_id`: Gingr record ID being represented.
- `related_provider_ids`: owner, animal, location, reservation type, invoice,
  payment, or service IDs.
- `extraction_batch_id`: batch/run that produced the snapshot.
- `pulled_at` / `received_at`: retrieval or webhook receipt timestamp.
- `request_scope`: subdomain/account label, location, date range, filters,
  pagination, and API-user caveat.
- `provider_schema_version`: known provider schema or local parser/schema
  version.
- `source_payload_hash`: hash of the quarantined raw source payload.
- `raw_payload_ref`: pointer to restricted local/object storage. Never inline
  production PII in normal logs.

Reservation/stay snapshots should preserve at least:

- provider reservation ID;
- owner, animal, location, reservation type, and service type IDs;
- provider status and lifecycle flags;
- check-in, check-out, start, end, cancellation, completion, and event times;
- whiteboard run/area and add-on service facts when available;
- invoice, estimate, transaction, payment, deposit, package, or subscription
  references when available;
- exact endpoint/request window that produced the row;
- webhook reconciliation facts when an event updates a periodically pulled source
  record.

Snapshots must be append/replay friendly. If later Gingr pulls conflict with
prior snapshots, keep both source facts and emit data-quality issues instead of
silently overwriting the contradiction.

## Source-agnostic promotion contract

Promotion turns Gingr source snapshots into NVA's source-agnostic contracts
through named validators and conversion functions. Promotion is where provider
strings, partial records, and ambiguous IDs become either source-level semantic
values, typed assumptions, or explicit data-quality issues. Domain entities and
workflow evidence may be built later from the promoted source contracts; they
should not consume Gingr adapter DTOs directly.

Promotion must:

- be explicit at every trust-boundary crossing;
- map provider IDs into semantic identity wrappers or documented external IDs;
- normalize provider statuses into domain enums only when their semantics are
  understood;
- preserve source provenance on promoted facts or associated audit records;
- return typed failures/issues for missing, conflicting, ambiguous, or unmapped
  facts;
- be covered by fixture-only tests before using live Gingr data.

Promotion must not:

- hide unknown statuses by defaulting to a happy-path enum variant;
- use report-specific BI column names in core domain entities;
- collapse owner/pet duplicate or merge ambiguity without a recorded issue;
- treat webhook facts as authoritative without reconciliation rules.

Canonical promotion path for the first slice:

```text
source::gingr::reservation::Snapshot
-> source::reservation::Snapshot
-> data_quality::Issue records for unresolved source problems
-> analytics::stay::Fact projection input
```

Existing narrow mappers for owner contact, pet name, and retail product
candidates can remain narrow. They should become inputs to fuller promotion only
when the required source facts and issue semantics are known.

## Data-quality issue contract

Data-quality issues are durable records that explain why a source fact is missing,
ambiguous, contradictory, unmapped, stale, or unsafe to project. They are not just
logs and not just mapper errors.

A source-data quality issue should include:

- `issue_id`: stable local issue identifier.
- `kind`: machine-readable issue kind.
- `severity`: informational, warning, blocking, or critical.
- `source_system`: Gingr.
- `provenance`: snapshot/provenance reference and extraction batch.
- `affected_provider_ids`: reservation, owner, animal, location, invoice,
  payment, or other affected provider IDs.
- `evidence`: redacted summary of the failing facts.
- `detected_at`: when NVA detected the issue.
- `resolution_status`: open, acknowledged, ignored, repaired, or superseded.
- `visible_to_bi`: whether read models should expose the issue.
- `workflow_blocking`: whether Statum validators must block or route for review.

Initial issue kinds should include:

- `MissingRequiredField`
- `UnknownProviderStatus`
- `ConflictingTimestamps`
- `DuplicateProviderRecord`
- `AmbiguousOwnerPetRelationship`
- `UnmappedServiceType`
- `LocationScopeAmbiguity`
- `PaymentStateConflict`
- `CheckoutEvidenceMissing`
- `UnclosedReservation`
- `IncompletePetProfile`
- `MissingVaccinationRecord`
- `SensitivePayloadQuarantined`

Data-quality issues should be queryable by BI/reporting, audit/replay jobs,
manager review queues, and workflow validators. A BI projection may include rows
with warning-level issues, but blocking issues must be visible in the projection
or excluded with an auditable reason.

## Analytics/read-model contract

Analytics/read models are stable, denormalized outputs built from promoted source
facts and data-quality signals. They optimize reporting and local lab queries;
they are not the source of operational truth.

Candidate first read models:

```rust
analytics::stay::Fact
analytics::customer::Dimension
analytics::pet::Dimension
analytics::revenue::Fact
analytics::service_utilization::Fact
analytics::capacity::Snapshot
analytics::retention::Opportunity
analytics::data_quality::IssueFact
```

A read-model row must carry:

- local read-model ID;
- source snapshot/provenance references;
- source freshness timestamp;
- semantic entity IDs or provider IDs where domain IDs do not exist yet;
- projection version;
- data-quality status and issue references;
- redaction/access classification for fields derived from PII or payment data.

A `stay` fact should be able to answer:

- which reservation/stay happened or is scheduled;
- which customer, pet, service, reservation type, and location are involved;
- lifecycle status and timestamps;
- check-in/check-out evidence;
- revenue/payment/deposit references when safe and available;
- whether the fact is complete, partial, disputed, or manager-review required;
- which source pull/webhook/batch generated the projected row.

Read models must be regenerated deterministically from snapshots. If a projection
uses a local lab DB, the database is an execution detail; the contract lives in
code, migrations/fixtures, and docs.

## Statum workflow-validator input contract

Future Statum validators should consume validated evidence, not raw Gingr status
strings. The validator input should be a semantic bundle derived from snapshots,
promotion, and data-quality issues.

A checkout validator input should include:

- reservation/stay identity;
- provider status and normalized domain status;
- check-in and check-out timestamps;
- payment/deposit state evidence;
- incident or manager-review indicators;
- blocking and warning data-quality issues;
- provenance references for every source fact used by the decision.

Example acceptance path:

```text
Gingr reservation status says checked out
+ checkout timestamp exists
+ payment/deposit evidence is settled or not required
+ no blocking incident or data-quality issue
-> CheckoutMachine<ReadyForCompletion>
```

Example review path:

```text
Gingr reservation status says checked out
+ checkout timestamp is missing or payment state conflicts
-> CheckoutMachine<NeedsManagerReview>
```

Example block path:

```text
Gingr reservation status is unknown to NVA
+ location scope is ambiguous
+ no reliable checkout evidence exists
-> CheckoutMachine<Blocked>
```

The validator contract should remain independent of the specific BI storage
engine. It should depend on semantic evidence structs and issue references that
can be assembled from snapshots or read models.

## Local-lab database recommendation

Do not create database artifacts in this documentation task.

For the next implementation lane, use a disposable local-lab database only to
exercise deterministic extraction/projection tests. Start with SQLite when the
immediate need is fast, local, fixture-driven projection tests and snapshot
replay. Prefer SQLite for:

- single-developer local tests;
- fixture snapshots and deterministic read-model assertions;
- validating projection contracts before real BI DB details are known;
- avoiding Docker/service setup for the first thin slice.

Promote to local Postgres later if the team needs:

- SQL semantics closer to an existing BI warehouse;
- concurrent ingestion/projection workers;
- richer JSON/index/query behavior;
- migration compatibility with a production Postgres-like target;
- integration testing against app/storage adapters that assume Postgres.

Recommended ignored local path for SQLite:

```text
.var/nva-source-lab.sqlite3
```

Recommended repo-owned artifacts, when a future code task creates them:

```text
analytics/migrations/
analytics/tests/fixtures/
integrations/gingr/tests/fixtures/
docs/integrations/gingr/bi-read-model-contract.md
```

Rules for any local-lab DB:

- never commit real Gingr/customer/payment data;
- store only synthetic or redacted fixtures in git;
- keep local DB files ignored;
- make projections replayable from checked-in fixtures;
- document import adapters separately when Tyler's existing BI database becomes
  inspectable.

## First thin-slice contract

The first implementation slice should be reservation/stay centered:

1. Capture a synthetic raw Gingr reservation response fixture.
2. Wrap it in a `ReservationSnapshot` with provenance and payload identity.
3. Promote it into semantic reservation/stay evidence.
4. Emit data-quality issues for missing checkout timestamp, unknown status,
   missing owner/animal/location, payment-state conflict, or location-scope
   ambiguity.
5. Project it into a deterministic `stay` fact with provenance and issue refs.
6. Build a checkout workflow-validator input from the same promoted evidence.

Useful test names:

```rust
gingr_reservation_snapshot_preserves_provider_provenance
reservation_snapshot_projects_to_stay_fact_with_issue_refs
missing_checkout_timestamp_marks_snapshot_as_data_quality_issue
checkout_validator_requires_semantic_evidence_not_raw_status
```

## Open questions before productionizing

- What database engine or warehouse currently stores the existing Gingr-derived
  BI data?
- Is that database raw-payload storage, normalized operational tables, BI
  fact/dimension tables, or a mixture?
- Which Gingr endpoints, reports, exports, or webhooks feed it?
- Can NVA inspect the messy code path that constructs current BI-facing types?
- Which provider IDs are stable enough for joins across endpoints, webhooks,
  invoices, payments, exports, and historical backfills?
- Which statuses, service names, reservation types, invoice states, and custom
  fields have location-specific or overloaded meanings?
- How should owner, animal, custom-field, and payment payloads be redacted or
  quarantined in source snapshot storage?
- Are webhook events used today for BI freshness, or only periodic pulls and
  exports?
- How are deleted, merged, duplicate, or re-created Gingr records represented?
- Which BI outputs are business-critical first: stay/utilization, revenue,
  capacity, retention, labor/revenue, service-line performance, or data-quality
  dashboards?

## Acceptance criteria for downstream code

A downstream code change satisfies this contract when it:

- keeps raw DTOs confined to the Gingr integration boundary;
- creates snapshots with required provenance and payload identity;
- promotes through named semantic functions or validators;
- emits first-class data-quality issue records instead of hiding contradictions;
- generates deterministic read-model rows from snapshots/promoted facts;
- carries issue/provenance refs into BI outputs;
- feeds Statum validators with semantic evidence, not raw status strings;
- uses only synthetic/redacted fixtures in git;
- keeps local database files disposable and ignored.
