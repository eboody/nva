"use client";

import { useState } from "react";

type EvidenceStatus = "ready" | "review" | "blocked";
type ReportLine = {
  id: string;
  title: string;
  costSignal: string;
  executiveReadout: string;
  managerAction: string;
  monthlyImpact: string;
  evidenceIds: string[];
  status: EvidenceStatus;
};

type EvidenceNode = {
  id: string;
  source: string;
  rawSignal: string;
  normalizedFact: string;
  reportContribution: string;
  createdBy: string;
  gate: string;
};

const statusLabels: Record<EvidenceStatus, string> = {
  ready: "manager-ready",
  review: "review-needed",
  blocked: "automation-locked"
};

const reportLines: ReportLine[] = [
  {
    id: "labor",
    title: "Labor risk before 10am",
    costSignal: "2 short · 12 arrivals · 4 enrichment adds",
    executiveReadout: "Morning coverage mismatch is the largest avoidable labor leak today.",
    managerAction: "Move one cross-trained lead to lobby until 10:30; defer non-critical enrichment calls.",
    monthlyImpact: "$18.4k modeled avoidable labor / rework",
    evidenceIds: ["reservations", "timeclock", "capacity"],
    status: "ready"
  },
  {
    id: "docs",
    title: "Vaccine/document rework",
    costSignal: "7 arrivals · 2 unclear rabies proofs",
    executiveReadout: "Front desk will spend the first rush chasing documents unless review starts now.",
    managerAction: "Pre-clear two documents and assign one front-desk owner before lobby opens.",
    monthlyImpact: "31 staff-hours recoverable",
    evidenceIds: ["documents", "reservations"],
    status: "review"
  },
  {
    id: "rooms",
    title: "Premium room constraint",
    costSignal: "quiet-room request · room inventory tight",
    executiveReadout: "One high-value stay has a preventable service-risk flag.",
    managerAction: "Hold quiet room for Miso; manager approves any trade-off before customer confirmation.",
    monthlyImpact: "$6.7k retention-risk protected",
    evidenceIds: ["profile", "capacity", "reservations"],
    status: "blocked"
  }
];

const evidenceNodes: EvidenceNode[] = [
  {
    id: "reservations",
    source: "PMS reservation feed",
    rawSignal: "12 arrivals before 10 · Jul 3 boarding · enrichment add-ons",
    normalizedFact: "arrival_density=high · revenue_services=4 · operating_day=2026-07-03",
    reportContribution: "Ranks labor-risk line and attaches affected reservations.",
    createdBy: "read-only source adapter → reservation fact",
    gate: "no PMS write"
  },
  {
    id: "timeclock",
    source: "labor schedule / timeclock",
    rawSignal: "AM role coverage: -2 vs forecast · kennel lead starts 11:00",
    normalizedFact: "coverage_gap=2 · role=front_desk+kennel · confidence=.84",
    reportContribution: "Turns demand into a costed staffing exception instead of a vague alert.",
    createdBy: "schedule import → labor variance model",
    gate: "manager owns staffing choice"
  },
  {
    id: "documents",
    source: "uploaded vaccine documents",
    rawSignal: "rabies attachment present · expiry field unreadable",
    normalizedFact: "document_state=needs_review · blocker=rabies_expiry",
    reportContribution: "Creates a pre-rush document-review task with owner and reason.",
    createdBy: "OCR/extraction → document quality flag",
    gate: "human document review"
  },
  {
    id: "profile",
    source: "pet profile + stay notes",
    rawSignal: "Miso noise-sensitive · quiet-room request buried in note",
    normalizedFact: "care_constraint=quiet_room · customer_value=risk_protect",
    reportContribution: "Promotes hidden care note into manager-visible room decision.",
    createdBy: "note parser → care constraint fact",
    gate: "no autonomous care decision"
  },
  {
    id: "capacity",
    source: "room inventory projection",
    rawSignal: "premium rooms tight · enrichment waitlist likely",
    normalizedFact: "capacity_pressure=medium_high · caveat=projection",
    reportContribution: "Explains why the top actions matter today, not next week.",
    createdBy: "ops projection → caveated capacity fact",
    gate: "projection caveat visible"
  }
];

const creationPipeline = [
  "read source record",
  "preserve provenance",
  "normalize into operating fact",
  "score labor/cost impact",
  "rank manager action",
  "lock unsafe side effects",
  "record outcome"
];

const boardMetrics = [
  { label: "modeled monthly cost", value: "$25.1k", sub: "labor + rework + retention risk" },
  { label: "manager prep removed", value: "48m", sub: "from source chasing to review" },
  { label: "live side effects", value: "0", sub: "sends / writes / schedule changes" }
];

export default function Home() {
  const [selectedReportId, setSelectedReportId] = useState(reportLines[0].id);
  const selectedLine = reportLines.find((line) => line.id === selectedReportId) ?? reportLines[0];
  const selectedEvidence = evidenceNodes.filter((node) => selectedLine.evidenceIds.includes(node.id));

  return (
    <main className="stage">
      <section className="ceo-board" aria-label="CEO cost reduction daily report demo">
        <header className="topbar">
          <div className="brand-mark">N</div>
          <div>
            <p className="eyebrow">Portfolio cost control</p>
            <h1>Daily Manager Report</h1>
            <p className="subtitle">what the CEO wants: fewer wasted manager hours, fewer avoidable misses, proof behind every recommendation</p>
          </div>
          <aside className="safety-chip" aria-label="Safety boundary">
            <b>sample workspace</b>
            <span>read-only inputs · writes locked</span>
          </aside>
        </header>

        <section className="metric-strip" aria-label="Executive impact metrics">
          {boardMetrics.map((metric) => (
            <article key={metric.label}>
              <span>{metric.label}</span>
              <strong>{metric.value}</strong>
              <small>{metric.sub}</small>
            </article>
          ))}
        </section>

        <section className="workspace-grid">
          <section className="report-panel panel" aria-label="Generated executive report">
            <div className="panel-head">
              <div>
                <p className="eyebrow">Generated 6:15am · Sample Pet Resort</p>
                <h2>Today’s cost-reduction brief</h2>
              </div>
              <span className="report-badge">3 ranked levers</span>
            </div>

            <div className="report-list">
              {reportLines.map((line, index) => (
                <button
                  key={line.id}
                  className={`report-line ${selectedReportId === line.id ? "selected" : ""}`}
                  onClick={() => setSelectedReportId(line.id)}
                  aria-pressed={selectedReportId === line.id}
                >
                  <b>{index + 1}</b>
                  <span className="line-main">
                    <strong>{line.title}</strong>
                    <small>{line.costSignal}</small>
                  </span>
                  <i className={`status-pill ${line.status}`}>{statusLabels[line.status]}</i>
                </button>
              ))}
            </div>
          </section>

          <section className="executive-panel panel" aria-label="Selected CEO readout">
            <p className="eyebrow">CEO readout</p>
            <h2>{selectedLine.title}</h2>
            <p className="readout">{selectedLine.executiveReadout}</p>
            <div className="impact-card">
              <span>modeled impact</span>
              <strong>{selectedLine.monthlyImpact}</strong>
            </div>
            <div className="manager-action">
              <span>manager gets this action</span>
              <p>{selectedLine.managerAction}</p>
            </div>
            <div className="locked-actions" aria-label="Locked actions">
              <span>customer send locked</span>
              <span>PMS write locked</span>
              <span>schedule change locked</span>
            </div>
          </section>

          <section className="evidence-panel panel" aria-label="Evidence used by selected report line">
            <div className="panel-head compact">
              <div>
                <p className="eyebrow">Evidence chain</p>
                <h2>Why this appeared in the report</h2>
              </div>
              <span>{selectedEvidence.length} sources</span>
            </div>
            <div className="evidence-stack">
              {selectedEvidence.map((node) => (
                <article className="evidence-card" key={node.id}>
                  <div className="source-title"><b>{node.source}</b><i>{node.gate}</i></div>
                  <dl>
                    <div><dt>raw signal</dt><dd>{node.rawSignal}</dd></div>
                    <div><dt>created fact</dt><dd>{node.normalizedFact}</dd></div>
                    <div><dt>report use</dt><dd>{node.reportContribution}</dd></div>
                  </dl>
                </article>
              ))}
            </div>
          </section>

          <section className="factory-panel panel" aria-label="How report pieces are created">
            <p className="eyebrow">Report factory</p>
            <h2>How the pieces get created</h2>
            <ol className="pipeline-list">
              {creationPipeline.map((step) => (
                <li key={step}>{step}</li>
              ))}
            </ol>
            <div className="artifact-card">
              <span>selected lineage</span>
              <strong>{selectedEvidence.map((node) => node.createdBy).join("  →  ")}</strong>
            </div>
          </section>
        </section>

        <section className="validation-strip" aria-label="What makes this real with approved access">
          <b>What makes it real</b>
          <span>read-only exports</span>
          <span>field dictionaries</span>
          <span>BI query inventory</span>
          <i>then the same report is generated from actual operating data, still with sends/writes locked</i>
        </section>

        <details className="proof-drawer">
          <summary>Proof package behind this screen</summary>
          <div className="proof-grid">
            <article>
              <b>Source contracts</b>
              <p>Each recommendation carries source name, raw signal, normalized fact, caveat/gate, and report contribution.</p>
            </article>
            <article>
              <b>Cost model</b>
              <p>The page separates executive impact, manager action, and measurable labor/rework/retention value.</p>
            </article>
            <article>
              <b>Safety posture</b>
              <p>The workflow can generate review-ready recommendations while customer sends, PMS writes, schedule changes, payments, and medical decisions remain locked.</p>
            </article>
            <article>
              <b>Next validation</b>
              <p>With read-only exports, field dictionaries, and BI query inventory, the same report can be grounded in real NVA/Gingr operating data without asking for write access.</p>
            </article>
          </div>
        </details>
      </section>
    </main>
  );
}
