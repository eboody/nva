#!/usr/bin/env python3
"""Deterministic, ratcheted architecture-quality checks for NVA.

The checker scans production Rust source directly and compares current structural
measurements with an exact checked debt manifest. Repository-wide dependency-cycle
debt comes from the checked analyzer manifest because Repowise is not available in
every CI runner; entity concept-owner cycles are derived from live Rust imports.
Any debt reduction makes the manifest stale on purpose, forcing the checked ceiling
downward before later changes can reintroduce it.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from generate_public_api_inventory import (
    ApiItem,
    ScipReferenceIndex,
    SourceIndex,
    consumers as inventory_consumers,
    source_index,
    symbol_is_referenced,
)

DEFAULT_BASELINE = Path("docs/quality/architecture-quality-baseline.json")
EXCLUDED_SOURCE_PARTS = frozenset({"target", "tests", "test", "examples", "benches"})
COMPATIBILITY_BOUNDARY_PARTS = frozenset({"compatibility"})
COMPATIBILITY_IDENTIFIER = re.compile(
    r"\b[A-Za-z0-9_]*(?:legacy|compatibility|historical)[A-Za-z0-9_]*\b",
    re.IGNORECASE,
)
HISTORICAL_PUBLIC_PATH = re.compile(r"\bstrategic_ai_ops\b")
BROAD_REEXPORT = re.compile(r"\bpub\s+use\s+[^;]*\*\s*;", re.MULTILINE)
PUBLIC_REEXPORT_ALIAS = re.compile(
    r"\bpub\s+use\s+[^;]*\bas\s+[A-Za-z_][A-Za-z0-9_]*[^;]*;",
    re.MULTILINE,
)
PROVIDER_DTO_PATH = re.compile(r"\bgingr\s*::\s*(?:dto|response)\b")
COMPATIBILITY_DEPENDENCY_PATH = re.compile(r"\bcompatibility\s*::")
PROVIDER_BRAND = re.compile(r"\bgingr\b", re.IGNORECASE)
PROVIDER_MODEL_PATH_LITERAL = re.compile(
    r"\bgingr\s*::\s*(?:dto|response)\b", re.IGNORECASE
)
PROVIDER_FIXTURE_EVIDENCE = re.compile(
    r"\b[A-Za-z0-9_]*gingr[A-Za-z0-9_]*\b|fixture://(?:mock-)?gingr[^\"\s]*|Mock Gingr[^\"\n]*|\bprovider_(?:reservation_record|payload)\b",
    re.IGNORECASE,
)
APP_SOURCE_PAYLOAD_TYPE = re.compile(
    r"\bpub\s+struct\s+(?P<name>(?:[A-Za-z_][A-Za-z0-9_]*)?Source[A-Za-z0-9_]*Payload[A-Za-z0-9_]*)\b"
)
APP_SOURCE_EVIDENCE_MODEL_PATH = re.compile(
    r"\b(?P<name>source_model_path|provider_model_path|promotion_target_path|nva_target_model_path)\b",
    re.IGNORECASE,
)
PUBLIC_API_DECLARATION = re.compile(
    r"\bpub\s+(?:unsafe\s+)?(?P<kind>struct|enum|trait|type|fn)\s+"
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)\b"
)
HIGH_RISK_PUBLIC_FAMILY = re.compile(
    r"(?:Action|Boundary|Classification|Packet|Plan|Recommendation|Record|Row)$"
)
PUBLIC_ACTION_ENUM = re.compile(
    r"\bpub\s+enum\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*(?:Action|Boundary))\s*\{"
)
ACTIVE_DOC_ROOTS = frozenset(
    {"architecture", "demo", "domain", "ops", "presentation", "quality", "runbooks"}
)
ARCHIVED_DOC_MARKER = "<!-- archived-historical-record -->"
AUTHORITY_TYPE_NAME = r"[A-Za-z_][A-Za-z0-9_]*(?:Authority|Authorization|Capability)"
AUTHORITY_STRUCT_HEADER = re.compile(
    rf"(?P<attributes>(?:\s*#\[[^\]]*\]\s*)*)"
    rf"(?P<visibility>pub(?:\([^)]*\))?\s+)?struct\s+"
    rf"(?P<name>{AUTHORITY_TYPE_NAME})\b",
    re.MULTILINE,
)
PLAIN_PUBLIC_FIELD = re.compile(
    r"(?:^|,)\s*pub\s+[A-Za-z_][A-Za-z0-9_]*\s*:", re.MULTILINE
)
PLAIN_PUBLIC_TUPLE_FIELD = re.compile(r"(?:^|,)\s*pub\s+")
FORBIDDEN_DOMAIN_DEPENDENCIES = frozenset(
    {
        "app",
        "storage",
        "gingr",
        "pet-resort-api",
        "pet-resort-worker",
        "nva-spacetimedb",
    }
)
RUNTIME_SHELLS = {
    "apps/api/Cargo.toml": ("pet-resort-api", "pet-resort-worker"),
    "apps/worker/Cargo.toml": ("pet-resort-worker", "pet-resort-api"),
}


@dataclass(frozen=True, order=True)
class Finding:
    category: str
    path: str
    detail: str

    def render(self) -> str:
        return f"{self.category}: {self.path}: {self.detail}"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check ratcheted NVA architecture quality."
    )
    parser.add_argument(
        "--repo-root",
        default=".",
        help="Repository root to check (default: current directory).",
    )
    parser.add_argument(
        "--baseline",
        default=None,
        help=(
            "Checked JSON baseline (default: "
            "docs/quality/architecture-quality-baseline.json under the repo root)."
        ),
    )
    return parser.parse_args(argv)


def production_rust_files(repo_root: Path) -> list[Path]:
    files: list[Path] = []
    for path in repo_root.rglob("*.rs"):
        relative = path.relative_to(repo_root)
        parts = relative.parts
        if "src" not in parts:
            continue
        if any(part in EXCLUDED_SOURCE_PARTS for part in parts):
            continue
        if path.stem.endswith(("_test", "_tests")):
            continue
        if path.is_file():
            files.append(path)
    return sorted(files, key=lambda path: path.relative_to(repo_root).as_posix())


def physical_line_count(source: str) -> int:
    return len(source.splitlines())


def compatibility_boundary(path: Path) -> bool:
    semantic_parts = {part.lower() for part in path.with_suffix("").parts}
    return bool(semantic_parts & COMPATIBILITY_BOUNDARY_PARTS)


def raw_string_end(source: str, start: int) -> int | None:
    prefix_length = 2 if source.startswith("br", start) else 1
    if not source.startswith("r", start + prefix_length - 1):
        return None
    if start > 0 and (source[start - 1].isalnum() or source[start - 1] == "_"):
        return None

    delimiter = start + prefix_length
    while delimiter < len(source) and source[delimiter] == "#":
        delimiter += 1
    if delimiter >= len(source) or source[delimiter] != '"':
        return None

    closing = '"' + source[start + prefix_length : delimiter]
    closing_start = source.find(closing, delimiter + 1)
    return len(source) if closing_start == -1 else closing_start + len(closing)


def quoted_string_end(source: str, quote: int) -> int:
    cursor = quote + 1
    while cursor < len(source):
        if source[cursor] == "\\":
            cursor += 2
        elif source[cursor] == '"':
            return cursor + 1
        else:
            cursor += 1
    return len(source)


def character_literal_end(source: str, quote: int) -> int | None:
    cursor = quote + 1
    if cursor >= len(source) or source[cursor] in "'\r\n":
        return None
    if source[cursor] != "\\":
        cursor += 1
    elif cursor + 1 >= len(source):
        return None
    elif source[cursor + 1] == "x":
        cursor += 4
    elif source[cursor + 1] == "u" and source.startswith("{", cursor + 2):
        closing_brace = source.find("}", cursor + 3)
        if closing_brace == -1:
            return None
        cursor = closing_brace + 1
    else:
        cursor += 2
    return cursor + 1 if cursor < len(source) and source[cursor] == "'" else None


def blank_span(masked: list[str], start: int, end: int) -> None:
    for index in range(start, end):
        if masked[index] not in "\r\n":
            masked[index] = " "


def code_without_comments_or_literals(source: str) -> str:
    masked = list(source)
    cursor = 0
    while cursor < len(source):
        end: int | None = None
        if source.startswith("//", cursor):
            newline = source.find("\n", cursor + 2)
            end = len(source) if newline == -1 else newline
        elif source.startswith("/*", cursor):
            depth = 1
            end = cursor + 2
            while end < len(source) and depth:
                if source.startswith("/*", end):
                    depth += 1
                    end += 2
                elif source.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
        elif source.startswith(("br", "r"), cursor):
            end = raw_string_end(source, cursor)
        elif source.startswith('b"', cursor):
            end = quoted_string_end(source, cursor + 1)
        elif source[cursor] == '"':
            end = quoted_string_end(source, cursor)
        elif source.startswith("b'", cursor):
            end = character_literal_end(source, cursor + 1)
        elif source[cursor] == "'":
            end = character_literal_end(source, cursor)

        if end is None:
            cursor += 1
            continue
        blank_span(masked, cursor, end)
        cursor = end
    return "".join(masked)


def top_level_use_declarations(code: str) -> list[str]:
    declarations: list[str] = []
    depth = 0
    index = 0
    while index < len(code):
        is_use = code.startswith("use", index)
        starts_at_boundary = index == 0 or not (
            code[index - 1].isalnum() or code[index - 1] == "_"
        )
        end_of_keyword = index + 3
        ends_at_boundary = end_of_keyword >= len(code) or not (
            code[end_of_keyword].isalnum() or code[end_of_keyword] == "_"
        )
        if depth == 0 and is_use and starts_at_boundary and ends_at_boundary:
            end = code.find(";", index)
            if end == -1:
                break
            declarations.append(code[index : end + 1])
            index = end + 1
            continue
        if code[index] == "{":
            depth += 1
        elif code[index] == "}":
            depth = max(0, depth - 1)
        index += 1
    return declarations


def expanded_use_paths(declaration: str) -> list[tuple[tuple[str, ...], str | None]]:
    """Expand one Rust use tree into canonical paths and optional aliases."""
    tree = declaration.strip()
    if not tree.startswith("use") or not tree.endswith(";"):
        return []
    tokens = re.findall(r"[A-Za-z_][A-Za-z0-9_]*|::|[{},*]", tree[3:-1])
    index = 0

    def parse_group(
        prefix: tuple[str, ...],
    ) -> list[tuple[tuple[str, ...], str | None]]:
        nonlocal index
        paths: list[tuple[tuple[str, ...], str | None]] = []
        index += 1  # opening brace
        while index < len(tokens) and tokens[index] != "}":
            paths.extend(parse_tree(prefix))
            if index < len(tokens) and tokens[index] == ",":
                index += 1
        if index < len(tokens) and tokens[index] == "}":
            index += 1
        return paths

    def parse_tree(prefix: tuple[str, ...]) -> list[tuple[tuple[str, ...], str | None]]:
        nonlocal index
        while index < len(tokens) and tokens[index] == "::":
            index += 1
        if index >= len(tokens):
            return []
        if tokens[index] == "{":
            return parse_group(prefix)

        segment = tokens[index]
        index += 1
        path = prefix if segment == "self" and prefix else prefix + (segment,)
        while index < len(tokens) and tokens[index] == "::":
            index += 1
            if index < len(tokens) and tokens[index] == "{":
                return parse_group(path)
            if index >= len(tokens):
                break
            path += (tokens[index],)
            index += 1

        alias = None
        if index < len(tokens) and tokens[index] == "as":
            index += 1
            if index < len(tokens):
                alias = tokens[index]
                index += 1
        return [(path, alias)]

    return parse_tree(())


def entity_owner_dependencies(repo_root: Path) -> dict[str, set[str]]:
    owner_root = repo_root / "domain" / "src" / "entities"
    if not owner_root.is_dir():
        return {}

    owner_paths = sorted(owner_root.glob("*.rs"), key=lambda path: path.name)
    owners = {path.stem for path in owner_paths}
    graph = {owner: set() for owner in sorted(owners)}
    for path in owner_paths:
        code = code_without_comments_or_literals(path.read_text(encoding="utf-8"))
        use_paths = [
            use_path
            for declaration in top_level_use_declarations(code)
            for use_path in expanded_use_paths(declaration)
        ]
        canonical_owner_roots = {("super",), ("crate", "entities")}
        owner_root_aliases = {
            alias
            for use_path, alias in use_paths
            if use_path in canonical_owner_roots and alias is not None
        }

        for use_path, _alias in use_paths:
            if use_path[:1] == ("super",) and len(use_path) > 1:
                dependency = use_path[1]
            elif use_path[:2] == ("crate", "entities") and len(use_path) > 2:
                dependency = use_path[2]
            elif (
                use_path[:1] in {(alias,) for alias in owner_root_aliases}
                and len(use_path) > 1
            ):
                dependency = use_path[1]
            else:
                continue
            if dependency in owners and dependency != path.stem:
                graph[path.stem].add(dependency)

        owner_roots = [r"super", r"crate\s*::\s*entities"]
        owner_roots.extend(re.escape(alias) for alias in sorted(owner_root_aliases))
        owner_root = "|".join(owner_roots)
        graph[path.stem].update(
            owner
            for owner in owners
            if owner != path.stem
            and re.search(
                rf"\b(?:{owner_root})\s*::\s*{re.escape(owner)}\b",
                code,
            )
            is not None
        )
    return graph


def first_dependency_cycle(graph: dict[str, set[str]]) -> list[str] | None:
    visited: set[str] = set()
    active: list[str] = []
    active_positions: dict[str, int] = {}

    def visit(owner: str) -> list[str] | None:
        visited.add(owner)
        active_positions[owner] = len(active)
        active.append(owner)
        for dependency in sorted(graph[owner]):
            if dependency in active_positions:
                start = active_positions[dependency]
                return active[start:] + [dependency]
            if dependency not in visited:
                cycle = visit(dependency)
                if cycle is not None:
                    return cycle
        active.pop()
        del active_positions[owner]
        return None

    for owner in sorted(graph):
        if owner not in visited:
            cycle = visit(owner)
            if cycle is not None:
                return cycle
    return None


def entity_owner_cycle_findings(repo_root: Path) -> list[Finding]:
    cycle = first_dependency_cycle(entity_owner_dependencies(repo_root))
    if cycle is None:
        return []
    return [
        Finding(
            "entity owner dependency cycle",
            "domain/src/entities",
            " -> ".join(cycle),
        )
    ]


def historical_public_path_findings(repo_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    for path in production_rust_files(repo_root):
        relative = path.relative_to(repo_root)
        if compatibility_boundary(relative):
            continue
        code = code_without_comments_or_literals(path.read_text(encoding="utf-8"))
        if HISTORICAL_PUBLIC_PATH.search(code):
            findings.append(
                Finding(
                    "historical public path",
                    relative.as_posix(),
                    "active production code references denied path `strategic_ai_ops`",
                )
            )
    return findings


def workspace_dependencies(manifest_path: Path) -> dict[str, object]:
    for directory in (manifest_path.parent, *manifest_path.parents):
        candidate = directory / "Cargo.toml"
        if not candidate.is_file():
            continue
        manifest = tomllib.loads(candidate.read_text(encoding="utf-8"))
        workspace = manifest.get("workspace")
        if not isinstance(workspace, dict):
            continue
        dependencies = workspace.get("dependencies")
        return dependencies if isinstance(dependencies, dict) else {}
    return {}


def cargo_dependencies(manifest_path: Path) -> dict[str, str]:
    manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    inherited_dependencies = workspace_dependencies(manifest_path)
    dependencies: dict[str, str] = {}

    def add_table(table: object) -> None:
        if not isinstance(table, dict):
            return
        for declared_name, declaration in table.items():
            effective_declaration = declaration
            if isinstance(declaration, dict) and declaration.get("workspace") is True:
                effective_declaration = inherited_dependencies.get(
                    declared_name, declaration
                )
            package_name = (
                effective_declaration.get("package", declared_name)
                if isinstance(effective_declaration, dict)
                else declared_name
            )
            if isinstance(package_name, str):
                dependencies[declared_name] = package_name

    for section_name in ("dependencies", "build-dependencies"):
        add_table(manifest.get(section_name))
    targets = manifest.get("target")
    if isinstance(targets, dict):
        for target in targets.values():
            if isinstance(target, dict):
                for section_name in ("dependencies", "build-dependencies"):
                    add_table(target.get(section_name))
    return dependencies


def cargo_dependency_names(manifest_path: Path) -> set[str]:
    return set(cargo_dependencies(manifest_path).values())


def manifest_dependency_findings(repo_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    domain_manifest = repo_root / "domain" / "Cargo.toml"
    if domain_manifest.is_file():
        forbidden = sorted(
            cargo_dependency_names(domain_manifest) & FORBIDDEN_DOMAIN_DEPENDENCIES
        )
        for dependency in forbidden:
            findings.append(
                Finding(
                    "bounded-context dependency",
                    "domain/Cargo.toml",
                    f"domain must not depend on storage, integration, or runtime crate `{dependency}`",
                )
            )

    app_manifest = repo_root / "app" / "Cargo.toml"
    if app_manifest.is_file() and "storage" in cargo_dependency_names(app_manifest):
        findings.append(
            Finding(
                "bounded-context dependency",
                "app/Cargo.toml",
                "application contracts must not depend on storage-owned semantics",
            )
        )

    for relative, (shell_name, peer_name) in RUNTIME_SHELLS.items():
        manifest_path = repo_root / relative
        if not manifest_path.is_file():
            continue
        dependencies = cargo_dependency_names(manifest_path)
        if "app" not in dependencies:
            findings.append(
                Finding(
                    "runtime application contract",
                    relative,
                    f"{shell_name} must depend on application-owned contracts through `app`",
                )
            )
        if peer_name in dependencies:
            findings.append(
                Finding(
                    "runtime shell dependency",
                    relative,
                    f"{shell_name} must not depend on peer shell `{peer_name}`",
                )
            )
    for manifest_path in sorted(repo_root.rglob("Cargo.toml")):
        relative = manifest_path.relative_to(repo_root)
        if relative.parts[:2] == ("integrations", "gingr"):
            continue
        gingr_aliases = sorted(
            declared_name
            for declared_name, package_name in cargo_dependencies(manifest_path).items()
            if package_name == "gingr"
        )
        if gingr_aliases:
            aliases = ", ".join(f"`{alias}`" for alias in gingr_aliases)
            findings.append(
                Finding(
                    "provider DTO quarantine",
                    relative.as_posix(),
                    "provider crate dependency `gingr` "
                    f"(declared as {aliases}) is only legal in integrations/gingr",
                )
            )
    return findings


def source_boundary_findings(repo_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    for path in production_rust_files(repo_root):
        relative = path.relative_to(repo_root)
        relative_text = relative.as_posix()
        code = code_without_comments_or_literals(path.read_text(encoding="utf-8"))
        if relative.parts[:2] != ("integrations", "gingr"):
            provider_identifier = re.search(
                r"\bGingr[A-Za-z0-9_]*\b|::\s*Gingr\b", code
            )
            if provider_identifier is not None:
                findings.append(
                    Finding(
                        "provider vocabulary quarantine",
                        relative_text,
                        "provider-branded identifiers are only legal in integrations/gingr",
                    )
                )
            provider_path = PROVIDER_DTO_PATH.search(code)
            if provider_path is not None:
                findings.append(
                    Finding(
                        "provider DTO quarantine",
                        relative_text,
                        f"provider path `{normalized_declaration(provider_path.group(0))}` is only legal in integrations/gingr",
                    )
                )
        if relative_text == "apps/api/src/public_contract.rs" and re.search(
            r"\bstorage\s*::", code
        ):
            findings.append(
                Finding(
                    "API semantic ownership",
                    relative_text,
                    "public wire contracts must promote into application-owned semantic types before storage",
                )
            )
        compatibility_identifier = re.search(
            r"\b(?:LegacyBookingStatus|HistoricalAppointmentStatus|LegacyEstimatedSavings|LegacyReportedCompletionEvidence)\b",
            code,
        )
        if (
            not compatibility_boundary(relative)
            and compatibility_identifier is not None
        ):
            findings.append(
                Finding(
                    "compatibility leakage",
                    relative_text,
                    f"active semantic surface contains `{compatibility_identifier.group(0)}`",
                )
            )
        if not compatibility_boundary(
            relative
        ) and COMPATIBILITY_DEPENDENCY_PATH.search(code):
            findings.append(
                Finding(
                    "compatibility dependency isolation",
                    relative_text,
                    "active production code must not depend on a compatibility module",
                )
            )
    return findings


def quoted_string_literals(source: str) -> tuple[str, ...]:
    """Return ordinary and raw Rust string bodies without masking executable text."""
    literals: list[str] = []
    cursor = 0
    while cursor < len(source):
        if source.startswith(("br", "r"), cursor):
            end = raw_string_end(source, cursor)
            if end is not None:
                opening = source.find('"', cursor, end)
                hashes = source[cursor:opening].count("#")
                literals.append(source[opening + 1 : end - hashes - 1])
                cursor = end
                continue
        quote = cursor + 1 if source.startswith('b"', cursor) else cursor
        if quote < len(source) and source[quote] == '"':
            end = quoted_string_end(source, quote)
            literals.append(source[quote + 1 : max(quote + 1, end - 1)])
            cursor = end
            continue
        cursor += 1
    return tuple(literals)


def public_rustdoc_and_literal_findings(repo_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    for path in production_rust_files(repo_root):
        relative = path.relative_to(repo_root)
        if relative.parts[:2] == ("integrations", "gingr"):
            continue
        relative_text = relative.as_posix()
        source = path.read_text(encoding="utf-8")
        if relative.parts[0] in {"app", "domain"}:
            rustdoc = "\n".join(
                line
                for line in source.splitlines()
                if line.lstrip().startswith(("///", "//!"))
            )
            if PROVIDER_BRAND.search(rustdoc):
                findings.append(
                    Finding(
                        "provider vocabulary quarantine",
                        relative_text,
                        "public Rustdoc names provider-specific Gingr vocabulary outside the integration boundary",
                    )
                )
        runtime_fixture_adapter = relative.parts[:4] == (
            "apps",
            "api",
            "src",
            "http",
        ) and path.stem.endswith("fixture")
        if not runtime_fixture_adapter and any(
            PROVIDER_MODEL_PATH_LITERAL.search(literal)
            for literal in quoted_string_literals(source)
        ):
            findings.append(
                Finding(
                    "provider DTO quarantine",
                    relative_text,
                    "executable literal contains a provider DTO/model path outside integrations/gingr",
                )
            )
    return findings


def provider_fixture_quarantine_findings(repo_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    app_root = repo_root / "app" / "src"
    if not app_root.is_dir():
        return findings
    for path in sorted(app_root.rglob("*.rs")):
        source = path.read_text(encoding="utf-8")
        for match in PROVIDER_FIXTURE_EVIDENCE.finditer(source):
            findings.append(
                Finding(
                    "provider fixture quarantine",
                    path.relative_to(repo_root).as_posix(),
                    f"provider-branded fixture evidence `{match.group(0)}` belongs in an integration or runtime adapter",
                )
            )
        for match in APP_SOURCE_PAYLOAD_TYPE.finditer(source):
            findings.append(
                Finding(
                    "provider fixture quarantine",
                    path.relative_to(repo_root).as_posix(),
                    f"public app source-payload contract `{match.group('name')}` belongs in an integration or runtime adapter",
                )
            )
        for match in APP_SOURCE_EVIDENCE_MODEL_PATH.finditer(source):
            findings.append(
                Finding(
                    "provider fixture quarantine",
                    path.relative_to(repo_root).as_posix(),
                    f"public app source-evidence model mapping `{match.group('name')}` belongs in an integration or runtime adapter",
                )
            )
    return findings


def source_without_public_use(code: str) -> str:
    return re.sub(r"\bpub\s+use\s+[^;]*;", "", code, flags=re.MULTILINE)


def rust_source_module_path(repo_root: Path, relative: str) -> str:
    path = Path(relative)
    parts = path.parts
    source_index_position = parts.index("src")
    crate_root = repo_root.joinpath(*parts[:source_index_position])
    manifest_path = crate_root / "Cargo.toml"
    crate_name = parts[source_index_position - 1]
    if manifest_path.is_file():
        manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
        library = manifest.get("lib", {})
        package = manifest.get("package", {})
        if isinstance(library, dict) and isinstance(library.get("name"), str):
            crate_name = library["name"]
        elif isinstance(package, dict) and isinstance(package.get("name"), str):
            crate_name = package["name"]
    crate_name = crate_name.replace("-", "_")
    module_parts = list(parts[source_index_position + 1 :])
    module_parts[-1] = Path(module_parts[-1]).stem
    if module_parts[-1] in {"lib", "main", "mod"}:
        module_parts.pop()
    return "::".join((crate_name, *module_parts))


def canonical_public_source_indexes(
    repo_root: Path, sources: dict[str, str]
) -> dict[str, SourceIndex]:
    indexes: dict[str, SourceIndex] = {}
    crate_names = {
        rust_source_module_path(repo_root, relative).split("::", 1)[0]
        for relative in sources
    }
    for relative, code in sources.items():
        crate_name = rust_source_module_path(repo_root, relative).split("::", 1)[0]
        without_public_reexports = source_without_public_use(code)
        canonical_crate_paths = re.sub(
            r"\bcrate\s*::", f"{crate_name}::", without_public_reexports
        )
        indexed = source_index(canonical_crate_paths)
        module_path = rust_source_module_path(repo_root, relative)
        module_parts = tuple(module_path.split("::"))
        imported_locals = {
            path: set(locals_) for path, locals_ in indexed.imported_locals.items()
        }
        for declaration in top_level_use_declarations(without_public_reexports):
            for use_path, alias in expanded_use_paths(declaration):
                resolved = list(module_parts)
                remaining = list(use_path)
                if remaining[:1] == ["crate"]:
                    resolved = [crate_name]
                    remaining.pop(0)
                elif remaining[:1] == ["self"]:
                    remaining.pop(0)
                else:
                    while remaining[:1] == ["super"]:
                        if len(resolved) > 1:
                            resolved.pop()
                        remaining.pop(0)
                    if remaining and remaining[0] in crate_names:
                        resolved = []
                if not remaining:
                    continue
                canonical = "::".join((*resolved, *remaining))
                imported_locals.setdefault(canonical, set()).add(
                    alias or remaining[-1]
                )
        local_qualified_paths = {
            f"{module_path}::{path}"
            for path in indexed.paths
            if path.split("::", 1)[0] not in crate_names | {"crate", "self", "super"}
        }
        indexes[relative] = SourceIndex(
            indexed.paths | frozenset(local_qualified_paths),
            {
                path: frozenset(locals_)
                for path, locals_ in imported_locals.items()
            },
            indexed.implemented_trait_methods,
            indexed.code,
        )
    return indexes


def canonical_reexport_paths(
    repo_root: Path, sources: dict[str, str]
) -> dict[str, set[str]]:
    reexports: dict[str, set[str]] = {}
    crate_names = {
        rust_source_module_path(repo_root, relative).split("::", 1)[0]
        for relative in sources
    }
    for relative, code in sources.items():
        module_path = rust_source_module_path(repo_root, relative)
        module_parts = tuple(module_path.split("::"))
        crate_name = module_path.split("::", 1)[0]
        for match in re.finditer(r"\bpub\s+use\s+[^;]*;", code, re.DOTALL):
            declaration = match.group(0)
            use_declaration = re.sub(r"^\s*pub\s+", "", declaration, count=1)
            for path, alias in expanded_use_paths(use_declaration):
                if path[:1] == ("crate",):
                    canonical_parts = (crate_name, *path[1:])
                elif path[:1] == ("self",):
                    canonical_parts = (*module_parts, *path[1:])
                elif path[:1] == ("super",):
                    canonical_parts = (*module_parts[:-1], *path[1:])
                elif path[:1] and path[0] in crate_names:
                    canonical_parts = path
                else:
                    canonical_parts = (*module_parts, *path)
                canonical = "::".join(canonical_parts)
                public_name = alias or canonical_parts[-1]
                public_path = f"{module_path}::{public_name}"
                reexports.setdefault(canonical, set()).add(public_path)
    return reexports


def canonical_symbol_is_referenced(
    item_paths: tuple[str, ...],
    kind: str,
    owner: str,
    indexes: dict[str, SourceIndex],
) -> bool:
    return any(
        any(
            symbol_is_referenced(index, ApiItem(item_path, kind, owner))
            for item_path in item_paths
        )
        for index in indexes.values()
    )


def inline_module_names_at(code: str, position: int) -> tuple[str, ...]:
    stack: list[str | None] = []
    token = re.compile(
        r"\b(?:pub(?:\([^)]*\))?\s+)?mod\s+"
        r"(?P<module>[A-Za-z_][A-Za-z0-9_]*)\s*\{|(?P<brace>[{}])"
    )
    for match in token.finditer(code, 0, position):
        module = match.group("module")
        if module is not None:
            stack.append(module)
        elif match.group("brace") == "{":
            stack.append(None)
        elif stack:
            stack.pop()
    return tuple(name for name in stack if name is not None)


def declared_symbol_path(
    repo_root: Path, relative: str, code: str, position: int, name: str
) -> str:
    return "::".join(
        (
            rust_source_module_path(repo_root, relative),
            *inline_module_names_at(code, position),
            name,
        )
    )


def same_source_declared_symbol_is_referenced(
    code: str, declaration: re.Match[str], name: str
) -> bool:
    remainder = code[: declaration.start()] + code[declaration.end() :]
    remainder = re.sub(
        rf"\bimpl(?:\s*<[^>{{}}]*>)?\s+"
        rf"(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*{re.escape(name)}\b",
        "impl ",
        remainder,
    )
    return re.search(rf"\b{re.escape(name)}\b", remainder) is not None


def public_api_inventory_exemptions(
    repo_root: Path,
) -> tuple[set[tuple[str, str]], set[tuple[str, str, str]]]:
    inventory_path = repo_root / "docs" / "quality" / "public-api-inventory.json"
    if not inventory_path.is_file():
        return set(), set()
    # Legacy path/name allowlists remain readable for stale-entry diagnostics,
    # but never exempt a declaration or variant from concrete-consumer checks.
    return set(), set()


def exact_external_contract_entry_is_auditable(
    entry: object, repo_root: Path
) -> bool:
    if not isinstance(entry, dict):
        return False
    rationale = entry.get("rationale")
    evidence = entry.get("evidence")
    if not isinstance(rationale, str) or not rationale.strip() or not isinstance(evidence, dict):
        return False
    if set(evidence) != {"kind", "path", "locator"}:
        return False
    if evidence.get("kind") not in {"runtime_contract", "generated_runtime_registration"}:
        return False
    relative = evidence.get("path")
    locator = evidence.get("locator")
    if not all(isinstance(value, str) and value.strip() for value in (relative, locator)):
        return False
    path = repo_root / str(relative)
    return (
        path.is_file()
        and not any(part in {"tests", "examples"} for part in path.parts)
        and str(locator) in path.read_text(encoding="utf-8", errors="replace")
    )


def exact_external_contract_symbols(repo_root: Path) -> frozenset[str]:
    inventory_path = repo_root / "docs" / "quality" / "public-api-inventory.json"
    if not inventory_path.is_file():
        return frozenset()
    inventory = json.loads(inventory_path.read_text(encoding="utf-8"))
    return frozenset(
        entry["symbol"]
        for entry in inventory.get("external_contract_symbols", [])
        if isinstance(entry, dict)
        and isinstance(entry.get("symbol"), str)
        and exact_external_contract_entry_is_auditable(entry, repo_root)
    )


def consumerless_public_api_findings(repo_root: Path) -> list[Finding]:
    external_contracts, _ = public_api_inventory_exemptions(repo_root)
    exact_contracts = exact_external_contract_symbols(repo_root)
    sources: dict[str, str] = {}
    for path in production_rust_files(repo_root):
        relative = path.relative_to(repo_root).as_posix()
        sources[relative] = code_without_comments_or_literals(
            path.read_text(encoding="utf-8")
        )
    indexes = canonical_public_source_indexes(repo_root, sources)
    reexports = canonical_reexport_paths(repo_root, sources)
    scip_path = repo_root / "index.scip"
    scip = ScipReferenceIndex.load(scip_path) if scip_path.is_file() else None
    inventory_path = repo_root / "docs" / "quality" / "public-api-inventory.json"
    has_complete_symbol_inventory = False
    if inventory_path.is_file():
        inventory = json.loads(inventory_path.read_text(encoding="utf-8"))
        has_complete_symbol_inventory = isinstance(
            inventory.get("external_contract_symbols"), list
        )

    findings: list[Finding] = []
    for relative, code in sources.items():
        for match in PUBLIC_API_DECLARATION.finditer(code):
            name = match.group("name")
            if match.group("kind") == "fn":
                continue
            if not has_complete_symbol_inventory and not HIGH_RISK_PUBLIC_FAMILY.search(name):
                continue
            if (relative, name) in external_contracts:
                continue
            canonical = declared_symbol_path(
                repo_root, relative, code, match.start(), name
            )
            paths = tuple(
                dict.fromkeys((canonical, *sorted(reexports.get(canonical, set()))))
            )
            if any(
                path in exact_contracts
                or any(symbol.startswith(f"{path}::") for symbol in exact_contracts)
                for path in paths
            ):
                continue
            if scip is not None and any(
                inventory_consumers(
                    ApiItem(path, match.group("kind"), relative), indexes, scip
                )
                for path in paths
            ):
                continue
            if not canonical_symbol_is_referenced(
                paths, match.group("kind"), relative, indexes
            ) and not same_source_declared_symbol_is_referenced(code, match, name):
                findings.append(
                    Finding(
                        "consumerless public API",
                        relative,
                        f"`{name}` has no concrete non-test production consumer",
                    )
                )
    return findings


def enum_variant_names(body: str) -> tuple[str, ...]:
    names: list[str] = []
    start = 0
    depths = {"(": 0, "{": 0, "[": 0}
    closing = {")": "(", "}": "{", "]": "["}
    segments: list[str] = []
    for index, character in enumerate(body):
        if character in depths:
            depths[character] += 1
        elif character in closing:
            opener = closing[character]
            depths[opener] = max(0, depths[opener] - 1)
        elif character == "," and not any(depths.values()):
            segments.append(body[start:index])
            start = index + 1
    segments.append(body[start:])
    for segment in segments:
        without_attributes = re.sub(r"^\s*(?:#\s*\[[^]]*\]\s*)*", "", segment)
        match = re.match(r"(?P<name>[A-Z][A-Za-z0-9_]*)\b", without_attributes)
        if match is not None:
            names.append(match.group("name"))
    return tuple(dict.fromkeys(names))


def self_qualified_member_is_referenced(
    code: str, owner: str, member: str
) -> bool:
    implementation = re.compile(
        rf"\bimpl(?:\s*<[^{{}}]*>)?\s+{re.escape(owner)}\b[^{{}}]*\{{",
        re.DOTALL,
    )
    for match in implementation.finditer(code):
        opening = code.rfind("{", match.start(), match.end())
        closing_index = matching_delimiter(code, opening, "{", "}")
        if re.search(
            rf"\bSelf\s*::\s*{re.escape(member)}\b",
            code[opening + 1 : closing_index - 1],
        ):
            return True
    return False


def dormant_public_variant_findings(repo_root: Path) -> list[Finding]:
    _, externally_consumed_variants = public_api_inventory_exemptions(repo_root)
    exact_contracts = exact_external_contract_symbols(repo_root)
    sources = {
        path.relative_to(repo_root).as_posix(): code_without_comments_or_literals(
            path.read_text(encoding="utf-8")
        )
        for path in production_rust_files(repo_root)
    }
    indexes = canonical_public_source_indexes(repo_root, sources)
    reexports = canonical_reexport_paths(repo_root, sources)
    findings: list[Finding] = []
    for relative, code in sources.items():
        for match in PUBLIC_ACTION_ENUM.finditer(code):
            opening = code.find("{", match.start(), match.end())
            closing = matching_delimiter(code, opening, "{", "}")
            if closing > len(code):
                continue
            enum_name = match.group("name")
            canonical_owner = declared_symbol_path(
                repo_root, relative, code, match.start(), enum_name
            )
            owner_paths = tuple(
                dict.fromkeys(
                    (canonical_owner, *sorted(reexports.get(canonical_owner, set())))
                )
            )
            for variant in enum_variant_names(code[opening + 1 : closing - 1]):
                if (relative, enum_name, variant) in externally_consumed_variants:
                    continue
                variant_paths = tuple(
                    f"{owner_path}::{variant}" for owner_path in owner_paths
                )
                if any(path in exact_contracts for path in variant_paths):
                    continue
                if not canonical_symbol_is_referenced(
                    variant_paths, "variant", relative, indexes
                ) and not self_qualified_member_is_referenced(
                    code, enum_name, variant
                ):
                    findings.append(
                        Finding(
                            "dormant public enum variant",
                            relative,
                            f"`{enum_name}::{variant}` has no production construction or match arm",
                        )
                    )
    return findings


def public_api_inventory_integrity_findings(repo_root: Path) -> list[Finding]:
    inventory_path = repo_root / "docs" / "quality" / "public-api-inventory.json"
    if not inventory_path.is_file():
        return []
    inventory = json.loads(inventory_path.read_text(encoding="utf-8"))
    sources = {
        path.relative_to(repo_root).as_posix(): code_without_comments_or_literals(
            path.read_text(encoding="utf-8")
        )
        for path in production_rust_files(repo_root)
    }
    findings: list[Finding] = []
    for entry in inventory.get("external_contract_symbols", []):
        if exact_external_contract_entry_is_auditable(entry, repo_root):
            continue
        symbol = entry.get("symbol") if isinstance(entry, dict) else None
        findings.append(
            Finding(
                "public API inventory",
                "docs/quality/public-api-inventory.json",
                f"`{symbol}` has unauditable external-contract evidence; declarations, Rustdoc, tests, examples, and hypothetical adapters are not consumers",
            )
        )
    for entry in inventory.get("externally_consumed_contracts", []):
        path = entry.get("path")
        name = entry.get("name")
        rationale = entry.get("rationale")
        if not isinstance(rationale, str) or not rationale.strip():
            findings.append(
                Finding("public API inventory", str(path), f"`{name}` has no rationale")
            )
            continue
        code = sources.get(path, "")
        if not re.search(
            rf"\bpub\s+(?:unsafe\s+)?(?:struct|enum|trait|type|fn|const)\s+"
            rf"{re.escape(str(name))}\b",
            code,
        ):
            findings.append(
                Finding(
                    "public API inventory", str(path), f"stale public contract `{name}`"
                )
            )
    for entry in inventory.get("externally_consumed_variants", []):
        path = entry.get("path")
        enum_name = entry.get("enum")
        variant = entry.get("variant")
        rationale = entry.get("rationale")
        if not isinstance(rationale, str) or not rationale.strip():
            findings.append(
                Finding(
                    "public API inventory",
                    str(path),
                    f"`{enum_name}::{variant}` has no rationale",
                )
            )
            continue
        code = sources.get(path, "")
        enum_match = re.search(
            rf"\bpub\s+enum\s+{re.escape(str(enum_name))}\s*\{{", code
        )
        if enum_match is None:
            exists = False
        else:
            opening = code.find("{", enum_match.start(), enum_match.end())
            closing = matching_delimiter(code, opening, "{", "}")
            exists = str(variant) in enum_variant_names(code[opening + 1 : closing - 1])
        if not exists:
            findings.append(
                Finding(
                    "public API inventory",
                    str(path),
                    f"stale public variant `{enum_name}::{variant}`",
                )
            )
    return findings


def untyped_claim_normalization_findings(repo_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    for path in production_rust_files(repo_root):
        relative = path.relative_to(repo_root).as_posix()
        code = code_without_comments_or_literals(path.read_text(encoding="utf-8"))
        has_untyped_wire_field = re.search(
            r"\bstruct\s+[A-Za-z_][A-Za-z0-9_]*\s*\{[^}]*:\s*(?:serde_json\s*::\s*)?Value\b",
            code,
            re.DOTALL,
        )
        has_custom_deserializer = re.search(
            r"\bimpl\s*(?:<[^>]*>)?\s*Deserialize(?:\s*<[^>]*>)?\s+for\s+",
            code,
        )
        if has_untyped_wire_field is not None and has_custom_deserializer is not None:
            findings.append(
                Finding(
                    "untyped serialized claim normalization",
                    relative,
                    "custom deserialization discards an untyped wire claim instead of admitting one canonical model",
                )
            )
    return findings


def typed_claim_normalization_findings(repo_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    for path in production_rust_files(repo_root):
        code = code_without_comments_or_literals(path.read_text(encoding="utf-8"))
        if not re.search(
            r"\bimpl\s*(?:<[^>]*>)?\s*Deserialize(?:\s*<[^>]*>)?\s+for\s+",
            code,
        ):
            continue
        discarded_fields = re.findall(
            r"\blet\s+_[A-Za-z0-9_]*\s*=\s*[A-Za-z_][A-Za-z0-9_]*\.([A-Za-z_][A-Za-z0-9_]*)\s*;",
            code,
        )
        discarded_fields.extend(
            re.findall(
                r"\blet\s+_\s*=\s*[A-Za-z_][A-Za-z0-9_]*\.([A-Za-z_][A-Za-z0-9_]*)\s*;",
                code,
            )
        )
        discarded_fields.extend(
            field
            for destructure in re.finditer(
                r"\blet\s+[A-Za-z_][A-Za-z0-9_]*\s*\{(?P<body>[^}]*)\}\s*=",
                code,
                re.DOTALL,
            )
            for field in re.findall(
                r"\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*_\b",
                destructure.group("body"),
            )
        )
        discarded_fields.extend(
            re.findall(
                r"#\s*\[\s*serde\s*\(\s*rename\s*=\s*[^)]*\)\s*\]\s*_[A-Za-z_][A-Za-z0-9_]*\s*:",
                path.read_text(encoding="utf-8"),
            )
        )
        if discarded_fields:
            findings.append(
                Finding(
                    "typed serialized claim normalization",
                    path.relative_to(repo_root).as_posix(),
                    "custom deserialization discards typed wire fields: "
                    + ", ".join(sorted(set(discarded_fields))),
                )
            )
    return findings


def explicitly_inert_public_surface_findings(repo_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    documented_variant = re.compile(
        r"(?P<docs>(?:\s*///[^\n]*\n)+)\s*(?P<variant>[A-Z][A-Za-z0-9_]*)",
        re.MULTILINE,
    )
    ignored_method = re.compile(
        r"\bpub\s+fn\s+(?P<method>[A-Za-z_][A-Za-z0-9_]*)"
        r"\s*\([^)]*_[A-Za-z_][A-Za-z0-9_]*[^)]*\)[^{]*\{\s*self\s*\}",
        re.DOTALL,
    )
    non_emittable = re.compile(
        r"\b(?:inert|never\s+(?:emits?|emitted|constructs?|constructed)|"
        r"does\s+not\s+(?:emit|construct)|not\s+(?:emitted|constructed|reachable))\b",
        re.IGNORECASE,
    )
    for path in production_rust_files(repo_root):
        relative = path.relative_to(repo_root).as_posix()
        source = path.read_text(encoding="utf-8")
        for enum_match in re.finditer(
            r"\bpub\s+enum\s+(?P<enum>[A-Za-z_][A-Za-z0-9_]*)\s*\{(?P<body>.*?)\n\}",
            source,
            re.DOTALL,
        ):
            for variant_match in documented_variant.finditer(enum_match.group("body")):
                if non_emittable.search(variant_match.group("docs")) is None:
                    continue
                findings.append(
                    Finding(
                        "explicitly inert public variant",
                        relative,
                        f"`{enum_match.group('enum')}::{variant_match.group('variant')}` is documented as non-emittable",
                    )
                )
        for match in ignored_method.finditer(source):
            findings.append(
                Finding(
                    "ignored public method",
                    relative,
                    f"`{match.group('method')}` accepts and discards a public argument",
                )
            )
    return findings


def active_contract_findings(repo_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    router = repo_root / "apps" / "api" / "src" / "http" / "router.rs"
    if router.is_file():
        source = router.read_text(encoding="utf-8")
        for match in re.finditer(
            r"\.route\s*\(\s*\"(?P<path>/(?!v1/)[^\"]*)\"", source
        ):
            findings.append(
                Finding(
                    "canonical API route inventory",
                    "apps/api/src/http/router.rs",
                    f"unversioned executable product route `{match.group('path')}`",
                )
            )

    prohibited_doc_phrases = (
        "intentionally tolerated legacy states",
        "compatibility-only decoding",
        "legacy compatibility",
    )
    docs_root = repo_root / "docs"
    active_docs: list[Path] = []
    if docs_root.is_dir():
        active_docs = [
            path
            for path in sorted(docs_root.rglob("*.md"))
            if "archive" not in path.relative_to(docs_root).parts
            and ARCHIVED_DOC_MARKER not in path.read_text(encoding="utf-8")
        ]
        for path in active_docs:
            text = path.read_text(encoding="utf-8").lower()
            for phrase in prohibited_doc_phrases:
                if phrase in text:
                    findings.append(
                        Finding(
                            "active architecture compatibility contract",
                            path.relative_to(repo_root).as_posix(),
                            f"active architecture text preserves migration contract `{phrase}`",
                        )
                    )
    if docs_root.is_dir():
        for path in active_docs:
            relative = path.relative_to(repo_root)
            if relative.name == "generated-public-api-inventory.md":
                continue
            text = path.read_text(encoding="utf-8")
            for prohibited in (
                "domain/src/strategic_ai_ops_bridge.rs",
                "strategic_ai_ops_bridge",
                "domain::strategic_ai_ops",
                "domain::service",
                "domain/tests/service_module_architecture.rs",
            ):
                if prohibited in text:
                    findings.append(
                        Finding(
                            "active documentation retired contract",
                            relative.as_posix(),
                            f"active documentation presents deleted/nonexistent contract `{prohibited}`",
                        )
                    )
            for match in re.finditer(
                r"\bOpenAPI\s+(?:version\s+)?v?0\b", text, re.IGNORECASE
            ):
                findings.append(
                    Finding(
                        "active documentation route version",
                        relative.as_posix(),
                        f"stale active-contract prose `{match.group(0)}`",
                    )
                )
            if re.search(r"(?<![A-Za-z0-9_])/v0(?:/|\b)", text):
                findings.append(
                    Finding(
                        "active documentation route version",
                        relative.as_posix(),
                        "active documentation declares noncanonical `/v0` product routes",
                    )
                )
            relative_text = relative.as_posix()
            markdown_links_are_current_contracts = relative_text.startswith(
                ("docs/architecture/", "docs/demo/")
            )
            for target in (
                re.findall(r"\]\(([^)\s]+)(?:\s+[^)]*)?\)", text)
                if markdown_links_are_current_contracts
                else ()
            ):
                if target.startswith(("#", "/", "http://", "https://", "mailto:")):
                    continue
                path_target = target.split("#", 1)[0].split("?", 1)[0]
                if not path_target:
                    continue
                resolved = (path.parent / path_target).resolve()
                try:
                    repository_target = resolved.relative_to(repo_root.resolve())
                except ValueError:
                    continue
                if not resolved.exists():
                    findings.append(
                        Finding(
                            "active documentation declared path",
                            relative.as_posix(),
                            f"declared repository path `{repository_target.as_posix()}` does not exist",
                        )
                    )
            for declared in re.findall(r"`([^`\n]+)`", text):
                # Free-form design, workflow, and historical planning documents may
                # name proposed paths. Only architecture documents assert every
                # inline-code path as a current repository contract. Explicitly
                # retired owners above remain checked across all active documentation.
                if not relative_text.startswith("docs/architecture/"):
                    continue
                candidate = declared.rstrip(".,;:")
                if (
                    "/" not in candidate
                    or " " in candidate
                    or "::" in candidate
                    or candidate.startswith(("/", "http://", "https://"))
                    or any(character in candidate for character in "*{}[]()")
                ):
                    continue
                root = candidate.split("/", 1)[0]
                if root not in {
                    "app",
                    "apps",
                    "domain",
                    "docs",
                    "integrations",
                    "scripts",
                    "storage",
                }:
                    continue
                if not (repo_root / candidate).exists():
                    findings.append(
                        Finding(
                            "active documentation declared path",
                            relative.as_posix(),
                            f"declared repository path `{candidate}` does not exist",
                        )
                    )
    return findings


def normalized_declaration(declaration: str) -> str:
    return " ".join(declaration.split())


def measured_source_debt(
    repo_root: Path,
) -> tuple[
    dict[str, int],
    dict[str, int],
    dict[str, set[str]],
    dict[str, set[str]],
]:
    line_counts: dict[str, int] = {}
    compatibility_counts: dict[str, int] = {}
    broad_reexports: dict[str, set[str]] = {}
    public_reexport_aliases: dict[str, set[str]] = {}

    for path in production_rust_files(repo_root):
        relative = path.relative_to(repo_root)
        relative_text = relative.as_posix()
        source = path.read_text(encoding="utf-8")
        code = code_without_comments_or_literals(source)

        line_counts[relative_text] = physical_line_count(source)
        compatibility_count = len(COMPATIBILITY_IDENTIFIER.findall(code))
        if compatibility_boundary(relative):
            compatibility_count += 1
        if compatibility_count:
            compatibility_counts[relative_text] = compatibility_count
        broad_declarations = {
            normalized_declaration(match.group(0))
            for match in BROAD_REEXPORT.finditer(code)
        }
        if broad_declarations:
            broad_reexports[relative_text] = broad_declarations
        alias_declarations = {
            normalized_declaration(match.group(0))
            for match in PUBLIC_REEXPORT_ALIAS.finditer(code)
        }
        if alias_declarations:
            public_reexport_aliases[relative_text] = alias_declarations

    return line_counts, compatibility_counts, broad_reexports, public_reexport_aliases


def require_object(value: object, location: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ValueError(f"{location} must be an object")
    return value


def require_nonnegative_int(value: object, location: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise ValueError(f"{location} must be a non-negative integer")
    return value


def debt_map(section: dict[str, Any], location: str) -> dict[str, int]:
    raw = require_object(
        section.get("debt_ceiling_by_path"), f"{location}.debt_ceiling_by_path"
    )
    result: dict[str, int] = {}
    for path, value in raw.items():
        if not isinstance(path, str) or not path or Path(path).is_absolute():
            raise ValueError(
                f"{location}.debt_ceiling_by_path keys must be relative paths"
            )
        result[path] = require_nonnegative_int(
            value,
            f"{location}.debt_ceiling_by_path[{path!r}]",
        )
    return result


def public_surface_allowlist(
    section: dict[str, Any], location: str
) -> dict[str, set[str]]:
    raw = section.get("allowlist")
    if not isinstance(raw, list):
        raise ValueError(f"{location}.allowlist must be an array")

    result: dict[str, set[str]] = {}
    seen: set[tuple[str, str]] = set()
    for index, raw_entry in enumerate(raw):
        entry_location = f"{location}.allowlist[{index}]"
        entry = require_object(raw_entry, entry_location)
        path = entry.get("path")
        declaration = entry.get("declaration")
        rationale = entry.get("rationale")
        if not isinstance(path, str) or not path or Path(path).is_absolute():
            raise ValueError(f"{entry_location}.path must be a relative path")
        if not isinstance(declaration, str) or not declaration.strip():
            raise ValueError(f"{entry_location}.declaration must be a non-empty string")
        if not isinstance(rationale, str) or not rationale.strip():
            raise ValueError(f"{entry_location}.rationale must be a non-empty string")
        normalized = normalized_declaration(declaration)
        key = (path, normalized)
        if key in seen:
            raise ValueError(f"{location}.allowlist contains a duplicate declaration")
        seen.add(key)
        result.setdefault(path, set()).add(normalized)
    return result


def public_surface_findings(
    *,
    category: str,
    measured: dict[str, set[str]],
    allowed: dict[str, set[str]],
) -> list[Finding]:
    findings: list[Finding] = []
    for path in sorted(set(measured) | set(allowed)):
        actual = measured.get(path, set())
        expected = allowed.get(path, set())
        for declaration in sorted(actual - expected):
            findings.append(
                Finding(
                    category,
                    path,
                    f"1 > allowed 0; unapproved declaration `{declaration}`",
                )
            )
        for declaration in sorted(expected - actual):
            findings.append(
                Finding(
                    f"{category} stale allowlist",
                    path,
                    f"remove missing declaration `{declaration}` from the checked inventory",
                )
            )
    return findings


def compare_exact_debt(
    *,
    category: str,
    measured: dict[str, int],
    completion_maximum: int,
    ceilings: dict[str, int],
) -> list[Finding]:
    findings: list[Finding] = []
    for path in sorted(set(measured) | set(ceilings)):
        actual = measured.get(path, 0)
        recorded = ceilings.get(path)
        allowed = recorded if recorded is not None else completion_maximum
        if actual > allowed:
            findings.append(Finding(category, path, f"{actual} > allowed {allowed}"))
            continue
        if recorded is not None and actual != recorded:
            findings.append(
                Finding(
                    f"{category} stale debt ceiling",
                    path,
                    f"measured {actual}; ratchet baseline to {actual}",
                )
            )
    return findings


def load_baseline(path: Path) -> dict[str, Any]:
    try:
        baseline = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ValueError("baseline file is missing") from exc
    except json.JSONDecodeError as exc:
        raise ValueError(f"baseline is invalid JSON: {exc}") from exc

    baseline = require_object(baseline, "baseline")
    if baseline.get("schema_version") != 1:
        raise ValueError("baseline.schema_version must be 1")
    if not isinstance(baseline.get("source"), str) or not baseline["source"].strip():
        raise ValueError("baseline.source must be a non-empty string")
    return baseline


def documented_inventory(
    section: dict[str, Any],
    key: str,
    fields: tuple[str, ...],
) -> set[tuple[str, ...]]:
    raw_entries = section.get(key)
    if not isinstance(raw_entries, list):
        raise ValueError(f"authority_capabilities.{key} must be an array")
    result: set[tuple[str, ...]] = set()
    for index, raw_entry in enumerate(raw_entries):
        location = f"authority_capabilities.{key}[{index}]"
        entry = require_object(raw_entry, location)
        values: list[str] = []
        for field in fields:
            value = entry.get(field)
            if not isinstance(value, str) or not value.strip():
                raise ValueError(f"{location}.{field} must be a non-empty string")
            values.append(value)
        rationale = entry.get("rationale")
        if not isinstance(rationale, str) or not rationale.strip():
            raise ValueError(f"{location}.rationale must be a non-empty string")
        identity = tuple(values)
        if identity in result:
            raise ValueError(f"authority_capabilities.{key} contains a duplicate entry")
        result.add(identity)
    return result


def matching_delimiter(code: str, opening: int, opener: str, closer: str) -> int:
    depth = 1
    cursor = opening + 1
    while cursor < len(code) and depth:
        if code[cursor] == opener:
            depth += 1
        elif code[cursor] == closer:
            depth -= 1
        cursor += 1
    return cursor


def balanced_delimiter_end(code: str, opening: int) -> int | None:
    """Return the offset after one balanced Rust signature delimiter."""
    closing_for = {"(": ")", "[": "]", "{": "}", "<": ">"}
    opener = code[opening] if opening < len(code) else ""
    closing = closing_for.get(opener)
    if closing is None:
        return None

    stack = [closing]
    cursor = opening + 1
    while cursor < len(code):
        character = code[cursor]
        if character in closing_for:
            stack.append(closing_for[character])
        elif character == stack[-1]:
            if character == ">" and cursor > 0 and code[cursor - 1] in "-=":
                cursor += 1
                continue
            stack.pop()
            if not stack:
                return cursor + 1
        cursor += 1
    return None


def function_signatures(code: str) -> list[tuple[str, str]]:
    """Extract function names and return types using balanced Rust syntax."""
    signatures: list[tuple[str, str]] = []
    for match in re.finditer(
        r"\bfn\s+(?P<function>[A-Za-z_][A-Za-z0-9_]*)\b",
        code,
    ):
        cursor = match.end()
        while cursor < len(code) and code[cursor].isspace():
            cursor += 1
        if cursor < len(code) and code[cursor] == "<":
            generic_end = balanced_delimiter_end(code, cursor)
            if generic_end is None:
                continue
            cursor = generic_end
            while cursor < len(code) and code[cursor].isspace():
                cursor += 1
        if cursor >= len(code) or code[cursor] != "(":
            continue
        parameters_end = balanced_delimiter_end(code, cursor)
        if parameters_end is None:
            continue
        cursor = parameters_end
        while cursor < len(code) and code[cursor].isspace():
            cursor += 1
        if not code.startswith("->", cursor):
            continue

        cursor += 2
        return_start = cursor
        stack: list[str] = []
        closing_for = {"(": ")", "[": "]", "{": "}", "<": ">"}
        while cursor < len(code):
            character = code[cursor]
            if not stack and character in "{;":
                break
            if (
                not stack
                and code.startswith("where", cursor)
                and (
                    cursor == return_start
                    or not (code[cursor - 1].isalnum() or code[cursor - 1] == "_")
                )
                and (
                    cursor + len("where") >= len(code)
                    or not (
                        code[cursor + len("where")].isalnum()
                        or code[cursor + len("where")] == "_"
                    )
                )
            ):
                break
            if character in closing_for:
                stack.append(closing_for[character])
            elif stack and character == stack[-1]:
                if character != ">" or cursor == 0 or code[cursor - 1] not in "-=":
                    stack.pop()
            cursor += 1

        return_type = code[return_start:cursor].strip()
        if return_type:
            signatures.append((match.group("function"), return_type))
    return signatures


def local_type_aliases(code: str) -> dict[str, tuple[str, ...]]:
    """Extract local Rust type-alias targets without interpreting their bodies."""
    aliases: dict[str, list[str]] = {}
    for match in re.finditer(
        r"\btype\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\b",
        code,
    ):
        cursor = match.end()
        while cursor < len(code) and code[cursor].isspace():
            cursor += 1
        if cursor < len(code) and code[cursor] == "<":
            generic_end = balanced_delimiter_end(code, cursor)
            if generic_end is None:
                continue
            cursor = generic_end

        stack: list[str] = []
        closing_for = {"(": ")", "[": "]", "{": "}", "<": ">"}
        while cursor < len(code):
            character = code[cursor]
            if not stack and character == ";":
                break
            if not stack and character == "=":
                break
            if character in closing_for:
                stack.append(closing_for[character])
            elif (
                stack
                and character == stack[-1]
                and (character != ">" or cursor == 0 or code[cursor - 1] not in "-=")
            ):
                stack.pop()
            cursor += 1
        if cursor >= len(code) or code[cursor] != "=":
            continue

        cursor += 1
        target_start = cursor
        stack = []
        while cursor < len(code):
            character = code[cursor]
            if not stack and character == ";":
                break
            if character in closing_for:
                stack.append(closing_for[character])
            elif (
                stack
                and character == stack[-1]
                and (character != ">" or cursor == 0 or code[cursor - 1] not in "-=")
            ):
                stack.pop()
            cursor += 1
        if cursor >= len(code):
            continue
        target = code[target_start:cursor].strip()
        if target:
            aliases.setdefault(match.group("name"), []).append(target)
    return {name: tuple(targets) for name, targets in aliases.items()}


@dataclass(frozen=True, order=True)
class RustModuleLocation:
    source_root: str
    module: tuple[str, ...]


@dataclass(frozen=True)
class RustSourceScope:
    location: RustModuleLocation
    code: str


def rust_module_location(relative: str) -> RustModuleLocation | None:
    """Map a conventional Cargo source path to its crate-local module path."""
    parts = Path(relative).parts
    try:
        source_index = parts.index("src")
    except ValueError:
        return None
    source_root = Path(*parts[: source_index + 1]).as_posix()
    source_parts = parts[source_index + 1 :]
    if not source_parts:
        return None
    filename = source_parts[-1]
    if not filename.endswith(".rs"):
        return None
    stem = filename[:-3]
    parents = source_parts[:-1]
    module = parents if stem in {"lib", "main", "mod"} else parents + (stem,)
    return RustModuleLocation(source_root, tuple(module))


def rust_source_scopes(
    code: str, location: RustModuleLocation
) -> tuple[RustSourceScope, ...]:
    """Split one Rust file into its file module and nested inline-module scopes."""
    scopes: list[RustSourceScope] = []

    def visit(scope_code: str, scope_location: RustModuleLocation) -> None:
        children: list[tuple[int, int, str, str]] = []
        depth = 0
        consumed = 0
        for match in re.finditer(
            r"\bmod\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\{", scope_code
        ):
            for character in scope_code[consumed : match.start()]:
                if character == "{":
                    depth += 1
                elif character == "}" and depth:
                    depth -= 1
            consumed = match.start()
            if depth:
                continue
            opening = scope_code.find("{", match.start(), match.end())
            closing = matching_delimiter(scope_code, opening, "{", "}")
            if closing > len(scope_code) or scope_code[closing - 1 : closing] != "}":
                continue
            children.append(
                (
                    opening,
                    closing,
                    match.group("name"),
                    scope_code[opening + 1 : closing - 1],
                )
            )

        masked = list(scope_code)
        for opening, closing, _name, _body in children:
            for index in range(opening + 1, closing - 1):
                if masked[index] != "\n":
                    masked[index] = " "
        scopes.append(RustSourceScope(scope_location, "".join(masked)))
        for _opening, _closing, name, body in children:
            visit(
                body,
                RustModuleLocation(
                    scope_location.source_root, scope_location.module + (name,)
                ),
            )

    visit(code, location)
    return tuple(scopes)


def local_alias_imports(code: str) -> dict[str, tuple[tuple[str, ...], ...]]:
    """Index local names and glob sources introduced by top-level Rust uses."""
    imports: dict[str, list[tuple[str, ...]]] = {}
    for declaration in top_level_use_declarations(code):
        for use_path, alias in expanded_use_paths(declaration):
            if not use_path:
                continue
            local_name = "*" if use_path[-1] == "*" else alias or use_path[-1]
            imports.setdefault(local_name, []).append(use_path)
    return {name: tuple(paths) for name, paths in imports.items()}


def authority_return_resolution(
    return_type: str,
    location: RustModuleLocation,
    aliases: dict[tuple[str, tuple[str, ...], str], tuple[str, ...]],
    imports_by_module: dict[
        tuple[str, tuple[str, ...]], dict[str, tuple[tuple[str, ...], ...]]
    ],
    authority_names: set[str],
) -> tuple[set[str], set[str]]:
    """Resolve crate-local alias returns to authority owners and fail closed."""
    resolved: set[str] = set()
    hazards: set[str] = set()

    def is_authority_shaped(identifier: str) -> bool:
        return re.fullmatch(AUTHORITY_TYPE_NAME, identifier) is not None

    def canonical_alias_candidates(
        path: tuple[str, ...],
        context: RustModuleLocation,
    ) -> set[tuple[str, tuple[str, ...], str]]:
        if not path:
            return set()
        alias_name = path[-1]
        prefix = path[:-1]
        module_candidates: set[tuple[str, ...]] = set()
        if not prefix:
            module_candidates.add(context.module)
        elif prefix[0] == "crate":
            module_candidates.add(prefix[1:])
        elif prefix[0] == "self":
            module_candidates.add(context.module + prefix[1:])
        elif prefix[0] == "super":
            remaining = list(prefix)
            parent = list(context.module)
            while remaining and remaining[0] == "super":
                if parent:
                    parent.pop()
                remaining.pop(0)
            module_candidates.add(tuple(parent) + tuple(remaining))
        else:
            module_candidates.add(context.module + prefix)
            module_candidates.add(prefix)
        return {
            (context.source_root, module, alias_name)
            for module in module_candidates
            if (context.source_root, module, alias_name) in aliases
        }

    def alias_candidates(
        path: tuple[str, ...],
        context: RustModuleLocation,
    ) -> set[tuple[str, tuple[str, ...], str]]:
        candidates = canonical_alias_candidates(path, context)
        module_key = (context.source_root, context.module)
        local_imports = imports_by_module.get(module_key, {})
        for imported_path in local_imports.get(path[0], ()):
            candidates.update(
                canonical_alias_candidates(imported_path + path[1:], context)
            )
        for glob_path in local_imports.get("*", ()):
            candidates.update(
                canonical_alias_candidates(glob_path[:-1] + path, context)
            )
        return candidates

    def alias_label(key: tuple[str, tuple[str, ...], str]) -> str:
        _source_root, module, name = key
        return "::".join((*module, name))

    def resolve_alias(
        key: tuple[str, tuple[str, ...], str],
        stack: tuple[tuple[str, tuple[str, ...], str], ...],
    ) -> tuple[set[str], bool]:
        name = key[2]
        if key in stack:
            cycle = stack[stack.index(key) :] + (key,)
            shaped = any(is_authority_shaped(part[2]) for part in cycle)
            if shaped:
                hazards.add(
                    "cyclic alias " + " -> ".join(alias_label(part) for part in cycle)
                )
            return set(), shaped

        targets = aliases.get(key, ())
        if not targets:
            return set(), is_authority_shaped(name)

        alias_authorities: set[str] = set()
        shaped = is_authority_shaped(name)
        alias_context = RustModuleLocation(key[0], key[1])
        for target in targets:
            target_authorities, target_shaped = resolve_type(
                target,
                alias_context,
                stack + (key,),
            )
            alias_authorities.update(target_authorities)
            shaped = shaped or target_shaped
        if len(targets) > 1 and shaped and not alias_authorities:
            hazards.add(f"ambiguous alias `{alias_label(key)}`")
        if shaped and not alias_authorities and key not in stack:
            hazards.add(f"unresolved authority-shaped alias `{alias_label(key)}`")
        return alias_authorities, shaped

    def resolve_type(
        type_expression: str,
        context: RustModuleLocation,
        stack: tuple[tuple[str, tuple[str, ...], str], ...],
    ) -> tuple[set[str], bool]:
        type_authorities: set[str] = set()
        shaped = False
        type_paths = re.findall(
            r"\b[A-Za-z_][A-Za-z0-9_]*(?:\s*::\s*[A-Za-z_][A-Za-z0-9_]*)*",
            type_expression,
        )
        for type_path in type_paths:
            path = tuple(re.findall(r"[A-Za-z_][A-Za-z0-9_]*", type_path))
            identifier = path[-1]
            if identifier in authority_names:
                type_authorities.add(identifier)
                shaped = True
                continue
            candidates = alias_candidates(path, context)
            if not candidates:
                shaped = shaped or is_authority_shaped(identifier)
                continue
            candidate_results = [
                resolve_alias(candidate, stack) for candidate in sorted(candidates)
            ]
            for alias_authorities, alias_shaped in candidate_results:
                type_authorities.update(alias_authorities)
                shaped = shaped or alias_shaped
            if len(candidates) > 1 and shaped and not type_authorities:
                hazards.add(f"ambiguous alias `{identifier}`")
        return type_authorities, shaped

    resolved, _shaped = resolve_type(return_type, location, ())
    return resolved, hazards


def authority_structs(code: str) -> list[tuple[str, str, str | None, str, str]]:
    structs: list[tuple[str, str, str | None, str, str]] = []
    for match in AUTHORITY_STRUCT_HEADER.finditer(code):
        cursor = match.end()
        while cursor < len(code) and code[cursor].isspace():
            cursor += 1
        if cursor < len(code) and code[cursor] == "<":
            cursor = matching_delimiter(code, cursor, "<", ">")
            while cursor < len(code) and code[cursor].isspace():
                cursor += 1
        if cursor >= len(code):
            continue
        if code[cursor] == "(":
            end = matching_delimiter(code, cursor, "(", ")")
            if end <= len(code) and code[end - 1 : end] == ")":
                structs.append(
                    (
                        match.group("name"),
                        match.group("attributes"),
                        match.group("visibility"),
                        "tuple",
                        code[cursor + 1 : end - 1],
                    )
                )
            continue

        angle_depth = 0
        parenthesis_depth = 0
        bracket_depth = 0
        while cursor < len(code):
            character = code[cursor]
            if character == "<":
                angle_depth += 1
            elif character == ">" and angle_depth:
                angle_depth -= 1
            elif character == "(":
                parenthesis_depth += 1
            elif character == ")" and parenthesis_depth:
                parenthesis_depth -= 1
            elif character == "[":
                bracket_depth += 1
            elif character == "]" and bracket_depth:
                bracket_depth -= 1
            elif (
                not (angle_depth or parenthesis_depth or bracket_depth)
                and character in "{;"
            ):
                break
            cursor += 1
        if cursor >= len(code):
            continue
        if code[cursor] == "{":
            end = matching_delimiter(code, cursor, "{", "}")
            if end <= len(code) and code[end - 1 : end] == "}":
                structs.append(
                    (
                        match.group("name"),
                        match.group("attributes"),
                        match.group("visibility"),
                        "brace",
                        code[cursor + 1 : end - 1],
                    )
                )
        elif code[cursor] == ";":
            structs.append(
                (
                    match.group("name"),
                    match.group("attributes"),
                    match.group("visibility"),
                    "unit",
                    "",
                )
            )
    return structs


def inherent_authority_impls(code: str) -> list[tuple[str, str]]:
    blocks: list[tuple[str, str]] = []
    for match in re.finditer(r"\bimpl\b", code):
        cursor = match.end()
        while cursor < len(code) and code[cursor].isspace():
            cursor += 1
        if cursor < len(code) and code[cursor] == "<":
            cursor = matching_delimiter(code, cursor, "<", ">")
            while cursor < len(code) and code[cursor].isspace():
                cursor += 1
        owner = re.match(
            rf"(?:(?:crate|self|super|[A-Za-z_][A-Za-z0-9_]*)\s*::\s*)*"
            rf"(?P<name>{AUTHORITY_TYPE_NAME})\b",
            code[cursor:],
        )
        if owner is None:
            continue
        name = owner.group("name")
        cursor += owner.end()
        while cursor < len(code) and code[cursor].isspace():
            cursor += 1
        if cursor < len(code) and code[cursor] == "<":
            cursor = matching_delimiter(code, cursor, "<", ">")
        opening = code.find("{", cursor)
        if opening < 0 or re.search(r"\bfor\b", code[cursor:opening]):
            continue
        closing = matching_delimiter(code, opening, "{", "}")
        if closing <= len(code) and code[closing - 1 : closing] == "}":
            blocks.append((name, code[opening + 1 : closing - 1]))
    return blocks


def authority_capability_findings(
    repo_root: Path,
    section: dict[str, Any],
) -> tuple[list[Finding], int, int]:
    policy_only_types = {
        (path, name, trait)
        for path, name, trait in documented_inventory(
            {
                **section,
                "policy_only_types": section.get("policy_only_types", []),
            },
            "policy_only_types",
            ("path", "name", "trait"),
        )
    }
    allowed_types = documented_inventory(section, "types", ("path", "name"))
    allowed_issuers = documented_inventory(
        section,
        "issuers",
        ("path", "function", "authority"),
    )
    measured_types: set[tuple[str, str]] = set()
    issuer_candidates: set[tuple[str, str, str]] = set()
    encountered_policy_only_types: set[tuple[str, str, str]] = set()
    findings: list[Finding] = []

    scopes_by_path: dict[str, tuple[RustSourceScope, ...]] = {}
    imports_by_module: dict[
        tuple[str, tuple[str, ...]], dict[str, tuple[tuple[str, ...], ...]]
    ] = {}
    aliases: dict[tuple[str, tuple[str, ...], str], tuple[str, ...]] = {}
    for path in production_rust_files(repo_root):
        relative = path.relative_to(repo_root).as_posix()
        code = code_without_comments_or_literals(path.read_text(encoding="utf-8"))
        location = rust_module_location(relative)
        if location is None:
            continue
        scopes = rust_source_scopes(code, location)
        scopes_by_path[relative] = scopes
        for scope in scopes:
            module_key = (scope.location.source_root, scope.location.module)
            imports_by_module[module_key] = local_alias_imports(scope.code)
            for name, targets in local_type_aliases(scope.code).items():
                key = (*module_key, name)
                aliases[key] = aliases.get(key, ()) + targets

    for relative, scopes in scopes_by_path.items():
        for scope in scopes:
            code = scope.code
            for name, attributes, visibility, form, body in authority_structs(code):
                policy_entries = {
                    entry
                    for entry in policy_only_types
                    if entry[0] == relative and entry[1] == name
                }
                if policy_entries:
                    encountered_policy_only_types.update(policy_entries)
                    invalid_reasons: list[str] = []
                    if len(policy_entries) != 1:
                        invalid_reasons.append(
                            "must have exactly one policy-only classification"
                        )
                    if form != "unit":
                        invalid_reasons.append("must be a fieldless unit struct")
                    if re.search(r"\b(?:Serialize|Deserialize)\b", attributes):
                        invalid_reasons.append(
                            "must not derive Serialize or Deserialize"
                        )

                    for _path, _name, trait in sorted(policy_entries):
                        if (
                            re.search(
                                rf"\bimpl\s+{re.escape(trait)}\s+for\s+{re.escape(name)}\b",
                                code,
                            )
                            is None
                        ):
                            invalid_reasons.append(f"must implement exactly `{trait}`")

                    manufacture_functions = set()
                    for candidate_scope in scopes:
                        for function, return_type in function_signatures(
                            candidate_scope.code
                        ):
                            returned_authorities, _hazards = (
                                authority_return_resolution(
                                    return_type,
                                    candidate_scope.location,
                                    aliases,
                                    imports_by_module,
                                    {name},
                                )
                            )
                            if name in returned_authorities:
                                manufacture_functions.add(function)
                        for authority, impl_body in inherent_authority_impls(
                            candidate_scope.code
                        ):
                            if authority != name:
                                continue
                            for function, return_type in function_signatures(impl_body):
                                returned_authorities, _hazards = (
                                    authority_return_resolution(
                                        return_type,
                                        candidate_scope.location,
                                        aliases,
                                        imports_by_module,
                                        {name},
                                    )
                                )
                                if (
                                    re.search(r"\bSelf\b", return_type)
                                    or name in returned_authorities
                                ):
                                    manufacture_functions.add(function)
                    if manufacture_functions:
                        invalid_reasons.append(
                            "must not have manufacture functions: "
                            + ", ".join(sorted(manufacture_functions))
                        )

                    if not invalid_reasons:
                        continue
                    findings.append(
                        Finding(
                            "authority policy-only classification",
                            relative,
                            f"`{name}` " + "; ".join(invalid_reasons),
                        )
                    )
                measured_types.add((relative, name))
                publicly_constructible = (
                    (form == "brace" and PLAIN_PUBLIC_FIELD.search(body) is not None)
                    or (
                        form == "tuple"
                        and PLAIN_PUBLIC_TUPLE_FIELD.search(body) is not None
                    )
                    or (form == "unit" and visibility is not None)
                )
                if (
                    re.search(r"\b(?:Serialize|Deserialize)\b", attributes)
                    or publicly_constructible
                ):
                    findings.append(
                        Finding(
                            "authority opacity",
                            relative,
                            f"`{name}` must have no public fields and no Serialize/Deserialize derive",
                        )
                    )
    authority_names = {name for _path, name in measured_types}
    for relative, scopes in scopes_by_path.items():
        for scope in scopes:
            for function, return_type in function_signatures(scope.code):
                returned_authorities, alias_hazards = authority_return_resolution(
                    return_type,
                    scope.location,
                    aliases,
                    imports_by_module,
                    authority_names,
                )
                for authority in returned_authorities:
                    issuer_candidates.add((relative, function, authority))
                for hazard in sorted(alias_hazards):
                    findings.append(
                        Finding(
                            "authority alias resolution",
                            relative,
                            f"`{function}` cannot be classified safely: {hazard}",
                        )
                    )
            for authority, impl_body in inherent_authority_impls(scope.code):
                if authority not in authority_names:
                    continue
                for function, return_type in function_signatures(impl_body):
                    if re.search(r"\bSelf\b", return_type):
                        issuer_candidates.add((relative, function, authority))

    measured_issuers = {
        candidate for candidate in issuer_candidates if candidate[2] in authority_names
    }

    for path, name, trait in sorted(policy_only_types - encountered_policy_only_types):
        findings.append(
            Finding(
                "authority policy-only classification stale entry",
                path,
                f"remove missing `{name}` classified by `{trait}`",
            )
        )
    for path, name in sorted(measured_types - allowed_types):
        findings.append(
            Finding(
                "authority capability inventory",
                path,
                f"undocumented opaque capability `{name}`",
            )
        )
    for path, name in sorted(allowed_types - measured_types):
        findings.append(
            Finding(
                "authority capability inventory stale entry",
                path,
                f"remove missing capability `{name}`",
            )
        )
    for path, function, authority in sorted(measured_issuers - allowed_issuers):
        findings.append(
            Finding(
                "authority issuer inventory",
                path,
                f"undocumented issuer `{function}` -> `{authority}`",
            )
        )
    for path, function, authority in sorted(allowed_issuers - measured_issuers):
        findings.append(
            Finding(
                "authority issuer inventory stale entry",
                path,
                f"remove missing issuer `{function}` -> `{authority}`",
            )
        )
    return findings, len(measured_types), len(measured_issuers)


def dependency_cycle_findings(section: dict[str, Any]) -> tuple[list[Finding], int]:
    completion_maximum = require_nonnegative_int(
        section.get("completion_maximum"),
        "dependency_cycles.completion_maximum",
    )
    ceiling = require_nonnegative_int(
        section.get("debt_ceiling"), "dependency_cycles.debt_ceiling"
    )
    cycles = section.get("cycles")
    if not isinstance(cycles, list):
        raise ValueError("dependency_cycles.cycles must be an array")

    ids: list[str] = []
    for index, raw_cycle in enumerate(cycles):
        cycle = require_object(raw_cycle, f"dependency_cycles.cycles[{index}]")
        cycle_id = cycle.get("id")
        title = cycle.get("title")
        members = cycle.get("members")
        if not isinstance(cycle_id, str) or not cycle_id:
            raise ValueError(
                f"dependency_cycles.cycles[{index}].id must be a non-empty string"
            )
        if not isinstance(title, str) or not title:
            raise ValueError(
                f"dependency_cycles.cycles[{index}].title must be a non-empty string"
            )
        if (
            not isinstance(members, list)
            or not members
            or not all(isinstance(member, str) and member for member in members)
        ):
            raise ValueError(
                f"dependency_cycles.cycles[{index}].members must be a non-empty string array"
            )
        ids.append(cycle_id)
    if len(ids) != len(set(ids)):
        raise ValueError("dependency_cycles.cycles contains duplicate ids")

    actual = len(cycles)
    if actual > ceiling:
        detail = f"{actual} > allowed {ceiling}; ids={','.join(sorted(ids))}"
        return [
            Finding("dependency cycles", "checked analyzer manifest", detail)
        ], actual
    if actual != ceiling:
        detail = f"measured {actual}; ratchet baseline to {actual}"
        return [
            Finding(
                "dependency cycles stale debt ceiling",
                "checked analyzer manifest",
                detail,
            )
        ], actual
    return [], actual


def check(
    repo_root: Path, baseline: dict[str, Any]
) -> tuple[list[Finding], dict[str, int]]:
    (
        line_counts,
        compatibility_counts,
        broad_reexports,
        public_reexport_aliases,
    ) = measured_source_debt(repo_root)

    production = require_object(baseline.get("production_rust"), "production_rust")
    maximum_lines = require_nonnegative_int(
        production.get("completion_maximum_physical_lines"),
        "production_rust.completion_maximum_physical_lines",
    )
    line_ceilings = debt_map(production, "production_rust")
    oversized = {
        path: count for path, count in line_counts.items() if count > maximum_lines
    }

    compatibility = require_object(
        baseline.get("compatibility_leakage"),
        "compatibility_leakage",
    )
    compatibility_maximum = require_nonnegative_int(
        compatibility.get("completion_maximum"),
        "compatibility_leakage.completion_maximum",
    )

    reexports = require_object(baseline.get("broad_reexports"), "broad_reexports")
    reexport_maximum = require_nonnegative_int(
        reexports.get("completion_maximum"),
        "broad_reexports.completion_maximum",
    )
    reexport_allowlist = public_surface_allowlist(reexports, "broad_reexports")
    aliases = require_object(
        baseline.get("public_reexport_aliases"),
        "public_reexport_aliases",
    )
    alias_allowlist = public_surface_allowlist(aliases, "public_reexport_aliases")

    findings: list[Finding] = []
    findings.extend(
        compare_exact_debt(
            category="production Rust file size",
            measured=oversized,
            completion_maximum=maximum_lines,
            ceilings=line_ceilings,
        )
    )
    findings.extend(
        compare_exact_debt(
            category="compatibility leakage",
            measured=compatibility_counts,
            completion_maximum=compatibility_maximum,
            ceilings=debt_map(compatibility, "compatibility_leakage"),
        )
    )
    if reexport_maximum != 0 or debt_map(reexports, "broad_reexports"):
        raise ValueError(
            "broad_reexports must use an exact documented allowlist instead of debt ceilings"
        )
    findings.extend(
        public_surface_findings(
            category="broad re-export policy",
            measured=broad_reexports,
            allowed=reexport_allowlist,
        )
    )
    findings.extend(
        public_surface_findings(
            category="public re-export alias policy",
            measured=public_reexport_aliases,
            allowed=alias_allowlist,
        )
    )
    findings.extend(entity_owner_cycle_findings(repo_root))
    findings.extend(historical_public_path_findings(repo_root))
    inventory_findings = public_api_inventory_integrity_findings(repo_root)
    canonical_inventory_is_available = (
        repo_root / "docs/quality/generated-public-api-inventory.md"
    ).is_file() and not inventory_findings
    fallback_public_surface_findings = (
        []
        if canonical_inventory_is_available
        else consumerless_public_api_findings(repo_root)
        + dormant_public_variant_findings(repo_root)
    )
    dependency_boundary_findings = (
        manifest_dependency_findings(repo_root)
        + source_boundary_findings(repo_root)
        + public_rustdoc_and_literal_findings(repo_root)
        + provider_fixture_quarantine_findings(repo_root)
        + fallback_public_surface_findings
        + inventory_findings
        + untyped_claim_normalization_findings(repo_root)
        + typed_claim_normalization_findings(repo_root)
        + explicitly_inert_public_surface_findings(repo_root)
        + active_contract_findings(repo_root)
    )
    findings.extend(dependency_boundary_findings)
    authority_findings, authority_type_count, authority_issuer_count = (
        authority_capability_findings(
            repo_root,
            require_object(
                baseline.get("authority_capabilities"), "authority_capabilities"
            ),
        )
    )
    findings.extend(authority_findings)
    cycle_findings, cycle_count = dependency_cycle_findings(
        require_object(baseline.get("dependency_cycles"), "dependency_cycles")
    )
    findings.extend(cycle_findings)

    measurements = {
        "production_rust_files": len(line_counts),
        "oversized_files": len(oversized),
        "compatibility_leaks": sum(compatibility_counts.values()),
        "broad_reexports": sum(
            len(declarations) for declarations in broad_reexports.values()
        ),
        "public_reexport_aliases": sum(
            len(declarations) for declarations in public_reexport_aliases.values()
        ),
        "dependency_cycles": cycle_count,
        "dependency_boundary_violations": len(dependency_boundary_findings),
        "authority_capabilities": authority_type_count,
        "authority_issuers": authority_issuer_count,
    }
    return sorted(findings), measurements


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    repo_root = Path(args.repo_root).resolve()
    if not repo_root.is_dir():
        print(
            "architecture quality configuration error: repository root is missing",
            file=sys.stderr,
        )
        return 2

    baseline_path = (
        Path(args.baseline) if args.baseline else repo_root / DEFAULT_BASELINE
    )
    if not baseline_path.is_absolute():
        baseline_path = repo_root / baseline_path

    try:
        baseline = load_baseline(baseline_path)
        findings, measurements = check(repo_root, baseline)
    except (OSError, UnicodeError, ValueError) as exc:
        print(f"architecture quality configuration error: {exc}", file=sys.stderr)
        return 2

    if findings:
        print("architecture quality gate failed:", file=sys.stderr)
        for finding in findings:
            print(f"- {finding.render()}", file=sys.stderr)
        return 1

    print(
        "architecture_quality_ok "
        + " ".join(f"{key}={value}" for key, value in measurements.items())
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
