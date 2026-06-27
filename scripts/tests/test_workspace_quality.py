import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
CHECK = REPO_ROOT / "scripts" / "check_workspace_quality.py"


class WorkspaceQualityGateTest(unittest.TestCase):
    def run_check(self, root: Path):
        return subprocess.run(
            [sys.executable, str(CHECK), "--repo-root", str(root)],
            text=True,
            capture_output=True,
            check=False,
        )

    def init_repo(self, root: Path):
        subprocess.run(["git", "init", "-q"], cwd=root, check=True)
        subprocess.run(["git", "config", "user.email", "test@example.invalid"], cwd=root, check=True)
        subprocess.run(["git", "config", "user.name", "Workspace Quality Test"], cwd=root, check=True)

    def write_minimal_workspace(self, root: Path):
        api_src = root / "apps" / "api" / "src"
        api_src.mkdir(parents=True, exist_ok=True)
        (api_src / "http.rs").write_text(
            "pub fn router_with_state() {\n"
            "    Router::new()\n"
            "        .route(\"/healthz\", get(healthz))\n"
            "        .route(\"/v0/healthz\", get(healthz))\n"
            "        .route(\"/v0/read-models/source-quality-backlog\", get(source_quality_backlog));\n"
            "}\n",
            encoding="utf-8",
        )
        openapi = root / "apps" / "api" / "openapi"
        openapi.mkdir(parents=True, exist_ok=True)
        (openapi / "owned-operations-v0.openapi.json").write_text(
            json.dumps(
                {
                    "openapi": "3.1.0",
                    "paths": {
                        "/v0/healthz": {"get": {}},
                        "/v0/read-models/source-quality-backlog": {"get": {}},
                    },
                    "components": {"schemas": {}},
                },
                indent=2,
            ),
            encoding="utf-8",
        )
        (root / "README.md").write_text("# Demo\nClean docs.\n", encoding="utf-8")

    def commit_all(self, root: Path):
        subprocess.run(["git", "add", "-A"], cwd=root, check=True)
        subprocess.run(["git", "commit", "-q", "-m", "baseline"], cwd=root, check=True)

    def test_gate_passes_for_clean_workspace_contracts(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.init_repo(root)
            self.write_minimal_workspace(root)
            self.commit_all(root)

            result = self.run_check(root)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("workspace_quality_ok", result.stdout)
        self.assertIn("openapi_v0_routes=2", result.stdout)

    def test_gate_fails_for_stale_wording_in_changed_markdown(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.init_repo(root)
            self.write_minimal_workspace(root)
            self.commit_all(root)
            (root / "README.md").write_text(
                "# Demo\nThis placeholder still needs cleanup.\n",
                encoding="utf-8",
            )

            result = self.run_check(root)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("stale/noisy wording", result.stderr.lower())
        self.assertIn("README.md", result.stderr)
        self.assertIn("placeholder", result.stderr)

    def test_gate_fails_when_openapi_omits_v0_axum_route(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.init_repo(root)
            self.write_minimal_workspace(root)
            self.commit_all(root)
            spec_path = root / "apps" / "api" / "openapi" / "owned-operations-v0.openapi.json"
            spec = json.loads(spec_path.read_text(encoding="utf-8"))
            del spec["paths"]["/v0/read-models/source-quality-backlog"]
            spec_path.write_text(json.dumps(spec, indent=2), encoding="utf-8")

            result = self.run_check(root)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("openapi missing v0 routes", result.stderr.lower())
        self.assertIn("/v0/read-models/source-quality-backlog", result.stderr)

    def test_gate_scans_untracked_markdown_before_closeout(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.init_repo(root)
            self.write_minimal_workspace(root)
            self.commit_all(root)
            new_doc = root / "docs" / "handoff.md"
            new_doc.parent.mkdir(parents=True, exist_ok=True)
            new_doc.write_text("# Handoff\nTemporary placeholder wording.\n", encoding="utf-8")

            result = self.run_check(root)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("stale/noisy wording", result.stderr.lower())
        self.assertIn("docs/handoff.md", result.stderr)

    def test_gate_fails_for_untracked_cache_artifacts(self):
        if not shutil.which("git"):
            self.skipTest("git is required")
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.init_repo(root)
            self.write_minimal_workspace(root)
            self.commit_all(root)
            cache = root / "scripts" / "__pycache__" / "check.cpython-311.pyc"
            cache.parent.mkdir(parents=True, exist_ok=True)
            cache.write_bytes(b"cache")

            result = self.run_check(root)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("noisy workspace artifact", result.stderr.lower())
        self.assertIn("__pycache__", result.stderr)


if __name__ == "__main__":
    unittest.main()
