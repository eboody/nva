#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
lcov_path="coverage/lcov.info"
summary_path="coverage/rust-coverage-summary.json"
html_path="coverage"
base_ref="${RUST_COVERAGE_BASE_REF:-}"
generate=1
coverage_toolchain="${RUST_COVERAGE_TOOLCHAIN:-nightly-2026-08-15}"
wasm_lcov_path="${SPACETIMEDB_COVERAGE_LCOV:-coverage/spacetimedb-wasm.lcov}"

usage() {
  cat <<'USAGE'
Usage: scripts/check_rust_coverage.sh [options]

Generate measured workspace Rust coverage and enforce changed-production-line policy.

Options:
  --no-generate       Validate existing LCOV evidence without running tests.
  --repo-root PATH    Repository root (default: parent of scripts/).
  --base REF          Git base used to identify changed production lines.
  --lcov PATH         LCOV input/output (default: coverage/lcov.info).
  --summary PATH      Deterministic JSON summary (default: coverage/rust-coverage-summary.json).
  -h, --help          Show this help.
USAGE
}

while (($#)); do
  case "$1" in
    --no-generate)
      generate=0
      shift
      ;;
    --repo-root)
      repo_root="$2"
      shift 2
      ;;
    --base)
      base_ref="$2"
      shift 2
      ;;
    --lcov)
      lcov_path="$2"
      shift 2
      ;;
    --summary)
      summary_path="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'coverage gate: unknown argument: %s\n' "$1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

repo_root="$(cd "$repo_root" && pwd)"
if [[ "$lcov_path" != /* ]]; then
  lcov_path="$repo_root/$lcov_path"
fi
if [[ "$summary_path" != /* ]]; then
  summary_path="$repo_root/$summary_path"
fi
if [[ "$html_path" != /* ]]; then
  html_path="$repo_root/$html_path"
fi
if [[ "$wasm_lcov_path" != /* ]]; then
  wasm_lcov_path="$repo_root/$wasm_lcov_path"
fi

cd "$repo_root"

append_relocated_workspace_coverage() {
  local destination="$1"
  local profile_path target_libdir llvm_cov raw_lcov object_path
  local -a artifacts objects object_arguments
  mapfile -t artifacts < <(python3 - "$repo_root" <<'PY'
import os
import sys
import tomllib
from pathlib import Path

root = Path(sys.argv[1])
coverage_target = root / "target" / "llvm-cov-target"
workspace = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))["workspace"]
package_names = []
for member in workspace["members"]:
    manifest = tomllib.loads((root / member / "Cargo.toml").read_text(encoding="utf-8"))
    package_names.append(manifest["package"]["name"])
objects = []
for package_name in package_names:
    for path in coverage_target.glob(f"debug/build/{package_name}/*/out/*"):
        if path.is_file() and os.access(path, os.X_OK) and not path.suffix:
            objects.append(path)
objects.sort(key=lambda path: path.as_posix())
profiles = sorted(coverage_target.glob("*.profdata"), key=lambda path: path.stat().st_mtime_ns)
if not objects or not profiles:
    raise SystemExit("coverage gate: relocated workspace coverage artifacts are absent")
print(profiles[-1])
for path in objects:
    print(path)
PY
)
  profile_path="${artifacts[0]}"
  objects=("${artifacts[@]:1}")
  if ((${#objects[@]} == 0)); then
    printf 'coverage gate: relocated workspace coverage objects are absent\n' >&2
    return 1
  fi
  for object_path in "${objects[@]:1}"; do
    object_arguments+=(--object "$object_path")
  done
  target_libdir="$(rustc "+$coverage_toolchain" --print target-libdir)"
  llvm_cov="$(dirname "$target_libdir")/bin/llvm-cov"
  raw_lcov="$(mktemp)"
  trap 'rm -f "$raw_lcov"' RETURN
  "$llvm_cov" export -format=lcov -instr-profile="$profile_path" "${objects[0]}" "${object_arguments[@]}" >"$raw_lcov"
  python3 - "$repo_root" "$raw_lcov" "$destination" <<'PY'
import sys
from pathlib import Path

source_prefix = f"SF:{Path(sys.argv[1]).resolve()}/"
raw_path = Path(sys.argv[2])
destination = Path(sys.argv[3])
records = raw_path.read_text(encoding="utf-8").split("end_of_record\n")
selected = []
for record in records:
    source_lines = [line for line in record.splitlines() if line.startswith("SF:")]
    if not source_lines or not source_lines[0].startswith(source_prefix):
        continue
    relative = source_lines[0][len(source_prefix):]
    if any(part in {"tests", "examples", "benches"} for part in Path(relative).parts):
        continue
    selected.append(record)
if not selected:
    raise SystemExit("coverage gate: relocated workspace objects have no source evidence")
with destination.open("a", encoding="utf-8") as output:
    if destination.stat().st_size and not destination.read_bytes().endswith(b"\n"):
        output.write("\n")
    for record in selected:
        output.write(record)
        output.write("end_of_record\n")
PY
}

capture_spacetimedb_wasm_coverage() {
  local output="$1"

  rm -f "$output"
  "$repo_root/scripts/capture_spacetimedb_wasm_coverage.sh" --output "$output"
}

append_spacetimedb_wasm_lcov() {
  local destination="$1"
  local wasm_evidence="$2"

  python3 - "$repo_root" "$wasm_evidence" <<'PY'
import sys
from pathlib import Path

repo_root = Path(sys.argv[1]).resolve()
evidence_path = Path(sys.argv[2])
if not evidence_path.is_file() or evidence_path.stat().st_size == 0:
    raise SystemExit(f"coverage gate: SpaceTimeDB WASM LCOV is absent or empty: {evidence_path}")

source_prefix = f"{repo_root}/apps/spacetimedb/src/"
current_source = None
record_has_line = False
has_spacetimedb_record = False
has_positive_spacetimedb_line = False
for raw_line in evidence_path.read_text(encoding="utf-8").splitlines():
    line = raw_line.strip()
    if line.startswith("SF:"):
        if current_source is not None:
            raise SystemExit("coverage gate: malformed SpaceTimeDB WASM LCOV: nested source record")
        current_source = str(Path(line[3:]).resolve())
        record_has_line = False
    elif line.startswith("DA:") and current_source is not None:
        record_has_line = True
        try:
            hits = int(line.split(",", 2)[1])
        except (IndexError, ValueError):
            raise SystemExit("coverage gate: malformed SpaceTimeDB WASM LCOV line evidence")
        if current_source.startswith(source_prefix) and hits > 0:
            has_positive_spacetimedb_line = True
    elif line == "end_of_record":
        if current_source is None or not record_has_line:
            raise SystemExit("coverage gate: malformed SpaceTimeDB WASM LCOV source record")
        if current_source.startswith(source_prefix):
            has_spacetimedb_record = True
        current_source = None
if current_source is not None:
    raise SystemExit("coverage gate: malformed SpaceTimeDB WASM LCOV: unterminated source record")
if not has_spacetimedb_record or not has_positive_spacetimedb_line:
    raise SystemExit("coverage gate: SpaceTimeDB WASM LCOV lacks executed production reducer evidence")
PY

  printf '\n' >>"$destination"
  cat "$wasm_evidence" >>"$destination"
}

if [[ -z "$base_ref" ]]; then
  if git rev-parse --verify --quiet origin/main >/dev/null; then
    base_ref="$(git merge-base HEAD origin/main)"
  elif git rev-parse --verify --quiet HEAD^ >/dev/null; then
    base_ref="HEAD^"
  else
    base_ref="HEAD"
  fi
fi
if ! git rev-parse --verify --quiet "${base_ref}^{commit}" >/dev/null; then
  printf 'coverage gate: base ref is not a commit: %s\n' "$base_ref" >&2
  exit 2
fi

if ((generate)); then
  if ! command -v cargo-llvm-cov >/dev/null 2>&1 && ! cargo llvm-cov --version >/dev/null 2>&1; then
    printf 'coverage gate: cargo-llvm-cov is required to generate measured evidence\n' >&2
    exit 2
  fi
  if [[ -z "${TEST_DATABASE_URL:-}" || -z "${DATABASE_URL:-}" ]]; then
    printf 'coverage gate: TEST_DATABASE_URL and DATABASE_URL must be set; DB-backed tests may not be silently skipped\n' >&2
    exit 2
  fi
  mkdir -p "$(dirname "$lcov_path")"
  capture_spacetimedb_wasm_coverage "$wasm_lcov_path"
  env -u DATABASE_URL RUSTUP_TOOLCHAIN="$coverage_toolchain" cargo llvm-cov --workspace --all-features --all-targets --locked --branch --lcov --output-path "$lcov_path" -- --skip executable_authority_and_outbox_records_enforce_compile_time_protocols --skip booking_triage_typestate_blocks_policy_decision_before_source_evidence_order --skip approved_internal_handoff_authority_is_opaque_one_shot_and_not_serde --skip executable_authority_cannot_be_manufactured_or_replayed_by_callers --skip reservation_aggregate_fields_cannot_be_bypassed_with_struct_literals --skip read_model_public_paths_preserve_schema_compatibility_without_flattening
  append_relocated_workspace_coverage "$lcov_path"
  append_spacetimedb_wasm_lcov "$lcov_path" "$wasm_lcov_path"
  env -u DATABASE_URL RUSTUP_TOOLCHAIN="$coverage_toolchain" cargo llvm-cov report --html --output-dir "$html_path"
fi

if [[ ! -s "$lcov_path" ]]; then
  printf 'coverage gate: LCOV evidence is absent: %s\n' "$lcov_path" >&2
  exit 1
fi

mkdir -p "$(dirname "$summary_path")"
python3 - "$repo_root" "$base_ref" "$lcov_path" "$summary_path" <<'PY'
import json
import re
import subprocess
import sys
from pathlib import Path


repo_root = Path(sys.argv[1]).resolve()
base_ref = sys.argv[2]
lcov_path = Path(sys.argv[3])
summary_path = Path(sys.argv[4])


def fail(message: str) -> None:
    print(f"coverage gate: {message}", file=sys.stderr)
    raise SystemExit(1)


def normalized_source(raw: str) -> Path:
    source = Path(raw)
    if not source.is_absolute():
        source = repo_root / source
    return source.resolve()


sources: dict[Path, dict[str, dict[object, int]]] = {}
current_source: Path | None = None
current_lines: dict[int, int] = {}
current_branches: dict[tuple[int, str, str], int] = {}
record_open = False

try:
    lcov_lines = lcov_path.read_text(encoding="utf-8").splitlines()
except (OSError, UnicodeError) as error:
    fail(f"malformed LCOV: cannot read evidence: {error}")

for line_number, raw_line in enumerate(lcov_lines, start=1):
    line = raw_line.strip()
    if not line:
        continue
    if line.startswith("SF:"):
        if record_open:
            fail(f"malformed LCOV at line {line_number}: nested SF record")
        source_text = line[3:]
        if not source_text:
            fail(f"malformed LCOV at line {line_number}: empty source path")
        current_source = normalized_source(source_text)
        current_lines = {}
        current_branches = {}
        record_open = True
    elif line.startswith("DA:"):
        if not record_open:
            fail(f"malformed LCOV at line {line_number}: DA outside a source record")
        match = re.fullmatch(r"DA:(\d+),(\d+)(?:,[^,]+)?", line)
        if match is None or int(match.group(1)) < 1:
            fail(f"malformed LCOV at line {line_number}: invalid DA record")
        source_line = int(match.group(1))
        current_lines[source_line] = max(current_lines.get(source_line, 0), int(match.group(2)))
    elif line.startswith("BRDA:"):
        if not record_open:
            fail(f"malformed LCOV at line {line_number}: BRDA outside a source record")
        match = re.fullmatch(r"BRDA:(\d+),([^,]+),([^,]+),(\d+|-)", line)
        if match is None or int(match.group(1)) < 1:
            fail(f"malformed LCOV at line {line_number}: invalid BRDA record")
        key = (int(match.group(1)), match.group(2), match.group(3))
        taken = 0 if match.group(4) == "-" else int(match.group(4))
        current_branches[key] = max(current_branches.get(key, 0), taken)
    elif line == "end_of_record":
        if not record_open or current_source is None:
            fail(f"malformed LCOV at line {line_number}: end without source record")
        if not current_lines:
            fail(f"malformed LCOV at line {line_number}: source record has no line evidence")
        evidence = sources.setdefault(current_source, {"lines": {}, "branches": {}})
        for source_line, hits in current_lines.items():
            evidence["lines"][source_line] = max(evidence["lines"].get(source_line, 0), hits)
        for branch, hits in current_branches.items():
            evidence["branches"][branch] = max(evidence["branches"].get(branch, 0), hits)
        current_source = None
        current_lines = {}
        current_branches = {}
        record_open = False
    elif line.startswith(("TN:", "FN:", "FNDA:", "FNF:", "FNH:", "LF:", "LH:", "BRF:", "BRH:")):
        continue

if record_open:
    fail("malformed LCOV: final source record is missing end_of_record")
if not sources:
    fail("malformed LCOV: no complete source records")
if not any(evidence["branches"] for evidence in sources.values()):
    fail("malformed LCOV: no branch evidence; generate with cargo llvm-cov --branch")


def is_production_rust(path: str) -> bool:
    candidate = Path(path)
    return (
        path.endswith(".rs")
        and (path.startswith("src/") or "/src/" in path)
        and not path.startswith("tools/")
        and "tests" not in candidate.parts
        and not candidate.name.endswith("_tests.rs")
    )


def brace_delta(line: str) -> int:
    code = re.sub(r'"(?:\\.|[^"\\])*"', '""', line).split("//", 1)[0]
    return code.count("{") - code.count("}")


def generated_declaration_lines(path: Path) -> set[int]:
    """Lines whose LCOV counters come only from SpacetimeDB proc-macro expansion."""
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeError):
        return set()
    generated: set[int] = set()
    for marker_index, line in enumerate(lines):
        if "#[spacetimedb::table" not in line and "spacetimedb::SpacetimeType" not in line:
            continue
        item_index = marker_index
        while item_index < len(lines) and not re.search(r"\b(?:struct|enum)\b", lines[item_index]):
            item_index += 1
        if item_index == len(lines):
            continue
        depth = 0
        saw_brace = False
        end_index = item_index
        for end_index in range(item_index, len(lines)):
            depth += brace_delta(lines[end_index])
            saw_brace = saw_brace or "{" in lines[end_index]
            if saw_brace and depth == 0:
                break
        generated.update(range(marker_index + 1, end_index + 2))
    return generated


def test_only_lines(path: Path) -> set[int]:
    """Lines inside inline cfg(test) modules are test evidence, not production code."""
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeError):
        return set()
    test_lines: set[int] = set()
    for marker_index, line in enumerate(lines):
        if line.strip() != "#[cfg(test)]":
            continue
        item_index = marker_index + 1
        while item_index < len(lines) and not re.search(r"\bmod\b", lines[item_index]):
            item_index += 1
        if item_index == len(lines):
            continue
        depth = 0
        saw_brace = False
        end_index = item_index
        for end_index in range(item_index, len(lines)):
            depth += brace_delta(lines[end_index])
            saw_brace = saw_brace or "{" in lines[end_index]
            if saw_brace and depth == 0:
                break
        test_lines.update(range(marker_index + 1, end_index + 2))
    return test_lines


def non_executable_declaration_lines(path: Path) -> set[int]:
    """Rust declaration syntax that LLVM may map but cannot independently execute."""
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeError):
        return set()

    ignored = {
        index
        for index, line in enumerate(lines, start=1)
        if not line.strip()
        or line.lstrip().startswith(("//", "#["))
    }

    declaration = re.compile(
        r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|union)\s+"
        r"[A-Za-z_][A-Za-z0-9_]*"
    )
    implementation = re.compile(r"^\s*(?:unsafe\s+)?impl(?:<[^>]*>)?\b.*\{")

    for start_index, line in enumerate(lines):
        if declaration.search(line):
            if ";" in line and "{" not in line:
                ignored.add(start_index + 1)
                continue
            depth = 0
            saw_brace = False
            for end_index in range(start_index, len(lines)):
                depth += brace_delta(lines[end_index])
                saw_brace = saw_brace or "{" in lines[end_index]
                ignored.add(end_index + 1)
                if saw_brace and depth == 0:
                    break
        elif implementation.search(line):
            depth = 0
            saw_brace = False
            for end_index in range(start_index, len(lines)):
                depth += brace_delta(lines[end_index])
                saw_brace = saw_brace or "{" in lines[end_index]
                if saw_brace and depth == 0:
                    ignored.update({start_index + 1, end_index + 1})
                    break

    return ignored


def matching_brace_end(code: str, opening: int) -> int | None:
    depth = 0
    for index in range(opening, len(code)):
        if code[index] == "{":
            depth += 1
        elif code[index] == "}":
            depth -= 1
            if depth == 0:
                return index + 1
    return None


def unwrap_inline_modules(code: str) -> str:
    """Remove module wrapper braces while retaining their declarations or code."""
    module_start = re.compile(
        r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?mod\s+"
        r"[A-Za-z_][A-Za-z0-9_]*\s*\{"
    )
    while match := module_start.search(code):
        opening = code.rfind("{", match.start(), match.end())
        closing = matching_brace_end(code, opening)
        if closing is None:
            break
        body = code[opening + 1 : closing - 1]
        code = code[: match.start()] + body + code[closing:]
    return code


def strip_data_type_declarations(code: str) -> str:
    """Remove enum/struct declarations, including nested struct-like enum variants."""
    declaration_start = re.compile(
        r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:enum|struct)\s+"
        r"[A-Za-z_][A-Za-z0-9_]*"
    )
    while match := declaration_start.search(code):
        opening = code.find("{", match.end())
        semicolon = code.find(";", match.end())
        if semicolon >= 0 and (opening < 0 or semicolon < opening):
            code = code[: match.start()] + code[semicolon + 1 :]
            continue
        if opening < 0:
            break
        closing = matching_brace_end(code, opening)
        if closing is None:
            break
        code = code[: match.start()] + code[closing:]
    return code


def is_thin_module_index(path: Path) -> bool:
    try:
        source = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError):
        return False
    code = re.sub(r"/\*.*?\*/", "", source, flags=re.DOTALL)
    code = re.sub(r"//[^\n]*", "", code)
    code = re.sub(r"#\[[^\]]*\]", "", code, flags=re.DOTALL)
    code = unwrap_inline_modules(code)
    code = strip_data_type_declarations(code)
    type_declaration = re.compile(
        r"(?:pub(?:\([^)]*\))?\s+)?trait\s+"
        r"[A-Za-z_][A-Za-z0-9_]*[^\{;]*(?:\{[^\{\}]*\}|;)",
        flags=re.DOTALL,
    )
    code = type_declaration.sub("", code)
    declaration = re.compile(
        r"(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?(?:"
        r"mod\s+[A-Za-z_][A-Za-z0-9_]*\s*;|"
        r"use\s+.*?;|"
        r"const\s+[A-Za-z_][A-Za-z0-9_]*\s*:[^=;]+=[^;]+;|"
        r"type\s+[A-Za-z_][A-Za-z0-9_]*(?:<[^;=]+>)?\s*=[^;]+;)",
        flags=re.DOTALL,
    )
    code = declaration.sub("", code)
    empty_module = re.compile(
        r"(?:pub(?:\([^)]*\))?\s+)?mod\s+[A-Za-z_][A-Za-z0-9_]*\s*\{\s*\}"
    )
    return empty_module.sub("", code).strip() == ""


diff = subprocess.run(
    [
        "git",
        "-c",
        "color.diff.newMoved=blue",
        "-c",
        "color.diff.newMovedAlternative=blue",
        "diff",
        "--color=always",
        "--color-moved=blocks",
        "--unified=0",
        "--diff-filter=AM",
        base_ref,
        "--",
        "*.rs",
    ],
    cwd=repo_root,
    text=True,
    capture_output=True,
    check=False,
)
if diff.returncode != 0:
    fail(f"cannot compute changed Rust lines from {base_ref}: {diff.stderr.strip()}")

changed: dict[str, set[int]] = {}
current_path: str | None = None
current_new_line: int | None = None
ansi_escape = re.compile(r"\x1b\[[0-9;]*m")
for raw_line in diff.stdout.splitlines():
    line = ansi_escape.sub("", raw_line)
    if line.startswith("diff --git "):
        current_path = None
        current_new_line = None
        continue
    if line.startswith("+++ b/"):
        candidate = line[6:]
        current_path = candidate if is_production_rust(candidate) else None
        if current_path is not None:
            changed.setdefault(current_path, set())
        continue
    if line.startswith("@@"):
        if current_path is None:
            current_new_line = None
            continue
        match = re.search(r"\+(\d+)(?:,(\d+))?", line)
        if match is None:
            fail(f"cannot parse git diff hunk: {line}")
        current_new_line = int(match.group(1))
        continue
    if current_path is None or current_new_line is None:
        continue
    if line.startswith("+") and not line.startswith("+++"):
        if "\x1b[34m" not in raw_line:
            changed[current_path].add(current_new_line)
        current_new_line += 1
    elif line.startswith("-") and not line.startswith("---"):
        continue
    elif line.startswith(" "):
        current_new_line += 1

changed_executable = 0
changed_covered = 0
uncovered: list[str] = []
for relative_path in sorted(changed):
    if not changed[relative_path]:
        continue
    absolute_path = (repo_root / relative_path).resolve()
    evidence = sources.get(absolute_path)
    if evidence is None:
        if is_thin_module_index(absolute_path):
            continue
        fail(f"changed production Rust file is absent from LCOV: {relative_path}")
    ignored_lines = (
        generated_declaration_lines(absolute_path)
        | test_only_lines(absolute_path)
        | non_executable_declaration_lines(absolute_path)
    )
    for source_line in sorted(changed[relative_path]):
        if source_line in ignored_lines:
            continue
        if source_line not in evidence["lines"]:
            continue
        changed_executable += 1
        if evidence["lines"][source_line] > 0:
            changed_covered += 1
        else:
            uncovered.append(f"{relative_path}:{source_line}")

if uncovered:
    fail("changed executable Rust line is uncovered: " + ", ".join(uncovered))

line_total = sum(len(evidence["lines"]) for evidence in sources.values())
line_covered = sum(
    sum(1 for hits in evidence["lines"].values() if hits > 0)
    for evidence in sources.values()
)
branch_total = sum(len(evidence["branches"]) for evidence in sources.values())
branch_covered = sum(
    sum(1 for hits in evidence["branches"].values() if hits > 0)
    for evidence in sources.values()
)


def percent(covered: int, total: int) -> float:
    return round(covered * 100.0 / total, 2) if total else 100.0


summary = {
    "branch": {
        "covered": branch_covered,
        "percent": percent(branch_covered, branch_total),
        "total": branch_total,
    },
    "changed_production": {
        "covered_executable_lines": changed_covered,
        "executable_lines": changed_executable,
        "files": len(changed),
        "uncovered_executable_lines": len(uncovered),
    },
    "line": {
        "covered": line_covered,
        "percent": percent(line_covered, line_total),
        "total": line_total,
    },
    "schema_version": 1,
}
summary_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(
    "coverage gate: "
    f"lines {line_covered}/{line_total} ({summary['line']['percent']:.2f}%), "
    f"branches {branch_covered}/{branch_total} ({summary['branch']['percent']:.2f}%), "
    f"changed executable lines {changed_covered}/{changed_executable}"
)
PY
