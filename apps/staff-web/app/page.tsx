"use client";

import { useState } from "react";

type StepId = "collect" | "track" | "brief" | "outcome";
type Tone = "ok" | "warn" | "hold" | "info";

const steps: Array<{ id: StepId; number: string; label: string; explainer: string }> = [
  { id: "collect", number: "01", label: "messy morning", explainer: "Start with the scattered signals a manager would otherwise chase across notes, rooms, documents, capacity, and labor." },
  { id: "track", number: "02", label: "facts tracked", explainer: "Each signal keeps its source, field path, freshness, caveat, review gate, and labor estimate visible before it becomes advice." },
  { id: "brief", number: "03", label: "manager brief", explainer: "The workflow turns those source-backed facts into a ranked daily action plan a GM or front desk lead can review quickly." },
  { id: "outcome", number: "04", label: "review recorded", explainer: "No customer send or PMS write happens here; the demo records review status and minutes saved as synthetic proof." }
];

const morningChaos = [
  "7:20am lobby rush",
  "12 arrivals before 10",
  "rabies proof unclear",
  "coverage 2 short",
  "quiet-room request buried"
];

const sourceFacts: Array<{
  name: string;
  system: string;
  fields: string;
  tracked: string;
  tone: Tone;
}> = [
  { name: "Reservation", system: "PMS", fields: "Jul 3–7 · boarding · enrichment", tracked: "record id · updated 6:41", tone: "ok" },
  { name: "Pet profile", system: "staff note", fields: "noise-sensitive · quiet room", tracked: "author · timestamp · source ref", tone: "info" },
  { name: "Rabies proof", system: "document", fields: "attachment present · expiry unclear", tracked: "OCR · reviewer gate", tone: "warn" },
  { name: "Capacity", system: "ops sheet", fields: "rooms tight · enrichment waitlist", tracked: "projection version · caveat", tone: "hold" },
  { name: "Labor plan", system: "timeclock", fields: "AM coverage 2 short", tracked: "shift id · confidence", tone: "warn" }
];

const briefActions: Array<{
  rank: string;
  title: string;
  owner: string;
  why: string;
  gate: string;
  before: string;
  after: string;
  tone: Tone;
}> = [
  { rank: "1", title: "Review boarding vs labor", owner: "GM", why: "capacity + AM coverage risk", gate: "manager approval", before: "45m", after: "15m", tone: "warn" },
  { rank: "2", title: "Clear rabies document", owner: "front desk", why: "attached, expiry needs review", gate: "document review", before: "20m", after: "8m", tone: "hold" },
  { rank: "3", title: "Quiet-room plan for Miso", owner: "kennel lead", why: "noise-sensitive stay note", gate: "review ready", before: "8m", after: "2m", tone: "ok" }
];

const auditEvents = [
  "source refs attached",
  "data caveat visible",
  "send blocked",
  "PMS write blocked",
  "manager outcome recorded"
];

export default function Home() {
  const [activeStep, setActiveStep] = useState<StepId>("collect");
  const [approved, setApproved] = useState(false);
  const activeIndex = steps.findIndex((step) => step.id === activeStep);
  const activeStepDetail = steps[activeIndex]?.explainer;

  return (
    <main className="stage" data-step={activeStep}>
      <section className="demo-frame" aria-label="Manager daily brief demo">
        <header className="hero-row">
          <div className="brand-mark"><span>N</span></div>
          <div className="title-stack">
            <h1>Manager Daily Brief</h1>
            <p>synthetic · source-backed · review-gated</p>
          </div>
          <div className="saved-meter" aria-label="Labor saved metric">
            <strong>48</strong><span>min saved</span>
          </div>
        </header>

        <section className="chaos-strip" aria-label="Synthetic morning before the brief">
          <strong>Before the brief:</strong>
          <div>
            {morningChaos.map((signal) => (
              <span key={signal}>{signal}</span>
            ))}
          </div>
        </section>

        <nav className="step-dock" aria-label="Demo steps">
          {steps.map((step, index) => (
            <button
              key={step.id}
              className={step.id === activeStep ? "lit active" : index <= activeIndex ? "lit" : ""}
              onClick={() => setActiveStep(step.id)}
              aria-pressed={step.id === activeStep}
            >
              <b>{step.number}</b><span>{step.label}</span>
            </button>
          ))}
        </nav>

        <p className="step-explainer">{activeStepDetail}</p>

        <section className="brief-lab" aria-label="Manager brief construction scene">
          <aside className="source-board panel">
            <div className="panel-kicker">collected facts</div>
            <h2>One morning, five signals</h2>
            <div className="source-list">
              {sourceFacts.map((fact) => (
                <article className={`source-card ${fact.tone}`} key={fact.name}>
                  <div className="source-head"><b>{fact.name}</b><span>{fact.system}</span></div>
                  <p>{fact.fields}</p>
                  <i>{fact.tracked}</i>
                </article>
              ))}
            </div>
          </aside>

          <section className="assembly-column" aria-label="Tracking and review pipeline">
            <div className="pipeline-card panel">
              <div className="panel-kicker">tracked on every fact</div>
              <div className="chip-grid">
                <span>source ref</span>
                <span>field path</span>
                <span>freshness</span>
                <span>quality flag</span>
                <span>review gate</span>
                <span>labor estimate</span>
              </div>
              <div className="flow-line"><i /><i /><i /></div>
            </div>

            <div className="gate-card panel">
              <div className="panel-kicker">automation boundary</div>
              <div className="gate-row locked"><span className="gate-icon mail-icon" /><b>customer send</b><i>locked</i></div>
              <div className="gate-row locked"><span className="gate-icon pms-icon" /><b>PMS write</b><i>locked</i></div>
              <div className="gate-row open"><span className="gate-icon review-icon" /><b>manager review</b><i>ready</i></div>
            </div>
          </section>

          <article className="manager-brief panel">
            <div className="brief-header">
              <div>
                <div className="panel-kicker">today · jul 3</div>
                <h2>ranked action plan</h2>
              </div>
              <div className="brief-score"><strong>3</strong><span>actions</span></div>
            </div>

            <div className="action-list">
              {briefActions.map((action) => (
                <section className={`brief-action ${action.tone}`} key={action.rank}>
                  <div className="rank">{action.rank}</div>
                  <div className="action-copy">
                    <h3>{action.title}</h3>
                    <div className="action-meta">
                      <span>{action.owner}</span>
                      <span>{action.gate}</span>
                    </div>
                    <p>{action.why}</p>
                  </div>
                  <div className="minutes"><b>{action.before}</b><i>→</i><b>{action.after}</b></div>
                </section>
              ))}
            </div>
          </article>

          <aside className="proof-board panel">
            <div className="panel-kicker">outcome proof</div>
            <div className="proof-number"><strong>48</strong><span>{approved ? "actual min saved" : "estimated min saved"}</span></div>
            <ul>
              {auditEvents.map((event, index) => (
                <li key={event} className={index <= activeIndex || approved ? "on" : ""}><span />{event}</li>
              ))}
            </ul>
            <button className={approved ? "approval-button approved" : "approval-button"} onClick={() => setApproved((value) => !value)}>
              <span>{approved ? "✓" : "•"}</span>{approved ? "outcome recorded" : "record review"}
            </button>
          </aside>
        </section>
      </section>
    </main>
  );
}
