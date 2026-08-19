import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ATLAS = REPO_ROOT / "docs" / "architecture" / "semantic-domain-contract-atlas.md"
ADR = REPO_ROOT / "docs" / "architecture" / "semantic-domain-ownership-adr.md"


class SemanticDomainContractAtlasTest(unittest.TestCase):
    def test_atlas_and_adr_cover_required_ownership_relationships(self):
        atlas = ATLAS.read_text(encoding="utf-8")
        adr = ADR.read_text(encoding="utf-8")

        for required in (
            "lead ↔ customer/pet/site/service",
            "event ↔ SLA ↔ attempt ↔ message ↔ conversion",
            "note ↔ evidence/visibility/consent",
            "demand ↔ coverage ↔ shift/skill",
            "document ↔ applicability ↔ citation",
            "recommendation ↔ reviewed action ↔ reported outcome",
        ):
            self.assertIn(required, atlas)

        self.assertIn("unconditionally returns `NotClaimed`", atlas)
        self.assertIn("new opaque, non-serializable measurement authority", atlas)
        self.assertNotIn("before value claim", atlas)

        for current_owner in (
            "domain::source::System",
            "domain::consent",
            "domain::customer::intelligence",
            "domain::operations::capacity",
            "domain::analytics::finance",
            "domain::analytics::outcome",
        ):
            self.assertIn(current_owner, atlas)

        self.assertNotIn("strategic_ai_ops::", atlas)
        self.assertNotIn("strategic_ai_ops::", adr)
        self.assertNotIn("strategic bridge", atlas.lower())

    def test_atlas_describes_current_fail_closed_evidence_and_authority_contracts(self):
        atlas = ATLAS.read_text(encoding="utf-8")

        for required in (
            "Observed<T> -> Candidate<T> -> Accepted<T>",
            "provider-neutral",
            "private issuance",
            "validated rehydration",
            "fail closed",
            "opaque",
            "non-serializable",
        ):
            self.assertIn(required, atlas)


if __name__ == "__main__":
    unittest.main()