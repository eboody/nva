#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

api_addr="${PET_RESORT_API_ADDR:-127.0.0.1:3001}"
api_url="http://${api_addr}"

cargo run -p pet-resort-api > /tmp/pet-resort-api-smoke.log 2>&1 &
api_pid=$!
trap 'kill "$api_pid" 2>/dev/null || true' EXIT

for _ in {1..160}; do
  if curl -fsS "$api_url/healthz" >/tmp/pet-resort-health.json; then
    break
  fi
  if ! kill -0 "$api_pid" 2>/dev/null; then
    echo "pet-resort-api exited before readiness" >&2
    cat /tmp/pet-resort-api-smoke.log >&2 || true
    exit 1
  fi
  sleep 0.25
done

if ! curl -fsS "$api_url/healthz" | tee /tmp/pet-resort-health.json; then
  echo "pet-resort-api did not become ready within smoke timeout" >&2
  cat /tmp/pet-resort-api-smoke.log >&2 || true
  exit 1
fi
printf '\n'
curl -fsS "$api_url/readyz" | tee /tmp/pet-resort-ready.json
printf '\nSmoke shell OK; no live side effects were enabled.\n'
