# SpacetimeDB realtime queue demo

This is the smallest local demo for the Data-Quality Hygiene realtime queue slice.
It is meant for a presenter who needs to show the value of realtime staff/manager
queue updates even on a laptop where the SpacetimeDB host is not compatible with
the module ABI.

## What the demo proves

The script uses deterministic fixture actors and issue data:

- Alice: front desk lead scoped to Location 101.
- Sam: front desk lead scoped to Location 202.
- Morgan: general manager scoped to Location 101.
- Queue item: `dq-action-location-101`, seeded from `gingr:reservation:abc` and
  requiring manager approval.

Required demo moments:

1. Alice's Location 101 `staff_queue_item` subscription receives updates without a
   refresh/static re-query.
2. Sam's Location 202 view does not receive Location 101 work.
3. Sam's attempted Location 101 mutation is blocked and projected to
   `blocked_action_notice`.
4. Morgan's Location 101 `manager_queue_item` view sees the manager-gated row and
   records the manager outcome.
5. An unsafe live customer/provider side effect stays blocked, with audit/outcome
   evidence in the terminal.

## Run it

From the repository root:

```sh
scripts/spacetimedb_realtime_queue_demo.sh
```

For a fast deterministic run that skips the publish probe:

```sh
scripts/spacetimedb_realtime_queue_demo.sh --force-fallback
```

For CI/smoke validation of the presenter markers:

```sh
scripts/spacetimedb_realtime_queue_demo.sh --self-test
```

Expected self-test output:

```text
spacetimedb realtime queue demo self-test passed
```

## What to watch for

The script prints subscription-prefixed lines. Those are the presenter's cue that a
view changed after a reducer-style step rather than from a static dump.

Important markers:

- `ALICE_UPDATE_SEEN`
- `SAM_LOCATION_101_HIDDEN`
- `SAM_MUTATION_BLOCKED`
- `MORGAN_ACTION_SEEN`
- `UNSAFE_SIDE_EFFECT_BLOCKED`
- `AUDIT_OUTCOME_EVIDENCE`

## SpacetimeDB fallback behavior

The script first probes a local publish unless `--force-fallback` or `--no-probe`
is passed. It starts an in-memory local server, builds `apps/spacetimedb`, and tries
to publish a disposable database.

On this workstation, `spacetime build --project-path apps/spacetimedb` succeeds, but
`spacetime publish` currently fails because the local standalone host implements ABI
`10.0` while the module built from the current Rust SDK reports ABI `10.4`:

```text
Error: abi version 10.4 is not supported (host implements 10.0)
```

When that happens, the script intentionally falls back to a deterministic terminal
event-stream demo. The fallback uses the same actor/location fixture, reducer names,
public read-model names, and safety/audit moments, but it is not a live SpacetimeDB
subscription. Do not describe it as a successful module publish.

If a compatible SpacetimeDB host is installed later, use the probe output plus these
manual commands for deeper inspection:

```sh
spacetime start --listen-addr 127.0.0.1:3011 --in-memory
spacetime publish --server http://127.0.0.1:3011 --anonymous --yes --delete-data \
  --project-path apps/spacetimedb nva-realtime-queue-demo
spacetime subscribe --server http://127.0.0.1:3011 --anonymous --print-initial-update \
  nva-realtime-queue-demo 'SELECT * FROM staff_queue_item'
```

Regenerate or extend client bindings only after the ABI/runtime mismatch is cleared.
