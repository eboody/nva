from pathlib import Path


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
        "cargo doc --workspace --no-deps --locked",
        "python scripts/check_rustdoc_completeness.py",
        "python scripts/check_workspace_quality.py --repo-root .",
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
        "TEST_DATABASE_URL:",
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
