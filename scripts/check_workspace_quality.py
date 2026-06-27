#!/usr/bin/env python3
"""Workspace guardrails for Kanban closeout quality.

This check is intentionally small and mechanical. It catches the kinds of
stale/noisy state that are easy to miss in a busy shared Kanban workspace:
changed Markdown that reintroduces stale-process wording, tracked/untracked
cache artifacts, and drift between the Axum v0 route surface and the checked
OpenAPI artifact.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

STALE_MARKDOWN_TERMS: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("TODO", re.compile(r"\bTODO\b", re.IGNORECASE)),
    ("planned_not_wired", re.compile(r"planned_not_wired", re.IGNORECASE)),
    ("not wired", re.compile(r"\bnot\s+wired\b", re.IGNORECASE)),
    ("placeholder", re.compile(r"\bplaceholders?\b", re.IGNORECASE)),
    ("final-integration", re.compile(r"\bfinal[- ]integration\b", re.IGNORECASE)),
    ("scaffold", re.compile(r"\bscaffolds?\b|\bscaffolded\b", re.IGNORECASE)),
)

NOISY_ARTIFACT_PATTERNS: tuple[re.Pattern[str], ...] = (
    re.compile(r"(^|/)__pycache__(/|$)"),
    re.compile(r"\.py[co]$"),
    re.compile(r"(^|/)\.pytest_cache(/|$)"),
    re.compile(r"(^|/)\.ruff_cache(/|$)"),
    re.compile(r"(^|/)\.mypy_cache(/|$)"),
    re.compile(r"(^|/)node_modules(/|$)"),
    re.compile(r"(^|/)target/(debug|release|doc|tmp|docs-smoke)(/|$)"),
    re.compile(r"\.tsbuildinfo$"),
    re.compile(r"\.(dump|sqlite3?|db|bak|tar|tgz|gz|zip)$", re.IGNORECASE),
)

ROUTE_LITERAL = re.compile(r"\.route\(\s*\"([^\"]+)\"", re.DOTALL)


@dataclass(frozen=True)
class Finding:
    category: str
    path: str
    detail: str

    def render(self) -> str:
        return f"{self.category}: {self.path}: {self.detail}"


def run_git(repo_root: Path, args: list[str]) -> list[str]:
    result = subprocess.run(
        ["git", *args],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        return []
    return [line for line in result.stdout.splitlines() if line]


def repo_paths(repo_root: Path, args: list[str]) -> set[str]:
    return {line.strip() for line in run_git(repo_root, args) if line.strip()}


def changed_paths(repo_root: Path) -> set[str]:
    paths: set[str] = set()
    paths.update(repo_paths(repo_root, ["diff", "--name-only", "--diff-filter=ACMR", "HEAD"]))
    paths.update(
        repo_paths(repo_root, ["diff", "--cached", "--name-only", "--diff-filter=ACMR"])
    )
    if paths:
        return paths
    return repo_paths(repo_root, ["ls-files"])


def untracked_paths(repo_root: Path) -> set[str]:
    return repo_paths(repo_root, ["ls-files", "--others", "--exclude-standard"])


def is_noisy_artifact(path: str) -> bool:
    normalized = path.replace("\\", "/")
    return any(pattern.search(normalized) for pattern in NOISY_ARTIFACT_PATTERNS)


def markdown_paths_to_scan(repo_root: Path, paths: set[str]) -> list[Path]:
    selected = []
    for rel in sorted(paths):
        if Path(rel).suffix.lower() not in {".md", ".markdown"}:
            continue
        full = repo_root / rel
        if full.is_file():
            selected.append(full)
    return selected


def stale_markdown_findings(repo_root: Path, paths: set[str]) -> list[Finding]:
    findings: list[Finding] = []
    for path in markdown_paths_to_scan(repo_root, paths):
        rel = path.relative_to(repo_root).as_posix()
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            for label, pattern in STALE_MARKDOWN_TERMS:
                if pattern.search(line):
                    findings.append(
                        Finding(
                            "stale/noisy wording",
                            rel,
                            f"line {lineno}: term `{label}` requires explicit owner-visible rationale or removal",
                        )
                    )
    return findings


def noisy_artifact_findings(paths: set[str], category: str) -> list[Finding]:
    return [
        Finding("noisy workspace artifact", path, category)
        for path in sorted(paths)
        if is_noisy_artifact(path)
    ]


def openapi_route_findings(repo_root: Path) -> tuple[list[Finding], int, int]:
    http_rs = repo_root / "apps" / "api" / "src" / "http.rs"
    openapi_json = repo_root / "apps" / "api" / "openapi" / "owned-operations-v0.openapi.json"
    if not http_rs.exists() and not openapi_json.exists():
        return [], 0, 0
    if not http_rs.exists():
        return [Finding("OpenAPI/source consistency", http_rs.as_posix(), "missing Axum route source")], 0, 0
    if not openapi_json.exists():
        return [Finding("OpenAPI/source consistency", openapi_json.as_posix(), "missing OpenAPI artifact")], 0, 0

    source = http_rs.read_text(encoding="utf-8")
    axum_v0_routes = sorted({route for route in ROUTE_LITERAL.findall(source) if route.startswith("/v0/")})
    try:
        spec = json.loads(openapi_json.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return [Finding("OpenAPI/source consistency", openapi_json.as_posix(), f"invalid JSON: {exc}")], len(axum_v0_routes), 0

    openapi_paths = spec.get("paths")
    if not isinstance(openapi_paths, dict):
        return [Finding("OpenAPI/source consistency", openapi_json.as_posix(), "missing object `paths`")], len(axum_v0_routes), 0

    spec_routes = sorted(openapi_paths)
    missing = [route for route in axum_v0_routes if route not in openapi_paths]
    extra = [route for route in spec_routes if route.startswith("/v0/") and route not in axum_v0_routes]

    findings: list[Finding] = []
    if missing:
        findings.append(
            Finding(
                "OpenAPI missing v0 routes",
                openapi_json.relative_to(repo_root).as_posix(),
                ", ".join(missing),
            )
        )
    if extra:
        findings.append(
            Finding(
                "OpenAPI has stale v0 routes",
                openapi_json.relative_to(repo_root).as_posix(),
                ", ".join(extra),
            )
        )
    return findings, len(axum_v0_routes), len([route for route in spec_routes if route.startswith("/v0/")])


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Check NVA workspace closeout guardrails.")
    parser.add_argument("--repo-root", default=".", help="Repository root to check (default: current directory).")
    parser.add_argument(
        "--all-markdown",
        action="store_true",
        help="Scan every tracked Markdown file for stale wording instead of only changed files.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    repo_root = Path(args.repo_root).resolve()
    if not repo_root.exists():
        print(f"repo root does not exist: {repo_root}", file=sys.stderr)
        return 2

    tracked_paths = repo_paths(repo_root, ["ls-files"])
    untracked = untracked_paths(repo_root)
    paths_for_markdown = tracked_paths if args.all_markdown else changed_paths(repo_root) | untracked

    findings: list[Finding] = []
    findings.extend(noisy_artifact_findings(tracked_paths, "tracked file should be removed or ignored"))
    findings.extend(noisy_artifact_findings(untracked, "untracked file should be removed, ignored, or intentionally staged elsewhere"))
    findings.extend(stale_markdown_findings(repo_root, paths_for_markdown))
    openapi_findings, axum_v0_count, openapi_v0_count = openapi_route_findings(repo_root)
    findings.extend(openapi_findings)

    if findings:
        print("workspace quality guardrail failed:", file=sys.stderr)
        for finding in findings:
            print(f"- {finding.render()}", file=sys.stderr)
        return 1

    markdown_count = len(markdown_paths_to_scan(repo_root, paths_for_markdown))
    print(
        "workspace_quality_ok "
        f"markdown_scanned={markdown_count} "
        f"openapi_v0_routes={openapi_v0_count} "
        f"axum_v0_routes={axum_v0_count} "
        f"untracked_noisy_artifacts=0"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
