import json
import os
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
GATE = REPO_ROOT / "scripts" / "check_rust_coverage.sh"


def run(*args: str, cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [*args],
        cwd=cwd,
        text=True,
        capture_output=True,
        check=False,
    )


def make_changed_rust_repo(tmp_path: Path) -> tuple[Path, Path]:
    repo = tmp_path / "repo"
    source = repo / "crate" / "src" / "lib.rs"
    source.parent.mkdir(parents=True)
    source.write_text("pub fn answer() -> u8 {\n    1\n}\n")
    run("git", "init", "-q", cwd=repo)
    run("git", "config", "user.email", "coverage-test@example.invalid", cwd=repo)
    run("git", "config", "user.name", "Coverage Test", cwd=repo)
    run("git", "add", ".", cwd=repo)
    run("git", "commit", "-qm", "baseline", cwd=repo)
    source.write_text("pub fn answer() -> u8 {\n    2\n}\n")
    return repo, source


def invoke_gate(repo: Path, lcov: Path, summary: Path | None = None) -> subprocess.CompletedProcess[str]:
    command = [
        "bash",
        str(GATE),
        "--no-generate",
        "--repo-root",
        str(repo),
        "--base",
        "HEAD",
        "--lcov",
        str(lcov),
    ]
    if summary is not None:
        command.extend(["--summary", str(summary)])
    return run(*command, cwd=repo)


def lcov_record(source: Path, *, changed_line_hits: int = 1) -> str:
    return (
        "TN:\n"
        f"SF:{source.resolve()}\n"
        "DA:1,1\n"
        f"DA:2,{changed_line_hits}\n"
        "DA:3,1\n"
        "BRDA:1,0,0,1\n"
        "BRDA:1,0,1,0\n"
        "end_of_record\n"
    )


def test_gate_rejects_absent_lcov(tmp_path: Path):
    repo, _ = make_changed_rust_repo(tmp_path)

    result = invoke_gate(repo, tmp_path / "missing.info")

    assert result.returncode != 0
    assert "LCOV evidence is absent" in result.stderr


def test_gate_rejects_malformed_lcov(tmp_path: Path):
    repo, source = make_changed_rust_repo(tmp_path)
    lcov = tmp_path / "malformed.info"
    lcov.write_text(f"SF:{source.resolve()}\nDA:not-a-line,1\n")

    result = invoke_gate(repo, lcov)

    assert result.returncode != 0
    assert "malformed LCOV" in result.stderr


def test_gate_rejects_lcov_that_omits_a_changed_production_file(tmp_path: Path):
    repo, _ = make_changed_rust_repo(tmp_path)
    unrelated = repo / "other" / "src" / "lib.rs"
    unrelated.parent.mkdir(parents=True)
    unrelated.write_text("pub fn other() {}\n")
    lcov = tmp_path / "incomplete.info"
    lcov.write_text(lcov_record(unrelated))

    result = invoke_gate(repo, lcov)

    assert result.returncode != 0
    assert "changed production Rust file is absent from LCOV" in result.stderr
    assert "crate/src/lib.rs" in result.stderr


def test_gate_accepts_omitted_changed_thin_module_index(tmp_path: Path):
    repo, source = make_changed_rust_repo(tmp_path)
    source.write_text(
        'pub const SCHEMA_VERSION: &str = "v1";\n'
        "pub mod workflow;\n"
        "pub use workflow::Packet;\n"
        "pub mod prelude {\n    pub use crate::workflow::Packet;\n}\n"
        "pub trait Port {\n    fn load(&self) -> u8;\n}\n"
        "pub enum Queue {\n    Staff,\n    Manager,\n}\n"
        "pub type Result<T> = core::result::Result<T, Queue>;\n"
    )
    unrelated = repo / "other" / "src" / "lib.rs"
    unrelated.parent.mkdir(parents=True)
    unrelated.write_text("pub fn other() {}\n")
    lcov = tmp_path / "thin-index.info"
    lcov.write_text(lcov_record(unrelated))
    summary = tmp_path / "summary.json"

    result = invoke_gate(repo, lcov, summary)

    assert result.returncode == 0, result.stderr
    assert json.loads(summary.read_text())["changed_production"] == {
        "covered_executable_lines": 0,
        "executable_lines": 0,
        "files": 1,
        "uncovered_executable_lines": 0,
    }


def test_gate_accepts_omitted_data_only_enum_with_struct_variants(tmp_path: Path):
    repo, source = make_changed_rust_repo(tmp_path)
    source.write_text(
        "pub mod evidence {\n"
        "    pub trait Intake { fn load(&self) -> Decision; }\n"
        "    pub enum Decision {\n"
        "        Reported { reason: Reason, snapshot_id: String },\n"
        "        Unavailable { reason: Reason },\n"
        "    }\n"
        "    pub enum Reason { CapacityHeld, PolicyHardStop }\n"
        "}\n"
    )
    unrelated = repo / "other" / "src" / "lib.rs"
    unrelated.parent.mkdir(parents=True)
    unrelated.write_text("pub fn other() {}\n")
    lcov = tmp_path / "data-only-enum.info"
    lcov.write_text(lcov_record(unrelated))
    summary = tmp_path / "summary.json"

    result = invoke_gate(repo, lcov, summary)

    assert result.returncode == 0, result.stderr
    assert json.loads(summary.read_text())["changed_production"] == {
        "covered_executable_lines": 0,
        "executable_lines": 0,
        "files": 1,
        "uncovered_executable_lines": 0,
    }


def test_gate_does_not_treat_exactly_relocated_code_as_new(tmp_path: Path):
    repo, source = make_changed_rust_repo(tmp_path)
    moved_function = (
        "pub fn relocated() -> u8 {\n"
        "    let values = [\n"
        "        1,\n"
        "        2,\n"
        "        3,\n"
        "        4,\n"
        "        5,\n"
        "        6,\n"
        "        7,\n"
        "    ];\n"
        "    values[6]\n"
        "}\n"
    )
    padding = "".join(f"pub const PADDING_{index}: u16 = {index};\n" for index in range(100))
    source.write_text(moved_function + padding)
    run("git", "add", ".", cwd=repo)
    run("git", "commit", "-qm", "large module baseline", cwd=repo)
    source.write_text(padding)
    relocated = repo / "crate" / "src" / "relocated.rs"
    relocated.write_text(moved_function)
    run("git", "add", ".", cwd=repo)
    unrelated = repo / "other" / "src" / "lib.rs"
    unrelated.parent.mkdir(parents=True)
    unrelated.write_text("pub fn other() {}\n")
    lcov = tmp_path / "relocated.info"
    lcov.write_text(lcov_record(unrelated))
    summary = tmp_path / "summary.json"

    result = invoke_gate(repo, lcov, summary)

    assert result.returncode == 0, result.stderr
    assert json.loads(summary.read_text())["changed_production"] == {
        "covered_executable_lines": 0,
        "executable_lines": 0,
        "files": 2,
        "uncovered_executable_lines": 0,
    }


def test_gate_still_requires_coverage_for_code_changed_during_relocation(tmp_path: Path):
    repo, source = make_changed_rust_repo(tmp_path)
    original = (
        "pub fn relocated() -> u8 {\n"
        "    let values = [\n"
        "        1,\n"
        "        2,\n"
        "        3,\n"
        "        4,\n"
        "        5,\n"
        "        6,\n"
        "        7,\n"
        "    ];\n"
        "    values[6]\n"
        "}\n"
    )
    padding = "".join(f"pub const PADDING_{index}: u16 = {index};\n" for index in range(100))
    source.write_text(original + padding)
    run("git", "add", ".", cwd=repo)
    run("git", "commit", "-qm", "large module baseline", cwd=repo)
    source.write_text(padding)
    relocated = repo / "crate" / "src" / "relocated.rs"
    relocated.write_text(original.replace("values[6]", "values[5]"))
    run("git", "add", ".", cwd=repo)
    lcov = tmp_path / "changed-relocation.info"
    lcov.write_text(
        "TN:\n"
        f"SF:{relocated.resolve()}\n"
        + "".join(f"DA:{line},{0 if line == 11 else 1}\n" for line in range(1, 13))
        + "BRDA:1,0,0,1\n"
        + "end_of_record\n"
    )

    result = invoke_gate(repo, lcov)

    assert result.returncode != 0
    assert "changed executable Rust line is uncovered" in result.stderr
    assert "crate/src/relocated.rs:11" in result.stderr


def test_gate_rejects_uncovered_changed_executable_line(tmp_path: Path):
    repo, source = make_changed_rust_repo(tmp_path)
    lcov = tmp_path / "uncovered.info"
    lcov.write_text(lcov_record(source, changed_line_hits=0))

    result = invoke_gate(repo, lcov)

    assert result.returncode != 0
    assert "changed executable Rust line is uncovered" in result.stderr
    assert "crate/src/lib.rs:2" in result.stderr


def test_gate_ignores_non_executable_rust_declaration_lines_with_zero_counters(
    tmp_path: Path,
):
    repo, source = make_changed_rust_repo(tmp_path)
    source.write_text(
        "#[derive(Clone)]\n"
        "pub struct Evidence {\n"
        "    /// Retained source value.\n"
        "    value: u8,\n"
        "}\n"
        "\n"
        "impl Evidence {\n"
        "    /// Returns the retained value.\n"
        "    pub fn value(&self) -> u8 {\n"
        "        self.value\n"
        "    }\n"
        "}\n"
    )
    lcov = tmp_path / "declarations.info"
    lcov.write_text(
        "TN:\n"
        f"SF:{source.resolve()}\n"
        + "".join(f"DA:{line},{1 if line in {9, 10, 11} else 0}\n" for line in range(1, 13))
        + "BRDA:9,0,0,1\n"
        + "end_of_record\n"
    )
    summary = tmp_path / "summary.json"

    result = invoke_gate(repo, lcov, summary)

    assert result.returncode == 0, result.stderr
    assert json.loads(summary.read_text())["changed_production"] == {
        "covered_executable_lines": 3,
        "executable_lines": 3,
        "files": 1,
        "uncovered_executable_lines": 0,
    }


def test_gate_ignores_changed_rust_coverage_harness_tools(tmp_path: Path):
    repo, source = make_changed_rust_repo(tmp_path)
    source.write_text("pub fn answer() -> u8 {\n    1\n}\n")
    run("git", "add", ".", cwd=repo)
    run("git", "commit", "-qm", "restore production source", cwd=repo)
    harness = repo / "tools" / "coverage-harness" / "src" / "lib.rs"
    harness.parent.mkdir(parents=True)
    harness.write_text("pub fn exercise_instrumented_runtime() { panic!() }\n")
    run("git", "add", ".", cwd=repo)
    lcov = tmp_path / "tooling.info"
    lcov.write_text(lcov_record(source))
    summary = tmp_path / "summary.json"

    result = invoke_gate(repo, lcov, summary)

    assert result.returncode == 0, result.stderr
    assert json.loads(summary.read_text())["changed_production"] == {
        "covered_executable_lines": 0,
        "executable_lines": 0,
        "files": 0,
        "uncovered_executable_lines": 0,
    }


def test_gate_ignores_test_only_sources_and_proc_macro_generated_table_metadata(tmp_path: Path):
    repo, source = make_changed_rust_repo(tmp_path)
    source.write_text(
        "#[spacetimedb::table(accessor = old_queue)]\n"
        "#[derive(Clone)]\n"
        "pub struct QueueRow {\n"
        "    pub id: u64,\n"
        "}\n"
        "#[derive(Clone, spacetimedb::SpacetimeType)]\n"
        "pub enum Status { Ready }\n"
        "pub fn answer() -> u8 {\n"
        "    2\n"
        "}\n"
        "#[cfg(test)]\n"
        "mod tests { fn old_assertion() {} }\n"
    )
    test_source = repo / "crate" / "src" / "queue_tests.rs"
    test_source.write_text("pub fn old_assertion() {}\n")
    run("git", "add", ".", cwd=repo)
    run("git", "commit", "-qm", "generated baseline", cwd=repo)
    source.write_text(
        "#[spacetimedb::table(accessor = queue)]\n"
        "#[derive(Clone)]\n"
        "pub struct QueueRow {\n"
        "    pub id: u64,\n"
        "}\n"
        "#[derive(Clone, spacetimedb::SpacetimeType)]\n"
        "pub enum Status { Ready, Blocked }\n"
        "pub fn answer() -> u8 {\n"
        "    3\n"
        "}\n"
        "#[cfg(test)]\n"
        "mod tests { fn assertion_panic_path() { panic!() } }\n"
    )
    test_source.write_text("pub fn assertion_panic_path() { panic!() }\n")
    lcov = tmp_path / "generated.info"
    lcov.write_text(
        "TN:\n"
        f"SF:{source.resolve()}\n"
        "DA:1,0\nDA:5,0\nDA:6,0\nDA:7,0\nDA:8,1\nDA:9,1\nDA:10,1\nDA:12,0\n"
        "BRDA:8,0,0,1\n"
        "end_of_record\n"
        "TN:\n"
        f"SF:{test_source.resolve()}\n"
        "DA:1,0\n"
        "end_of_record\n"
    )
    summary = tmp_path / "summary.json"

    result = invoke_gate(repo, lcov, summary)

    assert result.returncode == 0, result.stderr
    assert json.loads(summary.read_text())["changed_production"] == {
        "covered_executable_lines": 1,
        "executable_lines": 1,
        "files": 1,
        "uncovered_executable_lines": 0,
    }


def test_gate_accepts_integration_coverage_without_paired_test_filename(tmp_path: Path):
    repo, source = make_changed_rust_repo(tmp_path)
    integration_test = repo / "crate" / "tests" / "workflow_contract.rs"
    integration_test.parent.mkdir(parents=True)
    integration_test.write_text("// Deliberately not named after src/lib.rs.\n")
    lcov = tmp_path / "complete.info"
    lcov.write_text(lcov_record(source))
    summary = tmp_path / "summary.json"

    result = invoke_gate(repo, lcov, summary)

    assert result.returncode == 0, result.stderr
    assert json.loads(summary.read_text()) == {
        "branch": {"covered": 1, "percent": 50.0, "total": 2},
        "changed_production": {
            "covered_executable_lines": 1,
            "executable_lines": 1,
            "files": 1,
            "uncovered_executable_lines": 0,
        },
        "line": {"covered": 3, "percent": 100.0, "total": 3},
        "schema_version": 1,
    }


def test_generation_contract_runs_all_workspace_targets_and_requires_database_evidence():
    script = GATE.read_text()

    assert "cargo llvm-cov --workspace --all-features --all-targets" in script
    assert '"color.diff.newMoved=blue"' in script
    assert '"--color-moved=blocks"' in script
    assert 'coverage_toolchain="${RUST_COVERAGE_TOOLCHAIN:-nightly-2026-08-15}"' in script
    assert 'RUSTUP_TOOLCHAIN="$coverage_toolchain" cargo llvm-cov' in script
    assert "--branch" in script
    assert "TEST_DATABASE_URL" in script
    assert "DATABASE_URL" in script
    assert 'env -u DATABASE_URL RUSTUP_TOOLCHAIN="$coverage_toolchain" cargo llvm-cov' in script
    assert 'cargo llvm-cov report --html --output-dir "$html_path"' in script
    assert 'html_path="coverage"' in script
    assert "--locked" in script
    assert 'append_relocated_workspace_coverage "$lcov_path"' in script
    assert 'capture_spacetimedb_wasm_coverage "$wasm_lcov_path"' in script
    assert 'append_spacetimedb_wasm_lcov "$lcov_path" "$wasm_lcov_path"' in script
    assert "RUST_COVERAGE_ADDITIONAL_LCOV" not in script
    assert (REPO_ROOT / "scripts" / "capture_spacetimedb_wasm_coverage.sh").is_file()
    assert (REPO_ROOT / "scripts" / "spacetimedb_wasm_coverage.py").is_file()
    for trybuild_harness in [
        "executable_authority_and_outbox_records_enforce_compile_time_protocols",
        "booking_triage_typestate_blocks_policy_decision_before_source_evidence_order",
        "approved_internal_handoff_authority_is_opaque_one_shot_and_not_serde",
        "executable_authority_cannot_be_manufactured_or_replayed_by_callers",
        "reservation_aggregate_fields_cannot_be_bypassed_with_struct_literals",
        "read_model_public_paths_preserve_schema_compatibility_without_flattening",
    ]:
        assert f"--skip {trybuild_harness}" in script


def test_zero_hit_coverage_has_no_exemption_surface():
    script = GATE.read_text()
    manifest = REPO_ROOT / "scripts" / "rust_coverage_unmintable.json"

    assert not manifest.exists()
    assert "unmintable-coverage" not in script
    assert '"documented_unmintable_lines"' not in script


def test_canonical_generation_invokes_and_merges_wasm_capture(tmp_path: Path):
    repo, source = make_changed_rust_repo(tmp_path)
    (repo / "scripts").mkdir(parents=True)
    (repo / "scripts" / "rust_coverage_unmintable.json").write_text(
        '{"exemptions": [], "schema_version": 1}\n', encoding="utf-8"
    )
    (repo / "Cargo.toml").write_text(
        '[workspace]\nmembers = ["crate"]\nresolver = "3"\n', encoding="utf-8"
    )
    (repo / "crate" / "Cargo.toml").write_text(
        '[package]\nname = "coverage-fixture"\nversion = "0.1.0"\nedition = "2024"\n',
        encoding="utf-8",
    )
    wasm_source = repo / "apps" / "spacetimedb" / "src" / "lib.rs"
    wasm_source.parent.mkdir(parents=True)
    wasm_source.write_text("pub fn reducer() {}\n", encoding="utf-8")
    capture = repo / "scripts" / "capture_spacetimedb_wasm_coverage.sh"
    capture.parent.mkdir(parents=True, exist_ok=True)
    capture.write_text(
        """#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == "--output" ]]
printf 'invoked\n' >"$CAPTURE_LOG"
mkdir -p "$(dirname "$2")"
if [[ "${CAPTURE_EMPTY:-0}" == "1" ]]; then
    : >"$2"
    exit 0
fi
cat >"$2" <<EOF
TN:
SF:$WASM_SOURCE
DA:1,1
BRDA:1,0,0,1
end_of_record
EOF
""",
        encoding="utf-8",
    )
    capture.chmod(0o755)

    bin_dir = tmp_path / "bin"
    llvm_dir = tmp_path / "llvm" / "bin"
    bin_dir.mkdir()
    llvm_dir.mkdir(parents=True)
    fake_cargo = bin_dir / "cargo"
    fake_cargo.write_text(
        r"""#!/usr/bin/env python3
import os
import sys
from pathlib import Path

args = sys.argv[1:]
if args[:2] == ["llvm-cov", "--version"]:
    raise SystemExit(0)
if args and args[0] == "llvm-cov" and "--output-path" in args:
    output = Path(args[args.index("--output-path") + 1])
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        "TN:\nSF:" + os.environ["NATIVE_SOURCE"]
        + "\nDA:1,1\nDA:2,1\nDA:3,1\nBRDA:1,0,0,1\nend_of_record\n"
    )
    target = Path(os.environ["FIXTURE_REPO"]) / "target" / "llvm-cov-target"
    object_path = target / "debug" / "build" / "coverage-fixture" / "hash" / "out" / "object"
    object_path.parent.mkdir(parents=True, exist_ok=True)
    object_path.write_text("object")
    object_path.chmod(0o755)
    (target / "coverage.profdata").write_text("profile")
    raise SystemExit(0)
if args[:2] == ["llvm-cov", "report"]:
    raise SystemExit(0)
raise SystemExit("unexpected cargo invocation: " + repr(args))
""",
        encoding="utf-8",
    )
    fake_cargo.chmod(0o755)
    fake_rustc = bin_dir / "rustc"
    fake_rustc.write_text(
        f'#!/usr/bin/env bash\nprintf "%s\\n" "{tmp_path / "llvm" / "rustlib"}"\n',
        encoding="utf-8",
    )
    fake_rustc.chmod(0o755)
    fake_llvm_cov = llvm_dir / "llvm-cov"
    fake_llvm_cov.write_text(
        """#!/usr/bin/env bash
cat <<EOF
TN:
SF:$NATIVE_SOURCE
DA:1,1
DA:2,1
DA:3,1
BRDA:1,0,0,1
end_of_record
EOF
""",
        encoding="utf-8",
    )
    fake_llvm_cov.chmod(0o755)

    lcov = tmp_path / "canonical.info"
    summary = tmp_path / "summary.json"
    capture_log = tmp_path / "capture.log"
    env = os.environ.copy()
    env.update(
        {
            "CAPTURE_LOG": str(capture_log),
            "DATABASE_URL": "postgres://coverage.invalid/db",
            "FIXTURE_REPO": str(repo),
            "NATIVE_SOURCE": str(source.resolve()),
            "PATH": f"{bin_dir}:{env['PATH']}",
            "TEST_DATABASE_URL": "postgres://coverage.invalid/test",
            "WASM_SOURCE": str(wasm_source.resolve()),
        }
    )
    result = subprocess.run(
        [
            "bash",
            str(GATE),
            "--repo-root",
            str(repo),
            "--base",
            "HEAD",
            "--lcov",
            str(lcov),
            "--summary",
            str(summary),
        ],
        cwd=repo,
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )

    assert result.returncode == 0, result.stderr
    assert capture_log.read_text(encoding="utf-8") == "invoked\n"
    evidence = lcov.read_text(encoding="utf-8")
    assert f"SF:{source.resolve()}" in evidence
    assert f"SF:{wasm_source.resolve()}" in evidence

    env["CAPTURE_EMPTY"] = "1"
    absent = subprocess.run(
        [
            "bash",
            str(GATE),
            "--repo-root",
            str(repo),
            "--base",
            "HEAD",
            "--lcov",
            str(tmp_path / "absent-wasm.info"),
            "--summary",
            str(tmp_path / "absent-wasm-summary.json"),
        ],
        cwd=repo,
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )
    assert absent.returncode != 0
    assert "SpaceTimeDB WASM LCOV is absent or empty" in absent.stderr
