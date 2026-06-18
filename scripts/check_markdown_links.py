#!/usr/bin/env python3
"""Repo-local Markdown navigation contracts for the maintainer wiki.

This script is intentionally offline and deterministic. It checks only repository
files and guards the README/wiki surface that helps humans and agents orient to
labor-cost workflows without stale links or missing module entrypoints.
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path
from urllib.parse import unquote, urlparse

EXCLUDED_DIR_NAMES = {
    ".git",
    ".hg",
    ".svn",
    "target",
    "node_modules",
    "vendor",
    "generated",
    "dist",
    "build",
    ".next",
    "coverage",
    ".venv",
    "venv",
}

REQUIRED_DOMAIN_MODULE_READMES = (
    "boarding",
    "daycare",
    "grooming",
    "money",
    "payment",
    "reservation",
    "retail",
    "training",
)

REQUIRED_STATIC_READMES = (
    "storage/src/service_line/README.md",
    "integrations/gingr/src/endpoint/README.md",
    "integrations/gingr/src/dto/README.md",
    "integrations/gingr/src/mapping/README.md",
    "docs/integrations/gingr/README.md",
    "docs/integrations/gingr/fixtures/webhooks/README.md",
)

LINK_PATTERN = re.compile(r"(?<!!)\[[^\]]+\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")
REFERENCE_LINK_PATTERN = re.compile(r"^\s*\[[^\]]+\]:\s+(\S+)", re.MULTILINE)
HEADING_MARKER_PATTERN = re.compile(r"[^a-z0-9 _-]")
WHITESPACE_PATTERN = re.compile(r"\s+")
CODE_FENCE_PATTERN = re.compile(r"```.*?```", re.DOTALL)


def iter_markdown_files(repo_root: Path):
    for path in sorted(repo_root.rglob("*.md")):
        relative_parts = path.relative_to(repo_root).parts
        if any(part in EXCLUDED_DIR_NAMES for part in relative_parts):
            continue
        yield path


def is_external_or_nonlocal_link(raw_target: str) -> bool:
    parsed = urlparse(raw_target)
    if parsed.scheme in {"http", "https", "mailto", "tel", "sms", "data"}:
        return True
    if parsed.scheme and parsed.scheme not in {"file"}:
        return True
    return False


def markdown_anchor_for(heading_line: str) -> str | None:
    stripped = heading_line.strip()
    if not stripped.startswith("#"):
        return None
    title = stripped.lstrip("#").strip().rstrip("#").strip()
    if not title:
        return None
    lowered = title.lower()
    cleaned = HEADING_MARKER_PATTERN.sub("", lowered)
    collapsed = WHITESPACE_PATTERN.sub("-", cleaned.strip())
    return collapsed


def anchors_in(path: Path) -> set[str]:
    anchors: set[str] = {""}
    counts: dict[str, int] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        anchor = markdown_anchor_for(line)
        if not anchor:
            continue
        duplicate_index = counts.get(anchor, 0)
        counts[anchor] = duplicate_index + 1
        anchors.add(anchor if duplicate_index == 0 else f"{anchor}-{duplicate_index}")
    return anchors


def link_targets(markdown: str) -> list[str]:
    body = CODE_FENCE_PATTERN.sub("", markdown)
    return LINK_PATTERN.findall(body) + REFERENCE_LINK_PATTERN.findall(body)


def split_link_target(raw_target: str) -> tuple[str, str | None]:
    target = raw_target.strip("<>")
    parsed = urlparse(target)
    if parsed.scheme == "file":
        target = parsed.path
    if "#" not in target:
        return target, None
    path_part, fragment = target.split("#", 1)
    return path_part, unquote(fragment) if fragment else None


def resolve_local_target(markdown_file: Path, path_part: str) -> Path:
    if not path_part:
        return markdown_file
    decoded = unquote(path_part)
    return (markdown_file.parent / decoded).resolve()


def is_within_repo(repo_root: Path, target: Path) -> bool:
    try:
        target.relative_to(repo_root.resolve())
    except ValueError:
        return False
    return True


def check_local_markdown_links(repo_root: Path) -> list[str]:
    failures: list[str] = []
    repo_root = repo_root.resolve()
    anchor_cache: dict[Path, set[str]] = {}
    for markdown_file in iter_markdown_files(repo_root):
        relative_file = markdown_file.relative_to(repo_root)
        text = CODE_FENCE_PATTERN.sub("", markdown_file.read_text(encoding="utf-8"))
        for line_number, line in enumerate(text.splitlines(), start=1):
            for raw_target in LINK_PATTERN.findall(line) + REFERENCE_LINK_PATTERN.findall(line):
                if is_external_or_nonlocal_link(raw_target):
                    continue
                path_part, fragment = split_link_target(raw_target)
                target = resolve_local_target(markdown_file, path_part)
                if not is_within_repo(repo_root, target):
                    failures.append(
                        f"{relative_file}:{line_number}: local link leaves repository: {raw_target}"
                    )
                    continue
                if not target.exists():
                    failures.append(
                        f"{relative_file}:{line_number}: missing local markdown link target: {raw_target}"
                    )
                    continue
                if fragment and target.suffix.lower() in {".md", ".markdown"}:
                    anchors = anchor_cache.setdefault(target, anchors_in(target))
                    if fragment not in anchors:
                        failures.append(
                            f"{relative_file}:{line_number}: missing markdown anchor #{fragment} in {target.relative_to(repo_root)}"
                        )
    return failures


def workspace_members(repo_root: Path) -> list[str]:
    cargo_toml = repo_root / "Cargo.toml"
    if not cargo_toml.exists():
        return []
    data = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    members = data.get("workspace", {}).get("members", [])
    return [member for member in members if isinstance(member, str)]


def root_readme_links_to(repo_root: Path, relative_target: str) -> bool:
    readme = repo_root / "README.md"
    if not readme.exists():
        return False
    normalized = relative_target.replace("\\", "/")
    return normalized in readme.read_text(encoding="utf-8")


def check_required_readme_coverage(repo_root: Path) -> list[str]:
    failures: list[str] = []
    repo_root = repo_root.resolve()

    for member in workspace_members(repo_root):
        readme_relative = f"{member}/README.md"
        if not (repo_root / readme_relative).exists():
            failures.append(f"workspace member {member} is missing README.md at {readme_relative}")
        if not root_readme_links_to(repo_root, readme_relative):
            failures.append(f"root README is missing link to {readme_relative}")

    for module in REQUIRED_DOMAIN_MODULE_READMES:
        readme_relative = f"domain/src/{module}/README.md"
        if not (repo_root / readme_relative).exists():
            failures.append(f"domain module {module} is missing {readme_relative}")
        if not root_readme_links_to(repo_root, readme_relative):
            failures.append(f"root README is missing link to {readme_relative}")

    for readme_relative in REQUIRED_STATIC_READMES:
        if not (repo_root / readme_relative).exists():
            failures.append(f"missing required navigation README at {readme_relative}")
        if not root_readme_links_to(repo_root, readme_relative):
            failures.append(f"root README is missing link to {readme_relative}")

    return failures


def run(repo_root: Path) -> int:
    link_failures = check_local_markdown_links(repo_root)
    coverage_failures = check_required_readme_coverage(repo_root)

    if link_failures or coverage_failures:
        print("Markdown README/wiki contract check failed:", file=sys.stderr)
        if link_failures:
            print("broken local markdown links:", file=sys.stderr)
            for failure in link_failures:
                print(f"- {failure}", file=sys.stderr)
        if coverage_failures:
            print("missing required docs or README coverage:", file=sys.stderr)
            for failure in coverage_failures:
                print(f"- {failure}", file=sys.stderr)
        return 1

    repo_root = repo_root.resolve()
    markdown_count = sum(1 for _ in iter_markdown_files(repo_root))
    required_count = len(workspace_members(repo_root)) + len(REQUIRED_DOMAIN_MODULE_READMES) + len(REQUIRED_STATIC_READMES)
    print(
        f"Markdown README/wiki contract check passed: {markdown_count} markdown files scanned; "
        f"{required_count} required README entries checked."
    )
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Check repo-local Markdown links and required README coverage.")
    parser.add_argument(
        "--repo-root",
        "--root",
        dest="repo_root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="repository root to check (defaults to the parent of scripts/)",
    )
    args = parser.parse_args(argv)
    return run(args.repo_root)


if __name__ == "__main__":
    raise SystemExit(main())
