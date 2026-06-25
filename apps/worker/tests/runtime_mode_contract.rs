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
