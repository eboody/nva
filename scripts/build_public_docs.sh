#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

public_src="docs/public/index.html"
public_out="target/doc/index.html"

if [[ ! -f "${public_src}" ]]; then
  echo "missing public landing source: ${public_src}" >&2
  exit 1
fi

if [[ "${SKIP_CARGO_DOC:-0}" != "1" ]]; then
  cargo doc --workspace --no-deps
fi

mkdir -p target/doc
cp "${public_src}" "${public_out}"

# If future landing assets are added next to index.html, preserve them under a
# namespaced generated path without touching Cargo-generated Rustdoc pages.
if compgen -G "docs/public/*" > /dev/null; then
  mkdir -p target/doc/public
  for asset in docs/public/*; do
    [[ "${asset}" == "${public_src}" ]] && continue
    cp -R "${asset}" target/doc/public/
  done
fi

python - <<'PY'
from pathlib import Path

html = Path('target/doc/index.html').read_text(encoding='utf-8')
required_fragments = [
    'Start with the pet-resort entity atlas',
    'entity-first operating model for labor-cost reduction',
    'Browse entity index',
    'Choose a reading path',
    'Workflow-to-entity map',
    '170-site pet-resort',
    'Manager Daily Brief',
    'Data-Quality Hygiene',
    'Gingr/source-system integration',
    'Operations leader',
    'AI program evaluator',
    'Safety/compliance reviewer',
    'Gingr/integration owner',
    'Developer',
    'Rustdocs are evidence, not production authorization',
    'domain/',
    'app/',
    'storage/',
    'gingr/',
    'pet_resort_api/',
    'pet_resort_worker/',
    'cli/',
]
missing = [fragment for fragment in required_fragments if fragment not in html]
if missing:
    raise SystemExit(f'public docs landing missing required fragments: {missing}')
print('public docs landing smoke passed: target/doc/index.html')
PY
