use chrono::{TimeZone, Utc};
use domain::{entities, policy, strategic_ai_ops as ops};
use uuid::Uuid;

fn location_id() -> entities::LocationId {
    entities::LocationId(Uuid::from_u128(0x170))
}

fn customer_id() -> entities::CustomerId {
    entities::CustomerId(Uuid::from_u128(0xc0570))
}

fn pet_id() -> entities::PetId {
    entities::PetId(Uuid::from_u128(0x0d06))
}

#[test]
fn realtime_lead_model_tracks_sla_contact_attempts_consent_and_conversion_attribution() {
    let received_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 0, 0).unwrap();
    let due_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 5, 0).unwrap();

    let event = ops::lead_response::Event::builder()
        .id(ops::lead_response::EventId::try_new("missed-call-42").unwrap())
        .idempotency_key(
            ops::lead_response::IdempotencyKey::try_new("phone:loc170:+15551234567:20260812T1400")
                .unwrap(),
        )
        .location_id(location_id())
        .kind(ops::lead_response::EventKind::MissedCall)
        .received_at(received_at)
        .source_system(ops::source::System::Telephony)
        .customer_match(ops::identity::Match::Candidate {
            customer_id: customer_id(),
            confidence: ops::identity::Confidence::High,
        })
        .service_intent(entities::ServiceKind::Boarding)
        .build();

    let sla = ops::lead_response::ResponseSla::builder()
        .target(ops::lead_response::SlaTarget::FirstResponseWithinMinutes(
            ops::lead_response::Minutes::try_new(5).unwrap(),
        ))
        .received_at(received_at)
        .due_at(due_at)
        .status(ops::lead_response::SlaStatus::Open)
        .build();

    let attempt = ops::lead_response::ContactAttempt::builder()
        .attempted_at(Utc.with_ymd_and_hms(2026, 8, 12, 14, 2, 0).unwrap())
        .channel(ops::communication::Channel::Sms)
        .purpose(ops::communication::Purpose::TransactionalLeadResponse)
        .outcome(ops::lead_response::AttemptOutcome::DraftedForReview)
        .review_gate(policy::ReviewGate::CustomerMessageApproval)
        .build();

    let packet = ops::lead_response::ResponsePacket::builder()
        .event(event)
        .sla(sla)
        .consent(
            ops::communication::ConsentEvidence::builder()
                .channel(ops::communication::Channel::Sms)
                .purpose(ops::communication::Purpose::TransactionalLeadResponse)
                .status(ops::communication::ConsentStatus::Granted)
                .source(ops::source::System::Crm)
                .build(),
        )
        .attempts(vec![attempt])
        .attribution(
            ops::lead_response::ConversionAttribution::builder()
                .source(ops::lead_response::AttributionSource::MissedCall)
                .estimated_value_cents(ops::financial::MoneyCents::new(45_000))
                .build(),
        )
        .build();

    assert!(
        packet.first_response_sla_is_met_at(Utc.with_ymd_and_hms(2026, 8, 12, 14, 3, 0).unwrap())
    );
    assert!(packet.requires_customer_message_approval());
    assert!(packet.consent_allows_response(
        ops::communication::Channel::Sms,
        ops::communication::Purpose::TransactionalLeadResponse
    ));
    assert_eq!(
        packet.event().source_system(),
        ops::source::System::Telephony
    );
}

#[test]
fn lead_sla_rejects_pre_receipt_and_breached_response_times() {
    let received_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 0, 0).unwrap();
    let due_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 5, 0).unwrap();

    let open_sla = ops::lead_response::ResponseSla::builder()
        .target(ops::lead_response::SlaTarget::FirstResponseWithinMinutes(
            ops::lead_response::Minutes::try_new(5).unwrap(),
        ))
        .received_at(received_at)
        .due_at(due_at)
        .status(ops::lead_response::SlaStatus::Open)
        .build();

    assert!(!open_sla.is_met_at(Utc.with_ymd_and_hms(2026, 8, 12, 13, 59, 0).unwrap()));

    let breached_sla = ops::lead_response::ResponseSla::builder()
        .target(ops::lead_response::SlaTarget::FirstResponseWithinMinutes(
            ops::lead_response::Minutes::try_new(5).unwrap(),
        ))
        .received_at(received_at)
        .due_at(due_at)
        .status(ops::lead_response::SlaStatus::Breached)
        .build();

    assert!(!breached_sla.is_met_at(Utc.with_ymd_and_hms(2026, 8, 12, 14, 3, 0).unwrap()));
}

#[test]
fn crm_intelligence_model_separates_operations_personalization_from_marketing_segments() {
    let note = ops::crm::StructuredNote::builder()
        .id(ops::crm::NoteId::try_new("note-123").unwrap())
        .customer_id(customer_id())
        .pet_id(pet_id())
        .kind(ops::crm::NoteKind::PetHandlingPreference)
        .body(
            ops::crm::NoteBody::try_new("Bella prefers a quiet handoff and peanut-butter Kong.")
                .unwrap(),
        )
        .visibility(ops::access::VisibilityScope::OperationsOnly)
        .allowed_uses(vec![ops::access::AllowedUse::ServicePersonalization])
        .source(ops::crm::SignalSource::StaffObservation)
        .confidence(ops::identity::Confidence::Medium)
        .review_state(ops::crm::ReviewState::Accepted)
        .recorded_at(Utc.with_ymd_and_hms(2026, 8, 12, 15, 0, 0).unwrap())
        .build();

    let segment = ops::crm::SegmentMembership::builder()
        .customer_id(customer_id())
        .segment(ops::crm::Segment::ServiceRecoveryWatchlist)
        .basis(vec![ops::crm::SegmentBasis::ComplaintResolvedRecently])
        .visibility(ops::access::VisibilityScope::OperationsOnly)
        .marketing_allowed(false)
        .evidence_note_ids(vec![note.id().clone()])
        .build();

    assert!(note.can_support(ops::access::AllowedUse::ServicePersonalization));
    assert!(!note.can_support(ops::access::AllowedUse::MarketingCampaign));
    assert!(!segment.marketing_allowed());
    assert!(!format!("{note:?}").contains("Bella prefers"));
}

#[test]
fn capacity_labor_model_carries_time_bucket_constraints_shift_cost_and_reviewed_recommendation() {
    let bucket = ops::time::Window::new(
        Utc.with_ymd_and_hms(2026, 8, 12, 7, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 8, 12, 11, 0, 0).unwrap(),
    )
    .unwrap();

    let demand = ops::capacity::DemandUnit::builder()
        .location_id(location_id())
        .service(entities::ServiceKind::Boarding)
        .bucket(bucket)
        .quantity(ops::capacity::Quantity::try_new(18).unwrap())
        .labor_minutes_per_unit(ops::labor::Minutes::try_new(12).unwrap())
        .constraints(vec![ops::capacity::Constraint::CheckInCheckoutBottleneck])
        .build();

    let coverage = ops::labor::ScheduledCoverage::builder()
        .location_id(location_id())
        .bucket(bucket)
        .role(ops::labor::Role::FrontDesk)
        .scheduled_people(ops::labor::PeopleCount::try_new(1).unwrap())
        .scheduled_minutes(ops::labor::Minutes::try_new(240).unwrap())
        .loaded_cost_cents(ops::financial::MoneyCents::new(7_200))
        .build();

    let recommendation = ops::capacity::OptimizationRecommendation::builder()
        .objective(ops::capacity::OptimizationObjective::ReduceFrontDeskBottleneck)
        .demand(demand)
        .coverage(coverage)
        .action(ops::capacity::RecommendedAction::AddRoleCoverage {
            role: ops::labor::Role::FrontDesk,
            minutes: ops::labor::Minutes::try_new(60).unwrap(),
        })
        .expected_labor_delta_minutes(ops::labor::SignedMinutes::new(60))
        .review_gate(policy::ReviewGate::ManagerApproval)
        .build();

    assert_eq!(
        recommendation.review_gate(),
        policy::ReviewGate::ManagerApproval
    );
    assert!(recommendation.blocks_live_schedule_change());
}

#[test]
fn knowledge_assistant_model_requires_role_location_scope_fresh_citations_and_escalation() {
    let document = ops::knowledge::Document::builder()
        .id(ops::knowledge::DocumentId::try_new("sop-boarding-v1").unwrap())
        .title(ops::knowledge::Title::try_new("Boarding check-in SOP").unwrap())
        .kind(ops::knowledge::DocumentKind::Sop)
        .status(ops::knowledge::ApprovalStatus::Approved)
        .applicability(
            ops::knowledge::Applicability::builder()
                .locations(vec![location_id()])
                .services(vec![entities::ServiceKind::Boarding])
                .roles(vec![ops::access::ActorRole::FrontDesk])
                .build(),
        )
        .effective_at(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap())
        .review_due_at(Utc.with_ymd_and_hms(2027, 1, 1, 0, 0, 0).unwrap())
        .build();

    let context = ops::assistant::ActorContext::builder()
        .actor_id(ops::access::ActorId::try_new("front-desk-alice").unwrap())
        .role(ops::access::ActorRole::FrontDesk)
        .title(ops::access::Title::try_new("Front Desk Lead").unwrap())
        .location_id(location_id())
        .purpose(ops::assistant::Purpose::SopLookup)
        .allowed_uses(vec![ops::access::AllowedUse::InternalDecisionSupport])
        .build();

    let answer = ops::assistant::AnswerPacket::builder()
        .context(context)
        .answer(ops::assistant::AnswerText::try_new("Use the boarding check-in checklist and route vaccine ambiguity to manager review.").unwrap())
        .citations(vec![ops::knowledge::Citation::builder()
            .document_id(document.id().clone())
            .section(ops::knowledge::SectionRef::try_new("check-in.required-documents").unwrap())
            .build()])
        .confidence(ops::identity::Confidence::High)
        .build();

    assert!(document.applies_to(
        location_id(),
        entities::ServiceKind::Boarding,
        ops::access::ActorRole::FrontDesk
    ));
    assert!(answer.is_cited());
    assert!(!format!("{answer:?}").contains("boarding check-in checklist"));
}

#[test]
fn financial_insight_model_is_site_period_source_backed_and_blocks_price_discount_payment_changes()
{
    let period = ops::financial::SitePeriod::builder()
        .location_id(location_id())
        .start(Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap())
        .end(Utc.with_ymd_and_hms(2026, 8, 31, 23, 59, 59).unwrap())
        .build()
        .unwrap();

    let fact = ops::financial::RevenueFact::builder()
        .period(period)
        .service(entities::ServiceKind::Grooming)
        .gross_revenue_cents(ops::financial::MoneyCents::new(900_000))
        .discount_cents(ops::financial::MoneyCents::new(75_000))
        .refund_cents(ops::financial::MoneyCents::new(20_000))
        .labor_cost_cents(ops::financial::MoneyCents::new(310_000))
        .source_system(ops::source::System::FinanceAccounting)
        .build();

    let insight = ops::financial::Insight::builder()
        .fact(fact)
        .kind(ops::financial::InsightKind::DiscountLeakage)
        .expected_impact_cents(ops::financial::MoneyCents::new(50_000))
        .recommendation(ops::financial::Recommendation::ReviewDiscountPolicy)
        .review_gate(policy::ReviewGate::ManagerApproval)
        .build();

    assert_eq!(insight.net_revenue_cents().get(), 805_000);
    assert!(insight.blocks_financial_mutation());
}

#[test]
fn financial_money_cents_rejects_negative_amounts_at_constructor_and_serde_boundaries() {
    assert!(ops::financial::MoneyCents::try_new(0).is_ok());
    assert!(ops::financial::MoneyCents::try_new(-1).is_err());

    let rejected = serde_json::from_str::<ops::financial::MoneyCents>("-1");
    assert!(rejected.is_err());
}

#[test]
fn generalized_outcome_attribution_links_recommendations_to_measured_business_results() {
    let outcome = ops::outcome::Record::builder()
        .id(ops::outcome::Id::try_new("outcome-lead-42").unwrap())
        .workstream(ops::outcome::Workstream::LeadResponse)
        .location_id(location_id())
        .metric(ops::outcome::Metric::BookingConverted)
        .before_value(ops::outcome::MetricValue::Count(0))
        .after_value(ops::outcome::MetricValue::Count(1))
        .attribution(ops::outcome::Attribution::ReviewedAction)
        .source(ops::source::System::Crm)
        .recorded_at(Utc.with_ymd_and_hms(2026, 8, 12, 16, 0, 0).unwrap())
        .build();

    assert!(outcome.can_support_value_claim());
    assert_eq!(outcome.workstream(), ops::outcome::Workstream::LeadResponse);
}
