import { readFileSync } from "node:fs";
import { test } from "node:test";
import assert from "node:assert/strict";

const page = readFileSync(new URL("../app/page.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("../app/globals.css", import.meta.url), "utf8");
const localDemoApiRoute = readFileSync(new URL("../app/api/local-demo/[...path]/route.ts", import.meta.url), "utf8");

const literalPattern = (text) => new RegExp(text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i");

const primaryPageCopy = page.replace(/<details className="proof-drawer">[\s\S]*?<\/details>/, "");

test("manager brief workspace shows the full source-to-brief contract", () => {
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
    assert.match(page, literalPattern(expected));
  }
});

test("primary page copy stays product-first, not architecture-first", () => {
  for (const forbidden of [
    /owned backend migration map/i,
    /architecture story/i,
    /\bDTOs?\b/i,
    /read model replacement strategy/i,
    /technical proof first/i
  ]) {
    assert.doesNotMatch(primaryPageCopy, forbidden);
  }
});

test("workflow steps narrate the human manager workflow", () => {
  for (const expected of [
    "messy morning",
    "facts tracked",
    "manager brief",
    "review recorded",
    "step-explainer"
  ]) {
    assert.match(page, literalPattern(expected));
  }
});

test("primary page copy keeps the before-after-safety-next-ask anchors", () => {
  for (const expected of [
    "Before the brief:",
    "messy morning",
    "Manager Daily Brief",
    "review recorded",
    "Sample workspace",
    "What real access would unlock"
  ]) {
    assert.match(primaryPageCopy, literalPattern(expected));
  }
});

test("primary screen reads like software in use, not a labeled sales artifact", () => {
  for (const expected of [
    "Pet resort ops",
    "turn the morning mess into reviewed work",
    "Sample Pet Resort",
    "open risks",
    "safe actions",
    "System actions",
    "customer message locked",
    "PMS update locked",
    "manager review open"
  ]) {
    assert.match(primaryPageCopy, literalPattern(expected));
  }

  for (const forbidden of [
    /DEMO MODE/i,
    /proper demo page/i,
    /Show this safely/i,
    /No production connection/i,
    /synthetic fixture only/i,
    /Do not claim this is live/i,
    /presenter/i,
    /talk track/i
  ]) {
    assert.doesNotMatch(primaryPageCopy, forbidden);
  }

  for (const expectedClass of [
    "app-frame",
    "command-bar",
    "shift-console",
    "action-console"
  ]) {
    assert.match(page + styles, literalPattern(expectedClass));
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
    "review gate",
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

test("sample workspace card names blocked boundaries and validation next step", () => {
  for (const expected of [
    "blockedBoundaries",
    "honesty-card",
    "Sample workspace",
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
    "source refs/caveats attach",
    "estimated vs reviewed minutes",
    "sample data stays contained until read-only exports",
    "local operations API proof",
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
