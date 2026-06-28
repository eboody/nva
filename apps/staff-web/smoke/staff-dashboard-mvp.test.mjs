import { readFileSync } from "node:fs";
import { test } from "node:test";
import assert from "node:assert/strict";

const page = readFileSync(new URL("../app/page.tsx", import.meta.url), "utf8");
const localDemoApiRoute = readFileSync(new URL("../app/api/local-demo/[...path]/route.ts", import.meta.url), "utf8");

test("staff demo leads with a concrete show-not-tell operator workflow", () => {
  for (const expected of [
    "Watch one messy pet-resort request become a safe manager action plan",
    "Avery: Can Miso board July 3–7",
    "Miso • Boarding July 3–7",
    "Manager Daily Brief",
    "23 min",
    "estimated labor removed today",
    "Step {stepNumber} / 4",
    "Simulate staff approval"
  ]) {
    assert.match(page, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"), `missing show-not-tell evidence: ${expected}`);
  }
});

test("sensitive actions remain visibly blocked or review-gated", () => {
  for (const expected of [
    "no live NVA data",
    "no customer sends",
    "no PMS writes",
    "No availability promised",
    "Confirm or reject booking",
    "Approve medical/vaccine record",
    "Change capacity or staff schedule",
    "Send customer messages",
    "0 unsafe automations enabled",
    "cannot send, confirm, charge, or mutate provider systems"
  ]) {
    assert.match(page, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"), `missing safety boundary: ${expected}`);
  }
});

test("demo honestly distinguishes live/local API proof from fallback fixtures", () => {
  for (const expected of [
    "Live/local API proof is connected",
    "Fallback mode is honestly labeled",
    "Live local API data",
    "Fallback fixture data",
    "API unavailable or unconfigured; page does not claim DB evidence",
    "DB-backed read-model records",
    "Static fallback rows",
    "PET_RESORT_API_BASE_URL not configured or API unreachable",
    "/v0/readyz",
    "/v0/ops/metrics/summary",
    "/v0/read-models/source-quality-backlog",
    "/v0/agent/context/manager-daily-brief"
  ]) {
    assert.match(page, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"), `missing live/fallback posture evidence: ${expected}`);
  }
});

test("job-contact close frames the no-access prototype without overclaiming", () => {
  for (const expected of [
    "I did not have access, so I built the safe seam first",
    "What is strong now",
    "What it does not claim",
    "What access would unlock",
    "No production NVA/Gingr data",
    "Read-only source snapshots",
    "one instrumented pilot lane"
  ]) {
    assert.match(page, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"), `missing no-access presentation framing: ${expected}`);
  }
});

test("local demo API proxy rejects path traversal before upstream fetch", () => {
  assert.match(localDemoApiRoute, /function safeLocalDemoApiPath/);
  assert.match(localDemoApiRoute, /segments\[0\] !== allowedPathRoot/);
  assert.match(localDemoApiRoute, /segment === "\."/);
  assert.match(localDemoApiRoute, /segment === "\.\."/);
  assert.match(localDemoApiRoute, /segment\.includes\("\/"\)/);
  assert.match(localDemoApiRoute, /encodeURIComponent\(segment\)/);
  assert.doesNotMatch(localDemoApiRoute, /fetch\(`\$\{apiBaseUrl\}\/\$\{path\}`/);
});
