#!/usr/bin/env python3
"""Validate the checked-in public Rustdoc landing page source."""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LANDING = ROOT / "docs" / "public" / "index.html"

html = LANDING.read_text(encoding="utf-8")
lower = html.lower()

required_fragments = [
    "start with the pet-resort entity atlas",
    "entity-first operating model for labor-cost reduction",
    "browse entity index",
    "choose a reading path",
    "workflow-to-entity map",
    "operations leader",
    "ai program evaluator",
    "safety/compliance reviewer",
    "gingr/integration owner",
    "rustdocs as code-derived evidence",
    "operational translation",
    "technical surface",
    "domain/",
    "app/",
    "storage/",
    "gingr/",
    "pet_resort_api/",
    "pet_resort_worker/",
    "cli/",
]
missing = [fragment for fragment in required_fragments if fragment not in lower]
if missing:
    raise SystemExit(f"{LANDING}: missing required landing fragments: {missing}")

first_audience = lower.index("audience paths")
first_rustdoc = lower.index("rustdocs as code-derived evidence")
if first_audience > first_rustdoc:
    raise SystemExit(f"{LANDING}: audience paths must appear before the Rustdoc crate list")

for jargon_only_label in [
    "semantic pet-resort service and operations contracts",
    "review-gated agent workflows and tool ports",
    "persistence boundary contracts",
    "source adapter/request/webhook boundaries",
]:
    if jargon_only_label in lower:
        raise SystemExit(
            f"{LANDING}: found old jargon-first crate label without operational translation: "
            f"{jargon_only_label!r}"
        )

hrefs = re.findall(r'href="([^"]+)"', html)
required_hrefs = {
    "domain/",
    "app/",
    "storage/",
    "gingr/",
    "pet_resort_api/",
    "pet_resort_worker/",
    "cli/",
}
missing_hrefs = sorted(required_hrefs - set(hrefs))
if missing_hrefs:
    raise SystemExit(f"{LANDING}: missing required crate hrefs: {missing_hrefs}")

print("public docs landing source check passed")
