"use client";

import { useEffect, useRef, useState } from "react";
import {
  dbProjectionLifecycleProofs,
  hermesProcessorPanel,
  informationLifespanNetworkRequests,
  informationLifespanStages,
  laborToolPortfolioCards,
  lineageEdges,
  managerDailyReportArtifact,
  notAskingItems,
  ownedBackendSpineStages,
  pilotAskItems,
  pilotSuccessCriteria,
  portfolioMetrics,
  proofArtifacts,
  roiPilotCloseCards,
  safeNextAskItems,
  safetyLocks,
  sourceEventCards,
  sourceEvidenceCards
} from "./owned-platform-demo-data";

type NetworkExchange = {
  id: string;
  method: "GET" | "POST";
  path: string;
  upstreamPath: string;
  status: "pending" | number;
  durationMs?: number;
  proofPurpose: string;
  responsePreview: Record<string, string | number | boolean>;
};

type StageMachineStatus = "idle" | "running" | "done" | "error";

const lineageLabel = "source evidence -> owned fact -> workflow packet -> review gate -> outcome/read model";
const stageAdvanceMs = 360;

const initialNetworkExchanges = (): NetworkExchange[] => informationLifespanNetworkRequests.map((request) => ({
  id: request.id,
  method: request.method,
  path: request.browserPath,
  upstreamPath: request.upstreamPath,
  status: "pending",
  proofPurpose: request.proofPurpose,
  responsePreview: request.responsePreview
}));

export default function Home() {
  const [selectedToolId, setSelectedToolId] = useState(laborToolPortfolioCards[0].id);
  const [processorView, setProcessorView] = useState<"running" | "done" | "unavailable">("done");
  const [stageMachineStatus, setStageMachineStatus] = useState<StageMachineStatus>("idle");
  const [activeStageIndex, setActiveStageIndex] = useState(-1);
  const [activeCorrelationId, setActiveCorrelationId] = useState(managerDailyReportArtifact.correlationId);
  const [runSequence, setRunSequence] = useState(0);
  const [informationRun, setInformationRun] = useState<{
    status: StageMachineStatus;
    correlationId?: string;
    artifactRef?: string;
    stageCount?: number;
    error?: string;
  }>({ status: "idle" });
  const [networkExchanges, setNetworkExchanges] = useState<NetworkExchange[]>(initialNetworkExchanges);
  const selectedTool = laborToolPortfolioCards.find((tool) => tool.id === selectedToolId) ?? laborToolPortfolioCards[0];
  const selectedLineage = lineageEdges.find((edge) => edge.toolId === selectedTool.id) ?? lineageEdges[0];
  const visibleProcessorStages = hermesProcessorPanel.stages.filter((stage) =>
    processorView === "unavailable" ? stage.state === "error" : stage.state !== "error"
  );
  const processorStatusLabel = processorView === "unavailable" ? "unavailable" : processorView === "running" ? "running replay" : hermesProcessorPanel.status;
  const activeStage = activeStageIndex >= 0 ? informationLifespanStages[activeStageIndex] : undefined;
  const completedStageCount = stageMachineStatus === "done" ? informationLifespanStages.length : Math.max(0, activeStageIndex);
  const stageProgressPercent = Math.round(((completedStageCount + (stageMachineStatus === "running" ? 1 : 0)) / informationLifespanStages.length) * 100);
  const latestRunId = useRef(0);

  useEffect(() => {
    if (stageMachineStatus !== "running") {
      return;
    }
    if (activeStageIndex >= informationLifespanStages.length - 1) {
      return;
    }
    const nextStageIndex = activeStageIndex + 1;
    const timer = window.setTimeout(() => {
      setActiveStageIndex(nextStageIndex);
    }, stageAdvanceMs);
    return () => window.clearTimeout(timer);
  }, [activeStageIndex, runSequence, stageMachineStatus]);

  const summarizeInformationLifespanResponse = (payload: any): Record<string, string | number | boolean> => ({
    workflow: payload?.api_contract?.workflow ?? "unknown",
    correlation_id: payload?.correlation_id ?? "unavailable",
    artifact_ref: payload?.final_report?.artifact_ref ?? payload?.processor_proof?.final_report?.artifact_ref ?? "unavailable",
    stage_count: Array.isArray(payload?.trace?.stages) ? payload.trace.stages.length : 0,
    reported_estimated_labor_minutes_difference: Number(payload?.processor_proof?.calculations?.reported_estimated_labor_minutes_difference ?? 42),
    review_gate_count: Array.isArray(payload?.safety?.review_gates) ? payload.safety.review_gates.length : 0,
    live_side_effects_allowed: payload?.safety?.live_side_effects_allowed ?? payload?.live_side_effects_allowed ?? false,
    processor_fallback_labeled: Boolean(payload?.processor_proof?.simulated)
  });

  const recordNetworkExchange = (exchange: NetworkExchange) => {
    setNetworkExchanges((current) => current.map((item) => (item.id === exchange.id ? exchange : item)));
  };

  const clearStaleProofForReplay = () => {
    setNetworkExchanges(initialNetworkExchanges());
    setActiveCorrelationId(managerDailyReportArtifact.correlationId);
    setActiveStageIndex(-1);
    setInformationRun({ status: "idle" });
  };

  const resetInformationLifespanDemo = () => {
    latestRunId.current += 1;
    clearStaleProofForReplay();
    setStageMachineStatus("idle");
    setProcessorView("done");
  };

  const waitForStageMachineReplay = () => new Promise<void>((resolve) => {
    window.setTimeout(resolve, stageAdvanceMs * informationLifespanStages.length);
  });

  const runInformationLifespanDemo = async () => {
    latestRunId.current += 1;
    const currentRunId = latestRunId.current;
    const visualReplayPromise = waitForStageMachineReplay();
    setProcessorView("running");
    setStageMachineStatus("running");
    setRunSequence((current) => current + 1);
    clearStaleProofForReplay();
    setInformationRun({ status: "running" });
    try {
      const runStartedAt = performance.now();
      const response = await fetch("/api/local-demo/v1/demo/information-lifespan/run", { method: "POST" });
      const payload = await response.json();
      recordNetworkExchange({
        id: "run-report-post",
        method: "POST",
        path: "/api/local-demo/v1/demo/information-lifespan/run",
        upstreamPath: "/v1/demo/information-lifespan/run",
        status: response.status,
        durationMs: Math.max(1, Math.round(performance.now() - runStartedAt)),
        proofPurpose: informationLifespanNetworkRequests[0].proofPurpose,
        responsePreview: response.ok ? summarizeInformationLifespanResponse(payload) : {
          error_code: payload?.error?.code ?? "local_demo_api_error",
          message: payload?.error?.message ?? `HTTP ${response.status}`,
          live_side_effects_allowed: payload?.live_side_effects_allowed ?? false
        }
      });
      if (!response.ok) {
        throw new Error(payload?.error?.message ?? `HTTP ${response.status}`);
      }
      setActiveCorrelationId(payload.correlation_id);

      const encodedCorrelationId = encodeURIComponent(payload.correlation_id);
      const reportPath = `/api/local-demo/v1/demo/information-lifespan/${encodedCorrelationId}/report`;
      const reportStartedAt = performance.now();
      const reportResponse = await fetch(reportPath, { method: "GET" });
      const reportPayload = await reportResponse.json();
      recordNetworkExchange({
        id: "report-replay-get",
        method: "GET",
        path: reportPath,
        upstreamPath: `/v1/demo/information-lifespan/${encodedCorrelationId}/report`,
        status: reportResponse.status,
        durationMs: Math.max(1, Math.round(performance.now() - reportStartedAt)),
        proofPurpose: informationLifespanNetworkRequests[1].proofPurpose,
        responsePreview: reportResponse.ok ? summarizeInformationLifespanResponse(reportPayload) : {
          error_code: reportPayload?.error?.code ?? "local_demo_report_error",
          message: reportPayload?.error?.message ?? `HTTP ${reportResponse.status}`,
          live_side_effects_allowed: reportPayload?.live_side_effects_allowed ?? false
        }
      });
      if (!reportResponse.ok) {
        throw new Error(reportPayload?.error?.message ?? `HTTP ${reportResponse.status}`);
      }

      await visualReplayPromise;
      if (currentRunId !== latestRunId.current) {
        return;
      }

      setProcessorView("done");
      setStageMachineStatus("done");
      setActiveStageIndex(informationLifespanStages.length - 1);
      setInformationRun({
        status: "done",
        correlationId: payload.correlation_id,
        artifactRef: payload.final_report?.artifact_ref,
        stageCount: payload.trace?.stages?.length
      });
    } catch (error) {
      setProcessorView("unavailable");
      setStageMachineStatus("error");
      setInformationRun({
        status: "error",
        error: error instanceof Error ? error.message : "information lifespan API request failed"
      });
    }
  };

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

        <nav className="three-demo-planes" aria-label="Demo organization">
          <span><b>Product story</b><small>read-only source evidence becomes owned work</small></span>
          <i aria-hidden="true">{"story -> machine -> report"}</i>
          <span><b>Stage/proof machine</b><small>cards unlock logs, rows, network calls, and calculations</small></span>
          <i aria-hidden="true">proof drawer updates</i>
          <span><b>Final report</b><small>manager actions keep lineage, locks, and labor value visible</small></span>
        </nav>

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

        <section className="information-lifespan-stage-machine" aria-label="Interactive information lifespan stage machine">
          <span className="rube-goldberg-orb" aria-hidden="true" />
          <div className="stage-machine-head">
            <div>
              <p className="eyebrow">Interactive Rube Goldberg trace</p>
              <h2>Run information lifespan</h2>
              <p>
                One click replays source evidence → NVA-owned models → DB projections → Hermes processor → network proof → Manager Daily Report, with proof panels keyed by correlation id.
              </p>
            </div>
            <div className="stage-machine-controls" role="group" aria-label="Information lifespan replay controls">
              <button className="api-run-button" onClick={runInformationLifespanDemo} disabled={stageMachineStatus === "running"} aria-pressed={stageMachineStatus === "running"}>
                {stageMachineStatus === "running" ? "Running stage machine…" : "Run information lifespan"}
              </button>
              <button className="reset-run-button" onClick={resetInformationLifespanDemo}>Replay / reset</button>
            </div>
          </div>
          <div className="stage-machine-live-region" aria-live="polite">
            {stageMachineStatus === "error"
              ? `request failed safely; no live side effects attempted · ${informationRun.error ?? "local demo API unavailable"}`
              : activeStage
                ? `${activeStage.order} ${activeStage.label} · active correlation id ${activeCorrelationId}`
                : `ready · ${informationLifespanStages.length} lifecycle steps · active correlation id ${activeCorrelationId}`}
          </div>
          <div
            className="stage-machine-meter"
            aria-label="Information lifespan progress"
            role="progressbar"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={stageProgressPercent}
            aria-valuetext={`${stageProgressPercent}% complete · ${stageMachineStatus}`}
          >
            <span style={{ width: `${stageProgressPercent}%` }} />
          </div>
          <ol className="lifespan-stage-list">
            {informationLifespanStages.map((stage, index) => {
              const stateClass = stageMachineStatus === "error" && index === activeStageIndex
                ? "error"
                : index < activeStageIndex || stageMachineStatus === "done"
                  ? "done"
                  : index === activeStageIndex
                    ? "active"
                    : "queued";
              return (
                <li className={`lifespan-stage-card ${stateClass}`} data-state-label={stateClass === "active" ? "lifespan-stage-card active" : stateClass} key={stage.id}>
                  <span className="stage-light" aria-hidden="true" />
                  <span className="lifespan-stage-order">{stage.order}</span>
                  <div>
                    <strong>{stage.label}</strong>
                    <b>{stage.headline}</b>
                    <p>{stage.proofSummary}</p>
                    <small>{stage.proofKind} proof · correlation_id {activeCorrelationId}</small>
                  </div>
                  <details open={stateClass === "active" || stateClass === "done"}>
                    <summary>unlocking proof drawer</summary>
                    <div className="lifespan-stage-proof">
                      {stage.inspect.map((path) => <code key={path}>{path}</code>)}
                    </div>
                  </details>
                </li>
              );
            })}
          </ol>
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
            <div className="source-event-list" aria-label="Initial mocked Gingr source events">
              {sourceEventCards.map((event) => (
                <article className="source-event-card" key={event.id}>
                  <div className="source-event-main">
                    <span className="source-event-time">{event.receivedAt}</span>
                    <h3>{event.eventLabel}</h3>
                    <p>{event.eventSummary}</p>
                    <div className="source-boundary-tags" aria-label="Source boundary">
                      {event.boundaryTags.map((tag) => <span key={tag}>{tag}</span>)}
                    </div>
                  </div>
                  <details className="source-event-proof" open={event.initiallyOpen}>
                    <summary>model/type badges + payload preview</summary>
                    <div className="model-badge-row" aria-label="model/type badges">
                      <span className="model-badge"><b>provider source model</b><code>{event.providerModelPath}</code></span>
                      <span className="model-badge"><b>NVA target model</b><code>{event.nvaTargetModelPath}</code></span>
                      <span className="model-badge"><b>trace contract</b><code>{event.traceModelPath}</code></span>
                    </div>
                    <dl className="source-event-proof-list">
                      <div><dt>source ref</dt><dd>{event.sourceRef}</dd></div>
                      <div><dt>payload preview</dt><dd><pre className="payload-preview" tabIndex={0} aria-label={`${event.eventLabel} payload preview`}>{JSON.stringify(event.payloadPreview, null, 2)}</pre></dd></div>
                    </dl>
                  </details>
                </article>
              ))}
            </div>
            <div className="source-card-list" tabIndex={0} aria-label="Source evidence cards">
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

          <section className="db-proof-panel panel" aria-label="Local DB and projection lifecycle proof">
            <div className="panel-head compact">
              <div>
                <p className="eyebrow">Local DB / projection proof</p>
                <h2>actual rows read back, compact proof only</h2>
              </div>
              <span>correlation_id {dbProjectionLifecycleProofs[0].correlationId}</span>
            </div>
            <p className="panel-copy">
              This lane lights up after source/model stages: tables show actual local Postgres proof, while the final view is deterministic trace proof that joins redacted row refs for engineering inspection.
            </p>
            <div className="db-proof-timeline" aria-label="DB row timeline">
              {dbProjectionLifecycleProofs.map((artifact) => (
                <article className={`db-proof-card ${artifact.artifactKind}`} key={artifact.id}>
                  <div className="db-proof-card-head">
                    <span>{artifact.order}</span>
                    <div>
                      <b>{artifact.artifactName}</b>
                      <small>{artifact.artifactKind} · row count {artifact.rowCount} · {artifact.proofPosture}</small>
                    </div>
                  </div>
                  <p>{artifact.summary}</p>
                  <details className="db-proof-row-preview">
                    <summary>compact redacted row JSON</summary>
                    <pre className="db-proof-row-json">{JSON.stringify(artifact.rowPreview, null, 2)}</pre>
                  </details>
                  <div className="db-proof-inspect" aria-label={`${artifact.artifactName} inspect paths`}>
                    {artifact.inspect.map((path) => <code key={path}>{path}</code>)}
                  </div>
                </article>
              ))}
            </div>
          </section>

          <section className="hermes-processor-panel panel" aria-label="Hermes processor container live output panel">
            <div className="panel-head compact processor-head">
              <div>
                <p className="eyebrow">Hermes processor container</p>
                <h2>{hermesProcessorPanel.serviceLabel}</h2>
                <p className="processor-boundary">{hermesProcessorPanel.boundary}</p>
              </div>
              <span className={`processor-status ${processorView}`}>{processorStatusLabel}</span>
            </div>
            <div className="processor-control-row" role="group" aria-label="Processor state examples">
              <button className={processorView === "running" ? "selected" : ""} aria-pressed={processorView === "running"} onClick={() => setProcessorView("running")}>show running</button>
              <button className={processorView === "done" ? "selected" : ""} aria-pressed={processorView === "done"} onClick={() => setProcessorView("done")}>show done</button>
              <button className={processorView === "unavailable" ? "selected" : ""} aria-pressed={processorView === "unavailable"} onClick={() => setProcessorView("unavailable")}>show unavailable</button>
              <button className="api-run-button" onClick={runInformationLifespanDemo} disabled={informationRun.status === "running"}>
                {informationRun.status === "running" ? "running API replay…" : "Run information lifespan"}
              </button>
            </div>
            <article className={`network-run-card ${informationRun.status}`} aria-label="Information lifespan API/network request proof">
              <div className="network-run-head">
                <div>
                  <b>network-visible run API / response proof</b>
                  <span>Browser route proxy shows real local requests; upstream API stays synthetic/read-only: POST /v1/demo/information-lifespan/run → GET /v1/demo/information-lifespan/:correlation_id/report.</span>
                </div>
                <strong>{informationRun.status === "done" ? "200 OK replay" : informationRun.status === "error" ? "safe failure visible" : "ready"}</strong>
              </div>
              <div className="network-exchange-list">
                {networkExchanges.map((exchange) => (
                  <article className={`network-exchange ${typeof exchange.status === "number" && exchange.status >= 400 ? "failed" : ""}`} key={exchange.id}>
                    <div className="network-exchange-line">
                      <strong>{exchange.method}</strong>
                      <code>{exchange.path}</code>
                      <span>{exchange.status === "pending" ? "pending" : `${exchange.status} ${exchange.durationMs ?? "?"}ms`}</span>
                    </div>
                    <small>upstream {exchange.upstreamPath} · {exchange.proofPurpose}</small>
                    <details open={informationRun.status !== "idle"}>
                      <summary>response preview</summary>
                      <pre>{JSON.stringify(exchange.responsePreview, null, 2)}</pre>
                    </details>
                  </article>
                ))}
              </div>
              {informationRun.status === "done" ? (
                <code>{informationRun.correlationId} · {informationRun.stageCount} stages · {informationRun.artifactRef}</code>
              ) : informationRun.status === "error" ? (
                <code>request failed safely · {informationRun.error}</code>
              ) : (
                <code>ready; response includes trace, processor proof, safety gates, calculations, and final report · network proof included</code>
              )}
            </article>
            <div className="processor-summary-grid">
              <article>
                <b>input summary</b>
                {hermesProcessorPanel.inputSummary.map((item) => <span key={item}>{item}</span>)}
              </article>
              <article>
                <b>runtime / container</b>
                <span>{hermesProcessorPanel.containerName}</span>
                <span>{hermesProcessorPanel.mode}</span>
                <code>{hermesProcessorPanel.runtimeStatus}</code>
              </article>
              <article className="processor-degrade-card">
                <b>graceful degradation</b>
                <span>{hermesProcessorPanel.unavailableDegradation}</span>
              </article>
            </div>
            <ol className="processor-stage-rail" aria-label="Hermes processor stage states">
              {visibleProcessorStages.map((stage) => (
                <li className={`processor-stage-card ${stage.state}`} key={stage.id}>
                  <span className="processor-stage-order">{stage.order}</span>
                  <div>
                    <strong>{stage.label}</strong>
                    <b>{stage.headline}</b>
                    <p>{stage.detail}</p>
                    <code>{stage.evidence}</code>
                  </div>
                </li>
              ))}
            </ol>
            {processorView !== "unavailable" ? (
              <div className="processor-output-grid">
                <article className="processor-log-panel" aria-label="Structured Hermes processor logs" tabIndex={0}>
                  <b>structured log output · correlation_id {hermesProcessorPanel.correlationId}</b>
                  {hermesProcessorPanel.logLines.map((line) => (
                    <pre className={`processor-log-line ${line.level.toLowerCase()}`} key={line.event}>{JSON.stringify({ timestamp: line.timestamp, level: line.level, target: line.target, event: line.event, correlation_id: line.correlationId, summary: line.summary }, null, 2)}</pre>
                  ))}
                </article>
                <article className="processor-report-fragment" aria-label="Generated Manager Daily Report fragment">
                  <span>generated/enriched report fragment</span>
                  <h3>{hermesProcessorPanel.reportFragment.title}</h3>
                  <p>{hermesProcessorPanel.reportFragment.summary}</p>
                  <code>{hermesProcessorPanel.reportFragment.artifactRef}</code>
                  <strong>{hermesProcessorPanel.reportFragment.calculation}</strong>
                  <ul>
                    {hermesProcessorPanel.reportFragment.managerActions.map((action) => <li key={action}>{action}</li>)}
                  </ul>
                </article>
              </div>
            ) : (
              <article className="processor-unavailable-card" aria-label="Processor unavailable fallback">
                <b>processor unavailable</b>
                <span>No container output is claimed. The page still shows safe input, locked-side-effect posture, and inspect paths for a local retry.</span>
              </article>
            )}
            <div className="processor-inspect-row" aria-label="Hermes processor inspect paths">
              {hermesProcessorPanel.inspect.map((path) => <code key={path}>{path}</code>)}
            </div>
          </section>

          <section className="manager-report-panel panel" aria-label="Final Manager Daily Report artifact">
            <div className="report-hero">
              <div>
                <p className="eyebrow">Final artifact</p>
                <h2>{managerDailyReportArtifact.title}</h2>
                <p>{managerDailyReportArtifact.summary}</p>
                <code>{managerDailyReportArtifact.artifactRef}</code>
                <small>{managerDailyReportArtifact.generatedBy} · correlation_id {managerDailyReportArtifact.correlationId}</small>
              </div>
              <aside className="report-value-card">
                <span>reported estimated minute difference</span>
                <strong>{managerDailyReportArtifact.valueProof.reportedEstimatedMinutesDifference}</strong>
                <small>{managerDailyReportArtifact.valueProof.sourceSnapshots} sources · {managerDailyReportArtifact.valueProof.normalizedFacts} facts · {managerDailyReportArtifact.valueProof.dbProofRefs} DB refs · {managerDailyReportArtifact.valueProof.reviewLocks} locks</small>
              </aside>
            </div>
            <div className="report-action-list" aria-label="Ranked manager actions">
              {managerDailyReportArtifact.rankedActions.map((action) => (
                <article className="report-action-card" key={action.title}>
                  <div className="report-action-heading">
                    <span>#{action.rank}</span>
                    <div>
                      <h3>{action.title}</h3>
                      <small>{action.owner} · {action.urgency} · {action.reportedEstimatedMinutesDifference}m modeled</small>
                    </div>
                  </div>
                  <p>{action.recommendation}</p>
                  <div className="report-proof-columns">
                    <article><b>source lineage</b>{action.sourceLineage.map((item) => <code key={item}>{item}</code>)}</article>
                    <article><b>calculations</b>{action.calculations.map((item) => <span key={item}>{item}</span>)}</article>
                    <article><b>review requirements</b>{action.reviewRequirements.map((item) => <span key={item}>{item}</span>)}</article>
                    <article className="locked-proof"><b>locked side effects</b>{action.lockedSideEffects.map((item) => <span key={item}>{item}</span>)}</article>
                  </div>
                </article>
              ))}
            </div>
            <div className="report-footer-grid">
              <article><b>lineage summary</b>{managerDailyReportArtifact.sourceLineageSummary.map((item) => <span key={item}>{item}</span>)}</article>
              <article><b>calculation proof</b>{managerDailyReportArtifact.calculationProof.map((item) => <code key={item}>{item}</code>)}</article>
              <article><b>review gates</b>{managerDailyReportArtifact.reviewGates.map((item) => <span key={item}>{item}</span>)}</article>
              <article className="locked-proof"><b>side-effect locks</b>{managerDailyReportArtifact.lockedSideEffects.map((item) => <span key={item}>{item}</span>)}</article>
            </div>
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
