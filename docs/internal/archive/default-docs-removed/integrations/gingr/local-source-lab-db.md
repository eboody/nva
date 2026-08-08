# Gingr local source-lab database decision

Decision date: 2026-06-16

## Decision

Do not provision a disposable local source-lab database for the first Gingr
projection tests yet.

The first slice should stay fixture-only and function-level: synthetic raw Gingr
responses become snapshot-like test inputs, promotion outputs, data-quality issue
records, and read-model rows that can be asserted directly in Rust tests. A local
SQLite file would be useful once tests need replay across SQL migrations or need
to verify SQL projection queries, but it would add storage surface before the
snapshot and projection contracts exist in code.

## Evidence from the current repo

Current static inspection found:

- `integrations/gingr` contains provider DTOs, endpoint builders, webhook
  verification, and narrow mappers for contact, pet-name, and retail-product
  candidates.
- The repo does not yet contain Gingr source snapshots, provenance records,
  source-data-quality issue records, or Gingr analytics/read-model projection
  types.
- Existing migration coverage under `storage` is for the MVP app schema, not a
  Gingr source-lab schema.
- `docs/integrations/gingr/bi-read-model-contract.md` already defines the desired
  source flow and recommends SQLite only for a later deterministic local-lab
  execution detail.

Because there is no committed Gingr projection/storage implementation yet, a DB
would not materially improve the first tests. It would mostly test schema shape
before the semantic contract is executable.

## First projection-test posture

Keep the first tests independent of a database:

1. Use synthetic or redacted fixture payloads only.
2. Parse raw Gingr DTOs at the provider boundary.
3. Build explicit snapshot/provenance values in memory.
4. Promote through named semantic conversion or validator functions.
5. Emit data-quality issues as first-class values.
6. Project deterministic read-model rows in memory.
7. Assert provenance, issue references, freshness, and redaction/access
   classification directly.

This keeps the first test suite focused on semantic truth rather than SQL
plumbing. The call sites should read like an executable glossary of the source
contract, for example:

```rust
gingr_reservation_snapshot_preserves_provider_provenance
reservation_snapshot_projects_to_stay_fact_with_issue_refs
missing_checkout_timestamp_marks_snapshot_as_data_quality_issue
checkout_validator_requires_semantic_evidence_not_raw_status
```

## When to introduce the SQLite lab DB

Introduce the local SQLite source-lab database when at least one projection test
needs a real SQL boundary, such as:

- replaying multiple source snapshots through an ordered migration/projection
  pipeline;
- validating SQL views or denormalized fact/dimension tables;
- checking idempotent upsert behavior for snapshots, data-quality issues, or
  read-model facts;
- asserting join behavior across reservation, owner, animal, location, payment,
  and issue tables;
- preserving generated projection rows across process boundaries during a test.

At that point, use this ignored disposable path:

```text
.var/nva-source-lab.sqlite3
```

The database file is local runtime state only. It must be reproducible from
committed migrations, schema files, and synthetic/redacted fixtures.

## Repo-owned artifacts for the later SQLite step

When the SQL boundary becomes useful, commit only source artifacts such as:

```text
analytics/migrations/
analytics/tests/fixtures/
integrations/gingr/tests/fixtures/
docs/integrations/gingr/local-source-lab-db.md
```

Do not commit `.var/nva-source-lab.sqlite3`, WAL/SHM sidecars, exported real
Gingr payloads, customer data, payment data, secrets, or credentials.

## Guardrails

- Gingr remains an upstream operational source, not NVA's domain model.
- Raw DTOs and provider field names stay quarantined at the Gingr boundary.
- Domain promotion must cross a named semantic conversion/validation boundary.
- Data-quality issues should be durable test values, not logs hidden inside a
  mapper.
- Read models must be deterministic from snapshots and fixtures; the local DB is
  never the contract.
- Tyler's existing BI database should be inspected separately before copying any
  warehouse-specific assumptions into NVA.

## Current repository change

This decision records the no-DB stance and adds a git ignore rule for `.var/` so
future disposable local database files do not get staged accidentally. No local
SQLite database, migrations, schema files, or fixtures are created by this task.
