import re
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STRICT_BOUNDARY_FILES = (
    REPO_ROOT / "apps/api/src/http.rs",
    REPO_ROOT / "apps/api/src/public_contract.rs",
)
PUBLIC_CONTRACT_NESTED_REQUEST_TYPES = {
    "DataQualityHygieneOutcomeActor",
    "OutcomeAudit",
    "SourceRecordRef",
}
DESERIALIZED_STRUCT = re.compile(
    r"(?P<attrs>(?:#\[[^\n]+\]\n)+)"
    r"(?:pub(?:\([^)]*\))?\s+)?struct\s+(?P<name>[A-Za-z0-9_]+)",
)


def test_every_api_deserialized_struct_rejects_unknown_fields():
    missing = []
    checked = []
    for path in STRICT_BOUNDARY_FILES:
        source = path.read_text()
        for match in DESERIALIZED_STRUCT.finditer(source):
            attrs = match.group("attrs")
            if "Deserialize" not in attrs:
                continue
            name = match.group("name")
            if path.name == "public_contract.rs" and not (
                name.endswith(("Request", "Query", "SubmittedAction"))
                or name in PUBLIC_CONTRACT_NESTED_REQUEST_TYPES
            ):
                continue
            checked.append(f"{path.relative_to(REPO_ROOT)}::{name}")
            if "serde(deny_unknown_fields)" not in attrs:
                missing.append(f"{path.relative_to(REPO_ROOT)}::{name}")

    assert checked, "strict-boundary guard found no deserialized API structs"
    assert not missing, "deserialized API structs must be strict: " + ", ".join(missing)
