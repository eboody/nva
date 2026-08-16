use super::*;
use serde_json::json;

#[test]
fn caller_reported_approval_cannot_issue_projection_bound_authority() {
    let mut projection = approved_projection_with_outbox_id("00000000-0000-0000-0000-000000000104");
    let reviewer = CurrentApprovalReviewerCapability::try_new(
        ActorKindCode::Manager,
        "general-manager-1".to_owned(),
    )
    .expect("manager is review capable");

    assert!(
        projection
            .authorize_internal_handoff(&reviewer, expected_handoff())
            .is_err()
    );
    assert!(projection.outbox_candidate().is_none());
    assert_eq!(
        projection.workflow_result.status,
        WorkflowResultStatusCode::NeedsReview
    );
    assert_eq!(
        projection.review_packet.status,
        ReviewPacketStatusCode::ReadyForReview
    );
    assert_eq!(projection.approval_record.status, "approval_requested");
}

#[test]
fn internal_handoff_topics_are_closed_and_non_live() {
    assert_eq!(
        InternalHandoffTopic::DataQualityHygieneReviewedHandoff.as_str(),
        "internal.data_quality_hygiene.reviewed_handoff"
    );
    assert_eq!(
        InternalHandoffTopic::SiteFinanceReviewedHandoff.as_str(),
        "internal.site_finance.reviewed_handoff"
    );
}

fn expected_handoff() -> InternalHandoff {
    InternalHandoff::new(
        InternalHandoffTopic::DataQualityHygieneReviewedHandoff,
        json!({
            "action_id": "dq-action-1",
            "correlation_id": "dqh-correlation-1",
            "issue_refs": ["dq-issue-1"],
            "source_refs": [],
            "internal_handoff_only": true,
            "live_delivery_allowed": false
        }),
    )
}

fn approved_projection_with_outbox_id(outbox_record_id: &str) -> ApprovalOutboxProjection {
    let approval_record_id = "00000000-0000-0000-0000-000000000103".to_owned();
    let subject_id = "00000000-0000-0000-0000-000000000001".to_owned();
    ApprovalOutboxProjection::from_reviewed_internal_handoff(
        ApprovalOutboxLineageIds::builder()
            .workflow_event_id("00000000-0000-0000-0000-000000000101".to_owned())
            .review_packet_id("00000000-0000-0000-0000-000000000102".to_owned())
            .approval_record_id(approval_record_id.clone())
            .outbox_record_id(outbox_record_id.to_owned())
            .subject_kind("location".to_owned())
            .subject_id(subject_id.clone())
            .idempotency_key("dqh:location-1:2026-06-17:context".to_owned())
            .recorded_at("2026-06-17T13:15:00Z".to_owned())
            .build(),
        ApprovalOutboxProjectionInput::builder()
            .workflow_name("data-quality-hygiene".to_owned())
            .event_kind("context_created".to_owned())
            .gate(ReviewGateCode::ManagerApproval)
            .disposition(ApprovalReviewDisposition::approved(
                ActorKindCode::Manager,
                "general-manager-1".to_owned(),
                "2026-06-17T13:15:00Z".to_owned(),
                None,
                ApprovalTargetBinding::new(
                    approval_record_id,
                    "message".to_owned(),
                    subject_id,
                    ReviewGateCode::ManagerApproval,
                ),
            ))
            .target_kind("message".to_owned())
            .agent_actor_id("data-quality-hygiene-agent".to_owned())
            .workflow_payload(json!({"live_side_effects_allowed": false}))
            .result_payload(json!({"reviewable_output_only": true}))
            .audit_action("data_quality_hygiene.reported_outcome_recorded".to_owned())
            .audit_metadata(json!({"live_side_effects_allowed": false}))
            .internal_handoff(expected_handoff())
            .build(),
    )
}
