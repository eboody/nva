use storage::{
    operations::{ActorKindCode, ApprovalReviewDisposition, ApprovalTargetBinding, ReviewGateCode},
    workflow_repository::PostgresSourceQualityBacklog,
};

#[test]
fn postgres_source_quality_backlog_debug_redacts_database_credentials() {
    let credential = "postgres://reviewer:card20-secret@database.internal/nva";
    let adapter = PostgresSourceQualityBacklog::new(credential);

    let debug = format!("{adapter:?}");

    assert!(!debug.contains(credential));
    assert!(!debug.contains("reviewer"));
    assert!(!debug.contains("card20-secret"));
    assert!(debug.contains("<redacted>"));
}

#[test]
fn approval_decision_debug_redacts_actor_identity_and_free_form_reason() {
    let actor_id = "manager-sensitive-17";
    let reason = "private medical review rationale";
    let disposition = ApprovalReviewDisposition::approved(
        ActorKindCode::Manager,
        actor_id.to_owned(),
        "2026-08-14T15:00:00Z".to_owned(),
        Some(reason.to_owned()),
        ApprovalTargetBinding::new(
            "approval-sensitive-17".to_owned(),
            "vaccine_record".to_owned(),
            "vaccine-sensitive-17".to_owned(),
            ReviewGateCode::MedicalDocumentReview,
        ),
    );

    let debug = format!("{disposition:?}");

    assert!(!debug.contains(actor_id));
    assert!(!debug.contains(reason));
    assert!(debug.contains("Approved"));
    assert!(debug.contains("<redacted>"));
}
