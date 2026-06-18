#!/usr/bin/env python3
"""Rustdoc completeness guardrail for source-of-truth external docs.

The gate has two parts:
1. Run the strict missing-docs command documented in the source-of-truth plan.
2. Build rendered Rustdocs and smoke-check representative public pages so the
   generated site keeps showing the semantic contract surface humans rely on.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from dataclasses import dataclass
from html import unescape
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
DOC_ROOT = REPO_ROOT / "target" / "doc"
STRICT_COMMAND = ["cargo", "doc", "--workspace", "--no-deps"]
STRICT_ENV = {"RUSTDOCFLAGS": "-D missing_docs"}

# statum 0.9 currently expands public typestate artifacts from these two source
# attributes without inheriting source-level docs/allow attributes in a way that
# satisfies `RUSTDOCFLAGS='-D missing_docs'`. Keep this whitelist narrow: only
# missing-docs diagnostics emitted from the exact app booking-triage statum macro
# sites are accepted. Any new source item or macro site still fails the gate.
ALLOWED_STATUM_MISSING_DOCS_SITES = {
    ("app/src/booking_triage.rs", 116, "#[state]"),
    ("app/src/booking_triage.rs", 127, "#[machine]"),
}


@dataclass(frozen=True)
class RenderedDocExpectation:
    relative_path: str
    required_fragments: tuple[str, ...]


RENDERED_DOC_EXPECTATIONS = (
    RenderedDocExpectation(
        "app/agents/struct.AgentPromptPacket.html",
        (
            "Safe prompt-and-evidence packet exchanged with an automation agent.",
            "workflow_name",
            "Workflow identifier that ties the packet to a specific agent spec.",
            "policies",
            "Policy instructions the runner must include in the agent context.",
            "Name of the output schema expected from the agent.",
        ),
    ),
    RenderedDocExpectation(
        "app/agents/struct.AgentPromptPacketBuilder.html",
        (
            "Use builder syntax to set the inputs and finish with build()",
            "method.workflow_name",
            "Workflow identifier that ties the packet to a specific agent spec.",
            "method.policies",
            "Policy instructions the runner must include in the agent context.",
            "method.build",
        ),
    ),
    RenderedDocExpectation(
        "app/booking_triage/struct.Request.html",
        (
            "RequestStateTrait",
            "RequestIntakeBuilder",
            "__StatumRequestIntakeBuilderMissingSlot0Reservation",
            "method.attach_pet_profile",
            "Attaches pet profile evidence before the request can move to policy decisioning.",
            "method.attach_policy_snapshot",
            "method.mark_ready_for_policy_decision",
            "Returns the reservation carried by this booking-readiness workflow value.",
        ),
    ),
)


def run_command(command: list[str], *, extra_env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    if extra_env:
        env.update(extra_env)
    return subprocess.run(
        command,
        cwd=REPO_ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )


def relative_source(path: str) -> str:
    path = path.replace("\\", "/")
    marker = "/rustdoc-guardrails/"
    if marker in path:
        return path.split(marker, 1)[1]
    return path


def strict_missing_docs_failure_is_allowed(output: str) -> bool:
    error_count = output.count("error: missing documentation")
    allowed_count = 0
    for match in re.finditer(r"\s*--> (?P<path>[^:]+):(?P<line>\d+):\d+\n\s+\|\n\s*\d+ \| (?P<macro>#\[(?:state|machine)\])", output):
        site = (relative_source(match.group("path")), int(match.group("line")), match.group("macro"))
        if site in ALLOWED_STATUM_MISSING_DOCS_SITES:
            allowed_count += 1
        else:
            return False

    if error_count == 0 or allowed_count != error_count:
        return False

    disallowed_error_headers = [
        line
        for line in output.splitlines()
        if line.startswith("error:")
        and not line.startswith("error: missing documentation")
        and "could not document `app`" not in line
    ]
    return not disallowed_error_headers


def run_strict_missing_docs_gate() -> None:
    command_for_humans = "RUSTDOCFLAGS='-D missing_docs' cargo doc --workspace --no-deps"
    print(f"running strict rustdoc gate: {command_for_humans}", flush=True)
    result = run_command(STRICT_COMMAND, extra_env=STRICT_ENV)
    if result.returncode == 0:
        print("strict rustdoc missing-docs gate passed", flush=True)
        return

    if strict_missing_docs_failure_is_allowed(result.stdout):
        print(
            "accepted narrow statum-generated missing-docs exception: "
            "app/src/booking_triage.rs #[state]/#[machine] only",
            flush=True,
        )
        return

    print(result.stdout, file=sys.stderr)
    raise SystemExit(result.returncode)


def text_from_html(raw: str) -> str:
    without_tags = re.sub(r"<[^>]+>", " ", raw)
    return re.sub(r"\s+", " ", unescape(without_tags))


def smoke_check_rendered_docs() -> None:
    print("rendering rustdocs for HTML smoke checks: cargo doc --workspace --no-deps", flush=True)
    result = run_command(STRICT_COMMAND)
    if result.returncode != 0:
        print(result.stdout, file=sys.stderr)
        raise SystemExit(result.returncode)

    failures: list[str] = []
    for expectation in RENDERED_DOC_EXPECTATIONS:
        path = DOC_ROOT / expectation.relative_path
        if not path.exists():
            failures.append(f"missing rendered doc page: {path}")
            continue
        raw = path.read_text(encoding="utf-8")
        searchable = f"{raw}\n{text_from_html(raw)}"
        for fragment in expectation.required_fragments:
            if fragment not in searchable:
                failures.append(f"{expectation.relative_path}: missing fragment {fragment!r}")

    if failures:
        print("rendered rustdoc smoke check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        raise SystemExit(1)

    print("rendered rustdoc smoke check passed", flush=True)


def main() -> None:
    run_strict_missing_docs_gate()
    smoke_check_rendered_docs()


if __name__ == "__main__":
    main()
