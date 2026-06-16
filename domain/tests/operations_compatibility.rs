use domain::{daily_brief, entities, lead, operations, reputation, staff, workflow};

#[test]
fn operations_call_sites_keep_service_owner_modules_visible() {
    let brief = daily_brief::Resort {
        operating_day: daily_brief::ResortOperatingDay {
            location_id: entities::LocationId(uuid::Uuid::nil()),
            date: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            snapshot_id: daily_brief::snapshot::Id::try_new("owner-brief").unwrap(),
        },
        sections: vec![daily_brief::Section::Labor(daily_brief::LaborSnapshot {
            scheduled_staff_count: daily_brief::ScheduledStaffCount::new(6),
            labor_risk: daily_brief::LaborRisk::Understaffed,
        })],
        recommended_actions: vec![daily_brief::Action::SuggestScheduleReview {
            risk: daily_brief::LaborRisk::Understaffed,
        }],
        risks: vec![daily_brief::Risk::LaborMismatch {
            risk: daily_brief::LaborRisk::Understaffed,
        }],
    };

    assert!(brief.has_manager_attention_required());

    let task = staff::Task::builder()
        .location_id(entities::LocationId(uuid::Uuid::nil()))
        .kind(staff::task::Kind::CustomerFollowUp {
            customer_id: entities::CustomerId(uuid::Uuid::nil()),
            reason: daily_brief::FollowUpReason::LeadNeedsResponse,
        })
        .title(workflow::task::Title::try_new("Call lead").unwrap())
        .status(staff::task::Status::Open)
        .priority(staff::task::Priority::Normal)
        .due_at(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
        .assignment(staff::task::Assignment::Role(staff::Role::FrontDesk))
        .source(staff::task::Source::Customer(entities::CustomerId(
            uuid::Uuid::nil(),
        )))
        .build();

    assert!(!task.requires_manager_attention());

    let lead = lead::Triage {
        customer_id: None,
        source: lead::Source::WebsiteForm,
        intent: lead::Intent::DaycareTrial,
        stage: lead::ConversionStage::New,
        requested_service: Some(entities::ServiceKind::DayPlay),
        next_action: lead::NextAction::DraftReply,
    };
    assert_eq!(lead.intent, lead::Intent::DaycareTrial);

    let reputation_signal = reputation::Signal {
        location_id: entities::LocationId(uuid::Uuid::nil()),
        platform: reputation::PlatformName::try_new("Google").unwrap(),
        review_id: reputation::Id::try_new("review-owner").unwrap(),
        sentiment: reputation::Sentiment::Mixed,
        themes: vec![reputation::Theme::StaffExperience],
        escalation: reputation::Escalation::ManagerReviewRequired,
    };
    assert_eq!(
        reputation_signal.escalation,
        reputation::Escalation::ManagerReviewRequired
    );

    let _operations_contract = operations::TechnologyEcosystem::builder()
        .core_portal(operations::service_core::OperatingSystem::Gingr)
        .data_access(vec![operations::DataAccessPattern::Api])
        .adjacent_systems(vec![operations::AdjacentSystem::Reviews])
        .build();
}
