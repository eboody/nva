import json
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = REPO_ROOT / "docs" / "plans" / "2026-08-13-semantic-modeling-audit-manifest.json"


REQUIRED_FINDINGS = {
    "construction_serde_bypass",
    "approval_authority",
    "repeated_vaccine_decisions",
    "http_trusted_actor_boundary",
    "public_contract_identity",
    "spacetimedb_lossiness",
    "provenance_attachment",
}


def test_semantic_modeling_audit_manifest_covers_every_required_regression_contract():
    manifest = json.loads(MANIFEST_PATH.read_text())

    assert manifest["plan"] == "docs/plans/2026-08-13-semantic-modeling-perfection.md"
    assert manifest["schema_version"] == "semantic-modeling-audit-manifest.v1"
    finding_ids = {finding["id"] for finding in manifest["findings"]}
    assert REQUIRED_FINDINGS <= finding_ids

    for finding in manifest["findings"]:
        assert finding["work_program_items"], finding["id"]
        assert finding["invariant"], finding["id"]
        assert finding["current_contract_tests"], finding["id"]
        for contract in finding["current_contract_tests"]:
            assert contract["crate"], finding["id"]
            assert contract["path"], finding["id"]
            assert contract["test"], finding["id"]
            assert contract["covers"] in {"current_acceptance", "future_acceptance_scaffold"}


def test_manifest_has_no_future_acceptance_scaffolds_after_remediation_review():
    manifest = json.loads(MANIFEST_PATH.read_text())

    future = [
        f"{finding['id']}:{contract['path']}::{contract['test']}"
        for finding in manifest["findings"]
        for contract in finding["current_contract_tests"]
        if contract["covers"] == "future_acceptance_scaffold"
    ]

    assert future == []


def test_each_plan_work_program_item_is_traceable_to_manifest_acceptance():
    manifest = json.loads(MANIFEST_PATH.read_text())

    covered_items = {
        item
        for finding in manifest["findings"]
        for item in finding["work_program_items"]
    }
    assert set(range(1, 22)) <= covered_items

    successor_items = {
        implication["work_program_item"]
        for finding in manifest["findings"]
        for implication in finding["successor_implications"]
    }
    assert set(range(2, 22)) <= successor_items


def test_every_manifest_contract_names_an_existing_test():
    manifest = json.loads(MANIFEST_PATH.read_text())

    missing = []
    for finding in manifest["findings"]:
        for contract in finding["current_contract_tests"]:
            contract_path = REPO_ROOT / contract["path"]
            if not contract_path.is_file():
                missing.append(f"{finding['id']}:{contract['path']} (missing file)")
                continue
            if f"fn {contract['test']}" not in contract_path.read_text():
                missing.append(
                    f"{finding['id']}:{contract['path']}::{contract['test']} (missing test)"
                )

    assert missing == []
