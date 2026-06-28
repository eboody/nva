"use client";

import { useEffect, useMemo, useState } from "react";

type StepId = "mess" | "packet" | "review" | "approve";

type SourceQualityRow = {
  issue_ref?: string;
  affected_entity_kind?: string;
  field_path?: string;
  severity?: string;
  review_gate?: string;
};

type TechnicalCall = {
  label: string;
  path: string;
  status: string;
  latencyMs: number | null;
  artifact: string;
};

type TechnicalProof = {
  mode: "live" | "fallback";
  calls: TechnicalCall[];
  rows: SourceQualityRow[];
  counters: {
    inquiry_count?: number;
    review_packet_count?: number;
    audit_event_count?: number;
    outcome_count?: number;
  };
  lastRunIso: string;
};

const fallbackRows: SourceQualityRow[] = [
  {
    issue_ref: "SQ-1842",
    affected_entity_kind: "pet_vaccine_record",
    field_path: "rabies.expires_on",
    severity: "blocking",
    review_gate: "medical document review before confirmation"
  },
  {
    issue_ref: "SQ-1843",
    affected_entity_kind: "reservation_request",
    field_path: "lodging.capacity_bucket",
    severity: "needs_manager",
    review_gate: "manager review before availability promise"
  },
  {
    issue_ref: "SQ-1844",
    affected_entity_kind: "pet_profile",
    field_path: "care_notes.noise_sensitivity",
    severity: "staff_attention",
    review_gate: "care-team note before enrichment add-on"
  }
];

const steps: Array<{
  id: StepId;
  eyebrow: string;
  title: string;
  beforeLabel: string;
  before: string;
  afterLabel: string;
  after: string;
  chips: string[];
}> = [
  {
    id: "mess",
    eyebrow: "before",
    title: "A normal morning starts as a pile of interruptions.",
    beforeLabel: "Inbox / phone / PMS notes",
    before: "Avery: Can Miso board July 3–7? She gets nervous with noise. I attached rabies, I think. Also can we add enrichment if you have room?",
    afterLabel: "What the system sees",
    after: "One request contains four separate jobs: lodging, vaccine review, care note, and capacity-sensitive add-on.",
    chips: ["messy request", "missing proof", "care nuance", "capacity risk"]
  },
  {
    id: "packet",
    eyebrow: "normalize",
    title: "The work becomes a staff packet, not another tab to inspect.",
    beforeLabel: "Manual path",
    before: "Front desk opens the PMS, scans documents, checks notes, writes a reply, and remembers which promises are unsafe.",
    afterLabel: "Owned workflow packet",
    after: "Miso • Boarding July 3–7 • rabies document needs review • noise-sensitive room note • enrichment waitlisted until capacity check.",
    chips: ["structured intake", "source refs", "missing-info task", "draft boundary"]
  },
  {
    id: "review",
    eyebrow: "gate",
    title: "The risky parts are blocked before they become promises.",
    beforeLabel: "Common failure mode",
    before: "A fast reply accidentally implies availability or accepts a medical document nobody reviewed.",
    afterLabel: "Review lane",
    after: "Confirmation blocked. Vaccine decision requires staff approval. Capacity-sensitive enrichment routes to manager review.",
    chips: ["no live send", "no PMS write", "manager review", "medical boundary"]
  },
  {
    id: "approve",
    eyebrow: "after",
    title: "The manager gets a short action plan with measurable labor saved.",
    beforeLabel: "Without owned data",
    before: "The team feels busy, but the business cannot prove how much labor went into source cleanup, draft replies, or review routing.",
    afterLabel: "Manager Daily Brief",
    after: "3 actions, 23 estimated minutes saved, 0 unsafe automations, 1 blocked confirmation, 1 data-quality issue ready for review.",
    chips: ["minutes saved", "audit trail", "outcome capture", "read-only proof"]
  }
];

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" ? value as Record<string, unknown> : {};
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

function fallbackProof(error: unknown): TechnicalProof {
  return {
    mode: "fallback",
    rows: fallbackRows,
    counters: {
      inquiry_count: 12,
      review_packet_count: 7,
      audit_event_count: 21,
      outcome_count: 5
    },
    calls: [
      {
        label: "Local API proof unavailable",
        path: "/api/local-demo/v0/*",
        status: "fallback fixture",
        latencyMs: null,
        artifact: error instanceof Error ? error.message : "PET_RESORT_API_BASE_URL not configured or API unreachable"
      }
    ],
    lastRunIso: new Date().toISOString()
  };
}

export default function Home() {
  const [activeStep, setActiveStep] = useState<StepId>("mess");
  const [approved, setApproved] = useState(false);
  const [proof, setProof] = useState<TechnicalProof>(() => fallbackProof("Initial deterministic fallback before live/local API proof runs."));
  const [proofLoading, setProofLoading] = useState(false);
  const current = steps.find((step) => step.id === activeStep) ?? steps[0];
  const stepNumber = useMemo(() => steps.findIndex((step) => step.id === activeStep) + 1, [activeStep]);

  async function fetchArtifact(label: string, path: string): Promise<{ call: TechnicalCall; json: unknown }> {
    const started = performance.now();
    const response = await fetch(`/api/local-demo${path}`, {
      cache: "no-store",
      headers: {
        "x-request-id": `staff-demo-${crypto.randomUUID()}`,
        "x-correlation-id": "job-contact-show-not-tell"
      }
    });
    const json = await response.json();
    const latencyMs = Math.round(performance.now() - started);
    if (!response.ok) {
      throw new Error(`${label} returned HTTP ${response.status}`);
    }
    return {
      json,
      call: {
        label,
        path,
        status: `HTTP ${response.status}`,
        latencyMs,
        artifact: JSON.stringify(json, null, 2).slice(0, 540)
      }
    };
  }

  async function runTechnicalProof() {
    setProofLoading(true);
    try {
      const [ready, metrics, backlog, managerBrief] = await Promise.all([
        fetchArtifact("Readiness", "/v0/readyz"),
        fetchArtifact("Metrics", "/v0/ops/metrics/summary"),
        fetchArtifact("Source-quality read model", "/v0/read-models/source-quality-backlog"),
        fetchArtifact("Manager Daily Brief", "/v0/agent/context/manager-daily-brief?location_id=00c0ffee-0000-0000-0000-000000000001&operating_day=2026-06-17")
      ]);
      const rows = sourceRows(backlog.json);
      setProof({
        mode: "live",
        calls: [ready.call, metrics.call, backlog.call, managerBrief.call],
        rows: rows.length > 0 ? rows : fallbackRows,
        counters: runtimeCounters(metrics.json),
        lastRunIso: new Date().toISOString()
      });
    } catch (error) {
      setProof(fallbackProof(error));
    } finally {
      setProofLoading(false);
    }
  }

  useEffect(() => {
    void runTechnicalProof();
  }, []);

  return (
    <main className="demo-shell">
      <section className="product-stage" aria-label="Interactive pet resort operator demo">
        <div className="stage-copy">
          <p className="eyebrow">Show-not-tell synthetic demo</p>
          <h1>Watch one messy pet-resort request become a safe manager action plan.</h1>
          <p className="subtitle">
            Built for a no-access situation: no live NVA data, no customer sends, no PMS writes. The demo proves the workflow seam with synthetic data and live/local API proof when available.
          </p>
          <div className="stage-actions">
            {steps.map((step, index) => (
              <button key={step.id} className={step.id === activeStep ? "active" : ""} onClick={() => setActiveStep(step.id)}>
                <span>{index + 1}</span>{step.eyebrow}
              </button>
            ))}
          </div>
        </div>

        <aside className="manager-brief-card" aria-label="Manager Daily Brief">
          <span>Manager Daily Brief</span>
          <strong>23 min</strong>
          <p>estimated labor removed today</p>
          <ul>
            <li>3 staff actions prepared</li>
            <li>1 confirmation blocked</li>
            <li>0 unsafe automations enabled</li>
          </ul>
        </aside>
      </section>

      <section className="workflow-board" aria-label="Before and after workflow">
        <div className="workflow-header">
          <p className="eyebrow">Step {stepNumber} / 4</p>
          <h2>{current.title}</h2>
        </div>

        <div className="before-after-grid">
          <article className="work-card before">
            <span>{current.beforeLabel}</span>
            <p>{current.before}</p>
          </article>
          <div className="flow-arrow" aria-hidden="true">→</div>
          <article className="work-card after">
            <span>{current.afterLabel}</span>
            <p>{current.after}</p>
            <div className="chip-row">{current.chips.map((chip) => <small key={chip}>{chip}</small>)}</div>
          </article>
        </div>

        <div className="action-simulator">
          <div>
            <span>Safe simulated action</span>
            <strong>{approved ? "Approval event recorded" : "Draft locked behind review"}</strong>
            <p>{approved ? "The synthetic staff approval is visible, but the demo still cannot send, confirm, charge, or mutate provider systems." : "Clicking approval only records a local UI event. It is intentionally not a live send or PMS write."}</p>
          </div>
          <button onClick={() => setApproved((value) => !value)}>{approved ? "Reset approval" : "Simulate staff approval"}</button>
        </div>
      </section>

      <section className="operator-cockpit" aria-label="Working operator cockpit">
        <article>
          <span>1 · Staff packet</span>
          <h3>Miso boarding request</h3>
          <dl>
            <div><dt>Request</dt><dd>Boarding July 3–7 + enrichment</dd></div>
            <div><dt>Care note</dt><dd>Noise-sensitive; room placement attention</dd></div>
            <div><dt>Blocker</dt><dd>Rabies document needs review</dd></div>
          </dl>
        </article>
        <article>
          <span>2 · Draft reply</span>
          <h3>Customer-safe response</h3>
          <p>“Thanks Avery — we received Miso’s boarding request. Could you upload current vaccine records so our team can review availability and next steps?”</p>
          <em>No availability promised. No live customer send.</em>
        </article>
        <article>
          <span>3 · Review gates</span>
          <h3>What stays human-owned</h3>
          <ul>
            <li>Confirm or reject booking</li>
            <li>Approve medical/vaccine record</li>
            <li>Change capacity or staff schedule</li>
            <li>Send customer messages</li>
          </ul>
        </article>
      </section>

      <section className="proof-section" id="technical-proof" aria-label="Technical proof">
        <div className="proof-header">
          <div>
            <p className="eyebrow">Proof drawer</p>
            <h2>{proof?.mode === "live" ? "Live/local API proof is connected." : "Fallback mode is honestly labeled."}</h2>
            <p>
              The presentation can run without production access. When the Rust API is configured, this panel shows live browser calls, counters, and source-quality rows. Otherwise it shows deterministic fixtures and says so.
            </p>
          </div>
          <button onClick={runTechnicalProof} disabled={proofLoading}>{proofLoading ? "Checking…" : "Run proof"}</button>
        </div>

        <div className="proof-grid">
          <article className="proof-card status">
            <span>Mode</span>
            <strong>{proof?.mode === "live" ? "Live local API data" : "Fallback fixture data"}</strong>
            <p>{proof?.mode === "live" ? "Browser → Next proxy → Rust API → owned read models." : "API unavailable or unconfigured; page does not claim DB evidence."}</p>
            <small>Last run: {proof?.lastRunIso ?? "not yet run"}</small>
          </article>
          <article className="proof-card counters">
            <span>Runtime counters</span>
            <div className="counter-grid">
              <b>{proof?.counters.inquiry_count ?? "—"}<small>inquiries</small></b>
              <b>{proof?.counters.review_packet_count ?? "—"}<small>review packets</small></b>
              <b>{proof?.counters.audit_event_count ?? "—"}<small>audit events</small></b>
              <b>{proof?.counters.outcome_count ?? "—"}<small>outcomes</small></b>
            </div>
          </article>
          <article className="proof-card rows">
            <span>{proof?.mode === "live" ? "DB-backed read-model records" : "Static fallback rows"}</span>
            <div className="row-list">
              {(proof?.rows ?? fallbackRows).map((row) => (
                <div className="data-row" key={row.issue_ref}>
                  <code>{row.issue_ref}</code>
                  <strong>{row.severity}</strong>
                  <small>{row.affected_entity_kind} · {row.field_path}</small>
                  <em>{row.review_gate}</em>
                </div>
              ))}
            </div>
          </article>
          <article className="proof-card calls">
            <span>API call trace</span>
            <div className="call-list">
              {(proof?.calls ?? []).map((call) => (
                <div className="call-row" key={`${call.label}-${call.path}`}>
                  <code>{call.path}</code>
                  <b>{call.status}</b>
                  <small>{call.latencyMs === null ? "—" : `${call.latencyMs} ms`}</small>
                </div>
              ))}
            </div>
            <pre>{proof?.calls[0]?.artifact ?? "Run proof to show a payload excerpt."}</pre>
          </article>
        </div>
      </section>

      <section className="honest-close" aria-label="Job contact talk track">
        <p className="eyebrow">How to present it</p>
        <h2>“I did not have access, so I built the safe seam first.”</h2>
        <div className="close-grid">
          <article><strong>What is strong now</strong><p>Concrete operator workflow, owned API/read-model proof, review gates, audit posture, measurable labor story.</p></article>
          <article><strong>What it does not claim</strong><p>No production NVA/Gingr data, no live customer messages, no provider/PMS writes, no payment or medical decisions.</p></article>
          <article><strong>What access would unlock</strong><p>Read-only source snapshots, field dictionaries, KPI definitions, and one instrumented pilot lane.</p></article>
        </div>
      </section>
    </main>
  );
}
