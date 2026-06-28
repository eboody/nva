import { readFileSync } from "node:fs";
import { test } from "node:test";
import assert from "node:assert/strict";

const page = readFileSync(new URL("../app/page.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("../app/globals.css", import.meta.url), "utf8");
const localDemoApiRoute = readFileSync(new URL("../app/api/local-demo/[...path]/route.ts", import.meta.url), "utf8");

test("manager brief demo shows the full source-to-brief contract", () => {
  for (const expected of [
    "Manager Daily Brief",
    "collected facts",
    "Reservation",
    "Pet profile",
    "Rabies proof",
    "Capacity",
    "Labor plan",
    "ranked action plan",
    "Review boarding vs labor",
    "Clear rabies document",
    "Quiet-room plan for Miso",
    "48",
    "min saved"
  ]) {
    assert.match(page, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"));
  }

  assert.doesNotMatch(page, /architecture story/i);
  assert.doesNotMatch(page, /owned backend migration map/i);
  assert.doesNotMatch(page, /technical proof/i);
});

test("demo steps narrate the human manager workflow", () => {
  for (const expected of [
    "messy morning",
    "facts tracked",
    "manager brief",
    "review recorded",
    "step-explainer"
  ]) {
    assert.match(page, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"));
  }
});

test("before the brief strip shows concrete morning pain", () => {
  for (const expected of [
    "Before the brief:",
    "7:20am lobby rush",
    "12 arrivals before 10",
    "rabies proof unclear",
    "coverage 2 short",
    "quiet-room request buried",
    "chaos-strip"
  ]) {
    assert.match(page + styles, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"));
  }
});

test("brief action schema keeps the thought-through pieces visible", () => {
  for (const expected of [
    "source ref",
    "field path",
    "freshness",
    "quality flag",
    "review gate",
    "labor estimate",
    "manager approval",
    "document review",
    "review ready",
    "before",
    "after"
  ]) {
    assert.match(page, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"));
  }
});

test("safety remains visual and side effects stay blocked", () => {
  for (const expected of [
    "synthetic",
    "review-gated",
    "customer send",
    "PMS write",
    "locked",
    "manager review",
    "ready",
    "record review",
    "outcome recorded"
  ]) {
    assert.match(page, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"));
  }
});

test("visual layout contains collection, tracking, brief, and proof panels", () => {
  for (const expectedClass of [
    "brief-lab",
    "source-board",
    "source-card",
    "assembly-column",
    "pipeline-card",
    "gate-card",
    "manager-brief",
    "brief-action",
    "proof-board",
    "approval-button"
  ]) {
    assert.match(styles + page, new RegExp(expectedClass, "i"));
  }
});

test("local demo API proxy still rejects path traversal before upstream fetch", () => {
  assert.match(localDemoApiRoute, /function safeLocalDemoApiPath/);
  assert.match(localDemoApiRoute, /segments\[0\] !== allowedPathRoot/);
  assert.match(localDemoApiRoute, /segment === "\."/);
  assert.match(localDemoApiRoute, /segment === "\.\."/);
  assert.match(localDemoApiRoute, /segment\.includes\("\/"\)/);
  assert.match(localDemoApiRoute, /encodeURIComponent\(segment\)/);
  assert.doesNotMatch(localDemoApiRoute, /fetch\(`\$\{apiBaseUrl\}\/\$\{path\}`/);
});
