#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

OPENVIKING_HEALTH_URL="${OPENVIKING_HEALTH_URL:-http://127.0.0.1:${PET_RESORT_OPENVIKING_HOST_PORT:-1933}/health}"
OPENVIKING_CONFIG_FILE="${OPENVIKING_CONFIG_FILE:-/app/.openviking/ov.conf}"
OPENVIKING_PREFLIGHT_HEALTH_ATTEMPTS="${OPENVIKING_PREFLIGHT_HEALTH_ATTEMPTS:-30}"
OPENVIKING_PREFLIGHT_HEALTH_DELAY="${OPENVIKING_PREFLIGHT_HEALTH_DELAY:-2}"
OPENVIKING_STATUS_FILE="${OPENVIKING_STATUS_FILE:-}"
ALLOW_UNINITIALIZED=0

usage() {
  cat <<'USAGE'
Usage: scripts/preflight_openviking_agent_infra.sh [--allow-uninitialized] [--status-file PATH]

Starts the optional local OpenViking Compose service, verifies that the mounted
OpenViking config file exists, and checks the health endpoint.

Default behavior exits non-zero with remediation when the local Docker volume is
not initialized. Use --allow-uninitialized from app-owned smoke tests that should
document the agent-infra blocker but continue proving deterministic app rails.

Environment:
  PET_RESORT_OPENVIKING_HOST_PORT      Host port for OpenViking, default 1933.
  OPENVIKING_HEALTH_URL                Health URL, default http://127.0.0.1:${PET_RESORT_OPENVIKING_HOST_PORT:-1933}/health.
  OPENVIKING_CONFIG_FILE               In-container config path, default /app/.openviking/ov.conf.
  OPENVIKING_CONF_CONTENT              Optional full ov.conf JSON; Compose passes it to the upstream entrypoint on startup.
  OPENVIKING_PREFLIGHT_HEALTH_ATTEMPTS Health curl attempts after config is present, default 30.
  OPENVIKING_PREFLIGHT_HEALTH_DELAY    Seconds between health attempts, default 2.
USAGE
}

log() {
  printf '[openviking-preflight] %s\n' "$*" >&2
}

require() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'missing required command: %s\n' "$1" >&2
    exit 127
  fi
}

write_status() {
  local status="$1"
  local remediation="${2:-}"
  if [[ -n "$OPENVIKING_STATUS_FILE" ]]; then
    {
      printf 'openviking_status=%s\n' "$status"
      printf 'health_url=%s\n' "$OPENVIKING_HEALTH_URL"
      printf 'config_file=%s\n' "$OPENVIKING_CONFIG_FILE"
      if [[ -n "$remediation" ]]; then
        printf 'remediation=%s\n' "$remediation"
      fi
    } >"$OPENVIKING_STATUS_FILE"
  fi
}

remediation_text() {
  cat <<EOF
OpenViking local agent-infra is not initialized: ${OPENVIKING_CONFIG_FILE} is missing or empty inside the openviking container.

This is optional Hermes/OpenViking memory/context infrastructure only; it is not the NVA app source of truth for facts, policy, review gates, audit, or side effects.

Remediation options, from the repository root:

  # Interactive, keeps provider keys out of git:
  docker compose --profile agent-infra up -d openviking
  docker compose exec openviking openviking-server init
  docker compose restart openviking
  scripts/preflight_openviking_agent_infra.sh

  # Non-interactive when an operator supplies the full ov.conf JSON through a local secret channel:
  export OPENVIKING_CONF_CONTENT='<full ov.conf JSON from .env, 1Password, or another local secret source>'
  docker compose --profile agent-infra up -d openviking
  scripts/preflight_openviking_agent_infra.sh

Do not commit real provider keys, root_api_key values, or generated ov.conf files.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --allow-uninitialized)
      ALLOW_UNINITIALIZED=1
      shift
      ;;
    --status-file)
      OPENVIKING_STATUS_FILE="${2:?--status-file requires a path}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'unknown argument: %s\n' "$1" >&2
      usage >&2
      exit 64
      ;;
  esac
done

require docker
require curl

log 'validating docker compose agent-infra configuration'
docker compose --profile agent-infra config >/dev/null

log 'starting optional OpenViking service'
docker compose --profile agent-infra up -d openviking >/dev/null

if ! docker compose exec -T openviking sh -lc "test -s \"${OPENVIKING_CONFIG_FILE}\"" >/dev/null 2>&1; then
  remediation="initialize ${OPENVIKING_CONFIG_FILE} with docker compose exec openviking openviking-server init, or provide OPENVIKING_CONF_CONTENT from a local secret source, then restart openviking"
  write_status 'uninitialized' "$remediation"
  remediation_text >&2
  if [[ "$ALLOW_UNINITIALIZED" -eq 1 ]]; then
    exit 0
  fi
  exit 78
fi

log "found ${OPENVIKING_CONFIG_FILE}; waiting for health at ${OPENVIKING_HEALTH_URL}"
for ((i = 1; i <= OPENVIKING_PREFLIGHT_HEALTH_ATTEMPTS; i++)); do
  if curl -fsS "$OPENVIKING_HEALTH_URL" >/dev/null 2>&1; then
    write_status 'healthy'
    log 'OpenViking health check passed'
    exit 0
  fi
  sleep "$OPENVIKING_PREFLIGHT_HEALTH_DELAY"
done

write_status 'unhealthy' "OpenViking config exists but ${OPENVIKING_HEALTH_URL} did not return HTTP 2xx; inspect docker compose logs openviking"
printf 'OpenViking config exists but health did not become ready at %s after %s attempts.\n' "$OPENVIKING_HEALTH_URL" "$OPENVIKING_PREFLIGHT_HEALTH_ATTEMPTS" >&2
docker compose --profile agent-infra ps openviking >&2 || true
docker compose --profile agent-infra logs --no-color --tail=80 openviking >&2 || true
exit 1
