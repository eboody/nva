import json
import re
import subprocess
import sys
from pathlib import Path

import yaml


REPO_ROOT = Path(__file__).resolve().parents[2]
CI = REPO_ROOT / ".github" / "workflows" / "ci.yml"


def test_ci_enforces_reproducible_enterprise_release_gates():
    workflow = CI.read_text()

    required_fragments = [
        "permissions:\n  contents: read",
        "concurrency:",
        "cancel-in-progress: true",
        "timeout-minutes:",
        "cargo fmt --all -- --check",
        "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings",
        "cargo clippy --release --workspace --all-targets --all-features --locked -- -D warnings",
        "cargo test --workspace --all-targets --all-features --locked",
        "cargo doc --workspace --all-features --no-deps --locked",
        "python scripts/check_rustdoc_completeness.py",
        "python scripts/check_workspace_quality.py --repo-root .",
        "python scripts/check_architecture_quality.py --repo-root .",
        "./scripts/check_rust_coverage.sh",
        "python scripts/check_markdown_links.py",
        "cargo audit --deny warnings",
        "cargo deny check",
        "npm ci --include=dev",
        "npm audit --omit=dev --audit-level=high",
        "playwright install --with-deps chromium",
        "npm run typecheck:web",
        "npm run lint:web",
        "npm run test:web",
        "npm run test:web:e2e",
        "npm run build:web",
        "npm run test:web:standalone",
        "npm audit --audit-level=high",
        "sqlx migrate run",
        "psql \"$DATABASE_URL\" -v ON_ERROR_STOP=1 -f fixtures/seed/local-demo.sql",
        "psql \"$DATABASE_URL\" -v ON_ERROR_STOP=1 -f fixtures/seed/local-demo-data-quality.sql",
        "TEST_DATABASE_URL:",
        "cargo-llvm-cov",
        "coverage/lcov.info",
        "coverage/rust-coverage-summary.json",
        "uv run --with pytest --with pyyaml pytest scripts/tests -q",
    ]

    missing = [fragment for fragment in required_fragments if fragment not in workflow]
    assert not missing, f"CI is missing enterprise release gates: {missing}"


def test_ci_runs_for_feature_branch_pushes_and_does_not_float_toolchain():
    workflow = CI.read_text()

    assert "branches: [main]" not in workflow
    assert "dtolnay/rust-toolchain@stable" not in workflow
    assert (REPO_ROOT / "rust-toolchain.toml").read_text().count('channel = "stable"') == 0


def test_supply_chain_policy_is_checked_in():
    deny = (REPO_ROOT / "deny.toml").read_text()
    dependabot = (REPO_ROOT / ".github" / "dependabot.yml").read_text()

    assert "[advisories]" in deny
    assert "[licenses]" in deny
    assert "[bans]" in deny
    assert "[sources]" in deny
    assert 'package-ecosystem: "cargo"' in dependabot
    assert 'package-ecosystem: "npm"' in dependabot
    assert 'package-ecosystem: "github-actions"' in dependabot


def test_ci_owns_pinned_wasm_coverage_prerequisites_and_canonical_invocation():
    workflow = yaml.safe_load(CI.read_text())
    coverage_steps = workflow["jobs"]["rust-coverage"]["steps"]
    named_steps = {step.get("name"): step for step in coverage_steps if "name" in step}

    prerequisites = named_steps["Install pinned SpacetimeDB WASM coverage prerequisites"]["run"]
    assert "apt-get install --yes clang" in prerequisites
    assert (
        "https://github.com/clockworklabs/SpacetimeDB/releases/download/"
        "v2.6.0/spacetime-x86_64-unknown-linux-gnu.tar.gz"
    ) in prerequisites
    assert "24f5c011a1c5b14e9458f230f67f4700f2c89ce1d26bfee35c1f4d3570dc1ac8" in prerequisites
    assert "sha256sum --check" in prerequisites
    assert "install " in prerequisites and "/usr/local/bin/spacetime" in prerequisites
    assert "spacetime --version" in prerequisites

    coverage_run = named_steps["Generate and enforce measured Rust coverage"]["run"]
    assert "./scripts/capture_spacetimedb_wasm_coverage.sh" not in coverage_run
    assert coverage_run.count("./scripts/check_rust_coverage.sh") == 1
    assert "RUST_COVERAGE_ADDITIONAL_LCOV" not in coverage_run


def test_canonical_quality_audit_matches_live_architecture_and_coverage_evidence():
    architecture = subprocess.run(
        [sys.executable, "scripts/check_architecture_quality.py", "--repo-root", "."],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert architecture.returncode == 0, architecture.stderr
    match = re.search(r"production_rust_files=(\d+)", architecture.stdout)
    assert match is not None

    audit = (REPO_ROOT / "docs" / "quality" / "codebase-quality-convergence-audit.md").read_text()
    summary = json.loads(
        (REPO_ROOT / "coverage" / "rust-coverage-summary.json").read_text()
    )
    assert f"- {match.group(1)} production Rust files" in audit
    assert (
        f"- {summary['line']['covered']:,} / {summary['line']['total']:,} lines covered "
        f"({summary['line']['percent']:.2f}%)"
    ) in audit
    assert (
        f"- {summary['branch']['covered']:,} / {summary['branch']['total']:,} branches covered "
        f"({summary['branch']['percent']:.2f}%)"
    ) in audit
    assert (
        f"- {summary['changed_production']['covered_executable_lines']:,} / "
        f"{summary['changed_production']['executable_lines']:,} changed executable lines covered"
    ) in audit
    assert "documented unmintable" not in audit.lower()
    assert "SpaceTimeDB WASM" in audit
    assert "Blocking:" not in audit
