#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

mkdir -p target
docs_target_dir="$(mktemp -d "target/docs-smoke.XXXXXX")"
cleanup_docs_target_dir() {
    rm -rf "${docs_target_dir}"
}
trap cleanup_docs_target_dir EXIT

export CARGO_TARGET_DIR="${docs_target_dir}"

for package in domain app storage gingr; do
    cargo test -p "${package}" --doc
done
python scripts/check_rustdoc_completeness.py
python scripts/check_markdown_links.py
python scripts/check_public_docs_landing.py
