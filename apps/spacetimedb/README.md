# NVA SpacetimeDB runtime adapter

This crate is the realtime storage/runtime adapter for NVA operations workflows.
It lives under `apps/spacetimedb/` because a SpacetimeDB module is deployed as a
runtime entrypoint, not as the canonical domain or app-service layer.

Layering rules:

- `domain` and `app` stay authoritative for vocabulary, invariants, workflow
  policy, and app-owned ports.
- SpacetimeDB `#[table]` structs are storage rows or public subscription read
  models, not domain entities.
- Private persisted/queryable facts live under `storage/<workflow>/row.rs` and
  include SpacetimeDB metadata such as indexes, timestamps, and `schema_version`.
- Small nested storage/value columns use `#[derive(spacetimedb::SpacetimeType)]`
  under `storage/<workflow>/status_column.rs`; indexed/queryable fields such as
  `location_id`, `actor_id`, `status`, `source_ref_id`, `created_at`, and
  `updated_at` stay flattened on rows.
- Translation lives in explicit `storage/<workflow>/codec.rs` functions. Storage
  to app/domain promotion is fallible when persisted facts need validation;
  app/domain to storage/read-model projection is named and local to the adapter.
- Public tables under `read_model/` are intentional client subscription contracts
  for realtime dashboards. They are denormalized projections from private rows
  and app/domain decisions, not accidental exposure of persistence internals.
- Reducers in `reducers/` or `reducers.rs` are thin adapters: sender identity ->
  app actor -> app service -> storage/read-model/audit rows.
- Reducers must not call external customer, provider/PMS, payment, schedule,
  medical, or safety systems. Live side effects stay outside this module.

Current review-queue local contract surface:

```text
src/storage/review_queue/row.rs             # private persisted/queryable rows
src/storage/review_queue/status_column.rs   # SpacetimeType storage columns
src/storage/review_queue/codec.rs           # explicit conversions/projections
src/read_model/staff_queue_item.rs          # staff subscription contracts
src/read_model/manager_queue_item.rs        # manager subscription contracts
src/reducers.rs                             # current thin reducer entrypoint
```

Local verification posture:

```sh
cargo check -p nva-spacetimedb
cargo check --workspace
command -v spacetime && spacetime --version || true
```

If the SpacetimeDB CLI is installed, also run from the repository root:

```sh
spacetime build --project-path apps/spacetimedb
scripts/spacetimedb_realtime_queue_demo.sh --self-test
```

Presenter demo:

```sh
scripts/spacetimedb_realtime_queue_demo.sh
```

The presenter script probes a local SpacetimeDB publish first, then falls back to a
fully deterministic terminal event-stream if the local host cannot publish the module.
See `docs/ops/spacetimedb-realtime-queue-demo.md` for the runbook and for the
currently observed ABI mismatch fallback (`module abi 10.4`, host `10.0`).

Install the CLI if needed:

```sh
curl -sSf https://install.spacetimedb.com | sh
```
