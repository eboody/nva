use chrono::{DateTime, NaiveDate, Utc};
use strum::VariantArray;
use uuid::Uuid;

use app::{checkout_completion, crm_retention, manager_daily_brief};
use domain::{analytics, data_quality, entities, message, operations, policy, source};

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

    assert_eq!(packet.actions().len(), 2);
    assert!(packet.all_actions_are_source_grounded());
    assert_eq!(packet.before_minutes().get(), 75);
    assert_eq!(packet.after_minutes().get(), 25);
    assert_eq!(packet.minutes_saved(), 50);
    assert!(
        packet
            .safe_agent_actions()
            .contains(&manager_daily_brief::SafeAgentAction::EstimateLaborMinutesSaved)
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
    assert_eq!(demand_action.labor_impact().minutes_saved(), 30);
    assert_eq!(
        demand_action.source_facts()[0].kind(),
        manager_daily_brief::SourceFactKind::ServiceDemandForecast
    );

    let retention_action = packet
        .actions()
        .iter()
        .find(|action| {
            action.kind() == manager_daily_brief::BriefActionKind::ApproveRetentionFollowUpDraft
        })
        .expect("retention action exists");
    assert_eq!(
        retention_action.owner_persona(),
        manager_daily_brief::ManagerBriefPersona::FrontDeskLead
    );
    assert_eq!(
        retention_action.removed_manual_work(),
        manager_daily_brief::RemovedManualWork::RetentionFollowUpQueuePrioritization
    );
    assert_eq!(
        retention_action.required_review_gates(),
        &[policy::ReviewGate::CustomerMessageApproval]
    );
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
        entities::LocationId(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0002));
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
    assert_eq!(demand_actions[0].labor_impact().minutes_saved(), 30);
    assert_eq!(packet.before_minutes().get(), 45);
    assert_eq!(packet.after_minutes().get(), 15);
}

#[test]
fn manager_daily_brief_ignores_checkout_and_retention_packets_outside_requested_scope() {
    let other_location =
        entities::LocationId(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0002));
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

    assert_eq!(packet.actions().len(), 2);
    assert_eq!(packet.before_minutes().get(), 50);
    assert_eq!(packet.after_minutes().get(), 18);
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
    assert_eq!(packet.minutes_saved(), 0);
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
fn manager_daily_brief_retention_action_includes_checkout_and_consent_source_evidence() {
    let request = manager_daily_brief::Request::builder()
        .location_id(location_id())
        .operating_day(operating_day())
        .prepared_for(manager_daily_brief::ManagerBriefPersona::GeneralManager)
        .demand_attention_threshold(manager_daily_brief::DemandThresholdUnits::try_new(10).unwrap())
        .retention_packets(vec![scoped_retention_packet(eligible_retention_packet())])
        .build();

    let packet = manager_daily_brief::Workflow::evaluate(request);
    let retention_action = packet
        .actions()
        .iter()
        .find(|action| {
            action.kind() == manager_daily_brief::BriefActionKind::ApproveRetentionFollowUpDraft
        })
        .expect("retention action exists");

    assert!(
        retention_action.source_facts()[0]
            .source_record_refs()
            .len()
            >= 3,
        "retention action should include opportunity evidence, checkout provenance, and contact/consent provenance"
    );
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

    assert_eq!(outcome.actual_minutes_saved(), 33);
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
fn manager_daily_brief_trace_links_source_fact_reviewable_action_and_completed_outcome() {
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
        manager_daily_brief::LaborSavingsClaim::Supported { minutes: 33 }
    );
    assert!(outcome.counts_as_labor_savings_for_action(action));
    assert_eq!(
        outcome.labor_savings_claim(),
        manager_daily_brief::LaborSavingsClaim::NotClaimed {
            reason: manager_daily_brief::LaborSavingsNotClaimedReason::MissingReviewableActionTrace
        }
    );
    assert_eq!(
        outcome.review_disposition(),
        manager_daily_brief::ReviewDisposition::CompletedWithMeasuredLaborSavings
    );
    assert!(outcome.records_feedback_without_external_mutation());
    assert!(
        outcome
            .blocked_actions()
            .contains(&manager_daily_brief::BlockedAction::ChangeStaffSchedule)
    );
}

#[test]
fn manager_daily_brief_completed_outcome_needs_matching_action_and_source_trace_before_claiming_savings()
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
            reason: manager_daily_brief::LaborSavingsNotClaimedReason::MissingReviewableActionTrace
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
            reason: manager_daily_brief::LaborSavingsNotClaimedReason::MissingActionSourceEvidence
        }
    );
}

#[test]
fn manager_daily_brief_wrong_source_deferred_and_suppressed_outcomes_do_not_claim_labor_savings() {
    for (outcome, expected_reason) in [
        (
            manager_daily_brief::FeedbackOutcome::Deferred,
            manager_daily_brief::LaborSavingsNotClaimedReason::ManagerDeferredReview,
        ),
        (
            manager_daily_brief::FeedbackOutcome::SuppressedByManager,
            manager_daily_brief::LaborSavingsNotClaimedReason::ManagerSuppressedAction,
        ),
        (
            manager_daily_brief::FeedbackOutcome::SourceFactWasWrong,
            manager_daily_brief::LaborSavingsNotClaimedReason::SourceFactWasWrong,
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
        assert_eq!(record.actual_minutes_saved(), 33);
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
        .completed_by(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("front-desk-erin").unwrap(),
        })
        .completed_at(DateTime::<Utc>::UNIX_EPOCH)
        .belongings_status(checkout_completion::BelongingsStatus::ReturnedToCustomer)
        .care_summary(checkout_completion::CareSummary::try_new("Clean checkout.").unwrap())
        .departure_notes_review(checkout_completion::DepartureNotesReview::StaffReviewed)
        .build()
}

fn open_staff_handoff() -> checkout_completion::StaffHandoff {
    checkout_completion::StaffHandoff::builder()
        .completed_by(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("front-desk-erin").unwrap(),
        })
        .completed_at(DateTime::<Utc>::UNIX_EPOCH)
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
    entities::LocationId(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0001))
}

fn customer_id() -> entities::CustomerId {
    entities::CustomerId(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0099))
}

fn reservation_id() -> entities::reservation::Id {
    entities::reservation::Id(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0042))
}

fn operating_day() -> operations::operating_day::Date {
    operations::operating_day::Date::try_new(NaiveDate::from_ymd_opt(2026, 6, 17).unwrap()).unwrap()
}
