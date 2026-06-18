import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
DOCS_CHECK = REPO_ROOT / "scripts" / "check_markdown_links.py"


class MarkdownDocsGateTest(unittest.TestCase):
    def run_check(self, root: Path):
        return subprocess.run(
            [sys.executable, str(DOCS_CHECK), "--root", str(root)],
            text=True,
            capture_output=True,
            check=False,
        )

    def write_minimal_required_readmes(self, root: Path):
        required_paths = [
            "domain/README.md",
            "app/README.md",
            "storage/README.md",
            "integrations/gingr/README.md",
            "domain/src/boarding/README.md",
            "domain/src/daycare/README.md",
            "domain/src/grooming/README.md",
            "domain/src/money/README.md",
            "domain/src/payment/README.md",
            "domain/src/reservation/README.md",
            "domain/src/retail/README.md",
            "domain/src/training/README.md",
            "storage/src/service_line/README.md",
            "integrations/gingr/src/endpoint/README.md",
            "integrations/gingr/src/dto/README.md",
            "integrations/gingr/src/mapping/README.md",
            "docs/integrations/gingr/README.md",
            "docs/integrations/gingr/fixtures/webhooks/README.md",
        ]
        for path in required_paths:
            target = root / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"# {path}\n", encoding="utf-8")
        return required_paths

    def test_gate_passes_for_existing_local_links_and_required_readmes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            required_paths = self.write_minimal_required_readmes(root)
            root_links = "\n".join(f"- [{path}]({path})" for path in required_paths)
            (root / "README.md").write_text(
                "# Root\n"
                "See [domain](domain/README.md) and [boarding](domain/src/boarding/README.md#domainsrcboardingreadmemd).\n"
                f"{root_links}\n",
                encoding="utf-8",
            )

            result = self.run_check(root)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("checked", result.stdout)

    def test_gate_fails_for_broken_local_markdown_link(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_minimal_required_readmes(root)
            (root / "README.md").write_text(
                "# Root\nSee [missing](docs/missing.md).\n",
                encoding="utf-8",
            )

            result = self.run_check(root)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("broken local markdown links", result.stderr.lower())
        self.assertIn("docs/missing.md", result.stderr)

    def test_gate_fails_when_required_navigation_readme_is_missing(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_minimal_required_readmes(root)
            (root / "domain/src/daycare/README.md").unlink()
            (root / "README.md").write_text("# Root\n", encoding="utf-8")

            result = self.run_check(root)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing required docs", result.stderr.lower())
        self.assertIn("domain/src/daycare/README.md", result.stderr)


if __name__ == "__main__":
    unittest.main()
