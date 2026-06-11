#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
: "${DATABASE_URL:=postgres://pet_resort:pet_resort@127.0.0.1:54329/pet_resort}"

if ! command -v sqlx >/dev/null 2>&1; then
  echo "sqlx CLI is required for migrations. Install with: cargo install sqlx-cli --no-default-features --features postgres,rustls" >&2
  exit 1
fi

sqlx migrate run --source migrations --database-url "$DATABASE_URL"
