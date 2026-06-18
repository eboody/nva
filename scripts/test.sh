#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p domain -p app -p storage -p gingr --doc
python -m unittest scripts.tests.test_hermes_agent_bridge scripts.tests.test_markdown_contracts scripts.tests.test_markdown_docs_gate -v
./scripts/check_markdown_links.py

if [[ -d node_modules ]]; then
  npm --workspace @pet-resort/staff-web run typecheck
  npm --workspace @pet-resort/staff-web run lint
  npm --workspace @pet-resort/staff-web run test
else
  echo "Skipping frontend gates: run npm install first."
fi
