#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

log() {
  printf '[data-quality-hygiene-smoke] %s\n' "$*" >&2
}

require() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'missing required command: %s\n' "$1" >&2
    exit 127
  fi
}

require cargo
require python

SMOKE_TMP_DIR="${SMOKE_TMP_DIR:-$(mktemp -d)}"
OUTPUT_FILE="${SMOKE_TMP_DIR}/data-quality-hygiene-local-smoke.txt"

log "running fixture-only app-owned data-quality hygiene loop"
log "no live customer sends, provider/PMS writes, schedule changes, payment/refund/discount movement, or medical/safety decisions are attempted"

cargo run -p app --example data_quality_hygiene_local_smoke --quiet | tee "$OUTPUT_FILE"

python - "$OUTPUT_FILE" <<'PY'
import re
import sys

text = open(sys.argv[1], encoding="utf-8").read()
required = [
    "context_ok",
    "draft_validation_ok",
    "blocked_draft_validation_ok",
    "outcome_ok",
    "live_side_effects_allowed=false",
]
missing = [needle for needle in required if needle not in text]
if missing:
    raise SystemExit(f"missing smoke markers: {missing}\n{text}")

estimated = re.search(r"estimated_minutes_saved=(\d+)", text)
actual = re.search(r"actual_minutes_saved=(\d+)", text)
if not estimated or not actual:
    raise SystemExit(f"missing labor-savings metrics\n{text}")
if int(estimated.group(1)) <= 0 or int(actual.group(1)) <= 0:
    raise SystemExit(f"labor-savings metrics must be positive\n{text}")

print(
    "smoke_assertions_ok estimated_minutes_saved={estimated} actual_minutes_saved={actual}".format(
        estimated=estimated.group(1),
        actual=actual.group(1),
    )
)
PY

log "smoke artifacts written under ${SMOKE_TMP_DIR}"
log "local fake-data Data-Quality Hygiene loop passed without live/customer/provider side effects"
