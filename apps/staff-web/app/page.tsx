"use client";

import { useEffect, useMemo, useState } from "react";

type StepId = "intake" | "draft" | "review" | "proof";

type ApiSnapshot = {
  ready: string;
  metrics: string;
  backlog: string;
};

type TechnicalCall = {
  label: string;
  method: "GET";
  path: string;
  status: string;
  latencyMs: number | null;
  artifact: string;
};

type SourceQualityRow = {
  issue_ref?: string;
  affected_entity_kind?: string;
  field_path?: string;
  issue_kind?: string;
  severity?: string;
  workflow_blocking?: string;
  review_gate?: string;
  projection_version?: string;
  caveats?: string[];
};

type TechnicalProof = {
  calls: TechnicalCall[];
  dbRows: SourceQualityRow[];
  counters: {
    inquiry_count?: number;
    review_packet_count?: number;
    audit_event_count?: number;
    outcome_count?: number;
  };
  lastRunIso: string;
};

const steps: Array<{
  id: StepId;
  label: string;
  title: string;
  promise: string;
  action: string;
}> = [
  {
    id: "intake",
    label: "1. Capture",
    title: "Inquiry becomes a clean work packet",
    promise: "The system turns a messy customer message into reservation facts, missing info, and a staff task.",
    action: "Parse inquiry"
  },
  {
    id: "draft",
    label: "2. Draft",
    title: "AI writes, staff stays in control",
    promise: "It drafts a customer-safe reply, but the send button is locked behind human review.",
    action: "Generate draft"
  },
  {
    id: "review",
    label: "3. Gate",
    title: "Risky decisions route to manager review",
    promise: "Document gaps, exceptions, and policy-sensitive actions are separated from routine staff work.",
    action: "Route review"
  },
  {
    id: "proof",
    label: "4. Prove",
    title: "Every action leaves outcome evidence",
    promise: "The demo shows labor saved, open tasks, and audit trail without touching live systems.",
    action: "Show proof"
  }
];

const stepCopy: Record<StepId, {
  before: string;
  afterTitle: string;
  afterBody: string;
  chips: string[];
  primaryMetric: string;
  secondaryMetric: string;
}> = {
  intake: {
    before: "Hi — can Miso board July 3–7? She is gentle but nervous with noise. I think her rabies record is attached, not sure if you need anything else.",
    afterTitle: "Structured intake packet",
    afterBody: "Miso • Boarding + enrichment • Jul 3–7 • needs vaccine document review • front desk follow-up created.",
    chips: ["missing vaccine proof", "noise-sensitive", "boarding request", "front-desk task"],
    primaryMetric: "8 min",
    secondaryMetric: "manual intake avoided"
  },
  draft: {
    before: "Staff used to rewrite the same missing-document response by hand, then remember not to promise availability too early.",
    afterTitle: "Draft reply, not a live send",
    afterBody: "Thanks Avery — we received Miso’s boarding request. Could you upload current vaccine records so our team can review availability and next steps?",
    chips: ["staff approval required", "no live send", "availability not promised", "customer-safe"],
    primaryMetric: "1 click",
    secondaryMetric: "to prepare a safe reply"
  },
  review: {
    before: "Policy-sensitive work gets buried beside routine check-in tasks, so managers find problems late.",
    afterTitle: "Manager review lane",
    afterBody: "Rabies document requires human review before confirmation. Booking confirmation remains blocked until staff approval.",
    chips: ["manager gate", "document review", "confirmation blocked", "policy boundary"],
    primaryMetric: "0",
    secondaryMetric: "unsafe automations enabled"
  },
  proof: {
    before: "Without owned workflow data, the value story is anecdotal: people feel busy but cannot prove where time went.",
    afterTitle: "Outcome and audit evidence",
    afterBody: "Inquiry normalized, draft created, review routed, outcome captured. Aggregate metrics prove labor reduction without exposing customer data.",
    chips: ["audit trail", "labor rollup", "aggregate only", "synthetic data"],
    primaryMetric: "23 min",
    secondaryMetric: "estimated daily labor saved"
  }
};

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" ? value as Record<string, unknown> : {};
}

function statusText(value: unknown) {
  const record = asRecord(value);
  if (typeof record.status === "string") return record.status;
  if (typeof record.service === "string") return `${record.service} responded`;
  const database = asRecord(record.database);
  if (typeof database.status === "string") return `database ${database.status}`;
  return "responded";
}

function sourceRows(value: unknown): SourceQualityRow[] {
  const record = asRecord(value);
  const records = Array.isArray(record.records) ? record.records : [];
  return records.slice(0, 4).map((row) => asRecord(row) as SourceQualityRow);
}

function runtimeCounters(value: unknown): TechnicalProof["counters"] {
  const record = asRecord(value);
  const counters = asRecord(record.local_runtime_counters);
  return {
    inquiry_count: typeof counters.inquiry_count === "number" ? counters.inquiry_count : undefined,
    review_packet_count: typeof counters.review_packet_count === "number" ? counters.review_packet_count : undefined,
    audit_event_count: typeof counters.audit_event_count === "number" ? counters.audit_event_count : undefined,
    outcome_count: typeof counters.outcome_count === "number" ? counters.outcome_count : undefined
  };
}

export default function Home() {
  const [activeStep, setActiveStep] = useState<StepId>("intake");
  const [approved, setApproved] = useState(false);
  const [api, setApi] = useState<ApiSnapshot>({ ready: "checking…", metrics: "checking…", backlog: "checking…" });
  const [proof, setProof] = useState<TechnicalProof | null>(null);
  const [proofLoading, setProofLoading] = useState(false);

  const current = stepCopy[activeStep];
  const progress = useMemo(() => steps.findIndex((step) => step.id === activeStep) + 1, [activeStep]);

  async function fetchArtifact(label: string, path: string): Promise<{ call: TechnicalCall; json: unknown }> {
    const started = performance.now();
    const response = await fetch(`/api/local-demo${path}`, {
      cache: "no-store",
      headers: {
        "x-request-id": `staff-demo-${crypto.randomUUID()}`,
        "x-correlation-id": "job-contact-technical-proof"
      }
    });
    const json = await response.json();
    const latencyMs = Math.round(performance.now() - started);
    return {
      json,
      call: {
        label,
        method: "GET",
        path,
        status: `HTTP ${response.status}`,
        latencyMs,
        artifact: JSON.stringify(json, null, 2).slice(0, 520)
      }
    };
  }

  async function runTechnicalProof() {
    setProofLoading(true);
    try {
      const [ready, metrics, backlog] = await Promise.all([
        fetchArtifact("API readiness", "/v0/readyz"),
        fetchArtifact("Runtime counters", "/v0/ops/metrics/summary"),
        fetchArtifact("Postgres read-model", "/v0/read-models/source-quality-backlog")
      ]);
      setProof({
        calls: [ready.call, metrics.call, backlog.call],
        counters: runtimeCounters(metrics.json),
        dbRows: sourceRows(backlog.json),
        lastRunIso: new Date().toISOString()
      });
      setApi({
        ready: statusText(ready.json),
        metrics: "aggregate metrics live",
        backlog: `${sourceRows(backlog.json).length} DB-backed rows`
      });
    } finally {
      setProofLoading(false);
    }
  }

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void runTechnicalProof();
    }, 0);
    return () => window.clearTimeout(timer);
    // Run once on first paint so the proof panel is populated before a presenter scrolls to it.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <main className="demo-shell">
      <section className="hero-card">
        <div className="hero-copy">
          <p className="eyebrow">Synthetic no-access product demo</p>
          <h1>Pet resort work, condensed into one safe staff cockpit.</h1>
          <p className="hero-subtitle">
            A visual prototype for owned operations: capture messy requests, draft staff-safe replies,
            gate risky decisions, and prove labor savings — without live customer, PMS, payment, or provider access.
          </p>
          <div className="hero-actions" aria-label="Demo controls">
            <button onClick={() => setActiveStep("intake")}>Start 2-minute walkthrough</button>
            <a href="#technical-proof">Show technical proof</a>
          </div>
        </div>
        <div className="impact-card" aria-label="Demo impact summary">
          <span>Today’s synthetic shift</span>
          <strong>23 min</strong>
          <p>estimated manual work removed from intake, review routing, and manager briefing.</p>
          <div className="mini-bars" aria-hidden="true"><i style={{ height: "68%" }} /><i style={{ height: "44%" }} /><i style={{ height: "82%" }} /><i style={{ height: "56%" }} /></div>
        </div>
      </section>

      <section className="kpi-strip" aria-label="Key demo proof points">
        <article><strong>0</strong><span>live sends / PMS writes</span></article>
        <article><strong>3</strong><span>browser API calls</span></article>
        <article><strong>{proof?.dbRows.length ?? "…"}</strong><span>Postgres read-model rows</span></article>
        <article><strong>{proof?.calls.length ?? "…"}</strong><span>visible JSON artifacts</span></article>
      </section>

      <section className="walkthrough-card" aria-label="Interactive walkthrough">
        <div className="step-rail">
          {steps.map((step, index) => (
            <button key={step.id} className={step.id === activeStep ? "step-button active" : "step-button"} onClick={() => setActiveStep(step.id)} aria-pressed={step.id === activeStep}>
              <span>{step.label}</span><strong>{step.action}</strong>{index + 1 < progress ? <em>done</em> : null}
            </button>
          ))}
        </div>
        <div className="demo-stage">
          <div className="stage-header">
            <p className="eyebrow">Step {progress} of 4</p>
            <h2>{steps.find((step) => step.id === activeStep)?.title}</h2>
            <p>{steps.find((step) => step.id === activeStep)?.promise}</p>
          </div>
          <div className="before-after">
            <article className="message-card before"><span>Before</span><p>{current.before}</p></article>
            <div className="arrow" aria-hidden="true">→</div>
            <article className="message-card after"><span>Owned workflow output</span><h3>{current.afterTitle}</h3><p>{current.afterBody}</p><div className="chip-row">{current.chips.map((chip) => <small key={chip}>{chip}</small>)}</div></article>
          </div>
          <div className="interaction-panel">
            <div><span className="metric-big">{current.primaryMetric}</span><p>{current.secondaryMetric}</p></div>
            <button onClick={() => setApproved((value) => !value)}>{approved ? "Approval recorded" : "Simulate staff approval"}</button>
            <p className={approved ? "approval on" : "approval"}>{approved ? "UI event recorded: staff reviewed synthetic draft. Live send remains disabled." : "Locked: this demo cannot send messages or mutate provider systems."}</p>
          </div>
        </div>
      </section>

      <section className="technical-proof" id="technical-proof" aria-label="Live technical artifacts">
        <div className="technical-header">
          <div>
            <p className="eyebrow">Live technical artifacts</p>
            <h2>Show the API and DB doing work.</h2>
            <p>Click once during the presentation: the browser calls the deployed Next.js proxy, the proxy calls the internal Rust API, and one endpoint reads a Postgres view seeded by migrations.</p>
          </div>
          <button onClick={runTechnicalProof} disabled={proofLoading}>{proofLoading ? "Running…" : "Run live proof again"}</button>
        </div>

        <div className="artifact-grid">
          <article className="artifact-card wide">
            <span>Call trace</span>
            <div className="call-list">
              {(proof?.calls ?? []).map((call) => (
                <div className="call-row" key={call.path}>
                  <code>{call.method} {call.path}</code>
                  <b>{call.status}</b>
                  <small>{call.latencyMs} ms</small>
                </div>
              ))}
            </div>
            <p className="muted-line">Last run: {proof?.lastRunIso ?? "waiting for browser proof"}</p>
          </article>

          <article className="artifact-card">
            <span>Runtime counters from API</span>
            <div className="counter-grid">
              <b>{proof?.counters.inquiry_count ?? "—"}<small>inquiries</small></b>
              <b>{proof?.counters.review_packet_count ?? "—"}<small>review packets</small></b>
              <b>{proof?.counters.audit_event_count ?? "—"}<small>audit events</small></b>
              <b>{proof?.counters.outcome_count ?? "—"}<small>outcomes</small></b>
            </div>
          </article>

          <article className="artifact-card db-card">
            <span>DB-backed read model</span>
            <h3>Postgres view: source_quality_backlog</h3>
            <div className="db-table" role="table" aria-label="Source quality backlog rows">
              {(proof?.dbRows ?? []).map((row) => (
                <div className="db-row" key={row.issue_ref} role="row">
                  <code>{row.issue_ref}</code>
                  <strong>{row.severity}</strong>
                  <small>{row.affected_entity_kind} · {row.field_path}</small>
                  <em>{row.review_gate}</em>
                </div>
              ))}
            </div>
            <p className="muted-line">These rows come from the Rust API querying Postgres through the `source_quality_backlog` projection, not hardcoded page text.</p>
          </article>

          <article className="artifact-card json-card">
            <span>Raw JSON excerpt</span>
            <pre>{proof?.calls[2]?.artifact ?? "Run the proof to display an API payload excerpt."}</pre>
          </article>
        </div>
      </section>

      <section className="proof-grid" id="proof" aria-label="Readiness proof and safety boundaries">
        <article className="proof-card"><p className="eyebrow">Live readiness</p><h2>API is checked from the browser.</h2><dl><div><dt>/v0/readyz</dt><dd>{api.ready}</dd></div><div><dt>metrics</dt><dd>{api.metrics}</dd></div><div><dt>backlog</dt><dd>{api.backlog}</dd></div></dl></article>
        <article className="proof-card safety"><p className="eyebrow">Hard boundary</p><h2>Built to demo without access.</h2><ul><li>No live customer messages</li><li>No PMS/provider writes</li><li>No payment/refund actions</li><li>No autonomous medical/safety decisions</li></ul></article>
        <article className="proof-card architecture"><p className="eyebrow">Architecture story</p><h2>Job-contact version</h2><div className="pipeline" aria-label="Architecture pipeline"><span>Staff UI</span><b>→</b><span>Next proxy</span><b>→</b><span>Rust API</span><b>→</b><span>Postgres read model</span></div><p>The page now shows both the product story and the technical evidence: live browser calls, HTTP statuses, latency, counters, DB projection rows, and raw JSON excerpts.</p></article>
      </section>
    </main>
  );
}
