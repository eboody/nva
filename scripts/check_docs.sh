#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

cargo test -p domain -p app -p storage -p gingr --doc
python scripts/check_markdown_links.py
