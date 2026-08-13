# NVA Pet Resorts pilot pilot proof

This repository is a local, code-backed proof of safe AI-assisted operations workflows for NVA Pet Resorts.

It is meant to show how I would approach a first pilot before touching production systems: source-grounded workflow packets, review gates, blocked side effects, and measurable outcome capture.

## What this is

- A Rust-first local pilot pilot proof.
- A typed Pet Resorts domain and workflow model.
- A safe API/demo surface for reviewed internal work.
- A fixture-backed proof for two initial workflows:
  - Data Quality Hygiene
  - Manager Daily Brief
- A Gingr/provider adapter pilot proof for source evidence, not live production writes.

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

## Canonical docs path

Start here when orienting the repo or checking navigation. The current reader path is the root README, then the pilot workflows and safety boundaries above, then the code and proof surfaces below. Older architecture, Kanban, and presentation material is retained under [docs/internal](docs/internal/README.md) and should not override the current pilot pilot proof unless a task explicitly asks for archive archaeology.

## Presentation path: safe local owned API proof

Use the local demo walkthrough and fallback material only as safe, synthetic proof. No live NVA/Gingr data, customer sends, provider/PMS writes, schedule changes, payment actions, deployment, or measured NVA value claims are implied by the demo artifacts.

## Workspace navigation

- [Application workflow layer](app/README.md)
- [CLI shell](apps/cli/README.md)
- [API shell](apps/api/README.md)
- [SpacetimeDB shell](apps/spacetimedb/README.md)
- [Worker shell](apps/worker/README.md)
- [Domain semantic vocabulary](domain/README.md)
- [Storage projections](storage/README.md)
- [Gingr provider adapter](integrations/gingr/README.md)

## Service-line and provider proof surfaces

- [Boarding domain summary](domain/src/boarding/README.md)
- [Daycare domain summary](domain/src/daycare/README.md)
- [Grooming domain summary](domain/src/grooming/README.md)
- [Money domain summary](domain/src/money/README.md)
- [Payment domain summary](domain/src/payment/README.md)
- [Reservation domain summary](domain/src/reservation/README.md)
- [Retail domain summary](domain/src/retail/README.md)
- [Training domain summary](domain/src/training/README.md)
- [Storage service-line adapters](storage/src/service_line/README.md)
- [Gingr endpoint builders](integrations/gingr/src/endpoint/README.md)
- [Gingr DTO quarantine](integrations/gingr/src/dto/README.md)
- [Gingr mapping boundary](integrations/gingr/src/mapping/README.md)
- [Gingr docs index](docs/integrations/gingr/README.md)
- [Gingr webhook fixtures](docs/integrations/gingr/fixtures/webhooks/README.md)

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
