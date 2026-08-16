import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ATLAS = REPO_ROOT / "docs" / "architecture" / "semantic-domain-contract-atlas.md"
ADR = REPO_ROOT / "docs" / "architecture" / "semantic-domain-ownership-adr.md"


class SemanticDomainContractAtlasTest(unittest.TestCase):
    def test_atlas_and_adr_cover_required_ownership_relationships_and_strategic_overlaps(self):
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

        for strategic_overlap in (
            "strategic_ai_ops::source::System",
            "strategic_ai_ops::communication::Channel",
            "strategic_ai_ops::crm::StructuredNote",
            "strategic_ai_ops::capacity::OptimizationRecommendation",
            "strategic_ai_ops::financial::MoneyCents",
            "strategic_ai_ops::outcome::Record",
        ):
            self.assertIn(strategic_overlap, atlas)
            self.assertIn(strategic_overlap, adr)

        for decision in ("Migrate", "Bridge", "Delete"):
            self.assertIn(decision, adr)


if __name__ == "__main__":
    unittest.main()