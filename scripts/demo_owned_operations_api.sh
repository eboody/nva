#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

section() {
  printf '\n== %s ==\n' "$*"
}

section "Contract lane: checked OpenAPI boundary"
python - <<'PY'
import json
from pathlib import Path

path = Path("apps/api/openapi/owned-operations-v0.openapi.json")
data = json.loads(path.read_text(encoding="utf-8"))
paths = sorted(data.get("paths", {}))
print(f"openapi_title={data['info']['title']}")
print(f"openapi_version={data['info']['version']}")
print(f"openapi_paths={len(paths)}")
for route in paths:
    if "data-quality" in route or "read-models" in route or "ops/metrics" in route or "manager-daily-brief" in route:
        print(f"owned_route={route}")
print("contract_lane_ok live_side_effects_allowed=false")
PY

section "Workflow lane: Data-Quality Hygiene local loop"
./scripts/smoke_data_quality_hygiene_local_loop.sh

section "Operations lane: disabled worker/outbox proof"
./scripts/smoke_data_quality_hygiene_disabled_worker_outbox.sh

section "Demo complete"
printf 'demo_owned_operations_api_ok local_fixture_only=true live_side_effects_allowed=false\n'
