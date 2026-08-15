#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

PET_RESORT_API_HOST_PORT="${PET_RESORT_API_HOST_PORT:-3001}"
PET_RESORT_API_URL="${PET_RESORT_API_URL:-http://127.0.0.1:${PET_RESORT_API_HOST_PORT}}"
SMOKE_TMP_DIR="${SMOKE_TMP_DIR:-$(mktemp -d)}"
export PET_RESORT_API_URL

log() {
  printf '[manager-daily-brief-smoke] %s\n' "$*" >&2
}

require() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'missing required command: %s\n' "$1" >&2
    exit 127
  fi
}

wait_for_http() {
  local url="$1"
  local label="$2"
  local attempts="${3:-60}"
  local delay="${4:-2}"
  local i
  for ((i = 1; i <= attempts; i++)); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      log "$label is reachable at $url"
      return 0
    fi
    sleep "$delay"
  done
  printf '%s did not become reachable after %s attempts\n' "$label" "$attempts" >&2
  exit 1
}

require docker
require curl
require python

log "starting the local fail-closed API stack"
docker compose up --build -d postgres minio pet-resort-api pet-resort-worker
wait_for_http "${PET_RESORT_API_URL}/healthz" "pet-resort-api"

log "verifying production routes do not manufacture trusted actor authority"
if scripts/hermes-tools/get_manager_daily_brief_context \
  --location-id 00c0ffee-0000-0000-0000-000000000001 \
  --operating-day 2026-06-17 \
  >"${SMOKE_TMP_DIR}/unexpected-context.json" \
  2>"${SMOKE_TMP_DIR}/authority-boundary.err"; then
  printf 'production Manager Daily Brief route unexpectedly issued context without authenticated authority\n' >&2
  exit 1
fi

if ! grep -q 'HTTP 401 from app API' "${SMOKE_TMP_DIR}/authority-boundary.err"; then
  printf 'expected a secret-safe HTTP 401 authority-boundary response\n' >&2
  exit 1
fi
if [[ -s "${SMOKE_TMP_DIR}/unexpected-context.json" ]]; then
  printf 'unauthorized context request returned a payload\n' >&2
  exit 1
fi

printf 'authority_boundary_ok http_401=true trusted_actor_issuer_available=false live_side_effects_allowed=false\n'

log "running the deterministic app-owned workflow contracts separately from production authority issuance"
unset TEST_DATABASE_URL DATABASE_URL
cargo test -p app --test manager_daily_brief_workflow_contracts --locked --quiet

log "smoke artifacts written under ${SMOKE_TMP_DIR}"
log "production API stayed fail closed; deterministic workflow contracts passed without live side effects"
