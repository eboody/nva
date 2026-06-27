# Local Docker Compose demo runbook

This runbook starts the safe local demo stack only. It uses synthetic seed data, fake/deterministic agent posture, and side-effect-disabled flags. It does not use NVA/Gingr production credentials and does not perform live customer sends, provider/PMS writes, payment movement, schedule changes, or medical/safety decisions.

## Core demo

From the repository root:

```sh
docker compose up --build -d --wait
./scripts/smoke_local_demo.sh
```

If local default ports are already allocated, use the same overrides for both Compose and the smoke script:

```sh
PET_RESORT_API_HOST_PORT=33101 \
PET_RESORT_STAFF_WEB_HOST_PORT=33100 \
PET_RESORT_POSTGRES_HOST_PORT=35532 \
PET_RESORT_MINIO_HOST_PORT=39020 \
PET_RESORT_MINIO_CONSOLE_HOST_PORT=39021 \
  docker compose up --build -d --wait

PET_RESORT_API_HOST_PORT=33101 \
PET_RESORT_STAFF_WEB_HOST_PORT=33100 \
PET_RESORT_POSTGRES_HOST_PORT=35532 \
PET_RESORT_MINIO_HOST_PORT=39020 \
PET_RESORT_MINIO_CONSOLE_HOST_PORT=39021 \
  ./scripts/smoke_local_demo.sh
```

`docker compose up --build -d --wait` starts the core stack:

- `postgres` with non-secret local credentials (`pet_resort` / `pet_resort`)
- `minio` with non-secret local credentials (`pet_resort_local` / `pet_resort_local_password`)
- `migrate-seed`, a one-shot service that applies `migrations/*.sql`, `fixtures/seed/local-demo.sql`, and `fixtures/seed/local-demo-data-quality.sql`
- `pet-resort-api` on `127.0.0.1:${PET_RESORT_API_HOST_PORT:-3001}`
- `pet-resort-worker` in fake/stubbed mode
- `staff-web` on `127.0.0.1:${PET_RESORT_STAFF_WEB_HOST_PORT:-3000}`. Its browser code defaults to the same-origin `/api/local-demo` proxy; the server-side proxy uses `PET_RESORT_API_BASE_URL=http://pet-resort-api:3001`. `NEXT_PUBLIC_PET_RESORT_API_BASE_URL` is only for direct browser fetch overrides outside the Compose proxy path, so Compose intentionally does not set it.

The smoke script verifies:

- API `/v0/healthz` and `/v0/readyz`
- fake deterministic runtime and disabled live customer/provider side effects
- staff-web reachability
- seeded Postgres tables/views for the Data-Quality Hygiene slice
- outbox handoff records remain internal and `live_delivery_allowed=false`
- optional agent-infra profile state is reported separately from core demo failures

## Migration and seed only

If the database is already running and you only need to re-apply schema/seed data:

```sh
docker compose run --rm migrate-seed
```

or, with local `psql` installed:

```sh
./scripts/migrate-local.sh
```

Both paths use idempotent SQL for the local demo seed.

## Optional agent infrastructure

OpenViking is optional and profile-gated. It is intentionally not part of the core demo health path:

```sh
docker compose --profile agent-infra up --build -d
./scripts/smoke_local_demo.sh
```

By default, an unhealthy optional `openviking` service is reported as an optional warning and does not fail the core smoke. To make optional infra strict, run:

```sh
REQUIRE_OPTIONAL_AGENT_INFRA=1 ./scripts/smoke_local_demo.sh
```

## Reset local state

To discard local Postgres/MinIO/OpenViking volumes and rebuild from scratch:

```sh
docker compose down -v
docker compose up --build -d
./scripts/smoke_local_demo.sh
```
