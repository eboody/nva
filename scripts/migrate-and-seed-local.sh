#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

compose_bin=(docker compose)
if ! docker compose version >/dev/null 2>&1; then
  if command -v docker-compose >/dev/null 2>&1; then
    compose_bin=(docker-compose)
  else
    echo "docker compose is required" >&2
    exit 1
  fi
fi

"${compose_bin[@]}" up -d postgres

for _ in {1..60}; do
  if "${compose_bin[@]}" exec -T postgres pg_isready -U pet_resort -d pet_resort >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

"${compose_bin[@]}" exec -T postgres pg_isready -U pet_resort -d pet_resort >/dev/null

for migration in migrations/0001_mvp_foundation.sql migrations/0002_data_quality_read_models.sql; do
  echo "applying ${migration}"
  "${compose_bin[@]}" exec -T postgres psql -v ON_ERROR_STOP=1 -U pet_resort -d pet_resort < "${migration}"
done

echo "seeding safe synthetic local demo data"
"${compose_bin[@]}" exec -T postgres psql -v ON_ERROR_STOP=1 -U pet_resort -d pet_resort < fixtures/seed/local-demo.sql
"${compose_bin[@]}" exec -T postgres psql -v ON_ERROR_STOP=1 -U pet_resort -d pet_resort < fixtures/seed/local-demo-data-quality.sql

"${compose_bin[@]}" exec -T postgres psql -U pet_resort -d pet_resort -c \
  "SELECT 'source_quality_backlog' AS read_model, count(*) FROM source_quality_backlog UNION ALL SELECT 'data_quality_hygiene_labor_outcomes', count(*) FROM data_quality_hygiene_labor_outcomes UNION ALL SELECT 'import_freshness', count(*) FROM import_freshness;"
