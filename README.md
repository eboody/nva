# NVA Pet Resorts pilot scaffold

This repository is a local, code-backed proof of safe AI-assisted operations workflows for NVA Pet Resorts.

It is meant to show how I would approach a first pilot before touching production systems: source-grounded workflow packets, review gates, blocked side effects, and measurable outcome capture.

## What this is

- A Rust-first local pilot scaffold.
- A typed Pet Resorts domain and workflow model.
- A safe API/demo surface for reviewed internal work.
- A fixture-backed proof for two initial workflows:
  - Data Quality Hygiene
  - Manager Daily Brief
- A Gingr/provider adapter scaffold for source evidence, not live production writes.

## What this is not

- Not connected to live NVA systems.
- Not a production Gingr integration.
- Not an autonomous agent that can message customers, update reservations, move money, or change schedules.
- Not a claim of measured NVA labor savings yet.

## First pilot workflows

1. [Data Quality Hygiene](docs/pilot/data-quality-cleanup.md)
2. [Manager Daily Brief](docs/pilot/manager-daily-brief.md)

Safety framing:

- [Safety boundaries](docs/pilot/safety-boundaries.md)
- [Access needed next](docs/pilot/next-access-needed.md)

## Run the local pilot proof

```sh
./scripts/demo_nva_pilot.sh
```

The demo uses local/fixture data and prints the two workflow smoke paths. Live side effects remain disabled.

## Useful checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets --locked
npm ci
npm --workspace @pet-resort/staff-web run typecheck
npm --workspace @pet-resort/staff-web run lint
```

## Internal archive

Architecture audits, Kanban boards, broad presentation material, glossary inventories, and exploratory infrastructure notes are retained under [docs/internal](docs/internal/README.md). They are not the recommended first read for NVA stakeholders.
