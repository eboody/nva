#![allow(deprecated)]

use domain::{entities, operations, workflow};

#[test]
fn operations_reexports_cross_service_shared_contracts_for_legacy_call_sites() {
    let brief = operations::ResortDailyBrief {
        operating_day: operations::ResortOperatingDay {
            location_id: entities::LocationId(uuid::Uuid::nil()),
            date: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            snapshot_id: operations::SnapshotId::try_new("legacy-brief").unwrap(),
        },
        sections: vec![operations::DailyBriefSection::Labor(
            operations::LaborSnapshot {
                scheduled_staff_count: operations::ScheduledStaffCount::new(6),
                labor_risk: operations::LaborRisk::Understaffed,
            },
        )],
        recommended_actions: vec![operations::Action::SuggestScheduleReview {
            risk: operations::LaborRisk::Understaffed,
        }],
        risks: vec![operations::Risk::LaborMismatch {
            risk: operations::LaborRisk::Understaffed,
        }],
    };

    assert!(brief.has_manager_attention_required());

    let task = operations::StaffTask::builder()
        .location_id(entities::LocationId(uuid::Uuid::nil()))
        .kind(operations::StaffTaskKind::CustomerFollowUp {
            customer_id: entities::CustomerId(uuid::Uuid::nil()),
            reason: operations::FollowUpReason::LeadNeedsResponse,
        })
        .title(workflow::task::Title::try_new("Call lead").unwrap())
        .status(operations::StaffTaskStatus::Open)
        .priority(operations::StaffTaskPriority::Normal)
        .due_at(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
        .assignment(operations::StaffTaskAssignment::Role(
            operations::StaffRole::FrontDesk,
        ))
        .source(operations::StaffTaskSource::Customer(entities::CustomerId(
            uuid::Uuid::nil(),
        )))
        .build();

    assert!(!task.requires_manager_attention());

    let lead = operations::Lead {
        customer_id: None,
        source: operations::LeadSource::WebsiteForm,
        intent: operations::LeadIntent::DaycareTrial,
        stage: operations::LeadConversionStage::New,
        requested_service: Some(entities::ServiceKind::DayPlay),
        next_action: operations::LeadNextAction::DraftReply,
    };
    assert_eq!(lead.intent, operations::LeadIntent::DaycareTrial);

    let reputation_signal = operations::ReputationSignal {
        location_id: entities::LocationId(uuid::Uuid::nil()),
        platform: operations::ReviewPlatformName::try_new("Google").unwrap(),
        review_id: operations::ReviewId::try_new("review-legacy").unwrap(),
        sentiment: operations::ReviewSentiment::Mixed,
        themes: vec![operations::ReviewTheme::StaffExperience],
        escalation: operations::ReviewEscalation::ManagerReviewRequired,
    };
    assert_eq!(
        reputation_signal.escalation,
        operations::ReviewEscalation::ManagerReviewRequired
    );
}
