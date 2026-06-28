import { readFileSync } from "node:fs";
import { test } from "node:test";
import assert from "node:assert/strict";

const page = readFileSync(new URL("../app/page.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("../app/globals.css", import.meta.url), "utf8");
const localDemoApiRoute = readFileSync(new URL("../app/api/local-demo/[...path]/route.ts", import.meta.url), "utf8");

test("visual demo starts from one workflow instead of an architecture pitch", () => {
  for (const expected of [
    "Miso → manager brief",
    "Board July 3–7?",
    "rabies attached?",
    "noise-sensitive",
    "add enrichment?",
    "23",
    "min saved",
    "approve draft"
  ]) {
    assert.match(page, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"));
  }

  assert.doesNotMatch(page, /architecture story/i);
  assert.doesNotMatch(page, /owned backend migration map/i);
  assert.doesNotMatch(page, /technical proof/i);
});

test("show-dont-tell version keeps copy intentionally sparse", () => {
  const visibleWords = page
    .replace(/<[^>]+>/g, " ")
    .replace(/[{}()[\]=>?:;.,`"'|/]/g, " ")
    .split(/\s+/)
    .filter((word) => /^[A-Za-z][A-Za-z-]*$/.test(word));

  assert.ok(visibleWords.length < 450, `page has too many visible/candidate words: ${visibleWords.length}`);
});

test("safety is visual and terse", () => {
  for (const expected of [
    "no live sends",
    "no PMS writes",
    "send",
    "PMS",
    "locked",
    "review",
    "ready",
    "0 unsafe"
  ]) {
    assert.match(page, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"));
  }
});

test("visual layout contains the staged product scene", () => {
  for (const expectedClass of [
    "visual-flow",
    "phone-card",
    "work-packet",
    "gate-panel",
    "brief-card",
    "audit-dots",
    "pulse-ring"
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
