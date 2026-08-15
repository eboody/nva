use serde_json::json;
use storage::operations::{
    ActorKindCode, ApprovalOutboxLineageIds, ApprovalOutboxProjection,
    ApprovalOutboxProjectionInput, ApprovalReviewDisposition, ApprovalTargetBinding,
    InternalHandoff, InternalHandoffTopic, ReviewGateCode, ReviewPacketStatusCode,
    SiteFinanceLocalPersistenceRecords, SiteFinanceOutcomeRecord, StoredSourceRecordRef,
    WorkflowResultStatusCode,
};

#[test]
fn approval_outbox_projection_preserves_shared_review_lifecycle_for_multiple_slices() {
    let data_quality = ApprovalOutboxProjection::from_reviewed_internal_handoff(
        ApprovalOutboxLineageIds::builder()
            .workflow_event_id("00000000-0000-0000-0000-000000000101".to_owned())
            .review_packet_id("00000000-0000-0000-0000-000000000102".to_owned())
            .approval_record_id("00000000-0000-0000-0000-000000000103".to_owned())
            .outbox_record_id("00000000-0000-0000-0000-000000000104".to_owned())
            .subject_kind("location".to_owned())
            .subject_id("00000000-0000-0000-0000-000000000001".to_owned())
            .idempotency_key("dqh:location-1:2026-06-17:context".to_owned())
            .recorded_at("2026-06-17T14:30:00Z".to_owned())
            .build(),
        ApprovalOutboxProjectionInput::builder()
            .workflow_name("data-quality-hygiene".to_owned())
            .event_kind("context_created".to_owned())
            .gate(ReviewGateCode::ManagerApproval)
            .disposition(ApprovalReviewDisposition::approved(
                ActorKindCode::Manager,
                "general-manager-1".to_owned(),
                "2026-06-17T14:30:00Z".to_owned(),
                Some("front desk lead reviewed source-backed cleanup handoff".to_owned()),
                ApprovalTargetBinding::new(
                    "00000000-0000-0000-0000-000000000103".to_owned(),
                    "message".to_owned(),
                    "00000000-0000-0000-0000-000000000001".to_owned(),
                    ReviewGateCode::ManagerApproval,
                ),
            ))
            .target_kind("message".to_owned())
            .agent_actor_id("data-quality-hygiene-agent".to_owned())
            .workflow_payload(json!({
                "correlation_id": "data-quality-hygiene:location-1:2026-06-17",
                "provider_writes_allowed": false,
                "customer_messages_allowed": false
            }))
            .result_payload(json!({
                "reviewable_output_only": true,
                "live_side_effects_allowed": false
            }))
            .audit_action("data_quality_hygiene.reviewed_outcome_recorded".to_owned())
            .audit_metadata(json!({
                "action_id": "dq-action-dq-missing-vaccine-42",
                "live_side_effects_allowed": false
            }))
            .internal_handoff(InternalHandoff::new(
                InternalHandoffTopic::DataQualityHygieneReviewedHandoff,
                json!({
                    "action_id": "dq-action-dq-missing-vaccine-42",
                    "internal_handoff_only": true,
                    "live_delivery_allowed": false
                }),
            ))
            .build(),
    );

    let site_finance = ApprovalOutboxProjection::from_reviewed_internal_handoff(
        ApprovalOutboxLineageIds::builder()
            .workflow_event_id("00000000-0000-0000-0000-000000000201".to_owned())
            .review_packet_id("00000000-0000-0000-0000-000000000202".to_owned())
            .approval_record_id("00000000-0000-0000-0000-000000000203".to_owned())
            .outbox_record_id("00000000-0000-0000-0000-000000000204".to_owned())
            .subject_kind("location".to_owned())
            .subject_id("00c0ffee-0000-0000-0000-000000000001".to_owned())
            .idempotency_key("site-finance:00c0ffee:2026-06:review".to_owned())
            .recorded_at("2026-07-02T12:00:00Z".to_owned())
            .build(),
        ApprovalOutboxProjectionInput::builder()
            .workflow_name("site-finance".to_owned())
            .event_kind("reviewed_recommendation_recorded".to_owned())
            .gate(ReviewGateCode::ManagerApproval)
            .disposition(ApprovalReviewDisposition::approved(
                ActorKindCode::Manager,
                "gm-finance-review".to_owned(),
                "2026-07-02T12:00:00Z".to_owned(),
                Some("manager reviewed finance handoff without payment authority".to_owned()),
                ApprovalTargetBinding::new(
                    "00000000-0000-0000-0000-000000000203".to_owned(),
                    "message".to_owned(),
                    "00c0ffee-0000-0000-0000-000000000001".to_owned(),
                    ReviewGateCode::ManagerApproval,
                ),
            ))
            .target_kind("message".to_owned())
            .agent_actor_id("site-finance-agent".to_owned())
            .workflow_payload(json!({
                "correlation_id": "site-finance:00c0ffee:2026-06",
                "payment_actions_allowed": false,
                "accounting_mutations_allowed": false
            }))
            .result_payload(json!({
                "record_only_finance_action": true,
                "live_side_effects_allowed": false
            }))
            .audit_action("site_finance.reviewed_recommendation_recorded".to_owned())
            .audit_metadata(json!({
                "legal_action": "record_reviewed_recommendation_only",
                "payment_actions_allowed": false
            }))
            .internal_handoff(InternalHandoff::new(
                InternalHandoffTopic::SiteFinanceReviewedHandoff,
                json!({
                    "recommendation_id": "site-finance-review:00c0ffee:2026-06",
                    "internal_handoff_only": true,
                    "live_delivery_allowed": false
                }),
            ))
            .build(),
    );

    for records in [&data_quality, &site_finance] {
        assert_eq!(
            records.workflow_result.status,
            WorkflowResultStatusCode::Succeeded
        );
        assert_eq!(records.review_packet.gate, ReviewGateCode::ManagerApproval);
        assert_eq!(
            records.review_packet.status,
            ReviewPacketStatusCode::Approved
        );
        assert_eq!(records.approval_record.status, "approved");
        assert_eq!(records.audit_events.len(), 2);

        assert!(records.outbox_candidate().is_none());
    }
}

#[test]
fn approval_outbox_projection_keeps_unapproved_or_deferred_work_out_of_outbox() {
    let records = ApprovalOutboxProjection::from_reviewed_internal_handoff(
        ApprovalOutboxLineageIds::builder()
            .workflow_event_id("00000000-0000-0000-0000-000000000301".to_owned())
            .review_packet_id("00000000-0000-0000-0000-000000000302".to_owned())
            .approval_record_id("00000000-0000-0000-0000-000000000303".to_owned())
            .outbox_record_id("00000000-0000-0000-0000-000000000304".to_owned())
            .subject_kind("location".to_owned())
            .subject_id("00000000-0000-0000-0000-000000000001".to_owned())
            .idempotency_key("dqh:location-1:2026-06-17:deferred".to_owned())
            .recorded_at("2026-06-17T14:30:00Z".to_owned())
            .build(),
        ApprovalOutboxProjectionInput::builder()
            .workflow_name("data-quality-hygiene".to_owned())
            .event_kind("context_created".to_owned())
            .gate(ReviewGateCode::ManagerApproval)
            .disposition(ApprovalReviewDisposition::pending())
            .target_kind("message".to_owned())
            .agent_actor_id("data-quality-hygiene-agent".to_owned())
            .workflow_payload(json!({"live_side_effects_allowed": false}))
            .result_payload(json!({"reviewable_output_only": true}))
            .audit_action("data_quality_hygiene.review_pending".to_owned())
            .audit_metadata(json!({"live_side_effects_allowed": false}))
            .internal_handoff(InternalHandoff::new(
                InternalHandoffTopic::DataQualityHygieneReviewedHandoff,
                json!({"live_delivery_allowed": false}),
            ))
            .build(),
    );

    assert_eq!(
        records.workflow_result.status,
        WorkflowResultStatusCode::NeedsReview
    );
    assert_eq!(
        records.review_packet.status,
        ReviewPacketStatusCode::ReadyForReview
    );
    assert_eq!(records.approval_record.status, "approval_requested");
    assert_eq!(records.approval_record.decided_by_actor_kind, None);
    assert_eq!(records.approval_record.decided_by_actor_id, None);
    assert_eq!(records.approval_record.decided_at, None);
    assert!(records.outbox_candidate().is_none());
}

#[test]
fn approval_outbox_projection_records_rejection_evidence_without_approved_rows_or_outbox() {
    let records = ApprovalOutboxProjection::from_reviewed_internal_handoff(
        ApprovalOutboxLineageIds::builder()
            .workflow_event_id("00000000-0000-0000-0000-000000000501".to_owned())
            .review_packet_id("00000000-0000-0000-0000-000000000502".to_owned())
            .approval_record_id("00000000-0000-0000-0000-000000000503".to_owned())
            .outbox_record_id("00000000-0000-0000-0000-000000000504".to_owned())
            .subject_kind("location".to_owned())
            .subject_id("00000000-0000-0000-0000-000000000001".to_owned())
            .idempotency_key("dqh:location-1:2026-06-17:rejected".to_owned())
            .recorded_at("2026-06-17T14:30:00Z".to_owned())
            .build(),
        ApprovalOutboxProjectionInput::builder()
            .workflow_name("data-quality-hygiene".to_owned())
            .event_kind("context_created".to_owned())
            .gate(ReviewGateCode::ManagerApproval)
            .disposition(ApprovalReviewDisposition::rejected(
                ActorKindCode::Manager,
                "general-manager-1".to_owned(),
                "2026-06-17T14:35:00Z".to_owned(),
                Some("source record was stale after staff review".to_owned()),
                ApprovalTargetBinding::new(
                    "00000000-0000-0000-0000-000000000503".to_owned(),
                    "message".to_owned(),
                    "00000000-0000-0000-0000-000000000001".to_owned(),
                    ReviewGateCode::ManagerApproval,
                ),
            ))
            .target_kind("message".to_owned())
            .agent_actor_id("data-quality-hygiene-agent".to_owned())
            .workflow_payload(json!({"live_side_effects_allowed": false}))
            .result_payload(json!({"reviewable_output_only": true}))
            .audit_action("data_quality_hygiene.review_rejected".to_owned())
            .audit_metadata(json!({"live_side_effects_allowed": false}))
            .internal_handoff(InternalHandoff::new(
                InternalHandoffTopic::DataQualityHygieneReviewedHandoff,
                json!({"live_delivery_allowed": false}),
            ))
            .build(),
    );

    assert_eq!(
        records.workflow_result.status,
        WorkflowResultStatusCode::NeedsReview
    );
    assert_eq!(
        records.review_packet.status,
        ReviewPacketStatusCode::Rejected
    );
    assert_eq!(records.approval_record.status, "rejected");
    assert_eq!(
        records.approval_record.decided_by_actor_kind,
        Some(ActorKindCode::Manager)
    );
    assert_eq!(
        records.approval_record.decided_by_actor_id.as_deref(),
        Some("general-manager-1")
    );
    assert_eq!(
        records.approval_record.decided_at.as_deref(),
        Some("2026-06-17T14:35:00Z")
    );
    assert!(records.outbox_candidate().is_none());
}

#[test]
fn site_finance_persistence_records_reuse_approval_outbox_spine_without_payment_authority() {
    let records = SiteFinanceLocalPersistenceRecords::from_reviewed_outcome(
        ApprovalOutboxLineageIds::builder()
            .workflow_event_id("00000000-0000-0000-0000-000000000401".to_owned())
            .review_packet_id("00000000-0000-0000-0000-000000000402".to_owned())
            .approval_record_id("00000000-0000-0000-0000-000000000403".to_owned())
            .outbox_record_id("00000000-0000-0000-0000-000000000404".to_owned())
            .subject_kind("location".to_owned())
            .subject_id("00c0ffee-0000-0000-0000-000000000001".to_owned())
            .idempotency_key("site-finance:00c0ffee:2026-06:review".to_owned())
            .recorded_at("2026-07-02T12:00:00Z".to_owned())
            .build(),
        site_finance_outcome(),
    );

    assert_eq!(records.workflow_event.workflow_name, "site-finance");
    assert_eq!(
        records.workflow_result.result["payment_actions_allowed"],
        serde_json::Value::Null
    );
    assert_eq!(
        records.review_packet.status,
        ReviewPacketStatusCode::Approved
    );
    assert_eq!(records.approval_record.status, "approved");
    assert_eq!(records.outcome.workflow_event_id, records.workflow_event.id);
    assert_eq!(
        records.outcome.approval_record_id,
        records.approval_record.id
    );

    assert!(records.outbox_candidate.is_none());
    assert_eq!(
        records.audit_events[1].metadata["payment_actions_allowed"],
        false
    );
    assert_eq!(
        records.audit_events[1].metadata["accounting_mutations_allowed"],
        false
    );
}

#[test]
fn site_finance_value_claim_evidence_does_not_control_workflow_completion_or_manager_approval() {
    let records = SiteFinanceLocalPersistenceRecords::from_reviewed_outcome(
        ApprovalOutboxLineageIds::builder()
            .workflow_event_id("00000000-0000-0000-0000-000000000501".to_owned())
            .review_packet_id("00000000-0000-0000-0000-000000000502".to_owned())
            .approval_record_id("00000000-0000-0000-0000-000000000503".to_owned())
            .outbox_record_id("00000000-0000-0000-0000-000000000504".to_owned())
            .subject_kind("location".to_owned())
            .subject_id("00c0ffee-0000-0000-0000-000000000001".to_owned())
            .idempotency_key("site-finance:00c0ffee:2026-06:weak-approved-review".to_owned())
            .recorded_at("2026-07-02T12:00:00Z".to_owned())
            .build(),
        site_finance_outcome()
            .with_value_attribution(
                storage::operations::SiteFinanceValueAttribution::CorrelatedOnly,
            )
            .with_workflow_completion(storage::operations::SiteFinanceWorkflowCompletion::Completed)
            .with_manager_approval(
                storage::operations::SiteFinanceManagerApproval::approved_by_manager(
                    "general-manager-1".to_owned(),
                    "2026-07-02T12:00:00Z".to_owned(),
                    None,
                    "00000000-0000-0000-0000-000000000503".to_owned(),
                    "message".to_owned(),
                    "00c0ffee-0000-0000-0000-000000000001".to_owned(),
                ),
            ),
    );

    assert_eq!(
        records.outcome.record.value_attribution,
        storage::operations::SiteFinanceValueAttribution::CorrelatedOnly
    );
    assert_eq!(
        records.workflow_result.status,
        WorkflowResultStatusCode::Succeeded
    );
    assert_eq!(
        records.review_packet.status,
        ReviewPacketStatusCode::Approved
    );
    assert_eq!(records.approval_record.status, "approved");
    assert!(records.outbox_candidate.is_none());
    assert_eq!(
        records.workflow_result.result["value_attribution"],
        "correlated_only"
    );
    assert_eq!(
        records.workflow_result.result["workflow_completion"],
        "completed"
    );
    assert_eq!(
        records.workflow_result.result["manager_approval"],
        "approved"
    );
    assert_eq!(
        records.audit_events[1].metadata["value_attribution"],
        "correlated_only"
    );
}

#[test]
fn site_finance_manager_approval_and_completion_do_not_manufacture_value_claim_authority() {
    let records = SiteFinanceLocalPersistenceRecords::from_reviewed_outcome(
        ApprovalOutboxLineageIds::builder()
            .workflow_event_id("00000000-0000-0000-0000-000000000601".to_owned())
            .review_packet_id("00000000-0000-0000-0000-000000000602".to_owned())
            .approval_record_id("00000000-0000-0000-0000-000000000603".to_owned())
            .outbox_record_id("00000000-0000-0000-0000-000000000604".to_owned())
            .subject_kind("location".to_owned())
            .subject_id("00c0ffee-0000-0000-0000-000000000001".to_owned())
            .idempotency_key("site-finance:00c0ffee:2026-06:strong-pending-review".to_owned())
            .recorded_at("2026-07-02T12:00:00Z".to_owned())
            .build(),
        site_finance_outcome()
            .with_workflow_completion(
                storage::operations::SiteFinanceWorkflowCompletion::NeedsReview,
            )
            .with_manager_approval(storage::operations::SiteFinanceManagerApproval::pending()),
    );

    assert_eq!(
        records.outcome.record.value_attribution,
        storage::operations::SiteFinanceValueAttribution::ReportedReviewedAction
    );
    assert!(!records.outcome.record.can_support_value_claim());
    assert_eq!(
        records.workflow_result.status,
        WorkflowResultStatusCode::NeedsReview
    );
    assert_eq!(
        records.review_packet.status,
        ReviewPacketStatusCode::ReadyForReview
    );
    assert_eq!(records.approval_record.status, "approval_requested");
    assert!(records.approval_record.decided_by_actor_id.is_none());
    assert!(records.outbox_candidate.is_none());
    assert_eq!(
        records.workflow_result.result["can_support_value_claim"],
        false
    );
    assert_eq!(
        records.workflow_result.result["workflow_completion"],
        "needs_review"
    );
    assert_eq!(
        records.workflow_result.result["manager_approval"],
        "pending"
    );
}

fn site_finance_outcome() -> SiteFinanceOutcomeRecord {
    SiteFinanceOutcomeRecord::builder()
        .correlation_id("site-finance:00c0ffee:2026-06".to_owned())
        .location_id("00c0ffee-0000-0000-0000-000000000001".to_owned())
        .period_start("2026-06-01T00:00:00Z".to_owned())
        .period_end("2026-07-01T00:00:00Z".to_owned())
        .service("boarding".to_owned())
        .recommendation_id("site-finance-review:00c0ffee:2026-06".to_owned())
        .review_packet_id("site-finance-review:00c0ffee:2026-06".to_owned())
        .audit_event_id("audit:site-finance-review:00c0ffee:2026-06".to_owned())
        .legal_action("record_reviewed_recommendation_only".to_owned())
        .value_attribution(storage::operations::SiteFinanceValueAttribution::ReportedReviewedAction)
        .workflow_completion(storage::operations::SiteFinanceWorkflowCompletion::Completed)
        .manager_approval(
            storage::operations::SiteFinanceManagerApproval::approved_by_manager(
                "general-manager-1".to_owned(),
                "2026-07-02T12:00:00Z".to_owned(),
                None,
                "00000000-0000-0000-0000-000000000403".to_owned(),
                "message".to_owned(),
                "00c0ffee-0000-0000-0000-000000000001".to_owned(),
            ),
        )
        .currency("usd".to_owned())
        .net_revenue_minor_units(159_000)
        .variance_minor_units(9_000)
        .source_refs(vec![
            StoredSourceRecordRef::builder()
                .system("finance_accounting".to_owned())
                .record_type("site_period_finance".to_owned())
                .record_id("site-finance-boarding-2026-06-current".to_owned())
                .observed_at("2026-07-01T12:00:00Z".to_owned())
                .adapter_version("site-finance-fixture-v1".to_owned())
                .build(),
        ])
        .recorded_at("2026-07-02T12:00:00Z".to_owned())
        .build()
}
