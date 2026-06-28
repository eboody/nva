import { readFileSync } from "node:fs";
import { test } from "node:test";
import assert from "node:assert/strict";

const page = readFileSync(new URL("../app/page.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("../app/globals.css", import.meta.url), "utf8");
const localDemoApiRoute = readFileSync(new URL("../app/api/local-demo/[...path]/route.ts", import.meta.url), "utf8");

const literalPattern = (text) => new RegExp(text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i");
const primaryPageCopy = page.replace(/<details className="proof-drawer">[\s\S]*?<\/details>/, "");

test("surface is framed for a CEO cost-reduction daily report", () => {
  for (const expected of [
    "Portfolio cost control",
    "Daily Manager Report",
    "CEO wants",
    "fewer wasted manager hours",
    "modeled monthly cost",
    "$25.1k",
    "manager prep removed",
    "live side effects",
    "0"
  ]) {
    assert.match(page, literalPattern(expected));
  }
});

test("generated report contains concrete ranked cost levers", () => {
  for (const expected of [
    "Today’s cost-reduction brief",
    "3 ranked levers",
    "Labor risk before 10am",
    "Vaccine/document rework",
    "Premium room constraint",
    "2 short · 12 arrivals",
    "31 staff-hours recoverable",
    "$6.7k retention-risk protected"
  ]) {
    assert.match(page, literalPattern(expected));
  }
});

test("selected report line separates CEO readout from manager action", () => {
  for (const expected of [
    "CEO readout",
    "executiveReadout",
    "managerAction",
    "modeled impact",
    "manager gets this action",
    "Move one cross-trained lead",
    "Pre-clear two documents",
    "Hold quiet room for Miso"
  ]) {
    assert.match(page, literalPattern(expected));
  }
});

test("evidence chain shows where each report piece comes from", () => {
  for (const expected of [
    "Evidence chain",
    "Why this appeared in the report",
    "PMS reservation feed",
    "labor schedule / timeclock",
    "uploaded vaccine documents",
    "pet profile + stay notes",
    "room inventory projection",
    "raw signal",
    "created fact",
    "report use"
  ]) {
    assert.match(page, literalPattern(expected));
  }
});

test("report factory explains how pieces are created before appearing in report", () => {
  for (const expected of [
    "Report factory",
    "How the pieces get created",
    "read source record",
    "preserve provenance",
    "normalize into operating fact",
    "score labor/cost impact",
    "rank manager action",
    "lock unsafe side effects",
    "record outcome",
    "selected lineage"
  ]) {
    assert.match(page, literalPattern(expected));
  }
});

test("source facts include provenance, transformation, gate, and contribution", () => {
  for (const expected of [
    "read-only source adapter → reservation fact",
    "schedule import → labor variance model",
    "OCR/extraction → document quality flag",
    "note parser → care constraint fact",
    "ops projection → caveated capacity fact",
    "no PMS write",
    "manager owns staffing choice",
    "human document review",
    "no autonomous care decision",
    "projection caveat visible"
  ]) {
    assert.match(page, literalPattern(expected));
  }
});

test("safety is visible without pretending to have live access", () => {
  for (const expected of [
    "sample workspace",
    "read-only inputs · writes locked",
    "customer send locked",
    "PMS write locked",
    "schedule change locked",
    "read-only exports",
    "field dictionaries",
    "BI query inventory"
  ]) {
    assert.match(primaryPageCopy, literalPattern(expected));
  }

  for (const forbidden of [
    /DEMO MODE/i,
    /proper demo page/i,
    /presenter/i,
    /talk track/i,
    /Comprehensive Summary/i,
    /Migration Strategy/i,
    /architecture story/i,
    /\bDTOs?\b/i
  ]) {
    assert.doesNotMatch(primaryPageCopy, forbidden);
  }
});

test("visual layout supports report, executive readout, evidence, and factory panels", () => {
  for (const expectedClass of [
    "ceo-board",
    "metric-strip",
    "workspace-grid",
    "report-panel",
    "executive-panel",
    "evidence-panel",
    "factory-panel",
    "report-line",
    "evidence-card",
    "pipeline-list",
    "proof-drawer"
  ]) {
    assert.match(page + styles, literalPattern(expectedClass));
  }
});

test("proof drawer preserves technical/evidence package below the primary CEO view", () => {
  assert.match(page, /<details className="proof-drawer">/);
  assert.match(page, /<summary>Proof package behind this screen<\/summary>/);
  for (const expected of [
    "Source contracts",
    "Cost model",
    "Safety posture",
    "Next validation",
    "source name, raw signal, normalized fact, caveat/gate",
    "labor/rework/retention value",
    "customer sends, PMS writes, schedule changes",
    "real NVA/Gingr operating data"
  ]) {
    assert.match(page, literalPattern(expected));
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
