use app::information_lifespan::{
    CalculationProof, CorrelationId, DbProofEntry, FinalArtifact, FinalArtifactKind, HttpMethod,
    LogProofEntry, ModelPath, NetworkProofEntry, ObservedSourceEvidence, SafetyGateKind,
    SafetyGateProof, SourceEvidenceKind, SourceSystem, StageKind, TraceEnvelope,
    TraceSchemaVersion, TraceStage,
};
use serde_json::json;

pub(super) fn mock_gingr_manager_daily_report_trace() -> TraceEnvelope {
    let source_evidence = vec![
        ObservedSourceEvidence::new(
            SourceEvidenceKind::Reservation,
            "fixture://mock-gingr/reservations/9001001.json",
            json!({
                "synthetic": true,
                "id": 9001001,
                "owner_id": 7001,
                "animal_id": 8101,
                "status": "checked_in",
                "service_type": "boarding",
                "start_at": "2026-06-29T13:00:00Z",
                "end_at": "2026-07-02T15:00:00Z",
                "why_received": "manager daily report needs today's in-house boarding demand and source lineage"
            }),
        ),
        ObservedSourceEvidence::new(
            SourceEvidenceKind::CareNote,
            "fixture://mock-gingr/care-notes/9001001-feeding.json",
            json!({
                "synthetic": true,
                "reservation_id": 9001001,
                "animal_id": 8101,
                "note_type": "feeding",
                "body": "Ate breakfast; monitor dinner appetite.",
                "recorded_at": "2026-06-29T14:30:00Z",
                "visibility": "internal_only",
                "why_received": "manager report highlights care exceptions without customer sends"
            }),
        ),
        ObservedSourceEvidence::new(
            SourceEvidenceKind::Vaccine,
            "fixture://mock-gingr/vaccines/8101-rabies.json",
            json!({
                "synthetic": true,
                "animal_id": 8101,
                "vaccine_name": "rabies",
                "expires_on": "2026-07-05",
                "verification_status": "needs_staff_review",
                "why_received": "manager report surfaces near-expiry vaccine work as a review gate, not an automated medical decision"
            }),
        ),
    ];

    TraceEnvelope::builder()
        .schema_version(TraceSchemaVersion::V1)
        .correlation_id(CorrelationId::new("info-lifespan-demo-2026-06-29"))
        .source_system(SourceSystem::MockProviderReadOnlyFixture)
        .synthetic_data_only(true)
        .provider_payloads_are_source_evidence_only(true)
        .live_side_effects_allowed(false)
        .source_evidence(source_evidence)
        .stages(vec![
            TraceStage::new(
                StageKind::SourceEvidenceReceived,
                "Mock Gingr event received",
                false,
                None,
                vec![ModelPath::new(
                    "app::information_lifespan::ObservedSourceEvidence",
                    "synthetic read-only source evidence",
                )],
            ),
            TraceStage::new(
                StageKind::ProviderDtoPreserved,
                "Gingr DTO/source model preserved",
                false,
                None,
                vec![
                    ModelPath::new(
                        "gingr::response::ReservationRecord",
                        "provider reservation evidence",
                    ),
                    ModelPath::new(
                        "gingr::response::ReportCardRecord",
                        "provider-shaped care evidence",
                    ),
                    ModelPath::new(
                        "gingr::response::ImmunizationRecord",
                        "provider-shaped vaccine evidence",
                    ),
                ],
            ),
            TraceStage::new(
                StageKind::NormalizedNvaModels,
                "NVA-owned models normalized",
                false,
                None,
                vec![
                    ModelPath::new("domain::source::RecordRef", "source lineage pointer"),
                    ModelPath::new(
                        "app::manager_daily_brief::SourceFact",
                        "reviewable NVA source fact",
                    ),
                    ModelPath::new(
                        "app::manager_daily_brief::Packet",
                        "manager-owned workflow packet",
                    ),
                ],
            ),
            TraceStage::new(
                StageKind::DatabaseProjectionProof,
                "Database rows/projections prove lineage",
                false,
                None,
                vec![
                    ModelPath::new(
                        "migrations::source_import_runs/source_quality_issues/workflow_events/review_packets/approval_records/audit_events/manager_daily_brief_outcomes",
                        "local Postgres seed rows linked by correlation id",
                    ),
                    ModelPath::new(
                        "migrations::information_lifespan_db_lifecycle_proof",
                        "queryable lifecycle projection for the demo trace",
                    ),
                ],
            ),
            TraceStage::new(
                StageKind::HermesProcessorRun,
                "Hermes processor container enriches trace",
                false,
                None,
                vec![
                    ModelPath::new(
                        "apps::hermes_processor::processor",
                        "Docker Compose service that consumes the synthetic trace and emits report-ready JSON",
                    ),
                    ModelPath::new(
                        "schemas::information_lifespan_hermes_processor_output",
                        "validated processor output contract for API/UI handoff",
                    ),
                ],
            ),
            TraceStage::new(
                StageKind::CalculationApplied,
                "Calculations and ranking applied",
                false,
                None,
                vec![ModelPath::new(
                    "app::manager_daily_brief::LaborImpactEstimate",
                    "labor-value calculation",
                )],
            ),
            TraceStage::new(
                StageKind::ReviewGateLocked,
                "Unsafe side effects locked behind review",
                false,
                None,
                vec![ModelPath::new(
                    "app::manager_daily_brief::BlockedAction",
                    "explicit side-effect lock enum",
                )],
            ),
            TraceStage::new(
                StageKind::ManagerDailyReportArtifact,
                "Manager Daily Report artifact produced",
                false,
                None,
                vec![
                    ModelPath::new(
                        "app::information_lifespan::FinalArtifact",
                        "report artifact contract emitted by the API response",
                    ),
                    ModelPath::new(
                        "apps::api::http::information_lifespan_run_payload",
                        "local API renderer for the final Manager Daily Report artifact",
                    ),
                ],
            ),
        ])
        .log_proof_entries(vec![
            LogProofEntry::new(
                "INFO",
                "information_lifespan",
                "mock Gingr source evidence accepted from fixture",
            ),
            LogProofEntry::new(
                "INFO",
                "information_lifespan",
                "provider payload retained as source evidence only",
            ),
            LogProofEntry::new(
                "WARN",
                "information_lifespan.safety",
                "provider writes/customer sends/payment movement locked",
            ),
        ])
        .db_proof_entries(vec![
            DbProofEntry::new(
                "source_import_runs",
                "source_import_run:info-lifespan-demo-2026-06-29;correlation_id:info-lifespan-demo-2026-06-29",
                "migrations::source_import_runs",
            ),
            DbProofEntry::new(
                "source_quality_issues",
                "source_quality_issue:vaccine-near-expiry:8101;correlation_id:info-lifespan-demo-2026-06-29",
                "migrations::source_quality_issues",
            ),
            DbProofEntry::new(
                "workflow_events",
                "workflow_event:manager-daily-report:2026-06-29;correlation_id:info-lifespan-demo-2026-06-29",
                "app::manager_daily_brief::Request",
            ),
            DbProofEntry::new(
                "review_packets",
                "review_packet:vaccine-near-expiry:8101;correlation_id:info-lifespan-demo-2026-06-29",
                "app::manager_daily_brief::BriefAction",
            ),
            DbProofEntry::new(
                "manager_daily_brief_outcomes",
                "manager_daily_brief_outcome:synthetic-2026-06-29;correlation_id:info-lifespan-demo-2026-06-29",
                "storage::operations::ManagerDailyBriefOutcomeRecord",
            ),
            DbProofEntry::new(
                "information_lifespan_db_lifecycle_proof",
                "correlation_id:info-lifespan-demo-2026-06-29",
                "migrations::information_lifespan_db_lifecycle_proof",
            ),
        ])
        .network_proof_entries(vec![
            NetworkProofEntry::new(
                HttpMethod::Post,
                "/v1/demo/information-lifespan/run",
                200,
                "response://information-lifespan/info-lifespan-demo-2026-06-29",
            ),
            NetworkProofEntry::new(
                HttpMethod::Get,
                "/v1/demo/information-lifespan/info-lifespan-demo-2026-06-29/report",
                200,
                "response://manager-daily-report/synthetic-2026-06-29",
            ),
        ])
        .calculations(vec![
            CalculationProof::new("source_snapshots", "reservation + care_note + vaccine", "3"),
            CalculationProof::new(
                "normalized_facts",
                "reservation demand + care exception + vaccine review",
                "3",
            ),
            CalculationProof::new(
                "reported_estimated_labor_minutes_difference",
                "60 minute caller-reported manual baseline - 18 minute caller-reported workflow estimate",
                "42",
            ),
        ])
        .safety_gates(vec![
            SafetyGateProof::new(
                SafetyGateKind::ProviderWriteLocked,
                true,
                "demo is read-only and never mutates Gingr/PMS records",
            ),
            SafetyGateProof::new(
                SafetyGateKind::CustomerSendLocked,
                true,
                "manager report can draft internal tasks only; no live sends",
            ),
            SafetyGateProof::new(
                SafetyGateKind::MedicalReviewRequired,
                true,
                "vaccine fact is surfaced for staff review, not auto-accepted",
            ),
            SafetyGateProof::new(
                SafetyGateKind::ScheduleChangeLocked,
                true,
                "staffing/demand recommendation cannot change schedules",
            ),
            SafetyGateProof::new(
                SafetyGateKind::PaymentMovementLocked,
                true,
                "payments, refunds, and discounts are outside this demo authority",
            ),
        ])
        .final_artifact(FinalArtifact::new(
            FinalArtifactKind::ManagerDailyReport,
            "Manager Daily Report — synthetic 2026-06-29",
            "artifact://manager-daily-report/synthetic-2026-06-29",
            "3 source snapshots, 3 normalized facts, 6 DB proof refs, 5 review locks, 42 reported estimated labor minute difference",
        ))
        .build()
}
