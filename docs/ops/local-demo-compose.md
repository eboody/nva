# Local Docker Compose demo runbook

This runbook starts the safe local demo stack only. It uses synthetic seed data, fake/deterministic agent posture, and side-effect-disabled flags. It does not use NVA/Gingr production credentials and does not perform live customer sends, provider/PMS writes, payment movement, schedule changes, or medical/safety decisions.

## One-command information-lifespan demo

From the repository root:

```sh
./scripts/demo_information_lifespan.sh
```

That is the preferred operator path for the NVA/job-contact/CEO/CTO information-lifespan demo. It performs the full local sequence:

1. Builds and runs the `hermes-processor` container against `fixtures/information-lifespan/hermes-processor-input.json`.
2. Validates `.var/information-lifespan/processor-output.json` and `.var/information-lifespan/processor-log.jsonl` against the deterministic output contract.
3. Starts the Compose stack with Postgres, MinIO, the one-shot `migrate-seed` service, `pet-resort-api`, `pet-resort-worker`, and `staff-web`.
4. Waits for API `/v0/healthz`, API `/v0/readyz`, and the staff-web root page.
5. Calls `POST /v0/demo/information-lifespan/run` and `GET /v0/demo/information-lifespan/{correlation_id}/report`.
6. Verifies that the API response is using the Dockerized Hermes processor artifact, not the labeled local fallback.
7. Runs `./scripts/smoke_local_demo.sh` for the broader compose/database/read-model smoke.

Proof artifacts are written to `.var/information-lifespan/` for processor files and `.artifacts/information-lifespan/` for script/API/transcript files:

- `.artifacts/information-lifespan/demo-information-lifespan-transcript.log` — exact command transcript and helpful failure logs.
- `.var/information-lifespan/processor-output.json` — deterministic Hermes processor output.
- `.var/information-lifespan/processor-log.jsonl` — structured processor events, including report enrichment and locked side effects.
- `.artifacts/information-lifespan/api-information-lifespan-run.json` — API run response with trace, DB proof, network proof, processor proof, calculations, safety locks, and final Manager Daily Report.
- `.artifacts/information-lifespan/api-information-lifespan-report.json` — report replay response for the same deterministic correlation id.

Expected success anchors include:

```text
processor_contract_ok correlation_id=info-lifespan-demo-2026-06-29 ... reported_estimated_labor_minutes_difference=42
api_run_ok correlation_id=info-lifespan-demo-2026-06-29
api_report_replay_ok final_report=artifact://manager-daily-report/synthetic-2026-06-29
[smoke-local-demo] core_demo_ok local_fixture_only=true live_side_effects_allowed=false optional_agent_infra_required=0
information_lifespan_demo_ok local_fixture_only=true dockerized_hermes_processor=true live_side_effects_allowed=false
```

If the script fails, it exits nonzero and prints `docker compose ps` plus recent logs for Postgres, MinIO, migration/seed, Hermes processor, API, worker, and staff-web. The transcript path is printed near the top of the run.

## Presenter walkthrough and fallback path

Use [Information lifespan demo presenter walkthrough](../demo/information-lifespan-presenter-walkthrough.md) as the 3-5 minute talk track for a job-contact/CEO/CTO demo. It gives the spoken sequence, click order, labels, and fallback wording for the complete Rube Goldberg path:

```text
mocked Gingr/source evidence
  -> provider/source model
  -> source snapshot + provenance
  -> NVA-owned models
  -> local DB/projection proof
  -> Dockerized Hermes processor output/logs
  -> API/network response proof
  -> Manager Daily Report with lineage, locks, calculations, and labor value
```

If the live stack fails during a conversation, do not describe stale output as live proof. Switch to the walkthrough's fallback packet: the board artifact, last successful transcript under `.artifacts/information-lifespan/`, saved API/report JSON, saved processor JSON/JSONL, and any explicitly labeled screenshot outside git such as `/tmp/nva-information-lifespan-demo.png`.

## Compose-only core demo

If you only need the core Compose stack after the processor artifact already exists, run:

```sh
docker compose up --build -d --wait pet-resort-api pet-resort-worker staff-web
./scripts/smoke_local_demo.sh
```

If local default ports are already allocated, use the same overrides for both Compose and the smoke/demo script:

```sh
PET_RESORT_API_HOST_PORT=33101 \
PET_RESORT_STAFF_WEB_HOST_PORT=33100 \
PET_RESORT_POSTGRES_HOST_PORT=35532 \
PET_RESORT_MINIO_HOST_PORT=39020 \
PET_RESORT_MINIO_CONSOLE_HOST_PORT=39021 \
  ./scripts/demo_information_lifespan.sh
```

`docker compose up --build -d --wait pet-resort-api pet-resort-worker staff-web` starts the core stack and its dependencies:

- `postgres` with non-secret local credentials (`pet_resort` / `pet_resort`)
- `minio` with non-secret local credentials (`pet_resort_local` / `pet_resort_local_password`)
- `migrate-seed`, a one-shot service that applies `migrations/*.sql`, `fixtures/seed/local-demo.sql`, and `fixtures/seed/local-demo-data-quality.sql`
- `hermes-processor`, a deterministic local bridge run by `./scripts/demo_information_lifespan.sh` rather than a long-lived daemon
- `pet-resort-api` on `127.0.0.1:${PET_RESORT_API_HOST_PORT:-3001}` with `.var/information-lifespan` mounted read-only so `/v0/demo/information-lifespan/*` can show the processor artifact
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

OpenViking is not part of the default local demo stack.

```sh
REQUIRE_OPTIONAL_AGENT_INFRA=1 ./scripts/smoke_local_demo.sh
```

## Reset local state

To discard local Postgres/MinIO/OpenViking volumes and rebuild from scratch:

```sh
docker compose down -v
./scripts/demo_information_lifespan.sh
```
