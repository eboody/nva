#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python -m unittest scripts.tests.test_hermes_agent_bridge -v

if [[ -d node_modules ]]; then
  npm --workspace @pet-resort/staff-web run typecheck
  npm --workspace @pet-resort/staff-web run lint
  npm --workspace @pet-resort/staff-web run test
else
  echo "Skipping frontend gates: run npm install first."
fi
