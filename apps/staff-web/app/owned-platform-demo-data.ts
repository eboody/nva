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

export type SourceEventCard = {
  id: string;
  eventLabel: string;
  receivedAt: string;
  boundaryTags: string[];
  eventSummary: string;
  sourceRef: string;
  providerModelPath: string;
  nvaTargetModelPath: string;
  traceModelPath: string;
  payloadPreview: Record<string, string | number | boolean>;
  initiallyOpen: boolean;
};

export type DbProjectionProofArtifact = {
  id: string;
  order: string;
  artifactName: string;
  artifactKind: "table" | "view";
  proofPosture: "actual local Postgres proof" | "deterministic trace proof";
  rowCount: number;
  correlationId: string;
  stageLink: string;
  summary: string;
  rowPreview: Record<string, string | number | boolean | string[]>;
  inspect: string[];
};

export type HermesProcessorStage = {
  id: string;
  order: string;
  label: string;
  state: "queued" | "running" | "done" | "error";
  headline: string;
  detail: string;
  evidence: string;
};

export type HermesProcessorLogLine = {
  timestamp: string;
  level: "INFO" | "WARN" | "ERROR";
  target: string;
  event: string;
  correlationId: string;
  summary: string;
};

export type HermesProcessorPanel = {
  serviceLabel: string;
  containerName: string;
  correlationId: string;
  status: "done" | "running" | "unavailable";
  runtimeStatus: string;
  mode: string;
  boundary: string;
  unavailableDegradation: string;
  inputSummary: string[];
  stages: HermesProcessorStage[];
  logLines: HermesProcessorLogLine[];
  reportFragment: {
    title: string;
    artifactRef: string;
    summary: string;
    calculation: string;
    managerActions: string[];
  };
  inspect: string[];
};

export type NetworkRequestProof = {
  id: string;
  method: "GET" | "POST";
  browserPath: string;
  upstreamPath: string;
  expectedStatus: number;
  proofPurpose: string;
  responsePreview: Record<string, string | number | boolean>;
};

export type ManagerReportAction = {
  rank: number;
  title: string;
  owner: string;
  urgency: string;
  recommendation: string;
  sourceLineage: string[];
  calculations: string[];
  reportedEstimatedMinutesDifference: number;
  reviewRequirements: string[];
  lockedSideEffects: string[];
};

export type ManagerDailyReportArtifact = {
  title: string;
  artifactRef: string;
  correlationId: string;
  generatedBy: string;
  summary: string;
  valueProof: {
    sourceSnapshots: number;
    normalizedFacts: number;
    dbProofRefs: number;
    reviewLocks: number;
    reportedEstimatedMinutesDifference: number;
  };
  rankedActions: ManagerReportAction[];
  sourceLineageSummary: string[];
  calculationProof: string[];
  reviewGates: string[];
  lockedSideEffects: string[];
};

export type InformationLifespanStage = {
  id: string;
  order: string;
  label: string;
  headline: string;
  proofKind: "source" | "model" | "db" | "processor" | "calculation" | "network" | "report";
  proofSummary: string;
  inspect: string[];
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

export const informationLifespanStages: InformationLifespanStage[] = [
  {
    id: "mock-source-received",
    order: "01",
    label: "Mock Gingr event received",
    headline: "Synthetic read-only source evidence enters the trace.",
    proofKind: "source",
    proofSummary: "Mocked Gingr reservation, care note, and vaccine evidence are accepted as fixture evidence only.",
    inspect: ["fixtures/information-lifespan/hermes-processor-input.json", "integrations/gingr/src/response.rs"]
  },
  {
    id: "provider-dto-model",
    order: "02",
    label: "Provider DTO / source model shown",
    headline: "Provider-shaped DTOs are visible before NVA owns meaning.",
    proofKind: "model",
    proofSummary: "gingr::response shapes map toward app::information_lifespan::SourcePayload without claiming provider authority.",
    inspect: ["integrations/gingr/src/dto/README.md", "integrations/gingr/src/mapping/"]
  },
  {
    id: "snapshot-provenance-stored",
    order: "03",
    label: "Source snapshot + provenance stored",
    headline: "Correlation id ties source snapshots to import/run evidence.",
    proofKind: "db",
    proofSummary: "source_import_runs and workflow_events rows preserve provenance for the same correlation id.",
    inspect: ["migrations/0001_mvp_foundation.sql", "fixtures/seed/local-demo.sql"]
  },
  {
    id: "nva-models-normalized",
    order: "04",
    label: "NVA-owned models normalized",
    headline: "Source evidence becomes NVA-owned workflow facts.",
    proofKind: "model",
    proofSummary: "Domain/app models carry reservation, care-note, and vaccine-review meaning into manager brief packets.",
    inspect: ["app/src/information_lifespan.rs", "app/src/manager_daily_brief.rs"]
  },
  {
    id: "db-projections-read",
    order: "05",
    label: "Database rows/projections written/read",
    headline: "Local DB rows and projection proof light up in sequence.",
    proofKind: "db",
    proofSummary: "information_lifespan_db_lifecycle_proof joins import, workflow, issue, outcome, and audit rows.",
    inspect: ["migrations/0002_data_quality_read_models.sql", "fixtures/seed/local-demo-data-quality.sql"]
  },
  {
    id: "hermes-processor-runs",
    order: "06",
    label: "Hermes processor container runs",
    headline: "Dockerized Hermes bridge enriches the report artifact.",
    proofKind: "processor",
    proofSummary: "The local processor writes structured JSON/JSONL output or labels unavailable fallback honestly.",
    inspect: ["apps/hermes-processor/processor.py", ".var/information-lifespan/processor-output.json"]
  },
  {
    id: "calculations-review-gates",
    order: "07",
    label: "Calculations/ranking/review gates applied",
    headline: "Labor value and unsafe-action locks are computed into the report.",
    proofKind: "calculation",
    proofSummary: "60 minute manual morning scan - 18 minute reviewed packet = 42 reported estimated minute difference; five review locks remain closed.",
    inspect: ["app/src/manager_daily_brief.rs", "app/tests/information_lifespan_trace_contract.rs"]
  },
  {
    id: "api-network-response",
    order: "08",
    label: "API/network response returned",
    headline: "Browser-visible POST/GET calls prove the local API path.",
    proofKind: "network",
    proofSummary: "POST /run returns the trace, then GET /:correlation_id/report replays the same artifact.",
    inspect: ["apps/api/src/http.rs", "apps/staff-web/app/api/local-demo/[...path]/route.ts"]
  },
  {
    id: "manager-report-appears",
    order: "09",
    label: "Manager Daily Report appears",
    headline: "The final manager packet shows lineage, calculations, locks, and labor-value proof.",
    proofKind: "report",
    proofSummary: "artifact://manager-daily-report/synthetic-2026-06-29 stays tied to the active correlation id.",
    inspect: ["app/src/manager_daily_brief.rs", "apps/staff-web/app/page.tsx"]
  }
];

export const sourceEventCards: SourceEventCard[] = [
  {
    id: "mock-gingr-reservation-received",
    eventLabel: "Mock Gingr event received",
    receivedAt: "2026-06-29 13:00Z",
    boundaryTags: ["mocked Gingr", "read-only evidence", "not product truth"],
    eventSummary: "System receives a synthetic boarding reservation payload for the Manager Daily Report trace.",
    sourceRef: "fixture://mock-gingr/reservations/9001001.json",
    providerModelPath: "gingr::response::ReservationRecord",
    nvaTargetModelPath: "domain::reservation::StayFact",
    traceModelPath: "app::information_lifespan::SourcePayload",
    payloadPreview: {
      synthetic: true,
      id: 9001001,
      animal_id: 8101,
      status: "checked_in",
      service_type: "boarding",
      why_received: "manager daily report needs today's in-house boarding demand and source lineage"
    },
    initiallyOpen: true
  },
  {
    id: "mock-gingr-care-note-received",
    eventLabel: "Mock Gingr care note received",
    receivedAt: "2026-06-29 14:30Z",
    boundaryTags: ["mocked Gingr", "read-only evidence", "not product truth"],
    eventSummary: "Internal-only care note enters the same source-evidence lane before any customer send is possible.",
    sourceRef: "fixture://mock-gingr/care-notes/9001001-feeding.json",
    providerModelPath: "gingr::response::provider::Payload",
    nvaTargetModelPath: "domain::care::CareNoteFact",
    traceModelPath: "app::information_lifespan::SourcePayload",
    payloadPreview: {
      synthetic: true,
      reservation_id: 9001001,
      animal_id: 8101,
      note_type: "feeding",
      visibility: "internal_only"
    },
    initiallyOpen: false
  },
  {
    id: "mock-gingr-vaccine-received",
    eventLabel: "Mock Gingr vaccine review received",
    receivedAt: "2026-06-29 14:35Z",
    boundaryTags: ["mocked Gingr", "read-only evidence", "not product truth"],
    eventSummary: "Near-expiry vaccine evidence becomes a staff review gate, not an automated medical decision.",
    sourceRef: "fixture://mock-gingr/vaccines/8101-rabies.json",
    providerModelPath: "gingr::response::provider::Payload",
    nvaTargetModelPath: "domain::vaccine::ReviewFact",
    traceModelPath: "app::information_lifespan::SourcePayload",
    payloadPreview: {
      synthetic: true,
      animal_id: 8101,
      vaccine_name: "rabies",
      expires_on: "2026-07-05",
      verification_status: "needs_staff_review"
    },
    initiallyOpen: false
  }
];

export const dbProjectionLifecycleProofs: DbProjectionProofArtifact[] = [
  {
    id: "source-import-run-row",
    order: "01",
    artifactName: "source_import_runs",
    artifactKind: "table",
    proofPosture: "actual local Postgres proof",
    rowCount: 1,
    correlationId: "info-lifespan-demo-2026-06-29",
    stageLink: "lights up after source/model stages",
    summary: "Synthetic mocked Gingr import run persisted locally before NVA workflow/report projections read it.",
    rowPreview: {
      id: "00000000-0000-4000-8000-00000000a801",
      source_system: "mock_gingr_readonly_fixture",
      mode: "read_only_snapshot",
      status: "completed",
      record_count: 3,
      rejected_count: 0,
      redaction_posture: "raw_payloads_redacted_or_referenced"
    },
    inspect: ["migrations/0002_data_quality_read_models.sql", "fixtures/seed/local-demo.sql"]
  },
  {
    id: "workflow-event-row",
    order: "02",
    artifactName: "workflow_events",
    artifactKind: "table",
    proofPosture: "actual local Postgres proof",
    rowCount: 1,
    correlationId: "info-lifespan-demo-2026-06-29",
    stageLink: "lights up after source/model stages",
    summary: "NVA-owned workflow event carries the correlation_id and source_import_run_id that tie source evidence to report work.",
    rowPreview: {
      workflow_name: "information_lifespan_manager_daily_report",
      event_kind: "manager_daily_report.trace_replayed",
      subject_kind: "location",
      correlation_id: "info-lifespan-demo-2026-06-29",
      source_import_run_id: "00000000-0000-4000-8000-00000000a801",
      live_side_effects_disabled: true
    },
    inspect: ["migrations/0001_mvp_foundation.sql", "fixtures/seed/local-demo.sql"]
  },
  {
    id: "source-quality-issue-row",
    order: "03",
    artifactName: "source_quality_issues",
    artifactKind: "table",
    proofPosture: "actual local Postgres proof",
    rowCount: 1,
    correlationId: "info-lifespan-demo-2026-06-29",
    stageLink: "lights up after source/model stages",
    summary: "A near-expiry vaccine source ambiguity becomes a review issue instead of an automated medical/safety decision.",
    rowPreview: {
      issue_ref: "source_quality_issue:vaccine-near-expiry:8101",
      affected_entity_kind: "pet",
      field_path: "vaccine.rabies.expires_on",
      severity: "medium",
      sensitivity: "medical_or_vaccination",
      review_gate: "manager_approval",
      raw_payload: "redacted/reference-only"
    },
    inspect: ["migrations/0002_data_quality_read_models.sql", "fixtures/seed/local-demo.sql"]
  },
  {
    id: "manager-brief-outcome-row",
    order: "04",
    artifactName: "manager_daily_brief_outcomes",
    artifactKind: "table",
    proofPosture: "actual local Postgres proof",
    rowCount: 1,
    correlationId: "info-lifespan-demo-2026-06-29",
    stageLink: "lights up after source/model stages",
    summary: "Manager Daily Report outcome records labor-value proof while side effects stay unavailable.",
    rowPreview: {
      id: "00000000-0000-4000-8000-00000000ae01",
      action_id: "manager_daily_brief_outcome:synthetic-2026-06-29",
      reported_estimated_minutes_difference: 42,
      source_snapshot_count: 3,
      review_gate_count: 5,
      live_side_effects_disabled: true
    },
    inspect: ["app/src/manager_daily_brief.rs", "fixtures/seed/local-demo.sql"]
  },
  {
    id: "audit-events-readback",
    order: "05",
    artifactName: "audit_events",
    artifactKind: "table",
    proofPosture: "actual local Postgres proof",
    rowCount: 2,
    correlationId: "info-lifespan-demo-2026-06-29",
    stageLink: "lights up after source/model stages",
    summary: "Two local audit rows prove the trace was reviewed and side-effect locks stayed visible.",
    rowPreview: {
      actions: ["information_lifespan.db_projection_rows_written", "information_lifespan.manager_daily_report_review_required"],
      actor_kinds: ["agent", "manager"],
      subject_kind: "workflow_event",
      live_side_effects_disabled: true,
      row_count: 2
    },
    inspect: ["migrations/0001_mvp_foundation.sql", "fixtures/seed/local-demo.sql"]
  },
  {
    id: "lifecycle-proof-view",
    order: "06",
    artifactName: "information_lifespan_db_lifecycle_proof",
    artifactKind: "view",
    proofPosture: "deterministic trace proof",
    rowCount: 1,
    correlationId: "info-lifespan-demo-2026-06-29",
    stageLink: "lights up after source/model stages",
    summary: "Projection joins import, workflow, issue, review, outcome, and audit rows into one engineering proof view.",
    rowPreview: {
      projection_version: "information_lifespan_db_lifecycle_proof.v1",
      correlation_id: "info-lifespan-demo-2026-06-29",
      source_system: "mock_gingr_readonly_fixture",
      source_quality_issue_refs: ["source_quality_issue:vaccine-near-expiry:8101"],
      audit_event_count: 2,
      caveats: ["synthetic_local_demo_only", "live_side_effects_disabled", "raw_payloads_redacted_or_referenced"]
    },
    inspect: ["migrations/0002_data_quality_read_models.sql", "scripts/migrate-and-seed-local.sh"]
  }
];

export const informationLifespanNetworkRequests: NetworkRequestProof[] = [
  {
    id: "run-report-post",
    method: "POST",
    browserPath: "/api/local-demo/v0/demo/information-lifespan/run",
    upstreamPath: "/v0/demo/information-lifespan/run",
    expectedStatus: 200,
    proofPurpose: "Starts the deterministic trace replay and returns the Manager Daily Report artifact payload.",
    responsePreview: {
      correlation_id: "info-lifespan-demo-2026-06-29",
      stage_count: 8,
      artifact_ref: "artifact://manager-daily-report/synthetic-2026-06-29",
      reported_estimated_labor_minutes_difference: 42,
      live_side_effects_allowed: false,
      synthetic_data_only: true
    }
  },
  {
    id: "report-replay-get",
    method: "GET",
    browserPath: "/api/local-demo/v0/demo/information-lifespan/info-lifespan-demo-2026-06-29/report",
    upstreamPath: "/v0/demo/information-lifespan/info-lifespan-demo-2026-06-29/report",
    expectedStatus: 200,
    proofPurpose: "Replays the same report by correlation id so the final artifact is addressable, not just fixture copy.",
    responsePreview: {
      workflow: "information_lifespan_demo_report",
      correlation_id: "info-lifespan-demo-2026-06-29",
      artifact_ref: "artifact://manager-daily-report/synthetic-2026-06-29",
      locked_side_effect_count: 5,
      response_contains_lineage: true,
      live_side_effects_allowed: false
    }
  }
];

export const managerDailyReportArtifact: ManagerDailyReportArtifact = {
  title: "Manager Daily Report — synthetic 2026-06-29",
  artifactRef: "artifact://manager-daily-report/synthetic-2026-06-29",
  correlationId: "info-lifespan-demo-2026-06-29",
  generatedBy: "POST /v0/demo/information-lifespan/run → GET report replay",
  summary: "A manager-ready morning packet ranked by source-backed urgency; every action keeps the source, calculation, review gate, and locked side effect visible.",
  valueProof: {
    sourceSnapshots: 3,
    normalizedFacts: 3,
    dbProofRefs: 6,
    reviewLocks: 5,
    reportedEstimatedMinutesDifference: 42
  },
  rankedActions: [
    {
      rank: 1,
      title: "Review near-expiry rabies evidence before check-in exception work",
      owner: "Manager + trained staff reviewer",
      urgency: "High — medical/vaccine ambiguity affects today's boarding confidence",
      recommendation: "Open the review packet, inspect the mocked source refs, and record a human disposition before any eligibility or customer-facing claim.",
      sourceLineage: [
        "fixture://mock-gingr/vaccines/8101-rabies.json",
        "source_quality_issue:vaccine-near-expiry:8101",
        "review_packet:vaccine-near-expiry:8101"
      ],
      calculations: [
        "1 near-expiry vaccine issue",
        "manager_approval gate required",
        "medical/vaccine acceptance remains locked"
      ],
      reportedEstimatedMinutesDifference: 18,
      reviewRequirements: ["manager_approval", "medical_document_review", "record reviewed disposition"],
      lockedSideEffects: ["medical_or_vaccine_acceptance", "customer_sends", "provider_pms_writes"]
    },
    {
      rank: 2,
      title: "Compare boarding arrival density against morning coverage",
      owner: "General manager",
      urgency: "Medium — demand/staffing scan is useful but cannot change schedules",
      recommendation: "Use the ranked packet as prep for the manager huddle; any staffing response stays a manager decision outside the agent.",
      sourceLineage: [
        "fixture://mock-gingr/reservations/9001001.json",
        "workflow_event:manager-daily-report:2026-06-29",
        "manager_daily_brief_outcome:synthetic-2026-06-29"
      ],
      calculations: [
        "60 minute manual morning scan",
        "18 minute reviewed packet",
        "42 total reported estimated minute difference across the report"
      ],
      reportedEstimatedMinutesDifference: 16,
      reviewRequirements: ["manager_shift_review", "audit reviewed disposition"],
      lockedSideEffects: ["schedule_or_staffing_changes", "provider_pms_writes", "payments_refunds_discounts"]
    },
    {
      rank: 3,
      title: "Check internal feeding note before drafting any customer update",
      owner: "Front desk lead",
      urgency: "Normal — internal care context should not leak into a send path",
      recommendation: "Review the internal-only note in the manager packet; keep any customer-facing update locked until approved content and channel rules exist.",
      sourceLineage: [
        "fixture://mock-gingr/care-notes/9001001-feeding.json",
        "domain::care::CareNoteFact",
        "app::manager_daily_brief::SourceFact"
      ],
      calculations: ["1 internal-only care note", "customer-visible draft suppressed", "8 reported estimated review-minute difference"],
      reportedEstimatedMinutesDifference: 8,
      reviewRequirements: ["front_desk_or_manager_review", "customer_message_approval before any send"],
      lockedSideEffects: ["customer_sends", "medical_or_vaccine_acceptance", "provider_pms_writes"]
    }
  ],
  sourceLineageSummary: [
    "Mock Gingr/source payloads are read-only fixture evidence, not product truth.",
    "NVA-owned facts and workflow packets carry source refs into DB proof and report actions.",
    "The local API response repeats correlation id, network proof, log proof, calculations, and safety locks."
  ],
  calculationProof: [
    "source_snapshots = reservation + care_note + vaccine = 3",
    "normalized_facts = reservation demand + care exception + vaccine review = 3",
    "reported_estimated_labor_minutes_difference = 60 minute manual morning scan - 18 minute reviewed packet = 42"
  ],
  reviewGates: ["provider_write_locked", "customer_send_locked", "medical_review_required", "schedule_change_locked", "payment_movement_locked"],
  lockedSideEffects: ["provider_pms_writes", "customer_sends", "medical_or_vaccine_acceptance", "schedule_or_staffing_changes", "payments_refunds_discounts"]
};

export const hermesProcessorPanel: HermesProcessorPanel = {
  serviceLabel: "Hermes processor container",
  containerName: "docker compose service hermes-processor",
  correlationId: "info-lifespan-demo-2026-06-29",
  status: "done",
  runtimeStatus: "hermes_cli_unavailable_explicit_fallback",
  mode: "deterministic_local_bridge",
  boundary: "containerized local processor · synthetic input only · no live writes · no real credentials required",
  unavailableDegradation: "If the processor container or artifact is unavailable, this panel keeps the last safe input summary visible and shows an unavailable state instead of inventing output.",
  inputSummary: [
    "3 synthetic mock Gingr source payloads accepted as provider evidence only",
    "DB projection proof read from information_lifespan_db_lifecycle_proof",
    "calculation input: 60 minute manual morning scan - 18 minute reviewed packet",
    "live_side_effects_allowed=false; provider writes, sends, schedule/payment/medical paths locked"
  ],
  stages: [
    {
      id: "input-accepted",
      order: "01",
      label: "input accepted",
      state: "done",
      headline: "synthetic trace mounted into container",
      detail: "fixtures/information-lifespan/hermes-processor-input.json carries source refs, DB projection ids, calculations, and safety gates.",
      evidence: "accepted_synthetic_trace"
    },
    {
      id: "runtime-checked",
      order: "02",
      label: "runtime posture checked",
      state: "done",
      headline: "Hermes runtime posture explicit",
      detail: "The local Docker bridge reports hermes_cli_unavailable_explicit_fallback and can fail hard when require-runtime mode is enabled.",
      evidence: "runtime_posture_checked"
    },
    {
      id: "locks-applied",
      order: "03",
      label: "review locks applied",
      state: "running",
      headline: "unsafe side effects remain locked",
      detail: "Provider/PMS writes, customer sends, schedule changes, payment movement, and medical/vaccine decisions stay review-required.",
      evidence: "unsafe_side_effects_locked"
    },
    {
      id: "report-enriched",
      order: "04",
      label: "report fragment generated",
      state: "done",
      headline: "Manager Daily Report enriched",
      detail: "The processor writes .var/information-lifespan/processor-output.json and structured JSONL logs with the same correlation id.",
      evidence: "manager_daily_report_enriched"
    },
    {
      id: "unavailable-fallback",
      order: "fallback",
      label: "unavailable state",
      state: "error",
      headline: "processor unavailable is visible",
      detail: "The UI degrades to this redacted status instead of claiming a live run when no processor output is present.",
      evidence: "hermes_processor_unavailable"
    }
  ],
  logLines: [
    {
      timestamp: "2026-06-29T15:00:00Z",
      level: "INFO",
      target: "hermes_processor.information_lifespan",
      event: "accepted_synthetic_trace",
      correlationId: "info-lifespan-demo-2026-06-29",
      summary: "synthetic_data_only=true"
    },
    {
      timestamp: "2026-06-29T15:00:00Z",
      level: "INFO",
      target: "hermes_processor.runtime",
      event: "runtime_posture_checked",
      correlationId: "info-lifespan-demo-2026-06-29",
      summary: "mode=deterministic_local_bridge runtime_status=hermes_cli_unavailable_explicit_fallback"
    },
    {
      timestamp: "2026-06-29T15:00:00Z",
      level: "WARN",
      target: "hermes_processor.safety",
      event: "unsafe_side_effects_locked",
      correlationId: "info-lifespan-demo-2026-06-29",
      summary: "provider/PMS writes, customer sends, schedule changes, payments/refunds/discounts, and medical/vaccine acceptance decisions locked"
    },
    {
      timestamp: "2026-06-29T15:00:00Z",
      level: "INFO",
      target: "hermes_processor.report",
      event: "manager_daily_report_enriched",
      correlationId: "info-lifespan-demo-2026-06-29",
      summary: "artifact://manager-daily-report/synthetic-2026-06-29 reported_estimated_labor_minutes_difference=42"
    }
  ],
  reportFragment: {
    title: "Manager Daily Report — synthetic 2026-06-29",
    artifactRef: "artifact://manager-daily-report/synthetic-2026-06-29",
    summary: "3 source snapshots, 3 normalized facts, 1 workflow packet, 5 review locks, 42 reported estimated labor minute difference",
    calculation: "60 minute manual morning scan - 18 minute reviewed packet = 42 reported estimated minute difference",
    managerActions: [
      "Review near-expiry rabies vaccine evidence for animal 8101",
      "Check dinner appetite after internal feeding note before any customer-facing update"
    ]
  },
  inspect: [
    "docker-compose.yml",
    "apps/hermes-processor/processor.py",
    ".var/information-lifespan/processor-output.json",
    ".var/information-lifespan/processor-log.jsonl",
    "schemas/information-lifespan-hermes-processor-output.schema.json"
  ]
};

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
    outcomeMetric: "reported cleanup-time difference, front-desk rework avoided, source-quality backlog aging, reviewed-resolution rate",
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
    outcomeMetric: "reported review-time difference per intake; avoidable back-and-forth reduced; intake_queue readout remains sample/modelled",
    proofHooks: ["workflow_packet_id=intake_booking_triage", "outbox_candidate_id=triage_reply_candidate", "review_gate_id=front_desk_or_manager", "customer_send_locked=true"],
    lineageSteps: ["intake message: two dogs for holiday boarding", "missing-info checklist", "safe draft reply", "send locked", "intake_queue readout"],
    lineageId: "lineage-intake-booking-triage"
  },
  {
    id: "bi-read-model",
    name: "BI / Read Model Reporting",
    summary: "Gives portfolio reporting NVA-owned meaning instead of reverse-engineering provider tables.",
    sourceSignals: ["current BI query inventory", "workflow outcome events", "review dispositions", "labor/rework metrics"],
    normalizedFacts: ["reviewed business meaning", "reported labor-time difference", "workflow aging", "projection freshness"],
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
  { label: "reported time difference / rework evidence", detail: "dual-run captures review time shifted away from source chasing and cleanup loops" },
  { label: "wrong-source findings", detail: "pilot counts source mismatches, stale fields, and unclear docs before automation" },
  { label: "read-model comparison against current BI", detail: "owned read models reconcile with existing BI questions before reporting claims" }
];
