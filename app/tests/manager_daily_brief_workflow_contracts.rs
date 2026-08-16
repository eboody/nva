use chrono::{DateTime, NaiveDate, Utc};
use strum::VariantArray;
use uuid::Uuid;

use app::{checkout_completion, crm_retention, manager_daily_brief};
use domain::{analytics, data_quality, entities, message, operations, policy, source};

#[test]
fn manager_brief_sensitive_text_debug_output_is_redacted() {
    let summary = manager_daily_brief::BriefSummary::try_new("private shift summary").unwrap();
    let rationale =
        manager_daily_brief::ActionRationale::try_new("private action rationale").unwrap();
    let feedback =
        manager_daily_brief::ManagerFeedback::try_new("private manager feedback").unwrap();
    let care = checkout_completion::CareSummary::try_new("private medication handoff").unwrap();

    assert_eq!(format!("{summary:?}"), "BriefSummary(<redacted>)");
    assert_eq!(format!("{rationale:?}"), "ActionRationale(<redacted>)");
    assert_eq!(format!("{feedback:?}"), "ManagerFeedback(<redacted>)");
    assert_eq!(format!("{care:?}"), "CareSummary(<redacted>)");

    let request = manager_daily_brief::Request::builder()
        .location_id(location_id())
        .operating_day(operating_day())
        .prepared_for(manager_daily_brief::ManagerBriefPersona::GeneralManager)
        .demand_attention_threshold(manager_daily_brief::DemandThresholdUnits::try_new(10).unwrap())
        .retention_packets(vec![scoped_retention_packet(eligible_retention_packet())])
        .build();
    let debug = format!("{request:?}");
    assert_eq!(debug, "Request([REDACTED])");

    let action_request = manager_daily_brief::Request::builder()
        .location_id(location_id())
        .operating_day(operating_day())
        .prepared_for(manager_daily_brief::ManagerBriefPersona::GeneralManager)
        .demand_attention_threshold(manager_daily_brief::DemandThresholdUnits::try_new(10).unwrap())
        .service_demand_facts(vec![service_demand_fact(12, vec![])])
        .build();
    let action_packet = manager_daily_brief::Workflow::evaluate(action_request);
    assert_eq!(
        format!("{:?}", action_packet.actions()[0].id()),
        "ActionId([REDACTED])"
    );
    assert_eq!(
        format!("{:?}", &action_packet.actions()[0]),
        "BriefAction([REDACTED])"
    );
    assert_eq!(format!("{action_packet:?}"), "Packet([REDACTED])");

    let scoped_checkout = scoped_checkout_packet(open_checkout_packet());
    assert_eq!(
        format!("{scoped_checkout:?}"),
        "ScopedCheckoutPacket([REDACTED])"
    );
    let scoped_retention = scoped_retention_packet(eligible_retention_packet());
    assert_eq!(
        format!("{scoped_retention:?}"),
        "ScopedRetentionPacket([REDACTED])"
    );
}

#[test]
fn manager_daily_brief_contract_builds_source_grounded_actions_with_labor_delta() {
    let request = manager_daily_brief::Request::builder()
        .location_id(location_id())
        .operating_day(operating_day())
        .prepared_for(manager_daily_brief::ManagerBriefPersona::GeneralManager)
        .demand_attention_threshold(manager_daily_brief::DemandThresholdUnits::try_new(10).unwrap())
        .service_demand_facts(vec![service_demand_fact(12, vec![])])
        .retention_packets(vec![scoped_retention_packet(eligible_retention_packet())])
        .build();

    let packet = manager_daily_brief::Workflow::evaluate(request);

    assert_eq!(packet.actions().len(), 1);
    assert!(packet.all_actions_are_source_grounded());
    assert_eq!(packet.before_minutes().get(), 45);
    assert_eq!(packet.after_minutes().get(), 15);
    assert_eq!(packet.reported_estimated_minutes_difference(), 30);
    assert!(
        packet
            .safe_agent_actions()
            .contains(&manager_daily_brief::SafeAgentAction::ReportLaborEstimateDifference)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&manager_daily_brief::BlockedAction::ChangeStaffSchedule)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&manager_daily_brief::BlockedAction::MutateProviderOrPmsRecord)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&manager_daily_brief::BlockedAction::SendCustomerMessage)
    );

    let demand_action = packet
        .actions()
        .iter()
        .find(|action| {
            action.kind() == manager_daily_brief::BriefActionKind::ReviewDemandAgainstStaffingPlan
        })
        .expect("demand action exists");
    assert_eq!(
        demand_action.owner_persona(),
        manager_daily_brief::ManagerBriefPersona::GeneralManager
    );
    assert_eq!(
        demand_action.removed_manual_work(),
        manager_daily_brief::RemovedManualWork::DemandVersusStaffingScan
    );
    assert_eq!(
        demand_action
            .labor_impact()
            .reported_estimated_minutes_difference(),
        30
    );
    assert_eq!(
        demand_action.source_facts()[0].kind(),
        manager_daily_brief::SourceFactKind::ServiceDemandForecast
    );

    assert!(!packet.actions().iter().any(|action| {
        action.kind() == manager_daily_brief::BriefActionKind::ApproveRetentionFollowUpDraft
    }));
}

#[test]
fn manager_daily_brief_retains_capacity_labor_recommendation_and_reported_outcome_trace() {
    let recommendation = capacity_labor_recommendation(
        entities::ServiceKind::Boarding,
        operations::labor::Role::FrontDesk,
        312,
        240,
        72,
    );
    let request = manager_daily_brief::Request::builder()
        .location_id(location_id())
        .operating_day(operating_day())
        .prepared_for(manager_daily_brief::ManagerBriefPersona::GeneralManager)
        .demand_attention_threshold(manager_daily_brief::DemandThresholdUnits::try_new(10).unwrap())
        .capacity_labor_recommendations(vec![recommendation.clone()])
        .build();

    let packet = manager_daily_brief::Workflow::evaluate(request);
    let action = packet
        .actions()
        .iter()
        .find(|action| {
            action.kind() == manager_daily_brief::BriefActionKind::ReviewCapacityLaborRecommendation
        })
        .expect("capacity labor recommendation appears in the brief");

    assert!(action.is_source_grounded());
    assert_eq!(
        action.removed_manual_work(),
        manager_daily_brief::RemovedManualWork::ServiceCapacityLaborPlanning
    );
    assert!(action.source_facts().iter().any(|fact| {
        fact.kind() == manager_daily_brief::SourceFactKind::CapacityLaborRecommendation
            && fact.source_record_refs().len() == 2
    }));
    assert_eq!(
        action.required_review_gates(),
        &[policy::ReviewGate::ManagerApproval]
    );
    assert_eq!(
        action
            .capacity_labor_recommendation()
            .expect("action retains relationship-checked recommendation")
            .expected_labor_delta_minutes()
            .get(),
        recommendation.expected_labor_delta_minutes().get()
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&manager_daily_brief::BlockedAction::ChangeStaffSchedule)
    );

    let outcome = manager_daily_brief::OutcomeRecord::builder()
        .action_id(action.id().clone())
        .recorded_by(entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("gm-capacity-review").unwrap(),
        })
        .outcome(manager_daily_brief::FeedbackOutcome::Completed)
        .before_minutes(action.labor_impact().before_minutes())
        .actual_minutes(manager_daily_brief::LaborMinutes::try_new(18).unwrap())
        .manager_feedback(manager_daily_brief::ManagerFeedback::try_new(
            "Reviewed capacity/labor evidence and adjusted the internal plan manually; no schedule mutation came from the agent.",
        ).unwrap())
        .source_record_refs(
            action
                .source_facts()
                .iter()
                .flat_map(|fact| fact.source_record_refs().iter().cloned())
                .collect::<Vec<_>>(),
        )
        .build();

    assert_eq!(
        outcome.labor_savings_claim_for_action(action),
        manager_daily_brief::LaborSavingsClaim::NotClaimed {
            reason: manager_daily_brief::LaborSavingsNotClaimedReason::ReportedCompletedLabel
        }
    );
    assert!(outcome.records_feedback_without_external_mutation());
}

#[test]
fn capacity_labor_recommendations_fail_closed_before_manager_brief_when_inputs_are_incompatible() {
    let bucket = capacity_bucket();
    let demand = capacity_demand_for(entities::ServiceKind::Boarding, bucket, 26, 12);
    let coverage = scheduled_coverage_for(operations::labor::Role::Groomer, bucket, 240);

    let error = operations::capacity::OptimizationRecommendation::builder()
        .objective(operations::capacity::OptimizationObjective::ReduceFrontDeskBottleneck)
        .demand(demand)
        .coverage(coverage)
        .source_evidence(vec![capacity_provenance(
            "capacity-demand-boarding-2026-06-17",
            source::System::BusinessIntelligence,
        )])
        .solver_status(operations::capacity::SolverStatus::Feasible)
        .alternatives(vec![
            operations::capacity::RecommendedAction::ManagerReviewOnly,
        ])
        .action(operations::capacity::RecommendedAction::add_role_coverage(
            operations::labor::Role::FrontDesk,
            operations::labor::Minutes::try_new(72).unwrap(),
        ))
        .review_gate(policy::ReviewGate::ManagerApproval)
        .build()
        .expect_err("role-mismatched capacity recommendations must fail before brief generation");

    assert_eq!(error, operations::capacity::Error::RoleMismatch);
}

#[test]
fn manager_daily_brief_contract_preserves_review_boundaries_and_data_quality_visibility() {
    let request = manager_daily_brief::Request::builder()
        .location_id(location_id())
        .operating_day(operating_day())
        .prepared_for(manager_daily_brief::ManagerBriefPersona::AssistantGeneralManager)
        .demand_attention_threshold(manager_daily_brief::DemandThresholdUnits::try_new(10).unwrap())
        .service_demand_facts(vec![service_demand_fact(
            18,
            vec![data_quality::Issue::new(
                data_quality::Kind::UnmappedServiceType,
                data_quality::Severity::Warning,
                source_provenance(),
                source::Timestamp::try_new("2026-06-17T00:00:00Z").unwrap(),
                false,
            )],
        )])
        .checkout_packets(vec![scoped_checkout_packet(open_checkout_packet())])
        .build();

    let packet = manager_daily_brief::Workflow::evaluate(request);

    assert_eq!(packet.actions().len(), 2);
    assert!(packet.all_actions_are_source_grounded());

    let demand_action = packet
        .actions()
        .iter()
        .find(|action| {
            action.kind() == manager_daily_brief::BriefActionKind::ReviewDemandAgainstStaffingPlan
        })
        .expect("demand action exists");
    assert!(
        demand_action
            .source_facts()
            .iter()
            .any(|fact| fact.kind() == manager_daily_brief::SourceFactKind::SourceDataQualityIssue)
    );
    assert_eq!(
        demand_action.required_review_gates(),
        &[policy::ReviewGate::ManagerApproval]
    );

    let checkout_action = packet
        .actions()
        .iter()
        .find(|action| {
            action.kind() == manager_daily_brief::BriefActionKind::ResolveCheckoutException
        })
        .expect("checkout action exists");
    assert_eq!(
        checkout_action.owner_persona(),
        manager_daily_brief::ManagerBriefPersona::FrontDeskLead
    );
    assert_eq!(
        checkout_action.removed_manual_work(),
        manager_daily_brief::RemovedManualWork::CheckoutExceptionAudit
    );
    assert_eq!(
        checkout_action.required_review_gates(),
        &[policy::ReviewGate::ManagerApproval]
    );
    assert!(
        checkout_action.source_facts().iter().any(
            |fact| fact.kind() == manager_daily_brief::SourceFactKind::CheckoutCompletionStatus
        )
    );
}

#[test]
fn manager_daily_brief_ignores_service_demand_outside_requested_location_or_day() {
    let other_location =
        entities::LocationId::new(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0002));
    let other_day =
        operations::operating_day::Date::try_new(NaiveDate::from_ymd_opt(2026, 6, 18).unwrap())
            .unwrap();
    let request = manager_daily_brief::Request::builder()
        .location_id(location_id())
        .operating_day(operating_day())
        .prepared_for(manager_daily_brief::ManagerBriefPersona::GeneralManager)
        .demand_attention_threshold(manager_daily_brief::DemandThresholdUnits::try_new(10).unwrap())
        .service_demand_facts(vec![
            service_demand_fact_for(other_location, operating_day(), 50, vec![]),
            service_demand_fact_for(location_id(), other_day, 50, vec![]),
            service_demand_fact_for(location_id(), operating_day(), 12, vec![]),
        ])
        .build();

    let packet = manager_daily_brief::Workflow::evaluate(request);

    let demand_actions = packet
        .actions()
        .iter()
        .filter(|action| {
            action.kind() == manager_daily_brief::BriefActionKind::ReviewDemandAgainstStaffingPlan
        })
        .collect::<Vec<_>>();
    assert_eq!(demand_actions.len(), 1);
    assert_eq!(
        demand_actions[0]
            .labor_impact()
            .reported_estimated_minutes_difference(),
        30
    );
    assert_eq!(packet.before_minutes().get(), 45);
    assert_eq!(packet.after_minutes().get(), 15);
}

#[test]
fn manager_daily_brief_ignores_checkout_and_retention_packets_outside_requested_scope() {
    let other_location =
        entities::LocationId::new(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0002));
    let request = manager_daily_brief::Request::builder()
        .location_id(location_id())
        .operating_day(operating_day())
        .prepared_for(manager_daily_brief::ManagerBriefPersona::GeneralManager)
        .demand_attention_threshold(manager_daily_brief::DemandThresholdUnits::try_new(10).unwrap())
        .checkout_packets(vec![
            scoped_checkout_packet_for(other_location, operating_day(), open_checkout_packet()),
            scoped_checkout_packet(open_checkout_packet()),
        ])
        .retention_packets(vec![
            scoped_retention_packet_for(
                other_location,
                operating_day(),
                eligible_retention_packet(),
            ),
            scoped_retention_packet(eligible_retention_packet()),
        ])
        .build();

    let packet = manager_daily_brief::Workflow::evaluate(request);

    assert_eq!(packet.actions().len(), 1);
    assert_eq!(packet.before_minutes().get(), 20);
    assert_eq!(packet.after_minutes().get(), 8);
}

#[test]
fn manager_daily_brief_empty_brief_reports_zero_labor_delta_and_all_safety_blockers() {
    let request = manager_daily_brief::Request::builder()
        .location_id(location_id())
        .operating_day(operating_day())
        .prepared_for(manager_daily_brief::ManagerBriefPersona::GeneralManager)
        .demand_attention_threshold(manager_daily_brief::DemandThresholdUnits::try_new(10).unwrap())
        .build();

    let packet = manager_daily_brief::Workflow::evaluate(request);

    assert!(packet.actions().is_empty());
    assert_eq!(packet.before_minutes().get(), 0);
    assert_eq!(packet.after_minutes().get(), 0);
    assert_eq!(packet.reported_estimated_minutes_difference(), 0);
    assert!(
        packet
            .blocked_actions()
            .contains(&manager_daily_brief::BlockedAction::MoveRefundDiscountOrPayment)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&manager_daily_brief::BlockedAction::HideSourceDataQualityIssue)
    );
}

#[test]
fn manager_daily_brief_does_not_promote_serialized_retention_claims_into_draft_authority() {
    let request = manager_daily_brief::Request::builder()
        .location_id(location_id())
        .operating_day(operating_day())
        .prepared_for(manager_daily_brief::ManagerBriefPersona::GeneralManager)
        .demand_attention_threshold(manager_daily_brief::DemandThresholdUnits::try_new(10).unwrap())
        .retention_packets(vec![scoped_retention_packet(eligible_retention_packet())])
        .build();

    let packet = manager_daily_brief::Workflow::evaluate(request);
    assert!(packet.actions().is_empty());
}

#[test]
fn manager_daily_brief_draft_side_effect_validation_rejects_known_blocked_effects_with_evidence() {
    assert_eq!(
        manager_daily_brief::requested_side_effect_rejection_reason("change_staff_schedule"),
        "blocked_side_effect:change_staff_schedule"
    );
    assert_eq!(
        manager_daily_brief::requested_side_effect_rejection_reason("send_customer_message"),
        "blocked_side_effect:send_customer_message"
    );
}

#[test]
fn manager_daily_brief_draft_side_effect_validation_rejects_unknown_effects_fail_closed() {
    assert_eq!(
        manager_daily_brief::requested_side_effect_rejection_reason("invent_new_live_side_effect"),
        "unsupported_side_effect:invent_new_live_side_effect"
    );
}

#[test]
fn manager_daily_brief_blocked_action_codes_roundtrip_through_strum_metadata() {
    for blocked_action in manager_daily_brief::BlockedAction::VARIANTS {
        assert_eq!(blocked_action.to_string(), blocked_action.code());
        assert_eq!(
            manager_daily_brief::BlockedAction::from_requested_side_effect_code(
                blocked_action.code()
            ),
            Some(*blocked_action)
        );
    }

    assert_eq!(
        manager_daily_brief::BlockedAction::from_requested_side_effect_code("unknown_side_effect"),
        None
    );
}

#[test]
fn manager_daily_brief_outcome_capture_records_feedback_without_external_mutation() {
    let outcome = manager_daily_brief::OutcomeRecord::builder()
        .action_id(
            manager_daily_brief::ActionId::try_new("demand-staffing-service-demand-42").unwrap(),
        )
        .recorded_by(entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("gm-riley").unwrap(),
        })
        .outcome(manager_daily_brief::FeedbackOutcome::Completed)
        .before_minutes(manager_daily_brief::LaborMinutes::try_new(45).unwrap())
        .actual_minutes(manager_daily_brief::LaborMinutes::try_new(12).unwrap())
        .source_record_refs(vec![source::RecordRef::from_provenance(
            &source_provenance(),
        )])
        .build();

    assert_eq!(format!("{outcome:?}"), "OutcomeRecord([REDACTED])");
    assert!(!outcome.counts_as_labor_savings());
    assert!(outcome.records_feedback_without_external_mutation());
    assert!(
        outcome
            .blocked_actions()
            .contains(&manager_daily_brief::BlockedAction::MutateProviderOrPmsRecord)
    );
    assert!(
        outcome
            .blocked_actions()
            .contains(&manager_daily_brief::BlockedAction::SendCustomerMessage)
    );
    assert!(
        outcome
            .blocked_actions()
            .contains(&manager_daily_brief::BlockedAction::ChangeStaffSchedule)
    );
    assert!(
        outcome
            .blocked_actions()
            .contains(&manager_daily_brief::BlockedAction::MoveRefundDiscountOrPayment)
    );
    assert!(
        outcome
            .blocked_actions()
            .contains(&manager_daily_brief::BlockedAction::HideSourceDataQualityIssue)
    );
}

#[test]
fn manager_daily_brief_trace_links_source_fact_reviewable_action_and_reported_outcome_label() {
    let request = manager_daily_brief::Request::builder()
        .location_id(location_id())
        .operating_day(operating_day())
        .prepared_for(manager_daily_brief::ManagerBriefPersona::GeneralManager)
        .demand_attention_threshold(manager_daily_brief::DemandThresholdUnits::try_new(10).unwrap())
        .service_demand_facts(vec![service_demand_fact(12, vec![])])
        .build();
    let packet = manager_daily_brief::Workflow::evaluate(request);
    let action = packet
        .actions()
        .iter()
        .find(|action| {
            action.kind() == manager_daily_brief::BriefActionKind::ReviewDemandAgainstStaffingPlan
        })
        .expect("demand action exists");
    let source_record_refs = action
        .source_facts()
        .iter()
        .flat_map(|fact| fact.source_record_refs().iter().cloned())
        .collect::<Vec<_>>();

    let outcome = manager_daily_brief::OutcomeRecord::builder()
        .action_id(action.id().clone())
        .recorded_by(entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("gm-riley").unwrap(),
        })
        .outcome(manager_daily_brief::FeedbackOutcome::Completed)
        .before_minutes(action.labor_impact().before_minutes())
        .actual_minutes(manager_daily_brief::LaborMinutes::try_new(12).unwrap())
        .manager_feedback(manager_daily_brief::ManagerFeedback::try_new(
            "Reviewed source demand, adjusted the internal plan manually, and did not touch Gingr from the agent.",
        ).unwrap())
        .source_record_refs(source_record_refs)
        .build();

    assert!(outcome.matches_action(action));
    assert!(outcome.cites_action_source_evidence(action));
    assert_eq!(
        outcome.labor_savings_claim_for_action(action),
        manager_daily_brief::LaborSavingsClaim::NotClaimed {
            reason: manager_daily_brief::LaborSavingsNotClaimedReason::ReportedCompletedLabel
        }
    );
    assert!(!outcome.counts_as_labor_savings_for_action(action));
    assert_eq!(
        outcome.labor_savings_claim(),
        manager_daily_brief::LaborSavingsClaim::NotClaimed {
            reason: manager_daily_brief::LaborSavingsNotClaimedReason::ReportedCompletedLabel
        }
    );
    assert_eq!(
        outcome.reported_disposition(),
        manager_daily_brief::ReportedDisposition::CompletedLabel
    );
    assert!(outcome.records_feedback_without_external_mutation());
    assert!(
        outcome
            .blocked_actions()
            .contains(&manager_daily_brief::BlockedAction::ChangeStaffSchedule)
    );
}

#[test]
fn manager_daily_brief_reported_completed_label_remains_nonclaimable_with_action_and_source_trace()
{
    let request = manager_daily_brief::Request::builder()
        .location_id(location_id())
        .operating_day(operating_day())
        .prepared_for(manager_daily_brief::ManagerBriefPersona::GeneralManager)
        .demand_attention_threshold(manager_daily_brief::DemandThresholdUnits::try_new(10).unwrap())
        .service_demand_facts(vec![service_demand_fact(12, vec![])])
        .build();
    let packet = manager_daily_brief::Workflow::evaluate(request);
    let action = packet.actions().first().expect("demand action exists");

    let wrong_action_record = manager_daily_brief::OutcomeRecord::builder()
        .action_id(manager_daily_brief::ActionId::try_new("wrong-action-id").unwrap())
        .recorded_by(entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("gm-riley").unwrap(),
        })
        .outcome(manager_daily_brief::FeedbackOutcome::Completed)
        .before_minutes(action.labor_impact().before_minutes())
        .actual_minutes(manager_daily_brief::LaborMinutes::try_new(12).unwrap())
        .source_record_refs(
            action
                .source_facts()
                .iter()
                .flat_map(|fact| fact.source_record_refs().iter().cloned())
                .collect::<Vec<_>>(),
        )
        .build();
    assert_eq!(
        wrong_action_record.labor_savings_claim_for_action(action),
        manager_daily_brief::LaborSavingsClaim::NotClaimed {
            reason: manager_daily_brief::LaborSavingsNotClaimedReason::ReportedCompletedLabel
        }
    );

    let missing_source_record = manager_daily_brief::OutcomeRecord::builder()
        .action_id(action.id().clone())
        .recorded_by(entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("gm-riley").unwrap(),
        })
        .outcome(manager_daily_brief::FeedbackOutcome::Completed)
        .before_minutes(action.labor_impact().before_minutes())
        .actual_minutes(manager_daily_brief::LaborMinutes::try_new(12).unwrap())
        .build();
    assert_eq!(
        missing_source_record.labor_savings_claim_for_action(action),
        manager_daily_brief::LaborSavingsClaim::NotClaimed {
            reason: manager_daily_brief::LaborSavingsNotClaimedReason::ReportedCompletedLabel
        }
    );
}

#[test]
fn caller_reported_wrong_source_deferred_and_suppressed_labels_do_not_claim_labor_savings() {
    for (outcome, expected_reason) in [
        (
            manager_daily_brief::FeedbackOutcome::Deferred,
            manager_daily_brief::LaborSavingsNotClaimedReason::ReportedDeferredLabel,
        ),
        (
            manager_daily_brief::FeedbackOutcome::SuppressedByManager,
            manager_daily_brief::LaborSavingsNotClaimedReason::ReportedSuppressedLabel,
        ),
        (
            manager_daily_brief::FeedbackOutcome::SourceFactWasWrong,
            manager_daily_brief::LaborSavingsNotClaimedReason::ReportedWrongSourceLabel,
        ),
    ] {
        let record = manager_daily_brief::OutcomeRecord::builder()
            .action_id(manager_daily_brief::ActionId::try_new("demand-staffing-service-demand-42").unwrap())
            .recorded_by(entities::ActorRef::Manager {
                manager_id: entities::ManagerId::try_new("gm-riley").unwrap(),
            })
            .outcome(outcome)
            .before_minutes(manager_daily_brief::LaborMinutes::try_new(45).unwrap())
            .actual_minutes(manager_daily_brief::LaborMinutes::try_new(12).unwrap())
            .manager_feedback(manager_daily_brief::ManagerFeedback::try_new(
                "Manager recorded disposition; no autonomous schedule, PMS, customer, or payment side effect occurred.",
            ).unwrap())
            .source_record_refs(vec![source::RecordRef::from_provenance(&source_provenance())])
            .build();

        assert_eq!(
            record.labor_savings_claim(),
            manager_daily_brief::LaborSavingsClaim::NotClaimed {
                reason: expected_reason
            }
        );
        assert!(!record.counts_as_labor_savings());
        assert!(!record.counts_as_labor_savings());
        assert!(record.records_feedback_without_external_mutation());
    }
}

fn service_demand_fact(
    demand_units: u32,
    issues: Vec<data_quality::Issue>,
) -> analytics::service_demand::Fact {
    service_demand_fact_for(location_id(), operating_day(), demand_units, issues)
}

fn service_demand_fact_for(
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    demand_units: u32,
    issues: Vec<data_quality::Issue>,
) -> analytics::service_demand::Fact {
    analytics::service_demand::Fact::try_new(
        analytics::service_demand::Id::try_new("service-demand-42").unwrap(),
        operations::operating_day::Key::new(
            location_id,
            operations::service_core::ServiceLine::Boarding,
            operating_day,
        ),
        analytics::service_demand::DemandUnits::try_new(demand_units).unwrap(),
        vec![source::RecordRef::from_provenance(&source_provenance())],
        analytics::ProjectionVersion::try_new("local-manager-brief-v1").unwrap(),
        issues,
    )
    .unwrap()
}

fn scoped_checkout_packet(
    packet: checkout_completion::Packet,
) -> manager_daily_brief::ScopedCheckoutPacket {
    scoped_checkout_packet_for(location_id(), operating_day(), packet)
}

fn scoped_checkout_packet_for(
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    packet: checkout_completion::Packet,
) -> manager_daily_brief::ScopedCheckoutPacket {
    manager_daily_brief::ScopedCheckoutPacket::builder()
        .location_id(location_id)
        .operating_day(operating_day)
        .packet(packet)
        .build()
}

fn scoped_retention_packet(
    packet: crm_retention::Packet,
) -> manager_daily_brief::ScopedRetentionPacket {
    scoped_retention_packet_for(location_id(), operating_day(), packet)
}

fn scoped_retention_packet_for(
    location_id: entities::LocationId,
    operating_day: operations::operating_day::Date,
    packet: crm_retention::Packet,
) -> manager_daily_brief::ScopedRetentionPacket {
    manager_daily_brief::ScopedRetentionPacket::builder()
        .location_id(location_id)
        .operating_day(operating_day)
        .packet(packet)
        .build()
}

fn eligible_retention_packet() -> crm_retention::Packet {
    crm_retention::Workflow::evaluate(
        crm_retention::Request::builder()
            .reservation_id(reservation_id())
            .customer_id(customer_id())
            .checkout_packet(verified_checkout_packet())
            .contact_permission(email_contact_permission())
            .opportunities(vec![retention_opportunity()])
            .build(),
    )
}

fn verified_checkout_packet() -> checkout_completion::Packet {
    checkout_completion::Workflow::evaluate(
        checkout_completion::Request::builder()
            .reservation_id(reservation_id())
            .source_provenance(source_provenance())
            .observed_source_status(source::reservation::Status::CheckedOut)
            .staff_handoff(resolved_staff_handoff())
            .build(),
    )
}

fn open_checkout_packet() -> checkout_completion::Packet {
    checkout_completion::Workflow::evaluate(
        checkout_completion::Request::builder()
            .reservation_id(reservation_id())
            .source_provenance(source_provenance())
            .observed_source_status(source::reservation::Status::CheckedOut)
            .staff_handoff(open_staff_handoff())
            .build(),
    )
}

fn retention_opportunity() -> crm_retention::RetentionOpportunity {
    crm_retention::RetentionOpportunity::builder()
        .kind(crm_retention::OpportunityKind::NextBoardingStay)
        .evidence(
            crm_retention::OpportunityEvidence::builder()
                .reason_code(crm_retention::SourceGroundedReasonCode::CompletedBoardingStay)
                .summary(
                    crm_retention::EvidenceSummary::try_new(
                        "Completed boarding stay and owner mentioned a return trip.",
                    )
                    .unwrap(),
                )
                .provenance(source_provenance())
                .build(),
        )
        .build()
}

fn capacity_labor_recommendation(
    service: entities::ServiceKind,
    role: operations::labor::Role,
    required_minutes: u32,
    scheduled_minutes: u16,
    add_minutes: u16,
) -> operations::capacity::OptimizationRecommendation {
    let bucket = capacity_bucket();
    operations::capacity::OptimizationRecommendation::builder()
        .objective(operations::capacity::OptimizationObjective::ReduceFrontDeskBottleneck)
        .demand(capacity_demand_for(
            service,
            bucket,
            required_minutes / 12,
            12,
        ))
        .coverage(scheduled_coverage_for(role, bucket, scheduled_minutes))
        .source_evidence(vec![
            capacity_provenance(
                "capacity-demand-boarding-2026-06-17",
                source::System::BusinessIntelligence,
            ),
            capacity_provenance(
                "labor-coverage-front-desk-2026-06-17",
                source::System::LaborScheduling,
            ),
        ])
        .solver_status(operations::capacity::SolverStatus::Feasible)
        .alternatives(vec![
            operations::capacity::RecommendedAction::ManagerReviewOnly,
            operations::capacity::RecommendedAction::reassign_coverage(
                operations::labor::Role::KennelTechnician,
                role,
                operations::labor::Minutes::try_new(add_minutes).unwrap(),
            ),
        ])
        .action(operations::capacity::RecommendedAction::add_role_coverage(
            role,
            operations::labor::Minutes::try_new(add_minutes).unwrap(),
        ))
        .review_gate(policy::ReviewGate::ManagerApproval)
        .build()
        .unwrap()
}

fn capacity_demand_for(
    service: entities::ServiceKind,
    bucket: operations::time_bucket::Window,
    quantity: u32,
    labor_minutes_per_unit: u16,
) -> operations::capacity::DemandUnit {
    operations::capacity::DemandUnit::builder()
        .location_id(location_id())
        .service(service)
        .bucket(bucket)
        .quantity(operations::capacity::Quantity::try_new(quantity).unwrap())
        .labor_minutes_per_unit(
            operations::labor::Minutes::try_new(labor_minutes_per_unit).unwrap(),
        )
        .constraints(vec![
            operations::capacity::Constraint::CheckInCheckoutBottleneck,
        ])
        .build()
}

fn scheduled_coverage_for(
    role: operations::labor::Role,
    bucket: operations::time_bucket::Window,
    scheduled_minutes: u16,
) -> operations::labor::ScheduledCoverage {
    operations::labor::ScheduledCoverage::builder()
        .location_id(location_id())
        .bucket(bucket)
        .role(role)
        .scheduled_people(operations::labor::PeopleCount::try_new(1).unwrap())
        .scheduled_minutes(operations::labor::Minutes::try_new(scheduled_minutes).unwrap())
        .loaded_cost(domain::money::Money::usd(7_200).unwrap())
        .build()
}

fn capacity_bucket() -> operations::time_bucket::Window {
    operations::time_bucket::Window::new(
        DateTime::<Utc>::UNIX_EPOCH,
        DateTime::<Utc>::UNIX_EPOCH + chrono::Duration::hours(4),
    )
    .unwrap()
}

fn capacity_provenance(record_id: &str, system: source::System) -> source::Provenance {
    source::Provenance::builder()
        .system(system)
        .endpoint(source::Endpoint::try_new("capacity-labor-forecast").unwrap())
        .record_id(source::record::Id::try_new(record_id).unwrap())
        .extraction_batch(
            source::ExtractionBatchId::try_new("manager-brief-capacity-batch").unwrap(),
        )
        .pulled_at(source::Timestamp::try_new("2026-06-17T06:00:00Z").unwrap())
        .request_scope(source::RequestScope::try_new("manager-daily-brief-capacity-labor").unwrap())
        .schema_version(source::SchemaVersion::try_new("capacity-labor-v1").unwrap())
        .payload_hash(source::PayloadHash::try_new(format!("sha256:{record_id}")).unwrap())
        .raw_payload_ref(
            source::RawPayloadRef::try_new(format!("fixtures/capacity-labor/{record_id}.json"))
                .unwrap(),
        )
        .build()
}

fn email_contact_permission() -> crm_retention::ContactPermission {
    crm_retention::ContactPermission::builder()
        .preferred_channel(message::Channel::Email)
        .allowed_channels(vec![message::Channel::Email])
        .marketing_consent(crm_retention::ConsentStatus::Granted)
        .transactional_consent(crm_retention::ConsentStatus::Granted)
        .source_record_refs(vec![source::RecordRef::from_provenance(
            &contact_provenance(),
        )])
        .build()
}

fn resolved_staff_handoff() -> checkout_completion::StaffHandoff {
    checkout_completion::StaffHandoff::builder()
        .reported_completed_by(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("front-desk-erin").unwrap(),
        })
        .reported_completed_at(DateTime::<Utc>::UNIX_EPOCH)
        .belongings_status(checkout_completion::BelongingsStatus::ReturnedToCustomer)
        .care_summary(checkout_completion::CareSummary::try_new("Clean checkout.").unwrap())
        .departure_notes_review(checkout_completion::DepartureNotesReview::StaffReviewed)
        .build()
}

fn open_staff_handoff() -> checkout_completion::StaffHandoff {
    checkout_completion::StaffHandoff::builder()
        .reported_completed_by(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("front-desk-erin").unwrap(),
        })
        .reported_completed_at(DateTime::<Utc>::UNIX_EPOCH)
        .belongings_status(checkout_completion::BelongingsStatus::NeedsStaffFollowUp)
        .care_summary(
            checkout_completion::CareSummary::try_new("Medication bag needs review.").unwrap(),
        )
        .departure_notes_review(checkout_completion::DepartureNotesReview::ManagerReviewRequired)
        .build()
}

fn source_provenance() -> source::Provenance {
    source::Provenance::builder()
        .system(source::System::Gingr)
        .endpoint(source::Endpoint::try_new("GET /reservations/{id}").unwrap())
        .record_id(source::record::Id::try_new("reservation-42").unwrap())
        .extraction_batch(source::ExtractionBatchId::try_new("manager-brief-batch-local").unwrap())
        .pulled_at(source::Timestamp::try_new("2026-06-17T00:00:00Z").unwrap())
        .request_scope(source::RequestScope::try_new("local-manager-daily-brief-contract").unwrap())
        .schema_version(source::SchemaVersion::try_new("gingr-v0-readonly").unwrap())
        .payload_hash(source::PayloadHash::try_new("sha256:managerbrieffixture").unwrap())
        .raw_payload_ref(
            source::RawPayloadRef::try_new("fixtures/gingr/manager-brief.json").unwrap(),
        )
        .build()
}

fn contact_provenance() -> source::Provenance {
    source::Provenance::builder()
        .system(source::System::Gingr)
        .endpoint(source::Endpoint::try_new("GET /customers/{id}/contact-permissions").unwrap())
        .record_id(source::record::Id::try_new("customer-contact-99").unwrap())
        .extraction_batch(source::ExtractionBatchId::try_new("manager-brief-batch-local").unwrap())
        .pulled_at(source::Timestamp::try_new("2026-06-17T00:00:00Z").unwrap())
        .request_scope(source::RequestScope::try_new("local-manager-daily-brief-contract").unwrap())
        .schema_version(source::SchemaVersion::try_new("gingr-v0-readonly").unwrap())
        .payload_hash(source::PayloadHash::try_new("sha256:managerbriefcontactfixture").unwrap())
        .raw_payload_ref(
            source::RawPayloadRef::try_new("fixtures/gingr/manager-brief-contact.json").unwrap(),
        )
        .build()
}

fn location_id() -> entities::LocationId {
    entities::LocationId::new(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0001))
}

fn customer_id() -> entities::CustomerId {
    entities::CustomerId::new(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0099))
}

fn reservation_id() -> entities::reservation::Id {
    entities::reservation::Id::new(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0042))
}

fn operating_day() -> operations::operating_day::Date {
    operations::operating_day::Date::try_new(NaiveDate::from_ymd_opt(2026, 6, 17).unwrap()).unwrap()
}

#[test]
fn caller_feedback_labels_remain_reported_dispositions_without_manager_or_source_authority() {
    use manager_daily_brief::{FeedbackOutcome, ReportedDisposition};

    assert_eq!(
        [
            FeedbackOutcome::Completed.reported_disposition(),
            FeedbackOutcome::Deferred.reported_disposition(),
            FeedbackOutcome::SuppressedByManager.reported_disposition(),
            FeedbackOutcome::SourceFactWasWrong.reported_disposition(),
        ],
        [
            ReportedDisposition::CompletedLabel,
            ReportedDisposition::DeferredLabel,
            ReportedDisposition::SuppressedLabel,
            ReportedDisposition::WrongSourceLabel,
        ]
    );
}
