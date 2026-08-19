from __future__ import annotations

import importlib.util
import subprocess
import sys
from pathlib import Path

import pytest

SCRIPT = Path(__file__).resolve().parents[1] / "generate_public_api_inventory.py"


def load_inventory_module():
    spec = importlib.util.spec_from_file_location("public_api_inventory", SCRIPT)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def write_rustdoc_fixture(
    root: Path,
    *,
    module_path: str = "domain/sample",
    kind: str = "struct",
    name: str = "Packet",
    associated: tuple[tuple[str, str], ...] = (),
    source_owner: str = "domain/sample.rs",
) -> None:
    module = root / "target" / "doc" / Path(module_path)
    module.mkdir(parents=True)
    (module / "sidebar-items.js").write_text(
        f'window.SIDEBAR_ITEMS = {{"{kind}":["{name}"]}};\n',
        encoding="utf-8",
    )
    sections = "".join(
        f'<section id="{associated_kind}.{associated_name}"></section>'
        for associated_kind, associated_name in associated
    )
    (module / f"{kind}.{name}.html").write_text(
        f'<a href="../../../src/{source_owner}.html#1-8">Source</a>{sections}',
        encoding="utf-8",
    )


def write_rust(root: Path, relative: str, source: str) -> None:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(source, encoding="utf-8")


def run_inventory(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            sys.executable,
            str(SCRIPT),
            "--repo-root",
            str(root),
            "--doc-root",
            str(root / "target" / "doc"),
            "--output",
            str(root / "inventory.md"),
        ],
        capture_output=True,
        text=True,
        check=False,
    )


def test_check_mode_fails_closed_without_rewriting_a_stale_checked_inventory(
    tmp_path: Path,
) -> None:
    write_rustdoc_fixture(tmp_path)
    write_rust(tmp_path, "domain/sample.rs", "pub struct Packet;\n")
    write_rust(
        tmp_path,
        "app/consumer.rs",
        "fn consume(_: domain::sample::Packet) {}\n",
    )
    checked = tmp_path / "inventory.md"
    checked.write_text("stale checked inventory\n", encoding="utf-8")

    result = subprocess.run(
        [
            sys.executable,
            str(SCRIPT),
            "--repo-root",
            str(tmp_path),
            "--doc-root",
            str(tmp_path / "target" / "doc"),
            "--output",
            str(checked),
            "--check",
        ],
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 1
    assert "checked public API inventory is stale" in result.stderr
    assert checked.read_text(encoding="utf-8") == "stale checked inventory\n"


def test_generated_rustdoc_inventory_includes_items_methods_variants_and_consumers(
    tmp_path: Path,
) -> None:
    module = tmp_path / "target" / "doc" / "domain" / "sample"
    module.mkdir(parents=True)
    (module / "sidebar-items.js").write_text(
        'window.SIDEBAR_ITEMS = {"enum":["Decision"],"struct":["Packet"]};\n',
        encoding="utf-8",
    )
    (module / "struct.Packet.html").write_text(
        '<a href="../../../src/domain/sample.rs.html#1-8">Source</a>'
        '<section id="method.new"></section>'
        '<section id="trait-implementations"><section id="method.blanket"></section>',
        encoding="utf-8",
    )
    (module / "enum.Decision.html").write_text(
        '<a href="../../../src/domain/sample.rs.html#10-14">Source</a>'
        '<section id="variant.Review"></section>',
        encoding="utf-8",
    )
    source = tmp_path / "domain" / "sample.rs"
    source.parent.mkdir(parents=True)
    source.write_text(
        "pub struct Packet;\npub enum Decision { Review }\n",
        encoding="utf-8",
    )
    consumer = tmp_path / "app" / "consumer.rs"
    consumer.parent.mkdir(parents=True)
    consumer.write_text(
        "let _ = domain::sample::Packet::new();\nlet _ = domain::sample::Decision::Review;\n",
        encoding="utf-8",
    )
    output = tmp_path / "inventory.md"

    result = subprocess.run(
        [
            sys.executable,
            str(SCRIPT),
            "--repo-root",
            str(tmp_path),
            "--doc-root",
            str(tmp_path / "target" / "doc"),
            "--output",
            str(output),
        ],
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr
    inventory = output.read_text(encoding="utf-8")
    assert "`domain::sample::Packet`" in inventory
    assert "`domain::sample::Packet::new`" in inventory
    assert "`domain::sample::Packet::blanket`" not in inventory
    assert "`domain::sample::Decision::Review`" in inventory
    assert "app/consumer.rs" in inventory


def test_same_leaf_name_in_unrelated_module_is_not_a_consumer(tmp_path: Path) -> None:
    write_rustdoc_fixture(tmp_path)
    write_rust(tmp_path, "domain/sample.rs", "pub struct Packet;\n")
    write_rust(tmp_path, "app/unrelated.rs", "pub struct Packet;\n")

    result = run_inventory(tmp_path)

    assert result.returncode == 1
    inventory = (tmp_path / "inventory.md").read_text(encoding="utf-8")
    assert "`domain::sample::Packet`" in inventory
    assert "`app/unrelated.rs`" not in inventory


def test_unrelated_generic_method_call_is_not_a_consumer(tmp_path: Path) -> None:
    write_rustdoc_fixture(tmp_path, associated=(("method", "build"),))
    write_rust(tmp_path, "domain/sample.rs", "pub struct Packet;\n")
    write_rust(
        tmp_path,
        "app/unrelated.rs",
        "fn render(other: OtherBuilder) { let _ = other.build(); }\n",
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 1
    assert "none found" in (tmp_path / "inventory.md").read_text(encoding="utf-8")
    assert "app/unrelated.rs" not in (tmp_path / "inventory.md").read_text(
        encoding="utf-8"
    )


def test_incomplete_scip_is_supplemented_by_exact_qualified_source_resolution() -> None:
    inventory = load_inventory_module()
    item = inventory.ApiItem("domain::sample::Packet", "struct", "domain/sample.rs")
    sources = {
        "domain/sample.rs": inventory.source_index("pub struct Packet;\n"),
        "app/consumer.rs": inventory.source_index(
            "fn consume(_: domain::sample::Packet) {}\n"
        ),
    }
    incomplete_scip = inventory.ScipReferenceIndex({})

    assert inventory.consumers(item, sources, incomplete_scip) == ["app/consumer.rs"]


def test_same_owner_field_type_is_a_concrete_consumer() -> None:
    inventory = load_inventory_module()
    item = inventory.ApiItem("domain::sample::Evidence", "struct", "domain/sample.rs")
    sources = {
        "domain/sample.rs": inventory.source_index(
            "pub struct Evidence;\npub struct Packet { evidence: Evidence }\n"
        )
    }

    assert inventory.consumers(item, sources, None) == ["domain/sample.rs"]


def test_same_owner_declaration_and_impl_only_are_not_consumers() -> None:
    inventory = load_inventory_module()
    item = inventory.ApiItem("domain::sample::Evidence", "struct", "domain/sample.rs")
    sources = {
        "domain/sample.rs": inventory.source_index(
            "pub struct Evidence;\nimpl Evidence { pub fn label(&self) {} }\n"
        )
    }

    assert inventory.consumers(item, sources, None) == []


def test_import_renamed_type_resolves_to_canonical_symbol(tmp_path: Path) -> None:
    write_rustdoc_fixture(tmp_path)
    write_rust(tmp_path, "domain/sample.rs", "pub struct Packet;\n")
    write_rust(
        tmp_path,
        "app/consumer.rs",
        "use domain::sample::Packet as ReviewPacket;\nfn consume(_: ReviewPacket) {}\n",
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 0, result.stderr
    assert "`app/consumer.rs`" in (tmp_path / "inventory.md").read_text(
        encoding="utf-8"
    )


def test_reexported_owner_resolves_to_public_symbol_not_rustdoc_source_path(
    tmp_path: Path,
) -> None:
    write_rustdoc_fixture(
        tmp_path,
        module_path="domain/review",
        name="Packet",
        source_owner="domain/internal/packet.rs",
    )
    write_rust(tmp_path, "domain/internal/packet.rs", "pub struct Packet;\n")
    write_rust(
        tmp_path, "app/consumer.rs", "fn consume(_: domain::review::Packet) {}\n"
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 0, result.stderr
    inventory = (tmp_path / "inventory.md").read_text(encoding="utf-8")
    assert "`domain::review::Packet`" in inventory
    assert "`domain/internal/packet.rs`" in inventory
    assert "`app/consumer.rs`" in inventory


def test_consumerless_symbol_fails_instead_of_becoming_a_candidate(
    tmp_path: Path,
) -> None:
    write_rustdoc_fixture(tmp_path)
    write_rust(tmp_path, "domain/sample.rs", "pub struct Packet;\n")

    result = run_inventory(tmp_path)

    assert result.returncode == 1
    assert "consumerless public API" in result.stderr
    assert "none found" in (tmp_path / "inventory.md").read_text(encoding="utf-8")


def test_enum_owner_is_consumed_when_a_concrete_variant_is_referenced(
    tmp_path: Path,
) -> None:
    write_rustdoc_fixture(
        tmp_path,
        kind="enum",
        name="Decision",
        associated=(("variant", "Review"),),
    )
    write_rust(tmp_path, "domain/sample.rs", "pub enum Decision { Review }\n")
    write_rust(
        tmp_path,
        "app/consumer.rs",
        "let decision = domain::sample::Decision::Review;\n",
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 0, result.stderr
    inventory = (tmp_path / "inventory.md").read_text(encoding="utf-8")
    assert "`domain::sample::Decision`" in inventory
    assert "`domain::sample::Decision::Review`" in inventory
    assert "`app/consumer.rs`" in inventory


def test_trait_method_is_consumed_by_a_non_test_implementation(tmp_path: Path) -> None:
    write_rustdoc_fixture(
        tmp_path,
        kind="trait",
        name="Workflow",
        associated=(("tymethod", "evaluate"),),
    )
    write_rust(
        tmp_path,
        "domain/sample.rs",
        "pub trait Workflow { fn evaluate(&self); }\n",
    )
    write_rust(
        tmp_path,
        "app/consumer.rs",
        "struct Booking; impl domain::sample::Workflow for Booking { fn evaluate(&self) {} }\n",
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 0, result.stderr
    inventory = (tmp_path / "inventory.md").read_text(encoding="utf-8")
    assert "`domain::sample::Workflow::evaluate`" in inventory
    assert "`app/consumer.rs`" in inventory


def test_trait_methods_resolve_the_trait_not_the_first_implementor(
    tmp_path: Path,
) -> None:
    write_rustdoc_fixture(
        tmp_path,
        kind="trait",
        name="Workflow",
        associated=(("tymethod", "evaluate"),),
    )
    trait_html = (
        tmp_path
        / "target"
        / "doc"
        / "domain"
        / "sample"
        / "trait.Workflow.html"
    )
    trait_html.write_text(
        '<a href="../../../src/domain/sample.rs.html#1-8">Source</a>'
        '<section id="tymethod.evaluate"></section>'
        '<section id="impl-Workflow-for-Booking"></section>',
        encoding="utf-8",
    )
    write_rust(
        tmp_path,
        "domain/sample.rs",
        "pub trait Workflow { fn evaluate(&self); }\n",
    )
    write_rust(
        tmp_path,
        "app/consumer.rs",
        "struct Booking; impl domain::sample::Workflow for Booking { fn evaluate(&self) {} }\n",
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 0, result.stderr
    assert "`app/consumer.rs`" in (tmp_path / "inventory.md").read_text(
        encoding="utf-8"
    )


def test_scip_trait_method_resolution_prefers_the_public_trait_owner() -> None:
    inventory = load_inventory_module()
    index = inventory.ScipReferenceIndex(
        {
            "rust-analyzer cargo app 0.1.0 daily_update/impl#[`Agent`][`WorkflowAgent<Input, Output>`]evaluate().": {
                "app/src/daily_update.rs": 1
            }
        }
    )
    item = inventory.ApiItem(
        path="app::agents::WorkflowAgent::evaluate",
        kind="tymethod",
        owner="app/agents.rs",
        canonical_owner="Agent",
    )

    assert index.consumers(item) == ["app/src/daily_update.rs"]


def test_scip_reexported_type_resolves_the_public_module_path() -> None:
    inventory = load_inventory_module()
    index = inventory.ScipReferenceIndex(
        {
            "rust-analyzer cargo app 0.1.0 data_quality_hygiene/OutcomeCaptureService#": {
                "apps/spacetimedb/src/runtime.rs": 1
            }
        }
    )
    item = inventory.ApiItem(
        path="app::data_quality_hygiene::OutcomeCaptureService",
        kind="struct",
        owner="app/data_quality_hygiene/outcome_capture.rs",
        canonical_owner="OutcomeCaptureService",
    )

    assert index.consumers(item) == ["apps/spacetimedb/src/runtime.rs"]


def test_scip_cross_crate_reexport_resolves_the_canonical_imported_symbol() -> None:
    inventory = load_inventory_module()
    index = inventory.ScipReferenceIndex(
        {
            "rust-analyzer cargo domain 0.1.0 agent/OutputSchemaName#": {
                "app/src/daily_update.rs": 1
            }
        }
    )
    item = inventory.ApiItem(
        path="app::agents::OutputSchemaName",
        kind="struct",
        owner="app/agents.rs",
    )
    sources = {
        "app/src/agents.rs": inventory.source_index(
            "pub use domain::agent::OutputSchemaName;"
        )
    }

    assert inventory.consumers(item, sources, index) == ["app/src/daily_update.rs"]


def test_reexported_generated_method_resolves_its_canonical_source_owner(
    tmp_path: Path,
) -> None:
    inventory = load_inventory_module()
    write_rust(
        tmp_path,
        "app/src/agents.rs",
        "pub use domain::agent::OutputSchemaName;\n",
    )
    canonical = tmp_path / "domain" / "src" / "agent.rs"
    write_rust(
        tmp_path,
        "domain/src/agent.rs",
        "#[nutype::nutype] pub struct OutputSchemaName(String);\n",
    )
    item = inventory.ApiItem(
        path="app::agents::OutputSchemaName::into_inner",
        kind="method",
        owner="core/clone.rs",
    )

    assert inventory.source_owner_path(item, tmp_path) == canonical
    assert inventory.associated_item_is_generated(item, tmp_path)


def test_scip_ignores_generic_impl_names_misread_as_canonical_owners() -> None:
    inventory = load_inventory_module()
    index = inventory.ScipReferenceIndex(
        {
            "rust-analyzer cargo app 0.1.0 booking_triage/service/Thing#": {
                "app/src/unrelated.rs": 1
            }
        }
    )
    item = inventory.ApiItem(
        path="app::booking_triage::Service",
        kind="struct",
        owner="app/booking_triage/service.rs",
        canonical_owner="T",
    )

    assert index.consumers(item) == []


def test_scip_requires_an_exact_type_descriptor_not_an_owner_substring() -> None:
    inventory = load_inventory_module()
    index = inventory.ScipReferenceIndex(
        {
            "rust-analyzer cargo app 0.1.0 booking_triage/service/ServicePolicy#": {
                "app/src/local_smoke.rs": 1
            }
        }
    )
    item = inventory.ApiItem(
        path="app::booking_triage::Service",
        kind="struct",
        owner="app/booking_triage/service.rs",
    )

    assert index.consumers(item) == []


def test_scip_resolves_exact_free_function_descriptors() -> None:
    inventory = load_inventory_module()
    index = inventory.ScipReferenceIndex(
        {
            "rust-analyzer cargo app 0.1.0 agents/baseline_agent_specs().": {
                "apps/cli/src/main.rs": 1
            }
        }
    )
    item = inventory.ApiItem(
        path="app::agents::baseline_agent_specs",
        kind="fn",
        owner="app/agents.rs",
    )

    assert index.consumers(item) == ["apps/cli/src/main.rs"]


def test_scip_resolution_retains_exact_non_test_trait_implementations() -> None:
    inventory = load_inventory_module()
    item = inventory.ApiItem(
        path="app::agents::WorkflowAgent::validate_output",
        kind="tymethod",
        owner="app/agents.rs",
    )
    sources = {
        "app/src/daily_update.rs": inventory.source_index(
            "impl app::agents::WorkflowAgent for Agent {"
            " fn spec(&self) {}"
            " fn validate_output(&self) {}"
            " }"
        )
    }

    assert inventory.consumers(
        item, sources, inventory.ScipReferenceIndex({})
    ) == ["app/src/daily_update.rs"]


def test_trait_implementation_through_a_renamed_module_resolves_canonically() -> None:
    inventory = load_inventory_module()
    item = inventory.ApiItem(
        path="app::data_quality_hygiene::ActorDirectory::resolve_actor",
        kind="tymethod",
        owner="app/data_quality_hygiene/outcome_capture.rs",
    )
    sources = {
        "apps/spacetimedb/src/adapter.rs": inventory.source_index(
            "use app::data_quality_hygiene as hygiene;\n"
            "impl hygiene::ActorDirectory for Adapter {\n"
            "    fn resolve_actor(&self) {}\n"
            "}\n"
        )
    }

    assert inventory.consumers(
        item, sources, inventory.ScipReferenceIndex({})
    ) == ["apps/spacetimedb/src/adapter.rs"]


def test_symbol_reference_through_a_renamed_module_resolves_canonically() -> None:
    inventory = load_inventory_module()
    source = inventory.source_index(
        "use domain::workflow as wf;\n"
        "fn consume() { let _ = wf::AllowedAction::CreateInternalTask; }\n"
    )
    item = inventory.ApiItem(
        path="domain::workflow::AllowedAction::CreateInternalTask",
        kind="variant",
        owner="domain/workflow/event.rs",
    )

    assert inventory.symbol_is_referenced(source, item)


def test_generated_associated_method_inherits_its_consumed_owner_contract(
    tmp_path: Path,
) -> None:
    write_rustdoc_fixture(tmp_path, associated=(("method", "build"),))
    write_rust(tmp_path, "domain/sample.rs", "pub struct Packet;\n")
    write_rust(
        tmp_path,
        "app/consumer.rs",
        "fn consume(_: domain::sample::Packet) {}\n",
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 0, result.stderr
    inventory = (tmp_path / "inventory.md").read_text(encoding="utf-8")
    assert "`domain::sample::Packet::build`" in inventory
    assert "`app/consumer.rs`" in inventory


def test_external_contract_evidence_must_name_the_exact_public_symbol(
    tmp_path: Path,
) -> None:
    inventory = load_inventory_module()
    quality = tmp_path / "docs" / "quality"
    quality.mkdir(parents=True)
    (quality / "public-api-inventory.json").write_text(
        '{"external_contract_symbols": ['
        '{"symbol": "domain::Metric", '
        '"rationale": "Externally serialized metric contract", '
        '"evidence": "domain/tests/metric_contract.rs"}]}'
    )
    owner = inventory.ApiItem("domain::Metric", "enum", "domain/src/lib.rs")
    method = inventory.ApiItem(
        "domain::Metric::as_str", "method", "domain/src/lib.rs"
    )

    assert not inventory.has_external_contract_evidence(owner, tmp_path)
    assert not inventory.has_external_contract_evidence(method, tmp_path)


def test_compatibility_only_rationale_is_not_current_external_contract_evidence(
    tmp_path: Path,
) -> None:
    inventory = load_inventory_module()
    quality = tmp_path / "docs" / "quality"
    quality.mkdir(parents=True)
    (quality / "public-api-inventory.json").write_text(
        '{"external_contract_symbols": ['
        '{"symbol": "domain::CurrentAction::Draft", '
        '"rationale": "Exact serialized compatibility case", '
        '"evidence": "Compatibility tests only"}]}'
    )
    item = inventory.ApiItem(
        "domain::CurrentAction::Draft", "variant", "domain/src/lib.rs"
    )

    assert not inventory.has_external_contract_evidence(item, tmp_path)


@pytest.mark.parametrize(
    ("rationale", "evidence"),
    [
        pytest.param(
            "Exact downstream-facing method retained on the public type",
            "Public Rustdoc contract and declaration at `domain/sample.rs`.",
            id="declaration-and-rustdoc-only",
        ),
        pytest.param(
            "Canonical serialized packet contract",
            "Serialized declaration plus executable boundary tests in `domain/tests/packet.rs`.",
            id="serialized-declaration-and-tests-only",
        ),
        pytest.param(
            "Public packet consumed by downstream workflow adapters",
            "The declaration is available for future downstream adapters.",
            id="hypothetical-downstream-adapter",
        ),
    ],
)
def test_weak_external_contract_assertions_cannot_exempt_a_consumerless_symbol(
    tmp_path: Path, rationale: str, evidence: str
) -> None:
    write_rustdoc_fixture(tmp_path)
    write_rust(tmp_path, "domain/sample.rs", "pub struct Packet;\n")
    quality = tmp_path / "docs" / "quality"
    quality.mkdir(parents=True)
    (quality / "public-api-inventory.json").write_text(
        '{"external_contract_symbols": ['
        '{"symbol": "domain::sample::Packet", '
        f'"rationale": {rationale!r}, "evidence": {evidence!r}'
        '}]}'.replace("'", '"'),
        encoding="utf-8",
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 1
    assert "external-contract inventory integrity failure" in result.stderr
    assert "domain::sample::Packet" in result.stderr


def test_generated_method_is_not_shadowed_by_an_unrelated_same_name_method(
    tmp_path: Path,
) -> None:
    write_rustdoc_fixture(tmp_path, associated=(("method", "build"),))
    write_rust(
        tmp_path,
        "domain/sample.rs",
        "pub struct Packet; struct Other; impl Other { pub fn build(&self) {} }\n",
    )
    write_rust(
        tmp_path,
        "app/consumer.rs",
        "fn consume(_: domain::sample::Packet) {}\n",
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 0, result.stderr
    inventory = (tmp_path / "inventory.md").read_text(encoding="utf-8")
    assert "`domain::sample::Packet::build`" in inventory
    assert "`app/consumer.rs`" in inventory


def test_spacetimedb_macro_generated_handles_have_structural_boundary_evidence(
    tmp_path: Path,
) -> None:
    write_rustdoc_fixture(
        tmp_path,
        module_path="nva_spacetimedb/sample",
        name="booking__TableHandle",
        source_owner="nva_spacetimedb/sample.rs",
    )
    write_rust(
        tmp_path,
        "nva_spacetimedb/sample.rs",
        "#[spacetimedb::table(name = booking, public)] pub struct Booking { pub id: u64 }\n",
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 0, result.stderr
    inventory = (tmp_path / "inventory.md").read_text(encoding="utf-8")
    assert "SpacetimeDB macro registration" in inventory
    assert "`nva_spacetimedb::sample::booking__TableHandle`" in inventory


def test_binary_main_and_generated_endpoint_have_structural_boundary_evidence(
    tmp_path: Path,
) -> None:
    inventory = load_inventory_module()
    repo_root = tmp_path / "repo"
    main = repo_root / "apps" / "cli" / "src" / "main.rs"
    endpoint = repo_root / "integrations" / "gingr" / "src" / "endpoint.rs"
    main.parent.mkdir(parents=True)
    endpoint.parent.mkdir(parents=True)
    main.write_text("fn main() {}\n", encoding="utf-8")
    endpoint.write_text(
        'simple_reference_endpoint!(GetBreeds, "breeds");\n', encoding="utf-8"
    )

    assert inventory.structural_consumer_evidence(
        inventory.ApiItem("cli::main", "fn", "cli/main.rs"), repo_root
    ) == ["apps/cli/src/main.rs (binary executable entry point)"]
    assert inventory.structural_consumer_evidence(
        inventory.ApiItem(
            "gingr::endpoint::GetBreeds", "struct", "gingr/endpoint.rs"
        ),
        repo_root,
    ) == [
        "integrations/gingr/src/endpoint.rs (generated provider endpoint registration)"
    ]


def test_serialized_variant_has_exact_consumed_owner_boundary_evidence(
    tmp_path: Path,
) -> None:
    write_rustdoc_fixture(
        tmp_path,
        kind="enum",
        name="Decision",
        associated=(("variant", "Review"),),
    )
    write_rust(
        tmp_path,
        "domain/sample.rs",
        "#[derive(serde::Serialize, serde::Deserialize)]\n"
        "/// Persisted decision vocabulary.\n"
        "pub enum Decision { Review }\n",
    )
    write_rust(
        tmp_path,
        "app/consumer.rs",
        "fn persist(_: domain::sample::Decision) {}\n",
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 0, result.stderr
    inventory = (tmp_path / "inventory.md").read_text(encoding="utf-8")
    assert "serialized enum boundary" in inventory
    assert "`domain::sample::Decision::Review`" in inventory


def test_thiserror_variant_has_generated_conversion_boundary_evidence(
    tmp_path: Path,
) -> None:
    inventory = load_inventory_module()
    repo_root = tmp_path / "repo"
    source = repo_root / "crate" / "src" / "lib.rs"
    source.parent.mkdir(parents=True)
    source.write_text(
        "#[derive(Debug, thiserror::Error)]\n"
        "pub enum Error {\n"
        "    #[error(\"invalid input\")]\n"
        "    InvalidInput(#[from] std::num::ParseIntError),\n"
        "}\n",
        encoding="utf-8",
    )
    variant = inventory.ApiItem(
        path="crate::Error::InvalidInput",
        kind="variant",
        owner="crate/src/lib.rs",
        canonical_owner="Error",
    )

    assert inventory.error_variant_evidence(variant, repo_root) == [
        "crate/src/lib.rs (generated error conversion boundary `crate::Error`)"
    ]


def test_derived_string_variant_has_exact_consumed_owner_boundary_evidence(
    tmp_path: Path,
) -> None:
    inventory = load_inventory_module()
    repo_root = tmp_path / "repo"
    source = repo_root / "crate" / "src" / "lib.rs"
    source.parent.mkdir(parents=True)
    source.write_text(
        "#[derive(strum::EnumString)]\n"
        "pub enum EventType { CheckIn }\n",
        encoding="utf-8",
    )
    variant = inventory.ApiItem(
        path="crate::EventType::CheckIn",
        kind="variant",
        owner="crate/src/lib.rs",
        canonical_owner="EventType",
    )

    assert inventory.derived_string_variant_evidence(
        variant, repo_root, {"crate::EventType": ["app/src/consumer.rs"]}
    ) == [
        "crate/src/lib.rs (derived external string vocabulary `crate::EventType` consumed by app/src/consumer.rs)"
    ]


def test_trait_method_source_implementation_is_consumer_evidence() -> None:
    inventory = load_inventory_module()
    item = inventory.ApiItem(
        path="app::tools::Gateway::send",
        kind="tymethod",
        owner="app/src/adapter.rs",
        canonical_owner="Gateway",
    )
    sources = {
        "app/src/adapter.rs": inventory.source_index(
            "impl Gateway for Adapter {\n"
            "    fn send(&self) {}\n"
            "}\n"
        )
    }

    assert inventory.consumers(item, sources, None) == ["app/src/adapter.rs"]


def test_explicit_unused_method_does_not_inherit_its_consumed_owner_contract(
    tmp_path: Path,
) -> None:
    write_rustdoc_fixture(tmp_path, associated=(("method", "build"),))
    write_rust(
        tmp_path,
        "domain/sample.rs",
        "pub struct Packet; impl Packet { pub fn build(&self) {} }\n",
    )
    write_rust(
        tmp_path,
        "app/consumer.rs",
        "fn consume(_: domain::sample::Packet) {}\n",
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 1
    assert "domain::sample::Packet::build" in result.stderr


def test_read_only_accessor_inherits_its_consumed_owner_contract(tmp_path: Path) -> None:
    write_rustdoc_fixture(tmp_path, associated=(("method", "name"),))
    write_rust(
        tmp_path,
        "domain/sample.rs",
        "pub struct Packet { name: String }\n"
        "impl Packet { pub fn name(&self) -> &str { self.name.as_str() } }\n",
    )
    write_rust(
        tmp_path,
        "app/consumer.rs",
        "fn consume(_: domain::sample::Packet) {}\n",
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 0, result.stderr
    inventory = (tmp_path / "inventory.md").read_text(encoding="utf-8")
    assert "read-only accessor on consumed owner" in inventory


def test_read_only_boolean_projection_inherits_its_consumed_owner_contract(
    tmp_path: Path,
) -> None:
    write_rustdoc_fixture(tmp_path, associated=(("method", "is_ready"),))
    write_rust(
        tmp_path,
        "domain/sample.rs",
        "pub struct Packet { flags: Vec<String>, approved: bool }\n"
        "impl Packet { pub fn is_ready(&self) -> bool { "
        "self.flags.is_empty() && self.approved } }\n",
    )
    write_rust(
        tmp_path,
        "app/consumer.rs",
        "fn consume(_: domain::sample::Packet) {}\n",
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 0, result.stderr


def test_side_effecting_no_argument_method_does_not_inherit_owner_contract(
    tmp_path: Path,
) -> None:
    write_rustdoc_fixture(tmp_path, associated=(("method", "send"),))
    write_rust(
        tmp_path,
        "domain/sample.rs",
        "pub struct Packet { gateway: Gateway }\n"
        "impl Packet { pub fn send(&self) { self.gateway.send() } }\n",
    )
    write_rust(
        tmp_path,
        "app/consumer.rs",
        "fn consume(_: domain::sample::Packet) {}\n",
    )

    result = run_inventory(tmp_path)

    assert result.returncode == 1
    assert "domain::sample::Packet::send" in result.stderr
