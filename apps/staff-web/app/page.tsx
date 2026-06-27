"use client";

import { useEffect, useMemo, useState } from "react";

type StepId = "intake" | "draft" | "review" | "proof";

type ApiSnapshot = {
  ready: string;
  metrics: string;
  backlog: string;
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

function statusText(value: unknown) {
  if (!value || typeof value !== "object") return "not checked";
  const record = value as Record<string, unknown>;
  if (typeof record.status === "string") return record.status;
  if (typeof record.service === "string") return `${record.service} responded`;
  return "responded";
}

export default function Home() {
  const [activeStep, setActiveStep] = useState<StepId>("intake");
  const [approved, setApproved] = useState(false);
  const [api, setApi] = useState<ApiSnapshot>({
    ready: "checking…",
    metrics: "checking…",
    backlog: "checking…"
  });

  const current = stepCopy[activeStep];
  const progress = useMemo(() => steps.findIndex((step) => step.id === activeStep) + 1, [activeStep]);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      const fetchJson = async (path: string) => {
        const response = await fetch(`/api/local-demo${path}`, { cache: "no-store" });
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        return response.json();
      };

      try {
        const [ready, metrics, backlog] = await Promise.allSettled([
          fetchJson("/v0/readyz"),
          fetchJson("/v0/ops/metrics/summary"),
          fetchJson("/v0/read-models/source-quality-backlog")
        ]);
        if (cancelled) return;
        setApi({
          ready: ready.status === "fulfilled" ? statusText(ready.value) : "offline fallback",
          metrics: metrics.status === "fulfilled" ? "aggregate metrics live" : "offline fallback",
          backlog: backlog.status === "fulfilled" ? "source-quality rows live" : "offline fallback"
        });
      } catch {
        if (!cancelled) {
          setApi({ ready: "offline fallback", metrics: "offline fallback", backlog: "offline fallback" });
        }
      }
    }
    void load();
    return () => {
      cancelled = true;
    };
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
            <a href="#proof">Jump to proof</a>
          </div>
        </div>
        <div className="impact-card" aria-label="Demo impact summary">
          <span>Today’s synthetic shift</span>
          <strong>23 min</strong>
          <p>estimated manual work removed from intake, review routing, and manager briefing.</p>
          <div className="mini-bars" aria-hidden="true">
            <i style={{ height: "68%" }} />
            <i style={{ height: "44%" }} />
            <i style={{ height: "82%" }} />
            <i style={{ height: "56%" }} />
          </div>
        </div>
      </section>

      <section className="kpi-strip" aria-label="Key demo proof points">
        <article><strong>0</strong><span>live sends / PMS writes</span></article>
        <article><strong>4</strong><span>clickable workflow steps</span></article>
        <article><strong>3</strong><span>human review gates</span></article>
        <article><strong>1</strong><span>owned API contract</span></article>
      </section>

      <section className="walkthrough-card" aria-label="Interactive walkthrough">
        <div className="step-rail">
          {steps.map((step, index) => (
            <button
              key={step.id}
              className={step.id === activeStep ? "step-button active" : "step-button"}
              onClick={() => setActiveStep(step.id)}
              aria-pressed={step.id === activeStep}
            >
              <span>{step.label}</span>
              <strong>{step.action}</strong>
              {index + 1 < progress ? <em>done</em> : null}
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
            <article className="message-card before">
              <span>Before</span>
              <p>{current.before}</p>
            </article>
            <div className="arrow" aria-hidden="true">→</div>
            <article className="message-card after">
              <span>Owned workflow output</span>
              <h3>{current.afterTitle}</h3>
              <p>{current.afterBody}</p>
              <div className="chip-row">
                {current.chips.map((chip) => <small key={chip}>{chip}</small>)}
              </div>
            </article>
          </div>

          <div className="interaction-panel">
            <div>
              <span className="metric-big">{current.primaryMetric}</span>
              <p>{current.secondaryMetric}</p>
            </div>
            <button onClick={() => setApproved((value) => !value)}>
              {approved ? "Approval recorded" : "Simulate staff approval"}
            </button>
            <p className={approved ? "approval on" : "approval"}>
              {approved
                ? "Audit event created: staff reviewed synthetic draft. Still no live customer send."
                : "Locked: this demo cannot send messages or mutate provider systems."}
            </p>
          </div>
        </div>
      </section>

      <section className="proof-grid" id="proof" aria-label="Readiness proof and safety boundaries">
        <article className="proof-card">
          <p className="eyebrow">Live readiness</p>
          <h2>API is checked from the browser.</h2>
          <dl>
            <div><dt>/v0/readyz</dt><dd>{api.ready}</dd></div>
            <div><dt>metrics</dt><dd>{api.metrics}</dd></div>
            <div><dt>backlog</dt><dd>{api.backlog}</dd></div>
          </dl>
        </article>
        <article className="proof-card safety">
          <p className="eyebrow">Hard boundary</p>
          <h2>Built to demo without access.</h2>
          <ul>
            <li>No live customer messages</li>
            <li>No PMS/provider writes</li>
            <li>No payment/refund actions</li>
            <li>No autonomous medical/safety decisions</li>
          </ul>
        </article>
        <article className="proof-card architecture">
          <p className="eyebrow">Architecture story</p>
          <h2>Job-contact version</h2>
          <div className="pipeline" aria-label="Architecture pipeline">
            <span>Staff UI</span><b>→</b><span>Owned API</span><b>→</b><span>Review gates</span><b>→</b><span>Audit + outcomes</span>
          </div>
          <p>This is the condensed visual layer; the deeper text dashboard should become a secondary appendix, not the first impression.</p>
        </article>
      </section>
    </main>
  );
}
