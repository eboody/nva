# Pet Resort MVP Project Skeleton Runbook

Status: local/CI skeleton implemented; no production deployment, live customer messaging, provider writes, payment actions, or autonomous eligibility decisions are enabled.

## Stack implemented

This skeleton follows `docs/roadmap/pet-resort-mvp-stack.md` and the roadmap handoff:

- Rust workspace remains the source-of-truth backend/domain boundary.
- `apps/api` is an Axum API shell with `/healthz` and `/readyz`.
- `apps/worker` is a worker shell with fake deterministic agent runtime and stubbed side-effect mode as defaults.
- `apps/staff-web` is a Next.js/TypeScript staff-dashboard shell.
- PostgreSQL and MinIO are local Docker Compose dependencies.
- SQLx migrations live in `migrations/`; the first migration is a baseline smoke migration for downstream data-model work.
- Local fixtures live in `fixtures/seed/` and `fixtures/smoke/`.
- CI gates Rust format/clippy/tests, frontend typecheck/lint, and migration application against disposable Postgres.

## Layout

```text
apps/
  api/          Rust Axum API shell
  worker/       Rust workflow worker shell
  staff-web/    Next.js staff/admin UI shell
domain/         Existing canonical business/domain types
storage/        Existing storage contract crate
integrations/   Existing external-system adapters
migrations/     SQLx migration files
fixtures/       Local seed and smoke fixtures
scripts/        Local dev, test, migration, and smoke helpers
.github/        CI workflow
```

## Local development

1. Copy local env and start dependencies:

   ```bash
   cp .env.example .env
   ./scripts/dev.sh
   ```

2. Run the API shell:

   ```bash
   cargo run -p pet-resort-api
   ```

3. Run the worker shell:

   ```bash
   cargo run -p pet-resort-worker
   ```

4. Install and run the staff UI:

   ```bash
   npm install
   npm --workspace @pet-resort/staff-web run dev
   ```

5. Apply local migrations after installing SQLx CLI:

   ```bash
   cargo install sqlx-cli --no-default-features --features postgres,rustls
   ./scripts/migrate-local.sh
   ```

## Quality gates

Rust:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Frontend after `npm install`:

```bash
npm --workspace @pet-resort/staff-web run typecheck
npm --workspace @pet-resort/staff-web run lint
```

Migration smoke against local Postgres:

```bash
./scripts/migrate-local.sh
```

API smoke:

```bash
./scripts/smoke-local.sh
```

## Preserved approval gates

The skeleton intentionally leaves these disabled or stubbed:

- Production deployment.
- Live customer messaging.
- Live Gingr/PMS/provider writes.
- Booking confirmation/rejection automation.
- Vaccine/medical/document auto-acceptance.
- Incident owner-message sends and eligibility-affecting behavior flags.
- Payment provider selection, checkout links, charges, refunds, discounts, waivers, or webhooks.

Downstream feature cards should wire vertical slices through typed events, validated workflow results, review packets, audit events, and human approval records instead of adding direct side-effect shortcuts.
