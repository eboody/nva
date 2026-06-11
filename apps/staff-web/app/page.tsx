"use client";

import { FormEvent, useMemo, useState } from "react";

type StaffRole = "staff" | "lead" | "manager";

type Readiness = "Ready" | "Needs front-desk collection" | "Needs care review" | "Needs manager review" | "Critical now";

type Task = {
  id: string;
  title: string;
  owner: string;
  due: string;
  readiness: Readiness;
  reviewGate: string;
  auditAction: string;
};

type StaffNote = {
  id: string;
  label: string;
  visibility: "internal-only" | "customer-safe candidate";
  body: string;
};

type IncidentType =
  | "injury"
  | "illness"
  | "bite/aggression"
  | "escape attempt"
  | "medication issue"
  | "feeding issue"
  | "bathroom concern"
  | "customer complaint"
  | "staff safety"
  | "property damage";

type SeverityDraft = "low" | "medium" | "high" | "emergency";

type IncidentStatus = "reported" | "needs_manager_review" | "customer_message_review" | "follow_up_open";

type Incident = {
  id: string;
  subject: string;
  type: IncidentType;
  severityDraft: SeverityDraft;
  status: IncidentStatus;
  observedFacts: string;
  classificationDraft: string;
  managerReview: string;
  ownerMessageDraft: string;
  followUpTask: string;
  behaviorFlagBoundary: string;
  reviewGates: string[];
  auditEvents: string[];
  nextStep: string;
};

type IncidentFormState = {
  subject: string;
  type: IncidentType;
  severitySignal: SeverityDraft;
  observedFacts: string;
  ownerDraftRequested: boolean;
  eligibilityFlagCandidate: boolean;
};

const incidentTypeOptions: IncidentType[] = [
  "injury",
  "illness",
  "bite/aggression",
  "escape attempt",
  "medication issue",
  "feeding issue",
  "bathroom concern",
  "customer complaint",
  "staff safety",
  "property damage"
];

const severityOptions: SeverityDraft[] = ["low", "medium", "high", "emergency"];

const session = {
  actor: "MVP Staff Demo",
  role: "lead" as StaffRole,
  location: "Local/dev kennel",
  policy: "draft role matrix: staff assigned/location; manager gates sensitive decisions"
};

const todayReservations = [
  {
    id: "RSV-1007",
    pet: "Miso",
    customer: "Avery Chen",
    service: "Boarding + enrichment",
    window: "Arrives 8:30–9:00",
    readiness: "Needs front-desk collection" as Readiness,
    blockers: ["signed waiver expected at desk", "rabies PDF awaiting reviewer"],
    draftMessage: "Missing-document reminder draft only — no live customer sends"
  },
  {
    id: "RSV-1012",
    pet: "Juniper",
    customer: "R. Patel",
    service: "Day play",
    window: "Pickup 17:00",
    readiness: "Ready" as Readiness,
    blockers: [],
    draftMessage: "Checkout summary draft waits for staff approval"
  }
];

const petProfile = {
  name: "Miso",
  household: "Avery Chen household",
  careLane: "boarding / individual enrichment",
  careFlags: ["reviewed feeding card", "medication source not modeled", "temperament: gentle, noise sensitive"],
  documentState: "rabies proof awaiting human document review",
  customerSafeSummary: "Settling in quietly; enrichment plan prepared after check-in. Draft only."
};

const inquiryIntakeQueue = [
  {
    id: "INQ-LOCAL-001",
    eventType: "inquiry.received",
    sourceEventKey: "local-smoke-inquiry-001",
    parsedLead: "Parsed lead: Avery Chen / Miso / boarding / 2026-07-03 to 2026-07-07",
    missingInfo: ["vaccine_records"],
    draftReply: "Inquiry draft reply: Thanks Avery — we received your inquiry for Miso. Could you send current vaccine records so our staff can review availability and next steps?",
    liveSendBoundary: "live_send_allowed: false — staff approval required before customer reply",
    task: "Missing-info task: collect vaccine_records and route to front desk review",
    audit: "inquiry.received.normalized / agent.inquiry-intake.fake_deterministic / message.draft.created"
  }
];

const bookingTriage = {
  request: "REQ-123 • Miso • Boarding + enrichment • requested 2026-06-20 to 2026-06-24",
  lifecycle: "booking_request → ready_for_staff_approval draft; provider confirmation is not executed",
  rules: [
    { id: "date_range_and_service_supported", decision: "pass", evidence: "policy:service-catalog:v1" },
    { id: "accommodation_availability", decision: "pass", evidence: "availability:snapshot:cap-123" },
    { id: "vaccine_requirements", decision: "needs medical document review", evidence: "document:rabies-upload-77" },
    { id: "deposit_and_pricing_requirements", decision: "pass", evidence: "deposit:paid:dep-9" }
  ],
  aiRecommendation: "Draft confirmation for staff approval only. AI may summarize evidence but cannot invent availability, reject, confirm, mutate provider records, move money, or send customer messages.",
  confirmationDraft: "Produce draft confirmation: We can prepare your booking confirmation after staff approval and document review are complete.",
  gates: [
    "confirmed booking automation requires staff approval",
    "reject/decline remains human approval gated",
    "special-care acceptance requires care-team approval",
    "behavior exceptions require behavior review",
    "customer message approval required before send"
  ]
};

const taskQueue: Task[] = [
  {
    id: "TASK-41",
    title: "Check-in prep: Miso",
    owner: "Front desk",
    due: "08:15",
    readiness: "Needs front-desk collection",
    reviewGate: "routine collection; manager gate if payment or eligibility exception appears",
    auditAction: "care task create/assign/start/complete/block"
  },
  {
    id: "TASK-52",
    title: "Feeding card verification",
    owner: "Kennel technician",
    due: "10:30",
    readiness: "Needs care review",
    reviewGate: "care source review before execution",
    auditAction: "staff note and care completion evidence"
  },
  {
    id: "TASK-59",
    title: "Draft checkout update: Juniper",
    owner: "Lead staff",
    due: "16:00",
    readiness: "Needs manager review",
    reviewGate: "customer message approval before send",
    auditAction: "message draft created/approval requested"
  }
];

const documentReviews = [
  {
    id: "DOC-77",
    subject: "Miso rabies.pdf",
    source: "customer upload",
    classification: "vaccine proof candidate",
    state: "Awaiting reviewer — AI/OCR cannot verify vaccine status",
    action: "Open review packet"
  },
  {
    id: "DOC-84",
    subject: "Juniper waiver scan",
    source: "staff scan",
    classification: "waiver candidate",
    state: "Needs metadata check",
    action: "Return for source confirmation"
  }
];

const vaccineDocumentReview = {
  sampleUploadAction: "Upload sample vaccine document",
  extractionSchema: "vaccine_extraction.v1",
  uncertaintyPolicy: "medical document uncertainty policy: medical_document_uncertainty_policy_requires_staff_review",
  reviewActions: ["Approve vaccine record", "Reject vaccine record"],
  eligibilityBoundary: "pet eligibility updates only after approval",
  auditEvents: ["document.received", "vaccine_record.review_requested", "approval.decision.recorded"]
};

const notes: StaffNote[] = [
  {
    id: "NOTE-18",
    label: "Shift handoff",
    visibility: "internal-only",
    body: "Miso prefers the quiet run; do not publish without manager-approved customer-safe wording."
  },
  {
    id: "NOTE-21",
    label: "Daily update draft",
    visibility: "customer-safe candidate",
    body: "Juniper enjoyed supervised play and is resting after lunch. Staff approval still required."
  }
];

const initialIncidentForm: IncidentFormState = {
  subject: "Miso / RSV-1007",
  type: "medication issue",
  severitySignal: "medium",
  observedFacts: "Staff reported the noon medication was not documented and source record needs confirmation.",
  ownerDraftRequested: true,
  eligibilityFlagCandidate: false
};

function reviewGatesFor(form: IncidentFormState): string[] {
  const gates = new Set<string>();

  if (form.ownerDraftRequested) {
    gates.add("CustomerMessageApproval");
  }

  if (["medium", "high", "emergency"].includes(form.severitySignal)) {
    gates.add("ManagerApproval");
  }

  if (["injury", "illness", "medication issue", "feeding issue", "bathroom concern"].includes(form.type)) {
    gates.add("MedicalDocumentReview");
  }

  if (["bite/aggression", "escape attempt", "staff safety"].includes(form.type) || form.eligibilityFlagCandidate) {
    gates.add("BehaviorReview");
    gates.add("ManagerApproval");
  }

  return Array.from(gates);
}

function statusFor(gates: string[], ownerDraftRequested: boolean): IncidentStatus {
  if (ownerDraftRequested) {
    return "customer_message_review";
  }

  if (gates.includes("ManagerApproval") || gates.includes("BehaviorReview")) {
    return "needs_manager_review";
  }

  return "follow_up_open";
}

function buildIncident(form: IncidentFormState, id: string): Incident {
  const reviewGates = reviewGatesFor(form);
  const seriousClassificationNeedsApproval = ["medium", "high", "emergency"].includes(form.severitySignal);
  const status = statusFor(reviewGates, form.ownerDraftRequested);
  const managerReview = seriousClassificationNeedsApproval
    ? "Route to manager-review queue — final classification requires manager approval."
    : "Lead/staff review can confirm low-severity follow-up before closure.";
  const ownerMessageDraft = form.ownerDraftRequested
    ? `Draft only: The record indicates ${form.observedFacts} We are reviewing the details and will follow up with approved next steps. Requires CustomerMessageApproval before any owner-facing send.`
    : "No owner-facing message requested; any later owner copy must be generated as a draft requiring CustomerMessageApproval.";
  const behaviorFlagBoundary = form.eligibilityFlagCandidate || reviewGates.includes("BehaviorReview")
    ? "Eligibility-impacting flag recommendation only — BehaviorReview and ManagerApproval required before application, clearance, downgrade, or group-play reinstatement."
    : "No eligibility-affecting flag selected; the workflow still cannot clear restrictions or finalize future care eligibility.";

  return {
    id,
    subject: form.subject,
    type: form.type,
    severityDraft: form.severitySignal,
    status,
    observedFacts: form.observedFacts,
    classificationDraft: `${form.type} classification draft with proposed ${form.severitySignal} severity; staff reported facts remain source-grounded and provisional.`,
    managerReview,
    ownerMessageDraft,
    followUpTask: `Create follow-up task: IncidentFollowUp for ${form.subject}; collect missing facts, evidence refs, owner-notice decision, and closure blockers.`,
    behaviorFlagBoundary,
    reviewGates,
    auditEvents: [
      "incident.report_drafted",
      "incident.triage_packet_created",
      "incident.review_gates_proposed",
      form.ownerDraftRequested ? "owner_notice.draft_created" : "owner_notice.not_requested",
      "staff_task.drafted_or_created"
    ],
    nextStep: status === "customer_message_review"
      ? "Generate owner-message draft, request manager/customer-message approval, and keep send disabled."
      : "Record incident facts, create follow-up task, and hold closure until open review gates are resolved."
  };
}

const initialIncidents: Incident[] = [
  buildIncident(
    {
      subject: "Juniper paw scrape observation",
      type: "injury",
      severitySignal: "medium",
      observedFacts: "Staff reported a small paw scrape after supervised play; no diagnosis is recorded and body-location details need manager review.",
      ownerDraftRequested: true,
      eligibilityFlagCandidate: false
    },
    "INC-5"
  ),
  buildIncident(
    {
      subject: "Yard gate latch check",
      type: "escape attempt",
      severitySignal: "high",
      observedFacts: "The record indicates a gate latch near-miss during yard transition; facility evidence and handling restriction need review.",
      ownerDraftRequested: false,
      eligibilityFlagCandidate: true
    },
    "INC-6"
  )
];

const auditTrail = [
  "staff.session.guard.allowed — actor lead at local/dev location",
  "task.draft.created — no provider mutation",
  "document.review.requested — human verification required",
  "message.draft.created — no live customer sends",
  "incident.report.drafted — manager review before severity/closure",
  "audit.append-only.event.visible — staff actions preserve policy refs"
];

function StatusBadge({ status }: { status: Readiness | IncidentStatus }) {
  return <span className="badge">{status.replaceAll("_", " ")}</span>;
}

function Panel({ title, eyebrow, children }: Readonly<{ title: string; eyebrow?: string; children: React.ReactNode }>) {
  return (
    <section aria-labelledby={`${title.toLowerCase().replace(/[^a-z0-9]+/g, "-")}-heading`} className="panel">
      {eyebrow ? <p className="eyebrow">{eyebrow}</p> : null}
      <h2 id={`${title.toLowerCase().replace(/[^a-z0-9]+/g, "-")}-heading`}>{title}</h2>
      {children}
    </section>
  );
}

export default function Home() {
  const [incidentForm, setIncidentForm] = useState<IncidentFormState>(initialIncidentForm);
  const [incidents, setIncidents] = useState<Incident[]>(initialIncidents);
  const currentDraft = useMemo(() => buildIncident(incidentForm, "INC-DRAFT"), [incidentForm]);
  const managerReviewQueue = incidents.filter((incident) =>
    incident.reviewGates.includes("ManagerApproval") || incident.reviewGates.includes("BehaviorReview")
  );

  function recordIncident(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIncidents((current) => [buildIncident(incidentForm, `INC-${current.length + 7}`), ...current]);
  }

  return (
    <main className="shell">
      <section className="hero">
        <p className="eyebrow">Local/dev staff dashboard MVP</p>
        <h1>Pet Resort Staff Operations</h1>
        <p>
          Authentication/session guard, role-aware queues, draft customer communications, human document review,
          incident review, and append-only audit cues for local happy-path practice. No live customer sends,
          provider writes, payment actions, eligibility-affecting flag writes, or autonomous medical/safety decisions are available from this surface.
        </p>
      </section>

      <nav aria-label="Staff dashboard sections" className="nav-card">
        {[
          "Session guard",
          "Today operations",
          "Pet profile",
          "Reservation view",
          "Inquiry intake queue",
          "Parsed lead",
          "Inquiry draft reply",
          "Missing-info task",
          "Booking triage",
          "Hard-rule results",
          "AI recommendation",
          "Staff confirmation controls",
          "Confirmation draft",
          "Task queue",
          "Document review queue",
          "Vaccine document MVP",
          "Staff notes",
          "Incident entry",
          "Manager review queue",
          "Incident list",
          "Audit-visible staff actions"
        ].map((surface) => (
          <a href={`#${surface.toLowerCase().replace(/[^a-z0-9]+/g, "-")}`} key={surface}>{surface}</a>
        ))}
      </nav>

      <Panel title="Session guard" eyebrow="authenticated staff context">
        <div className="identity-card">
          <div>
            <strong>{session.actor}</strong>
            <p>{session.role} • {session.location}</p>
          </div>
          <StatusBadge status="Ready" />
        </div>
        <p className="muted">{session.policy}. Anonymous/customer sessions should be denied before this dashboard renders.</p>
      </Panel>

      <Panel title="Today operations" eyebrow="operating day overview">
        <div className="metric-grid">
          <div><strong>2</strong><span>reservations in local data</span></div>
          <div><strong>3</strong><span>open staff tasks</span></div>
          <div><strong>2</strong><span>documents awaiting review</span></div>
          <div><strong>{incidents.length}</strong><span>incidents visible for review</span></div>
        </div>
      </Panel>

      <Panel title="Pet profile" eyebrow="role-scoped pet facts">
        <div className="split">
          <div>
            <h3>{petProfile.name}</h3>
            <p>{petProfile.household}</p>
            <p>{petProfile.careLane}</p>
            <p><strong>Document state:</strong> {petProfile.documentState}</p>
          </div>
          <ul>
            {petProfile.careFlags.map((flag) => <li key={flag}>{flag}</li>)}
          </ul>
        </div>
        <blockquote>{petProfile.customerSafeSummary}</blockquote>
      </Panel>

      <Panel title="Reservation view" eyebrow="happy-path reservation cards">
        <div className="card-grid">
          {todayReservations.map((reservation) => (
            <article className="record-card" key={reservation.id}>
              <div className="record-header"><h3>{reservation.pet}</h3><StatusBadge status={reservation.readiness} /></div>
              <p>{reservation.id} • {reservation.customer}</p>
              <p>{reservation.service} • {reservation.window}</p>
              <p><strong>Draft message boundary:</strong> {reservation.draftMessage}</p>
              {reservation.blockers.length > 0 ? <ul>{reservation.blockers.map((blocker) => <li key={blocker}>{blocker}</li>)}</ul> : <p className="muted">No visible blockers in demo data.</p>}
            </article>
          ))}
        </div>
      </Panel>

      <Panel title="Inquiry intake queue" eyebrow="inquiry.received normalized intake">
        {inquiryIntakeQueue.map((inquiry) => (
          <article className="record-card" key={inquiry.id}>
            <div className="record-header"><h3>{inquiry.id}</h3><span className="badge">{inquiry.eventType}</span></div>
            <p>{inquiry.sourceEventKey}</p>
            <p><strong>Parsed lead:</strong> {inquiry.parsedLead}</p>
            <p><strong>Inquiry draft reply:</strong> {inquiry.draftReply}</p>
            <p><strong>{inquiry.liveSendBoundary}</strong></p>
            <p><strong>Missing-info task:</strong> {inquiry.task}</p>
            <p><strong>Missing info:</strong> {inquiry.missingInfo.join(", ")}</p>
            <small>Audit: {inquiry.audit}</small>
          </article>
        ))}
      </Panel>

      <Panel title="Booking request packet" eyebrow="booking request review packet">
        <article className="record-card">
          <h3>{bookingTriage.request}</h3>
          <p>{bookingTriage.lifecycle}</p>
          <h4>Hard-rule results</h4>
          <ul>{bookingTriage.rules.map((rule) => <li key={rule.id}>{rule.id}: {rule.decision} ({rule.evidence})</li>)}</ul>
          <p><strong>AI recommendation:</strong> {bookingTriage.aiRecommendation}</p>
          <p><strong>Staff confirmation controls:</strong> confirmed booking automation requires staff approval; reject/decline remains human approval gated; special-care acceptance requires care-team approval; behavior exceptions require behavior review.</p>
          <p><strong>Confirmation draft:</strong> {bookingTriage.confirmationDraft}</p>
          <ul>{bookingTriage.gates.map((gate) => <li key={gate}>{gate}</li>)}</ul>
        </article>
      </Panel>

      <Panel title="Booking triage" eyebrow="deterministic gates before AI assistance">
        <div className="record-card">
          <div className="record-header"><h3>{bookingTriage.request}</h3><StatusBadge status="Needs manager review" /></div>
          <p><strong>Lifecycle:</strong> {bookingTriage.lifecycle}</p>
          <p className="muted">Staff can evaluate the request, inspect hard-rule outcomes, and produce draft confirmation copy. Confirmation, rejection, special-care acceptance, behavior exceptions, provider writes, and customer sends stay human approval gates.</p>
        </div>
      </Panel>

      <Panel title="Hard-rule results" eyebrow="availability/capacity and policy primitives">
        <div className="table-list" role="table" aria-label="Booking triage hard-rule results">
          {bookingTriage.rules.map((rule) => (
            <article role="row" className="list-row" key={rule.id}>
              <div><strong>{rule.id}</strong><p>{rule.evidence}</p></div>
              <span className="badge">{rule.decision}</span>
            </article>
          ))}
        </div>
      </Panel>

      <Panel title="AI recommendation" eyebrow="advisory boundary">
        <p>{bookingTriage.aiRecommendation}</p>
        <ul>
          {bookingTriage.gates.map((gate) => <li key={gate}>{gate}</li>)}
        </ul>
      </Panel>

      <Panel title="Staff confirmation controls" eyebrow="confirm/decline UI remains gated">
        <div className="split">
          <button type="button" disabled>Confirm booking (approval-gated)</button>
          <button type="button" disabled>Decline request (approval-gated)</button>
        </div>
        <p className="muted">Confirmed booking automation requires staff approval; reject/decline remains human approval gated.</p>
        <p className="muted">Special-care acceptance requires care-team approval; behavior exceptions require behavior review.</p>
      </Panel>

      <Panel title="Confirmation draft" eyebrow="customer-safe draft generation">
        <blockquote>{bookingTriage.confirmationDraft}</blockquote>
        <p className="muted">Draft only — customer message approval required before send.</p>
      </Panel>

      <Panel title="Task queue" eyebrow="staff work items">
        <div className="table-list" role="table" aria-label="Task queue">
          {taskQueue.map((task) => (
            <article role="row" className="list-row" key={task.id}>
              <div><strong>{task.title}</strong><p>{task.id} • {task.owner} • due {task.due}</p></div>
              <StatusBadge status={task.readiness} />
              <p>{task.reviewGate}</p>
              <small>Audit: {task.auditAction}</small>
            </article>
          ))}
        </div>
      </Panel>

      <Panel title="Document review queue" eyebrow="medical/document review gate">
        {documentReviews.map((document) => (
          <article className="list-row" key={document.id}>
            <div><strong>{document.subject}</strong><p>{document.id} • {document.source} • {document.classification}</p></div>
            <p>{document.state}</p>
            <button type="button" disabled>{document.action} (draft/review only)</button>
          </article>
        ))}
      </Panel>

      <Panel title="Vaccine document MVP" eyebrow="upload extraction review approval boundary">
        <article className="record-card">
          <div className="record-header">
            <h3>{vaccineDocumentReview.sampleUploadAction}</h3>
            <span className="badge">{vaccineDocumentReview.extractionSchema}</span>
          </div>
          <p>{vaccineDocumentReview.uncertaintyPolicy}</p>
          <p><strong>{vaccineDocumentReview.eligibilityBoundary}</strong></p>
          <div className="split">
            {vaccineDocumentReview.reviewActions.map((action) => (
              <button type="button" disabled key={action}>{action} (staff review required)</button>
            ))}
          </div>
          <ul>
            {vaccineDocumentReview.auditEvents.map((eventName) => <li key={eventName}>{eventName}</li>)}
          </ul>
        </article>
      </Panel>

      <Panel title="Staff notes" eyebrow="visibility-separated notes">
        {notes.map((note) => (
          <article className="note-card" key={note.id}>
            <div className="record-header"><strong>{note.label}</strong><span className="badge">{note.visibility}</span></div>
            <p>{note.body}</p>
          </article>
        ))}
      </Panel>

      <Panel title="Incident entry" eyebrow="record incident and draft routing">
        <form className="incident-form" aria-label="Record incident" onSubmit={recordIncident}>
          <div className="form-grid">
            <label>Subject
              <input value={incidentForm.subject} onChange={(event) => setIncidentForm({ ...incidentForm, subject: event.target.value })} />
            </label>
            <label>Incident type
              <select value={incidentForm.type} onChange={(event) => setIncidentForm({ ...incidentForm, type: event.target.value as IncidentType })}>
                {incidentTypeOptions.map((type) => <option key={type} value={type}>{type}</option>)}
              </select>
            </label>
            <label>Proposed severity
              <select value={incidentForm.severitySignal} onChange={(event) => setIncidentForm({ ...incidentForm, severitySignal: event.target.value as SeverityDraft })}>
                {severityOptions.map((severity) => <option key={severity} value={severity}>{severity}</option>)}
              </select>
            </label>
          </div>
          <label>Observed facts
            <textarea value={incidentForm.observedFacts} onChange={(event) => setIncidentForm({ ...incidentForm, observedFacts: event.target.value })} />
          </label>
          <label className="check-row">
            <input checked={incidentForm.ownerDraftRequested} type="checkbox" onChange={(event) => setIncidentForm({ ...incidentForm, ownerDraftRequested: event.target.checked })} />
            Generate owner-message draft (draft only; requires CustomerMessageApproval before send)
          </label>
          <label className="check-row">
            <input checked={incidentForm.eligibilityFlagCandidate} type="checkbox" onChange={(event) => setIncidentForm({ ...incidentForm, eligibilityFlagCandidate: event.target.checked })} />
            Eligibility-impacting flag recommendation only (BehaviorReview + ManagerApproval gate)
          </label>
          <button type="submit">Record incident</button>
        </form>

        <div className="draft-grid" aria-label="Incident classification draft">
          <article className="note-card">
            <h3>Classification draft</h3>
            <p>{currentDraft.classificationDraft}</p>
            <p>{currentDraft.managerReview}</p>
          </article>
          <article className="note-card">
            <h3>Generate owner-message draft</h3>
            <p>{currentDraft.ownerMessageDraft}</p>
          </article>
          <article className="note-card">
            <h3>Create follow-up task</h3>
            <p>{currentDraft.followUpTask}</p>
          </article>
          <article className="note-card">
            <h3>Behavior/eligibility flag boundary</h3>
            <p>{currentDraft.behaviorFlagBoundary}</p>
          </article>
        </div>
      </Panel>

      <Panel title="Manager review queue" eyebrow="manager-review queue for gated incidents">
        {managerReviewQueue.map((incident) => (
          <article className="list-row" key={`${incident.id}-review`}>
            <div><strong>{incident.subject}</strong><p>{incident.id} • {incident.severityDraft} {incident.type}</p></div>
            <StatusBadge status={incident.status} />
            <p>{incident.reviewGates.join(" + ")} — {incident.managerReview}</p>
          </article>
        ))}
      </Panel>

      <Panel title="Incident list" eyebrow="manager-gated incident workflow">
        {incidents.map((incident) => (
          <article className="list-row" key={incident.id}>
            <div><strong>{incident.subject}</strong><p>{incident.id} • {incident.classificationDraft}</p></div>
            <StatusBadge status={incident.status} />
            <p>{incident.nextStep}</p>
            <small>Audit: {incident.auditEvents.join(" / ")}</small>
          </article>
        ))}
      </Panel>

      <Panel title="Audit-visible staff actions" eyebrow="append-only audit evidence">
        <ol className="audit-list">
          {[...auditTrail, ...incidents.flatMap((incident) => incident.auditEvents.map((event) => `${event} — ${incident.id}`))].map((event) => <li key={event}>{event}</li>)}
        </ol>
      </Panel>
    </main>
  );
}
