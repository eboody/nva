#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
: "${DATABASE_URL:=postgres://pet_resort:pet_resort@127.0.0.1:${PET_RESORT_POSTGRES_HOST_PORT:-54329}/pet_resort}"

if command -v psql >/dev/null 2>&1; then
  for migration in migrations/*.sql; do
    echo "[migrate-local] applying ${migration}"
    psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 -f "${migration}"
  done
  echo "[migrate-local] applying synthetic local demo seed"
  psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 -f fixtures/seed/local-demo.sql
  psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 -f fixtures/seed/local-demo-data-quality.sql
  echo "[migrate-local] local schema and seed are ready"
  exit 0
fi

if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
  echo "[migrate-local] psql not found locally; using docker compose migrate-seed service"
  docker compose run --rm migrate-seed
  exit 0
fi

echo "psql or Docker Compose is required to apply local migrations and seed data." >&2
echo "Install postgresql-client, or run: docker compose run --rm migrate-seed" >&2
exit 1
