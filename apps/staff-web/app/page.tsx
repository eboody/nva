"use client";

import { useState } from "react";
import {
  laborToolPortfolioCards,
  lineageEdges,
  notAskingItems,
  ownedBackendSpineStages,
  pilotAskItems,
  pilotSuccessCriteria,
  portfolioMetrics,
  proofArtifacts,
  roiPilotCloseCards,
  safeNextAskItems,
  safetyLocks,
  sourceEvidenceCards
} from "./owned-platform-demo-data";

const lineageLabel = "source evidence -> owned fact -> workflow packet -> review gate -> outcome/read model";

export default function Home() {
  const [selectedToolId, setSelectedToolId] = useState(laborToolPortfolioCards[0].id);
  const selectedTool = laborToolPortfolioCards.find((tool) => tool.id === selectedToolId) ?? laborToolPortfolioCards[0];
  const selectedLineage = lineageEdges.find((edge) => edge.toolId === selectedTool.id) ?? lineageEdges[0];

  return (
    <main className="stage">
      <section className="ceo-board" aria-label="Owned operations platform sample workspace">
        <header className="topbar">
          <div className="brand-mark">N</div>
          <div className="hero-copy">
            <p className="eyebrow">Sample workspace</p>
            <h1>Owned Operations Platform</h1>
            <p className="subtitle">
              Source systems remain evidence while NVA owns operating facts, workflow packets, review gates,
              and read models that power reusable labor tools with writes locked.
            </p>
            <ul className="story-pills" aria-label="Platform story at a glance">
              <li>Evidence stays read-only</li>
              <li>Owned backend creates reviewable work</li>
              <li>Four tools reuse it</li>
              <li>Side effects stay locked</li>
            </ul>
          </div>
          <aside className="safety-chip" aria-label="Safety boundary">
            <b>Access posture</b>
            <span>sample workspace · read-only sources · write locked · manager review open · outbox candidate only</span>
          </aside>
        </header>

        <section className="metric-strip" aria-label="Portfolio risk/value strip">
          {portfolioMetrics.map((metric) => (
            <article key={metric.label}>
              <span>{metric.label}</span>
              <strong>{metric.value}</strong>
              <small>{metric.sub}</small>
            </article>
          ))}
        </section>

        <section className="operating-flow" aria-label="Source to owned backend to tools flow">
          <article className="flow-step source-flow">
            <span>01</span>
            <b>Read-only source evidence</b>
            <small>PMS, labor, document, capacity, and BI samples keep source refs and caveats visible.</small>
          </article>
          <article className="flow-step backend-flow">
            <span>02</span>
            <b>NVA-owned operating facts</b>
            <small>Source signals become workflow packets, review gates, audit events, and BI-ready outcomes.</small>
          </article>
          <article className="flow-step tools-flow">
            <span>03</span>
            <b>Tool portfolio on the same backend</b>
            <small>Manager, data-quality, intake, and BI tools reuse the same operating layer.</small>
          </article>
          <article className="flow-step locked-flow">
            <span>04</span>
            <b>Write locked</b>
            <small>Manager review is open; customer sends, PMS writes, schedules, payments, and medical decisions stay locked.</small>
          </article>
        </section>

        <section className="workspace-grid" aria-label="Owned operations platform cockpit">
          <section className="source-evidence-panel panel" aria-label="Read-only source evidence">
            <div className="panel-head compact">
              <div>
                <p className="eyebrow">Read-only source evidence</p>
                <h2>source refs preserved</h2>
              </div>
              <span>freshness + caveats visible</span>
            </div>
            <div className="source-card-list">
              {sourceEvidenceCards.map((source) => (
                <article className="source-card" key={source.id}>
                  <div className="source-title"><b>{source.title}</b><i>{source.readOnlyState}</i></div>
                  <dl>
                    <div><dt>source name</dt><dd>{source.sourceName}</dd></div>
                    <div><dt>raw signal</dt><dd>{source.rawSignal}</dd></div>
                    <div><dt>source ref</dt><dd>{source.sourceRef}</dd></div>
                    <div><dt>freshness</dt><dd>{source.freshness}</dd></div>
                    <div><dt>caveat</dt><dd>{source.caveat}</dd></div>
                  </dl>
                </article>
              ))}
            </div>
          </section>

          <section className="spine-panel panel" aria-label="NVA-owned operating facts spine">
            <p className="eyebrow">NVA-owned operating facts</p>
            <h2>source evidence → owned facts → reviewable work → outcomes</h2>
            <p className="panel-copy">
              NVA keeps the work rules, review decisions, labor outcomes, and reporting meaning in its own operating layer.
            </p>
            <ol className="spine-list">
              {ownedBackendSpineStages.map((stage) => (
                <li className="spine-stage" key={stage.id}>
                  <b>{stage.title}</b>
                  <span>{stage.businessPurpose}</span>
                  <code>{stage.proofLabel}</code>
                </li>
              ))}
            </ol>
          </section>

          <section className="tool-portfolio-panel panel" aria-label="Tool portfolio on the same backend">
            <div className="panel-head compact">
              <div>
                <p className="eyebrow">Tool portfolio on the same backend</p>
                <h2>4 reusable tools on one owned backend</h2>
              </div>
              <span>{selectedTool.name}</span>
            </div>
            <div className="tool-card-list" role="group" aria-label="Choose a reusable labor tool to inspect">
              {laborToolPortfolioCards.map((tool) => (
                <button
                  className={`tool-card ${selectedToolId === tool.id ? "selected" : ""}`}
                  key={tool.id}
                  onClick={() => setSelectedToolId(tool.id)}
                  aria-pressed={selectedToolId === tool.id}
                  aria-label={`Inspect ${tool.name} lineage`}
                >
                  <span className="tool-card-heading">
                    <strong>{tool.name}</strong>
                    {selectedToolId === tool.id ? <span className="selected-indicator">Selected</span> : null}
                  </span>
                  <small>{tool.summary}</small>
                </button>
              ))}
            </div>
          </section>

          <section className="lineage-panel panel" aria-label="Selected tool lineage and proof hooks">
            <p className="eyebrow">Inspectable tool lineage</p>
            <h2>{selectedTool.name}</h2>
            <p className="lineage-label">{lineageLabel}</p>
            <div className="tool-detail-grid">
              <article><b>source signals</b>{selectedTool.sourceSignals.map((item) => <span key={item}>{item}</span>)}</article>
              <article><b>normalized NVA facts</b>{selectedTool.normalizedFacts.map((item) => <span key={item}>{item}</span>)}</article>
              <article><b>workflow packet / read model</b><span>{selectedTool.workflowPacket}</span></article>
              <article><b>review gate</b><span>{selectedTool.reviewGate}</span></article>
              <article><b>locked side effects</b>{selectedTool.lockedSideEffects.map((item) => <span key={item}>{item}</span>)}</article>
              <article><b>output/action/readout</b><span>{selectedTool.outputReadout}</span></article>
              <article><b>outcome metric</b><span>{selectedTool.outcomeMetric}</span></article>
              <article><b>proof hooks</b>{selectedTool.proofHooks.map((hook) => <span key={hook}>{hook}</span>)}</article>
            </div>
            <div className="tool-lineage-rail" aria-label={`${selectedTool.name} concrete lineage`}>
              {selectedTool.lineageSteps.map((step, index) => (
                <span key={step}><i>{String(index + 1).padStart(2, "0")}</i>{step}</span>
              ))}
            </div>
            <div className="artifact-card lineage-card">
              <span>selected lineage</span>
              <strong>{selectedLineage.source} → {selectedLineage.fact} → {selectedLineage.workflowPacket} → {selectedLineage.gate} → {selectedLineage.outcomeReadModel}</strong>
            </div>
          </section>

          <section className="safety-locks-panel panel" aria-label="Locked side effects and review gates">
            <p className="eyebrow">Locked side effects / review gates</p>
            <h2>write locked, manager review open</h2>
            <div className="locked-actions">
              {safetyLocks.map((lock) => (
                <span title={lock.reason} key={lock.id}>{lock.label}<span className="sr-only">: {lock.reason}</span></span>
              ))}
            </div>
          </section>
        </section>

        <section className="pilot-ask-strip" aria-label="Safe pilot ask">
          <b>Safe pilot ask</b>
          {pilotAskItems.map((item) => (
            <span title={item.detail} key={item.label}>{item.label}<span className="sr-only">: {item.detail}</span></span>
          ))}
        </section>

        <section className="executive-close panel" aria-label="Pilot value, next ask, and success measures">
          <div className="close-heading">
            <p className="eyebrow">Executive close</p>
            <h2>Pilot ask: prove one read-only workflow slice before any write path exists.</h2>
            <p>
              The next step is narrow: approved extracts and definitions, one or two workflows, manager review,
              and a dual-run against today’s BI/workflow baseline.
            </p>
          </div>

          <div className="roi-card-row" aria-label="Modeled value and scale assumptions">
            {roiPilotCloseCards.map((card) => (
              <article className="roi-close-card" key={card.label}>
                <span>{card.label}</span>
                <strong>{card.value}</strong>
                <small>{card.detail}</small>
              </article>
            ))}
          </div>

          <div className="close-list-grid">
            <article className="close-list-card ask-card">
              <h3>Safe next ask</h3>
              <ul>
                {safeNextAskItems.map((item) => (
                  <li key={item.label}><b>{item.label}</b><span>{item.detail}</span></li>
                ))}
              </ul>
            </article>

            <article className="close-list-card lock-card">
              <h3>Not asking for</h3>
              <ul>
                {notAskingItems.map((item) => (
                  <li key={item.label}><b>{item.label}</b><span>{item.detail}</span></li>
                ))}
              </ul>
            </article>

            <article className="close-list-card success-card">
              <h3>Pilot success criteria</h3>
              <ul>
                {pilotSuccessCriteria.map((item) => (
                  <li key={item.label}><b>{item.label}</b><span>{item.detail}</span></li>
                ))}
              </ul>
            </article>
          </div>
        </section>

        <details className="proof-drawer">
          <summary>Proof behind the platform</summary>
          <p className="proof-drawer-copy">
            Static repo-backed proof is safer for the public page than depending on a private local service:
            each card names what exists now, what stays synthetic/no-access, what real access would validate,
            and where to inspect the contract or smoke proof.
          </p>
          <div className="proof-grid">
            {proofArtifacts.map((artifact) => (
              <article className="proof-card" key={artifact.id}>
                <b>{artifact.label}</b>
                <dl>
                  <div><dt>what exists now</dt><dd>{artifact.existsNow}</dd></div>
                  <div><dt>synthetic / no-access boundary</dt><dd>{artifact.syntheticBoundary}</dd></div>
                  <div><dt>what real access would validate</dt><dd>{artifact.realAccessValidation}</dd></div>
                  <div>
                    <dt>where to inspect</dt>
                    <dd>{artifact.inspect.map((path) => <code key={path}>{path}</code>)}</dd>
                  </div>
                </dl>
              </article>
            ))}
          </div>
        </details>
      </section>
    </main>
  );
}
