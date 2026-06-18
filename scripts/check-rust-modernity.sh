#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

python - <<'PY'
from __future__ import annotations

import pathlib
import sys
import tomllib

root = pathlib.Path.cwd()
errors: list[str] = []

manifest_paths = sorted(
    p
    for p in root.rglob("Cargo.toml")
    if "target" not in p.relative_to(root).parts
)

if not manifest_paths:
    errors.append("no Cargo.toml manifests found")

root_manifest = root / "Cargo.toml"
if root_manifest.exists():
    root_text = root_manifest.read_text()
    root_data = tomllib.loads(root_text)
    workspace = root_data.get("workspace", {})
    workspace_package = root_data.get("workspace", {}).get("package", {})
    if workspace.get("resolver") != "3":
        errors.append("root Cargo.toml must keep [workspace] resolver = \"3\"")
    if workspace_package.get("edition") != "2024":
        errors.append("root Cargo.toml must keep [workspace.package] edition = \"2024\"")
else:
    errors.append("root Cargo.toml missing")


def check_dependency_table(path: pathlib.Path, table_name: str, table: object) -> None:
    if not isinstance(table, dict):
        return
    for dep_name, spec in table.items():
        if path == root_manifest and table_name == "workspace.dependencies" and dep_name == "statum":
            # NVA intentionally tracks the latest Statum contract surface. This is
            # the one approved wildcard exception; do not generalize it.
            continue
        if isinstance(spec, str) and spec.strip() == "*":
            errors.append(f"{path.relative_to(root)} {table_name}.{dep_name} uses wildcard \"*\"")
        elif isinstance(spec, dict) and str(spec.get("version", "")).strip() == "*":
            errors.append(f"{path.relative_to(root)} {table_name}.{dep_name}.version uses wildcard \"*\"")


def walk_target_dependencies(path: pathlib.Path, target_table: object) -> None:
    if not isinstance(target_table, dict):
        return
    for target_name, target_spec in target_table.items():
        if not isinstance(target_spec, dict):
            continue
        for key in ("dependencies", "dev-dependencies", "build-dependencies"):
            check_dependency_table(path, f"target.{target_name}.{key}", target_spec.get(key))

for manifest in manifest_paths:
    text = manifest.read_text()
    data = tomllib.loads(text)

    package = data.get("package", {})
    if isinstance(package, dict) and package.get("edition") == "2021":
        errors.append(f"{manifest.relative_to(root)} has package.edition = \"2021\"")
    if 'edition = "2021"' in text:
        errors.append(f"{manifest.relative_to(root)} contains literal edition = \"2021\"")
    if 'resolver = "2"' in text:
        errors.append(f"{manifest.relative_to(root)} contains resolver = \"2\"")

    for key in ("dependencies", "dev-dependencies", "build-dependencies"):
        check_dependency_table(manifest, key, data.get(key))

    workspace = data.get("workspace", {})
    if isinstance(workspace, dict):
        for key in ("dependencies", "dev-dependencies", "build-dependencies"):
            check_dependency_table(manifest, f"workspace.{key}", workspace.get(key))

    walk_target_dependencies(manifest, data.get("target"))

if errors:
    print("Rust modernity manifest gate failed:", file=sys.stderr)
    for error in errors:
        print(f"- {error}", file=sys.stderr)
    sys.exit(1)

print(f"Rust modernity manifest gate passed for {len(manifest_paths)} manifests.")
PY

# This catches compatible lockfile drift. Major-version lags intentionally remain
# a policy review item rather than an automatic failure; cargo reports them as
# `Unchanged ... (available: ...)` below.
update_output="$(mktemp)"
trap 'rm -f "$update_output"' EXIT
cargo update --dry-run --verbose 2>&1 | tee "$update_output"
if grep -Eq '^[[:space:]]*(Adding|Downgrading|Removing) |^[[:space:]]*Updating [[:alnum:]_.+-]+ v[0-9]' "$update_output"; then
  echo "Compatible Cargo.lock updates are available; run cargo update and re-run this gate." >&2
  exit 1
fi

echo "Rust dependency modernity gate passed. Review any 'Unchanged ... (available: ...)' major-version lag deliberately."
