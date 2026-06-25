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
fn completed_data_quality_hygiene_projection_becomes_disabled_internal_handoff_proof() {
    let config = runtime::Config::disabled_for_tests();
    let records = reviewed_data_quality_hygiene_projection();

    let proof = config.process_data_quality_hygiene_projection(&records);

    let telemetry = proof.telemetry_fields();
    assert_eq!(telemetry.event(), "outbox_blocked");
    assert_eq!(telemetry.workflow_event_id(), "dqh-workflow-event:demo-1");
    assert_eq!(
        telemetry.correlation_id(),
        "data-quality-hygiene:demo-correlation"
    );
    assert_eq!(telemetry.worker_id(), "local-disabled-worker");
    assert_eq!(telemetry.attempt_count(), 1);
    assert_eq!(telemetry.safe_error_class(), "review_gated_stub");
    assert_eq!(telemetry.payload_logging(), "disabled");
    assert!(!telemetry.live_delivery_allowed());

    assert_eq!(proof.workflow_name(), "data-quality-hygiene");
    assert_eq!(proof.workflow_event_ref(), "dqh-workflow-event:demo-1");
    assert_eq!(
        proof.agent_runtime_mode(),
        runtime::AgentRuntimeMode::Disabled
    );
    assert_eq!(proof.side_effect_mode(), runtime::SideEffectMode::Stubbed);
    assert_eq!(
        proof.outbox_status(),
        runtime::OutboxProcessingStatus::ReviewGatedStub
    );
    assert_eq!(proof.outbox_candidate_id(), Some("dqh-outbox:demo-1"));
    assert_eq!(
        proof.outbox_topic(),
        Some("internal.data_quality_hygiene.reviewed_handoff")
    );
    assert!(proof.has_reviewed_outcome());
    assert!(proof.requires_human_review_before_external_delivery());
    assert!(proof.blocks_live_customer_messages());
    assert!(proof.blocks_live_provider_writes());
    assert!(proof.blocks_live_payment_actions());
    assert!(proof.is_fake_local_only());
    assert_eq!(proof.audit_event_count(), 2);
}

#[test]
fn unapproved_or_non_internal_handoff_projection_is_not_reviewed_outcome() {
    use storage::operations::{OutboxStatusCode, ReviewPacketStatusCode};

    let config = runtime::Config::disabled_for_tests();
    let mut records = reviewed_data_quality_hygiene_projection();
    records.review_packet.status = ReviewPacketStatusCode::ReadyForReview;

    let proof = config.process_data_quality_hygiene_projection(&records);

    assert!(!proof.has_reviewed_outcome());
    assert!(proof.requires_human_review_before_external_delivery());
    assert!(proof.blocks_live_customer_messages());
    assert!(proof.blocks_live_provider_writes());
    assert!(proof.blocks_live_payment_actions());

    let mut records = reviewed_data_quality_hygiene_projection();
    records.approval_record.status = "approval_requested".to_owned();

    let proof = config.process_data_quality_hygiene_projection(&records);

    assert!(!proof.has_reviewed_outcome());
    assert!(proof.is_fake_local_only());

    let mut records = reviewed_data_quality_hygiene_projection();
    let candidate = records
        .outbox_candidate
        .as_mut()
        .expect("demo outbox candidate");
    candidate.topic = "customer.sms.send".to_owned();
    candidate.payload = serde_json::json!({
        "internal_handoff_only": false,
        "live_delivery_allowed": true
    });
    candidate.status = OutboxStatusCode::Published;

    let proof = config.process_data_quality_hygiene_projection(&records);

    assert!(!proof.has_reviewed_outcome());
    assert_eq!(
        proof.outbox_status(),
        runtime::OutboxProcessingStatus::ReviewGatedStub
    );
    assert!(proof.requires_human_review_before_external_delivery());
    assert!(proof.blocks_live_customer_messages());
    assert!(proof.blocks_live_provider_writes());
    assert!(proof.blocks_live_payment_actions());
}

fn reviewed_data_quality_hygiene_projection()
-> storage::operations::DataQualityHygieneLocalPersistenceRecords {
    use storage::operations::{
        ActorKindCode, ApprovalRecordRow, AuditEventRecord, DataQualityHygieneActionKindCode,
        DataQualityHygieneLocalPersistenceRecords, DataQualityHygieneOutcomeCode,
        DataQualityHygieneOutcomeRecord, DataQualityHygieneOutcomeRow,
        DataQualityHygienePersonaCode, DataQualityResolutionStatusCode, OutboxRecord,
        OutboxStatusCode, ReviewGateCode, ReviewPacketRecord, ReviewPacketStatusCode,
        StoredDataQualityHygieneLaborMinutes, StoredSourceRecordRef, WorkflowEventRecord,
        WorkflowResultRecord, WorkflowResultStatusCode,
    };

    let recorded_at = "2026-06-17T13:15:00Z".to_owned();
    let outcome = DataQualityHygieneOutcomeRecord {
        action_id: "dq-action-demo-1".to_owned(),
        outcome: DataQualityHygieneOutcomeCode::Completed,
        before_minutes: StoredDataQualityHygieneLaborMinutes::try_new(20).unwrap(),
        actual_minutes: StoredDataQualityHygieneLaborMinutes::try_new(8).unwrap(),
        estimated_minutes_saved: 12,
        actor_id: "front-desk-lead-17".to_owned(),
        actor_persona: DataQualityHygienePersonaCode::FrontDeskLead,
        feedback: "Reviewed local cleanup handoff candidate without touching provider records."
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
        location_id: "00c0ffee-0000-0000-0000-000000000001".to_owned(),
        operating_day: "2026-06-17".to_owned(),
        action_kind: DataQualityHygieneActionKindCode::ReviewStaleVaccinationSourceFreshness,
        owner_persona: DataQualityHygienePersonaCode::GeneralManager,
    };

    DataQualityHygieneLocalPersistenceRecords {
        workflow_event: WorkflowEventRecord {
            id: "dqh-workflow-event:demo-1".to_owned(),
            workflow_name: "data-quality-hygiene".to_owned(),
            event_kind: "context_created".to_owned(),
            subject_kind: "location".to_owned(),
            subject_id: "00c0ffee-0000-0000-0000-000000000001".to_owned(),
            idempotency_key: "dqh-demo-1".to_owned(),
            payload: serde_json::json!({"live_side_effects_allowed": false}),
            occurred_at: recorded_at.clone(),
            recorded_at: recorded_at.clone(),
        },
        workflow_result: WorkflowResultRecord {
            id: "dqh-workflow-event:demo-1:result".to_owned(),
            workflow_event_id: "dqh-workflow-event:demo-1".to_owned(),
            status: WorkflowResultStatusCode::Succeeded,
            result: serde_json::json!({"reviewable_output_only": true, "live_side_effects_allowed": false}),
            error_code: None,
            created_at: recorded_at.clone(),
        },
        review_packet: ReviewPacketRecord {
            id: "dqh-review-packet:demo-1".to_owned(),
            subject_kind: "location".to_owned(),
            subject_id: "00c0ffee-0000-0000-0000-000000000001".to_owned(),
            gate: ReviewGateCode::ManagerApproval,
            status: ReviewPacketStatusCode::Approved,
            workflow_event_id: "dqh-workflow-event:demo-1".to_owned(),
            created_by_actor_kind: ActorKindCode::Agent,
            created_by_actor_id: "data-quality-hygiene-agent".to_owned(),
            created_at: recorded_at.clone(),
            updated_at: recorded_at.clone(),
        },
        approval_record: ApprovalRecordRow {
            id: "dqh-approval:demo-1".to_owned(),
            target_kind: "message".to_owned(),
            target_id: "00c0ffee-0000-0000-0000-000000000001".to_owned(),
            gate: ReviewGateCode::ManagerApproval,
            status: "approved".to_owned(),
            requested_by_actor_kind: ActorKindCode::Agent,
            requested_by_actor_id: "data-quality-hygiene-agent".to_owned(),
            requested_at: recorded_at.clone(),
            decided_by_actor_kind: Some(ActorKindCode::Staff),
            decided_by_actor_id: Some("front-desk-lead-17".to_owned()),
            decided_at: Some(recorded_at.clone()),
            review_packet_id: "dqh-review-packet:demo-1".to_owned(),
        },
        outcome: DataQualityHygieneOutcomeRow {
            workflow_event_id: "dqh-workflow-event:demo-1".to_owned(),
            approval_record_id: "dqh-approval:demo-1".to_owned(),
            record: outcome,
        },
        audit_events: vec![
            AuditEventRecord {
                actor_kind: ActorKindCode::Agent,
                actor_id: "data-quality-hygiene-agent".to_owned(),
                subject_kind: "workflow_event".to_owned(),
                subject_id: "dqh-workflow-event:demo-1".to_owned(),
                action: "data_quality_hygiene.context_created".to_owned(),
                workflow_event_id: "dqh-workflow-event:demo-1".to_owned(),
                metadata: serde_json::json!({"live_side_effects_allowed": false}),
                occurred_at: recorded_at.clone(),
                recorded_at: recorded_at.clone(),
            },
            AuditEventRecord {
                actor_kind: ActorKindCode::Staff,
                actor_id: "front-desk-lead-17".to_owned(),
                subject_kind: "approval".to_owned(),
                subject_id: "dqh-approval:demo-1".to_owned(),
                action: "data_quality_hygiene.reviewed_outcome_recorded".to_owned(),
                workflow_event_id: "dqh-workflow-event:demo-1".to_owned(),
                metadata: serde_json::json!({"live_side_effects_allowed": false}),
                occurred_at: recorded_at.clone(),
                recorded_at: recorded_at.clone(),
            },
        ],
        outbox_candidate: Some(OutboxRecord {
            id: "dqh-outbox:demo-1".to_owned(),
            idempotency_key: "dqh-demo-1:internal-reviewed-handoff".to_owned(),
            approval_record_id: "dqh-approval:demo-1".to_owned(),
            topic: "internal.data_quality_hygiene.reviewed_handoff".to_owned(),
            review_gate: ReviewGateCode::ManagerApproval,
            aggregate_kind: "message".to_owned(),
            aggregate_id: "00c0ffee-0000-0000-0000-000000000001".to_owned(),
            payload: serde_json::json!({
                "internal_handoff_only": true,
                "live_delivery_allowed": false
            }),
            status: OutboxStatusCode::Pending,
            available_at: recorded_at,
        }),
    }
}
