"use client";

import { FormEvent, ReactNode, useEffect, useMemo, useState } from "react";

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

type ManagerDailyBriefOutcome = "Completed" | "Deferred" | "SuppressedByManager" | "SourceFactWasWrong";

type ManagerDailyBriefAction = {
  id: string;
  kind: string;
  title: string;
  ownerPersona: string;
  priority: string;
  removedManualWork: string;
  rationale: string;
  sourceEvidenceSummary: string;
  sourceRefs: string[];
  reviewGates: string[];
  blockedActions: string[];
  beforeMinutes: number;
  estimatedAfterMinutes: number;
  recommendation: string;
};

type ManagerDailyBriefOutcomeState = {
  actionId: string;
  outcome: ManagerDailyBriefOutcome;
  actualMinutes: number;
  feedback: string;
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

const managerDailyBrief = {
  contextPacketId: "manager-daily-brief-context:local-dev-kennel:2026-06-20",
  correlationId: "manager-daily-brief:local-dev-kennel:2026-06-20",
  operatingDay: "2026-06-20",
  location: "Local/dev kennel",
  preparedFor: "General manager + front-desk lead",
  allowedAgentActions: [
    "summarize_source_evidence",
    "rank_manager_actions",
    "draft_internal_tasks_for_review",
    "record_manager_feedback",
    "estimate_labor_minutes_saved"
  ],
  blockedActions: [
    "change_staff_schedule",
    "mutate_provider_or_pms_record",
    "send_customer_message",
    "move_refund_discount_or_payment",
    "hide_source_data_quality_issue"
  ]
};

const managerDailyBriefActions: ManagerDailyBriefAction[] = [
  {
    id: "MDB-ACT-001",
    kind: "review_demand_against_staffing_plan",
    title: "Review daycare demand against staffing plan",
    ownerPersona: "General manager",
    priority: "high",
    removedManualWork: "Demand-versus-staffing dashboard scan",
    rationale: "Projected all-day-play demand is above the manager threshold while the morning yard handoff still has one unassigned staff block.",
    sourceEvidenceSummary: "analytics::service_demand fact from Gingr snapshot stay-count-v1 plus schedule source ref sched-2026-06-20-am.",
    sourceRefs: ["gingr:service-demand:2026-06-20:all-day-play", "labor:schedule:sched-2026-06-20-am"],
    reviewGates: ["manager_approval"],
    blockedActions: ["change_staff_schedule", "mutate_provider_or_pms_record"],
    beforeMinutes: 45,
    estimatedAfterMinutes: 15,
    recommendation: "Draft only: ask the GM to compare the source refs and approve an internal staffing review task; do not change the schedule automatically."
  },
  {
    id: "MDB-ACT-002",
    kind: "resolve_checkout_exception",
    title: "Resolve Juniper checkout completion exception",
    ownerPersona: "Front-desk lead",
    priority: "medium",
    removedManualWork: "Checkout exception audit",
    rationale: "Juniper has a completed stay with an open handoff note and a checkout-summary draft awaiting lead review.",
    sourceEvidenceSummary: "checkout_completion::Packet references reservation RSV-1012, care-note NOTE-21, and message draft MSG-DRAFT-52.",
    sourceRefs: ["gingr:reservation:RSV-1012", "staff-note:NOTE-21", "message-draft:MSG-DRAFT-52"],
    reviewGates: ["manager_approval"],
    blockedActions: ["send_customer_message", "mutate_provider_or_pms_record"],
    beforeMinutes: 20,
    estimatedAfterMinutes: 8,
    recommendation: "Draft internal task for front-desk review; customer copy remains a draft until CustomerMessageApproval is recorded."
  },
  {
    id: "MDB-ACT-003",
    kind: "approve_retention_follow_up_draft",
    title: "Review grooming rebooking follow-up draft",
    ownerPersona: "Front-desk lead",
    priority: "medium",
    removedManualWork: "Retention follow-up queue prioritization",
    rationale: "A completed boarding stay has safe grooming interest evidence, but owner-facing copy must stay review-gated.",
    sourceEvidenceSummary: "crm_retention::Packet includes prior grooming service, staff evidence, and draft-only follow-up eligibility.",
    sourceRefs: ["gingr:reservation:RSV-1007", "crm-retention:opportunity:GROOM-44", "staff-evidence:NOTE-18"],
    reviewGates: ["customer_message_approval"],
    blockedActions: ["send_customer_message", "move_refund_discount_or_payment"],
    beforeMinutes: 30,
    estimatedAfterMinutes: 10,
    recommendation: "Prepare a follow-up draft for human approval; no customer send, discount, refund, or payment movement is available here."
  },
  {
    id: "MDB-ACT-004",
    kind: "investigate_source_data_quality_issue",
    title: "Investigate missing medication source detail",
    ownerPersona: "General manager",
    priority: "high",
    removedManualWork: "Rediscovering nonblocking source ambiguity during downstream review",
    rationale: "Medication note exists as staff-reported text, but the source record does not prove final medication administration status.",
    sourceEvidenceSummary: "SourceDataQualityIssue from incident INC-DRAFT and reservation RSV-1007; ambiguity remains visible instead of hidden.",
    sourceRefs: ["incident:INC-DRAFT", "gingr:reservation:RSV-1007"],
    reviewGates: ["manager_approval"],
    blockedActions: ["hide_source_data_quality_issue", "mutate_provider_or_pms_record"],
    beforeMinutes: 12,
    estimatedAfterMinutes: 5,
    recommendation: "Ask a manager to verify the source fact and mark Source fact wrong if the source record contradicts the draft."
  }
];

const managerDailyBriefOutcomeOptions: { value: ManagerDailyBriefOutcome; label: string }[] = [
  { value: "Completed", label: "Approve action" },
  { value: "Deferred", label: "Defer action" },
  { value: "SuppressedByManager", label: "Suppress action" },
  { value: "SourceFactWasWrong", label: "Source fact wrong" }
];

const initialManagerDailyBriefOutcome: ManagerDailyBriefOutcomeState = {
  actionId: managerDailyBriefActions[0].id,
  outcome: "Completed",
  actualMinutes: 14,
  feedback: "Reviewed source refs, approved internal staffing review task, no schedule change executed."
};

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

const apiReadinessPosture = {
  routes: [
    {
      path: "/v0/readyz",
      workflow: "runtime_readiness",
      boundary: "api_runtime_dto",
      summary: "Readiness DTO reports database/object storage as not_configured when unset or env_configured_not_verified when local Docker env is present; agent runtime remains fake_deterministic, active adapter: in_memory, and planned adapter: postgres same-contract."
    },
    {
      path: "/v0/ops/metrics/summary",
      workflow: "ops_metrics_summary",
      boundary: "api_runtime_dto",
      summary: "Aggregate-only metrics DTO exposes product labor rollups plus inquiry_count, review_packet_count, audit_event_count, and outcome_count without raw customer/provider payloads."
    }
  ],
  safety: "live_side_effects: disabled; live customer messaging, provider/PMS writes, payment/refund movement, and hidden source cleanup are unavailable in this demo surface.",
  productionPlan: "Prometheus/OpenTelemetry plan: request_latency, error_rate, queue_depth, dead_letter_count, review_sla, outbox_failures, and worker_lease_age."
};

type ApiSourceLabel = "Live local API data" | "Fallback fixture data" | "Unavailable local API data";
type ApiProofCard = {
  title: string;
  endpoint: string;
  sourceLabel: ApiSourceLabel;
  status: string;
  summary: string;
  freshness: string;
};

type LocalApiProof = {
  sourceLabel: ApiSourceLabel;
  baseUrl: string;
  posture: string;
  cards: ApiProofCard[];
  caveat?: string;
};

const petResortApiBaseUrlEnvName = "NEXT_PUBLIC_PET_RESORT_API_BASE_URL";
const staffWebApiProxyBase = "/api/local-demo";
const fallbackApiBaseLabel = `not configured — staff-web uses /api/local-demo as a same-origin runtime proxy; set PET_RESORT_API_BASE_URL on the server or ${petResortApiBaseUrlEnvName} for direct browser fetches`;
const noLiveSideEffectBoundary = "No live provider/PMS writes, customer sends, payments, refunds, discounts, schedule changes, or medical/safety decisions.";

const fallbackLocalApiProof: LocalApiProof = {
  sourceLabel: "Fallback fixture data",
  baseUrl: fallbackApiBaseLabel,
  posture: "DB-backed when API is reachable; deterministic fallback when API is unavailable.",
  cards: [
    {
      title: "API health/readiness",
      endpoint: "/v0/readyz",
      sourceLabel: "Fallback fixture data",
      status: "fallback static readiness",
      summary: "Local presentation fallback mirrors the safe API posture: database/object storage not_configured, fake deterministic agent runtime, active adapter: in_memory, planned adapter: postgres same-contract.",
      freshness: "Loaded from deterministic fallback fixtures in staff-web; no terminal or API server required."
    },
    {
      title: "Labor outcomes",
      endpoint: "/v0/ops/metrics/summary",
      sourceLabel: "Fallback fixture data",
      status: "fallback aggregate metrics",
      summary: "Shows product labor rollups, inquiry_count, review_packet_count, audit_event_count, and outcome_count as aggregate-only proof without raw customer/provider payloads.",
      freshness: "Fallback counts are deterministic presentation fixtures; live/local counters replace them when the API responds."
    },
    {
      title: "Source-quality backlog",
      endpoint: "/v0/read-models/source-quality-backlog",
      sourceLabel: "Fallback fixture data",
      status: "static fallback; local DB read model unavailable",
      summary: "Backlog route is wired for the Data-Quality Hygiene read model; fallback keeps source ambiguity visible without claiming a fresh DB-backed projection.",
      freshness: "Use this card only as a no-terminal/static fallback when the local read-model API cannot be reached."
    },
    {
      title: "Audit lineage/freshness",
      endpoint: "/v0/readyz + /v0/ops/metrics/summary",
      sourceLabel: "Fallback fixture data",
      status: "review-gated audit proof",
      summary: "Audit lineage is represented by append-only event counts, source refs, correlation IDs, and review/outcome records; outbox/review records are handoff candidates, not send permission.",
      freshness: noLiveSideEffectBoundary
    }
  ]
};

function textField(value: unknown, fallback: string): string {
  return typeof value === "string" && value.length > 0 ? value : fallback;
}

function recordField(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value) ? value as Record<string, unknown> : undefined;
}

async function fetchLocalApiJson(baseUrl: string, endpoint: string, signal: AbortSignal): Promise<{ status: number; payload: Record<string, unknown> }> {
  const response = await fetch(`${baseUrl.replace(/\/$/, "")}${endpoint}`, { signal });
  const contentType = response.headers.get("content-type") ?? "";
  if (!contentType.toLowerCase().includes("application/json")) {
    throw new Error(`Local API ${endpoint} returned non-JSON content (${response.status})`);
  }
  const payload = await response.json() as Record<string, unknown>;
  return { status: response.status, payload };
}

function buildLiveApiProof(
  baseUrl: string,
  ready: { status: number; payload: Record<string, unknown> },
  metrics: { status: number; payload: Record<string, unknown> },
  backlog: { status: number; payload: Record<string, unknown> }
): LocalApiProof {
  const workflowRepository = recordField(ready.payload.workflow_repository);
  const observability = recordField(ready.payload.observability);
  const runtimeCounters = recordField(metrics.payload.local_runtime_counters);
  const productLaborMetrics = recordField(metrics.payload.product_labor_metrics);
  const backlogDatabase = recordField(backlog.payload.database);
  const backlogRecords = Array.isArray(backlog.payload.records) ? backlog.payload.records.length : 0;
  const backlogStatus = textField(backlogDatabase?.status, "unknown");
  const backlogIsConnected = backlog.status === 200 && backlogStatus === "connected";

  return {
    sourceLabel: "Live local API data",
    baseUrl,
    posture: "DB-backed when API is reachable; deterministic fallback when API is unavailable.",
    cards: [
      {
        title: "API health/readiness",
        endpoint: "/v0/readyz",
        sourceLabel: "Live local API data",
        status: `HTTP ${ready.status} • ${textField(ready.payload.service, "pet-resort-api")} • database ${textField(ready.payload.database, "unknown")}`,
        summary: `agent_runtime ${textField(ready.payload.agent_runtime, "unknown")}; active adapter: ${textField(workflowRepository?.active_adapter, "unknown")}; planned adapter: ${textField(workflowRepository?.postgres_adapter, "unknown").replaceAll("_", " ")}; live customer/provider writes ${textField(ready.payload.live_customer_messaging, "unknown")}/${textField(ready.payload.live_provider_writes, "unknown")}.`,
        freshness: textField(observability?.metrics_scope, "runtime freshness reported by local API readiness DTO")
      },
      {
        title: "Labor outcomes",
        endpoint: "/v0/ops/metrics/summary",
        sourceLabel: "Live local API data",
        status: `HTTP ${metrics.status} • aggregate_only metrics`,
        summary: `Runtime counters: inquiry_count ${String(runtimeCounters?.inquiry_count ?? 0)}, review_packet_count ${String(runtimeCounters?.review_packet_count ?? 0)}, audit_event_count ${String(runtimeCounters?.audit_event_count ?? 0)}, outcome_count ${String(runtimeCounters?.outcome_count ?? 0)}. Labor rollups are API-owned product metrics: ${Object.keys(productLaborMetrics ?? {}).join(", ") || "none yet"}.`,
        freshness: "Live/local API response fetched in the browser; payload logging remains disabled."
      },
      {
        title: "Source-quality backlog",
        endpoint: "/v0/read-models/source-quality-backlog",
        sourceLabel: backlogIsConnected ? "Live local API data" : "Unavailable local API data",
        status: backlog.status === 200 ? `HTTP ${backlog.status} • database ${backlogStatus}` : `HTTP ${backlog.status} • read model ${backlogStatus}`,
        summary: backlogIsConnected
          ? `Source-quality backlog route returned ${backlogRecords} live/local DB-backed read-model records; use rows only as local DB/read-model evidence.`
          : `Source-quality backlog route is wired but did not return DB-backed rows (${backlogStatus}); staff-web labels the read-model proof unavailable instead of claiming live DB evidence.`,
        freshness: `request/correlation: ${textField(backlog.payload.request_id, "not returned")} / ${textField(backlog.payload.correlation_id, "not returned")}`
      },
      {
        title: "Audit lineage/freshness",
        endpoint: "/v0/readyz + /v0/ops/metrics/summary",
        sourceLabel: "Live local API data",
        status: "review-gated audit proof",
        summary: "Readiness and metrics came from the live local API. Outbox/review records remain handoff candidates, not permission to send.",
        freshness: noLiveSideEffectBoundary
      }
    ]
  };
}

function StatusBadge({ status }: { status: Readiness | IncidentStatus }) {
  return <span className="badge">{status.replaceAll("_", " ")}</span>;
}

function Panel({ title, eyebrow, children }: Readonly<{ title: string; eyebrow?: string; children: ReactNode }>) {
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
  const [managerDailyBriefOutcome, setManagerDailyBriefOutcome] = useState<ManagerDailyBriefOutcomeState>(initialManagerDailyBriefOutcome);
  const currentDraft = useMemo(() => buildIncident(incidentForm, "INC-DRAFT"), [incidentForm]);
  const managerReviewQueue = incidents.filter((incident) =>
    incident.reviewGates.includes("ManagerApproval") || incident.reviewGates.includes("BehaviorReview")
  );
  const selectedDailyBriefAction = managerDailyBriefActions.find((action) => action.id === managerDailyBriefOutcome.actionId) ?? managerDailyBriefActions[0];
  const estimatedMinutesSaved = selectedDailyBriefAction.beforeMinutes - selectedDailyBriefAction.estimatedAfterMinutes;
  const actualMinutesSaved = selectedDailyBriefAction.beforeMinutes - managerDailyBriefOutcome.actualMinutes;
  const dailyBriefTotalBeforeMinutes = managerDailyBriefActions.reduce((total, action) => total + action.beforeMinutes, 0);
  const dailyBriefTotalEstimatedAfterMinutes = managerDailyBriefActions.reduce((total, action) => total + action.estimatedAfterMinutes, 0);
  const dailyBriefTotalEstimatedSavedMinutes = dailyBriefTotalBeforeMinutes - dailyBriefTotalEstimatedAfterMinutes;
  const [localApiProof, setLocalApiProof] = useState<LocalApiProof>(fallbackLocalApiProof);

  useEffect(() => {
    const configuredBaseUrl = process.env.NEXT_PUBLIC_PET_RESORT_API_BASE_URL?.trim() || staffWebApiProxyBase;

    const abortController = new AbortController();
    const apiBaseUrl = configuredBaseUrl;

    async function loadLocalApiProof() {
      try {
        const [ready, metrics, backlog] = await Promise.all([
          fetchLocalApiJson(apiBaseUrl, "/v0/readyz", abortController.signal),
          fetchLocalApiJson(apiBaseUrl, "/v0/ops/metrics/summary", abortController.signal),
          fetchLocalApiJson(apiBaseUrl, "/v0/read-models/source-quality-backlog", abortController.signal)
        ]);
        if (ready.status >= 500 || metrics.status >= 500) {
          throw new Error(`Local API readiness/metrics unavailable (${ready.status}/${metrics.status})`);
        }
        setLocalApiProof(buildLiveApiProof(apiBaseUrl, ready, metrics, backlog));
      } catch (error) {
        if (!abortController.signal.aborted) {
          setLocalApiProof({
            ...fallbackLocalApiProof,
            baseUrl: apiBaseUrl,
            caveat: `Fell back after local API fetch failed: ${error instanceof Error ? error.message : "unknown error"}`
          });
        }
      }
    }

    void loadLocalApiProof();

    return () => abortController.abort();
  }, []);

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
          "API readiness and observability contract",
          "Live API / fallback proof",
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
          "Manager Daily Brief",
          "Daily brief action review",
          "Review gates",
          "Blocked action boundaries",
          "Outcome capture",
          "Labor savings evidence",
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

      <Panel title="API readiness and observability contract" eyebrow="DTO/API/metrics posture">
        <p className="muted">
          Non-technical demo proof that the staff surface maps to product-owned API contracts: local readiness is honest,
          metrics are aggregate-only, review/audit counts are visible, and live side effects remain disabled.
        </p>
        <div className="card-grid">
          {apiReadinessPosture.routes.map((route) => (
            <article className="record-card" key={route.path}>
              <div className="record-header">
                <h3>{route.path}</h3>
                <span className="badge">{route.workflow}</span>
              </div>
              <p><strong>DTO boundary:</strong> {route.boundary}</p>
              <p>{route.summary}</p>
            </article>
          ))}
        </div>
        <div className="metric-grid">
          <div><strong>{inquiryIntakeQueue.length}</strong><span>inquiry_count local demo</span></div>
          <div><strong>{documentReviews.length + incidents.length}</strong><span>review_packet_count local demo</span></div>
          <div><strong>{auditTrail.length}</strong><span>audit_event_count baseline</span></div>
          <div><strong>1</strong><span>outcome_count demo record</span></div>
        </div>
        <p><strong>Safety:</strong> {apiReadinessPosture.safety}</p>
        <p><strong>Production metrics:</strong> {apiReadinessPosture.productionPlan}</p>
      </Panel>

      <Panel title="Live API / fallback proof" eyebrow="live local API loading + no-terminal fallback">
        <div className="record-card">
          <div className="record-header">
            <div>
              <h3>{localApiProof.sourceLabel}</h3>
              <p>Browser API base ({petResortApiBaseUrlEnvName} or same-origin proxy): {localApiProof.baseUrl}</p>
            </div>
            <span className="badge">{localApiProof.sourceLabel}</span>
          </div>
          <p>{localApiProof.posture}</p>
          <p><strong>Safety boundary:</strong> {noLiveSideEffectBoundary}</p>
          {localApiProof.caveat ? <p className="muted">{localApiProof.caveat}</p> : null}
        </div>
        <div className="card-grid">
          {localApiProof.cards.map((card) => (
            <article className="record-card" key={`${card.title}-${card.endpoint}`}>
              <div className="record-header">
                <h3>{card.title}</h3>
                <span className="badge">{card.sourceLabel}</span>
              </div>
              <p><strong>Endpoint:</strong> {card.endpoint}</p>
              <p><strong>Status:</strong> {card.status}</p>
              <p>{card.summary}</p>
              <small>Freshness/lineage: {card.freshness}</small>
            </article>
          ))}
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

      <Panel title="Manager Daily Brief" eyebrow="source-grounded labor-saving loop">
        <div className="record-card">
          <div className="record-header">
            <div>
              <h3>{managerDailyBrief.location} • {managerDailyBrief.operatingDay}</h3>
              <p>{managerDailyBrief.preparedFor}</p>
            </div>
            <span className="badge">draft/review only</span>
          </div>
          <p><strong>Context packet:</strong> {managerDailyBrief.contextPacketId}</p>
          <p><strong>Correlation:</strong> {managerDailyBrief.correlationId}</p>
          <p className="muted">Deterministic app owns source facts, policy, review gates, outcome persistence, audit, and side effects. AI can summarize and rank; staff/managers approve, defer, suppress, or mark source fact wrong.</p>
          <div className="metric-grid">
            <div><strong>{managerDailyBriefActions.length}</strong><span>daily brief actions</span></div>
            <div><strong>{dailyBriefTotalBeforeMinutes}</strong><span>estimated manual minutes before</span></div>
            <div><strong>{dailyBriefTotalEstimatedAfterMinutes}</strong><span>estimated minutes after review</span></div>
            <div><strong>{dailyBriefTotalEstimatedSavedMinutes}</strong><span>estimated minutes saved</span></div>
          </div>
        </div>
      </Panel>

      <Panel title="Daily brief action review" eyebrow="actions with rationale/source evidence summary">
        <div className="table-list" role="table" aria-label="Manager Daily Brief actions">
          {managerDailyBriefActions.map((action) => (
            <article role="row" className="list-row" key={action.id}>
              <div>
                <strong>{action.title}</strong>
                <p>{action.id} • {action.kind} • owner: {action.ownerPersona} • priority: {action.priority}</p>
                <p><strong>Removed manual work:</strong> {action.removedManualWork}</p>
              </div>
              <span className="badge">{action.beforeMinutes - action.estimatedAfterMinutes} min saved est.</span>
              <div>
                <p><strong>Rationale:</strong> {action.rationale}</p>
                <p><strong>source evidence summary:</strong> {action.sourceEvidenceSummary}</p>
                <p><strong>Draft recommendation:</strong> {action.recommendation}</p>
              </div>
            </article>
          ))}
        </div>
      </Panel>

      <Panel title="Review gates" eyebrow="required human decision boundaries">
        {managerDailyBriefActions.map((action) => (
          <article className="list-row" key={`${action.id}-gates`}>
            <div><strong>{action.title}</strong><p>{action.id}</p></div>
            <span className="badge">{action.reviewGates.join(" + ")}</span>
            <p>Sensitive actions remain draft/review oriented; approval records must cite source refs: {action.sourceRefs.join(", ")}.</p>
          </article>
        ))}
      </Panel>

      <Panel title="Blocked action boundaries" eyebrow="visible app-owned no-go list">
        <div className="split">
          <div>
            <h3>Workflow blocked actions</h3>
            <ul>{managerDailyBrief.blockedActions.map((blockedAction) => <li key={blockedAction}>{blockedAction}</li>)}</ul>
          </div>
          <div>
            <h3>Allowed agent actions</h3>
            <ul>{managerDailyBrief.allowedAgentActions.map((allowedAction) => <li key={allowedAction}>{allowedAction}</li>)}</ul>
          </div>
        </div>
        <p className="muted">This surface cannot change staff schedules, mutate provider/PMS records, send customer messages, move refunds/discounts/payments, or hide source data-quality issues.</p>
      </Panel>

      <Panel title="Outcome capture" eyebrow="approve/defer/suppress/source-fact-wrong">
        <form className="incident-form" aria-label="Manager Daily Brief outcome capture" onSubmit={(event) => event.preventDefault()}>
          <div className="form-grid">
            <label>Daily brief action
              <select value={managerDailyBriefOutcome.actionId} onChange={(event) => setManagerDailyBriefOutcome({ ...managerDailyBriefOutcome, actionId: event.target.value })}>
                {managerDailyBriefActions.map((action) => <option key={action.id} value={action.id}>{action.title}</option>)}
              </select>
            </label>
            <label>Outcome
              <select value={managerDailyBriefOutcome.outcome} onChange={(event) => setManagerDailyBriefOutcome({ ...managerDailyBriefOutcome, outcome: event.target.value as ManagerDailyBriefOutcome })}>
                {managerDailyBriefOutcomeOptions.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
              </select>
            </label>
            <label>actual minutes spent
              <input min="1" type="number" value={managerDailyBriefOutcome.actualMinutes} onChange={(event) => setManagerDailyBriefOutcome({ ...managerDailyBriefOutcome, actualMinutes: Number(event.target.value) })} />
            </label>
          </div>
          <label>Manager/staff feedback
            <textarea value={managerDailyBriefOutcome.feedback} onChange={(event) => setManagerDailyBriefOutcome({ ...managerDailyBriefOutcome, feedback: event.target.value })} />
          </label>
          <div className="split">
            {managerDailyBriefOutcomeOptions.map((option) => (
              <button type="button" key={option.value} onClick={() => setManagerDailyBriefOutcome({ ...managerDailyBriefOutcome, outcome: option.value })}>{option.label}</button>
            ))}
          </div>
          <p className="muted">Outcome capture records staff evidence only; live_side_effects_allowed remains false and blocked actions stay visible after submission.</p>
        </form>
      </Panel>

      <Panel title="Labor savings evidence" eyebrow="estimated vs actual labor minutes saved">
        <article className="record-card">
          <div className="record-header">
            <div>
              <h3>{selectedDailyBriefAction.title}</h3>
              <p>{managerDailyBriefOutcome.outcome} • {managerDailyBriefOutcome.feedback}</p>
            </div>
            <span className="badge">{actualMinutesSaved} actual minutes saved</span>
          </div>
          <div className="metric-grid">
            <div><strong>{selectedDailyBriefAction.beforeMinutes}</strong><span>before minutes</span></div>
            <div><strong>{selectedDailyBriefAction.estimatedAfterMinutes}</strong><span>estimated after minutes</span></div>
            <div><strong>{estimatedMinutesSaved}</strong><span>estimated minutes saved</span></div>
            <div><strong>{managerDailyBriefOutcome.actualMinutes}</strong><span>actual minutes spent</span></div>
          </div>
          <p><strong>estimated vs actual labor minutes saved:</strong> {estimatedMinutesSaved} estimated vs {actualMinutesSaved} actual.</p>
          <small>Audit: manager_daily_brief_outcome_recorded / policy_owner: deterministic_app / no live side effects.</small>
        </article>
      </Panel>

      <Panel title="Audit-visible staff actions" eyebrow="append-only audit evidence">
        <ol className="audit-list">
          {[...auditTrail, ...incidents.flatMap((incident) => incident.auditEvents.map((event) => `${event} — ${incident.id}`))].map((event) => <li key={event}>{event}</li>)}
        </ol>
      </Panel>
    </main>
  );
}
