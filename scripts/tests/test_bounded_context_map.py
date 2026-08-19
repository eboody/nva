from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAP = REPO_ROOT / "docs" / "architecture" / "bounded-context-map.md"
README = REPO_ROOT / "README.md"
OWNERSHIP_ADR = REPO_ROOT / "docs" / "architecture" / "semantic-domain-ownership-adr.md"


def test_bounded_context_map_owns_every_required_architecture_role() -> None:
    bounded_context_map = MAP.read_text(encoding="utf-8")

    for required_context in (
        "`domain`",
        "`app`",
        "`storage`",
        "`integrations/gingr`",
        "`apps/api`",
        "`apps/worker`",
        "`apps/spacetimedb`",
    ):
        assert required_context in bounded_context_map

    for required_role in (
        "Concept owner",
        "Error owner",
        "Conversion owner",
        "Compatibility codec owner",
        "Repository owner",
        "Authority issuer owner",
    ):
        assert required_role in bounded_context_map

    for required_rule in (
        "Observed<T> -> Candidate<T> -> Accepted<T>",
        "provider DTO",
        "dependency aliases",
        "CurrentApprovalReviewerCapability",
        "QueueContactAuthority",
        "private",
        "tuple and unit",
        "non-serializable",
        "no production issuer",
        "PostgreSQL",
        "SpacetimeDB",
        "OpenAPI",
    ):
        assert required_rule in bounded_context_map


def test_canonical_navigation_links_the_map_and_ownership_adr() -> None:
    readme = README.read_text(encoding="utf-8")
    ownership_adr = OWNERSHIP_ADR.read_text(encoding="utf-8")

    assert "[Bounded-context ownership map](docs/architecture/bounded-context-map.md)" in readme
    assert "[Semantic ownership ADR](docs/architecture/semantic-domain-ownership-adr.md)" in readme
    assert "[bounded-context ownership map](bounded-context-map.md)" in ownership_adr