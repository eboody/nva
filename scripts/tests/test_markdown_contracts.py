import importlib.util
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "check_markdown_links.py"


def load_markdown_contracts_module():
    spec = importlib.util.spec_from_file_location("check_markdown_links", SCRIPT_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"could not load {SCRIPT_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class MarkdownContractsTest(unittest.TestCase):
    def write(self, root, relative_path, content):
        path = root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(textwrap.dedent(content).lstrip(), encoding="utf-8")
        return path

    def test_local_markdown_link_contract_reports_missing_repo_relative_target(self):
        contracts = load_markdown_contracts_module()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write(
                root,
                "README.md",
                """
                # Root

                See [missing architecture doc](docs/architecture/missing.md).
                """,
            )

            failures = contracts.check_local_markdown_links(root)

        self.assertEqual(len(failures), 1)
        self.assertIn("README.md:3", failures[0])
        self.assertIn("docs/architecture/missing.md", failures[0])

    def test_local_markdown_link_contract_ignores_generated_vendor_and_external_links(self):
        contracts = load_markdown_contracts_module()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write(root, "README.md", "[valid](docs/ops/runbook.md) and [external](https://example.com)")
            self.write(root, "docs/ops/runbook.md", "# Runbook\n")
            self.write(root, "target/generated.md", "[generated broken](missing.md)")
            self.write(root, "node_modules/pkg/README.md", "[vendor broken](missing.md)")

            failures = contracts.check_local_markdown_links(root)

        self.assertEqual(failures, [])

    def test_readme_coverage_contract_requires_workspace_and_domain_module_readmes(self):
        contracts = load_markdown_contracts_module()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write(
                root,
                "Cargo.toml",
                """
                [workspace]
                members = ["domain", "app", "integrations/gingr"]
                """,
            )
            self.write(
                root,
                "README.md",
                """
                # Root

                - [domain](domain/README.md)
                - [integrations](integrations/gingr/README.md)
                - [Boarding](domain/src/boarding/README.md)
                """,
            )
            self.write(root, "domain/README.md", "# Domain\n")
            self.write(root, "integrations/gingr/README.md", "# Gingr\n")
            self.write(root, "domain/src/boarding/README.md", "# Boarding\n")

            failures = contracts.check_required_readme_coverage(root)

        self.assertIn("workspace member app is missing README.md", "\n".join(failures))
        self.assertIn("root README is missing link to app/README.md", "\n".join(failures))
        self.assertIn("domain module daycare is missing domain/src/daycare/README.md", "\n".join(failures))
        self.assertIn("root README is missing link to domain/src/daycare/README.md", "\n".join(failures))


if __name__ == "__main__":
    unittest.main()
