import json
import os
import subprocess
import sys
import tempfile
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOLS_DIR = REPO_ROOT / "scripts" / "hermes-tools"
LOCATION_ID = "00c0ffee-0000-0000-0000-000000000001"
OPERATING_DAY = "2026-06-17"


class FakePetResortApi:
    def __init__(self, *, fail_status=None):
        self.fail_status = fail_status
        self.requests = []
        self._server = None
        self._thread = None

    def __enter__(self):
        outer = self

        class Handler(BaseHTTPRequestHandler):
            def do_GET(self):
                outer._capture(self)
                if outer.fail_status:
                    self.send_response(outer.fail_status)
                    self.send_header("Content-Type", "application/json")
                    self.end_headers()
                    self.wfile.write(json.dumps({"error": "upstream exploded"}).encode())
                    return
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(
                    json.dumps(
                        {
                            "workflow": {"name": "manager_daily_brief"},
                            "location_id": LOCATION_ID,
                            "operating_day": OPERATING_DAY,
                            "audit": {"correlation_id": f"manager-daily-brief:{LOCATION_ID}:{OPERATING_DAY}"},
                        }
                    ).encode()
                )

            def do_POST(self):
                outer._capture(self)
                if outer.fail_status:
                    self.send_response(outer.fail_status)
                    self.send_header("Content-Type", "application/json")
                    self.end_headers()
                    self.wfile.write(json.dumps({"error": "upstream exploded"}).encode())
                    return
                response_body = {
                    "accepted": True,
                    "validation": {"status": "accepted"},
                    "live_side_effects_allowed": False,
                }
                status = 201
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps(response_body).encode())

            def log_message(self, *_args):
                return

        self._server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self._thread = threading.Thread(target=self._server.serve_forever, daemon=True)
        self._thread.start()
        return self

    def __exit__(self, *_exc):
        self._server.shutdown()
        self._thread.join(timeout=5)

    @property
    def url(self):
        host, port = self._server.server_address
        return f"http://{host}:{port}"

    def _capture(self, handler):
        length = int(handler.headers.get("Content-Length", "0"))
        raw_body = handler.rfile.read(length) if length else b""
        body = json.loads(raw_body.decode()) if raw_body else None
        self.requests.append(
            {
                "method": handler.command,
                "path": handler.path,
                "authorization": handler.headers.get("Authorization"),
                "body": body,
            }
        )


class HermesAgentBridgeScriptsTest(unittest.TestCase):
    def run_tool(self, name, *args, input_json=None, env=None):
        merged_env = os.environ.copy()
        merged_env.update(env or {})
        input_text = json.dumps(input_json) if input_json is not None else None
        return subprocess.run(
            [str(TOOLS_DIR / name), *args],
            input=input_text,
            text=True,
            capture_output=True,
            env=merged_env,
            check=False,
        )

    def test_get_manager_daily_brief_context_calls_configured_app_context_endpoint(self):
        with FakePetResortApi() as api:
            result = self.run_tool(
                "get_manager_daily_brief_context",
                "--location-id",
                LOCATION_ID,
                "--operating-day",
                OPERATING_DAY,
                env={"PET_RESORT_API_URL": api.url},
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["workflow"]["name"], "manager_daily_brief")
        self.assertEqual(api.requests[0]["method"], "GET")
        self.assertEqual(
            api.requests[0]["path"],
            f"/agent/context/manager-daily-brief?location_id={LOCATION_ID}&operating_day={OPERATING_DAY}",
        )
        self.assertEqual(result.stderr, "")

    def test_submit_manager_daily_brief_draft_posts_stdin_json_to_validation_endpoint(self):
        draft = {
            "context_packet_id": f"manager-daily-brief-context:{LOCATION_ID}:{OPERATING_DAY}",
            "correlation_id": f"manager-daily-brief:{LOCATION_ID}:{OPERATING_DAY}",
            "submitted_by": "hermes-agent",
            "actions": [],
        }
        with FakePetResortApi() as api:
            result = self.run_tool(
                "submit_manager_daily_brief_draft",
                env={"PET_RESORT_API_URL": api.url},
                input_json=draft,
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["validation"]["status"], "accepted")
        self.assertEqual(api.requests[0]["method"], "POST")
        self.assertEqual(api.requests[0]["path"], "/agent/drafts/manager-daily-brief")
        self.assertEqual(api.requests[0]["body"], draft)

    def test_record_manager_daily_brief_outcome_posts_reviewed_feedback_for_action(self):
        outcome = {
            "outcome": "completed",
            "actual_minutes": 12,
            "actor": {"id": "front-desk-lead-17", "persona": "front_desk_lead"},
            "feedback": "Resolved before checkout rush.",
            "source_refs": [],
            "timestamp": "2026-06-17T13:15:00Z",
            "audit": {"correlation_id": f"manager-daily-brief:{LOCATION_ID}:{OPERATING_DAY}"},
            "reporting": {"location_id": LOCATION_ID, "operating_day": OPERATING_DAY},
            "requested_side_effects": [],
        }
        with FakePetResortApi() as api:
            result = self.run_tool(
                "record_manager_daily_brief_outcome",
                "--action-id",
                "checkout-exception-reservation-4242",
                env={"PET_RESORT_API_URL": api.url},
                input_json=outcome,
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(json.loads(result.stdout)["accepted"])
        self.assertEqual(api.requests[0]["method"], "POST")
        self.assertEqual(
            api.requests[0]["path"],
            "/manager-daily-brief/actions/checkout-exception-reservation-4242/outcome",
        )
        self.assertEqual(api.requests[0]["body"], outcome)

    def test_bridge_error_output_is_safe_and_does_not_print_tokens(self):
        secret = "super-secret-agent-token"
        with FakePetResortApi(fail_status=500) as api:
            result = self.run_tool(
                "get_manager_daily_brief_context",
                "--location-id",
                LOCATION_ID,
                "--operating-day",
                OPERATING_DAY,
                env={"PET_RESORT_API_URL": api.url, "PET_RESORT_API_TOKEN": secret},
            )

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("pet resort api request failed", result.stderr.lower())
        self.assertNotIn(secret, result.stderr)
        self.assertNotIn(secret, result.stdout)
        self.assertEqual(api.requests[0]["authorization"], f"Bearer {secret}")


if __name__ == "__main__":
    unittest.main()
