export type SourceEvidenceCard = {
  id: string;
  title: string;
  sourceName: string;
  rawSignal: string;
  sourceRef: string;
  freshness: string;
  caveat: string;
  readOnlyState: string;
};

export type OwnedBackendSpineStage = {
  id: string;
  title: string;
  businessPurpose: string;
  proofLabel: string;
};

export type LaborToolPortfolioCard = {
  id: string;
  name: string;
  summary: string;
  sourceSignals: string[];
  normalizedFacts: string[];
  workflowPacket: string;
  reviewGate: string;
  lockedSideEffects: string[];
  outputReadout: string;
  outcomeMetric: string;
  proofHooks: string[];
  lineageSteps: string[];
  lineageId: string;
};

export type LineageEdge = {
  id: string;
  toolId: string;
  source: string;
  fact: string;
  workflowPacket: string;
  gate: string;
  outcomeReadModel: string;
};

export type SafetyLock = {
  id: string;
  label: string;
  reason: string;
};

export type ProofArtifact = {
  id: string;
  label: string;
  existsNow: string;
  syntheticBoundary: string;
  realAccessValidation: string;
  inspect: string[];
};

export type PortfolioMetric = {
  label: string;
  value: string;
  sub: string;
};

export type PilotAskItem = {
  label: string;
  detail: string;
};

export type CloseCard = {
  label: string;
  value: string;
  detail: string;
};

export type CloseListItem = {
  label: string;
  detail: string;
};

export const portfolioMetrics: PortfolioMetric[] = [
  {
    label: "Sample portfolio operating risk",
    value: "$25.1k",
    sub: "$25.1k modeled monthly labor + rework exposure"
  },
  {
    label: "Manager time shifted",
    value: "48m",
    sub: "48 manager minutes shifted from source chasing to review"
  },
  {
    label: "Reusable tools",
    value: "4",
    sub: "4 reusable tools on one owned backend"
  },
  {
    label: "Live side effects",
    value: "0",
    sub: "0 live side effects: sends, PMS writes, schedule changes, payments, medical decisions"
  }
];

export const sourceEvidenceCards: SourceEvidenceCard[] = [
  {
    id: "reservation-sample",
    title: "PMS reservation feed sample",
    sourceName: "PMS reservation export",
    rawSignal: "12 arrivals before 10 · Jul 3 boarding · enrichment add-ons",
    sourceRef: "source ref pms_sample/reservation_snapshot/2026-07-03",
    freshness: "sample snapshot observed 6:15am",
    caveat: "read-only source evidence; source refs preserved; sample rows only",
    readOnlyState: "read-only"
  },
  {
    id: "labor-sample",
    title: "Labor schedule / timeclock export sample",
    sourceName: "labor schedule / timeclock export",
    rawSignal: "AM role coverage -2 vs forecast; kennel lead starts 11:00",
    sourceRef: "source ref labor_sample/coverage_2026-07-03",
    freshness: "sample schedule snapshot; freshness + caveats visible",
    caveat: "coverage projection is modelled; manager owns staffing choice",
    readOnlyState: "read-only"
  },
  {
    id: "document-sample",
    title: "Uploaded document sample",
    sourceName: "uploaded vaccine document sample",
    rawSignal: "rabies attachment present; expiry field unreadable",
    sourceRef: "source ref document_sample/rabies_miso",
    freshness: "sample upload metadata retained",
    caveat: "document ambiguity requires human validation",
    readOnlyState: "read-only"
  },
  {
    id: "room-sample",
    title: "Room inventory projection sample",
    sourceName: "room inventory projection sample",
    rawSignal: "premium rooms tight; quiet-room request intersects capacity",
    sourceRef: "source ref capacity_sample/premium_rooms",
    freshness: "sample projection as of planning cut",
    caveat: "projection caveat visible; no schedule/capacity write",
    readOnlyState: "read-only"
  },
  {
    id: "bi-sample",
    title: "BI query inventory",
    sourceName: "BI query inventory sample",
    rawSignal: "recurring labor, cleanup backlog, and outbox posture questions",
    sourceRef: "source ref bi_sample/query_inventory",
    freshness: "sample query list mapped to read-model proof",
    caveat: "KPI meaning requires owner approval before production reporting claims",
    readOnlyState: "read-only"
  }
];

export const ownedBackendSpineStages: OwnedBackendSpineStage[] = [
  {
    id: "source-evidence",
    title: "Read-only source evidence",
    businessPurpose: "Source systems remain evidence while NVA owns review, outcomes, and reporting meaning.",
    proofLabel: "provenance_snapshot_id"
  },
  {
    id: "owned-facts",
    title: "NVA-owned operating facts",
    businessPurpose: "NVA keeps the work rules, review decisions, labor outcomes, and reporting meaning in its own operating layer.",
    proofLabel: "owned_fact_id"
  },
  {
    id: "workflow-packets",
    title: "Workflow packets",
    businessPurpose: "Reusable tools receive the same reviewable packet shape instead of one-off report copy.",
    proofLabel: "workflow_packet_id"
  },
  {
    id: "review-gates",
    title: "Review gates",
    businessPurpose: "Manager review stays open while unsafe side effects stay locked.",
    proofLabel: "review_gate_id"
  },
  {
    id: "audit-outcomes",
    title: "Audit + outcome events",
    businessPurpose: "Reviewed dispositions become auditable outcomes for labor and source-quality learning.",
    proofLabel: "audit_event_id"
  },
  {
    id: "read-models",
    title: "Read models for BI",
    businessPurpose: "Portfolio reporting reads reviewed operating meaning instead of reverse-engineering provider tables.",
    proofLabel: "read_model_projection"
  }
];

export const laborToolPortfolioCards: LaborToolPortfolioCard[] = [
  {
    id: "manager-daily-brief",
    name: "Manager Daily Brief",
    summary: "Turns messy morning source evidence into a reviewed action list before labor waste starts.",
    sourceSignals: ["reservation arrivals/departures", "labor schedule/timeclock coverage", "room/capacity projection", "open document/care-note flags"],
    normalizedFacts: ["arrival density", "coverage gap", "role pressure", "labor/rework exposure"],
    workflowPacket: "manager action packet with ranked risks, evidence links, and review owners",
    reviewGate: "manager chooses staffing response; no autonomous schedule change; customer communication remains locked",
    lockedSideEffects: ["schedule change locked", "customer send locked", "staffing mandate locked"],
    outputReadout: "ranked morning brief: arrival-density pressure, coverage gap, and care/document blockers",
    outcomeMetric: "manager prep minutes shifted from source chasing to review; modeled avoidable labor/rework dollars; outcome record labor_rework",
    proofHooks: ["workflow_packet_id=manager_daily_brief", "review_gate_id=manager_shift_review", "audit_event_id=reviewed_disposition", "read_model_projection=labor_rework"] ,
    lineageSteps: ["PMS export row: 12 arrivals before 10", "SourceSnapshot pms_sample/reservation_snapshot/2026-07-03", "arrival_density fact", "manager action packet", "outcome record labor_rework"],
    lineageId: "lineage-manager-daily-brief"
  },
  {
    id: "data-quality-hygiene",
    name: "Data Quality Hygiene",
    summary: "Turns source ambiguity into reviewable cleanup work instead of front-desk surprises.",
    sourceSignals: ["unreadable vaccine dates", "duplicate/missing pet or owner fields", "unsupported source values", "BI cleanup exceptions"],
    normalizedFacts: ["source quality issue", "affected workflow", "blocker reason", "source-field mapping gap"],
    workflowPacket: "reviewer packet with reason, source refs, suggested next review step, and blocked-action explanation",
    reviewGate: "human validates ambiguous documents/fields; no provider repair/write; no customer message; no destructive merge/delete",
    lockedSideEffects: ["PMS/provider write locked", "customer send locked", "destructive merge/delete locked"],
    outputReadout: "internal cleanup queue with wrong-source/resolved disposition options",
    outcomeMetric: "cleanup minutes saved, front-desk rework avoided, source-quality backlog aging, reviewed-resolution rate",
    proofHooks: ["workflow_packet_id=data_quality_hygiene", "blocked_draft_validation_ok", "review_gate_id=cleanup_review", "read_model_projection=source_quality_backlog"],
    lineageSteps: ["unclear rabies proof", "source_quality_issue fact", "reviewer packet", "wrong-source/resolved disposition", "source_quality_backlog"],
    lineageId: "lineage-data-quality-hygiene"
  },
  {
    id: "intake-booking-triage",
    name: "Intake / Booking Triage",
    summary: "Prepares the front desk with evidence and missing-info checks; it does not book or message for them.",
    sourceSignals: ["inbound booking request sample", "pet profile notes", "vaccination/document status", "room/capacity projection"],
    normalizedFacts: ["intake readiness", "missing requirement", "capacity fit", "triage priority"],
    workflowPacket: "triage queue item with missing-info checklist, evidence links, and locked candidate customer response",
    reviewGate: "manager/front-desk review before any customer response; no booking confirmation; no PMS/provider write; no payment/discount action",
    lockedSideEffects: ["send locked", "booking confirmation locked", "PMS/provider write locked", "payment/refund/discount locked"],
    outputReadout: "safe draft reply and missing-info checklist for front-desk review",
    outcomeMetric: "minutes saved per intake review; avoidable back-and-forth reduced; intake_queue readout remains sample/modelled",
    proofHooks: ["workflow_packet_id=intake_booking_triage", "outbox_candidate_id=triage_reply_candidate", "review_gate_id=front_desk_or_manager", "customer_send_locked=true"],
    lineageSteps: ["intake message: two dogs for holiday boarding", "missing-info checklist", "safe draft reply", "send locked", "intake_queue readout"],
    lineageId: "lineage-intake-booking-triage"
  },
  {
    id: "bi-read-model",
    name: "BI / Read Model Reporting",
    summary: "Gives portfolio reporting NVA-owned meaning instead of reverse-engineering provider tables.",
    sourceSignals: ["current BI query inventory", "workflow outcome events", "review dispositions", "labor/rework metrics"],
    normalizedFacts: ["reviewed business meaning", "labor minutes saved", "workflow aging", "projection freshness"],
    workflowPacket: "NVA read model over reviewed operating meaning, source caveats, and pilot comparison",
    reviewGate: "KPI definitions require owner approval; caveats remain visible; production reporting claims wait for read-only validation",
    lockedSideEffects: ["production reporting claim locked", "provider-table write locked", "KPI definition change locked"],
    outputReadout: "portfolio labor/rework trend and source-quality backlog readout",
    outcomeMetric: "analyst/reporting cleanup time reduced; clearer KPI definitions; fewer ad hoc spreadsheet reconciliations",
    proofHooks: ["read_model_projection=portfolio_operations", "Operations API contract", "audit_event_id=workflow_outcome", "BI query mapping table"],
    lineageSteps: ["provider-shaped export", "NVA read model", "portfolio labor/rework trend", "KPI owner review", "portfolio_operations projection"],
    lineageId: "lineage-bi-read-model"
  }
];

export const lineageEdges: LineageEdge[] = [
  {
    id: "lineage-manager-daily-brief",
    toolId: "manager-daily-brief",
    source: "reservation-sample + labor-sample + room-sample + document-sample",
    fact: "owned fact: arrival_density + coverage_gap + care/document blocker",
    workflowPacket: "workflow_packet_id=manager_daily_brief",
    gate: "review_gate_id=manager_shift_review",
    outcomeReadModel: "audit_event_id=reviewed_disposition -> read_model_projection=labor_rework"
  },
  {
    id: "lineage-data-quality-hygiene",
    toolId: "data-quality-hygiene",
    source: "document-sample + bi-sample",
    fact: "owned fact: source_quality_issue + blocker_reason + mapping_gap",
    workflowPacket: "workflow_packet_id=data_quality_hygiene",
    gate: "review_gate_id=cleanup_review",
    outcomeReadModel: "audit_event_id=cleanup_resolution -> read_model_projection=source_quality_backlog"
  },
  {
    id: "lineage-intake-booking-triage",
    toolId: "intake-booking-triage",
    source: "reservation-sample + document-sample + room-sample",
    fact: "owned fact: intake_readiness + missing_requirement + capacity_fit",
    workflowPacket: "workflow_packet_id=intake_booking_triage",
    gate: "review_gate_id=front_desk_or_manager",
    outcomeReadModel: "outbox_candidate_id=triage_reply_candidate -> read_model_projection=intake_queue"
  },
  {
    id: "lineage-bi-read-model",
    toolId: "bi-read-model",
    source: "bi-sample + workflow outcome events",
    fact: "owned fact: reviewed_business_meaning + KPI_definition_owner",
    workflowPacket: "workflow_packet_id=bi_read_model_reporting",
    gate: "review_gate_id=kpi_definition_owner",
    outcomeReadModel: "audit_event_id=report_definition_review -> read_model_projection=portfolio_operations"
  }
];

export const safetyLocks: SafetyLock[] = [
  { id: "write", label: "write locked", reason: "No source/provider mutation path is enabled." },
  { id: "manager-review", label: "manager review open", reason: "Internal review and evidence validation are allowed." },
  { id: "outbox", label: "outbox candidate only", reason: "Candidate drafts remain locked until owner-approved gates exist." },
  { id: "customer-send", label: "customer send locked", reason: "No member/customer communication is sent." },
  { id: "provider-write", label: "PMS/provider write locked", reason: "No PMS, Gingr, provider, room, profile, or booking record is written." },
  { id: "schedule", label: "schedule change locked", reason: "No autonomous schedule or capacity change is made." },
  { id: "payment", label: "payment/refund/discount locked", reason: "No financial action path is enabled." },
  { id: "medical", label: "medical/safety decision locked", reason: "Care or safety trade-offs require human policy and manager approval." },
  { id: "staffing", label: "staffing mandate locked", reason: "The platform recommends review work; it does not mandate staffing." }
];

export const proofArtifacts: ProofArtifact[] = [
  {
    id: "source-provenance-boundary",
    label: "source adapter / provenance boundary",
    existsNow: "Runtime contract docs and adapter-facing code quarantine provider-shaped DTOs as source evidence; provenance_snapshot_id/source refs keep raw signal, freshness, and caveat with derived facts.",
    syntheticBoundary: "Current public page uses sample workspace evidence and fixture/source snapshots only; it does not claim live NVA/Gingr credentials or live records.",
    realAccessValidation: "Read-only provider exports would validate field mappings, source drift, freshness rules, and whether source refs cover each resort workflow slice.",
    inspect: ["docs/architecture/runtime-contract-boundaries.md", "migrations/0001_mvp_foundation.sql"]
  },
  {
    id: "owned-operations-api-openapi",
    label: "owned operations API / OpenAPI",
    existsNow: "Operations API contract includes reviewable workflow responses, readiness/metrics paths, ReviewGateRef, AuditRef, and live_side_effects_allowed=false posture in checked OpenAPI and Rust handlers.",
    syntheticBoundary: "The public staff-web proof is static because a private local API would be brittle on nva-demo.eman.network; no browser claim depends on a running private service.",
    realAccessValidation: "A read-only pilot API run would validate payload shapes against real source extracts while keeping customer sends, PMS writes, schedules, payments, and safety decisions locked.",
    inspect: ["apps/api/openapi/owned-operations-v0.openapi.json", "apps/api/src/http.rs"]
  },
  {
    id: "storage-projections-outcomes",
    label: "storage / projections / outcome records",
    existsNow: "Storage records cover manager daily-brief labor outcomes, data-quality hygiene outcomes, source-system refs, approval/outbox ids, and public projections suitable for Postgres or fixtures.",
    syntheticBoundary: "Records are represented as deterministic local/fixture-backed proof here; storage code does not authorize live provider writes or customer messaging.",
    realAccessValidation: "Read-only pilot data would validate idempotency keys, source-record refs, outcome persistence, and projection freshness without mutating PMS/provider records.",
    inspect: ["storage/src/operations.rs", "migrations/0001_mvp_foundation.sql"]
  },
  {
    id: "review-gates-blocked-actions",
    label: "review gates / blocked actions",
    existsNow: "Workflow modules encode manager/front-desk review gates, blocked actions, workflow_packet_id references, and review packets for manager brief, data-quality hygiene, and booking triage.",
    syntheticBoundary: "The sample workspace leaves manager review open but locks customer send, PMS/provider write, booking confirmation, payment/discount, schedule, medical/safety, and staffing mandate paths.",
    realAccessValidation: "A pilot would validate which roles approve each packet and which source exceptions should remain blocked before any production write path exists.",
    inspect: ["app/src/manager_daily_brief.rs", "app/src/data_quality_hygiene.rs", "app/src/booking_triage.rs"]
  },
  {
    id: "audit-outbox-posture",
    label: "audit / outbox posture",
    existsNow: "Audit/outcome events, approval records, outbox_candidate_id, blocked-action policy, and live_side_effects_allowed=false are documented and exposed as candidate/review posture, not delivery authority.",
    syntheticBoundary: "Outbox candidates are local/synthetic proof only; nothing on the public page sends customer messages or writes provider/PMS state.",
    realAccessValidation: "Real access would validate audit completeness, approval provenance, and outbox handoff rules before any approved system-of-record adapter is considered.",
    inspect: ["docs/architecture/audit-reporting-evidence-backbone.md", "apps/api/openapi/owned-operations-v0.openapi.json", "scripts/demo_owned_operations_api.sh"]
  },
  {
    id: "read-models-bi-replacement",
    label: "read models / BI replacement",
    existsNow: "Read-model/projection proof maps reviewed operating meaning into read_model_projection outputs so BI reads owned facts, caveats, and owner-approved KPI definitions instead of reverse-engineered provider tables.",
    syntheticBoundary: "Current BI replacement proof is sample/query-inventory backed; production reporting claims stay locked until read-only validation and KPI owner review.",
    realAccessValidation: "Existing BI query inventory plus read-only extracts would validate projection coverage, KPI definitions, reconciliation deltas, and analyst cleanup time saved.",
    inspect: ["docs/architecture/audit-reporting-evidence-backbone.md", "storage/src/operations.rs", "docs/demo/local-demo-walkthrough.md"]
  },
  {
    id: "local-synthetic-smoke-proof",
    label: "local / synthetic smoke proof",
    existsNow: "Local smoke proof exercises the checked OpenAPI artifact, Data-Quality Hygiene loop, disabled worker/outbox proof, and prints demo_owned_operations_api_ok with live_side_effects_allowed=false.",
    syntheticBoundary: "Static repo-backed proof is safer for the public page than depending on a private local service; local smoke remains deterministic and fixture-only.",
    realAccessValidation: "A controlled read-only pilot would re-run the same anchors against approved extracts and compare outcomes before enabling any reviewed write adapter.",
    inspect: ["scripts/demo_owned_operations_api.sh", "docs/demo/local-demo-walkthrough.md", "apps/staff-web/smoke/staff-dashboard-mvp.test.mjs"]
  }
];

export const pilotAskItems: PilotAskItem[] = [
  { label: "One pilot slice", detail: "Validate one resort/workflow slice before any write path." },
  { label: "read-only exports", detail: "Source snapshots only; no credentials or mutations implied." },
  { label: "field dictionaries", detail: "Map source fields to NVA-owned operating facts." },
  { label: "BI query inventory", detail: "Compare read models against existing reporting questions." },
  { label: "Source snapshots or sample rows", detail: "Keep freshness and caveats next to every source." },
  { label: "One or two workflows to validate", detail: "Start with manager brief and data-quality hygiene before broader rollout." },
  { label: "Dual-run against current workflow before any write path", detail: "Validate reporting meaning and manager review flow first." }
];

export const roiPilotCloseCards: CloseCard[] = [
  {
    label: "One-location modeled value",
    value: "$25.1k/mo",
    detail: "Illustrative labor + rework exposure from sample arrivals, coverage gaps, document cleanup, and BI reconciliation; not measured production NVA performance."
  },
  {
    label: "Portfolio scale lens",
    value: "170-location scaler",
    detail: "If one validated workflow saves similar minutes across a 170-location owned portfolio, the opportunity compounds; this is a planning scaler, not a guarantee."
  },
  {
    label: "Assumptions visible",
    value: "sample + read-only",
    detail: "48 manager minutes, cleanup rework avoided, and reporting deltas become credible only after approved exports and owner-reviewed definitions."
  }
];

export const safeNextAskItems: CloseListItem[] = [
  { label: "approved read-only exports", detail: "reservation, labor/timeclock, document, capacity, and reporting extracts for one resort slice" },
  { label: "field dictionaries", detail: "source fields, meanings, enums, owner notes, and caveats needed to map NVA-owned facts" },
  { label: "sample rows/docs", detail: "small approved snapshots with source refs, freshness, and exception examples" },
  { label: "BI query inventory", detail: "current questions, source tables/exports, definitions, and reconciliation pain points" },
  { label: "one or two workflows to validate", detail: "manager daily brief and data-quality hygiene are narrow enough for a dual-run" }
];

export const notAskingItems: CloseListItem[] = [
  { label: "live customer sends", detail: "outbox candidate remains locked" },
  { label: "PMS/provider writes", detail: "source records stay read-only" },
  { label: "payment/refund/discount actions", detail: "financial paths stay absent" },
  { label: "schedule changes", detail: "manager review may decide; the product does not change schedules" },
  { label: "medical/safety decisions", detail: "policy and care trade-offs stay human-owned" },
  { label: "staffing mandate action", detail: "staffing choices remain manager-owned" }
];

export const pilotSuccessCriteria: CloseListItem[] = [
  { label: "source mapping confidence", detail: "fields map cleanly enough to create reviewed NVA-owned facts with caveats" },
  { label: "manager action usefulness", detail: "managers keep, edit, or reject recommendations with reasons" },
  { label: "minutes saved / rework avoided", detail: "dual-run captures review time shifted away from source chasing and cleanup loops" },
  { label: "wrong-source findings", detail: "pilot counts source mismatches, stale fields, and unclear docs before automation" },
  { label: "read-model comparison against current BI", detail: "owned read models reconcile with existing BI questions before reporting claims" }
];
