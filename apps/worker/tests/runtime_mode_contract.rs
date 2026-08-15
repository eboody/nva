use pet_resort_worker::runtime;

#[test]
fn default_worker_runtime_is_fake_and_side_effect_safe() {
    let config = runtime::Config::from_env_defaults();

    assert_eq!(
        config.agent_runtime_mode(),
        runtime::AgentRuntimeMode::FakeDeterministic
    );
    assert_eq!(config.side_effect_mode(), runtime::SideEffectMode::Stubbed);
}

#[test]
fn claimed_workflow_record_is_processed_as_review_gated_stubbed_work() {
    let config = runtime::Config::from_env_defaults();
    let claim = runtime::ClaimedWorkflowRecord::new(
        "workflow_event:booking.confirmation_needed:reservation:res_123",
        "booking.confirmation_needed",
        runtime::ReviewGate::CustomerMessageApproval,
    );

    let plan = config.processing_contract_for(&claim);

    assert_eq!(
        plan.workflow_event_ref(),
        "workflow_event:booking.confirmation_needed:reservation:res_123"
    );
    assert_eq!(
        plan.agent_runtime_mode(),
        runtime::AgentRuntimeMode::FakeDeterministic
    );
    assert_eq!(plan.side_effect_mode(), runtime::SideEffectMode::Stubbed);
    assert_eq!(
        plan.required_review_gate(),
        runtime::ReviewGate::CustomerMessageApproval
    );
    assert_eq!(
        plan.outbox_status(),
        runtime::OutboxProcessingStatus::ReviewGatedStub
    );
    assert!(plan.requires_human_review_before_external_delivery());
    assert!(plan.blocks_live_customer_messages());
    assert!(plan.blocks_live_provider_writes());
    assert!(plan.blocks_live_payment_actions());
}

#[test]
fn disabled_agent_runtime_still_keeps_side_effects_stubbed() {
    let config = runtime::Config::disabled_for_tests();
    let claim = runtime::ClaimedWorkflowRecord::new(
        "workflow_event:vaccine.extraction_needed:document:doc_123",
        "vaccine.extraction_needed",
        runtime::ReviewGate::MedicalDocumentReview,
    );

    let plan = config.processing_contract_for(&claim);

    assert_eq!(
        plan.agent_runtime_mode(),
        runtime::AgentRuntimeMode::Disabled
    );
    assert_eq!(plan.side_effect_mode(), runtime::SideEffectMode::Stubbed);
    assert_eq!(
        plan.required_review_gate(),
        runtime::ReviewGate::MedicalDocumentReview
    );
    assert!(plan.requires_human_review_before_external_delivery());
}

#[test]
fn reviewed_outcome_without_authenticated_admission_remains_non_executable() {
    let config = runtime::Config::disabled_for_tests();
    let records = unadmitted_data_quality_hygiene_projection();

    let proof = config.process_data_quality_hygiene_projection(&records);

    assert_eq!(proof.outbox_candidate_id(), None);
    assert_eq!(proof.outbox_topic(), None);
    assert!(!proof.has_reviewed_outcome());
    assert_eq!(
        proof.outbox_status(),
        runtime::OutboxProcessingStatus::ReviewGatedStub
    );
    assert!(proof.requires_human_review_before_external_delivery());
    assert!(proof.blocks_live_customer_messages());
    assert!(proof.blocks_live_provider_writes());
    assert!(proof.blocks_live_payment_actions());
    assert!(proof.is_fake_local_only());
}

fn unadmitted_data_quality_hygiene_projection()
-> storage::operations::DataQualityHygieneLocalPersistenceRecords {
    use storage::operations::{
        DataQualityHygieneActionKindCode, DataQualityHygieneLineageIds,
        DataQualityHygieneLocalPersistenceRecords, DataQualityHygieneOutcomeCode,
        DataQualityHygieneOutcomeRecord, DataQualityHygieneOutcomeSchemaVersion,
        DataQualityHygienePersonaCode, DataQualityResolutionStatusCode,
        StoredDataQualityHygieneLaborMinutes, StoredSourceRecordRef,
    };

    let recorded_at = "2026-06-17T13:15:00Z".to_owned();
    let subject_id = "00c0ffee-0000-0000-0000-000000000001".to_owned();
    let outcome = DataQualityHygieneOutcomeRecord {
        schema_version: DataQualityHygieneOutcomeSchemaVersion::V1,
        action_id: "dq-action-demo-1".to_owned(),
        outcome: DataQualityHygieneOutcomeCode::Completed,
        before_minutes: StoredDataQualityHygieneLaborMinutes::try_new(20).unwrap(),
        actual_minutes: StoredDataQualityHygieneLaborMinutes::try_new(8).unwrap(),
        reported_estimated_minutes_difference: 12,
        actor_id: "front-desk-lead-17".to_owned(),
        actor_persona: DataQualityHygienePersonaCode::FrontDeskLead,
        feedback: "Reviewed local cleanup evidence without granting execution authority."
            .to_owned(),
        source_refs: vec![StoredSourceRecordRef {
            system: "gingr".to_owned(),
            record_type: "source_record".to_owned(),
            record_id: "demo-source-1".to_owned(),
            observed_at: recorded_at.clone(),
            adapter_version: "local-demo".to_owned(),
        }],
        issue_refs: vec!["dq-issue-demo-1".to_owned()],
        resolution_status_after_review: DataQualityResolutionStatusCode::Acknowledged,
        recorded_at: recorded_at.clone(),
        correlation_id: "data-quality-hygiene:demo-correlation".to_owned(),
        location_id: subject_id,
        operating_day: "2026-06-17".to_owned(),
        action_kind: DataQualityHygieneActionKindCode::ReviewStaleVaccinationSourceFreshness,
        owner_persona: DataQualityHygienePersonaCode::GeneralManager,
    };

    DataQualityHygieneLocalPersistenceRecords::from_reviewed_outcome(
        DataQualityHygieneLineageIds::builder()
            .workflow_event_id("dqh-workflow-event:demo-1".to_owned())
            .review_packet_id("dqh-review-packet:demo-1".to_owned())
            .approval_record_id("dqh-approval:demo-1".to_owned())
            .outbox_record_id("dqh-outbox:demo-1".to_owned())
            .subject_id("00c0ffee-0000-0000-0000-000000000001".to_owned())
            .idempotency_key("dqh-demo-1:internal-reviewed-handoff".to_owned())
            .recorded_at(recorded_at)
            .build(),
        outcome,
    )
}
