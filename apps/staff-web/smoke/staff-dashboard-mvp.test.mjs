import { readFileSync } from "node:fs";
import { test } from "node:test";
import assert from "node:assert/strict";

const page = readFileSync(new URL("../app/page.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("../app/globals.css", import.meta.url), "utf8");
const demoData = readFileSync(new URL("../app/owned-platform-demo-data.ts", import.meta.url), "utf8");
const localDemoApiRoute = readFileSync(new URL("../app/api/local-demo/[...path]/route.ts", import.meta.url), "utf8");
const surface = `${page}\n${demoData}`;

const literalPattern = (text) => new RegExp(text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i");
const primaryPageCopy = page.replace(/<details className="proof-drawer">[\s\S]*?<\/details>/, "");

const assertContainsAll = (text, expectedValues) => {
  for (const expected of expectedValues) {
    assert.match(text, literalPattern(expected));
  }
};

test("surface is framed as an owned operations platform, not a single report", () => {
  assertContainsAll(surface, [
    "Owned Operations Platform",
    "Sample portfolio operating risk",
    "$25.1k modeled monthly labor + rework exposure",
    "48 manager minutes shifted from source chasing to review",
    "4 reusable tools on one owned backend",
    "0 live side effects: sends, PMS writes, schedule changes, payments, medical decisions",
    "sample workspace",
    "read-only source evidence",
    "NVA-owned operating facts",
    "workflow packets",
    "review gates",
    "Tool portfolio on the same backend",
    "source systems remain evidence",
    "Safe pilot ask"
  ]);
});

test("typed demo data models sources, spine, tools, lineage, locks, proofs, and pilot ask", () => {
  assertContainsAll(demoData, [
    "export type SourceEvidenceCard",
    "export type OwnedBackendSpineStage",
    "export type LaborToolPortfolioCard",
    "export type LineageEdge",
    "export type SafetyLock",
    "export type ProofArtifact",
    "export type CloseCard",
    "export type CloseListItem",
    "sourceEvidenceCards",
    "ownedBackendSpineStages",
    "laborToolPortfolioCards",
    "lineageEdges",
    "safetyLocks",
    "proofArtifacts",
    "pilotAskItems",
    "roiPilotCloseCards",
    "safeNextAskItems",
    "notAskingItems",
    "pilotSuccessCriteria"
  ]);
});

test("tool portfolio contains the four required reusable tools and their required fields", () => {
  assertContainsAll(demoData, [
    "Manager Daily Brief",
    "Data Quality Hygiene",
    "Intake / Booking Triage",
    "BI / Read Model Reporting"
  ]);

  const requiredFields = [
    /sourceSignals:\s*\[/,
    /normalizedFacts:\s*\[/,
    /workflowPacket:/,
    /reviewGate:/,
    /lockedSideEffects:\s*\[/,
    /outputReadout:/,
    /outcomeMetric:/,
    /proofHooks:\s*\[/
  ];

  for (const fieldPattern of requiredFields) {
    const matches = demoData.match(new RegExp(fieldPattern.source, "g")) ?? [];
    assert.ok(matches.length >= 4, `${fieldPattern} should appear for each tool`);
  }
});

const lineageExamplesByTool = {
  "Manager Daily Brief": [
    "PMS export row: 12 arrivals before 10",
    "SourceSnapshot pms_sample/reservation_snapshot/2026-07-03",
    "arrival_density fact",
    "manager action packet",
    "outcome record labor_rework"
  ],
  "Data Quality Hygiene": [
    "unclear rabies proof",
    "source_quality_issue fact",
    "reviewer packet",
    "wrong-source/resolved disposition",
    "source_quality_backlog"
  ],
  "Intake / Booking Triage": [
    "intake message: two dogs for holiday boarding",
    "missing-info checklist",
    "safe draft reply",
    "send locked",
    "intake_queue readout"
  ],
  "BI / Read Model Reporting": [
    "provider-shaped export",
    "NVA read model",
    "portfolio labor/rework trend",
    "KPI owner review",
    "portfolio_operations projection"
  ]
};

test("each selected tool has concrete source-to-fact-to-output lineage examples", () => {
  for (const [toolName, expectedValues] of Object.entries(lineageExamplesByTool)) {
    assert.match(demoData, literalPattern(toolName));
    assertContainsAll(demoData, expectedValues);
  }
});

test("inspectable tool cards reveal lineage from source signals through proof", () => {
  assertContainsAll(surface, [
    "source signals",
    "normalized NVA facts",
    "workflow packet / read model",
    "review gate",
    "locked side effects",
    "output/action/readout",
    "outcome metric",
    "proof hooks",
    "source evidence -> owned fact -> workflow packet -> review gate -> outcome/read model",
    "workflow_packet_id",
    "review_gate_id",
    "read_model_projection",
    "audit_event_id"
  ]);
});

test("read-only source evidence exposes source refs, freshness, caveats, and raw signals", () => {
  assertContainsAll(surface, [
    "PMS reservation feed sample",
    "Labor schedule / timeclock export sample",
    "Uploaded document sample",
    "Room inventory projection sample",
    "BI query inventory",
    "source refs preserved",
    "freshness + caveats visible",
    "raw signal",
    "source ref",
    "freshness",
    "caveat"
  ]);
});

test("owned backend spine keeps operating authority in NVA-owned reviewable layers", () => {
  assertContainsAll(surface, [
    "NVA keeps the work rules, review decisions, labor outcomes, and reporting meaning in its own operating layer.",
    "Read-only source evidence",
    "NVA-owned operating facts",
    "Workflow packets",
    "Review gates",
    "Audit + outcome events",
    "Read models for BI"
  ]);
});

test("safety locks are visible as product behavior with no live side effects", () => {
  assertContainsAll(`${primaryPageCopy}\n${demoData}`, [
    "write locked",
    "manager review open",
    "outbox candidate only",
    "customer send locked",
    "PMS/provider write locked",
    "schedule change locked",
    "payment/refund/discount locked",
    "medical/safety decision locked",
    "staffing mandate locked"
  ]);

  assert.match(primaryPageCopy, /outbox[^\n.]*candidate/i);
});

test("safe pilot ask stays read-only and reachable from the primary surface", () => {
  assertContainsAll(`${primaryPageCopy}\n${demoData}`, [
    "Safe pilot ask",
    "One pilot slice",
    "read-only exports",
    "field dictionaries",
    "BI query inventory",
    "Source snapshots or sample rows",
    "One or two workflows to validate",
    "Dual-run against current workflow before any write path"
  ]);
});

test("executive close frames modeled ROI, safe next ask, excluded actions, and pilot success", () => {
  assertContainsAll(`${primaryPageCopy}\n${demoData}`, [
    "Executive close",
    "Pilot ask: prove one read-only workflow slice before any write path exists.",
    "One-location modeled value",
    "$25.1k/mo",
    "Portfolio scale lens",
    "170-location scaler",
    "planning scaler, not a guarantee",
    "Assumptions visible",
    "sample + read-only",
    "Safe next ask",
    "approved read-only exports",
    "field dictionaries",
    "sample rows/docs",
    "BI query inventory",
    "one or two workflows to validate",
    "Not asking for",
    "live customer sends",
    "PMS/provider writes",
    "payment/refund/discount actions",
    "schedule changes",
    "medical/safety decisions",
    "staffing mandate action",
    "Pilot success criteria",
    "source mapping confidence",
    "manager action usefulness",
    "minutes saved / rework avoided",
    "wrong-source findings",
    "read-model comparison against current BI"
  ]);
});

test("visible surface rejects stale/meta framing and live-access claims", () => {
  for (const forbidden of [
    /DEMO MODE/i,
    /proper demo page/i,
    /presenter/i,
    /talk track/i,
    /Comprehensive Summary/i,
    /Migration Strategy/i,
    /architecture story/i,
    /<h1>\s*Daily Manager Report\s*<\/h1>/i,
    /CEO cost reduction daily report demo/i,
    /What makes it real/i,
    /live NVA access/i,
    /live Gingr access/i,
    /production credentials/i,
    /production data/i,
    /enabled customer sends/i,
    /enabled PMS\/provider writes/i,
    /enabled schedule changes/i,
    /enabled payments/i,
    /autonomous medical\/safety decisions/i,
    /staffing mandates/i,
    /\bDTOs?\b/i
  ]) {
    assert.doesNotMatch(primaryPageCopy, forbidden);
  }
});

test("visual layout supports platform cockpit lanes and proof mode", () => {
  assertContainsAll(page + styles, [
    "ceo-board",
    "metric-strip",
    "operating-flow",
    "flow-step source-flow",
    "flow-step backend-flow",
    "flow-step tools-flow",
    "flow-step locked-flow",
    "workspace-grid",
    "source-evidence-panel",
    "spine-panel",
    "tool-portfolio-panel",
    "lineage-panel",
    "safety-locks-panel",
    "pilot-ask-strip",
    "executive-close",
    "roi-card-row",
    "close-list-grid",
    "tool-card",
    "tool-lineage-rail",
    "source-card",
    "spine-stage",
    "proof-drawer"
  ]);
});

test("selected tool interaction is stateful, labeled, and visibly selected", () => {
  assert.match(page, /useState\(laborToolPortfolioCards\[0\]\.id\)/);
  assert.match(page, /setSelectedToolId\(tool\.id\)/);
  assert.match(page, /aria-pressed=\{selectedToolId === tool\.id\}/);
  assert.match(page, /aria-label=\{`Inspect \$\{tool\.name\} lineage`\}/);
  assert.match(page, /role="group"/);
  assert.match(page, /Choose a reusable labor tool to inspect/);
  assert.match(page, /selected-indicator/);
  assert.match(styles, /\.tool-card\.selected[\s\S]*box-shadow/);
  assert.match(styles, /\.tool-card:focus-visible/);
  assert.match(page, /selectedTool\.lineageSteps\.map/);
});

test("first-screen visual hierarchy and responsive accessibility are regression hardened", () => {
  assertContainsAll(`${page}\n${styles}`, [
    "story-pills",
    "Evidence stays read-only",
    "Owned backend creates reviewable work",
    "Four tools reuse it",
    "Side effects stay locked",
    "prefers-reduced-motion",
    "@media (max-width: 920px)",
    "@media (max-width: 560px)",
    "aria-label=\"Platform story at a glance\""
  ]);
});

test("primary surface keeps architecture proof language secondary to product cockpit labels", () => {
  assert.doesNotMatch(primaryPageCopy, /Operations API contract/i);
  assert.doesNotMatch(primaryPageCopy, /architecture/i);
  assert.doesNotMatch(primaryPageCopy, /\bDTOs?\b/i);

  const technicalTokens = primaryPageCopy.match(/provenance_snapshot_id|workflow_packet_id|review_gate_id|outbox_candidate_id|read_model_projection|audit_event_id/g) ?? [];
  assert.ok(technicalTokens.length <= 14, `expected concise proof tokens on primary surface, found ${technicalTokens.length}`);

  for (const productCue of ["modeled monthly labor", "manager minutes", "read-only source evidence", "Tool portfolio on the same backend", "write locked"]) {
    assert.match(`${primaryPageCopy}\n${demoData}`, literalPattern(productCue));
  }
});

test("proof drawer preserves repo-backed technical/evidence package below the primary CEO view", () => {
  assert.match(page, /<details className="proof-drawer">/);
  assertContainsAll(`${page}\n${demoData}`, [
    "Proof behind the platform",
    "source adapter / provenance boundary",
    "owned operations API / OpenAPI",
    "storage / projections / outcome records",
    "review gates / blocked actions",
    "audit / outbox posture",
    "read models / BI replacement",
    "local / synthetic smoke proof",
    "what exists now",
    "synthetic / no-access boundary",
    "what real access would validate",
    "where to inspect",
    "apps/api/openapi/owned-operations-v0.openapi.json",
    "migrations/0001_mvp_foundation.sql",
    "apps/api/src/http.rs",
    "app/src/manager_daily_brief.rs",
    "app/src/data_quality_hygiene.rs",
    "app/src/booking_triage.rs",
    "storage/src/operations.rs",
    "docs/architecture/runtime-contract-boundaries.md",
    "docs/architecture/audit-reporting-evidence-backbone.md",
    "scripts/demo_owned_operations_api.sh",
    "docs/demo/local-demo-walkthrough.md",
    "Operations API contract",
    "provenance_snapshot_id",
    "workflow_packet_id",
    "review_gate_id",
    "outbox_candidate_id",
    "read_model_projection",
    "audit_event_id",
    "live_side_effects_allowed=false",
    "blocked-action policy",
    "audit/outcome events",
    "Static repo-backed proof is safer for the public page than depending on a private local service"
  ]);
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
