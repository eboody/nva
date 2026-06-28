"use client";

import { useState } from "react";

type StepId = "inbox" | "packet" | "gate" | "brief";

const steps: Array<{
  id: StepId;
  number: string;
  label: string;
  scene: string;
}> = [
  { id: "inbox", number: "01", label: "mess", scene: "incoming" },
  { id: "packet", number: "02", label: "packet", scene: "normalized" },
  { id: "gate", number: "03", label: "gate", scene: "blocked" },
  { id: "brief", number: "04", label: "brief", scene: "done" }
];

export default function Home() {
  const [activeStep, setActiveStep] = useState<StepId>("inbox");
  const [approved, setApproved] = useState(false);
  const activeIndex = steps.findIndex((step) => step.id === activeStep);

  return (
    <main className="stage" data-step={activeStep}>
      <section className="demo-frame" aria-label="Visual pet resort workflow demo">
        <header className="top-strip">
          <div className="brand-mark"><span>N</span></div>
          <div className="title-stack">
            <h1>Miso → manager brief</h1>
            <p>synthetic · no live sends · no PMS writes</p>
          </div>
          <div className="saved-meter">
            <strong>23</strong><span>min saved</span>
          </div>
        </header>

        <nav className="step-dock" aria-label="Demo steps">
          {steps.map((step, index) => (
            <button
              key={step.id}
              className={index <= activeIndex ? "lit" : ""}
              onClick={() => setActiveStep(step.id)}
              aria-pressed={step.id === activeStep}
            >
              <b>{step.number}</b><span>{step.label}</span>
            </button>
          ))}
        </nav>

        <section className="visual-flow" aria-label="Before after workflow visualization">
          <article className="phone-card messy-card">
            <div className="phone-top"><i /><i /><i /></div>
            <div className="message-bubble big">Board July 3–7?</div>
            <div className="message-bubble">rabies attached?</div>
            <div className="message-bubble small">noise-sensitive</div>
            <div className="message-bubble">add enrichment?</div>
            <div className="notification-stack">
              <span className="dot call" /><span className="dot mail" /><span className="dot note" />
            </div>
          </article>

          <div className="transform-column" aria-hidden="true">
            <div className="pulse-ring"><span>→</span></div>
            <div className="trace-line" />
          </div>

          <article className="work-packet">
            <div className="pet-avatar" aria-hidden="true"><i /><b /></div>
            <div className="packet-main">
              <h2>Miso</h2>
              <div className="date-strip"><span>Jul 3</span><i /><span>Jul 7</span></div>
              <div className="packet-grid">
                <div className="tile ok"><b>boarding</b><span>ready</span></div>
                <div className="tile warn"><b>rabies</b><span>review</span></div>
                <div className="tile info"><b>room</b><span>quiet</span></div>
                <div className="tile hold"><b>enrich</b><span>capacity</span></div>
              </div>
            </div>
          </article>

          <aside className="gate-panel">
            <div className="gate-row locked"><span className="gate-icon mail-icon" /><b>send</b><i>locked</i></div>
            <div className="gate-row locked"><span className="gate-icon pms-icon" /><b>PMS</b><i>locked</i></div>
            <div className="gate-row open"><span className="gate-icon review-icon" /><b>review</b><i>ready</i></div>
          </aside>

          <aside className="brief-card">
            <div className="brief-top"><span>brief</span><b>today</b></div>
            <div className="brief-number">23</div>
            <div className="brief-bars"><i /><i /><i /></div>
            <ul>
              <li><span />3 actions</li>
              <li><span />1 blocked</li>
              <li><span />0 unsafe</li>
            </ul>
          </aside>
        </section>

        <section className="action-zone" aria-label="Simulated safe action">
          <button className={approved ? "approval-button approved" : "approval-button"} onClick={() => setApproved((value) => !value)}>
            <span>{approved ? "✓" : "•"}</span>{approved ? "review recorded" : "approve draft"}
          </button>
          <div className="audit-dots" aria-label="Audit trail visualization">
            <i className={activeIndex >= 0 ? "on" : ""} />
            <i className={activeIndex >= 1 ? "on" : ""} />
            <i className={activeIndex >= 2 ? "on" : ""} />
            <i className={approved || activeIndex >= 3 ? "on" : ""} />
          </div>
        </section>
      </section>
    </main>
  );
}
