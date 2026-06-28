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
    "before",
    "after"
  ]) {
    assert.match(page, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"));
  }
});

test("manager actions show review-ready and blocked statuses", () => {
  for (const expected of [
    "status: \"review-ready\"",
    "status: \"blocked\"",
    "review ready",
    "blocked",
    "status-badge",
    "status-review-ready",
    "status-blocked"
  ]) {
    assert.match(page + styles, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"));
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

test("no-access honesty card names blocked boundaries and validation next step", () => {
  for (const expected of [
    "blockedBoundaries",
    "honesty-card",
    "Built without live access",
    "no live NVA/Gingr data",
    "no customer sends",
    "no PMS writes",
    "no payment/schedule/medical decisions",
    "read-only validation"
  ]) {
    assert.match(page + styles, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"));
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

test("proof drawer keeps technical evidence behind the visual workflow", () => {
  assert.match(page, /<details className="proof-drawer">/);
  assert.match(page, /<summary>Proof behind the scene<\/summary>/);
  assert.match(page, /proofBullets/);

  for (const expected of [
    "staff-web smoke tests",
    "local API proof",
    "source refs/caveats attached",
    "estimated vs reviewed minutes",
    "synthetic fixture until read-only access approved",
    "./scripts/demo_owned_operations_api.sh",
    "side effects disabled",
    "outcome/labor proof"
  ]) {
    assert.match(page + styles, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"));
  }
});

test("next ask closes with narrow read-only real access request", () => {
  assert.match(page, /<section className="next-ask" aria-label="Safe real-access next ask">/);
  assert.match(page, /<h2>What real access would unlock<\/h2>/);

  for (const expected of [
    "next-ask",
    "read-only sample exports",
    "field dictionaries",
    "BI query inventory",
    "one workflow",
    "out of scope",
    "writes",
    "sends",
    "payments",
    "schedules",
    "medical/safety decisions"
  ]) {
    assert.match(page + styles, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"));
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
