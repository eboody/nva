import json
import os
import subprocess
import sys
from pathlib import Path

import pytest


REPO_ROOT = Path(__file__).resolve().parents[2]
CHECK = REPO_ROOT / "scripts" / "check_architecture_quality.py"


def write_manifest(root: Path, **overrides: object) -> Path:
    manifest: dict[str, object] = {
        "schema_version": 1,
        "source": "test fixture",
        "production_rust": {
            "completion_maximum_physical_lines": 1_500,
            "debt_ceiling_by_path": {},
        },
        "compatibility_leakage": {
            "completion_maximum": 0,
            "debt_ceiling_by_path": {},
        },
        "broad_reexports": {
            "completion_maximum": 0,
            "debt_ceiling_by_path": {},
            "allowlist": [],
        },
        "public_reexport_aliases": {
            "allowlist": [],
        },
        "dependency_cycles": {
            "completion_maximum": 0,
            "debt_ceiling": 0,
            "cycles": [],
        },
        "authority_capabilities": {
            "policy_only_types": [],
            "types": [],
            "issuers": [],
        },
    }
    manifest.update(overrides)
    path = root / "docs" / "quality" / "architecture-quality-baseline.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return path


def write_rust(root: Path, relative: str, source: str) -> None:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(source, encoding="utf-8")


def run_check(root: Path, manifest: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            sys.executable,
            str(CHECK),
            "--repo-root",
            str(root),
            "--baseline",
            str(manifest),
        ],
        text=True,
        capture_output=True,
        check=False,
    )


def run_cargo_check(root: Path) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.pop("TEST_DATABASE_URL", None)
    env.pop("DATABASE_URL", None)
    return subprocess.run(
        ["cargo", "check", "--quiet", "--manifest-path", str(root / "Cargo.toml")],
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )


def write_rust_2024_crate(root: Path, modules: dict[str, str]) -> None:
    (root / "Cargo.toml").write_text(
        '[package]\nname = "architecture-alias-fixture"\nversion = "0.1.0"\nedition = "2024"\n',
        encoding="utf-8",
    )
    write_rust(
        root,
        "src/lib.rs",
        "".join(f"mod {module};\n" for module in sorted(modules)),
    )
    for module, source in modules.items():
        write_rust(root, f"src/{module}.rs", source)


def write_inline_rust_2024_crate(root: Path, source: str) -> None:
    (root / "Cargo.toml").write_text(
        '[package]\nname = "architecture-inline-alias-fixture"\nversion = "0.1.0"\nedition = "2024"\n',
        encoding="utf-8",
    )
    write_rust(root, "src/lib.rs", source)


def test_clean_fixture_passes_with_stable_relative_summary(tmp_path: Path) -> None:
    write_rust(
        root=tmp_path, relative="domain/src/lib.rs", source="pub mod customer;\n"
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr
    assert result.stdout == (
        "architecture_quality_ok production_rust_files=1 oversized_files=0 "
        "compatibility_leaks=0 broad_reexports=0 public_reexport_aliases=0 "
        "dependency_cycles=0 dependency_boundary_violations=0 "
        "authority_capabilities=0 authority_issuers=0\n"
    )
    assert str(tmp_path) not in result.stdout + result.stderr


def test_test_only_rust_modules_under_src_are_not_production_debt(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "apps/example/src/realtime_queue_tests.rs",
        "pub fn legacy_fixture() {}\n",
    )
    write_rust(
        tmp_path, "apps/example/src/lib.rs", "#[cfg(test)]\nmod realtime_queue_tests;\n"
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr
    assert "production_rust_files=1" in result.stdout
    assert "compatibility_leaks=0" in result.stdout


def test_new_oversized_production_rust_file_is_rejected(tmp_path: Path) -> None:
    write_rust(tmp_path, "app/src/oversized.rs", "// line\n" * 1_501)
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "production Rust file size" in result.stderr
    assert "app/src/oversized.rs" in result.stderr
    assert "1501 > allowed 1500" in result.stderr


def test_existing_oversized_file_is_ratcheted_and_cannot_grow(tmp_path: Path) -> None:
    write_rust(tmp_path, "app/src/existing_debt.rs", "// line\n" * 1_601)
    manifest = write_manifest(
        tmp_path,
        production_rust={
            "completion_maximum_physical_lines": 1_500,
            "debt_ceiling_by_path": {"app/src/existing_debt.rs": 1_600},
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "app/src/existing_debt.rs" in result.stderr
    assert "1601 > allowed 1600" in result.stderr


def test_reduced_debt_requires_the_checked_ceiling_to_ratchet_down(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "app/src/existing_debt.rs", "// line\n" * 1_550)
    manifest = write_manifest(
        tmp_path,
        production_rust={
            "completion_maximum_physical_lines": 1_500,
            "debt_ceiling_by_path": {"app/src/existing_debt.rs": 1_600},
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "stale debt ceiling" in result.stderr
    assert "ratchet baseline to 1550" in result.stderr


def test_new_compatibility_vocabulary_outside_codec_boundary_is_rejected(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "domain/src/customer.rs",
        "pub struct LegacyCustomerId(String);\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "compatibility leakage" in result.stderr
    assert "domain/src/customer.rs" in result.stderr
    assert "1 > allowed 0" in result.stderr


def test_compatibility_vocabulary_is_detected_inside_snake_case_identifiers(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "storage/src/records.rs",
        "pub fn rehydrate_legacy_record() {}\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "compatibility leakage" in result.stderr
    assert "storage/src/records.rs" in result.stderr


def test_clean_slate_gate_rejects_compatibility_vocabulary_inside_compatibility_directories(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "storage/src/operations/compatibility/mod.rs",
        "pub struct LegacyRecord;\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "compatibility leakage" in result.stderr
    assert "storage/src/operations/compatibility/mod.rs" in result.stderr


def test_clean_slate_gate_rejects_compatibility_directories_even_without_legacy_identifiers(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "storage/src/operations/compatibility/mod.rs",
        "pub struct CurrentRecord;\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "compatibility leakage" in result.stderr
    assert "storage/src/operations/compatibility/mod.rs" in result.stderr


def test_rust_lifetimes_do_not_hide_compatibility_identifiers(tmp_path: Path) -> None:
    write_rust(
        tmp_path,
        "integrations/example/src/lib.rs",
        """
pub const BEFORE: &'static str = "before";
pub struct LegacyBoundary;
pub const AFTER: &'static str = "after";
""",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert (
        "compatibility leakage: integrations/example/src/lib.rs: 1 > allowed 0"
        in result.stderr
    )


def test_compatibility_occurrence_above_corrected_exact_ceiling_is_rejected(
    tmp_path: Path,
) -> None:
    path = "integrations/example/src/lib.rs"
    write_rust(
        tmp_path,
        path,
        """
pub fn one(_: &'static str) -> LegacyDateBoundary { todo!() }
pub fn two(_: &'static str) -> LegacyDateBoundary { todo!() }
pub fn three(_: &'static str) -> LegacyDateBoundary { todo!() }
pub fn four(_: &'static str) -> LegacyDateBoundary { todo!() }
""",
    )
    manifest = write_manifest(
        tmp_path,
        compatibility_leakage={
            "completion_maximum": 0,
            "debt_ceiling_by_path": {path: 3},
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert f"compatibility leakage: {path}: 4 > allowed 3" in result.stderr


def test_historical_public_facade_is_rejected_outside_codec_boundary(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "domain/src/lib.rs", "pub mod strategic_ai_ops;\n")
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "historical public path" in result.stderr
    assert "domain/src/lib.rs" in result.stderr
    assert "strategic_ai_ops" in result.stderr


def test_new_unrestricted_broad_reexport_is_rejected(tmp_path: Path) -> None:
    write_rust(tmp_path, "domain/src/lib.rs", "pub use customer::*;\n")
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "broad re-export policy" in result.stderr
    assert "domain/src/lib.rs" in result.stderr
    assert "1 > allowed 0" in result.stderr


def test_exact_broad_reexport_with_documented_runtime_rationale_is_allowed(
    tmp_path: Path,
) -> None:
    declaration = "pub use reducers::*;"
    write_rust(tmp_path, "apps/runtime/src/lib.rs", declaration + "\n")
    manifest = write_manifest(
        tmp_path,
        broad_reexports={
            "completion_maximum": 0,
            "debt_ceiling_by_path": {},
            "allowlist": [
                {
                    "path": "apps/runtime/src/lib.rs",
                    "declaration": declaration,
                    "rationale": "Runtime macro registration requires reducer exports at the crate root.",
                }
            ],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr


def test_new_public_reexport_alias_is_rejected(tmp_path: Path) -> None:
    write_rust(
        tmp_path,
        "app/src/checkout.rs",
        "pub use domain::payment::Exception as PaymentException;\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "public re-export alias policy" in result.stderr
    assert "app/src/checkout.rs" in result.stderr
    assert "PaymentException" in result.stderr


def test_new_grouped_public_reexport_alias_is_rejected(tmp_path: Path) -> None:
    write_rust(
        tmp_path,
        "domain/src/customer.rs",
        "pub use identity::{CustomerId as Id, MatchStatus as Status};\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "public re-export alias policy" in result.stderr
    assert "domain/src/customer.rs" in result.stderr
    assert "CustomerId as Id" in result.stderr
    assert "MatchStatus as Status" in result.stderr


def test_exact_public_reexport_alias_with_ownership_rationale_is_allowed(
    tmp_path: Path,
) -> None:
    declaration = "pub use domain::payment::Exception as PaymentException;"
    write_rust(tmp_path, "app/src/checkout.rs", declaration + "\n")
    manifest = write_manifest(
        tmp_path,
        public_reexport_aliases={
            "allowlist": [
                {
                    "path": "app/src/checkout.rs",
                    "declaration": declaration,
                    "rationale": "The checkout workflow owns this caller-facing role name.",
                }
            ]
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr


def test_public_surface_allowlist_entries_require_nonempty_rationales(
    tmp_path: Path,
) -> None:
    declaration = "pub use domain::payment::Exception as PaymentException;"
    write_rust(tmp_path, "app/src/checkout.rs", declaration + "\n")
    manifest = write_manifest(
        tmp_path,
        public_reexport_aliases={
            "allowlist": [
                {
                    "path": "app/src/checkout.rs",
                    "declaration": declaration,
                    "rationale": "",
                }
            ]
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 2
    assert "rationale must be a non-empty string" in result.stderr


def test_new_manifest_dependency_cycle_is_rejected(tmp_path: Path) -> None:
    write_rust(tmp_path, "domain/src/lib.rs", "pub mod customer;\n")
    manifest = write_manifest(
        tmp_path,
        dependency_cycles={
            "completion_maximum": 0,
            "debt_ceiling": 0,
            "cycles": [
                {
                    "id": "fixture-cycle",
                    "title": "Fixture cycle",
                    "members": ["domain/src/a.rs", "domain/src/b.rs"],
                }
            ],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "dependency cycles" in result.stderr
    assert "1 > allowed 0" in result.stderr
    assert "fixture-cycle" in result.stderr


def test_live_entity_owner_dependency_cycle_is_rejected(tmp_path: Path) -> None:
    write_rust(tmp_path, "domain/src/entities/alpha.rs", "use super::beta::Beta;\n")
    write_rust(tmp_path, "domain/src/entities/beta.rs", "use super::alpha::Alpha;\n")
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "entity owner dependency cycle" in result.stderr
    assert "alpha -> beta -> alpha" in result.stderr


def test_qualified_entity_owner_type_path_cycle_is_rejected(tmp_path: Path) -> None:
    write_rust(
        tmp_path,
        "domain/src/entities/alpha.rs",
        "pub struct Alpha(Box<super::beta::Beta>);\n",
    )
    write_rust(
        tmp_path,
        "domain/src/entities/beta.rs",
        "pub struct Beta(Box<super::alpha::Alpha>);\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "entity owner dependency cycle" in result.stderr
    assert "alpha -> beta -> alpha" in result.stderr


def test_aliased_entity_owner_root_cycle_is_rejected(tmp_path: Path) -> None:
    write_rust(
        tmp_path,
        "domain/src/entities/alpha.rs",
        "use super as owners;\nuse owners::beta::Beta;\n",
    )
    write_rust(
        tmp_path,
        "domain/src/entities/beta.rs",
        "use super as owners;\nuse owners::alpha::Alpha;\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "entity owner dependency cycle" in result.stderr
    assert "alpha -> beta -> alpha" in result.stderr


@pytest.mark.parametrize(
    ("alpha_source", "beta_source"),
    [
        pytest.param(
            "use super::{beta::Beta};\n",
            "use super::{alpha::Alpha};\n",
            id="grouped-super",
        ),
        pytest.param(
            "use crate::entities::{beta::Beta};\n",
            "use crate::entities::{alpha::Alpha};\n",
            id="grouped-crate-entities",
        ),
        pytest.param(
            "use crate::{entities::beta::Beta};\n",
            "use crate::{entities::alpha::Alpha};\n",
            id="grouped-crate-root",
        ),
        pytest.param(
            "use crate::entities::{self as owners};\nuse owners::beta::Beta;\n",
            "use crate::entities::{self as owners};\nuse owners::alpha::Alpha;\n",
            id="grouped-owner-root-alias",
        ),
    ],
)
def test_grouped_entity_owner_dependency_cycle_is_rejected(
    tmp_path: Path,
    alpha_source: str,
    beta_source: str,
) -> None:
    write_rust(tmp_path, "domain/src/entities/alpha.rs", alpha_source)
    write_rust(tmp_path, "domain/src/entities/beta.rs", beta_source)
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "entity owner dependency cycle" in result.stderr
    assert "alpha -> beta -> alpha" in result.stderr


def test_grouped_entity_owner_paths_in_comments_and_literals_are_ignored(
    tmp_path: Path,
) -> None:
    decoys = """
// use super::{other::Other};
/* use crate::entities::{other::Other}; */
const TEXT: &str = "use crate::{entities::other::Other};";
const BYTE_TEXT: &[u8] = b"use super::{other::Other};";
const RAW: &str = r###"use crate::entities::{self as owners}; use owners::other::Other;"###;
const BYTE_RAW: &[u8] = br##"use super::{other::Other};"##;
const MANY_HASH_RAW: &str = r#################"quoted " use super::{other::Other};"#################;
const CHARACTER: char = '{';
const BYTE_CHARACTER: u8 = b'{';
"""
    write_rust(
        tmp_path, "domain/src/entities/alpha.rs", decoys.replace("other", "beta")
    )
    write_rust(
        tmp_path, "domain/src/entities/beta.rs", decoys.replace("other", "alpha")
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr
    assert "dependency_cycles=0" in result.stdout


def test_grouped_entity_owner_paths_in_nested_block_comments_are_ignored(
    tmp_path: Path,
) -> None:
    alpha_source = """
/* outer comment
   /* inner comment */
   use super::{beta::Beta};
*/
pub struct Alpha;
"""
    beta_source = """
/* outer comment
   /* inner comment */
   use super::{alpha::Alpha};
*/
pub struct Beta;
"""
    write_rust(tmp_path, "domain/src/entities/alpha.rs", alpha_source)
    write_rust(tmp_path, "domain/src/entities/beta.rs", beta_source)
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr
    assert "dependency_cycles=0" in result.stdout


def test_domain_manifest_rejects_storage_and_runtime_dependencies(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "domain/src/lib.rs", "pub struct Customer;\n")
    (tmp_path / "domain" / "Cargo.toml").write_text(
        """
[package]
name = "domain"
version = "0.1.0"

[dependencies]
storage = { path = "../storage" }
pet-resort-api = { path = "../apps/api" }
""",
        encoding="utf-8",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "bounded-context dependency" in result.stderr
    assert "domain/Cargo.toml" in result.stderr
    assert "pet-resort-api" in result.stderr
    assert "storage" in result.stderr


def test_provider_dto_import_outside_provider_adapter_is_rejected(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "domain/src/customer.rs",
        "use gingr::dto::OwnerRecord;\npub struct Customer(OwnerRecord);\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "provider DTO quarantine" in result.stderr
    assert "domain/src/customer.rs" in result.stderr
    assert "gingr::dto" in result.stderr


def test_provider_vocabulary_in_public_domain_rustdoc_is_rejected(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "domain/src/source.rs",
        "/// Identifier copied from Gingr reservation records.\npub struct ReservationTypeId;\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "provider vocabulary quarantine" in result.stderr
    assert "public Rustdoc" in result.stderr
    assert "domain/src/source.rs" in result.stderr


def test_provider_model_path_in_executable_literal_is_rejected(tmp_path: Path) -> None:
    write_rust(
        tmp_path,
        "app/src/trace.rs",
        'pub fn source_model_path() -> &\'static str { "gingr::response::ReservationRecord" }\n',
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "provider DTO quarantine" in result.stderr
    assert "executable literal" in result.stderr
    assert "app/src/trace.rs" in result.stderr


@pytest.mark.parametrize(
    "source",
    [
        pytest.param(
            "extern crate provider as source;\nuse source::{dto::OwnerRecord};\n",
            id="extern-crate-alias-grouped-use",
        ),
        pytest.param(
            "use provider as source;\nuse source::{response::HttpStatus};\n",
            id="use-alias-grouped-use",
        ),
        pytest.param(
            "use provider::{dto::{self, OwnerRecord}};\n",
            id="renamed-dependency-grouped-use",
        ),
    ],
)
def test_aliased_provider_dto_import_outside_provider_adapter_is_rejected(
    tmp_path: Path,
    source: str,
) -> None:
    write_rust(tmp_path, "app/src/lib.rs", source)
    (tmp_path / "app" / "Cargo.toml").write_text(
        """
[package]
name = "app"
version = "0.1.0"

[dependencies]
provider = { package = "gingr", path = "../integrations/gingr" }
""",
        encoding="utf-8",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "provider DTO quarantine" in result.stderr
    assert "app/Cargo.toml" in result.stderr
    assert "gingr" in result.stderr


@pytest.mark.parametrize(
    "dependency_section",
    [
        pytest.param("[dependencies]", id="ordinary-workspace-inheritance"),
        pytest.param(
            "[target.'cfg(unix)'.dependencies]",
            id="target-specific-workspace-inheritance",
        ),
    ],
)
def test_workspace_inherited_provider_alias_outside_adapter_is_rejected(
    tmp_path: Path,
    dependency_section: str,
) -> None:
    write_rust(
        tmp_path,
        "app/src/lib.rs",
        "use provider::dto::OwnerRecord;\npub struct Customer(OwnerRecord);\n",
    )
    (tmp_path / "Cargo.toml").write_text(
        """
[workspace]
members = ["app", "integrations/gingr"]

[workspace.dependencies]
provider = { package = "gingr", path = "integrations/gingr" }
""",
        encoding="utf-8",
    )
    (tmp_path / "app" / "Cargo.toml").write_text(
        f"""
[package]
name = "app"
version = "0.1.0"

{dependency_section}
provider.workspace = true
""",
        encoding="utf-8",
    )
    (tmp_path / "integrations" / "gingr").mkdir(parents=True, exist_ok=True)
    (tmp_path / "integrations" / "gingr" / "Cargo.toml").write_text(
        '[package]\nname = "gingr"\nversion = "0.1.0"\n',
        encoding="utf-8",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "provider DTO quarantine" in result.stderr
    assert "app/Cargo.toml" in result.stderr
    assert "provider" in result.stderr


def test_provider_adapter_owner_remains_legal_with_workspace_provider_alias(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "integrations/gingr/src/lib.rs",
        "pub mod dto { pub struct OwnerRecord; }\nuse crate::dto::OwnerRecord;\npub struct Adapter(OwnerRecord);\n",
    )
    (tmp_path / "Cargo.toml").write_text(
        """
[workspace]
members = ["integrations/gingr"]

[workspace.dependencies]
provider = { package = "gingr", path = "integrations/gingr" }
""",
        encoding="utf-8",
    )
    (tmp_path / "integrations" / "gingr" / "Cargo.toml").write_text(
        '[package]\nname = "gingr"\nversion = "0.1.0"\n',
        encoding="utf-8",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr
    assert "dependency_boundary_violations=0" in result.stdout


def test_api_and_worker_require_app_contracts_without_shell_coupling(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "apps/api/src/lib.rs", "pub struct Api;\n")
    write_rust(tmp_path, "apps/worker/src/lib.rs", "pub struct Worker;\n")
    for crate, name, dependencies in (
        ("api", "pet-resort-api", 'pet-resort-worker = { path = "../worker" }\n'),
        ("worker", "pet-resort-worker", 'domain = { path = "../../domain" }\n'),
    ):
        (tmp_path / "apps" / crate / "Cargo.toml").write_text(
            f'[package]\nname = "{name}"\nversion = "0.1.0"\n\n[dependencies]\n{dependencies}',
            encoding="utf-8",
        )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "runtime application contract" in result.stderr
    assert "apps/api/Cargo.toml" in result.stderr
    assert "apps/worker/Cargo.toml" in result.stderr
    assert "runtime shell dependency" in result.stderr


def test_compatibility_module_cannot_become_a_general_dependency(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "app/src/workflow.rs",
        "use storage::operations::compatibility::LegacyOutcome;\npub struct Workflow(LegacyOutcome);\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "compatibility dependency isolation" in result.stderr
    assert "app/src/workflow.rs" in result.stderr


@pytest.mark.parametrize(
    ("relative", "source", "expected_category"),
    [
        (
            "domain/src/source.rs",
            "pub enum System { Gingr, Telephony }\n",
            "provider vocabulary quarantine",
        ),
        (
            "apps/api/src/public_contract.rs",
            "pub struct WireRequest { pub outcome: storage::operations::OutcomeCode }\n",
            "API semantic ownership",
        ),
        (
            "domain/src/grooming.rs",
            "pub enum LegacyBookingStatus { Completed }\n",
            "compatibility leakage",
        ),
    ],
)
def test_semantic_leak_fixtures_are_rejected(
    tmp_path: Path,
    relative: str,
    source: str,
    expected_category: str,
) -> None:
    write_rust(tmp_path, relative, source)
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert expected_category in result.stderr
    assert relative in result.stderr


def test_canonical_api_route_inventory_rejects_unversioned_product_alias(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "apps/api/src/http/router.rs",
        'pub fn router() { Router::new().route("/healthz", get(healthz)); }\n',
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "canonical API route inventory" in result.stderr
    assert "/healthz" in result.stderr


def test_active_architecture_docs_reject_migration_compatibility_contracts(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "domain/src/lib.rs", "pub struct Domain;\n")
    doc = tmp_path / "docs" / "architecture" / "lifecycle-rehydration-boundaries.md"
    doc.parent.mkdir(parents=True, exist_ok=True)
    doc.write_text("## Intentionally tolerated legacy states\n", encoding="utf-8")
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "active architecture compatibility contract" in result.stderr
    assert "lifecycle-rehydration-boundaries.md" in result.stderr


def test_active_demo_docs_reject_noncanonical_route_versions(tmp_path: Path) -> None:
    write_rust(tmp_path, "domain/src/lib.rs", "pub struct Domain;\n")
    doc = tmp_path / "docs" / "demo" / "walkthrough.md"
    doc.parent.mkdir(parents=True, exist_ok=True)
    doc.write_text("Call the public `/v0/ops/report` route.\n", encoding="utf-8")
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "active documentation route version" in result.stderr
    assert "docs/demo/walkthrough.md" in result.stderr
    assert "/v0" in result.stderr


def test_active_architecture_docs_reject_declared_repository_paths_that_do_not_exist(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "domain/src/lib.rs", "pub struct Domain;\n")
    doc = tmp_path / "docs" / "architecture" / "bounded-context-map.md"
    doc.parent.mkdir(parents=True, exist_ok=True)
    doc.write_text(
        "Compatibility is owned by `storage/src/operations/translation/`.\n",
        encoding="utf-8",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "active documentation declared path" in result.stderr
    assert "storage/src/operations/translation/" in result.stderr


@pytest.mark.parametrize(
    ("relative", "text", "expected"),
    [
        pytest.param(
            "docs/internal/reviews/current-review.md",
            "The current owner remains `domain/src/strategic_ai_ops_bridge.rs`.\n",
            "domain/src/strategic_ai_ops_bridge.rs",
            id="deleted-path-presented-as-current-outside-architecture",
        ),
        pytest.param(
            "docs/plans/current-plan.md",
            "The deprecated strategic_ai_ops_bridge facade remains for compatibility.\n",
            "strategic_ai_ops_bridge",
            id="deleted-facade-said-to-remain",
        ),
        pytest.param(
            "docs/domain/current-owner.md",
            "Current ownership is `domain::service::boarding`, proved by `domain/tests/service_module_architecture.rs`.\n",
            "domain::service",
            id="nonexistent-owner-and-test-path",
        ),
    ],
)
def test_all_active_docs_reject_deleted_or_nonexistent_current_contracts(
    tmp_path: Path, relative: str, text: str, expected: str
) -> None:
    write_rust(tmp_path, "domain/src/lib.rs", "pub struct Domain;\n")
    doc = tmp_path / relative
    doc.parent.mkdir(parents=True, exist_ok=True)
    doc.write_text(text, encoding="utf-8")
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "active documentation" in result.stderr
    assert relative in result.stderr
    assert expected in result.stderr


def test_active_demo_docs_reject_markdown_links_to_repository_paths_that_do_not_exist(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "domain/src/lib.rs", "pub struct Domain;\n")
    doc = tmp_path / "docs" / "demo" / "walkthrough.md"
    doc.parent.mkdir(parents=True, exist_ok=True)
    doc.write_text(
        "Inspect [the current schema](../../migrations/0002_removed.sql).\n",
        encoding="utf-8",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "active documentation declared path" in result.stderr
    assert "migrations/0002_removed.sql" in result.stderr


def test_public_family_used_only_by_tests_is_rejected_even_without_compatibility_words(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "storage/src/outcomes.rs",
        "pub struct RetentionCorrelationRow;\nimpl RetentionCorrelationRow { pub fn decode(raw: &str) -> Self { let _ = raw; Self } }\n",
    )
    test = tmp_path / "storage" / "tests" / "outcomes.rs"
    test.parent.mkdir(parents=True, exist_ok=True)
    test.write_text(
        "use storage::RetentionCorrelationRow;\n",
        encoding="utf-8",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "consumerless public API" in result.stderr
    assert "RetentionCorrelationRow" in result.stderr
    assert "storage/src/outcomes.rs" in result.stderr


def test_dormant_public_action_variants_are_rejected_under_semantic_renaming(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "app/src/workflow.rs",
        """
pub enum SafeAction {
    SummarizeEvidence,
    OpenStaffWorkItem,
    PreparePatronDraft,
}

pub fn safe_actions() -> Vec<SafeAction> {
    vec![SafeAction::SummarizeEvidence]
}
""",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "dormant public enum variant" in result.stderr
    assert "OpenStaffWorkItem" in result.stderr
    assert "PreparePatronDraft" in result.stderr


def test_custom_deserializer_cannot_discard_untyped_claim_fields_and_normalize_them(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "app/src/workflow.rs",
        """
use serde::Deserialize;

pub struct Packet { decision: bool }

#[derive(Deserialize)]
struct WirePacket { decision: serde_json::Value }

impl<'de> Deserialize<'de> for Packet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>
    {
        let wire = WirePacket::deserialize(deserializer)?;
        let _reported = wire.decision;
        Ok(Self { decision: false })
    }
}
""",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "untyped serialized claim normalization" in result.stderr
    assert "app/src/workflow.rs" in result.stderr


def test_public_api_inventory_rejects_stale_contract_and_variant_entries(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "domain/src/lib.rs", "pub struct CurrentPacket;\n")
    inventory = tmp_path / "docs" / "quality" / "public-api-inventory.json"
    inventory.parent.mkdir(parents=True, exist_ok=True)
    inventory.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "externally_consumed_contracts": [
                    {
                        "path": "domain/src/lib.rs",
                        "name": "RemovedPacket",
                        "rationale": "external contract",
                    }
                ],
                "externally_consumed_variants": [
                    {
                        "path": "domain/src/lib.rs",
                        "enum": "RemovedAction",
                        "variant": "RemovedVariant",
                        "rationale": "external variant",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "stale public contract `RemovedPacket`" in result.stderr
    assert "stale public variant `RemovedAction::RemovedVariant`" in result.stderr


def test_public_api_inventory_rejects_declaration_only_exact_symbol_evidence(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "domain/src/lib.rs", "pub struct CurrentPacket;\n")
    inventory = tmp_path / "docs" / "quality" / "public-api-inventory.json"
    inventory.parent.mkdir(parents=True, exist_ok=True)
    inventory.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "external_contract_symbols": [
                    {
                        "symbol": "domain::CurrentPacket",
                        "rationale": "Exact downstream-facing packet",
                        "evidence": "Public Rustdoc contract and declaration at domain/src/lib.rs",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "public API inventory" in result.stderr
    assert "declaration" in result.stderr.lower()
    assert "CurrentPacket" in result.stderr


def test_same_leaf_public_type_in_an_unrelated_module_is_not_a_consumer(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "domain/src/review.rs", "pub struct ReviewPacket;\n")
    write_rust(
        tmp_path,
        "app/src/unrelated.rs",
        "pub struct ReviewPacket;\nfn consume(_: ReviewPacket) {}\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "consumerless public API" in result.stderr
    assert "domain/src/review.rs" in result.stderr
    assert "ReviewPacket" in result.stderr


def test_qualified_and_renamed_public_type_references_are_concrete_consumers(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "domain/src/review.rs", "pub struct ReviewPacket;\n")
    write_rust(
        tmp_path,
        "app/src/consumer.rs",
        "use domain::review::ReviewPacket as Packet;\n"
        "fn consume_qualified(_: domain::review::ReviewPacket) {}\n"
        "fn consume_renamed(_: Packet) {}\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr


def test_same_module_type_reference_is_a_canonical_local_consumer(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "domain/src/review.rs",
        "pub struct ReviewPacket;\nfn consume(_: ReviewPacket) {}\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr


def test_inherent_impl_header_does_not_make_a_public_type_consumed(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "domain/src/review.rs",
        "pub struct ReviewPacket;\n"
        "impl ReviewPacket { pub fn new() -> Self { Self } }\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "consumerless public API" in result.stderr
    assert "ReviewPacket" in result.stderr


def test_reexported_public_owner_resolves_to_the_canonical_declaration(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "domain/src/internal.rs", "pub struct ReviewPacket;\n")
    write_rust(
        tmp_path,
        "domain/src/review.rs",
        "pub use crate::internal::ReviewPacket;\n",
    )
    write_rust(
        tmp_path,
        "app/src/consumer.rs",
        "fn consume(_: domain::review::ReviewPacket) {}\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr


def test_same_leaf_variants_on_an_unrelated_enum_do_not_hide_dormant_variants(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "domain/src/review.rs",
        "pub enum ReviewAction { Queue, Hold }\n",
    )
    write_rust(
        tmp_path,
        "app/src/unrelated.rs",
        "enum OtherAction { Queue, Hold }\n"
        "fn consume() { let _ = OtherAction::Queue; let _ = OtherAction::Hold; }\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "dormant public enum variant" in result.stderr
    assert "ReviewAction::Queue" in result.stderr
    assert "ReviewAction::Hold" in result.stderr


def test_qualified_public_variants_are_not_reported_as_dormant(tmp_path: Path) -> None:
    write_rust(
        tmp_path,
        "domain/src/review.rs",
        "pub enum ReviewAction { Queue, Hold }\n",
    )
    write_rust(
        tmp_path,
        "app/src/consumer.rs",
        "fn consume() {"
        " let _ = domain::review::ReviewAction::Queue;"
        " let _ = domain::review::ReviewAction::Hold;"
        " }\n",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr


def test_app_manifest_rejects_storage_semantic_dependency(tmp_path: Path) -> None:
    write_rust(tmp_path, "app/src/lib.rs", "pub struct App;\n")
    (tmp_path / "app" / "Cargo.toml").write_text(
        '[package]\nname="app"\nversion="0.1.0"\n[dependencies]\nstorage={path="../storage"}\n',
        encoding="utf-8",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "bounded-context dependency" in result.stderr
    assert "app/Cargo.toml" in result.stderr


def test_new_authority_type_and_issuer_require_exact_inventory(tmp_path: Path) -> None:
    write_rust(
        tmp_path,
        "app/src/authorization.rs",
        """
pub struct AdminAuthority {
    token: String,
}

pub fn issue_admin_authority() -> AdminAuthority {
    AdminAuthority { token: String::new() }
}
""",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "authority capability inventory" in result.stderr
    assert "AdminAuthority" in result.stderr
    assert "authority issuer inventory" in result.stderr
    assert "issue_admin_authority" in result.stderr


@pytest.mark.parametrize(
    "authority_source",
    [
        pytest.param(
            "#[derive(serde::Serialize)]\npub struct QueueAuthority { token: String }\n",
            id="serializable",
        ),
        pytest.param(
            "pub struct QueueAuthority { pub token: String }\n",
            id="public-field",
        ),
    ],
)
def test_inventoried_authority_must_remain_opaque_and_nonserializable(
    tmp_path: Path,
    authority_source: str,
) -> None:
    path = "domain/src/queue.rs"
    write_rust(tmp_path, path, authority_source)
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": "QueueAuthority",
                    "rationale": "Exact queue admission capability.",
                }
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "authority opacity" in result.stderr
    assert "QueueAuthority" in result.stderr


@pytest.mark.parametrize(
    "authority_source",
    [
        pytest.param(
            "#[derive(serde::Serialize)]\npub struct QueueAuthority(pub String);\n",
            id="serializable-tuple",
        ),
        pytest.param(
            "pub struct QueueAuthority(pub String);\n",
            id="public-tuple-field",
        ),
        pytest.param(
            "pub struct QueueAuthority(pub (String, fn()));\n",
            id="nested-public-tuple-field",
        ),
        pytest.param(
            "pub struct QueueAuthority<T>(pub T);\n",
            id="generic-public-tuple-field",
        ),
        pytest.param(
            "pub struct QueueAuthority { pub token: [u8; { 1 }] }\n",
            id="nested-brace-field-type",
        ),
        pytest.param("pub struct QueueAuthority;\n", id="public-unit"),
    ],
)
def test_tuple_and_unit_authority_construction_bypasses_are_rejected(
    tmp_path: Path,
    authority_source: str,
) -> None:
    write_rust(tmp_path, "domain/src/queue.rs", authority_source)
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority opacity" in result.stderr
    assert "QueueAuthority" in result.stderr


@pytest.mark.parametrize(
    "return_type, body",
    [
        pytest.param(
            "Result<Self, Error>",
            "unimplemented!()",
            id="inherent-self-result",
        ),
        pytest.param(
            "CurrentReviewerCapability",
            "Self { token: String::new() }",
            id="equivalent-explicit-type",
        ),
    ],
)
def test_capability_family_and_inherent_issuer_require_exact_inventory(
    tmp_path: Path,
    return_type: str,
    body: str,
) -> None:
    write_rust(
        tmp_path,
        "storage/src/reviewer.rs",
        f"""
pub struct CurrentReviewerCapability {{
    token: String,
}}

impl CurrentReviewerCapability {{
    pub(crate) fn try_new() -> {return_type} {{
        {body}
    }}
}}
""",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority capability inventory" in result.stderr
    assert "CurrentReviewerCapability" in result.stderr
    assert "authority issuer inventory" in result.stderr
    assert "try_new" in result.stderr


@pytest.mark.parametrize(
    "authority_source",
    [
        pytest.param("struct QueueContactAuthority;\n", id="private-unit"),
        pytest.param("struct QueueContactAuthority(String);\n", id="private-tuple"),
        pytest.param(
            """
pub struct ScopedAuthority<T>
where
    T: Clone,
{
    token: T,
}
""",
            id="where-qualified-brace",
        ),
        pytest.param(
            """
struct ScopedAuthority<T>
where
    T: Clone;
""",
            id="where-qualified-unit",
        ),
    ],
)
def test_private_and_where_qualified_authority_forms_require_exact_inventory(
    tmp_path: Path,
    authority_source: str,
) -> None:
    write_rust(tmp_path, "domain/src/queue.rs", authority_source)
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority capability inventory" in result.stderr
    assert "Authority" in result.stderr


@pytest.mark.parametrize(
    "impl_owner",
    [
        pytest.param("GenericAuthority<T>", id="generic-owner"),
        pytest.param("crate::queue::GenericAuthority<T>", id="qualified-generic-owner"),
    ],
)
def test_generic_inherent_self_issuer_requires_exact_inventory(
    tmp_path: Path,
    impl_owner: str,
) -> None:
    path = "domain/src/queue.rs"
    write_rust(
        tmp_path,
        path,
        """
pub struct GenericAuthority<T> {
    token: T,
}

impl<T> %s
where
    T: Default,
{
    pub(crate) fn issue() -> Self {
        Self { token: T::default() }
    }
}
"""
        % impl_owner,
    )
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": "GenericAuthority",
                    "rationale": "Generic exact-operation proof token.",
                }
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority issuer inventory" in result.stderr
    assert "issue" in result.stderr
    assert "GenericAuthority" in result.stderr


@pytest.mark.parametrize(
    "method_signature",
    [
        pytest.param("fn issue<U>() -> Self", id="generic-method"),
        pytest.param(
            "fn issue(_: Option<(u8, u8)>) -> Self",
            id="nested-parameter",
        ),
    ],
)
def test_generic_and_nested_parameter_inherent_issuers_require_exact_inventory(
    tmp_path: Path,
    method_signature: str,
) -> None:
    path = "domain/src/queue.rs"
    write_rust(
        tmp_path,
        path,
        f"""
pub struct GenericAuthority<T> {{
    token: T,
}}

impl<T: Default> GenericAuthority<T> {{
    pub(crate) {method_signature} {{
        Self {{ token: T::default() }}
    }}
}}
""",
    )
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": "GenericAuthority",
                    "rationale": "Generic exact-operation proof token.",
                }
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority issuer inventory" in result.stderr
    assert "issue" in result.stderr
    assert "GenericAuthority" in result.stderr


@pytest.mark.parametrize(
    "authority_source, function",
    [
        pytest.param(
            """
pub struct GenericAuthority<T> { token: T }

impl<T: Default> crate::queue::GenericAuthority<T> {
    fn issue_nested<U>(
        _: Option<Result<(u8, U), Vec<Vec<U>>>>,
    ) -> crate::queue::GenericAuthority<T>
    where
        U: Default,
    {
        Self { token: T::default() }
    }
}
""",
            "issue_nested",
            id="qualified-owner-explicit-return-nested-generics-where",
        ),
        pytest.param(
            """
pub struct GenericAuthority<T> { token: T }

fn issue_free<U>() -> GenericAuthority<U>
where
    U: Default,
{
    GenericAuthority { token: U::default() }
}
""",
            "issue_free",
            id="free-generic-function-with-where-clause",
        ),
    ],
)
def test_adversarial_authority_issuer_signatures_require_exact_inventory(
    tmp_path: Path,
    authority_source: str,
    function: str,
) -> None:
    path = "domain/src/queue.rs"
    write_rust(tmp_path, path, authority_source)
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": "GenericAuthority",
                    "rationale": "Generic exact-operation proof token.",
                }
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority issuer inventory" in result.stderr
    assert function in result.stderr
    assert "GenericAuthority" in result.stderr


def test_balanced_function_scanning_does_not_invent_authority_issuers(
    tmp_path: Path,
) -> None:
    path = "domain/src/queue.rs"
    write_rust(
        tmp_path,
        path,
        """
pub struct GenericAuthority<T> { token: T }

impl<T> GenericAuthority<T> {
    fn inspect<U>(_: Option<Result<(u8, U), Vec<Vec<U>>>>) -> Option<&T>
    where
        U: Default,
    {
        None
    }
}

fn ordinary<U>(_: Option<(U, U)>) -> Result<(), U> {
    Ok(())
}
""",
    )
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": "GenericAuthority",
                    "rationale": "Generic exact-operation proof token.",
                }
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr
    assert "authority_capabilities=1 authority_issuers=0" in result.stdout


@pytest.mark.parametrize(
    "authority_source, function, authority",
    [
        pytest.param(
            """
struct QueueAuthority { token: u8 }
type Issued = QueueAuthority;

fn issue_free() -> Issued { QueueAuthority { token: 0 } }
""",
            "issue_free",
            "QueueAuthority",
            id="free-direct-alias",
        ),
        pytest.param(
            """
struct QueueAuthority { token: u8 }
type Issued = QueueAuthority;

impl QueueAuthority {
    fn issue_inherent() -> Issued { Self { token: 0 } }
}
""",
            "issue_inherent",
            "QueueAuthority",
            id="inherent-direct-alias",
        ),
        pytest.param(
            """
struct GenericAuthority<T> { token: T }
type Base<T> = crate::queue::GenericAuthority<T>;
type Issued<T> = Base<T>;

fn issue_chain() -> crate::queue::Issued<u8> { GenericAuthority { token: 0 } }
""",
            "issue_chain",
            "GenericAuthority",
            id="free-qualified-generic-alias-chain",
        ),
        pytest.param(
            """
struct GenericAuthority<T> { token: T }
type Base<T> = self::GenericAuthority<T>;
type Issued<T> = crate::queue::Base<T>;

impl<T: Default> crate::queue::GenericAuthority<T> {
    fn issue_chain() -> crate::queue::Issued<T> { Self { token: T::default() } }
}
""",
            "issue_chain",
            "GenericAuthority",
            id="inherent-qualified-generic-alias-chain",
        ),
    ],
)
def test_authority_return_aliases_require_exact_issuer_inventory(
    tmp_path: Path,
    authority_source: str,
    function: str,
    authority: str,
) -> None:
    path = "domain/src/queue.rs"
    write_rust(tmp_path, path, authority_source)
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": authority,
                    "rationale": "Opaque exact-operation proof token.",
                }
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority issuer inventory" in result.stderr
    assert function in result.stderr
    assert authority in result.stderr


def test_alias_with_multiple_authority_owners_inventories_every_possible_issuer(
    tmp_path: Path,
) -> None:
    path = "domain/src/queue.rs"
    write_rust(
        tmp_path,
        path,
        """
struct QueueAuthority { token: u8 }
struct RetryAuthorization { token: u8 }
type Issued = Result<QueueAuthority, RetryAuthorization>;
fn issue_either() -> Issued { Ok(QueueAuthority { token: 0 }) }
""",
    )
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": "QueueAuthority",
                    "rationale": "Opaque queue proof token.",
                },
                {
                    "path": path,
                    "name": "RetryAuthorization",
                    "rationale": "Opaque retry proof token.",
                },
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert result.stderr.count("undocumented issuer `issue_either`") == 2
    assert "QueueAuthority" in result.stderr
    assert "RetryAuthorization" in result.stderr


@pytest.mark.parametrize(
    "manufacture, function",
    [
        pytest.param(
            "fn issue_policy() -> Policy { RoleLocationAuthorization }",
            "issue_policy",
            id="free-alias-return",
        ),
        pytest.param(
            (
                "impl RoleLocationAuthorization { "
                "fn issue_policy() -> Policy { RoleLocationAuthorization } "
                "}"
            ),
            "issue_policy",
            id="inherent-alias-return",
        ),
    ],
)
def test_policy_only_classification_rejects_alias_return_manufacture(
    tmp_path: Path,
    manufacture: str,
    function: str,
) -> None:
    path = "app/src/authorization.rs"
    write_rust(
        tmp_path,
        path,
        f"""
trait AuthorizationPolicy {{}}
struct RoleLocationAuthorization;
impl AuthorizationPolicy for RoleLocationAuthorization {{}}
type Policy = RoleLocationAuthorization;
{manufacture}
""",
    )
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "policy_only_types": [
                {
                    "path": path,
                    "name": "RoleLocationAuthorization",
                    "trait": "AuthorizationPolicy",
                    "rationale": "Stateless policy with no manufacture path.",
                }
            ],
            "types": [],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority policy-only classification" in result.stderr
    assert "must not have manufacture functions" in result.stderr
    assert function in result.stderr


@pytest.mark.parametrize(
    "queue_source, function",
    [
        pytest.param(
            """
use crate::aliases::Issued as QueueTicket;

pub(crate) struct QueueAuthority<T> { token: T }

fn issue_free() -> QueueTicket<u8> { QueueAuthority { token: 0 } }
""",
            "issue_free",
            id="free-imported-generic-alias-chain",
        ),
        pytest.param(
            """
pub(crate) struct QueueAuthority<T> { token: T }

impl<T: Default> QueueAuthority<T> {
    fn issue_inherent() -> crate::aliases::Issued<T> { Self { token: T::default() } }
}
""",
            "issue_inherent",
            id="inherent-qualified-generic-alias-chain",
        ),
    ],
)
def test_cross_module_authority_alias_returns_require_exact_issuer_inventory(
    tmp_path: Path,
    queue_source: str,
    function: str,
) -> None:
    write_rust_2024_crate(
        tmp_path,
        {
            "aliases": """
pub(crate) type Base<T> = crate::queue::QueueAuthority<T>;
pub(crate) type Issued<T> = Base<T>;
""",
            "queue": queue_source,
        },
    )
    compile_result = run_cargo_check(tmp_path)
    assert compile_result.returncode == 0, compile_result.stderr
    path = "src/queue.rs"
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": "QueueAuthority",
                    "rationale": "Opaque exact-operation queue proof token.",
                }
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority issuer inventory" in result.stderr
    assert function in result.stderr
    assert "QueueAuthority" in result.stderr


@pytest.mark.parametrize(
    "authorization_source, function",
    [
        pytest.param(
            """
use crate::aliases::Policy as ReviewedPolicy;

pub(crate) trait AuthorizationPolicy {}
pub(crate) struct RoleLocationAuthorization;
impl AuthorizationPolicy for RoleLocationAuthorization {}

fn issue_policy_free() -> ReviewedPolicy { RoleLocationAuthorization }
""",
            "issue_policy_free",
            id="free-imported-alias-chain",
        ),
        pytest.param(
            """
pub(crate) trait AuthorizationPolicy {}
pub(crate) struct RoleLocationAuthorization;
impl AuthorizationPolicy for RoleLocationAuthorization {}

impl RoleLocationAuthorization {
    fn issue_policy_inherent() -> crate::aliases::Policy { RoleLocationAuthorization }
}
""",
            "issue_policy_inherent",
            id="inherent-qualified-alias-chain",
        ),
    ],
)
def test_policy_only_classification_rejects_cross_module_alias_manufacture(
    tmp_path: Path,
    authorization_source: str,
    function: str,
) -> None:
    write_rust_2024_crate(
        tmp_path,
        {
            "aliases": """
pub(crate) type Base = crate::authorization::RoleLocationAuthorization;
pub(crate) type Policy = Base;
""",
            "authorization": authorization_source,
        },
    )
    compile_result = run_cargo_check(tmp_path)
    assert compile_result.returncode == 0, compile_result.stderr
    path = "src/authorization.rs"
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "policy_only_types": [
                {
                    "path": path,
                    "name": "RoleLocationAuthorization",
                    "trait": "AuthorizationPolicy",
                    "rationale": "Stateless policy with no manufacture path.",
                }
            ],
            "types": [],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority policy-only classification" in result.stderr
    assert "must not have manufacture functions" in result.stderr
    assert function in result.stderr


def test_cross_module_non_authority_alias_return_does_not_invent_an_issuer(
    tmp_path: Path,
) -> None:
    write_rust_2024_crate(
        tmp_path,
        {
            "aliases": "pub(crate) type Summary<T> = Vec<T>;\n",
            "queue": """
use crate::aliases::Summary;

pub(crate) struct QueueAuthority { token: u8 }

fn summarize() -> Summary<String> { Vec::new() }
""",
        },
    )
    compile_result = run_cargo_check(tmp_path)
    assert compile_result.returncode == 0, compile_result.stderr
    path = "src/queue.rs"
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": "QueueAuthority",
                    "rationale": "Opaque exact-operation proof token.",
                }
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr
    assert "authority_capabilities=1 authority_issuers=0" in result.stdout


@pytest.mark.parametrize(
    "manufacture, function",
    [
        pytest.param(
            "fn issue_inline_free() -> crate::aliases::Issued<u8> { QueueAuthority { token: 0 } }",
            "issue_inline_free",
            id="free-qualified-generic-chain",
        ),
        pytest.param(
            """
impl<T: Default> QueueAuthority<T> {
    fn issue_inline_inherent() -> crate::aliases::Issued<T> {
        Self { token: T::default() }
    }
}
""",
            "issue_inline_inherent",
            id="inherent-qualified-generic-chain",
        ),
    ],
)
def test_inline_sibling_authority_alias_returns_require_exact_issuer_inventory(
    tmp_path: Path,
    manufacture: str,
    function: str,
) -> None:
    write_inline_rust_2024_crate(
        tmp_path,
        f"""
mod aliases {{
    pub(crate) type Base<T> = crate::queue::QueueAuthority<T>;
    pub(crate) type Issued<T> = Base<T>;
}}

mod queue {{
    pub(crate) struct QueueAuthority<T> {{ token: T }}
    {manufacture}
}}
""",
    )
    compile_result = run_cargo_check(tmp_path)
    assert compile_result.returncode == 0, compile_result.stderr
    path = "src/lib.rs"
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": "QueueAuthority",
                    "rationale": "Opaque exact-operation queue proof token.",
                }
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority issuer inventory" in result.stderr
    assert function in result.stderr
    assert "QueueAuthority" in result.stderr


@pytest.mark.parametrize(
    "manufacture, function",
    [
        pytest.param(
            "fn issue_inline_policy_free() -> crate::aliases::Policy { RoleLocationAuthorization }",
            "issue_inline_policy_free",
            id="free-qualified-chain",
        ),
        pytest.param(
            """
impl RoleLocationAuthorization {
    fn issue_inline_policy_inherent() -> crate::aliases::Policy {
        RoleLocationAuthorization
    }
}
""",
            "issue_inline_policy_inherent",
            id="inherent-qualified-chain",
        ),
    ],
)
def test_policy_only_classification_rejects_inline_sibling_alias_manufacture(
    tmp_path: Path,
    manufacture: str,
    function: str,
) -> None:
    write_inline_rust_2024_crate(
        tmp_path,
        f"""
mod aliases {{
    pub(crate) type Base = crate::authorization::RoleLocationAuthorization;
    pub(crate) type Policy = Base;
}}

mod authorization {{
    pub(crate) trait AuthorizationPolicy {{}}
    pub(crate) struct RoleLocationAuthorization;
    impl AuthorizationPolicy for RoleLocationAuthorization {{}}
    {manufacture}
}}
""",
    )
    compile_result = run_cargo_check(tmp_path)
    assert compile_result.returncode == 0, compile_result.stderr
    path = "src/lib.rs"
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "policy_only_types": [
                {
                    "path": path,
                    "name": "RoleLocationAuthorization",
                    "trait": "AuthorizationPolicy",
                    "rationale": "Stateless policy with no manufacture path.",
                }
            ],
            "types": [],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority policy-only classification" in result.stderr
    assert "must not have manufacture functions" in result.stderr
    assert function in result.stderr


def test_inline_sibling_non_authority_alias_return_does_not_invent_an_issuer(
    tmp_path: Path,
) -> None:
    write_inline_rust_2024_crate(
        tmp_path,
        """
mod aliases {
    pub(crate) type Summary<T> = Vec<T>;
}

mod queue {
    pub(crate) struct QueueAuthority { token: u8 }
    fn summarize() -> crate::aliases::Summary<String> { Vec::new() }
}
""",
    )
    compile_result = run_cargo_check(tmp_path)
    assert compile_result.returncode == 0, compile_result.stderr
    path = "src/lib.rs"
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": "QueueAuthority",
                    "rationale": "Opaque exact-operation proof token.",
                }
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr
    assert "authority_capabilities=1 authority_issuers=0" in result.stdout


@pytest.mark.parametrize(
    "import_declaration, manufacture, function",
    [
        pytest.param(
            "use crate::aliases::*;",
            "fn issue_glob() -> Issued<u8> { QueueAuthority { token: 0 } }",
            "issue_glob",
            id="glob-import-free-return",
        ),
        pytest.param(
            "use crate::aliases as proof;",
            """
impl<T: Default> QueueAuthority<T> {
    fn issue_module_alias() -> proof::Issued<T> {
        Self { token: T::default() }
    }
}
""",
            "issue_module_alias",
            id="module-alias-inherent-return",
        ),
    ],
)
def test_inline_imported_authority_alias_returns_require_exact_issuer_inventory(
    tmp_path: Path,
    import_declaration: str,
    manufacture: str,
    function: str,
) -> None:
    write_inline_rust_2024_crate(
        tmp_path,
        f"""
mod aliases {{
    pub(crate) type Issued<T> = crate::queue::QueueAuthority<T>;
}}

mod queue {{
    {import_declaration}
    pub(crate) struct QueueAuthority<T> {{ token: T }}
    {manufacture}
}}
""",
    )
    compile_result = run_cargo_check(tmp_path)
    assert compile_result.returncode == 0, compile_result.stderr
    path = "src/lib.rs"
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": "QueueAuthority",
                    "rationale": "Opaque exact-operation queue proof token.",
                }
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority issuer inventory" in result.stderr
    assert function in result.stderr
    assert "QueueAuthority" in result.stderr


@pytest.mark.parametrize(
    "import_declaration, manufacture, function",
    [
        pytest.param(
            "use crate::aliases::*;",
            "fn issue_glob_policy() -> Policy { RoleLocationAuthorization }",
            "issue_glob_policy",
            id="glob-import-free-return",
        ),
        pytest.param(
            "use crate::aliases as proof;",
            """
impl RoleLocationAuthorization {
    fn issue_module_policy() -> proof::Policy {
        RoleLocationAuthorization
    }
}
""",
            "issue_module_policy",
            id="module-alias-inherent-return",
        ),
    ],
)
def test_policy_only_classification_rejects_inline_imported_alias_manufacture(
    tmp_path: Path,
    import_declaration: str,
    manufacture: str,
    function: str,
) -> None:
    write_inline_rust_2024_crate(
        tmp_path,
        f"""
mod aliases {{
    pub(crate) type Policy = crate::authorization::RoleLocationAuthorization;
}}

mod authorization {{
    pub(crate) trait AuthorizationPolicy {{}}
    {import_declaration}
    pub(crate) struct RoleLocationAuthorization;
    impl AuthorizationPolicy for RoleLocationAuthorization {{}}
    {manufacture}
}}
""",
    )
    compile_result = run_cargo_check(tmp_path)
    assert compile_result.returncode == 0, compile_result.stderr
    path = "src/lib.rs"
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "policy_only_types": [
                {
                    "path": path,
                    "name": "RoleLocationAuthorization",
                    "trait": "AuthorizationPolicy",
                    "rationale": "Stateless policy with no manufacture path.",
                }
            ],
            "types": [],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority policy-only classification" in result.stderr
    assert "must not have manufacture functions" in result.stderr
    assert function in result.stderr


@pytest.mark.parametrize(
    "import_declaration, return_type",
    [
        pytest.param("use crate::aliases::*;", "Summary<String>", id="glob-import"),
        pytest.param(
            "use crate::aliases as proof;",
            "proof::Summary<String>",
            id="module-alias",
        ),
    ],
)
def test_inline_non_authority_imported_alias_return_does_not_invent_an_issuer(
    tmp_path: Path,
    import_declaration: str,
    return_type: str,
) -> None:
    write_inline_rust_2024_crate(
        tmp_path,
        f"""
mod aliases {{
    pub(crate) type Summary<T> = Vec<T>;
}}

mod queue {{
    {import_declaration}
    pub(crate) struct QueueAuthority {{ token: u8 }}
    fn summarize() -> {return_type} {{ Vec::new() }}
}}
""",
    )
    compile_result = run_cargo_check(tmp_path)
    assert compile_result.returncode == 0, compile_result.stderr
    path = "src/lib.rs"
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": "QueueAuthority",
                    "rationale": "Opaque exact-operation proof token.",
                }
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr
    assert "authority_capabilities=1 authority_issuers=0" in result.stdout


@pytest.mark.parametrize(
    "alias_declarations, return_type",
    [
        pytest.param(
            "type DelegatedAuthority = external::Token;",
            "DelegatedAuthority",
            id="unresolved-authority-shaped-alias",
        ),
        pytest.param(
            "type DelegatedAuthority = AliasLoop; type AliasLoop = DelegatedAuthority;",
            "AliasLoop",
            id="cyclic-authority-shaped-alias",
        ),
    ],
)
def test_authority_shaped_unresolved_alias_returns_fail_closed(
    tmp_path: Path,
    alias_declarations: str,
    return_type: str,
) -> None:
    path = "domain/src/queue.rs"
    write_rust(
        tmp_path,
        path,
        f"""
struct QueueAuthority {{ token: u8 }}
{alias_declarations}
fn issue_unknown() -> {return_type} {{ todo!() }}
""",
    )
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": "QueueAuthority",
                    "rationale": "Opaque exact-operation proof token.",
                }
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    assert "authority alias resolution" in result.stderr
    assert "issue_unknown" in result.stderr


def test_non_authority_return_alias_does_not_invent_an_issuer(tmp_path: Path) -> None:
    path = "domain/src/queue.rs"
    write_rust(
        tmp_path,
        path,
        """
struct QueueAuthority { token: u8 }
type Summary = Vec<String>;
fn summarize() -> Summary { Vec::new() }
""",
    )
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "types": [
                {
                    "path": path,
                    "name": "QueueAuthority",
                    "rationale": "Opaque exact-operation proof token.",
                }
            ],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr
    assert "authority_capabilities=1 authority_issuers=0" in result.stdout


def test_exact_passive_policy_classification_is_not_an_authority_capability(
    tmp_path: Path,
) -> None:
    path = "app/src/authorization.rs"
    write_rust(
        tmp_path,
        path,
        """
pub trait AuthorizationPolicy {}

pub struct RoleLocationAuthorization;

impl AuthorizationPolicy for RoleLocationAuthorization {}
""",
    )
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "policy_only_types": [
                {
                    "path": path,
                    "name": "RoleLocationAuthorization",
                    "trait": "AuthorizationPolicy",
                    "rationale": "Stateless decision policy; it carries and issues no capability.",
                }
            ],
            "types": [],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr
    assert "authority_capabilities=0 authority_issuers=0" in result.stdout


@pytest.mark.parametrize(
    "authority_declaration, manufacture, expected_findings",
    [
        pytest.param(
            "struct BypassAuthority { token: u8 }",
            "",
            ("authority policy-only classification", "authority capability inventory"),
            id="opaque-capability",
        ),
        pytest.param(
            "pub struct BypassAuthority { pub token: u8 }",
            "",
            ("authority policy-only classification", "authority opacity"),
            id="public-construction",
        ),
        pytest.param(
            "#[derive(serde::Serialize)] struct BypassAuthority { token: u8 }",
            "",
            ("authority policy-only classification", "authority opacity"),
            id="serialization",
        ),
        pytest.param(
            "struct BypassAuthority { token: u8 }",
            "impl BypassAuthority { fn issue() -> Self { Self { token: 0 } } }",
            (
                "authority policy-only classification",
                "authority issuer inventory",
                "issue",
            ),
            id="inherent-self-issuer",
        ),
        pytest.param(
            "struct BypassAuthority { token: u8 }",
            (
                "impl BypassAuthority { "
                "fn issue_explicit() -> BypassAuthority { BypassAuthority { token: 0 } } "
                "}"
            ),
            (
                "authority policy-only classification",
                "authority issuer inventory",
                "issue_explicit",
            ),
            id="inherent-explicit-issuer",
        ),
        pytest.param(
            "struct BypassAuthority { token: u8 }",
            "fn issue_free() -> BypassAuthority { BypassAuthority { token: 0 } }",
            (
                "authority policy-only classification",
                "authority issuer inventory",
                "issue_free",
            ),
            id="free-issuer",
        ),
    ],
)
def test_policy_trait_name_cannot_hide_capability_properties_or_issuers(
    tmp_path: Path,
    authority_declaration: str,
    manufacture: str,
    expected_findings: tuple[str, ...],
) -> None:
    path = "domain/src/queue.rs"
    write_rust(
        tmp_path,
        path,
        f"""
trait MarkerPolicy {{}}

{authority_declaration}

impl MarkerPolicy for BypassAuthority {{}}

{manufacture}
""",
    )
    manifest = write_manifest(
        tmp_path,
        authority_capabilities={
            "policy_only_types": [
                {
                    "path": path,
                    "name": "BypassAuthority",
                    "trait": "MarkerPolicy",
                    "rationale": "Adversarial attempt to classify an executable type as policy-only.",
                }
            ],
            "types": [],
            "issuers": [],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1, result.stdout
    for expected in expected_findings:
        assert expected in result.stderr


def test_recorded_debt_passes_at_its_exact_ceiling(tmp_path: Path) -> None:
    write_rust(
        tmp_path,
        "storage/src/operations.rs",
        "pub struct HistoricalItem;\npub use records::*;\n" + "// line\n" * 1_499,
    )
    manifest = write_manifest(
        tmp_path,
        production_rust={
            "completion_maximum_physical_lines": 1_500,
            "debt_ceiling_by_path": {"storage/src/operations.rs": 1_501},
        },
        compatibility_leakage={
            "completion_maximum": 0,
            "debt_ceiling_by_path": {"storage/src/operations.rs": 1},
        },
        broad_reexports={
            "completion_maximum": 0,
            "debt_ceiling_by_path": {},
            "allowlist": [
                {
                    "path": "storage/src/operations.rs",
                    "declaration": "pub use records::*;",
                    "rationale": "Test fixture for an intentionally retained compatibility surface.",
                }
            ],
        },
        public_reexport_aliases={"allowlist": []},
        dependency_cycles={
            "completion_maximum": 0,
            "debt_ceiling": 1,
            "cycles": [
                {
                    "id": "recorded-cycle",
                    "title": "Recorded cycle",
                    "members": ["storage/src/operations.rs"],
                }
            ],
        },
    )

    result = run_check(tmp_path, manifest)

    assert result.returncode == 0, result.stderr
    assert "oversized_files=1" in result.stdout
    assert "compatibility_leaks=1" in result.stdout
    assert "broad_reexports=1" in result.stdout
    assert "dependency_cycles=1" in result.stdout


@pytest.mark.parametrize(
    ("source", "evidence"),
    [
        ("pub fn mock_gingr_report() {}\n", "mock_gingr_report"),
        (
            'pub const SOURCE: &str = "fixture://mock-gingr/report.json";\n',
            "fixture://mock-gingr",
        ),
        (
            'pub const LABEL: &str = "Mock Gingr event received";\n',
            "Mock Gingr event received",
        ),
        (
            'pub const MODEL: &str = "provider_reservation_record";\n',
            "provider_reservation_record",
        ),
    ],
)
def test_provider_branded_public_identifiers_fixture_uris_labels_and_typed_paths_are_rejected(
    tmp_path: Path,
    source: str,
    evidence: str,
) -> None:
    write_rust(tmp_path, "app/src/trace.rs", source)
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "provider fixture quarantine" in result.stderr
    assert evidence in result.stderr


@pytest.mark.parametrize(
    ("source", "evidence"),
    [
        (
            "pub struct SourcePayload { value: serde_json::Value }\n",
            "SourcePayload",
        ),
        (
            "pub struct ObservedSourceEvidence { source_model_path: String, value: serde_json::Value }\n",
            "source_model_path",
        ),
        (
            "pub struct ObservedSourceEvidence;\nimpl ObservedSourceEvidence { pub fn promotion_target_path(&self) -> &str { todo!() } }\n",
            "promotion_target_path",
        ),
    ],
)
def test_app_public_source_evidence_cannot_own_payload_or_model_mapping_contracts(
    tmp_path: Path,
    source: str,
    evidence: str,
) -> None:
    write_rust(tmp_path, "app/src/trace.rs", source)
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "provider fixture quarantine" in result.stderr
    assert evidence in result.stderr


def test_active_demo_docs_reject_stale_version_prose_without_route_token(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "domain/src/lib.rs", "pub struct Domain;\n")
    doc = tmp_path / "docs" / "demo" / "walkthrough.md"
    doc.parent.mkdir(parents=True, exist_ok=True)
    doc.write_text(
        "The checked OpenAPI v0 surface lists current routes.\n", encoding="utf-8"
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "active documentation route version" in result.stderr
    assert "OpenAPI v0" in result.stderr


def test_custom_deserializer_cannot_discard_typed_claim_fields_and_normalize_them(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "app/src/workflow.rs",
        """
use serde::Deserialize;
pub enum Decision { Pending, Approved }
pub struct Packet { decision: Decision }
#[derive(Deserialize)]
struct WirePacket { decision: Decision, required_review_gates: Vec<String> }
impl<'de> Deserialize<'de> for Packet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>
    {
        let wire = WirePacket::deserialize(deserializer)?;
        let _decision = wire.decision;
        let _required_review_gates = wire.required_review_gates;
        Ok(Self { decision: Decision::Pending })
    }
}
""",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "typed serialized claim normalization" in result.stderr


def test_explicitly_inert_public_variant_is_rejected_without_family_name_allowlist(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "domain/src/grooming.rs",
        """
pub enum Decision {
    /// This path is inert and current policy never emits it.
    Candidate,
    Suppressed,
}
pub fn decide() -> Decision { Decision::Suppressed }
""",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "explicitly inert public variant" in result.stderr
    assert "Decision::Candidate" in result.stderr


def test_ignored_public_compatibility_method_is_rejected_by_structure(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "domain/src/capacity.rs",
        """
pub struct Builder;
impl Builder {
    /// Accepted for older callers but ignored because the value is derived.
    pub fn expected_delta(self, _expected_delta: i32) -> Self { self }
}
""",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "ignored public method" in result.stderr
    assert "expected_delta" in result.stderr


def test_consumerless_public_type_is_rejected_without_a_family_name_allowlist(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "domain/src/quiet.rs", "pub struct QuietThing;\n")
    inventory = tmp_path / "docs" / "quality" / "public-api-inventory.json"
    inventory.parent.mkdir(parents=True, exist_ok=True)
    inventory.write_text('{"external_contract_symbols": []}\n', encoding="utf-8")
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "consumerless public API" in result.stderr
    assert "QuietThing" in result.stderr


def test_custom_deserializer_cannot_discard_typed_claims_by_destructuring(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "app/src/workflow.rs",
        """
use serde::Deserialize;
pub enum Decision { Pending, Approved }
pub struct Packet { decision: Decision }
#[derive(Deserialize)]
struct WirePacket { decision: Decision, required_review_gates: Vec<String> }
impl<'de> Deserialize<'de> for Packet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>
    {
        let WirePacket { decision: _, required_review_gates: _ } =
            WirePacket::deserialize(deserializer)?;
        Ok(Self { decision: Decision::Pending })
    }
}
""",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "typed serialized claim normalization" in result.stderr
    assert "decision" in result.stderr
    assert "required_review_gates" in result.stderr


def test_public_self_returning_method_cannot_silently_discard_an_argument(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "domain/src/capacity.rs",
        """
pub struct Builder;
impl Builder {
    pub fn expected_delta(self, _expected_delta: i32) -> Self { self }
}
""",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "ignored public method" in result.stderr
    assert "expected_delta" in result.stderr


@pytest.mark.parametrize(
    ("source", "evidence"),
    [
        ("pub fn gingr_report_fixture() {}\n", "gingr_report_fixture"),
        (
            'pub const SOURCE: &str = "fixture://gingr/report.json";\n',
            "fixture://gingr",
        ),
    ],
)
def test_provider_fixture_quarantine_rejects_equivalent_identifiers_and_uris(
    tmp_path: Path,
    source: str,
    evidence: str,
) -> None:
    write_rust(tmp_path, "app/src/trace.rs", source)
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "provider fixture quarantine" in result.stderr
    assert evidence in result.stderr


def test_active_demo_docs_reject_equivalent_stale_openapi_version_prose(
    tmp_path: Path,
) -> None:
    write_rust(tmp_path, "domain/src/lib.rs", "pub struct Domain;\n")
    doc = tmp_path / "docs" / "demo" / "walkthrough.md"
    doc.parent.mkdir(parents=True, exist_ok=True)
    doc.write_text(
        "The checked OpenAPI version 0 surface lists current routes.\n",
        encoding="utf-8",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "active documentation route version" in result.stderr
    assert "OpenAPI version 0" in result.stderr


def test_explicitly_non_emitted_public_variant_is_rejected_from_rustdoc_semantics(
    tmp_path: Path,
) -> None:
    write_rust(
        tmp_path,
        "domain/src/grooming.rs",
        """
pub enum Decision {
    /// Retained as a possible shape.
    /// Current policy does not emit this variant.
    Candidate,
    Suppressed,
}
pub fn decide() -> Decision { Decision::Suppressed }
""",
    )
    manifest = write_manifest(tmp_path)

    result = run_check(tmp_path, manifest)

    assert result.returncode == 1
    assert "explicitly inert public variant" in result.stderr
    assert "Decision::Candidate" in result.stderr
