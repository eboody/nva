use domain::{boarding, daycare, entities, grooming, money, operations, retail, training};
use uuid::Uuid;

#[test]
fn boarding_contract_encodes_capacity_stay_payment_housekeeping_handoff_and_upsell_rules() {
    let contract = boarding::Contract::builder()
        .capacity(boarding::CapacityPlan::new(
            boarding::RoomInventory::try_new(48).unwrap(),
            boarding::RoomAvailability::Limited,
        ))
        .arrival_window(
            boarding::ServiceWindow::new(
                boarding::HourOfDay::try_new(7).unwrap(),
                boarding::HourOfDay::try_new(18).unwrap(),
            )
            .unwrap(),
        )
        .departure_window(
            boarding::ServiceWindow::new(
                boarding::HourOfDay::try_new(7).unwrap(),
                boarding::HourOfDay::try_new(12).unwrap(),
            )
            .unwrap(),
        )
        .minimum_stay(boarding::minimum_stay::Policy::new(
            boarding::StayNights::try_new(2).unwrap(),
            boarding::minimum_stay::Reason::HolidayPeak,
        ))
        .cancellation(boarding::cancellation::Policy::new(
            boarding::NoticeHours::try_new(48).unwrap(),
            boarding::cancellation::Penalty::ForfeitDeposit,
        ))
        .deposit(boarding::DepositRule::Required {
            amount: money::Money::new(
                money::MinorUnits::try_new(5_000).unwrap(),
                money::Currency::Usd,
            ),
        })
        .payment(boarding::PaymentTiming::DueAtCheckout)
        .housekeeping(boarding::housekeeping::Cadence::DailyRoomReset)
        .handoff(boarding::handoff::Requirement::ArrivalCareReview)
        .upsells(vec![
            boarding::Upsell::ExitBath,
            boarding::Upsell::TrainingSession,
        ])
        .build();

    assert_eq!(contract.capacity.room_inventory().get(), 48);
    assert!(contract.requires_deposit_collection());
    assert_eq!(contract.minimum_stay.nights().get(), 2);
    assert!(boarding::RoomInventory::try_new(0).is_err());
    assert!(boarding::StayNights::try_new(0).is_err());
    assert!(boarding::NoticeHours::try_new(0).is_err());
    assert!(boarding::HourOfDay::try_new(24).is_err());
    assert!(
        boarding::ServiceWindow::new(
            boarding::HourOfDay::try_new(18).unwrap(),
            boarding::HourOfDay::try_new(7).unwrap(),
        )
        .is_err()
    );
}

#[test]
fn boarding_capacity_policy_never_uses_cat_condo_for_dog_request() {
    let snapshot = boarding::capacity::Snapshot::new(vec![
        boarding::capacity::NightlySegmentSnapshot::from_counts(
            boarding::capacity::SegmentCounts::builder()
                .accommodation(boarding::accommodation::Kind::CatCondo)
                .total(boarding::capacity::RoomCount::try_new(4).unwrap())
                .occupied(boarding::capacity::RoomCount::try_new(0).unwrap())
                .build(),
        ),
    ])
    .unwrap();
    let request = boarding::capacity::Request::new(
        entities::LocationId(Uuid::nil()),
        entities::Species::Dog,
        boarding::accommodation::Preference::Specific(boarding::accommodation::Kind::CatCondo),
    );

    let decision = boarding::capacity::Policy.evaluate(&request, &snapshot);

    assert!(matches!(
        decision,
        boarding::capacity::Decision::Deny {
            reason: boarding::capacity::DenialReason::SpeciesAccommodationMismatch,
            ..
        }
    ));
    assert_eq!(
        decision.required_review_gate(),
        Some(domain::policy::ReviewGate::ManagerApproval)
    );
}

#[test]
fn boarding_deposit_policy_requires_due_at_booking_collection_before_confirmation() {
    let rule = boarding::DepositRule::Required {
        amount: money::Money::new(
            money::MinorUnits::try_new(2_500).unwrap(),
            money::Currency::Usd,
        ),
    };
    let decision = boarding::deposit::Policy::new(rule, boarding::PaymentTiming::DueAtBooking)
        .readiness_for_confirmation(None);

    assert!(matches!(
        decision,
        boarding::deposit::ConfirmationReadiness::Blocked {
            blocker: boarding::deposit::Blocker::DepositRequired,
            review_gate: domain::policy::ReviewGate::RefundOrDepositException,
        }
    ));
}

#[test]
fn boarding_deposit_policy_treats_paid_deposit_with_reference_as_satisfied() {
    let amount = money::Money::new(
        money::MinorUnits::try_new(2_500).unwrap(),
        money::Currency::Usd,
    );
    let rule = boarding::DepositRule::Required {
        amount: amount.clone(),
    };
    let paid = domain::payment::Deposit::paid(
        amount,
        domain::payment::Reference::try_new("gingr-pay-123").unwrap(),
    );

    let decision = boarding::deposit::Policy::new(rule, boarding::PaymentTiming::DueAtBooking)
        .readiness_for_confirmation(Some(&paid));

    assert_eq!(decision, boarding::deposit::ConfirmationReadiness::Ready);
}

#[test]
fn boarding_care_policy_flags_missing_feeding_instruction_for_staff_review() {
    let care_profile = entities::CareProfile::default();
    let plan = boarding::care::Policy.plan_for_pet(entities::PetId(Uuid::nil()), &care_profile);

    assert_eq!(plan.readiness(), boarding::care::Readiness::Blocked);
    assert!(plan.gates().contains(&boarding::care::ReviewGate::new(
        boarding::care::GateReason::MissingFeedingInstruction,
        domain::policy::ReviewGate::MedicalDocumentReview,
    )));
}

#[test]
fn boarding_upsell_policy_recommends_exit_bath_only_when_eligible_and_not_care_unsafe() {
    let mut care_profile = entities::CareProfile::default();
    care_profile
        .allergies
        .push(domain::care::AllergyName::try_new("sensitive shampoo").unwrap());

    let recommendation = boarding::upsell::Policy.evaluate_exit_bath(
        entities::reservation::Id(Uuid::nil()),
        entities::PetId(Uuid::nil()),
        &care_profile,
    );

    assert!(matches!(
        recommendation.eligibility,
        boarding::upsell::Eligibility::NeedsStaffReview {
            gate: domain::policy::ReviewGate::MedicalDocumentReview,
            reason: boarding::upsell::ReviewReason::CareSafetyAmbiguity,
        }
    ));
    assert_eq!(
        recommendation.customer_offer_gate(),
        Some(domain::policy::ReviewGate::CustomerMessageApproval)
    );
}

#[test]
fn daycare_contract_encodes_attendance_packages_ratios_groups_incidents_and_eligibility() {
    let contract = daycare::Contract::builder()
        .attendance(daycare::AttendancePolicy::ReservationRequired)
        .package(daycare::PackagePolicy::PrepaidPasses {
            visits: daycare::PackageVisits::try_new(10).unwrap(),
        })
        .ratio(daycare::StaffPetRatio::new(
            daycare::StaffCount::try_new(1).unwrap(),
            daycare::PetCount::try_new(12).unwrap(),
        ))
        .group_assignment(daycare::GroupAssignmentRule::TemperamentAndSizeMatched)
        .incident(daycare::incident::Policy::ManagerReviewAndCustomerNotice)
        .eligibility(vec![
            daycare::EligibilityRequirement::TemperamentAssessment,
            daycare::EligibilityRequirement::VaccinesCurrent,
        ])
        .build();

    assert_eq!(contract.ratio.pets_per_staff().get(), 12);
    assert!(contract.requires_staff_review_before_group_play());
    assert!(daycare::PackageVisits::try_new(0).is_err());
    assert!(daycare::StaffCount::try_new(0).is_err());
    assert!(daycare::PetCount::try_new(0).is_err());
}

#[test]
fn daycare_service_variants_preserve_group_boarding_plus_room_and_cat_care_modes() {
    assert_eq!(
        daycare::ServiceVariant::AllDayPlay.care_mode(),
        daycare::CareMode::DogGroupPlay
    );
    assert_eq!(
        daycare::ServiceVariant::DayBoarding.care_mode(),
        daycare::CareMode::DogIndividualDayBoarding
    );
    assert_eq!(
        daycare::ServiceVariant::DayPlayPlusRoom.care_mode(),
        daycare::CareMode::DogHybridPlayAndRoom
    );
    assert_eq!(
        daycare::ServiceVariant::CatIndividualPlaytime.care_mode(),
        daycare::CareMode::CatIndividualEnrichment
    );
}

#[test]
fn daycare_group_play_eligibility_routes_intact_dogs_to_behavior_review_not_ready() {
    let evidence = daycare::eligibility::Evidence::builder()
        .pet_id(entities::PetId(Uuid::nil()))
        .species(entities::Species::Dog)
        .service(daycare::ServiceVariant::AllDayPlay)
        .temperament(daycare::eligibility::TemperamentAssessmentFreshness::Current)
        .vaccines(daycare::eligibility::VaccineReadiness::Current)
        .spay_neuter(entities::SpayNeuterStatus::Intact)
        .incident(daycare::incident::Restriction::None)
        .staff_coverage(daycare::coverage::Decision::Sufficient)
        .build();

    let decision = daycare::eligibility::GroupPlayPolicy.evaluate(&evidence);

    assert!(matches!(
        decision,
        daycare::eligibility::GroupPlayDecision::NeedsStaffReview {
            reason: daycare::eligibility::ReviewReason::SpayNeuterStatusRequiresReview,
            gate: domain::policy::ReviewGate::BehaviorReview,
        }
    ));
}

#[test]
fn daycare_staff_coverage_policy_rejects_rosters_that_exceed_contract_ratio() {
    let contract_ratio = daycare::StaffPetRatio::new(
        daycare::StaffCount::try_new(1).unwrap(),
        daycare::PetCount::try_new(8).unwrap(),
    );
    let roster = daycare::coverage::RosterSnapshot::new(
        daycare::StaffCount::try_new(2).unwrap(),
        daycare::PetCount::try_new(17).unwrap(),
    );

    let decision = daycare::coverage::Policy.evaluate(&roster, contract_ratio);

    assert_eq!(
        decision,
        daycare::coverage::Decision::Insufficient {
            reason: daycare::coverage::InsufficiencyReason::RatioExceeded,
            gate: domain::policy::ReviewGate::ManagerApproval,
        }
    );
}

#[test]
fn daycare_assignment_requires_group_play_eligibility_and_staff_coverage() {
    let request = daycare::assignment::Request::builder()
        .pet_id(entities::PetId(Uuid::nil()))
        .service(daycare::ServiceVariant::HalfDayPlay)
        .eligibility(daycare::eligibility::GroupPlayDecision::Eligible {
            basis: daycare::eligibility::EligibleBasis::CurrentEvidence,
        })
        .coverage(daycare::coverage::Decision::Insufficient {
            reason: daycare::coverage::InsufficiencyReason::RatioExceeded,
            gate: domain::policy::ReviewGate::ManagerApproval,
        })
        .playgroup(daycare::assignment::playgroup_id::Id::try_new(" small-dogs-am ").unwrap())
        .build();

    let decision = daycare::assignment::Service.assign(request);

    assert_eq!(
        decision,
        daycare::assignment::Decision::Waitlist {
            reason: daycare::assignment::WaitlistReason::StaffCoverageInsufficient,
            gate: domain::policy::ReviewGate::ManagerApproval,
        }
    );
}

#[test]
fn daycare_incident_policy_suspends_group_play_for_safety_incidents_until_manager_review() {
    let disposition = daycare::incident::Classifier.classify(
        entities::PetId(Uuid::nil()),
        daycare::incident::Severity::SuspendGroupPlay,
    );

    assert_eq!(
        disposition.restriction(),
        daycare::incident::Restriction::SuspendedPendingManagerReview {
            pet_id: entities::PetId(Uuid::nil()),
        }
    );
    assert_eq!(
        disposition.required_gate(),
        Some(domain::policy::ReviewGate::ManagerApproval)
    );
}

#[test]
fn daycare_recurring_attendance_materializes_only_requested_days_and_preserves_exceptions() {
    let recurrence = daycare::attendance::Recurrence::new(
        daycare::attendance::DateRange::new(
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        )
        .unwrap(),
        daycare::attendance::Days::try_new(vec![
            chrono::Weekday::Mon,
            chrono::Weekday::Wed,
            chrono::Weekday::Fri,
        ])
        .unwrap(),
    );

    let dates = daycare::attendance::Materializer.materialize(
        &recurrence,
        &[chrono::NaiveDate::from_ymd_opt(2026, 6, 3).unwrap()],
    );

    assert_eq!(
        dates,
        vec![
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
        ]
    );
}

#[test]
fn daycare_package_opportunity_never_overrides_safety_or_payment_review() {
    let evidence = daycare::package_opportunity::Evidence::builder()
        .customer_id(entities::CustomerId(Uuid::nil()))
        .pet_id(entities::PetId(Uuid::nil()))
        .attendance_visits(daycare::package_opportunity::AttendanceVisitCount::new(12))
        .eligibility(daycare::package_opportunity::CareEligibility::BlockedBySafetyReview)
        .package_state(daycare::package_opportunity::PackageState::PayPerVisit)
        .payment_state(daycare::package_opportunity::PaymentState::Current)
        .build();

    let decision = daycare::package_opportunity::Policy.classify(&evidence);

    assert_eq!(
        decision,
        daycare::package_opportunity::Decision::Suppressed {
            reason: daycare::package_opportunity::SuppressionReason::SafetyOrCareReviewRequired,
            gate: domain::policy::ReviewGate::BehaviorReview,
        }
    );
}

#[test]
fn daycare_front_desk_throughput_routes_ready_pets_to_fast_lane_without_customer_send() {
    let context = daycare::front_desk::ReadinessContext::builder()
        .reservation_id(entities::reservation::Id(Uuid::nil()))
        .service(daycare::ServiceVariant::AllDayPlay)
        .eligibility(daycare::front_desk::EligibilityReadiness::GroupPlay(
            daycare::eligibility::GroupPlayDecision::Eligible {
                basis: daycare::eligibility::EligibleBasis::CurrentEvidence,
            },
        ))
        .coverage(daycare::coverage::Decision::Sufficient)
        .care(daycare::front_desk::CareReadiness::Ready)
        .package(daycare::front_desk::PackageReadiness::Ready)
        .customer_message(daycare::front_desk::CustomerMessageReadiness::NoMessageNeeded)
        .build();

    let decision = daycare::front_desk::ThroughputPolicy.evaluate(&context);
    let ticket = daycare::front_desk::QueueTicket::new(
        daycare::front_desk::QueuePosition::try_new(1).unwrap(),
        decision.clone(),
    );

    assert_eq!(
        decision,
        daycare::front_desk::ReadinessDecision::ReadyToCheckIn
    );
    assert_eq!(ticket.lane(), daycare::front_desk::QueueLane::FastLane);
    assert_eq!(decision.customer_message_gate(), None);
}

#[test]
fn grooming_contract_encodes_calendar_estimates_no_shows_rebooking_reminders_and_history() {
    let estimate = grooming::breed_coat::TimeEstimate::new(
        grooming::breed_coat::BreedCategory::Doodle,
        grooming::breed_coat::CoatCondition::Matted,
        grooming::AppointmentMinutes::try_new(180).unwrap(),
    );
    let contract = grooming::Contract::builder()
        .calendar(grooming::calendar::Policy::GroomerSpecific)
        .time_estimates(vec![estimate])
        .no_show(grooming::no_show::Rule::RequireDepositForRebooking)
        .rebooking(grooming::rebooking::Cadence::EveryWeeks(
            grooming::rebooking::CadenceWeeks::try_new(6).unwrap(),
        ))
        .reminders(vec![
            grooming::reminder::Rule::FortyEightHoursBefore,
            grooming::reminder::Rule::MorningOf,
        ])
        .history(grooming::HistoryRequirement::KeepStyleNotesAndPhotos)
        .build();

    assert_eq!(contract.time_estimates[0].minutes().get(), 180);
    assert!(contract.requires_deposit_after_no_show());
    assert!(grooming::AppointmentMinutes::try_new(0).is_err());
    assert!(grooming::rebooking::CadenceWeeks::try_new(0).is_err());
}

#[test]
fn grooming_duration_estimate_requires_positive_minutes_and_explains_basis() {
    let estimate = grooming::EstimationPolicy.estimate(
        grooming::EstimationRequest::builder()
            .pet_id(entities::PetId(Uuid::nil()))
            .service(grooming::Service::FullGroom)
            .breed(grooming::breed_coat::BreedCategory::Doodle)
            .coat(grooming::breed_coat::CoatCondition::Maintained)
            .build(),
        &[],
        &grooming::Contract::standard_petsuites(),
    );

    assert_eq!(estimate.minutes().get(), 180);
    assert_eq!(estimate.basis(), grooming::EstimateBasis::BreedCoatPolicy);
    assert_eq!(estimate.review(), grooming::ReviewRequirement::None);
    assert_eq!(estimate.calendar_execution_gate(), None);
}

#[test]
fn matted_or_sensitive_coat_estimate_requires_staff_review_before_auto_scheduling() {
    let estimate = grooming::EstimationPolicy.estimate(
        grooming::EstimationRequest::builder()
            .pet_id(entities::PetId(Uuid::nil()))
            .service(grooming::Service::FullGroom)
            .breed(grooming::breed_coat::BreedCategory::Doodle)
            .coat(grooming::breed_coat::CoatCondition::Matted)
            .build(),
        &[],
        &grooming::Contract::standard_petsuites(),
    );

    assert_eq!(
        estimate.review(),
        grooming::ReviewRequirement::GroomerReview
    );
    assert_eq!(
        estimate.calendar_execution_gate(),
        Some(domain::policy::ReviewGate::ManagerApproval)
    );
}

#[test]
fn grooming_rebooking_cadence_accepts_two_to_eight_week_ordinary_window() {
    assert!(grooming::rebooking::OrdinaryCadenceWeeks::try_new(2).is_ok());
    assert!(grooming::rebooking::OrdinaryCadenceWeeks::try_new(8).is_ok());
    assert!(grooming::rebooking::OrdinaryCadenceWeeks::try_new(1).is_err());
    assert!(grooming::rebooking::OrdinaryCadenceWeeks::try_new(9).is_err());
}

#[test]
fn grooming_no_show_policy_requires_deposit_or_manager_review_for_repeat_no_show() {
    let decision =
        grooming::no_show::Policy::new(grooming::no_show::Rule::RequireDepositForRebooking)
            .evaluate(
                entities::CustomerId(Uuid::nil()),
                entities::PetId(Uuid::nil()),
                grooming::no_show::History::new(
                    grooming::no_show::Count::try_new(2).unwrap(),
                    grooming::no_show::LateCancelCount::try_new(1).unwrap(),
                ),
            );

    assert_eq!(
        decision,
        grooming::no_show::Decision::DepositRequired {
            gate: domain::policy::ReviewGate::RefundOrDepositException,
        }
    );
}

#[test]
fn grooming_rebooking_policy_marks_pet_overdue_from_last_service_history_and_cadence() {
    let history_entry = grooming::history::ServiceHistoryEntry::builder()
        .pet_id(entities::PetId(Uuid::nil()))
        .location_id(entities::LocationId(Uuid::nil()))
        .service(grooming::Service::FullGroom)
        .completed_on(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
        .outcome(grooming::history::ServiceOutcome::Completed)
        .approval(grooming::history::ApprovalState::ApprovedByGroomer {
            groomer_id: entities::StaffId::try_new("groomer-1").unwrap(),
        })
        .build();

    let recommendation = grooming::rebooking::Policy.recommend_from_history(
        entities::PetId(Uuid::nil()),
        &[history_entry],
        grooming::rebooking::Cadence::EveryWeeks(
            grooming::rebooking::CadenceWeeks::try_new(6).unwrap(),
        ),
        chrono::NaiveDate::from_ymd_opt(2026, 2, 20).unwrap(),
    );

    assert_eq!(recommendation.status, grooming::rebooking::Status::Overdue);
    assert_eq!(
        recommendation.rationale,
        grooming::rebooking::Rationale::LastCompletedServiceCadence
    );
}

#[test]
fn grooming_reminder_plan_requires_customer_consent_before_member_facing_send() {
    let plan = grooming::reminder::Policy.plan(
        entities::CustomerId(Uuid::nil()),
        grooming::reminder::Kind::RebookingDue,
        grooming::reminder::Consent::NotGranted,
    );

    assert_eq!(
        plan.send_boundary(),
        grooming::reminder::SendBoundary::SuppressedUntilConsent
    );
    assert_eq!(plan.customer_message_gate(), None);
}

#[test]
fn grooming_history_entry_separates_style_notes_from_care_or_medical_handling_refs() {
    let entry = grooming::history::ServiceHistoryEntry::builder()
        .pet_id(entities::PetId(Uuid::nil()))
        .location_id(entities::LocationId(Uuid::nil()))
        .service(grooming::Service::MiniGroom)
        .completed_on(chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap())
        .outcome(grooming::history::ServiceOutcome::Completed)
        .approval(grooming::history::ApprovalState::ReviewRequired {
            gate: domain::policy::ReviewGate::MedicalDocumentReview,
        })
        .style_notes(vec![
            grooming::history::style_note::StyleNote::try_new(" teddy bear face ").unwrap(),
        ])
        .care_refs(vec![grooming::history::CareReference::SensitiveSkinProduct])
        .build();

    assert_eq!(
        entry.style_notes()[0].clone().into_inner(),
        "teddy bear face"
    );
    assert_eq!(
        entry.care_refs(),
        &[grooming::history::CareReference::SensitiveSkinProduct]
    );
    assert!(entry.requires_review());
}

#[test]
fn training_contract_encodes_program_curriculum_progress_outcomes_availability_packages_and_follow_up()
 {
    let contract = training::Contract::builder()
        .program_duration(training::program::Duration::Weeks(
            training::program::DurationWeeks::try_new(3).unwrap(),
        ))
        .curriculum(vec![
            training::curriculum::Unit::LooseLeashWalking,
            training::curriculum::Unit::Recall,
        ])
        .progress(training::ProgressTracking::SessionNotesAndMilestones)
        .outcomes(vec![training::Outcome::CanineGoodCitizenReadiness])
        .trainer_availability(training::trainer::Availability::NamedTrainerRequired)
        .package(training::package::Policy::MultiSessionPackage {
            sessions: training::SessionCount::try_new(6).unwrap(),
        })
        .follow_up(training::FollowUpCadence::AfterProgramCompletion)
        .build();

    assert!(contract.requires_named_trainer());
    assert!(contract.has_outcome(&training::Outcome::CanineGoodCitizenReadiness));
    assert!(training::program::DurationWeeks::try_new(0).is_err());
    assert!(training::SessionCount::try_new(0).is_err());
}

#[test]
fn trainer_availability_waitlists_when_named_trainer_has_no_capacity() {
    let request = training::availability::Request::builder()
        .enrollment_id(training::enrollment::Id::try_new("enroll-123").unwrap())
        .pet_id(entities::PetId(Uuid::nil()))
        .program(training::Program::PrivateLesson)
        .requirement(training::trainer::Requirement::NamedTrainer {
            trainer_id: entities::StaffId::try_new("trainer-7").unwrap(),
        })
        .capacity(training::availability::CapacityDecision::Unavailable)
        .readiness(training::enrollment::Readiness::Ready)
        .build();

    let decision = training::availability::Policy.evaluate(&request);

    assert_eq!(
        decision,
        training::availability::Decision::Waitlist {
            reason: training::availability::WaitlistReason::RequestedTrainerUnavailable,
            gate: domain::policy::ReviewGate::ManagerApproval,
        }
    );
    assert_eq!(
        decision.provider_mutation_gate(),
        Some(domain::policy::ReviewGate::ManagerApproval)
    );
}

#[test]
fn progress_report_cannot_be_parent_facing_until_approved_even_when_evidence_exists() {
    let report = training::progress::Report::builder()
        .report_id(training::ProgressReportId::try_new("progress-1").unwrap())
        .enrollment_id(training::enrollment::Id::try_new("enroll-123").unwrap())
        .session_ref(training::SessionRef::try_new("session-4").unwrap())
        .evidence(vec![training::ProgressEvidence::TrainerNote {
            evidence_id: training::EvidenceId::try_new("evidence-1").unwrap(),
            note: training::ProgressNote::try_new(" recall improved with long line ").unwrap(),
        }])
        .milestones(vec![training::curriculum::Progress::new(
            training::curriculum::milestone::Id::try_new("recall").unwrap(),
            training::curriculum::milestone::Status::Introduced,
        )])
        .approval(training::ApprovalState::Draft)
        .build()
        .unwrap();

    assert_eq!(
        report.parent_facing_boundary(),
        training::MemberFacingBoundary::DraftRequiresApproval {
            gate: domain::policy::ReviewGate::CustomerMessageApproval,
        }
    );
    assert!(report.has_evidence());
    assert!(matches!(
        report.first_evidence(),
        training::ProgressEvidence::TrainerNote { .. }
    ));
}

#[test]
fn progress_evidence_set_exposes_a_total_first_evidence_accessor() {
    let evidence =
        training::progress::EvidenceSet::try_new(vec![training::ProgressEvidence::TrainerNote {
            evidence_id: training::EvidenceId::try_new("evidence-1").unwrap(),
            note: training::ProgressNote::try_new(" recall improved with long line ").unwrap(),
        }])
        .unwrap();

    assert!(matches!(
        evidence.first(),
        training::ProgressEvidence::TrainerNote { .. }
    ));
}

#[test]
fn progress_report_builder_returns_typed_errors_for_missing_fields_and_evidence() {
    assert_eq!(
        training::progress::Report::builder().build(),
        Err(training::Error::ProgressReportIdRequired)
    );

    let missing_evidence = training::progress::Report::builder()
        .report_id(training::ProgressReportId::try_new("progress-missing-evidence").unwrap())
        .enrollment_id(training::enrollment::Id::try_new("enroll-123").unwrap())
        .session_ref(training::SessionRef::try_new("session-4").unwrap())
        .build();

    assert_eq!(
        missing_evidence,
        Err(training::Error::ProgressEvidenceRequired)
    );
}

#[test]
fn outcome_documentation_builder_returns_typed_errors_for_missing_fields_and_claims() {
    assert_eq!(
        training::outcome::Documentation::builder().build(),
        Err(training::Error::OutcomeDocumentationIdRequired)
    );

    let missing_claims = training::outcome::Documentation::builder()
        .documentation_id(training::OutcomeDocumentationId::try_new("outcome-empty").unwrap())
        .enrollment_id(training::enrollment::Id::try_new("enroll-123").unwrap())
        .pet_id(entities::PetId(Uuid::nil()))
        .location_id(entities::LocationId(Uuid::nil()))
        .build();

    assert_eq!(missing_claims, Err(training::Error::OutcomeClaimRequired));
}

#[test]
fn achieved_outcome_claim_requires_evidence_before_documentation_can_be_member_facing() {
    let rejected = training::outcome::Claim::from_evidence(training::outcome::ClaimEvidence {
        outcome: training::Outcome::CanineGoodCitizenReadiness,
        status: training::outcome::ClaimStatus::Achieved,
        evidence: vec![],
        milestones: vec![training::curriculum::milestone::Id::try_new("cgc-readiness").unwrap()],
    });

    assert_eq!(rejected, Err(training::Error::OutcomeEvidenceRequired));

    let claim = training::outcome::Claim::from_evidence(training::outcome::ClaimEvidence {
        outcome: training::Outcome::CanineGoodCitizenReadiness,
        status: training::outcome::ClaimStatus::Readiness,
        evidence: vec![training::EvidenceId::try_new("rubric-1").unwrap()],
        milestones: vec![training::curriculum::milestone::Id::try_new("cgc-readiness").unwrap()],
    })
    .unwrap();
    let documentation = training::outcome::Documentation::builder()
        .documentation_id(training::OutcomeDocumentationId::try_new("outcome-1").unwrap())
        .enrollment_id(training::enrollment::Id::try_new("enroll-123").unwrap())
        .pet_id(entities::PetId(Uuid::nil()))
        .location_id(entities::LocationId(Uuid::nil()))
        .claims(vec![claim])
        .review(training::OutcomeReviewState::TrainerApproved {
            trainer_id: entities::StaffId::try_new("trainer-7").unwrap(),
        })
        .build()
        .unwrap();

    assert_eq!(
        documentation.member_facing_boundary(),
        training::MemberFacingBoundary::DraftRequiresApproval {
            gate: domain::policy::ReviewGate::CustomerMessageApproval,
        }
    );
}

#[test]
fn training_package_ledger_exposes_remaining_sessions_without_callers_recomputing_counts() {
    let package_id = training::package::Id::try_new("pkg-1").unwrap();
    let ledger = training::package::Ledger::open(training::package::OpeningLedger {
        package_id: package_id.clone(),
        customer_id: entities::CustomerId(Uuid::nil()),
        pet_id: entities::PetId(Uuid::nil()),
        policy: training::package::Policy::MultiSessionPackage {
            sessions: training::SessionCount::try_new(4).unwrap(),
        },
        entries: vec![
            training::package::LedgerEntry::Reserved {
                session_id: training::SessionId::try_new("session-1").unwrap(),
            },
            training::package::LedgerEntry::Consumed {
                session_id: training::SessionId::try_new("session-2").unwrap(),
            },
        ],
    })
    .unwrap();

    assert_eq!(ledger.balance().remaining().get(), 2);
    assert_eq!(
        training::package::UsagePolicy.decide_usage(&ledger),
        training::package::UsageDecision::ReserveNextSession {
            package_id,
            remaining_after_reservation: training::SessionBalance::new(1),
        }
    );
}

#[test]
fn follow_up_policy_creates_due_plan_for_after_each_session_with_progress_homework() {
    let plan = training::follow_up::Policy.plan(
        training::follow_up::Trigger::SessionCompleted {
            session_id: training::SessionId::try_new("session-4").unwrap(),
        },
        training::FollowUpCadence::AfterEachSession,
        training::follow_up::EvidenceReadiness::ProgressAndHomeworkReady,
    );

    assert_eq!(
        plan.state(),
        training::follow_up::State::DraftRequiresApproval {
            gate: domain::policy::ReviewGate::CustomerMessageApproval,
        }
    );
    assert_eq!(plan.purpose(), training::follow_up::Purpose::ProgressUpdate);
}

#[test]
fn retail_contract_encodes_product_pos_inventory_recommendation_and_reorder_rules() {
    let contract = retail::Contract::builder()
        .product(retail::Product::new(
            retail::Sku::try_new("  CALM-CARE-30  ").unwrap(),
            retail::product::Category::Supplement,
        ))
        .pos(retail::pos::Policy::IntegratedWithReservationCheckout)
        .inventory(retail::inventory::Policy::Tracked {
            on_hand: retail::inventory::UnitCount::try_new(8).unwrap(),
            reorder_at: retail::inventory::UnitCount::try_new(10).unwrap(),
        })
        .recommendation(retail::recommendation::Rule::AnxietySupportAfterBoarding)
        .reorder(retail::reorder::Policy::AutoCreateManagerTask)
        .build();

    assert_eq!(contract.product.sku().clone().into_inner(), "CALM-CARE-30");
    assert!(contract.should_reorder());
    assert!(retail::Sku::try_new("   ").is_err());
    assert!(retail::inventory::UnitCount::try_new(0).is_err());
}

#[test]
fn retail_inventory_position_derives_available_units_and_rejects_over_reserved_stock() {
    let position = retail::inventory::Position::record(retail::inventory::Stock {
        location_id: entities::LocationId(Uuid::nil()),
        sku: retail::Sku::try_new("CALM-CARE-30").unwrap(),
        on_hand: retail::inventory::OnHandUnits::new(8),
        reserved: retail::inventory::ReservedUnits::new(3),
        reorder_at: retail::inventory::UnitCount::try_new(10).unwrap(),
    })
    .unwrap();

    assert_eq!(
        position.available_units(),
        retail::inventory::AvailableUnits::new(5)
    );
    assert_eq!(
        retail::inventory::Position::record(retail::inventory::Stock {
            location_id: entities::LocationId(Uuid::nil()),
            sku: retail::Sku::try_new("CALM-CARE-30").unwrap(),
            on_hand: retail::inventory::OnHandUnits::new(2),
            reserved: retail::inventory::ReservedUnits::new(3),
            reorder_at: retail::inventory::UnitCount::try_new(10).unwrap(),
        }),
        Err(retail::Error::ReservedUnitsExceedOnHand)
    );
}

#[test]
fn retail_reorder_policy_routes_below_threshold_stock_to_reviewable_staff_task() {
    let sku = retail::Sku::try_new("CALM-CARE-30").unwrap();
    let position = retail::inventory::Position::record(retail::inventory::Stock {
        location_id: entities::LocationId(Uuid::nil()),
        sku: sku.clone(),
        on_hand: retail::inventory::OnHandUnits::new(4),
        reserved: retail::inventory::ReservedUnits::new(0),
        reorder_at: retail::inventory::UnitCount::try_new(10).unwrap(),
    })
    .unwrap();

    let decision = retail::reorder::Policy::AutoCreateManagerTask.evaluate(&position);

    assert_eq!(
        decision,
        retail::reorder::Decision::CreateStaffTask {
            location_id: entities::LocationId(Uuid::nil()),
            sku,
            reason: retail::reorder::Reason::AtOrBelowThreshold,
            gate: domain::policy::ReviewGate::ManagerApproval,
        }
    );
}

#[test]
fn retail_pos_policy_requires_manager_approval_for_comps_discounts_and_refunds() {
    let offering = retail::LocationOffering::builder()
        .location_id(entities::LocationId(Uuid::nil()))
        .product(retail::Product::new(
            retail::Sku::try_new("CALM-CARE-30").unwrap(),
            retail::product::Category::Supplement,
        ))
        .status(retail::OfferingStatus::Active)
        .usage(retail::product::Usage::CustomerSellable)
        .pos(retail::pos::Policy::IntegratedWithReservationCheckout)
        .inventory(retail::inventory::Policy::Tracked {
            on_hand: retail::inventory::UnitCount::try_new(5).unwrap(),
            reorder_at: retail::inventory::UnitCount::try_new(2).unwrap(),
        })
        .reorder(retail::reorder::Policy::ManualReview)
        .build();
    let request = retail::pos::Request::builder()
        .offering(offering)
        .quantity(retail::pos::Quantity::try_new(1).unwrap())
        .source(retail::pos::Source::ReservationCheckout {
            reservation_id: entities::reservation::Id(Uuid::nil()),
        })
        .price_adjustment(retail::pos::PriceAdjustment::ManagerComp {
            reason: retail::pos::PriceExceptionReason::ComplaintRecovery,
        })
        .build();

    let decision = retail::pos::Policy::IntegratedWithReservationCheckout.evaluate(&request);

    assert_eq!(
        decision,
        retail::pos::Decision::ReviewRequired {
            reason: retail::pos::ReviewReason::PriceException,
            gate: domain::policy::ReviewGate::ManagerApproval,
        }
    );
}

#[test]
fn retail_recommendation_policy_routes_care_sensitive_supplement_candidates_to_staff_review() {
    let candidate = retail::recommendation::Candidate::builder()
        .customer_id(entities::CustomerId(Uuid::nil()))
        .pet_id(entities::PetId(Uuid::nil()))
        .location_id(entities::LocationId(Uuid::nil()))
        .product(retail::Product::new(
            retail::Sku::try_new("CALM-CARE-30").unwrap(),
            retail::product::Category::Supplement,
        ))
        .reason(retail::recommendation::Reason::AnxietyOrStressSupport)
        .rationale(
            retail::recommendation::rationale::Text::try_new(
                "staff noted anxious boarding transition",
            )
            .unwrap(),
        )
        .care_sensitivity(retail::recommendation::CareSensitivity::SupplementOrDietReviewRequired)
        .inventory(retail::inventory::Availability::Available)
        .customer_preference(retail::recommendation::Preference::AllowsRetailRecommendations)
        .build();

    let decision = retail::recommendation::Policy.evaluate(&candidate);

    assert_eq!(
        decision,
        retail::recommendation::Decision::StaffReviewRequired {
            reason: retail::recommendation::ReviewReason::CareSensitiveProduct,
            gate: domain::policy::ReviewGate::MedicalDocumentReview,
        }
    );
}

#[test]
fn retail_customer_copy_policy_forbids_medical_claims_in_customer_drafts() {
    let copy = retail::recommendation::customer_copy::SafeCopy::try_new(
        "Virbac CalmCare treats anxiety for boarding dogs",
    )
    .unwrap();

    let decision = retail::recommendation::customer_copy::Policy.evaluate(&copy);

    assert_eq!(
        decision,
        retail::recommendation::customer_copy::Decision::Rejected {
            reason: retail::recommendation::customer_copy::RejectionReason::MedicalClaim,
            gate: domain::policy::ReviewGate::CustomerMessageApproval,
        }
    );
}

#[test]
fn core_service_contract_groups_all_petsuites_lines_without_raw_field_flags() {
    let service_contracts = operations::service_core::ServiceContracts::builder()
        .location_id(entities::LocationId(uuid::Uuid::nil()))
        .boarding(boarding::Contract::standard_petsuites())
        .daycare(daycare::Contract::standard_petsuites())
        .grooming(grooming::Contract::standard_petsuites())
        .training(training::Contract::standard_petsuites())
        .retail(retail::Contract::standard_petsuites())
        .build();

    assert_eq!(service_contracts.core_services().len(), 5);
    assert!(service_contracts.boarding.requires_deposit_collection());
    assert!(
        service_contracts
            .daycare
            .requires_staff_review_before_group_play()
    );
    assert!(service_contracts.grooming.requires_deposit_after_no_show());
    assert!(service_contracts.training.requires_named_trainer());
    assert!(service_contracts.retail.should_reorder());
}
