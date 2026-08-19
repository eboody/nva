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

#[test]
fn pending_outbox_admission_rejects_every_mismatched_binding_dimension() {
    type MutatePending = fn(&mut PendingOutboxRecord);
    let cases: [(&str, MutatePending, ApprovalOutboxAuthorityMismatch); 9] = [
        (
            "outbox id",
            |pending| pending.id = "different-outbox-id".to_owned(),
            ApprovalOutboxAuthorityMismatch::ApprovalRelation,
        ),
        (
            "idempotency key",
            |pending| pending.idempotency_key = "different-idempotency-key".to_owned(),
            ApprovalOutboxAuthorityMismatch::ApprovalRelation,
        ),
        (
            "availability timestamp",
            |pending| pending.available_at = "2026-06-18T13:15:00Z".to_owned(),
            ApprovalOutboxAuthorityMismatch::ApprovalRelation,
        ),
        (
            "approval record id",
            |pending| pending.approval_record_id = "different-approval-id".to_owned(),
            ApprovalOutboxAuthorityMismatch::ApprovalRelation,
        ),
        (
            "review gate",
            |pending| pending.review_gate = ReviewGateCode::BehaviorReview,
            ApprovalOutboxAuthorityMismatch::ApprovalRelation,
        ),
        (
            "aggregate kind",
            |pending| pending.aggregate_kind = "different-kind".to_owned(),
            ApprovalOutboxAuthorityMismatch::ApprovalRelation,
        ),
        (
            "aggregate id",
            |pending| pending.aggregate_id = "different-aggregate-id".to_owned(),
            ApprovalOutboxAuthorityMismatch::ApprovalRelation,
        ),
        (
            "handoff topic",
            |pending| pending.topic = InternalHandoffTopic::SiteFinanceReviewedHandoff,
            ApprovalOutboxAuthorityMismatch::InternalHandoff,
        ),
        (
            "handoff payload",
            |pending| pending.payload = json!({"different": true}),
            ApprovalOutboxAuthorityMismatch::InternalHandoff,
        ),
    ];

    for (dimension, mutate, expected_reason) in cases {
        let (mut projection, mut pending) = admitted_pending_outbox();
        mutate(&mut pending);

        let error = projection
            .record_pending_outbox(pending)
            .expect_err(&format!("mismatched {dimension} must be rejected"));
        assert!(
            matches!(
                error,
                Error::ApprovalOutboxAuthority { reason } if reason == expected_reason
            ),
            "mismatched {dimension} returned the wrong rejection: {error:?}"
        );
        assert!(projection.outbox_candidate().is_none());
    }
}

fn admitted_pending_outbox() -> (ApprovalOutboxProjection, PendingOutboxRecord) {
    let mut projection = approved_projection_with_outbox_id("00000000-0000-0000-0000-000000000104");
    projection.approval_record.status = "approved".to_owned();
    projection.approval_record.decided_by_actor_kind = Some(ActorKindCode::Manager);
    projection.approval_record.decided_by_actor_id = Some("general-manager-1".to_owned());
    projection.approval_record.decided_at = Some("2026-06-17T13:15:00Z".to_owned());
    projection.admission_authority_available = true;
    let reviewer = CurrentApprovalReviewerCapability::try_new(
        ActorKindCode::Manager,
        "general-manager-1".to_owned(),
    )
    .expect("manager is review capable");
    let authority = projection
        .authorize_internal_handoff(&reviewer, expected_handoff())
        .expect("matching current reviewer and binding issue one-shot authority");

    (projection, PendingOutboxRecord::admit(authority))
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
