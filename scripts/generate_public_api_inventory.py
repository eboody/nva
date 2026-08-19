#!/usr/bin/env python3
"""Generate a symbol-level public API inventory from built Rust documentation."""

from __future__ import annotations

import argparse
from collections import defaultdict
import json
import re
import shutil
import subprocess
import sys
import tempfile
from collections.abc import Iterator
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path

SIDEBAR_PREFIX = "window.SIDEBAR_ITEMS = "
ASSOCIATED_ID = re.compile(
    r'id="(?P<kind>method|tymethod|variant|associatedconstant)\.(?P<name>[^"]+)'
)
SOURCE_HREF = re.compile(r'href="[^"]*src/(?P<path>[^"#]+)\.html(?:#[^"]*)?"')


@dataclass(frozen=True, order=True)
class ApiItem:
    path: str
    kind: str
    owner: str
    canonical_owner: str = ""


@dataclass(frozen=True)
class SourceIndex:
    paths: frozenset[str]
    imported_locals: dict[str, frozenset[str]]
    implemented_trait_methods: dict[str, frozenset[str]]
    code: str


def protobuf_fields(payload: bytes) -> Iterator[tuple[int, int, int | bytes]]:
    cursor = 0
    while cursor < len(payload):
        key = 0
        shift = 0
        while True:
            byte = payload[cursor]
            cursor += 1
            key |= (byte & 0x7F) << shift
            if byte < 0x80:
                break
            shift += 7
        field_number, wire_type = key >> 3, key & 7
        if wire_type == 0:
            value = 0
            shift = 0
            while True:
                byte = payload[cursor]
                cursor += 1
                value |= (byte & 0x7F) << shift
                if byte < 0x80:
                    break
                shift += 7
            yield field_number, wire_type, value
        elif wire_type == 2:
            length = 0
            shift = 0
            while True:
                byte = payload[cursor]
                cursor += 1
                length |= (byte & 0x7F) << shift
                if byte < 0x80:
                    break
                shift += 7
            value = payload[cursor : cursor + length]
            cursor += length
            yield field_number, wire_type, value
        elif wire_type == 1:
            cursor += 8
        elif wire_type == 5:
            cursor += 4
        else:
            raise ValueError(f"unsupported protobuf wire type {wire_type}")


@dataclass(frozen=True)
class ScipReferenceIndex:
    references_by_symbol: dict[str, dict[str, int]]

    @classmethod
    def load(cls, path: Path) -> "ScipReferenceIndex":
        documents: dict[str, dict[str, int]] = defaultdict(lambda: defaultdict(int))
        for field, wire, document_payload in protobuf_fields(path.read_bytes()):
            if field != 2 or wire != 2:
                continue
            relative_path = ""
            references: list[str] = []
            for document_field, document_wire, value in protobuf_fields(
                document_payload
            ):
                if document_field == 1 and document_wire == 2:
                    assert isinstance(value, bytes)
                    relative_path = value.decode("utf-8")
                elif document_field == 2 and document_wire == 2:
                    assert isinstance(value, bytes)
                    symbol = ""
                    roles = 0
                    for (
                        occurrence_field,
                        occurrence_wire,
                        occurrence_value,
                    ) in protobuf_fields(value):
                        if occurrence_field == 2 and occurrence_wire == 2:
                            assert isinstance(occurrence_value, bytes)
                            symbol = occurrence_value.decode("utf-8")
                        elif occurrence_field == 3 and occurrence_wire == 0:
                            assert isinstance(occurrence_value, int)
                            roles = occurrence_value
                    if symbol and roles & 1 == 0:
                        references.append(symbol)
            if relative_path:
                for symbol in references:
                    documents[symbol][relative_path] += 1
        return cls({symbol: dict(paths) for symbol, paths in documents.items()})

    def consumers(self, item: ApiItem) -> list[str]:
        segments = item.path.split("::")
        owner_parts = item.owner.split("/")
        package = (owner_parts[0] if item.owner != "unknown" else segments[0]).replace(
            "_", "-"
        )
        source_modules = owner_parts[1:]
        if source_modules:
            source_modules[-1] = source_modules[-1].removesuffix(".rs")
            if source_modules[-1] in {"lib", "main", "mod"}:
                source_modules.pop()
        source_module = "/".join(source_modules)
        associated = item.kind in {
            "method",
            "tymethod",
            "variant",
            "associatedconstant",
        }
        if associated:
            public_owner = segments[-2]
            member = segments[-1]
            public_module = "/".join(segments[1:-2])
        else:
            public_owner = segments[-1]
            member = ""
            public_module = "/".join(segments[1:-1])
        canonical_owner = (
            item.canonical_owner if len(item.canonical_owner) > 2 else ""
        )
        owners = tuple(filter(None, dict.fromkeys((public_owner, canonical_owner))))
        modules = tuple(filter(None, dict.fromkeys((public_module, source_module))))

        matching_symbols: list[str] = []
        for symbol in self.references_by_symbol:
            parts = symbol.split(" ", 4)
            if len(parts) != 5 or parts[2].replace("_", "-") != package:
                continue
            descriptor = parts[4]
            if item.kind == "tymethod":
                trait_implementation = any(
                    re.search(
                        rf"\]\[`?(?:[^`]::)*{re.escape(candidate)}(?:<[^`]*>)?`?\]\]?"
                        rf"{re.escape(member)}\(\)\.$",
                        descriptor,
                    )
                    is not None
                    for candidate in owners
                )
                if trait_implementation:
                    matching_symbols.append(symbol)
                    continue
            if modules and not any(f"{module}/" in descriptor for module in modules):
                continue
            if not associated:
                if item.kind == "fn":
                    suffix = rf"(?:^|/){re.escape(public_owner)}\(\)\.$"
                elif item.kind == "mod":
                    suffix = rf"(?:^|/){re.escape(public_owner)}/$"
                else:
                    suffix = ""
                if not any(
                    re.search(
                        suffix or rf"(?:^|/){re.escape(owner)}#$", descriptor
                    )
                    is not None
                    for owner in owners
                ):
                    continue
            elif item.kind == "variant":
                if not any(
                    re.search(
                        rf"(?:^|/){re.escape(owner)}#{re.escape(member)}#$",
                        descriptor,
                    )
                    is not None
                    for owner in owners
                ):
                    continue
            elif item.kind in {"method", "tymethod"}:
                if not any(
                    re.search(
                        (
                            rf"(?:^|/){re.escape(owner)}#"
                            if item.kind == "tymethod"
                            else rf"(?:^|/)(?:impl#\[`?(?:[^`]::)*{re.escape(owner)}"
                            rf"(?:<[^`]*>)?`?\]|{re.escape(owner)}#(?:[^#/.]+#)?)"
                        )
                        + rf"{re.escape(member)}(?:\(\))?\.$",
                        descriptor,
                    )
                    is not None
                    for owner in owners
                ):
                    continue
            elif not any(
                re.search(
                    rf"(?:^|/)(?:{re.escape(owner)}#|impl#\[{re.escape(owner)}\])"
                    rf"{re.escape(member)}(?:\.|#)$",
                    descriptor,
                )
                is not None
                for owner in owners
            ):
                continue
            matching_symbols.append(symbol)

        paths = {
            path
            for symbol in matching_symbols
            for path in self.references_by_symbol[symbol]
            if "/tests/" not in f"/{path}"
            and "/examples/" not in f"/{path}"
            and not Path(path).name.startswith("test")
        }
        return sorted(paths)


def parse_sidebar(path: Path, doc_root: Path) -> list[ApiItem]:
    raw = path.read_text(encoding="utf-8").strip()
    if not raw.startswith(SIDEBAR_PREFIX) or not raw.endswith(";"):
        raise ValueError(f"unsupported rustdoc sidebar format: {path}")
    entries = json.loads(raw[len(SIDEBAR_PREFIX) : -1])
    module_parts = path.parent.relative_to(doc_root).parts
    module = "::".join(module_parts)
    items: list[ApiItem] = []
    for kind, names in entries.items():
        for name in names:
            html = path.parent / f"{kind}.{name}.html"
            owner = "unknown"
            canonical_owner = ""
            associated: list[tuple[str, str]] = []
            if html.is_file():
                content = html.read_text(encoding="utf-8", errors="replace")
                source_match = SOURCE_HREF.search(content)
                if source_match is not None:
                    owner = source_match.group("path")
                canonical_match = re.search(
                    r'id="impl-[^"]+-for-(?P<name>[A-Za-z_][A-Za-z0-9_]*)"',
                    content,
                )
                if canonical_match is not None:
                    canonical_owner = canonical_match.group("name")
                trait_implementations = content.find('id="trait-implementations"')
                owned_content = (
                    content
                    if trait_implementations < 0
                    else content[:trait_implementations]
                )
                associated = []
                for match in ASSOCIATED_ID.finditer(owned_content):
                    if kind == "type":
                        continue
                    tag_end = owned_content.find(">", match.start())
                    tag = owned_content[match.start() : tag_end]
                    associated_name = match.group("name")
                    if (
                        "trait-impl" in tag
                        or "." in associated_name
                        or "-" in associated_name
                    ):
                        continue
                    associated.append((match.group("kind"), associated_name))
            qualified = f"{module}::{name}"
            items.append(ApiItem(qualified, kind, owner, canonical_owner))
            for associated_kind, associated_name in associated:
                items.append(
                    ApiItem(
                        f"{qualified}::{associated_name}",
                        associated_kind,
                        owner,
                        canonical_owner,
                    )
                )
    return items


def matching_brace_end(source: str, opening: int) -> int:
    depth = 1
    cursor = opening + 1
    while cursor < len(source) and depth:
        if source[cursor] == "{":
            depth += 1
        elif source[cursor] == "}":
            depth -= 1
        cursor += 1
    return cursor


def source_index(source: str) -> SourceIndex:
    paths = {
        re.sub(r"\s*::\s*", "::", match.group(0))
        for match in re.finditer(
            r"\b[A-Za-z_][A-Za-z0-9_]*(?:\s*::\s*[A-Za-z_][A-Za-z0-9_]*)+\b",
            source,
        )
    }
    imported_locals: dict[str, set[str]] = defaultdict(set)
    implemented_trait_methods: dict[str, set[str]] = defaultdict(set)
    for match in re.finditer(
        r"\buse\s+(?P<path>[A-Za-z_][A-Za-z0-9_]*(?:\s*::\s*[A-Za-z_][A-Za-z0-9_]*)+)"
        r"(?:\s+as\s+(?P<alias>[A-Za-z_][A-Za-z0-9_]*))?\s*;",
        source,
    ):
        canonical = re.sub(r"\s*::\s*", "::", match.group("path"))
        imported_locals[canonical].add(
            match.group("alias") or canonical.rsplit("::", 1)[-1]
        )
    for match in re.finditer(
        r"\buse\s+(?P<prefix>[A-Za-z_][A-Za-z0-9_]*(?:\s*::\s*[A-Za-z_][A-Za-z0-9_]*)*)"
        r"\s*::\s*\{(?P<body>[^}]*)\}\s*;",
        source,
        re.DOTALL,
    ):
        prefix = re.sub(r"\s*::\s*", "::", match.group("prefix"))
        for member in match.group("body").split(","):
            member_match = re.fullmatch(
                r"\s*(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
                r"(?:\s+as\s+(?P<alias>[A-Za-z_][A-Za-z0-9_]*))?\s*",
                member,
            )
            if member_match is None:
                continue
            canonical = f"{prefix}::{member_match.group('name')}"
            imported_locals[canonical].add(
                member_match.group("alias") or member_match.group("name")
            )
    for match in re.finditer(
        r"\bimpl(?:\s*<[^>{}]*>)?\s+"
        r"(?P<trait>[A-Za-z_][A-Za-z0-9_]*(?:\s*::\s*[A-Za-z_][A-Za-z0-9_]*)*)"
        r"(?:\s*<[^>{}]*>)?\s+for\s+[^{}]+\{",
        source,
        re.DOTALL,
    ):
        canonical = re.sub(r"\s*::\s*", "::", match.group("trait"))
        for imported_path, local_names in imported_locals.items():
            resolved = next(
                (
                    imported_path + canonical[len(local_name) :]
                    for local_name in local_names
                    if canonical == local_name
                    or canonical.startswith(f"{local_name}::")
                ),
                None,
            )
            if resolved is not None:
                canonical = resolved
                break
        opening = source.rfind("{", match.start(), match.end())
        closing = matching_brace_end(source, opening)
        body = source[opening + 1 : closing - 1]
        implemented_trait_methods[canonical].update(
            method.group("name")
            for method in re.finditer(
                r"\bfn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\(",
                body,
            )
        )
    canonical_paths = set(paths)
    for path in paths:
        for imported_path, local_names in imported_locals.items():
            for local_name in local_names:
                if path == local_name or path.startswith(f"{local_name}::"):
                    canonical_paths.add(imported_path + path[len(local_name) :])
    return SourceIndex(
        frozenset(canonical_paths),
        {
            canonical: frozenset(locals_)
            for canonical, locals_ in imported_locals.items()
        },
        {
            canonical: frozenset(methods)
            for canonical, methods in implemented_trait_methods.items()
        },
        re.sub(
            r"\bpub\s+use\s+[^;]+;|\buse\s+[^;]+;",
            " ",
            re.sub(
                r'//[^\n]*|/\*.*?\*/|r#*".*?"#*|"(?:\\.|[^"\\])*"',
                " ",
                source,
                flags=re.DOTALL,
            ),
            flags=re.DOTALL,
        ),
    )


def production_rust_sources(repo_root: Path) -> dict[str, SourceIndex]:
    sources: dict[str, SourceIndex] = {}
    for path in repo_root.rglob("*.rs"):
        relative = path.relative_to(repo_root)
        if any(part in {".git", "target", "tests"} for part in relative.parts):
            continue
        if path.name == "build.rs" or "test" in path.stem:
            continue
        sources[relative.as_posix()] = source_index(
            path.read_text(encoding="utf-8", errors="replace")
        )
    return sources


def symbol_is_referenced(source: SourceIndex, item: ApiItem) -> bool:
    segments = item.path.split("::")
    candidates = (item.path, "::".join(segments[1:]))
    if item.kind in {"method", "tymethod", "variant", "associatedconstant"}:
        member = segments[-1]
        if item.kind == "tymethod" and any(
            member in source.implemented_trait_methods.get(owner_path, frozenset())
            for owner_path in (
                "::".join(segments[:-1]),
                "::".join(segments[1:-1]),
                segments[-2],
            )
        ):
            return True
        if any(candidate in source.paths for candidate in candidates):
            return True
        for owner_path in ("::".join(segments[:-1]), "::".join(segments[1:-1])):
            for local_owner in source.imported_locals.get(owner_path, frozenset()):
                if f"{local_owner}::{member}" in source.paths:
                    return True
        return False
    if any(
        path == candidate or path.startswith(f"{candidate}::")
        for candidate in candidates
        for path in source.paths
    ):
        return True
    for candidate in candidates:
        if candidate in source.imported_locals:
            return True
    return False


def owner_source_references_symbol(source: SourceIndex, item: ApiItem) -> bool:
    """Recognize exact same-module uses without counting the declaration itself."""
    segments = item.path.split("::")
    name = segments[-1]
    if item.kind in {"method", "tymethod", "variant", "associatedconstant"}:
        owner = segments[-2]
        return any(
            path in {f"{owner}::{name}", f"Self::{name}"}
            or path.endswith(f"::{owner}::{name}")
            for path in source.paths
        )

    code = re.sub(
        rf"\b(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|trait|type|union)\s+{re.escape(name)}\b",
        " ",
        source.code,
        count=1,
    )
    code = re.sub(
        rf"\bimpl(?:\s*<[^>{{}}]*>)?\s+{re.escape(name)}\b",
        "impl ",
        code,
    )
    return re.search(rf"\b{re.escape(name)}\b", code) is not None


def consumers(
    item: ApiItem,
    sources: dict[str, SourceIndex],
    scip: ScipReferenceIndex | None,
) -> list[str]:
    if scip is not None:
        resolved = set(scip.consumers(item))
        segments = item.path.split("::")
        associated = item.kind in {
            "method",
            "tymethod",
            "variant",
            "associatedconstant",
        }
        public_owner = segments[-2] if associated else segments[-1]
        crate_source_roots = {
            "app": "app",
            "cli": "apps/cli",
            "domain": "domain",
            "gingr": "integrations/gingr",
            "nva_spacetimedb": "apps/spacetimedb",
            "pet_resort_api": "apps/api",
            "pet_resort_worker": "apps/worker",
            "storage": "storage",
        }
        public_module = segments[1:-2] if associated else segments[1:-1]
        public_root = crate_source_roots.get(segments[0])
        public_source_paths: set[str] = set()
        if public_root is not None:
            module_path = "/".join(public_module)
            if module_path:
                public_source_paths.update(
                    {
                        f"{public_root}/src/{module_path}.rs",
                        f"{public_root}/src/{module_path}/mod.rs",
                    }
                )
            else:
                public_source_paths.add(f"{public_root}/src/lib.rs")
        owner_sources = {
            path: source
            for path, source in sources.items()
            if path in public_source_paths
            or path == item.owner
            or (
                "/" in item.owner
                and path.endswith("/src/" + item.owner.split("/", 1)[1])
            )
        }
        for owner_source in owner_sources.values():
            for canonical, local_names in owner_source.imported_locals.items():
                if public_owner not in local_names:
                    continue
                canonical_path = (
                    f"{canonical}::{segments[-1]}" if associated else canonical
                )
                resolved.update(
                    scip.consumers(
                        ApiItem(canonical_path, item.kind, "unknown")
                    )
                )
        if item.kind == "tymethod":
            resolved.update(
                path
                for path, source in sources.items()
                if symbol_is_referenced(source, item)
            )
        else:
            # rust-analyzer SCIP can omit documents when a workspace contains
            # cyclic dev dependencies or duplicate test-crate symbols. Preserve
            # fail-closed canonical resolution by supplementing SCIP with the
            # same exact qualified/import-aware source resolver used when SCIP is
            # unavailable; never fall back to a same-leaf-name search.
            resolved.update(
                path
                for path, source in sources.items()
                if path != item.owner and symbol_is_referenced(source, item)
            )
        for path, owner_source in sources.items():
            owner_matches = path == item.owner or (
                "/" in item.owner
                and path.endswith("/src/" + item.owner.split("/", 1)[1])
            )
            if owner_matches and (
                owner_source_references_symbol(owner_source, item)
                or (item.kind == "tymethod" and symbol_is_referenced(owner_source, item))
            ):
                resolved.add(path)
        return sorted(resolved)
    return sorted(
        path
        for path, source in sources.items()
        if (
            (path != item.owner and symbol_is_referenced(source, item))
            or (
                path == item.owner
                and (
                    owner_source_references_symbol(source, item)
                    or (item.kind == "tymethod" and symbol_is_referenced(source, item))
                )
            )
        )
    )


def source_owner_path(item: ApiItem, repo_root: Path) -> Path | None:
    direct = repo_root / item.owner
    if direct.is_file():
        return direct
    if "/" not in item.owner:
        return None
    owner_crate, relative = item.owner.split("/", 1)
    crate_roots = {
        "app": "app",
        "cli": "apps/cli",
        "domain": "domain",
        "gingr": "integrations/gingr",
        "nva_spacetimedb": "apps/spacetimedb",
        "pet_resort_api": "apps/api",
        "pet_resort_worker": "apps/worker",
        "storage": "storage",
    }
    crate_root = crate_roots.get(owner_crate)
    if crate_root is not None:
        candidate = repo_root / crate_root / "src" / relative
        if candidate.is_file():
            return candidate

    segments = item.path.split("::")
    associated = item.kind in {
        "method",
        "tymethod",
        "variant",
        "associatedconstant",
    }
    public_owner = segments[-2] if associated else segments[-1]
    public_modules = segments[1:-2] if associated else segments[1:-1]
    public_root = crate_roots.get(segments[0])
    if public_root is None:
        return None
    public_source_base = repo_root / public_root / "src"
    public_candidates = (
        (
            public_source_base / ("/".join(public_modules) + ".rs"),
            public_source_base / Path(*public_modules) / "mod.rs",
        )
        if public_modules
        else (public_source_base / "lib.rs",)
    )
    public_source = next(
        (candidate for candidate in public_candidates if candidate.is_file()), None
    )
    if public_source is None:
        return None
    imports = source_index(
        public_source.read_text(encoding="utf-8", errors="replace")
    ).imported_locals
    canonical = next(
        (
            path
            for path, local_names in imports.items()
            if public_owner in local_names
        ),
        None,
    )
    if canonical is None:
        return None
    canonical_parts = canonical.split("::")
    canonical_root = crate_roots.get(canonical_parts[0])
    if canonical_root is None or len(canonical_parts) < 2:
        return None
    canonical_modules = canonical_parts[1:-1]
    canonical_source_base = repo_root / canonical_root / "src"
    canonical_candidates = (
        (
            canonical_source_base / ("/".join(canonical_modules) + ".rs"),
            canonical_source_base / Path(*canonical_modules) / "mod.rs",
        )
        if canonical_modules
        else (canonical_source_base / "lib.rs",)
    )
    return next(
        (candidate for candidate in canonical_candidates if candidate.is_file()), None
    )


def named_block_bodies(text: str, declaration: re.Pattern[str]) -> Iterator[str]:
    for match in declaration.finditer(text):
        opening = text.rfind("{", match.start(), match.end())
        closing = matching_brace_end(text, opening)
        yield text[opening + 1 : closing - 1]


def associated_item_is_declared(item: ApiItem, text: str) -> bool:
    segments = item.path.split("::")
    public_owner = segments[-2]
    owners = tuple(dict.fromkeys((public_owner, item.canonical_owner)))
    member = re.escape(segments[-1])
    for owner in filter(None, owners):
        escaped_owner = re.escape(owner)
        if item.kind in {"method", "associatedconstant"}:
            impl = re.compile(
                rf"\bimpl(?:\s*<[^{{}}]*>)?\s+"
                rf"(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*{escaped_owner}"
                rf"(?:\s*<[^{{}}]*>)?(?:\s+where\b[^{{}}]*)?\s*\{{",
                re.DOTALL,
            )
            declaration = (
                rf"\bfn\s+{member}\b"
                if item.kind == "method"
                else rf"\bconst\s+{member}\b"
            )
            if any(
                re.search(declaration, body) is not None
                for body in named_block_bodies(text, impl)
            ):
                return True
        elif item.kind == "tymethod":
            trait = re.compile(
                rf"\btrait\s+{escaped_owner}(?:\s*<[^{{}}]*>)?[^{{}}]*\{{",
                re.DOTALL,
            )
            if any(
                re.search(rf"\bfn\s+{member}\b", body) is not None
                for body in named_block_bodies(text, trait)
            ):
                return True
        elif item.kind == "variant":
            enum = re.compile(
                rf"\benum\s+{escaped_owner}(?:\s*<[^{{}}]*>)?[^{{}}]*\{{",
                re.DOTALL,
            )
            if any(
                re.search(rf"(?m)^\s*{member}\s*(?:[({{,=]|$)", body)
                is not None
                for body in named_block_bodies(text, enum)
            ):
                return True
    return False


def associated_item_is_generated(item: ApiItem, repo_root: Path) -> bool:
    if item.kind not in {"method", "tymethod", "variant", "associatedconstant"}:
        return False
    source = source_owner_path(item, repo_root)
    if source is None:
        return False
    text = source.read_text(encoding="utf-8", errors="replace")
    return not associated_item_is_declared(item, text)


def structural_consumer_evidence(item: ApiItem, repo_root: Path) -> list[str]:
    source = source_owner_path(item, repo_root)
    if source is None:
        return []
    relative = source.relative_to(repo_root).as_posix()
    if item.path == "cli::main" and item.kind == "fn":
        return [f"{relative} (binary executable entry point)"]
    text = source.read_text(encoding="utf-8", errors="replace")
    if item.kind == "struct" and re.search(
        rf"\bsimple_reference_endpoint!\s*\(\s*"
        rf"{re.escape(item.path.rsplit('::', 1)[-1])}\s*,",
        text,
    ):
        return [f"{relative} (generated provider endpoint registration)"]
    if not item.path.startswith("nva_spacetimedb::"):
        return []
    if "#[spacetimedb::" not in text:
        return []
    segments = item.path.split("::")
    associated = item.kind in {
        "method",
        "tymethod",
        "variant",
        "associatedconstant",
    }
    generated_owner = segments[-2] if associated else segments[-1]
    generated_handle = re.search(
        r"(?:Row(?:Ix)?Cols|__(?:Table|View)Handle|__(?:query|view))$",
        generated_owner,
    )
    generated_reducer = item.kind == "struct" and re.search(
        rf"#\s*\[\s*spacetimedb::reducer[^]]*\]\s*"
        rf"pub\s+fn\s+{re.escape(generated_owner)}\b",
        text,
        re.DOTALL,
    )
    if generated_handle is None and generated_reducer is None:
        return []
    return [f"{relative} (SpacetimeDB macro registration)"]


def trait_implementation_evidence(item: ApiItem, repo_root: Path) -> list[str]:
    """Resolve trait-method declarations through concrete production impls."""
    if item.kind != "tymethod":
        return []
    trait_name, method_name = item.path.split("::")[-2:]
    implementation = re.compile(
        rf"\bimpl(?:\s*<[^{{}}]*>)?\s+[^{{}}]*\b{re.escape(trait_name)}\b"
        rf"[^{{}}]*\bfor\b[^{{}}]*\{{",
        re.DOTALL,
    )
    method = re.compile(rf"\bfn\s+{re.escape(method_name)}\s*(?:<[^>{{}}]*>)?\s*\(")
    consumers: list[str] = []
    for relative in production_rust_sources(repo_root):
        path = repo_root / relative
        text = path.read_text(encoding="utf-8", errors="replace")
        if any(method.search(body) for body in named_block_bodies(text, implementation)):
            consumers.append(
                f"{relative} (concrete `{trait_name}` implementation)"
            )
    return consumers


@lru_cache(maxsize=None)
def serialized_enum_names(source: Path) -> frozenset[str]:
    text = source.read_text(encoding="utf-8", errors="replace")
    names: set[str] = set()
    for declaration in re.finditer(r"\bpub\s+enum\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\b", text):
        prefix = text[max(0, declaration.start() - 2_000) : declaration.end()]
        derive = re.search(
            rf"#\s*\[\s*derive\s*\((?P<traits>.*?)\)\s*\]\s*"
            rf"(?:(?://[/!]\s*[^\n]*)\s*)*pub\s+enum\s+"
            rf"{re.escape(declaration.group('name'))}\b\s*$",
            prefix,
            re.DOTALL,
        )
        if derive is not None and re.search(
            r"\b(?:Serialize|Deserialize)\b", derive.group("traits")
        ):
            names.add(declaration.group("name"))
    return frozenset(names)


def serialized_variant_evidence(
    item: ApiItem,
    repo_root: Path,
    direct_by_path: dict[str, list[str]],
) -> list[str]:
    if item.kind != "variant":
        return []
    owner_path = item.path.rsplit("::", 1)[0]
    owner_consumers = direct_by_path.get(owner_path, [])
    if not owner_consumers:
        return []
    source = source_owner_path(item, repo_root)
    if source is None:
        return []
    public_owner = owner_path.rsplit("::", 1)[-1]
    owners = tuple(dict.fromkeys((public_owner, item.canonical_owner)))
    for owner in filter(None, owners):
        if owner not in serialized_enum_names(source):
            continue
        relative = source.relative_to(repo_root).as_posix()
        return [
            f"{relative} (serialized enum boundary `{owner_path}` consumed by "
            + ", ".join(owner_consumers)
            + ")"
        ]
    return []


@lru_cache(maxsize=None)
def derived_string_enum_names(source: Path) -> frozenset[str]:
    text = source.read_text(encoding="utf-8", errors="replace")
    names: set[str] = set()
    for declaration in re.finditer(
        r"\bpub\s+enum\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\b", text
    ):
        prefix = text[max(0, declaration.start() - 2_000) : declaration.end()]
        derive = re.search(
            rf"#\s*\[\s*derive\s*\((?P<traits>.*?)\)\s*\]\s*"
            rf"(?:#\s*\[[^]]*\]\s*)*"
            rf"(?:(?://[/!]\s*[^\n]*)\s*)*pub\s+enum\s+"
            rf"{re.escape(declaration.group('name'))}\b\s*$",
            prefix,
            re.DOTALL,
        )
        if derive is not None and re.search(
            r"\b(?:EnumString|Display)\b", derive.group("traits")
        ):
            names.add(declaration.group("name"))
    return frozenset(names)


def derived_string_variant_evidence(
    item: ApiItem,
    repo_root: Path,
    direct_by_path: dict[str, list[str]],
) -> list[str]:
    if item.kind != "variant":
        return []
    owner_path = item.path.rsplit("::", 1)[0]
    owner_consumers = direct_by_path.get(owner_path, [])
    if not owner_consumers:
        return []
    source = source_owner_path(item, repo_root)
    if source is None:
        return []
    public_owner = owner_path.rsplit("::", 1)[-1]
    owners = tuple(dict.fromkeys((public_owner, item.canonical_owner)))
    if not any(owner in derived_string_enum_names(source) for owner in owners):
        return []
    relative = source.relative_to(repo_root).as_posix()
    return [
        f"{relative} (derived external string vocabulary `{owner_path}` consumed by "
        + ", ".join(owner_consumers)
        + ")"
    ]


@lru_cache(maxsize=None)
def error_conversion_variants(source: Path) -> frozenset[tuple[str, str]]:
    text = source.read_text(encoding="utf-8", errors="replace")
    variants: set[tuple[str, str]] = set()
    declaration = re.compile(
        r"#\s*\[\s*derive\s*\((?P<traits>.*?)\)\s*\]\s*"
        r"(?:(?://[/!]\s*[^\n]*)\s*)*pub\s+enum\s+"
        r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)\b[^{}]*\{",
        re.DOTALL,
    )
    for match in declaration.finditer(text):
        if re.search(
            r"(?:^|[,:\s])(?:thiserror::)?Error(?:$|[,:\s])",
            match.group("traits"),
        ) is None:
            continue
        opening = text.rfind("{", match.start(), match.end())
        closing = matching_brace_end(text, opening)
        body = text[opening + 1 : closing - 1]
        for member in re.finditer(
            r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\((?P<fields>[^()]*)\)",
            body,
            re.DOTALL,
        ):
            if "#[from]" in member.group("fields"):
                variants.add((match.group("name"), member.group("name")))
    return frozenset(variants)


def error_variant_evidence(item: ApiItem, repo_root: Path) -> list[str]:
    if item.kind != "variant":
        return []
    source = source_owner_path(item, repo_root)
    if source is None:
        return []
    segments = item.path.split("::")
    owner, member = segments[-2:]
    owners = tuple(dict.fromkeys((owner, item.canonical_owner)))
    if not any(
        (candidate, member) in error_conversion_variants(source)
        for candidate in owners
    ):
        return []
    relative = source.relative_to(repo_root).as_posix()
    return [
        f"{relative} (generated error conversion boundary "
        f"`{item.path.rsplit('::', 1)[0]}`)"
    ]


def associated_item_is_read_only_accessor(item: ApiItem, repo_root: Path) -> bool:
    if item.kind != "method":
        return False
    source = source_owner_path(item, repo_root)
    if source is None:
        return False
    text = source.read_text(encoding="utf-8", errors="replace")
    segments = item.path.split("::")
    public_owner, member = segments[-2:]
    owners = tuple(dict.fromkeys((public_owner, item.canonical_owner)))
    for owner in filter(None, owners):
        implementation = re.compile(
            rf"\bimpl(?:\s*<[^{{}}]*>)?\s+(?:[^{{}}]*::)?"
            rf"{re.escape(owner)}(?:\s*<[^{{}}]*>)?[^{{}}]*\{{",
            re.DOTALL,
        )
        for implementation_body in named_block_bodies(text, implementation):
            declaration = re.compile(
                rf"\bpub\s+(?:const\s+)?fn\s+{re.escape(member)}"
                rf"(?:\s*<[^>{{}}]*>)?\s*\((?P<params>[^)]*)\)"
                rf"[^;{{}}]*\{{",
                re.DOTALL,
            )
            for method in declaration.finditer(implementation_body):
                params = re.sub(r"\s+", "", method.group("params"))
                if "&self" not in params or "&mutself" in params:
                    continue
                opening = implementation_body.rfind("{", method.start(), method.end())
                closing = matching_brace_end(implementation_body, opening)
                expression = implementation_body[opening + 1 : closing - 1].strip()
                if ";" in expression or re.search(r"(?<![=!<>])=(?!=)", expression):
                    continue
                if re.fullmatch(
                    r"&?\s*self\.[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*"
                    r"(?:\.(?:as_ref|as_str|as_slice|as_deref|iter|len|is_empty|clone|copied|cloned|get)"
                    r"\([^()]*\))*",
                    expression,
                ):
                    return True
                if params != "&self" or "self." not in expression:
                    continue
                calls = re.findall(r"\.([A-Za-z_][A-Za-z0-9_]*)\s*\(", expression)
                pure_calls = {
                    "all",
                    "and_then",
                    "any",
                    "as_deref",
                    "as_ref",
                    "as_slice",
                    "as_str",
                    "checked_add",
                    "checked_sub",
                    "clone",
                    "cloned",
                    "collect",
                    "contains",
                    "copied",
                    "count",
                    "filter",
                    "find",
                    "first",
                    "get",
                    "is_empty",
                    "is_none",
                    "is_some",
                    "iter",
                    "last",
                    "len",
                    "map",
                    "or_else",
                    "saturating_add",
                    "saturating_sub",
                    "sum",
                    "unwrap_or",
                    "unwrap_or_default",
                }
                if all(call in pure_calls for call in calls):
                    return True
    return False


def effective_consumers(
    item: ApiItem,
    repo_root: Path,
    direct_by_path: dict[str, list[str]],
    items_by_path: dict[str, ApiItem],
) -> list[str]:
    direct = direct_by_path[item.path]
    if direct:
        return direct
    if item.kind == "mod":
        descendant_consumers = sorted(
            {
                consumer
                for path, consumers_for_path in direct_by_path.items()
                if path.startswith(f"{item.path}::")
                for consumer in consumers_for_path
            }
        )
        if descendant_consumers:
            return [
                f"module namespace for consumed descendants: "
                + ", ".join(descendant_consumers)
            ]
    structural = structural_consumer_evidence(item, repo_root)
    if structural:
        return structural
    trait_implementation = trait_implementation_evidence(item, repo_root)
    if trait_implementation:
        return trait_implementation
    serialized_variant = serialized_variant_evidence(item, repo_root, direct_by_path)
    if serialized_variant:
        return serialized_variant
    derived_string_variant = derived_string_variant_evidence(
        item, repo_root, direct_by_path
    )
    if derived_string_variant:
        return derived_string_variant
    error_variant = error_variant_evidence(item, repo_root)
    if error_variant:
        return error_variant
    owner_path: str | None = None
    read_only_accessor = associated_item_is_read_only_accessor(item, repo_root)
    if read_only_accessor:
        owner_path = item.path.rsplit("::", 1)[0]
    elif associated_item_is_generated(item, repo_root):
        owner_path = item.path.rsplit("::", 1)[0]
    elif item.kind in {"enum", "struct"}:
        name = item.path.rsplit("::", 1)[-1]
        for suffix in ("Error", "Builder"):
            if name.endswith(suffix):
                candidate = item.path[: -len(suffix)]
                if candidate in items_by_path and source_owner_path(item, repo_root) == source_owner_path(
                    items_by_path[candidate], repo_root
                ):
                    owner_path = candidate
                    break
    if owner_path is None or owner_path == item.path:
        return []
    owner = items_by_path.get(owner_path)
    if owner is None:
        return direct_by_path.get(owner_path, [])
    inherited = effective_consumers(
        owner, repo_root, direct_by_path, items_by_path
    )
    if inherited and read_only_accessor:
        source = source_owner_path(item, repo_root)
        assert source is not None
        relative = source.relative_to(repo_root).as_posix()
        return [
            f"{relative} (read-only accessor on consumed owner `{owner_path}`: "
            + ", ".join(inherited)
            + ")"
        ]
    if (
        not inherited
        and associated_item_is_generated(item, repo_root)
        and has_external_contract_evidence(owner, repo_root)
    ):
        source = source_owner_path(item, repo_root)
        assert source is not None
        return [
            f"{source.relative_to(repo_root).as_posix()} "
            f"(generated public contract inherited from exact external-contract owner "
            f"`{owner_path}`)"
        ]
    return inherited


def render(
    items: list[ApiItem],
    sources: dict[str, SourceIndex],
    scip: ScipReferenceIndex | None,
    repo_root: Path,
    direct_by_path: dict[str, list[str]],
    items_by_path: dict[str, ApiItem],
) -> str:
    rows = [
        "# Generated Rust public API inventory",
        "",
        "Generated from `cargo doc --workspace --all-features --no-deps --locked`.",
        "Every rustdoc item, method, associated constant, and enum variant is listed with its source owner and symbol-resolved non-test workspace consumers. Consumerless symbols require explicit external-contract evidence or removal.",
        "",
        f"Total exported entries: {len(items)}",
        "",
        "| Public API | Kind | Source owner | Non-test workspace consumers |",
        "| --- | --- | --- | --- |",
    ]
    for item in sorted(set(items)):
        consumer_paths = effective_consumers(
            item, repo_root, direct_by_path, items_by_path
        )
        consumer_text = ", ".join(f"`{path}`" for path in consumer_paths)
        if not consumer_text:
            external = external_contract_evidence(item, repo_root)
            consumer_text = (
                f"external contract: {external}" if external is not None else "none found"
            )
        resolved_owner = source_owner_path(item, repo_root)
        owner_text = (
            resolved_owner.relative_to(repo_root).as_posix()
            if resolved_owner is not None
            else item.owner
        )
        rows.append(
            f"| `{item.path}` | {item.kind} | `{owner_text}` | {consumer_text} |"
        )
    rows.append("")
    return "\n".join(rows)


def valid_external_contract_evidence(
    evidence: object, owner: Path | None, repo_root: Path
) -> bool:
    """Accept only item-specific executable runtime-boundary evidence."""
    if not isinstance(evidence, dict) or set(evidence) != {"kind", "path", "locator"}:
        return False
    kind = evidence.get("kind")
    relative = evidence.get("path")
    locator = evidence.get("locator")
    if kind not in {"runtime_contract", "generated_runtime_registration"}:
        return False
    if not all(isinstance(value, str) and value.strip() for value in (relative, locator)):
        return False
    assert isinstance(relative, str)
    path = repo_root / relative
    if not path.is_file() or any(part in {"tests", "examples"} for part in path.parts):
        return False
    if owner is not None and path.resolve() == owner.resolve():
        return False
    assert isinstance(locator, str)
    return locator in path.read_text(encoding="utf-8", errors="replace")


def external_contract_evidence(item: ApiItem, repo_root: Path) -> str | None:
    owner = source_owner_path(item, repo_root)
    if owner is not None and f"external-contract: {item.path}" in owner.read_text(
        encoding="utf-8"
    ):
        return f"exact source annotation at `{owner.relative_to(repo_root).as_posix()}`"
    inventory = repo_root / "docs" / "quality" / "public-api-inventory.json"
    if not inventory.is_file():
        return None
    entries = json.loads(inventory.read_text(encoding="utf-8")).get(
        "external_contract_symbols", []
    )
    for entry in entries:
        rationale = entry.get("rationale")
        evidence = entry.get("evidence")
        if (
            entry.get("symbol") == item.path
            and isinstance(rationale, str)
            and rationale.strip()
            and valid_external_contract_evidence(evidence, owner, repo_root)
        ):
            assert isinstance(evidence, dict)
            return (
                f"{rationale} Evidence: {evidence['kind']} at "
                f"`{evidence['path']}` locator `{evidence['locator']}`"
            )
    return None


def has_external_contract_evidence(item: ApiItem, repo_root: Path) -> bool:
    return external_contract_evidence(item, repo_root) is not None


def generate_scip_index(
    repo_root: Path, requested: Path | None
) -> ScipReferenceIndex | None:
    if requested is not None:
        return ScipReferenceIndex.load(requested)
    if (
        not (repo_root / "Cargo.toml").is_file()
        or shutil.which("rust-analyzer") is None
    ):
        return None
    with tempfile.NamedTemporaryFile(suffix=".scip") as output:
        subprocess.run(
            [
                "rust-analyzer",
                "scip",
                ".",
                "--output",
                output.name,
                "--exclude-vendored-libraries",
            ],
            cwd=repo_root,
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=180,
        )
        return ScipReferenceIndex.load(Path(output.name))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--doc-root", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--scip-index", type=Path)
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail when the checked output differs without rewriting it",
    )
    args = parser.parse_args()

    repo_root = args.repo_root.resolve()
    doc_root = (args.doc_root or repo_root / "target" / "doc").resolve()
    output = (
        args.output
        or repo_root / "docs" / "quality" / "generated-public-api-inventory.md"
    ).resolve()
    sidebars = sorted(doc_root.rglob("sidebar-items.js"))
    if not sidebars:
        raise SystemExit(f"no rustdoc sidebar inventory found under {doc_root}")
    items = [item for sidebar in sidebars for item in parse_sidebar(sidebar, doc_root)]
    sources = production_rust_sources(repo_root)
    scip = generate_scip_index(repo_root, args.scip_index)
    direct_by_path = {
        item.path: consumers(item, sources, scip) for item in sorted(set(items))
    }
    items_by_path = {item.path: item for item in sorted(set(items))}
    inventory_contract_path = repo_root / "docs" / "quality" / "public-api-inventory.json"
    contract_entries = (
        json.loads(inventory_contract_path.read_text(encoding="utf-8")).get(
            "external_contract_symbols", []
        )
        if inventory_contract_path.is_file()
        else []
    )
    contract_symbols: list[str] = [
        symbol
        for entry in contract_entries
        if isinstance(entry, dict)
        and isinstance((symbol := entry.get("symbol")), str)
        and symbol.strip()
    ]
    malformed_contracts = [
        entry
        for entry in contract_entries
        if not isinstance(entry, dict)
        or not isinstance((symbol := entry.get("symbol")), str)
        or not symbol.strip()
        or not isinstance(entry.get("rationale"), str)
        or not entry["rationale"].strip()
        or not valid_external_contract_evidence(
            entry.get("evidence"),
            source_owner_path(items_by_path[symbol], repo_root)
            if symbol in items_by_path
            else None,
            repo_root,
        )
    ]
    duplicate_contracts = sorted(
        {symbol for symbol in contract_symbols if contract_symbols.count(symbol) > 1}
    )
    stale_contracts = sorted(
        symbol for symbol in contract_symbols if symbol not in items_by_path
    )
    if malformed_contracts or duplicate_contracts or stale_contracts:
        malformed_symbols = sorted(
            str(entry.get("symbol", "<missing-symbol>"))
            if isinstance(entry, dict)
            else repr(entry)
            for entry in malformed_contracts
        )
        print(
            "public_api_inventory_error: external-contract inventory integrity failure: "
            f"malformed={malformed_symbols} "
            f"duplicates={duplicate_contracts} stale={stale_contracts}",
            file=sys.stderr,
        )
        return 1
    rendered = render(items, sources, scip, repo_root, direct_by_path, items_by_path)
    consumerless = [
        item
        for item in sorted(set(items))
        if not effective_consumers(item, repo_root, direct_by_path, items_by_path)
        and not has_external_contract_evidence(item, repo_root)
    ]
    if consumerless:
        print(
            "public_api_inventory_error: consumerless public API requires removal or "
            "explicit external-contract evidence: "
            + ", ".join(item.path for item in consumerless),
            file=sys.stderr,
        )
        return 1
    if args.check:
        if not output.is_file() or output.read_text(encoding="utf-8") != rendered:
            print(
                f"public_api_inventory_error: checked public API inventory is stale: {output}",
                file=sys.stderr,
            )
            return 1
    else:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(rendered, encoding="utf-8")
    print(f"public_api_inventory_ok entries={len(set(items))} output={output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
