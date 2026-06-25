#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

log() {
  printf '[data-quality-hygiene-worker-outbox-smoke] %s\n' "$*" >&2
}

require() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'missing required command: %s\n' "$1" >&2
    exit 127
  fi
}

require cargo

log "running disabled/fake worker outbox proof"
log "no live customer sends, provider/PMS writes, schedule changes, payment/refund/discount movement, or medical/safety decisions are attempted"

cargo test -p pet-resort-worker \
  --test runtime_mode_contract \
  -- --nocapture

log "disabled worker/outbox proof passed as local internal handoff only"
