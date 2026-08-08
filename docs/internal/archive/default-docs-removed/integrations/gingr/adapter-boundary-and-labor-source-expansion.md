# Adapter boundary and labor-source expansion points

Date: 2026-06-17

## Purpose

This note helps regional ops leaders and engineering maintainers avoid re-reading
raw Gingr reservation exports every time a boarding exception queue changes. It
shows which Gingr reservation/stay facts can explain an open stay, checkout
exception, or capacity mismatch, which source-agnostic records carry that
evidence forward, and which live actions still require human-approved workflow
contracts.

Gingr contributes reservation/stay, customer/pet, service, revenue, location, and
limited labor/report evidence. That evidence can feed source-backed summaries and
review queues, but it does not authorize schedule changes, checkout decisions, or
customer outreach by itself, and its DTO names must not become the domain core.

## Final boundary chain

The allowed reservation/stay flow is:

```text
Gingr DTO / endpoint response
-> source::gingr::reservation::Snapshot
-> source::reservation::Snapshot
-> analytics::stay::Fact
-> future workflow-validator evidence
```

Each arrow is an explicit trust-boundary crossing:

1. **Gingr DTO -> Gingr snapshot**
   - Owns provider-shaped payload vocabulary, provider IDs, endpoint/report
     names, API-user/request scope, provider schema/parser version, and raw
     provider status text.
   - Preserves payload identity and provenance without deciding NVA business
     truth.
   - May speak Gingr names such as provider record ID, owner provider ID, animal
     provider ID, location provider ID, service type provider ID, endpoint, and
     provider status.

2. **Gingr snapshot -> source-agnostic reservation snapshot**
   - Promotes provider values through named conversion/promotion operations.
   - Replaces Gingr-specific identities with `source::record::*` identities and
     roles.
   - Replaces provider status strings with `source::reservation::Status` only
     when the mapping is understood; otherwise it carries an observed unknown
     status and/or typed data-quality issue.
   - Carries assumptions tied to the BI question buckets, such as provisional
     grain, join-key stability, raw-payload retention, and refresh/mutation
     behavior. Assumptions are not TODO comments; they are part of the fact.

3. **Source-agnostic reservation snapshot -> analytics stay fact**
   - `analytics::stay::Fact` projects from `source::reservation::Snapshot`, not
     from `source::gingr::*`.
   - The stay fact is a deterministic reporting/read-model fact with provenance,
     projection version, source freshness, semantic record references, and
     data-quality status.
   - It is not the source of operational truth and must be regenerable from
     snapshots and fixture inputs.

4. **Analytics/source evidence -> future workflow validators**
   - Workflow validators should consume semantic evidence and issue references,
     not raw Gingr statuses or BI column names.
   - A validator may depend on a stay fact, source snapshot evidence, and
     data-quality issues, but the validator contract must explain which evidence
     is required and what blocks, warns, or routes to manager review.

## Ownership rules

- `source::gingr::*` owns Gingr vocabulary only: endpoints, provider IDs,
  provider statuses, provider-shaped request scope, and Gingr snapshot promotion.
- `source::*`, `source::record::*`, and `source::reservation::*` own facts any
  source can provide: source system, provenance, source record identity, related
  record roles, reservation/stay snapshot grain, status, owner/pet relationship,
  and typed assumptions.
- `data_quality::*` may point to `source::Provenance` and source field paths, but
  must not mention Gingr-only types as its required input surface.
- `analytics::*` may depend on `source::reservation::Snapshot` and source-
  agnostic identifiers, but must not import Gingr adapter types.
- Future workflow modules may depend on semantic evidence bundles and issue
  references, but must not interpret provider status strings directly.
- BI discovery docs decide what is safe to model next. Unknowns become typed
  assumptions or data-quality issues instead of hard-coded Gingr/BI truths.

## Current code surface to preserve

Current source-contract work established these public shapes:

```rust
source::System
source::Provenance
source::record::Id
source::record::RelatedId
source::record::Role
source::reservation::Snapshot
source::reservation::Status
source::reservation::Assumption
source::gingr::Provenance
source::gingr::ProviderRecordId
source::gingr::ProviderStatus
source::gingr::reservation::Snapshot
analytics::stay::Fact::project_from_source_reservation(...)
```

Do not add masking aliases that let downstream code pretend Gingr and the core
source model are the same thing. If a call site needs Gingr details, it belongs
inside the Gingr adapter namespace. If a call site is analytics, data quality, or
workflow validation, it should consume the promoted source-agnostic contract.

## Expansion model for labor-cost sources

Labor-cost work needs more than Gingr. Later sources should join through the
same source-agnostic contracts, not by creating empty crates or premature module
families now.

Known future source families:

- **Timeclock**: clock-in/clock-out evidence, paid hours, role/location context,
  and edit/approval provenance when available.
- **Payroll/wage cost**: wage-rate or payroll-cost facts, pay period boundaries,
  role/location attribution, and confidence limits around what can be joined.
- **Labor scheduling**: scheduled shifts, roles, locations, planned coverage,
  callouts, swaps, and manager approvals when available.
- **Capacity inventory**: rooms/runs/kennels/service capacity, location/service
  constraints, and planned-vs-actual occupancy context.
- **POS/payments and BI/manual imports**: revenue/payment facts and trusted or
  disputed metric definitions when BI answers expose them.

These are source families, not implementation crates by default. Add concrete
modules, adapters, or crates only when a BI answer or inspected artifact proves a
stable grain, identity, provenance shape, and first projection need.

## How future sources plug in

For each new source family, use the same sequence:

1. Capture the raw source artifact or a synthetic/redacted fixture summary.
2. Define the source-specific adapter snapshot only as far as the artifact proves
   the provider/source vocabulary.
3. Promote into an existing or narrowly added `source::*` contract with explicit
   provenance, record identity, related-record roles, and assumptions.
4. Emit `data_quality::Issue` records for missing, ambiguous, contradictory,
   stale, or unsafe joins.
5. Project into analytics only after the grain and join assumptions are visible.
6. Feed workflow validators only with semantic evidence and data-quality issue
   references.

Examples of future projections are labor-cost-per-stay, scheduled-vs-actual
coverage, occupancy-to-staffing pressure, and service-line labor intensity. Those
should wait for BI/discovery answers about scheduling, timeclock, payroll, and
capacity grains. Until then, the docs may name the source family and question
bucket, but should not invent fields, tables, or operational rules.

## Guardrails

- Do not create `timeclock`, `payroll`, `scheduling`, or `capacity` crates merely
  because the strategy needs those sources eventually.
- Do not let a Gingr reservation snapshot grow fields that actually belong to
  payroll, scheduling, capacity, or BI metric reconciliation.
- Do not join source records by provider ID names alone; record the source system,
  record role, provenance, and any join-key assumptions.
- Do not treat BI tables as truth until the question/rubric docs identify their
  grain, refresh/mutation behavior, provenance, and trusted metric status.
- Do not build workflow validators on raw provider statuses. Status ambiguity is
  a data-quality issue or typed assumption until validated.
